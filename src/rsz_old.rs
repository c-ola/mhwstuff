use core::fmt;
use std::{any::Any, collections::HashMap, io::{Cursor, Result}, rc::Rc, sync::OnceLock};

use bincode::deserialize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::byte_reader::BytesFile;

#[derive(Debug)]
#[allow(dead_code)]
struct Extern {
    hash: u32,
    path: String,
}

#[derive(Debug, PartialEq)]
struct TypeDescriptor {
    hash: u32,
    crc: u32,
}

impl fmt::Display for TypeDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TypeDescriptor{{ hash: {}, crc: {:8x} }}",
            self.hash, self.crc
        )
    }
}

#[derive(Debug)]
pub struct Rsz {
    roots: Vec<u32>,
    extern_slots: HashMap<u32, Extern>,
    type_descriptors: Vec<TypeDescriptor>,
    data: Vec<u8>
}

impl fmt::Display for Rsz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}, {:?}\n", self.roots, self.extern_slots)?;
        for i in &self.type_descriptors {
            write!(f, "{}\n", i)?;
        }
        Ok(())
    }
}

impl Rsz {
    pub fn new(data: &mut BytesFile, base: usize) -> Result<Rsz> {
        data.index = base;
        let magic = data.readn::<u8, 4>()?;
        if &magic != b"RSZ\0" {
            panic!("Wrong magic for RSZ block");
        }

        let version = data.read::<u32>()?;
        if version != 0x10 {
            panic!("Unexpected RSZ version {}", version);
        }

        let root_count = data.read::<u32>()?;
        let type_descriptor_count = data.read::<u32>()?;
        let extern_count = data.read::<u32>()?;
        let padding = data.read::<u32>()?;
        if padding != 0 {
            panic!("Unexpected non-zero padding C: {}", padding);
        }
        let type_descriptor_offset = data.read::<u64>()?;
        let data_offset = data.read::<u64>()?;
        let string_table_offset = data.read::<u64>()?;

        let roots = (0..root_count)
            .map(|_| data.read::<u32>())
            .collect::<Result<Vec<_>>>()?;

        data.index = base as usize + type_descriptor_offset as usize;

        let type_descriptors = (0..type_descriptor_count)
            .map(|_| {
                let hash = data.read::<u32>()?;
                let crc = data.read::<u32>()?;
                Ok(TypeDescriptor { hash, crc })
            })
            .collect::<Result<Vec<_>>>()?;

        if type_descriptors.first() != Some(&TypeDescriptor { hash: 0, crc: 0 }) {
            panic!("The first type descriptor should be 0")
        }


        data.index = base as usize + string_table_offset as usize;

        let extern_slot_info = (0..extern_count)
            .map(|_| {
                let slot = data.read::<u32>()?;
                let hash = data.read::<u32>()?;
                let base = data.read::<u64>()?;
                Ok((slot, hash, base))
            })
            .collect::<Result<Vec<_>>>()?;

        let extern_slots = extern_slot_info
            .into_iter()
            .map(|(slot, hash, base)| {
                let path = data.read_utf16(base as usize + base as usize)?;
                if !path.ends_with(".user") {
                    panic!("Non-USER slot string");
                }
                let t = type_descriptors.get(slot as usize).expect("Invalid Type Descriptor Slot");
                if hash != t.hash {
                    panic!("slot hash mismatch")
                }
                Ok((slot, Extern { hash, path }))
            })
            .collect::<Result<HashMap<u32, Extern>>>()?;

        data.index = base as usize + data_offset as usize;
        let rsz_data = data.read_bytes_to_vec(data.len() - data.index)?;

        Ok(Rsz {
            roots,
            extern_slots,
            type_descriptors,
            data: rsz_data,
        })
    }

    pub fn deserialize(&mut self) -> Result<()> {
        let file = std::fs::File::create("item.json")?;
        println!("{}", size_of::<user_data_ItemData_cData>());


        for descriptor in &self.type_descriptors {
            // for each descriptor
            // get type information using hashmap
            let rsz_type = rsz_type_map().get(&descriptor.hash).unwrap();

            // then read into a dyn Any 
            let _ = (rsz_type.deserializer)(&self.data).expect("Could not deserialize");


            println!("{rsz_type:?}");
            let data: user_data_ItemData_cData = rsz_type.deserialize(&self.data).expect("error deserializing");
            serde_json::to_writer_pretty(&file, &data)?;
            self.data.drain(0..size_of::<user_data_ItemData_cData>() + 8);

        }
        Ok(())
    } 
}


pub struct RszData {
    data: Vec<u8>,
    cursor: Cursor<>
}


#[derive(Debug)]
pub struct AnyRsz {
    any: Rc<dyn Any>,
    type_info: RszType,
}

#[derive(Debug)]
pub struct RszType {
    deserializer: fn(Vec<u8>) -> Result<AnyRsz>,
    to_json: fn(&dyn Any) -> Result<String>,
}

pub fn rsz_type_map() -> &'static HashMap<u32, RszType> {
    static HASHMAP: OnceLock<HashMap<u32, RszType>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let mut m = HashMap::new();
        let rsz_type = RszType {
            deserializer: rsz_deserialize::<user_data_ItemData_cData>,
            to_json: rsz_to_json::<user_data_ItemData_cData>
        };
        m.insert(0x5a8f4fb8, rsz_type);
        m
    })
}

fn rsz_deserialize<'a, T: 'static + Serialize + Deserialize<'a>>(_data: Vec<u8>) -> Result<AnyRsz> {
    let deserialized = T::deserialize(&_data);
    Ok(AnyRsz {deserialized, })
}

fn rsz_to_json<T: 'static + Serialize>(_data: &dyn Any) -> Result<String> {

    Ok("".to_string())
}

#[allow(non_snake_case, unused, non_camel_case_types)]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct user_data_ItemData_cData {
    _Index: i32, // 0
    _ItemId: i32, // 4
    _RawName: Guid, // 8
    _RawExplain: Guid, // 24
    _align1: [u8; 2],
    _SortId: i16, // 40
    _Type: i32, // 44
    _TextType: i32, // 48
    _IconType: i32, // 52
    _EquipIcon: i32, // 56
    _IconColor: i32, // 60
    _AddIconType: i32, // 64
    _Rare: i32, // 68
    _MaxCount: i16, // 70
    _OtomoMax: i16, // 72 
    _EnableOnRaptor: bool,// 74
    _align2: [u8; 3], // 75
    _SellPrice: i32, // 75 -> 76
    _BuyPrice: i32, // 80
    _Fix: bool, // 84
    _Shikyu: bool, // 85
    _Eatable: bool, // 86
    _Window: bool, // 87
    _Infinit: bool, // 88
    _Heal: bool, // 89
    _Battle: bool, // 90
    _Special: bool, // 91
    _ForMoney: bool, // 92
    _OutBox: bool, // 93
    _NonLevelShell: bool, // 94
    _align3: [u8; 1], // 75
    _GetRank: i32, // 95 -> 96
}

#[allow(unused)]
pub struct Int2 {
    x: i32,
    y: i32,
}

#[allow(unused)]
pub struct Int3 {
    x: i32,
    y: i32,
    z: i32,
}

#[allow(unused)]
pub struct Uint2 {
    x: u32,
    y: u32,
}

#[allow(unused)]
pub struct Size {
    size: usize,
}

#[allow(unused)]
pub struct Float2 {
    x: f32,
    y: f32,
}

#[allow(unused)]
pub struct Float3 {
    x: f32,
    y: f32,
    z: f32,
}

#[allow(unused)]
pub struct Float4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}


#[allow(unused)]
pub struct F8 {
    x: u8
}

#[allow(unused)]
pub struct Color(u8, u8, u8, u8);

#[allow(unused)]
type Data = Vec<u8>;
#[allow(unused)]
struct Resource;
#[allow(unused)]
struct Object;
#[allow(unused)]
struct GameObjectRef;

#[allow(unused)]



#[derive(serde::Deserialize, Debug)]
struct Guid {
    id: [u8; 16]
}

impl Serialize for Guid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let uuid = Uuid::from_bytes(self.id).to_string();
        let state = serializer.serialize_str(&uuid);
        state
    }

}

#[allow(unused)]
struct UserData;

