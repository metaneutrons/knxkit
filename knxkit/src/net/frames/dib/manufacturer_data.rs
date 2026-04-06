// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use super::DescriptionType;
use crate::core::util::prelude::*;

// 3/8/2-7.5.4.7
/// Manufacturer data DIB (3/8/2-7.5.4.7).
#[derive(Clone, Debug)]
pub struct ManufacturerData {
    /// KNX manufacturer ID.
    pub manufacturer_id: u16,
    /// Manufacturer-specific data bytes.
    pub manufacturer_specific: Vec<u8>,
}

impl ManufacturerData {
    /// Parses a manufacturer data DIB from wire format.
    pub fn parse(input: Input) -> Result<Self> {
        let (input, length) = parse_u8(input)?;
        let (input, _) = parse_token(DescriptionType::MFRData.to_u8().unwrap())(input)?;

        let (input, manufacturer_id) = parse_u16(input)?;
        let (input, manufacturer_specific) = parse_take(length - 4)(input).map_value(Into::into)?;

        Ok((
            input,
            ManufacturerData {
                manufacturer_id,
                manufacturer_specific,
            },
        ))
    }
}
