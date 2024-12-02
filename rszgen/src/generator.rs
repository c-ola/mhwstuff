use convert_case::{Case, Casing};
use serde::Deserialize;
use core::fmt;
use std::{collections::HashMap, fmt::{format, write}, fs::File, io::{BufReader, Read, Result, Write}, path::PathBuf
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
    pub fn to_rust_field(&self, spaces: usize, parent: String) -> String {
        let mut type_name: String = type_map(self.name.clone()).to_string();
        let mut type_name = self.original_type.clone().replace(".", "_").to_case(Case::Pascal);
        if self.r#type == "Object" {
            if self.original_type == "ace.user_data.ExcelUserData.cData[]" {
                type_name = squish_name(&parent);
                type_name.push_str("cData")
            } else {
                type_name = squish_name(&self.original_type);
            }
        }
        if self.array {
            type_name = format!("Vec<{}>", type_name); 
        }
        let mut name = self.name.replace("_", "").to_case(Case::Snake);
        if name == "type" {
            name = "r#".to_string() + &name; 
        }
        let mut s = format!("{}{}: {},\n", " ".repeat(spaces), name, type_name);
        if self.original_type == "ace.user_data.ExcelUserData.cData[]" {
            s.push_str(&format!("{}_idk: u8,\n", " ".repeat(spaces)));
        }
        s
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
    pub fn to_rust_struct(&self) -> String {
        let mut s: String = "".to_string();
        // add macro wrapper
        let macro_name = "rsz_struct!".to_string();
        s.push_str(&format!("{macro_name} {{\n"));
        let rsz_macro = format!("    #[rsz(\"{}\", 0x{} = {})]\n", self.name, self.crc, 0);
        let derive = "    #[derive(Debug, Serialize)]\n";
        let allow = "    #[allow(dead_code)]\n";
        s.push_str(&rsz_macro);
        s.push_str(derive);
        s.push_str(allow);

        let name = squish_name(&self.name);
        let fields: String = self.fields.iter().map(|f| f.to_rust_field(8, self.name.clone())).collect();
        let identifier = "pub struct";
        s.push_str(&format!("    {} {} {{\n{}    }}\n", identifier, name, fields));

        s.push_str("}\n");

        s
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
pub struct Rsz {
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
                        //write!(f, "{tabs}namespace {} {{\n", key)?;
                        write!(f, "{val}")?;
                        //write!(f, "{tabs}}}\n")?;
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
            .filter(|x| { true
            })
            .collect::<Vec<RszStruct>>();

        let mut struct_file =
            std::fs::File::create(struct_file_path).expect("Could not create file");
        struct_file.write(b"// Auto Generated structs from mhwilds rsz dump\n")?;
        struct_file.write(b"// rustfmt::skip\n")?;

        let mut top_node = RszNode::NameSpace(HashMap::new(), 0);
        for rsz_struct in structs {
            /*let mut cur_node: &mut RszNode = &mut top_node;
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
            }*/


            if !rsz_struct.fields.is_empty() {
                let _ = struct_file
                    .write(rsz_struct.to_rust_struct().as_str().as_bytes())
                    .expect("Could not write to file");
            }
        }
        Ok(())
    }
}

fn squish_name(name: &str) -> String {
    if name.starts_with("System") {
    //    return String::from("");
    }
    let binding = name.replace("[]", "").replace("System.Collections.Generic.List`1<", "Vec<").replace(">", ">");
    let mut name = binding.as_str();
    if let Some(pos) = name.find('.') {
        name = name.split_at(pos + 1).1
    };

    let name: String = name.replace("_", "").split('.').collect();
    name
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
