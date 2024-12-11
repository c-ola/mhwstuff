use std::{
    collections::HashMap, fs::read_to_string, io::{BufReader, ErrorKind, Read, Seek}, path::PathBuf, sync::OnceLock
};

use crate::file_ext::*;

use anyhow::{anyhow, Context};
use nalgebra_glm::{Mat4x4, Vec2, Vec3, Vec4};
use serde::{de::value::UnitDeserializer, ser::{SerializeMap, SerializeSeq, SerializeStruct}, Deserialize, Serialize};
use uuid::Uuid;
use super::TypeDescriptor;

// enums to hold values in a lightweight Rsz Struct
#[derive(Debug, Clone)]
pub enum RszType {
    // Numbers
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    F8(u8),
    F16(u16),
    F32(f32),
    F64(f64),

    // Lin alg
    UInt2((u32, u32)),
    UInt3((u32, u32, u32)),
    UInt4((u32, u32, u32, u32)),
    Int2((i32, i32)),
    Int3((i32, i32, i32)),
    Int4((i32, i32, i32, i32)),
    Float2(Vec2),
    Float3(Vec3),
    Float4(Vec4),
    Mat4x4(Mat4x4),
    Vec2(Vec2), // might all be vec4
    Vec3(Vec3),
    Vec4(Vec4),

    Range((u32, u32)),
    RangeI((i32, i32)),

    // Shapes
    AABB(Vec3, Vec3),
    Capsule(Vec3, Vec3, Vec3),
    // ...
    Rect(u32, u32, u32, u32),
    
    Bool(bool),
    String(String),
    Guid([u8; 16]),
    Array(Vec<RszType>),
    Object(RszStruct<RszField>, u32),
    Enum(Box<RszType>, String),
    OBB,
    Data(Vec<u8>),
}

impl RszType {
    fn from_field<F: Read + Seek>(data: &mut F, field: &RszField) -> anyhow::Result<RszType> {
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

            "UInt2" => RszType::UInt2((data.read_u32()?, data.read_u32()?)),
            "UInt3" => RszType::UInt3((data.read_u32()?, data.read_u32()?, data.read_u32()?)),
            "UInt4" => RszType::UInt4((data.read_u32()?, data.read_u32()?, data.read_u32()?, data.read_u32()?)),
            "Int2" => RszType::Int2((data.read_i32()?, data.read_i32()?)),
            "Int3" => RszType::Int3((data.read_i32()?, data.read_i32()?, data.read_i32()?)),
            "Int4" => RszType::Int4((data.read_i32()?, data.read_i32()?, data.read_i32()?, data.read_i32()?)),
            "Vec2" => RszType::Vec2(data.read_f32vec2()?),
            "Vec3" => RszType::Vec3(data.read_f32vec3()?),
            "Vec4" => RszType::Vec4(data.read_f32vec4()?),
            "Float2" => RszType::Float2(data.read_f32vec2()?),
            "Float3" => RszType::Float3(data.read_f32vec3()?),
            "Float4" => RszType::Float4(data.read_f32vec4()?),
            "Mat4" => RszType::Mat4x4(data.read_f32m4x4()?),

            "Range" => RszType::Range((data.read_u32()?, data.read_u32()?)),
            "RangeI" => RszType::RangeI((data.read_i32()?, data.read_i32()?)),

            "Data" => {
                let mut v = Vec::new();
                //let n = data.read_u32();
                for _ in 0..field.size {
                    v.push(data.read_u8()?);
                }
                RszType::Data(v)
            },
            "AABB" => {
                RszType::AABB(data.read_f32vec3()?, data.read_f32vec3()?)
            },
            "Capsule" => {
                RszType::Capsule(data.read_f32vec3()?, data.read_f32vec3()?, data.read_f32vec3()?)
            },
            "Rect" => {
                RszType::Rect(data.read_u32()?, data.read_u32()?, data.read_u32()?, data.read_u32()?)
            },
            /*"OBB" => {
                data.seek_relative(field.size.into())?;
                RszType::OBB
            },*/
            "Guid" => {
                let mut buf = [0; 16];
                for i in 0..16 {
                    buf[i] = data.read_u8()?;
                }
                RszType::Guid(buf) // make it read
            },
            "Bool" => RszType::Bool(data.read_bool()?),
            "String" | "Resource" => RszType::String(data.read_utf16str()?),
            "Object" | "UserData" | "RuntimeType" => {
                let x;
                if let Some(mapped_crc) = RszDump::name_map().get(&field.original_type) {
                    if let Some(r#struct) = RszDump::crc_map().get(&mapped_crc) {
                        x = RszType::Object(r#struct.clone(), data.read_u32()?)
                    } else {
                        return Err(anyhow!("Name crc not in hash map {:X}", mapped_crc))
                    };
                } else {
                    return Err(anyhow!("field original type {:?} not in dump map", field))
                };
                x
            },
            _ => {
                return Err(anyhow!("Type {:?} is not implemented", field.r#type))
            }
        };


        if field.original_type.ends_with("Serializable") || field.original_type.ends_with("Fixed") 
        || field.original_type.ends_with("Serializable[]") || field.original_type.ends_with("Fixed[]"){
            Ok(RszType::Enum(Box::new(r#type), field.original_type.clone()))
        } else {
            Ok(r#type)
        }
    }
}

pub struct RszTypeWithInfo<'a>(&'a RszType, &'a Vec<RszValue>);

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
            Vec2(v) => v.serialize(serializer),
            Vec3(v) => v.serialize(serializer),
            Vec4(v) => v.serialize(serializer),
            UInt2(v) => v.serialize(serializer),
            UInt3(v) => v.serialize(serializer),
            UInt4(v) => v.serialize(serializer),
            Int2(v) => v.serialize(serializer),
            Int3(v) => v.serialize(serializer),
            Int4(v) => v.serialize(serializer),
            Float2(v) => v.serialize(serializer),
            Float3(v) => v.serialize(serializer),
            Float4(v) => v.serialize(serializer),
            Mat4x4(v) => v.serialize(serializer),

            Range(v) => v.serialize(serializer),
            RangeI(v) => v.serialize(serializer),

            Guid(id) => {
                let id = Uuid::from_bytes_le(*id);
                serializer.serialize_str(&id.to_string().as_str())
            },
            Object(_info, ptr) => {
                match &structs.get(*ptr as usize) {
                    Some(struct_derefed) => {
                        let val = RszValueWithInfo(struct_derefed, structs);
                        val.serialize(serializer)
                    }
                    None => {
                        eprintln!("{rsz_type:?}");
                        Err(serde::ser::Error::custom("Could not find Object pointer in data"))
                    }
                }
            },
            Enum(underlying, name) => {
                let underlying = *underlying.clone();
                match underlying {
                    Object(_info, ptr) => {
                        let res = &structs.get(ptr as usize);
                        let struct_derefed = match res {
                            Some(struct_derefed) => {
                                struct_derefed
                            }
                            None => {
                                eprintln!("{rsz_type:?}");
                                return Err(serde::ser::Error::custom("Could not find Enum Object pointer in data"))
                            }
                        };
                        let x = struct_derefed.fields[0].clone();
                        //serializer.serialize_str(format!("{x:?} name goes here").as_str());
                        let v = match x {
                            RszType::UInt64(v) => Ok(v.to_string()),
                            RszType::UInt32(v) => Ok(v.to_string()),
                            RszType::UInt16(v) => Ok(v.to_string()),
                            RszType::UInt8(v) => Ok(v.to_string()),
                            RszType::Int64(v) => Ok(v.to_string()),
                            RszType::Int32(v) => Ok(v.to_string()),
                            RszType::Int16(v) => Ok(v.to_string()),
                            RszType::Int8(v) => Ok(v.to_string()),
                            RszType::Object(_info, ptr) => {
                                match &structs.get(ptr as usize) {
                                    Some(struct_derefed) => {
                                        let val = RszValueWithInfo(struct_derefed, structs);
                                        return val.serialize(serializer)
                                    }
                                    None => {
                                        eprintln!("{rsz_type:?}");
                                        Err(serde::ser::Error::custom("Could not find Object pointer in data"))
                                    }
                                }
                            },
                            _ => {
                                eprintln!("{rsz_type:?}");
                                Err(serde::ser::Error::custom("Unknown underlying Enum type"))
                            }
                        }?;
                        match get_enum_name(name.to_string(), v.clone()){
                            None => serializer.serialize_str(format!("{v} // Could not find enum value in map {name}").as_str()),
                            Some(value) => serializer.serialize_str(&value)
                        }
                    },
                    Int32(v) => {
                        match get_enum_name(name.to_string(), v.to_string()) {
                            None => serializer.serialize_str(format!("{v} // Could not find enum value in map {name}").as_str()),
                            Some(value) => serializer.serialize_str(&value)
                        }
                    },
                    Int64(v) => {
                        match get_enum_name(name.to_string(), v.to_string()) {
                            None => serializer.serialize_str(format!("{v} // Could not find enum value in map {name}").as_str()),
                            Some(value) => serializer.serialize_str(&value)
                        }
                    },
                    UInt32(v) => {
                        match get_enum_name(name.to_string(), v.to_string()) {
                            None => serializer.serialize_str(format!("{v} // Could not find enum value in map {name}").as_str()),
                            Some(value) => serializer.serialize_str(&value)
                        }
                    },
                    UInt64(v) => {
                        match get_enum_name(name.to_string(), v.to_string()) {
                            None => serializer.serialize_str(format!("{v} // Could not find enum value in map {name}").as_str()),
                            Some(value) => serializer.serialize_str(&value)
                        }
                    }
                    _ => {
                        eprintln!("{rsz_type:?}");
                        Err(serde::ser::Error::custom("Unknown underlying Enum type"))
                    }
                }
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


#[derive(Debug, Clone, Deserialize)]
pub struct RszStruct<T> {
    name: String,
    crc: u32,
    hash: u32,
    fields: Vec<T>,
}

pub type RszValue = RszStruct<RszType>;

pub struct RszValueWithInfo<'a>(&'a RszValue, &'a Vec<RszValue>);

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



pub struct RszDump;

impl RszDump {
    pub fn parse_struct<'a, F: 'a + Read + Seek>(
        mut data: F,
        type_descriptor: TypeDescriptor,
    ) -> anyhow::Result<RszValue> {
        let struct_type = RszDump::crc_map()
            .get(&type_descriptor.hash)
            .with_context(|| format!("Unexpected Type: not in Rsz Dump"))?;
        
        println!("{:?}", struct_type);
        let mut field_values = Vec::new();
        for field in &struct_type.fields {
            if field.array {
                data.seek_align_up(4).with_context(||{
                    format!("{:?}", field)
                })?;
                let count = data.read_u32()?;
                let vals = (0..count).map(|_| {
                    RszType::from_field(&mut data, field)
                }).collect::<anyhow::Result<Vec<RszType>>>()?;
                field_values.push(RszType::Array(vals));
            } else {
                let r#type = RszType::from_field(&mut data, field)?;
                field_values.push(r#type);
            }
        }
        println!("{:?}", field_values);
        Ok(RszValue {
            name: struct_type.name.clone(),
            crc: struct_type.crc,
            hash: struct_type.hash,
            fields: field_values,
        })
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
            for i in 0..self.roots.len() {
            //for root in &self.roots { // do this to wrap in with context
                let r#struct = self.roots[i].clone();
                let x = RszDump::crc_map().get(&r#struct.hash);
                let name = match x {
                    Some(v) => &v.name,
                    None => "unknown struct?"
                };
                let val_with_context = RszValueWithInfo(&r#struct, &context);
                println!("{}", r#struct.name);
                state.serialize_field(name, &val_with_context)?;
            }
            state.end()
    }
}


fn enum_map() -> &'static HashMap<String, HashMap<String, String>> {
    static HASHMAP: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let json_data = std::fs::read_to_string("gen/enums.json").unwrap();
        let hashmap: HashMap<String, HashMap<String, String>> = serde_json::from_str(&json_data).unwrap();
        hashmap
    })
}

fn get_enum_name(name: String, value: String) -> Option<String> {
    let name = name.replace("[]", "").replace("_Serializable", "_Fixed");
    if let Some(map) = enum_map().get(&name) {
        if let Some(value) = map.get(&value){
            Some(value.to_string())
        } else {
            None
        }
    } else {
        None
    }
}

