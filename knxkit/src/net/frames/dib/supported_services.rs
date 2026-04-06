// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use crate::core::util::prelude::*;

// 3/8/2-7.5.4.3
/// KNXnet/IP service family identifier (3/8/2-7.5.4.3).
#[derive(PartialEq, Debug, Clone, FromPrimitive, ToPrimitive)]
pub enum ServiceFamily {
    /// Core services.
    Core = 0x02,
    /// Device management.
    DeviceManagement = 0x03,
    /// Tunneling.
    Tunneling = 0x04,
    /// Routing.
    Routing = 0x05,
    /// Remote diagnostics and configuration.
    RemoteDiagAndConfig = 0x07,
}

// 3/8/2-7.5.4.3
/// Supported service family entry (3/8/2-7.5.4.3).
#[derive(PartialEq, Debug, Clone)]
pub struct ServiceEntry {
    family: ServiceFamily,
    version: u8,
}

impl ServiceEntry {
    /// Parses a single service entry from wire format.
    pub fn parse(input: Input) -> Result<Self> {
        let (input, (family, version)) = (parse_enum(8), parse_u8).parse(input)?;

        Ok((input, ServiceEntry { family, version }))
    }

    /// Parses a supported service families DIB from wire format.
    pub fn parse_many(input0: Input) -> Result<Vec<Self>> {
        use super::DescriptionType;

        let (input, (length, type_)) = ((parse_u8, parse_u8)).parse(input0)?;

        if type_ != DescriptionType::SuppSvcFamilies.to_u8().unwrap() {
            return Err(nom::Err::Failure(Error::general("Unexpected DIB", input0)));
        }

        nom::multi::count(ServiceEntry::parse, ((length - 2) / 2) as usize).parse(input)
    }
}
