// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

//! Datapoint structures for [knxkit](https://crates.io/crates/knxkit)
//!
//! Create a specific datapoint and convert it to a binary representation
//!```rust
//!  use knxkit_dpt::specific::{SpecificDataPoint, DPT_3_7};   
//!
//!  let value37 = DPT_3_7 {
//!    Increase: true,
//!    StepCode: 1,
//!  };
//!
//!  let datapoint = value37.to_data_point();
//!     
//!  assert_eq!(datapoint, knxkit::core::DataPoint::Short(0x09));
//!```
//!
//! Decode a binary representation to a specific datapoint
//! ```rust
//!  use knxkit_dpt::specific::{SpecificDataPoint, DPT_3_7};   
//!
//!  let datapoint = knxkit::core::DataPoint::Short(0x09);
//!  let value37 = DPT_3_7::from_data_point(&datapoint).unwrap();
//!
//!  assert_eq!(value37, DPT_3_7 {Increase: true, StepCode: 1});
//! ```
//!
//! Decode a binary representation to a generic datapoint and then to JSON string
//!```rust
//!  use knxkit_dpt::specific::{SpecificDataPoint, DPT_3_7};
//!  use knxkit_dpt::generic;
//!
//!  let datapoint = knxkit::core::DataPoint::Short(0x09);
//!  let generic = generic::try_decode(DPT_3_7::DPT, &datapoint).unwrap();
//!  let json = generic.to_json_string();
//!  assert_eq!(json, "{\"Increase\":true,\"StepCode\":1}");
//!```
//! Lookup a datapoint info by DPT
//! ```rust
//!  use knxkit_dpt::specific::{SpecificDataPoint, DPT_3_7};
//!
//!  let info = knxkit_dpt::typeinfo::lookup(DPT_3_7::DPT).unwrap();   
//!
//!  assert_eq!(info.name, "DPT_Control_Dimming");
//!  assert_eq!(info.text, Some("dimming control"));
//!  assert_eq!(info.unit, None);
//! ```

mod error;
mod generated;
/// Type-erased generic datapoint encoding and decoding.
pub mod generic;
/// Project extension trait for DPT-aware value lookups.
pub mod project;
/// Typed (specific) datapoint structures and the `SpecificDataPoint` trait.
pub mod specific;
/// Datapoint type metadata lookup.
pub mod typeinfo;

/// Errors returned by datapoint encoding and decoding operations.
pub use error::Error;

#[cfg(test)]
mod tests {
    use crate::generic::*;
    use crate::specific::*;
    use knxkit::{core::DataPoint, project::DPT};

    #[test]
    fn test_decode_1_1() {
        assert_eq!(
            DPT_1_1::from_data_point(&DataPoint::Short(0x01)).unwrap().0,
            true
        );
        assert_eq!(
            DPT_1_1::from_data_point(&DataPoint::Short(0x00)).unwrap().0,
            false
        );
    }

    #[test]
    fn test_encode_1_1() {
        assert_eq!(DPT_1_1(true).to_data_point(), DataPoint::Short(0x01));
        assert_eq!(DPT_1_1(false).to_data_point(), DataPoint::Short(0x00));
    }

    #[test]
    fn test_encode_json_1_1() {
        assert_eq!(serde_json::to_string(&DPT_1_1(false)).unwrap(), "false");
    }

    #[test]
    fn test_encode_json_7_1() {
        assert_eq!(serde_json::to_string(&DPT_7_1(6)).unwrap(), "6");
    }

    #[test]
    fn test_decode_generic_1_1() {
        let opaque1 = try_decode(DPT::new(1, Some(1)), &DataPoint::Short(0x01)).unwrap();

        let json = serde_json::to_string(&opaque1.to_json_value()).unwrap();
        assert_eq!(json, "true");

        let _opaque2 =
            try_decode_json(DPT::new(1, Some(1)), serde_json::from_str(&json).unwrap()).unwrap();
    }

    // --- try_encode_json ---

    #[test]
    fn test_try_encode_json_bool() {
        let dp = try_encode_json(DPT::new(1, Some(1)), serde_json::json!(true)).unwrap();
        assert_eq!(dp, DataPoint::Short(0x01));

        let dp = try_encode_json(DPT::new(1, Some(1)), serde_json::json!(false)).unwrap();
        assert_eq!(dp, DataPoint::Short(0x00));
    }

    #[test]
    fn test_try_encode_json_unsigned() {
        let dp = try_encode_json(DPT::new(7, Some(1)), serde_json::json!(1000)).unwrap();
        let decoded = DPT_7_1::from_data_point(&dp).unwrap();
        assert_eq!(decoded.0, 1000);
    }

    #[test]
    fn test_try_encode_json_float() {
        let dp = try_encode_json(DPT::new(9, Some(1)), serde_json::json!(21.5)).unwrap();
        let decoded = DPT_9_1::from_data_point(&dp).unwrap();
        assert!((decoded.0 - 21.5).abs() < 0.1);
    }

    // --- try_from_json ---

    #[test]
    fn test_try_from_json_roundtrip() {
        let generic = try_from_json(DPT::new(9, Some(1)), serde_json::json!(19.5)).unwrap();
        assert_eq!(generic.dpt(), DPT::new(9, Some(1)));
        let dp = generic.to_data_point();
        let decoded = try_decode(DPT::new(9, Some(1)), &dp).unwrap();
        assert!((decoded.to_json_value().as_f64().unwrap() - 19.5).abs() < 0.1);
    }

    // --- round-trip encode/decode across DPT families ---

    #[test]
    fn test_roundtrip_dpt_1_1_bool() {
        let original = DPT_1_1(true);
        let dp = original.to_data_point();
        let decoded = DPT_1_1::from_data_point(&dp).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_dpt_5_1_percent() {
        let original = DPT_5_1(75);
        let dp = original.to_data_point();
        let decoded = DPT_5_1::from_data_point(&dp).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_dpt_7_1_counter() {
        let original = DPT_7_1(65535);
        let dp = original.to_data_point();
        let decoded = DPT_7_1::from_data_point(&dp).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_dpt_9_1_temperature() {
        let original = DPT_9_1(-10.5);
        let dp = original.to_data_point();
        let decoded = DPT_9_1::from_data_point(&dp).unwrap();
        assert!((original.0 - decoded.0).abs() < 0.5);
    }

    #[test]
    fn test_roundtrip_dpt_13_1_signed_counter() {
        let original = DPT_13_1(-100_000);
        let dp = original.to_data_point();
        let decoded = DPT_13_1::from_data_point(&dp).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_roundtrip_dpt_14_1_float() {
        let original = DPT_14_1(3.14);
        let dp = original.to_data_point();
        let decoded = DPT_14_1::from_data_point(&dp).unwrap();
        assert!((original.0 - decoded.0).abs() < 0.001);
    }

    // --- generic round-trip via JSON ---

    #[test]
    fn test_generic_json_roundtrip_multiple_dpts() {
        let cases: Vec<(DPT, serde_json::Value)> = vec![
            (DPT::new(1, Some(1)), serde_json::json!(true)),
            (DPT::new(5, Some(1)), serde_json::json!(200)),
            (DPT::new(7, Some(1)), serde_json::json!(50000)),
            (DPT::new(9, Some(1)), serde_json::json!(22.0)),
        ];

        for (dpt, json_val) in cases {
            let encoded = try_encode_json(dpt, json_val.clone()).unwrap();
            let decoded = try_decode(dpt, &encoded).unwrap();
            let json_back = decoded.to_json_value();

            match (&json_val, &json_back) {
                (serde_json::Value::Bool(a), serde_json::Value::Bool(b)) => assert_eq!(a, b),
                (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                    let diff = (a.as_f64().unwrap() - b.as_f64().unwrap()).abs();
                    assert!(diff < 1.0, "DPT {dpt}: {a} vs {b}, diff={diff}");
                }
                _ => panic!("type mismatch for DPT {dpt}"),
            }
        }
    }

    // --- error cases ---

    #[test]
    fn test_try_encode_json_invalid_dpt() {
        let result = try_encode_json(DPT::new(999, Some(999)), serde_json::json!(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_try_decode_wrong_payload() {
        // DPT 9.1 expects Long(2 bytes), not Short
        let result = try_decode(DPT::new(9, Some(1)), &DataPoint::Short(0x01));
        assert!(result.is_err());
    }
}
