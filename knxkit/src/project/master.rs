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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DatapointType {
    pub id: SharedString,
    pub dpt: DPT,

    pub name: SharedString,
    pub text: Option<SharedString>,

    pub size: u16,

    pub subtypes: Vec<Arc<DatapointSubtype>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DatapointSubtype {
    pub id: SharedString,
    pub dpt: DPT,
    pub size: u16,

    pub name: SharedString,
    pub text: Option<SharedString>,

    pub formats: Vec<Format>,
    pub default: Option<SharedString>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EnumerationValue {
    pub id: SharedString,
    pub value: u8,
    pub text: SharedString,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Format {
    Bit {
        name: Option<SharedString>,
        cleared: SharedString,
        set: SharedString,
    },

    Integer {
        name: Option<SharedString>,
        width: u8,
        signed: bool,
        min_inclusive: Option<i64>,
        max_inclusive: Option<i64>,
        coefficient: Option<f64>,
        unit: Option<SharedString>,
    },

    Float {
        name: Option<SharedString>,
        width: u8,
        min_value: Option<f64>,
        max_value: Option<f64>,
        unit: Option<SharedString>,
    },

    String {
        name: Option<SharedString>,
        encoding: SharedString,
        width: u16,
        variable_length: bool,
        null_terminated: bool,
    },

    Enumeration {
        name: Option<SharedString>,
        width: u8,
        values: Vec<EnumerationValue>,
    },

    Reserved {
        width: u8,
    },

    Reference(SharedString),
}

impl Format {
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
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MasterData {
    pub version: String,
    pub types: Vec<DatapointType>,
    pub subtypes: Vec<Arc<DatapointSubtype>>,
    pub by_id: HashMap<SharedString, usize>,
    pub by_dpt: HashMap<DPT, usize>,
    symbols: Arc<StringPool>,
}

impl MasterData {
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

    pub fn by_id(&self, id: &str) -> Option<&Arc<DatapointSubtype>> {
        let id = self.symbols.get(id);
        self.by_id.get(&id).map(|ix| &self.subtypes[*ix])
    }

    pub fn by_dpt(&self, dpt: DPT) -> Option<&Arc<DatapointSubtype>> {
        self.by_dpt.get(&dpt).map(|ix| &self.subtypes[*ix])
    }
}
