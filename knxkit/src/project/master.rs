// Copyright (c) 2024 Alexey Aristov <aav@acm.org> and others
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at http://
// www.eclipse.org/legal/epl-2.0, or the GNU General Public License, version 3
// which is available at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// SPDX-License-Identifier: EPL-2.0 OR GPL-3.0

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use interner::shared::StringPool;
use roxmltree::{Document, Node};

use super::SharedString;
use crate::project::{
    dpt::DPT,
    error::Error,
    util::{by_name, NodeExt},
};

/// A KNX datapoint main type definition from the master data.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DatapointType {
    /// Unique identifier from the KNX master XML.
    pub id: SharedString,
    /// Main DPT number (sub is always `None`).
    pub dpt: DPT,

    /// Human-readable name (e.g., `DPT_Switch`).
    pub name: SharedString,
    /// Optional descriptive text.
    pub text: Option<SharedString>,

    /// Size of the datapoint in bits.
    pub size: u16,

    /// Sub types belonging to this main type.
    pub subtypes: Vec<Arc<DatapointSubtype>>,
}

/// A KNX datapoint sub type definition describing a specific encoding.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DatapointSubtype {
    /// Unique identifier from the KNX master XML.
    pub id: SharedString,
    /// Full DPT number (main + sub).
    pub dpt: DPT,
    /// Size of the datapoint in bits.
    pub size: u16,

    /// Human-readable name (e.g., `DPT_Switch`).
    pub name: SharedString,
    /// Optional descriptive text.
    pub text: Option<SharedString>,

    /// Ordered list of format fields that make up the encoding.
    pub formats: Vec<Format>,
    /// Optional default value as a string.
    pub default: Option<SharedString>,
}

/// A single value in an enumeration format field.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EnumerationValue {
    /// Unique identifier from the KNX master XML.
    pub id: SharedString,
    /// Numeric value of this enum entry.
    pub value: u8,
    /// Human-readable label.
    pub text: SharedString,
}

/// Describes the encoding layout of a single field within a datapoint.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Format {
    /// Single-bit boolean field.
    Bit {
        /// Optional field name.
        name: Option<SharedString>,
        /// Label when the bit is 0.
        cleared: SharedString,
        /// Label when the bit is 1.
        set: SharedString,
    },

    /// Signed or unsigned integer field.
    Integer {
        /// Optional field name.
        name: Option<SharedString>,
        /// Bit width of the integer.
        width: u8,
        /// Whether the integer is signed.
        signed: bool,
        /// Minimum allowed value (inclusive).
        min_inclusive: Option<i64>,
        /// Maximum allowed value (inclusive).
        max_inclusive: Option<i64>,
        /// Scaling coefficient applied to the raw value.
        coefficient: Option<f64>,
        /// Physical unit string (e.g., `°C`).
        unit: Option<SharedString>,
    },

    /// Floating-point field.
    Float {
        /// Optional field name.
        name: Option<SharedString>,
        /// Bit width of the float.
        width: u8,
        /// Minimum representable value.
        min_value: Option<f64>,
        /// Maximum representable value.
        max_value: Option<f64>,
        /// Physical unit string.
        unit: Option<SharedString>,
    },

    /// Character string field.
    String {
        /// Optional field name.
        name: Option<SharedString>,
        /// Character encoding (e.g., `us-ascii`).
        encoding: SharedString,
        /// Width in characters.
        width: u16,
        /// Whether the string has variable length.
        variable_length: bool,
        /// Whether the string is null-terminated.
        null_terminated: bool,
    },

    /// Enumeration field mapping integer values to labels.
    Enumeration {
        /// Optional field name.
        name: Option<SharedString>,
        /// Bit width of the enumeration.
        width: u8,
        /// Possible enumeration values.
        values: Vec<EnumerationValue>,
    },

    /// Reserved (padding) bits.
    Reserved {
        /// Bit width of the reserved field.
        width: u8,
    },

    /// Reference to another format definition by id.
    Reference(SharedString),
}

impl Format {
    /// Returns the physical unit string, if this format carries one.
    pub fn unit(&self) -> Option<SharedString> {
        match self {
            Format::Integer { unit, .. } => unit.clone(),
            Format::Float { unit, .. } => unit.clone(),
            _ => None,
        }
    }
}

fn parse_format(f: Node, symbols: &StringPool) -> Result<(Option<SharedString>, Format), Error> {
    let ftype = f.tag_name().name();

    fn id(f: Node, symbols: &StringPool) -> Result<SharedString, Error> {
        Ok(f.intern(symbols, "Id")?)
    }

    match ftype {
        "Bit" => Ok((
            Some(id(f, symbols)?),
            Format::Bit {
                name: f.intern_opt(symbols, "Name"),

                cleared: f.intern(symbols, "Cleared")?,

                set: f.intern(symbols, "Set")?,
            },
        )),

        "UnsignedInteger" => Ok((
            Some(id(f, symbols)?),
            Format::Integer {
                name: f.intern_opt(symbols, "Name"),
                width: f.att("Width")?,
                signed: false,
                min_inclusive: f.att_opt("MinInclusive")?,
                max_inclusive: f.att_opt("MaxInclusive")?,
                coefficient: f.att_opt("Coefficient")?,
                unit: f.intern_opt(symbols, "Unit"),
            },
        )),

        "SignedInteger" => Ok((
            Some(id(f, symbols)?),
            Format::Integer {
                name: f.intern_opt(symbols, "Name"),
                width: f.att("Width")?,
                signed: true,
                min_inclusive: f.att_opt("MinInclusive")?,
                max_inclusive: f.att_opt("MaxInclusive")?,
                coefficient: f.att_opt("Coefficient")?,
                unit: f.intern_opt(symbols, "Unit"),
            },
        )),

        "Float" => Ok((
            Some(id(f, symbols)?),
            Format::Float {
                name: f.intern_opt(symbols, "Name"),
                width: f.att("Width")?,
                min_value: f.att_opt("MinValue")?,
                max_value: f.att_opt("MaxValue")?,
                unit: f.intern_opt(symbols, "Unit"),
            },
        )),

        "String" => Ok((
            Some(id(f, symbols)?),
            Format::String {
                name: f.intern_opt(symbols, "Name"),
                encoding: f.intern(symbols, "Encoding")?,
                width: f.att("Width")?,
                variable_length: f.att_opt("VariableLength")?.unwrap_or(false),
                null_terminated: f.att_opt("NullTerminated")?.unwrap_or(false),
            },
        )),

        "Enumeration" => Ok((
            Some(id(f, symbols)?),
            Format::Enumeration {
                name: f.intern_opt(symbols, "Name"),
                width: f.att("Width")?,
                values: {
                    let mut values = Vec::new();

                    for v in f.children().filter(by_name("EnumValue")) {
                        values.push(EnumerationValue {
                            id: v.intern(symbols, "Id")?,
                            value: v.att("Value")?,
                            text: v.intern(symbols, "Text")?,
                        })
                    }

                    values
                },
            },
        )),

        "Reserved" => Ok((
            None,
            Format::Reserved {
                width: f.att("Width")?,
            },
        )),

        "RefType" => Ok((None, Format::Reference(f.intern(symbols, "RefId")?))),

        unexpected => Err(Error::ParseError(
            format!("unexpected format: {}", unexpected).into(),
        )),
    }
}
/// Parsed KNX master data containing all datapoint type definitions.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MasterData {
    /// Master data schema version.
    pub version: String,
    /// All main datapoint types.
    pub types: Vec<DatapointType>,
    /// All datapoint subtypes (including generic main-type entries).
    pub subtypes: Vec<Arc<DatapointSubtype>>,
    /// Index from subtype id to position in `subtypes`.
    pub by_id: HashMap<SharedString, usize>,
    /// Index from DPT number to position in `subtypes`.
    pub by_dpt: HashMap<DPT, usize>,
    symbols: Arc<StringPool>,
}

impl MasterData {
    /// Parses master data from a KNX master XML document.
    pub fn parse(document: &Document, symbols: Arc<StringPool>) -> Result<MasterData, Error> {
        let masterdata = document.root().child("KNX")?.child("MasterData")?;
        let version = masterdata.att::<String>("Version")?;

        let mut types = Vec::new();
        let mut subtypes = Vec::new();
        let mut formats_map = HashMap::new();

        for type_node in masterdata
            .child("DatapointTypes")?
            .children()
            .filter(by_name("DatapointType"))
        {
            let mut datapoint_type = DatapointType {
                id: type_node.intern(&symbols, "Id")?,
                dpt: DPT::new(type_node.att("Number")?, None),
                name: type_node.intern(&symbols, "Name")?,
                text: type_node.intern_opt(&symbols, "Text"),
                size: type_node.att("SizeInBit")?,
                subtypes: Vec::new(),
            };

            let mut type_formats = None;

            for subtype_node in type_node
                .child("DatapointSubtypes")?
                .children()
                .filter(by_name("DatapointSubtype"))
            {
                let mut formats = Vec::new();

                for format_node in subtype_node
                    .child("Format")?
                    .children()
                    .filter(Node::is_element)
                {
                    let (id, mut format) = parse_format(format_node, &symbols)?;

                    if let Some(id) = &id {
                        if !formats_map.contains_key(id) {
                            formats_map.insert(id.clone(), format.clone());
                        }
                    }

                    if let Format::Reference(id) = format {
                        format = formats_map
                            .get(&id)
                            .ok_or_else(|| {
                                Error::ParseError(
                                    format!("undefined format reference: {}", id).into(),
                                )
                            })?
                            .clone();
                    }

                    formats.push(format);
                }

                if type_formats.is_none() {
                    type_formats = Some(formats.clone());
                }

                let subtype = Arc::new(DatapointSubtype {
                    id: subtype_node.intern(&symbols, "Id")?,
                    dpt: DPT::new(datapoint_type.dpt.main, Some(subtype_node.att("Number")?)),
                    size: datapoint_type.size,
                    name: subtype_node.intern(&symbols, "Name")?,
                    text: subtype_node.intern_opt(&symbols, "Text"),
                    default: subtype_node.intern_opt(&symbols, "Default"),
                    formats,
                });

                subtypes.push(subtype.clone());
                datapoint_type.subtypes.push(subtype);
            }

            let generic = Arc::new(DatapointSubtype {
                id: datapoint_type.id.clone(),
                dpt: DPT::new(datapoint_type.dpt.main, None),
                size: datapoint_type.size,
                name: datapoint_type.name.clone(),
                text: datapoint_type.text.clone(),
                default: type_node.intern_opt(&symbols, "Default"),
                formats: type_formats.unwrap(),
            });

            subtypes.push(generic);

            types.push(datapoint_type);
        }

        let mut by_id = HashMap::new();
        let mut by_dpt = HashMap::new();

        for (ix, subtype) in subtypes.iter().enumerate() {
            by_id.insert(subtype.id.clone(), ix);
            by_dpt.insert(subtype.dpt, ix);
        }

        Ok(MasterData {
            version,
            types,
            subtypes,
            by_id,
            by_dpt,
            symbols,
        })
    }

    /// Looks up a datapoint subtype by its XML identifier string.
    pub fn by_id(&self, id: &str) -> Option<&Arc<DatapointSubtype>> {
        let id = self.symbols.get(id);
        self.by_id.get(&id).map(|ix| &self.subtypes[*ix])
    }

    /// Looks up a datapoint subtype by its DPT number.
    pub fn by_dpt(&self, dpt: DPT) -> Option<&Arc<DatapointSubtype>> {
        self.by_dpt.get(&dpt).map(|ix| &self.subtypes[*ix])
    }
}
