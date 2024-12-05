use std::{
    collections::HashMap, fs::read_to_string, io::{BufReader, Read, Seek}, path::PathBuf, sync::OnceLock
};

use crate::file_ext::*;

use anyhow::Context;
use serde::Deserialize;

use super::TypeDescriptor;

#[derive(Debug, Clone, Deserialize)]
pub struct RszField {
    align: u32,
    array: bool,
    name: String,
    native: bool,          // almost always false, except for some via types
    original_type: String, //should also be used to index other structs
    size: u32,
    r#type: String, //basic type of the struct
}

// enums to hold values in a lightweight Rsz Struct
#[derive(Debug, Clone)]
pub enum RszType {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Bool(bool),
    String(String),
    Guid([u8; 16]),
    Array(Vec<RszType>),
    Object(RszValue),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RszStruct<T> {
    name: String,
    crc: u32,
    hash: u32,
    fields: Vec<T>,
}

pub type RszValue = RszStruct<RszType>;

#[derive(Debug)]
pub struct RszDump {
    hash_map: HashMap<u32, RszStruct<RszField>>,
    name_map: HashMap<String, u32>,
}

impl RszDump {
    pub fn parse_struct<'a, F: 'a + Read + Seek>(
        mut data: F,
        type_descriptor: TypeDescriptor,
    ) -> anyhow::Result<RszValue> {
        let struct_type = RszDump::crc_map()
            .get(&type_descriptor.hash)
            .with_context(|| format!("Unexpected Type: not in Rsz Dump"))?;

        let mut field_values = Vec::new();
        for field in &struct_type.fields {
            let r#type = RszDump::field_to_type(&mut data, field)?;
            field_values.push(r#type);
        }

        Ok(RszValue {
            name: struct_type.name.clone(),
            crc: struct_type.crc,
            hash: struct_type.hash,
            fields: field_values,
        })
    }

    fn field_to_type<F: Read + Seek>(mut data: &mut F, field: &RszField) -> anyhow::Result<RszType> {
        data.seek_align_up(field.align.into())?;
        let mut r#type = match field.r#type.as_str() {
            "S8" => RszType::Int8(data.read_i8()?),
            "S16" => RszType::Int16(data.read_i16()?),
            "S32" => RszType::Int32(data.read_i32()?),
            "S64" => RszType::Int64(data.read_i64()?),
            "U8" => RszType::UInt8(data.read_u8()?),
            "U16" => RszType::UInt16(data.read_u16()?),
            "U32" => RszType::UInt32(data.read_u32()?),
            "U64" => RszType::UInt64(data.read_u64()?),
            "Guid" => {
                RszType::Guid([0; 16]) // make it read
            },
            "Bool" => RszType::Bool(data.read_bool()?),
            "String" => RszType::String(data.read_u8str()?),
            "Object" => {
                if let Some(mapped_crc) = RszDump::name_map().get(&field.original_type) {
                    if let Some(r#struct) = RszDump::crc_map().get(&mapped_crc) {
                        println!("{:?}", r#struct);
                        //let values = self.parse_struct(&mut data, TypeDescriptor{crc: r#struct.crc, hash: r#struct.hash})?;
                        //RszType::Object(values)
                        RszType::Int8(0)
                    } else {
                        panic!("Name crc not in hash map {:X}", mapped_crc);
                    };
                } else {
                    panic!("field original type {:?} not in dump map", field);
                }
                RszType::Int8(0)
            }

            _ => panic!("Type {:?} does not have FromField implemented", field.r#type),
            // maybe get enum
        };
        if field.array {
            r#type = RszType::Array(Vec::new())
        }
        Ok(r#type)
    }
    fn crc_map() -> &'static HashMap<u32, RszStruct<RszField>> {
        #[derive(Debug, Clone, Deserialize)]
        pub struct RszStructTemp<T> {
            name: String,
            crc: String,
            fields: Vec<T>,
        }
        static HASHMAP: OnceLock<HashMap<u32, RszStruct<RszField>>> = OnceLock::new();
        HASHMAP.get_or_init(|| {
            let mut m = HashMap::new();
            let file = std::fs::File::open("rszmhwilds.json").unwrap();
            let reader = BufReader::new(file);
            let stream = serde_json::Deserializer::from_reader(reader).into_iter::<serde_json::Value>();

            //let mut structs: Vec<RszStruct<RszField>> = Vec::new();
            for value in stream {
                match value {
                    Ok(json_value) => {
                        if let serde_json::Value::Object(map) = json_value {
                            // This is where each struct is actually parsed
                            for (_key, value) in map {
                                //println!("{_key:?}, {value:?}");
                                let mut rsz_struct: RszStructTemp<RszField> =
                                    serde_json::from_value(value).expect("Could not parse struct");

                                for field in &mut rsz_struct.fields {
                                    if field.original_type == "ace.user_data.ExcelUserData.cData[]" {
                                        field.original_type = rsz_struct.name.clone() + ".cData[]"
                                    }
                                }
                                let rsz_struct: RszStruct<RszField> = RszStruct {
                                    name: rsz_struct.name,
                                    crc: u32::from_str_radix(&rsz_struct.crc, 16).unwrap(),
                                    hash: u32::from_str_radix(&_key, 16).unwrap(),
                                    fields: rsz_struct.fields
                                };
                                m.insert(rsz_struct.hash, rsz_struct);
                            }
                        } else {
                            println!("Expected a JSON object!");
                        }
                    }
                    Err(e) => eprintln!("Error parsing JSON: {}", e),
                }
            }
            m
        })
    }

    fn name_map() -> &'static HashMap<String, u32> {
        #[derive(Debug, Clone, Deserialize)]
        pub struct RszStructTemp<T> {
            name: String,
            crc: String,
            fields: Vec<T>,
        }
        static HASHMAP: OnceLock<HashMap<String, u32>> = OnceLock::new();
        HASHMAP.get_or_init(|| {
            let mut m = HashMap::new();
            let file = std::fs::File::open("rszmhwilds.json").unwrap();
            let reader = BufReader::new(file);
            let stream = serde_json::Deserializer::from_reader(reader).into_iter::<serde_json::Value>();

            //let mut structs: Vec<RszStruct<RszField>> = Vec::new();
            for value in stream {
                match value {
                    Ok(json_value) => {
                        if let serde_json::Value::Object(map) = json_value {
                            // This is where each struct is actually parsed
                            for (_key, value) in map {
                                //println!("{_key:?}, {value:?}");
                                let rsz_struct: RszStructTemp<RszField> =
                                    serde_json::from_value(value).expect("Could not parse struct");
                                let rsz_struct: RszStruct<RszField> = RszStruct {
                                    name: rsz_struct.name,
                                    crc: u32::from_str_radix(&rsz_struct.crc, 16).unwrap(),
                                    hash: u32::from_str_radix(&_key, 16).unwrap(),
                                    fields: rsz_struct.fields
                                };
                                m.insert(rsz_struct.name, rsz_struct.hash);
                            }
                        } else {
                            println!("Expected a JSON object!");
                        }
                    }
                    Err(e) => eprintln!("Error parsing JSON: {}", e),
                }
            }
            m
        })
    }
}


