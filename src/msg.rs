use std::{fs::File, io::Result, os::unix::fs, path::PathBuf};

use serde::{ser::SerializeStruct, Serialize};
use uuid::Uuid;

use crate::byte_reader::BytesFile;

const KEY: [u8; 16] = [207, 206, 251, 248, 236, 10, 51, 102, 147, 169, 29, 147, 80, 57, 95, 9];

#[derive(Debug)]
struct Entry {
    unkn: u32,
    guid: [u8; 16],
    hash: u32,
    name: String,
    attributes: u64,
    content: Vec<String>,
}

impl Entry {
    pub fn print(&self) {
        let json = serde_json::to_string(self).unwrap();
        println!("{}", json);
    }
}

impl Serialize for Entry {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut state = serializer.serialize_struct("Entry", 4)?;
        let uuid = Uuid::from_bytes_le(self.guid).to_string();
        state.serialize_field("guid", &uuid)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("hash", &self.hash)?;
        state.serialize_field("content", &self.content)?;
        state.end()
    }
}


#[derive(Debug, Default)]
pub struct Msg {
    entries: Vec<Entry>,
}

#[derive(Default)]
pub struct MsgHeader {
    version: u32,
    magic: [u8; 4],
    header_offset: u64,
    entry_count: u32,
    type_count: u32,
    lang_count: u32,
    data1_offset: u64,
    p_offset: u64,
    lang_offset: u64,
    type_offset: u64,
    type_name_offset: u64
}

impl Msg {
    pub fn new(file_name: String) -> Result<Msg> {
        let mut file = BytesFile::new(file_name)?;
        let version = file.read::<u32>()?;
        let magic = file.readn::<u8, 4>()?;
        let header_offset = file.read::<u64>()?;
        let entry_count = file.read::<u32>()?;
        let type_count = file.read::<u32>()?;
        let lang_count = file.read::<u32>()?;
        file.read::<u32>()?; // null
        let data_offset = file.read::<u64>()?;
        let p_offset = file.read::<u64>()?;
        let lang_offset = file.read::<u64>()?;
        let type_offset = file.read::<u64>()?;
        let type_name_offset = file.read::<u64>()?;
        let base_entry_offset = file.index;
        //println!("{entry_count}, {type_count}, {lang_count}");

        // Read Data
        file.index = data_offset as usize;
        let mut data = file.read_bytes_to_vec(file.len() - data_offset as usize)?;
        let mut b = 0;
        let mut num = 0;
        let mut num2 = 0;
        while num < data.len() {
            let b2 = b;
            b = data[num2];
            let num3 = num & 0xf;
            num += 1;
            data[num2] = b2 ^ b ^ KEY[num3];
            num2 = num;
        }

        let mut data = BytesFile {
            data,
            index: 0,
        };

        file.index = lang_offset as usize;
        let languages = (0..lang_count).map(|_| file.read::<u32>()).collect::<Result<Vec<_>>>()?;

        file.index = p_offset as usize;
        file.read::<u64>()?; // idk what this does

        file.index += type_offset as usize;
        let attribute_types = (0..type_count).map(|_| file.read::<u32>().unwrap_or(0) as i32).collect::<Vec<i32>>();
        file.index += type_name_offset as usize;
        let type_names = (0..type_count).map(|_| file.read::<u32>().unwrap_or(0) as i32).collect::<Vec<i32>>();


        let mut entries: Vec<Entry> = Vec::new();
        for i in 0..entry_count as usize {
            file.index = base_entry_offset + i * 8;
            let entry_offset = file.read::<u64>()?;
            file.index = entry_offset as usize;

            let guid_vec = file.read_bytes_to_vec(16)?;
            let mut guid: [u8; 16] = [0; 16];
            for i in 0..16 {
                guid[i] = guid_vec[i];
            }

            let unkn = file.read::<u32>()?;
            let hash = file.read::<u32>()?;
            let name = file.read::<u64>()?;
            let attributes = file.read::<u64>()?;
            let content = (0..lang_count).map(|_| {
                let offset = file.read::<u64>().unwrap_or(0) as usize;
                data.read_utf16(offset - data_offset as usize).unwrap_or("".to_string())
            }).collect::<Vec<_>>();
            let name = data.read_utf16(name as usize - data_offset as usize)?;
            entries.push(Entry { name, guid, unkn, hash, attributes, content });

        }
        Ok(Msg {
            entries
        })
    }

    pub fn save(&self, name: &str) {
        let mut path = PathBuf::from("outputs").join("msg");
        let _ = std::fs::create_dir_all(&path);
        path.push(name);
        let file: File = std::fs::File::create(path).expect("nah");
        serde_json::to_writer_pretty(&file, &self.entries).unwrap();
    }
}
