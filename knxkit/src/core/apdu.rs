// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use num_traits::FromPrimitive;

use super::DataPoint;
use crate::core::util::prelude::*;

/// KNX application-layer service type.
#[derive(Clone, Copy, Debug, num_derive::ToPrimitive, num_derive::FromPrimitive, PartialEq)]
pub enum Service {
    /// Read a group value (multicast).
    GroupValueRead = 0x000,
    /// Group value response with data.
    GroupValueResponse = 0x040,
    /// Group value write with data.
    GroupValueWrite = 0x080,

    /// Write an individual address (broadcast).
    IndividualAddressWrite = 0x0c0,
    /// Read individual address of devices in programming mode.
    IndividualAddressRead = 0x100,
    /// Response to individual address read.
    IndividualAddressResponse = 0x140,

    /// Read individual address by serial number.
    IndividualAddressSerialNumberRead = 0x3dc,
    /// Response to serial number address read.
    IndividualAddressSerialNumberResponse = 0x3dd,
    /// Write individual address by serial number.
    IndividualAddressSerialNumberWrite = 0x3de,
    // network parameter...

    // unicast connectionless
    /// Read device descriptor.
    DeviceDescriptorRead = 0x300,
    /// Response to device descriptor read.
    DeviceDescriptorResponse = 0x340,
    /// Restart the device.
    Restart = 0x380,

    /// Read an interface object property value.
    PropertyValueRead = 0x3d5,
    /// Response to property value read.
    PropertyValueResponse = 0x3d6,
    /// Write an interface object property value.
    PropertyValueWrite = 0x3d7,

    /// Read a property description.
    PropertyDescriptionRead = 0x3d8,
    /// Response to property description read.
    PropertyDescriptionResponse = 0x3d9,
    // link...

    // unicast connected
    /// Read device memory.
    MemoryRead = 0x200,
    /// Response to memory read.
    MemoryResponse = 0x240,
    /// Write device memory.
    MemoryWrite = 0x280,

    /// Read user memory.
    UserMemoryRead = 0x2c0,
    /// Response to user memory read.
    UserMemoryResponse = 0x2c1,
    /// Write user memory.
    UserMemoryWrite = 0x2c2,

    /// Read user manufacturer info.
    UserManufacturerInfoRead = 0x2c5,
    /// Response to user manufacturer info read.
    UserManufacturerInfoResponse = 0x2c6,

    /// Authorization request.
    AuthorizeRequest = 0x3d1,
    /// Authorization response.
    AuthorizeResponse = 0x3d2,

    /// Write access key.
    KeyWrite = 0x3d3,
    /// Response to key write.
    KeyResponse = 0x3d4,

    /// Read ADC value.
    ADCRead = 0x9999,
    /// Response to ADC read.
    ADCResponse = 0x9998,
}

/// Application Protocol Data Unit carrying a service and optional data.
#[derive(Debug, Clone)]
pub struct APDU {
    /// The application-layer service type.
    pub service: Service,
    /// Optional datapoint payload.
    pub data: Option<DataPoint>,
}

impl APDU {
    /// Parses an APDU from binary input given the TPCI prefix and NPDU length.
    pub fn parse(prefix: u8, input: Input, npdu_length: u8) -> Result<Self> {
        let (input, octet7) = parse_u8(input)?;

        let apci = (prefix as u16) << 8 | (octet7 as u16);

        let with_data = |service, data| APDU {
            service,
            data: Some(if npdu_length == 1 {
                DataPoint::Short(data)
            } else {
                DataPoint::Long(input.into())
            }),
        };

        let apdu = match (apci >> 6, (apci & 0b00111111) as u8) {
            (0b0001, data) => with_data(Service::GroupValueResponse, data),
            (0b0010, data) => with_data(Service::GroupValueWrite, data),

            (0b0110, data) => with_data(Service::ADCRead, data),
            (0b0111, data) => with_data(Service::ADCResponse, data),

            (0b1000, data) => with_data(Service::MemoryRead, data),
            (0b1001, data) => with_data(Service::MemoryResponse, data),
            (0b1010, data) => with_data(Service::MemoryWrite, data),
            _ => {
                let service = Service::from_u16(apci).ok_or(Err::Failure(Error::general(
                    "Unexpected APCI",
                    &[prefix, octet7],
                )))?;

                APDU {
                    service,
                    data: None,
                }
            }
        };

        Ok((&[], apdu))
    }

    pub(crate) fn suffix(&self) -> u8 {
        (self.service.to_u16().unwrap() >> 8) as u8
    }

    /// Serializes this APDU to binary.
    pub fn gen<W: Write>(&self) -> impl SerializeFn<W> + use<'_, W> {
        let code = (self.service.to_u16().unwrap() & 0xff) as u8;

        move |context| {
            match &self.data {
                Some(DataPoint::Short(s)) => {
                    //
                    gen_u8((code & 0b11000000) | (s & 0b00111111))(context)
                }

                Some(DataPoint::Long(l)) => {
                    //
                    gen_tuple((gen_u8(code & 0b11000000), gen_slice(l)))(context)
                }

                None => gen_u8(code)(context),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apdu_round_trip(original: &APDU) -> APDU {
        let mut bytes = Vec::new();
        cookie_factory::gen(original.gen(), &mut bytes).unwrap();
        let prefix = original.suffix();
        let npdu_length = bytes.len() as u8;
        let (_, parsed) = APDU::parse(prefix, &bytes, npdu_length).unwrap();
        parsed
    }

    #[test]
    fn apdu_group_value_write_short_round_trip() {
        let original = APDU {
            service: Service::GroupValueWrite,
            data: Some(DataPoint::Short(0x01)),
        };
        let parsed = apdu_round_trip(&original);
        assert_eq!(parsed.service, Service::GroupValueWrite);
        assert_eq!(parsed.data, Some(DataPoint::Short(0x01)));
    }

    #[test]
    fn apdu_group_value_write_long_round_trip() {
        let original = APDU {
            service: Service::GroupValueWrite,
            data: Some(DataPoint::Long(vec![0xAB, 0xCD])),
        };
        let parsed = apdu_round_trip(&original);
        assert_eq!(parsed.service, Service::GroupValueWrite);
        assert_eq!(parsed.data, Some(DataPoint::Long(vec![0xAB, 0xCD])));
    }

    #[test]
    fn apdu_group_value_response_short_round_trip() {
        let original = APDU {
            service: Service::GroupValueResponse,
            data: Some(DataPoint::Short(0x00)),
        };
        let parsed = apdu_round_trip(&original);
        assert_eq!(parsed.service, Service::GroupValueResponse);
        assert_eq!(parsed.data, Some(DataPoint::Short(0x00)));
    }

    #[test]
    fn apdu_group_value_read_round_trip() {
        let original = APDU {
            service: Service::GroupValueRead,
            data: None,
        };
        let parsed = apdu_round_trip(&original);
        assert_eq!(parsed.service, Service::GroupValueRead);
        assert_eq!(parsed.data, None);
    }
}
