// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use std::net::Ipv4Addr;

use super::DescriptionType;
use crate::core::util::prelude::*;

// 3/8/2-7.5.4.4
/// IP configuration DIB (3/8/2-7.5.4.4).
#[derive(Debug, Clone)]
pub struct IpConfig {
    /// Configured IP address.
    pub address: Ipv4Addr,
    /// Subnet mask.
    pub netmask: u32,
    /// Default gateway.
    pub gateway: Ipv4Addr,
    /// IP assignment capabilities bitmask.
    pub capabilities: u8,
    /// Current IP assignment method.
    pub assignment_method: u8,
}

impl IpConfig {
    /// Parses an IP configuration DIB from wire format.
    pub fn parse(input: Input) -> Result<Self> {
        let (input, _length) = parse_u8(input)?;
        let (input, _) = parse_token(DescriptionType::IpConfig.to_u8().unwrap())(input)?;

        let (input, address) = parse_u32(input).map_value(|a| Ipv4Addr::from_bits(a))?;
        let (input, netmask) = parse_u32(input)?;
        let (input, gateway) = parse_u32(input).map_value(|a| Ipv4Addr::from_bits(a))?;

        let (input, capabilities) = parse_u8(input)?;
        let (input, assignment_method) = parse_u8(input)?;

        Ok((
            input,
            IpConfig {
                address,
                netmask,
                gateway,
                capabilities,
                assignment_method,
            },
        ))
    }
}
