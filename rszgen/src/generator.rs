use convert_case::{Case, Casing};
use serde::Deserialize;
use core::fmt;
use std::{io::{BufReader, Read, Result, Write}, path::PathBuf};

#[derive(Deserialize, Debug, Clone)]
#[allow(unused)]
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
        let basic_type = type_map(self.original_type.clone());
        
        let mut type_name = match basic_type {
            Some(x) => x.to_string(),
            None => {
                self.original_type.clone().replace(".", "_").to_case(Case::Pascal).trim_start_matches("App").to_string().replace("[]", "")
            },
        };
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
        let s = format!("{}{}: {},\n", " ".repeat(spaces), name, type_name);
        //if self.original_type == "ace.user_data.ExcelUserData.cData[]" {
            //s.push_str(&format!("{}_idk: u8,\n", " ".repeat(spaces)));
        //}
        s
    }

    fn parse_type(&self, s: String) -> String {
        s
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


#[derive(Clone)]
pub struct Rsz {
    _structs: Vec<RszStruct>,
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

        for rsz_struct in structs {
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
    let binding = name.replace("[]", "").replace("System.Collections.Generic.List`1<", "Vec<").replace(">", ">");
    let mut name = binding.as_str();
    if let Some(pos) = name.find('.') {
        name = name.split_at(pos + 1).1
    };

    let name: String = name.replace("_", "").split('.').collect();
    let name = name.trim_start_matches("userdata");
    name.to_string()
}

fn type_map(r#type: String) -> Option<&'static str> {
    match r#type.as_str() {
        "System.Int8" => Some("i8"),
        "System.Int16" => Some("i16"),
        "System.Int32" => Some("i32"),
        "System.Int64" => Some("i64"),
        "System.UInt8" => Some("u8"),
        "System.UInt16" => Some("u16"),
        "System.Uint32" => Some("u32"),
        "System.UInt64" => Some("u64"),
        "via.Size" => Some("usize"),
        "via.f8" => Some("f8"),
        "via.f16" => Some("f16"),
        "via.f32" => Some("f32"),
        "via.f64" => Some("f64"),
        "System.Boolean" => Some("bool"),
        "System.Guid" => Some("Guid"),
        "String" => Some("String"),
        "Range" => Some("Range"),
        _ => None
    }
}
