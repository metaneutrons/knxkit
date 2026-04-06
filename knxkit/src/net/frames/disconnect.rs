// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use crate::net::frames::{hpai::HPAI, FramePayload, ServiceType};

use crate::core::util::prelude::*;

// 03_08_02-7.8.5
#[derive(Debug, Clone)]
pub struct DisconnectRequest {
    pub channel: u8,
    pub control: HPAI,
}

impl FramePayload for DisconnectRequest {
    const SERVICE_TYPE: ServiceType = ServiceType::DisconnectRequest;

    fn parse(input: Input) -> Result<Self> {
        let (input, (channel, _reserved, control)) =
            ((parse_u8, parse_u8, HPAI::parse)).parse(input)?;

        Ok((input, DisconnectRequest { channel, control }))
    }

    fn gen<W: Write>(&self) -> impl SerializeFn<W> {
        gen_tuple((gen_u8(self.channel), gen_u8(0x00), self.control.gen()))
    }
}

// 03_08_02-7.8.6
#[derive(Debug, Clone)]
pub struct DisconnectResponse {
    pub channel: u8,
    pub status: u8,
}

impl FramePayload for DisconnectResponse {
    const SERVICE_TYPE: ServiceType = ServiceType::DisconnectResponse;

    fn parse(input: Input) -> Result<Self> {
        let (input, (channel, status)) = ((parse_u8, parse_u8)).parse(input)?;
        Ok((input, DisconnectResponse { channel, status }))
    }

    fn gen<W: Write>(&self) -> impl SerializeFn<W> {
        gen_tuple((gen_u8(self.channel), gen_u8(self.status)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::frames::{Frame, FramePayload};
    use std::net::SocketAddr;

    #[test]
    fn disconnect_request_round_trip() {
        let addr: SocketAddr = "192.168.1.10:3671".parse().unwrap();
        let original = DisconnectRequest {
            channel: 0x15,
            control: HPAI::new_udp(addr),
        };

        let frame = Frame::from(original);
        let bytes: Vec<u8> = Vec::try_from(frame).unwrap();
        let frame = Frame::try_from(bytes.as_slice()).unwrap();
        let parsed = DisconnectRequest::try_parse(frame).unwrap();

        assert_eq!(parsed.channel, 0x15);
        assert_eq!(parsed.control, HPAI::new_udp(addr));
    }

    #[test]
    fn disconnect_response_round_trip() {
        let original = DisconnectResponse {
            channel: 0x21,
            status: 0x00,
        };

        let frame = Frame::from(original);
        let bytes: Vec<u8> = Vec::try_from(frame).unwrap();
        let frame = Frame::try_from(bytes.as_slice()).unwrap();
        let parsed = DisconnectResponse::try_parse(frame).unwrap();

        assert_eq!(parsed.channel, 0x21);
        assert_eq!(parsed.status, 0x00);
    }
}
