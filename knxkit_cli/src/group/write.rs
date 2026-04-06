// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use std::str::FromStr;

use anyhow::Result;

use knxkit::{
    connection::{ops::GroupOps, KnxBusConnection},
    core::DataPoint,
    project::{ProjectExt, DPT},
};

use knxkit_dpt::generic;

use super::{GroupCommand, ValueFormat};
use crate::{cli::CLI, util::connect};

/// Parse a human-friendly value string into a JSON value suitable for the given DPT.
///
/// - DPT 1.x (boolean): on/off, true/false, yes/no, 1/0
/// - DPT 2.x (control): `{"control":true,"value":true}` or JSON
/// - DPT 3.x (dimming): `{"control":true,"stepcode":5}` or JSON
/// - DPT 5–14.x (numeric): plain number
/// - Anything else: try JSON, fall back to JSON string
fn parse_value(dpt: DPT, input: &str) -> serde_json::Value {
    let lower = input.trim().to_lowercase();

    match dpt.main {
        // DPT 1.x — boolean
        1 => match lower.as_str() {
            "on" | "true" | "yes" | "1" => serde_json::Value::Bool(true),
            "off" | "false" | "no" | "0" => serde_json::Value::Bool(false),
            _ => json_or_string(input),
        },

        // DPT 5–8, 12–13 — integer types; DPT 9, 14 — float types
        5 | 6 | 7 | 8 | 9 | 12 | 13 | 14 => {
            if let Ok(n) = input.trim().parse::<f64>() {
                serde_json::Number::from_f64(n)
                    .map(serde_json::Value::Number)
                    .unwrap_or_else(|| json_or_string(input))
            } else {
                json_or_string(input)
            }
        }

        // Everything else (composite, string, etc.) — try JSON first
        _ => json_or_string(input),
    }
}

fn json_or_string(input: &str) -> serde_json::Value {
    serde_json::from_str(input).unwrap_or_else(|_| serde_json::Value::String(input.to_string()))
}

pub async fn command(command: &GroupCommand) -> Result<()> {
    crate::match_variant!(GroupCommand::Write {
        remote,
        group,
        value,
        format,
    } = command => {
        let mut connection = connect(&remote.remote).await.unwrap();

        let data = match format {
            ValueFormat::Raw => DataPoint::from_str(value)?,
            ValueFormat::Value => {
                let project = CLI.globals.project.as_ref();
                let dpt = project
                    .group_dpt(*group)
                    .ok_or_else(|| anyhow::anyhow!("no DPT for group {group} — is --project set?"))?;
                let json = parse_value(dpt, value);
                generic::try_encode_json(dpt, json)?
            }
        };

        let notify = connection.group_write(*group, data).await?;

        notify.notified().await;

        connection.terminate().await;

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use knxkit::project::DPT;

    fn dpt(main: u16) -> DPT {
        DPT::new(main, Some(1))
    }

    // --- DPT 1.x boolean parsing ---

    #[test]
    fn test_parse_value_bool_on_off() {
        assert_eq!(parse_value(dpt(1), "on"), serde_json::json!(true));
        assert_eq!(parse_value(dpt(1), "off"), serde_json::json!(false));
    }

    #[test]
    fn test_parse_value_bool_true_false() {
        assert_eq!(parse_value(dpt(1), "true"), serde_json::json!(true));
        assert_eq!(parse_value(dpt(1), "false"), serde_json::json!(false));
    }

    #[test]
    fn test_parse_value_bool_yes_no() {
        assert_eq!(parse_value(dpt(1), "yes"), serde_json::json!(true));
        assert_eq!(parse_value(dpt(1), "no"), serde_json::json!(false));
    }

    #[test]
    fn test_parse_value_bool_1_0() {
        assert_eq!(parse_value(dpt(1), "1"), serde_json::json!(true));
        assert_eq!(parse_value(dpt(1), "0"), serde_json::json!(false));
    }

    #[test]
    fn test_parse_value_bool_case_insensitive() {
        assert_eq!(parse_value(dpt(1), "ON"), serde_json::json!(true));
        assert_eq!(parse_value(dpt(1), "Off"), serde_json::json!(false));
        assert_eq!(parse_value(dpt(1), "TRUE"), serde_json::json!(true));
    }

    #[test]
    fn test_parse_value_bool_unknown_falls_back() {
        // Unknown string for bool DPT → tries JSON, falls back to string
        let v = parse_value(dpt(1), "maybe");
        assert_eq!(v, serde_json::json!("maybe"));
    }

    // --- Numeric DPTs ---

    #[test]
    fn test_parse_value_integer() {
        assert_eq!(parse_value(dpt(5), "200"), serde_json::json!(200.0));
        assert_eq!(parse_value(dpt(7), "65535"), serde_json::json!(65535.0));
    }

    #[test]
    fn test_parse_value_float() {
        assert_eq!(parse_value(dpt(9), "21.5"), serde_json::json!(21.5));
        assert_eq!(parse_value(dpt(14), "-3.14"), serde_json::json!(-3.14));
    }

    #[test]
    fn test_parse_value_negative_integer() {
        assert_eq!(parse_value(dpt(8), "-100"), serde_json::json!(-100.0));
        assert_eq!(parse_value(dpt(13), "-50000"), serde_json::json!(-50000.0));
    }

    #[test]
    fn test_parse_value_numeric_non_number_falls_back() {
        let v = parse_value(dpt(9), "warm");
        assert_eq!(v, serde_json::json!("warm"));
    }

    // --- Composite / JSON fallback ---

    #[test]
    fn test_parse_value_json_object() {
        let v = parse_value(dpt(2), r#"{"control":true,"value":false}"#);
        assert_eq!(v, serde_json::json!({"control": true, "value": false}));
    }

    #[test]
    fn test_parse_value_json_array() {
        let v = parse_value(dpt(232), "[255,128,0]");
        assert_eq!(v, serde_json::json!([255, 128, 0]));
    }

    #[test]
    fn test_parse_value_plain_string_fallback() {
        let v = parse_value(dpt(16), "Hello KNX");
        assert_eq!(v, serde_json::json!("Hello KNX"));
    }

    // --- Whitespace handling ---

    #[test]
    fn test_parse_value_trims_whitespace() {
        assert_eq!(parse_value(dpt(1), "  on  "), serde_json::json!(true));
        assert_eq!(parse_value(dpt(9), "  21.5  "), serde_json::json!(21.5));
    }
}
