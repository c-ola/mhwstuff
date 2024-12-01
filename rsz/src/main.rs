use serde::Deserialize;
use core::fmt;
use std::{
    any::{Any, TypeId}, collections::HashMap, fmt::{format, write}, fs::File, io::{BufReader, Read, Result, Write}, path::PathBuf
};

#[derive(Deserialize, Debug, Clone)]
struct RszField {
    align: u32,
    array: bool,
    name: String,
    native: bool,
    original_type: String,
    size: u32,
    r#type: String,
}

impl RszField {
    pub fn to_c_field(&self) -> String {
        format!("    {} {};\n", self.r#type, self.name)
    }

    pub fn to_rust_field(&self) -> String {
        format!("    {}: {},\n", self.name, type_map(self.r#type.clone()))
    }
}

impl fmt::Display for RszField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.r#type)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct RszStruct {
    crc: String,
    name: String,
    fields: Vec<RszField>,
}

impl RszStruct {
    pub fn to_c_struct(&self) -> String {
        if self.name.starts_with("System") {
            return String::from("");
        }
        format!("struct {} {{\n{}}};\n", self.name, self.to_c_fields())
    }

    pub fn to_rust_struct(&self) -> String {
        if self.name.starts_with("System") {
            return String::from("");
        }
        let binding = self.name.replace("[]", "Array");
        let mut name = binding.as_str();
        if let Some(pos) = name.find('.') {
            name = name.split_at(pos + 1).1
        };
        format!("struct {} {{\n{}}}\n", name.replace(".", "_"), self.to_rust_fields())
    }

    fn to_c_fields(&self) -> String {
        self.fields.iter().map(|f| f.to_c_field()).collect()
    }

    fn to_rust_fields(&self) -> String {
        self.fields.iter().map(|f| f.to_rust_field()).collect()
    }
}

impl fmt::Display for RszStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tabs = " ".repeat((self.name.matches('.').count() + 1) * 4);
        let name = self.name.split('.').last().unwrap();
        write!(f, "{tabs}struct {} {{\n", name)?;
        for i in &self.fields {
            write!(f, "    {tabs}{}\n", i)?;
        }
        write!(f, "{tabs}}}")

    }
}

#[derive(Clone)]
struct Rsz {
    _structs: Vec<RszStruct>,
}

#[derive(Debug)]
enum RszNode {
    r#Struct(RszStruct),
    NameSpace(HashMap<String, RszNode>, usize),
}

impl fmt::Display for RszNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RszNode::r#Struct(x) => write!(f, "{x}\n"),
            RszNode::NameSpace(x, level) => {
                for (key, val) in x {
                    let print_namespace = match val {
                        RszNode::r#Struct(x) => &x.name == key,
                        RszNode::NameSpace(x, _) => x.len() != 1, 
                    };
                    let tabs = " ".repeat(level * 4);
                    if !print_namespace{
                        write!(f, "{val}")?;
                    } else {
                        write!(f, "{tabs}namespace {} {{\n", key)?;
                        write!(f, "{val}")?;
                        write!(f, "{tabs}}}\n")?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl Rsz {
    pub fn gen_structs(path: &mut PathBuf) -> Result<()> {
        // Create a JSON stream deserializer
        let file = std::fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let stream = serde_json::Deserializer::from_reader(reader).into_iter::<serde_json::Value>();

        let mut structs: Vec<RszStruct> = Vec::new();
        for value in stream {
            match value {
                Ok(json_value) => {
                    if let serde_json::Value::Object(map) = json_value {
                        // This is where each struct is actually parsed
                        for (_key, value) in map {
                            let rsz_struct: RszStruct =
                                serde_json::from_value(value).expect("Could not parse struct");
                            structs.push(rsz_struct);
                        }
                    } else {
                        println!("Expected a JSON object!");
                    }
                }
                Err(e) => eprintln!("Error parsing JSON: {}", e),
            }
        }

        let mut struct_file_path = path.clone();
        struct_file_path.set_extension("rs.ext");

        structs.sort_by(|a, b| a.name.cmp(&b.name));
        structs = structs
            .into_iter()
            .filter(|x| {
                !(x.name.starts_with("ace")
                    || x.name.contains(['!', '<', '>'])
                    || x.name.contains("[["))
                    && x.name.starts_with("app")
            })
            .collect::<Vec<RszStruct>>();

        let mut struct_file =
            std::fs::File::create(struct_file_path).expect("Could not create file");
        struct_file.write(b"// Auto Generated structs from mhwilds rsz dump\n")?;
        struct_file.write(b"// rustfmt::skip\n")?;

        let mut top_node = RszNode::NameSpace(HashMap::new(), 0);
        for rsz_struct in structs {
            let mut cur_node: &mut RszNode = &mut top_node;
            let namespace: Vec<&str> = rsz_struct.name.split(".").collect();
            let count = namespace.len();
            for (i, name) in namespace.into_iter().enumerate() {
                let is_last = i == count - 1;
                match cur_node {
                    RszNode::NameSpace(x, level) => {
                        if x.contains_key(name) {
                        } else {
                            x.insert(name.to_string(), RszNode::NameSpace(HashMap::new(), *level + 1));
                        }
                        cur_node = x.get_mut(name).expect("Could not find map");
                    }
                    _ => ()
                }
                if is_last {
                    match cur_node {
                        RszNode::NameSpace(x, _level) => {
                            x.insert(rsz_struct.crc.to_string(), RszNode::Struct(rsz_struct.clone()));
                        }
                        _ => ()
                    }
                }
            }



            let _ = struct_file
                .write(rsz_struct.to_rust_struct().as_str().as_bytes())
                .expect("Could not write to file");
        }
        println!("{top_node}");
        Ok(())
    }
}

fn type_map(r#type: String) -> &'static str {
    match r#type.as_str() {
        "Int2" => "Int2",
        "Int3" => "Int3",
        "S8" => "i8",
        "S16" => "i16",
        "S32" => "i32",
        "S64" => "i64",
        "Uint2" => "Uint2",
        "U8" => "u8",
        "U16" => "u16",
        "U32" => "u32",
        "U64" => "u64",
        "Size" => "Size",
        "Float2" => "Float2",
        "Float3" => "Float3",
        "Float4" => "Float4",
        "F8" => "f8",
        "F16" => "f16",
        "F32" => "f32",
        "F64" => "f64",
        "Color" => "Color",
        "Range" => "Range",
        "Data" => "Data",
        "Resource" => "Resource",
        "String" => "String",
        "Object" => "Object",
        "GameObjectRef" => "GameObjectRef",
        "Bool" => "bool",
        "Guid" => "Guid",
        "UserData" => "UserData",
        "Vec2" => "Vec2",
        "Vec3" => "Vec3",
        "Vec4" => "Vec4",
        "Mat4" => "Mat4",
        "Quaternion" => "Quaternion",
        "Point" => "Point",
        "OBB" => "OBB",
        "RuntimeType" => "RuntimeType",
        "AABB" => "AABB",
        "Sphere" => "Sphere",
        "RangeI" => "RangeI",
        "DateTime" => "DateTime",
        "Capsule" => "Capsule",
        "Position" => "Position",
        "Rect" => "Rect",
        "Frustum" => "Frustum",

        x => {
            println!("{x}");
            "_UNKNOWN_"
        }
    }
}

pub mod app {
    pub mod user_data {
        pub struct AccessoryData {}
    }
}

fn main() {
    let x: app::user_data::AccessoryData;
    let _ = Rsz::gen_structs(&mut PathBuf::from("rszmhwilds.json"));
}
