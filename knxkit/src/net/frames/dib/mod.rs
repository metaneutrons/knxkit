// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use num_derive::{FromPrimitive, ToPrimitive};

use crate::core::util::prelude::*;

/// Device information DIB.
pub mod device_information;
/// IP configuration DIB.
pub mod ip_config;
/// Current IP configuration DIB.
pub mod ip_current_config;
/// KNX address DIB.
pub mod knx_addresses;
/// Manufacturer-specific data DIB.
pub mod manufacturer_data;
/// Supported service families DIB.
pub mod supported_services;

// 7.5.4
/// DIB description type code (7.5.4).
#[derive(Debug, Clone, PartialEq, FromPrimitive, ToPrimitive)]
pub enum DescriptionType {
    /// Device information.
    DeviceInfo = 0x01,
    /// Supported service families.
    SuppSvcFamilies = 0x02,
    /// IP configuration.
    IpConfig = 0x03,
    /// Current IP configuration.
    IpCurConfig = 0x04,
    /// KNX addresses.
    KNXAddresses = 0x05,
    /// Manufacturer data.
    MFRData = 0xfe,
}

// 7.5.4
/// DIB header (7.5.4).
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// Total length of this DIB including the header.
    pub length: u8,
    /// Description type code.
    pub type_: DescriptionType,
}

impl Header {
    /// Parses a DIB header from wire format.
    pub fn parse(input: Input) -> Result<Self> {
        let (input, length) = parse_u8(input)?;
        let (input, type_) = parse_enum(8)(input)?;

        Ok((input, Header { length, type_ }))
    }
}
