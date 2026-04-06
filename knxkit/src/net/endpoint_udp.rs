// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio::net::UdpSocket;
use tracing::error;

use crate::{
    error::Error,
    net::frames::{hpai::HPAI, Frame},
};

/// UDP socket wrapper for KNX/IP frame exchange.
pub struct UdpEndpoint {
    local: UdpSocket,
    hpai: HPAI,
    peer_control: SocketAddrV4,
    peer_data: Option<SocketAddrV4>,
}

impl UdpEndpoint {
    /// Bind a new UDP endpoint to a local address for communicating with a peer.
    pub async fn bind(
        local: Ipv4Addr,
        peer_control: SocketAddrV4,
        nat: bool,
    ) -> Result<Self, Error> {
        let local = UdpSocket::bind(SocketAddrV4::new(local, 0)).await?;

        let hpai = if nat {
            HPAI::UNSPECIFIED_UDP
        } else {
            HPAI::new_udp(local.local_addr()?)
        };

        Ok(Self {
            local,
            hpai,
            peer_control,
            peer_data: None,
        })
    }

    /// Set the peer address used for data channel communication.
    pub fn set_data_peer(&mut self, data: SocketAddrV4) {
        self.peer_data = Some(data)
    }

    /// Return the HPAI (host protocol address information) for this endpoint.
    pub fn hpai(&self) -> HPAI {
        self.hpai
    }

    /// Send a frame to the data peer (or control peer if no data peer is set).
    pub async fn send_data(&self, frame: Frame) -> Result<usize, Error> {
        //debug!(?frame, ?self.peer_data, "send data");

        let bytes: Vec<u8> = frame.try_into()?;

        self.local
            .send_to(&bytes, self.peer_data.unwrap_or(self.peer_control))
            .await
            .map_err(Into::into)
    }

    /// Send a frame to the control peer.
    pub async fn send_control(&self, frame: Frame) -> Result<usize, Error> {
        let bytes: Vec<u8> = frame.try_into()?;

        self.local
            .send_to(&bytes, self.peer_control)
            .await
            .map_err(Into::into)
    }

    /// Receive the next valid frame and its source address.
    pub async fn recv(&self) -> Result<(Frame, SocketAddr), std::io::Error> {
        loop {
            let mut buffer = [0_u8; 512];

            let (size, from) = self.local.recv_from(&mut buffer).await?;

            match Frame::try_from(&buffer[..size]) {
                Ok(frame) => {
                    break Ok((frame, from));
                }
                Err(_) => {
                    error!("cannot decode ingress frame")
                }
            };
        }
    }

    /// Return the standard KNX/IP multicast address (`224.0.23.12:3671`).
    pub fn multicast_peer() -> SocketAddrV4 {
        "224.0.23.12:3671".parse().unwrap()
    }
}
