use std::{
    collections::HashMap, fs::read_to_string, io::{BufReader, Read, Seek}, path::PathBuf, sync::OnceLock
};

use crate::file_ext::*;

use anyhow::Context;
use half::vec;
use serde::{ser::{SerializeMap, SerializeSeq, SerializeStruct}, Deserialize, Serialize};
use uuid::Uuid;

use super::TypeDescriptor;

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    F8(u8),
    F16(u16),
    F32(f32),
    F64(f64),
    Guid([u8; 16]),
    Array(Vec<RszType>),
    Object(RszStruct<RszField>, u32),
}

pub struct RszValueWithInfo<'a>(&'a RszValue, &'a Vec<RszValue>);
pub struct RszTypeWithInfo<'a>(&'a RszType, &'a Vec<RszValue>);

#[derive(Debug, Clone, Deserialize)]
pub struct RszStruct<T> {
    name: String,
    crc: u32,
    hash: u32,
    fields: Vec<T>,
}

pub type RszValue = RszStruct<RszType>;


#[derive(Debug, Clone)]
pub struct DeRsz {
    pub roots: Vec<RszValue>,
    pub structs: Vec<RszValue>,
}

impl<'a> Serialize for DeRsz {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut state = serializer.serialize_struct("Rsz", self.roots.len())?;
        let context = &self.structs;
        for root in &self.roots { // do this to wrap in with context
            let val_with_context = RszValueWithInfo(&root, &context);
            state.serialize_field("struct", &val_with_context)?;
        }
        state.end()
    }
}

impl<'a> Serialize for RszValueWithInfo<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
                
            let r#struct = self.0;
            let context = self.1;
            let struct_info = RszDump::crc_map().get(&r#struct.hash).expect("Could not find struct in dump");
            let mut state = serializer.serialize_struct("RszValue", r#struct.fields.len())?;
            for i in 0..struct_info.fields.len() {
                let field_value = &r#struct.fields[i];
                let field_info = &struct_info.fields[i];
                let name = &field_info.name;
                let serialize_context = RszTypeWithInfo(field_value, context);
                state.serialize_field(name, &serialize_context)?;
            }
            state.end()
        
    }
}

impl<'a> Serialize for RszTypeWithInfo<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer 
    {
        let rsz_type = self.0;
        let structs = self.1;
        use RszType::*;
        return match rsz_type {
            Int8(v) => serializer.serialize_i8(*v), 
            Int16(v) => serializer.serialize_i16(*v), 
            Int32(v) => serializer.serialize_i32(*v), 
            Int64(v) => serializer.serialize_i64(*v), 
            UInt8(v) => serializer.serialize_u8(*v), 
            UInt16(v) => serializer.serialize_u16(*v), 
            UInt32(v) => serializer.serialize_u32(*v), 
            UInt64(v) => serializer.serialize_u64(*v), 
            Bool(v) => serializer.serialize_bool(*v),
            String(v) => serializer.serialize_str(v),
            F8(v) => serializer.serialize_u8(*v), 
            F16(v) => serializer.serialize_u16(*v), 
            F32(v) => serializer.serialize_f32(*v), 
            F64(v) => serializer.serialize_f64(*v), 
            Guid(id) => {
                let id = Uuid::from_bytes_le(*id);
                serializer.serialize_str(&id.to_string().as_str())
            },
            Object(info, ptr) => {
                let struct_derefed = &structs.get(*ptr as usize).expect("Struct not in context");
                //println!("{:?}", struct_derefed);
                let val = RszValueWithInfo(struct_derefed, structs);
                val.serialize(serializer)
                //serializer.serialize_str("NOT IMPLEMENTED")
            },
            Array(vec_of_types) => {
                //let struct_derefed = &structs.get(*ptr as usize).expect("Struct not in context");
                let mut state = serializer.serialize_seq(Some(vec_of_types.len()))?;
                for r#type in vec_of_types {
                    let type_with_context = RszTypeWithInfo(r#type, structs);
                    state.serialize_element(&type_with_context)?;
                }
                state.end()
                //serializer.serialize_str("NOT IMPLEMENTED")

            }
            _ => serializer.serialize_str("NOT IMPLEMENTED")
        }
    
    }
}

/*impl RszType {
    fn serialize<S>(&self, serializer: S, context: &DeRsz) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        let structs = context.structs;
        use RszType::*;
        match self {
            Int8(v) => serializer.seria(name, &v)?, 
            Int16(v) => state.serialize_field(name, &v)?,
            Int32(v) => state.serialize_field(name, &v)?,
            Int64(v) => state.serialize_field(name, &v)?,
            UInt8(v) => state.serialize_field(name, &v)?, 
            UInt16(v) => state.serialize_field(name, &v)?,
            UInt32(v) => state.serialize_field(name, &v)?,
            UInt64(v) => state.serialize_field(name, &v)?,
            Guid(id) => {
                let id = Uuid::from_bytes_le(*id);
                state.serialize_field(name, &id.to_string())?;
            },
            _ => ()
        }
    }
}*/

impl Serialize for RszValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut state = serializer.serialize_struct("RszStruct", self.fields.len())?;
        use RszType::*;
        let base_struct = RszDump::crc_map().get(&self.hash).expect(
            "Could not find hash for struct in map"
        );

        
        for i in 0..base_struct.fields.len() {
            let r#type = &self.fields[i];
            let field_info = &base_struct.fields[i];
            let name = &field_info.name;

        }
        state.end()
    }
}


pub struct RszDump;

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
            if field.array {
                data.seek_align_up(4).with_context(||{
                    format!("{:?}", field)
                })?;
                let count = data.read_u32()?;
                let vals = (0..count).map(|_| {
                    RszDump::field_to_type(&mut data, field)
                }).collect::<anyhow::Result<Vec<RszType>>>()?;
                field_values.push(RszType::Array(vals));
            } else {
                let r#type = RszDump::field_to_type(&mut data, field)?;
                field_values.push(r#type);
            }
        }

        Ok(RszValue {
            name: struct_type.name.clone(),
            crc: struct_type.crc,
            hash: struct_type.hash,
            fields: field_values,
        })
    }

    fn field_to_type<F: Read + Seek>(mut data: &mut F, field: &RszField) -> anyhow::Result<RszType> {
        data.seek_align_up(field.align.into()).with_context(|| {
            format!("{:?}", field)
        })?;
        let r#type = match field.r#type.as_str() {
            "S8" => RszType::Int8(data.read_i8()?),
            "S16" => RszType::Int16(data.read_i16()?),
            "S32" => RszType::Int32(data.read_i32()?),
            "S64" => RszType::Int64(data.read_i64()?),
            "U8" => RszType::UInt8(data.read_u8()?),
            "U16" => RszType::UInt16(data.read_u16()?),
            "U32" => RszType::UInt32(data.read_u32()?),
            "U64" => RszType::UInt64(data.read_u64()?),
            "F8" => RszType::F8(data.read_u8()?),
            "F16" => RszType::F16(data.read_u16()?),
            "F32" => RszType::F32(data.read_f32()?),
            "F64" => RszType::F64(data.read_f64()?),
            "Guid" => {
                let mut buf = [0; 16];
                for i in 0..16 {
                    buf[i] = data.read_u8()?;
                }
                RszType::Guid(buf) // make it read
            },
            "Bool" => RszType::Bool(data.read_bool()?),
            "String" => RszType::String(data.read_u8str()?),
            "Object" => {
                let mut x = None;
                 if let Some(mapped_crc) = RszDump::name_map().get(&field.original_type) {
                    if let Some(r#struct) = RszDump::crc_map().get(&mapped_crc) {
                        //println!("{:?}", r#struct);
                        //let values = self.parse_struct(&mut data, TypeDescriptor{crc: r#struct.crc, hash: r#struct.hash})?;
                        //RszType::Object(values)
                        x = Some(RszType::Object(r#struct.clone(), data.read_u32()?))
                    } else {
                        panic!("Name crc not in hash map {:X}", mapped_crc);
                    };
                } else {
                    panic!("field original type {:?} not in dump map", field);
                };
                x.unwrap()
            }

            _ => panic!("Type {:?} does not have FromField implemented", field.r#type),
            // maybe get enum
        };
        if field.array {
            //r#type = RszType::Array(Vec::new())
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


