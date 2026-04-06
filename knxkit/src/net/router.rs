// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use tokio::net::UdpSocket;
use tokio::sync::Notify;
use tracing::{debug, warn};

use crate::{
    core::{address::IndividualAddress, cemi::CEMI},
    error::Error,
    net::frames::{Frame, ServiceType},
};

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 23, 12);
const MULTICAST_PORT: u16 = 3671;

/// KNX/IP routing connection over multicast UDP.
pub struct RouterConnection {
    socket: UdpSocket,
    multicast: SocketAddrV4,
}

impl RouterConnection {
    /// Join the KNX/IP multicast group and return a new router connection.
    pub async fn start(local: Ipv4Addr, multicast: SocketAddrV4) -> Result<Self, Error> {
        let bind_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, multicast.port());
        let socket = UdpSocket::bind(bind_addr).await?;

        socket.set_multicast_loop_v4(false)?;
        socket.join_multicast_v4(*multicast.ip(), local)?;

        debug!(?local, ?multicast, "joined multicast group");

        Ok(Self { socket, multicast })
    }

    /// Join the default KNX/IP multicast group (224.0.23.12:3671).
    pub async fn start_default(local: Ipv4Addr) -> Result<Self, Error> {
        Self::start(local, SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT)).await
    }
}

impl crate::connection::KnxBusConnection for RouterConnection {
    async fn send(&self, cemi: CEMI) -> Result<Arc<Notify>, Error> {
        debug!(?cemi, "routing egress cemi");

        let cemi_bytes: Vec<u8> = cemi.try_into()?;
        let frame = Frame {
            service_type: ServiceType::RoutingIndication,
            payload: cemi_bytes,
        };
        let bytes: Vec<u8> = frame.try_into()?;

        self.socket.send_to(&bytes, self.multicast).await?;

        let notify = Arc::new(Notify::new());
        notify.notify_one();
        Ok(notify)
    }

    async fn recv(&mut self) -> Option<Arc<CEMI>> {
        loop {
            let mut buf = [0u8; 512];
            let (len, _from) = match self.socket.recv_from(&mut buf).await {
                Ok(r) => r,
                Err(e) => {
                    warn!("router recv error: {e}");
                    return None;
                }
            };

            let frame = match Frame::try_from(&buf[..len]) {
                Ok(f) => f,
                Err(_) => continue,
            };

            if frame.service_type != ServiceType::RoutingIndication {
                continue;
            }

            match CEMI::try_from(frame.payload.as_slice()) {
                Ok(cemi) => {
                    debug!(?cemi, "routing ingress cemi");
                    return Some(Arc::new(cemi));
                }
                Err(e) => {
                    warn!("cannot decode routing cEMI: {e:?}");
                }
            }
        }
    }

    async fn terminate(self) {
        let _ = self
            .socket
            .leave_multicast_v4(*self.multicast.ip(), Ipv4Addr::UNSPECIFIED);
        debug!("left multicast group");
    }

    fn address(&self) -> IndividualAddress {
        IndividualAddress::new_zero()
    }
}
