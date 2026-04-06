// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, Notify};

use super::KnxBusConnection;
use crate::core::{address::IndividualAddress, cemi::CEMI};
use crate::error::Error;

/// A multiplexer that wraps a `KnxBusConnection` and fans out received frames
/// to multiple handles via a broadcast channel.
pub struct Multiplexer<T: KnxBusConnection> {
    connection: T,
    broadcast_tx: broadcast::Sender<Arc<CEMI>>,
    send_rx: mpsc::Receiver<CEMI>,
    send_tx: mpsc::Sender<CEMI>,
}

impl<T: KnxBusConnection> Multiplexer<T> {
    /// Create a new multiplexer wrapping the given connection.
    pub fn new(connection: T) -> Self {
        let (broadcast_tx, _) = broadcast::channel(64);
        let (send_tx, send_rx) = mpsc::channel(64);
        Self {
            connection,
            broadcast_tx,
            send_rx,
            send_tx,
        }
    }

    /// Create a new handle that implements `KnxBusConnection`.
    pub fn handle(&self) -> MultiplexHandle {
        MultiplexHandle {
            broadcast_rx: self.broadcast_tx.subscribe(),
            send_tx: self.send_tx.clone(),
            address: self.connection.address(),
        }
    }

    /// Run the multiplexer loop. Forwards received frames to all handles
    /// and send requests from handles to the underlying connection.
    /// Returns when the underlying connection closes.
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                frame = self.connection.recv() => {
                    match frame {
                        Some(cemi) => { let _ = self.broadcast_tx.send(cemi); }
                        None => break,
                    }
                }
                Some(cemi) = self.send_rx.recv() => {
                    let _ = self.connection.send(cemi).await;
                }
            }
        }
        self.connection.terminate().await;
    }
}

/// A handle to a multiplexed connection. Each handle receives all frames
/// from the underlying connection via broadcast.
pub struct MultiplexHandle {
    broadcast_rx: broadcast::Receiver<Arc<CEMI>>,
    send_tx: mpsc::Sender<CEMI>,
    address: IndividualAddress,
}

impl KnxBusConnection for MultiplexHandle {
    async fn send(&self, cemi: CEMI) -> Result<Arc<Notify>, Error> {
        self.send_tx
            .send(cemi)
            .await
            .map_err(|_| Error::General("multiplexer closed".into()))?;
        let notify = Arc::new(Notify::new());
        notify.notify_one();
        Ok(notify)
    }

    async fn recv(&mut self) -> Option<Arc<CEMI>> {
        loop {
            match self.broadcast_rx.recv().await {
                Ok(cemi) => return Some(cemi),
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }

    async fn terminate(self) {}

    fn address(&self) -> IndividualAddress {
        self.address
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        address::{DestinationAddress, GroupAddress},
        apdu::{Service, APDU},
        cemi::{CEMIFlags, Priority},
        npdu::NPDU,
        tpdu::TPDU,
        DataPoint,
    };
    use tokio::sync::mpsc;

    fn test_cemi() -> CEMI {
        CEMI {
            mc: 0x29,
            flags: CEMIFlags::empty(),
            hops: 6,
            prio: Priority::Normal,
            source: IndividualAddress::new(0),
            destination: DestinationAddress::Group(GroupAddress::new(1)),
            npdu: NPDU {
                tpdu: TPDU::DataGroup(APDU {
                    service: Service::GroupValueWrite,
                    data: Some(DataPoint::Short(1)),
                }),
            },
        }
    }

    /// Mock connection backed by mpsc channels.
    struct MockConnection {
        recv_rx: mpsc::Receiver<Arc<CEMI>>,
        send_tx: mpsc::Sender<CEMI>,
        address: IndividualAddress,
    }

    impl KnxBusConnection for MockConnection {
        async fn send(&self, cemi: CEMI) -> Result<Arc<Notify>, Error> {
            self.send_tx
                .send(cemi)
                .await
                .map_err(|_| Error::General("mock closed".into()))?;
            let n = Arc::new(Notify::new());
            n.notify_one();
            Ok(n)
        }

        async fn recv(&mut self) -> Option<Arc<CEMI>> {
            self.recv_rx.recv().await
        }

        async fn terminate(self) {}

        fn address(&self) -> IndividualAddress {
            self.address
        }
    }

    fn mock_connection() -> (
        MockConnection,
        mpsc::Sender<Arc<CEMI>>, // push frames into recv
        mpsc::Receiver<CEMI>,    // read frames from send
    ) {
        let (recv_tx, recv_rx) = mpsc::channel(16);
        let (send_tx, send_rx) = mpsc::channel(16);
        let conn = MockConnection {
            recv_rx,
            send_tx,
            address: IndividualAddress::new(1),
        };
        (conn, recv_tx, send_rx)
    }

    #[tokio::test]
    async fn single_handle_receives_frame() {
        let (conn, feeder, _send_rx) = mock_connection();
        let mux = Multiplexer::new(conn);
        let mut handle = mux.handle();

        tokio::spawn(mux.run());

        feeder.send(Arc::new(test_cemi())).await.unwrap();
        let received = handle.recv().await.unwrap();
        assert_eq!(received.mc, 0x29);
    }

    #[tokio::test]
    async fn multiple_handles_receive_same_frame() {
        let (conn, feeder, _send_rx) = mock_connection();
        let mux = Multiplexer::new(conn);
        let mut h1 = mux.handle();
        let mut h2 = mux.handle();

        tokio::spawn(mux.run());

        feeder.send(Arc::new(test_cemi())).await.unwrap();

        let r1 = h1.recv().await.unwrap();
        let r2 = h2.recv().await.unwrap();
        assert_eq!(r1.mc, 0x29);
        assert_eq!(r2.mc, 0x29);
    }

    #[tokio::test]
    async fn send_through_handle_reaches_underlying() {
        let (conn, _feeder, mut send_rx) = mock_connection();
        let mux = Multiplexer::new(conn);
        let handle = mux.handle();

        tokio::spawn(mux.run());

        handle.send(test_cemi()).await.unwrap();
        let received = send_rx.recv().await.unwrap();
        assert_eq!(received.mc, 0x29);
    }

    #[tokio::test]
    async fn handle_drop_does_not_kill_multiplexer() {
        let (conn, feeder, _send_rx) = mock_connection();
        let mux = Multiplexer::new(conn);
        let h1 = mux.handle();
        let mut h2 = mux.handle();

        tokio::spawn(mux.run());

        drop(h1);

        feeder.send(Arc::new(test_cemi())).await.unwrap();
        let received = h2.recv().await.unwrap();
        assert_eq!(received.mc, 0x29);
    }

    #[tokio::test]
    async fn multiplexer_terminate_shuts_down() {
        let (conn, feeder, _send_rx) = mock_connection();
        let mux = Multiplexer::new(conn);
        let mut handle = mux.handle();

        tokio::spawn(mux.run());

        // Closing the feeder causes the mock's recv to return None,
        // which terminates the multiplexer's run loop.
        drop(feeder);

        let result = handle.recv().await;
        assert!(result.is_none());
    }
}
