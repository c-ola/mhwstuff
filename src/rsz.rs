
use crate::dersz::*;

use crate::file_ext::*;
use anyhow::{bail, Context, Result};
use serde::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::io::{Cursor, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct Extern {
    pub hash: u32,
    pub path: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypeDescriptor {
    pub hash: u32,
    pub crc: u32,
}

#[derive(Debug)]
pub struct Rsz {
    pub roots: Vec<u32>,
    pub extern_slots: HashMap<u32, Extern>,
    pub type_descriptors: Vec<TypeDescriptor>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum NodeSlot {
    None,
    Extern(String),
    Instance(RszValue),
}

#[allow(dead_code)]
impl NodeSlot {
    fn get_extern(&self) -> Result<&str> {
        match self {
            NodeSlot::Extern(path) => Ok(path),
            _ => bail!("The node slot doesn't contain extern: {:?}", self),
        }
    }

    fn get_instance(&self) -> Result<&RszValue> {
        match self {
            NodeSlot::Instance(rsz) => Ok(rsz),
            _ => bail!("The node slot doesn't contain instance: {:?}", self),
        }
    }

    fn take_instance(&mut self) -> Result<RszValue> {
        if matches!(self, NodeSlot::Instance(_)) {
            let NodeSlot::Instance(rsz) = std::mem::replace(self, NodeSlot::None) else {
                unreachable!()
            };
            Ok(rsz)
        } else {
            bail!("The node slot doesn't contain instance: {:?}", self)
        }
    }
}

impl Rsz {
    pub fn new<F: Read + Seek>(mut file: F, base: u64) -> Result<Rsz> {
        file.seek(SeekFrom::Start(base))?;
        let magic = file.read_magic()?;
        if &magic != b"RSZ\0" {
            bail!("Wrong magic for RSZ block");
        }

        let version = file.read_u32()?;
        if version != 0x10 {
            bail!("Unexpected RSZ version {}", version);
        }

        let root_count = file.read_u32()?;
        let type_descriptor_count = file.read_u32()?;
        let extern_count = file.read_u32()?;
        let padding = file.read_u32()?;
        if padding != 0 {
            bail!("Unexpected non-zero padding C: {}", padding);
        }
        let type_descriptor_offset = file.read_u64()?;
        let data_offset = file.read_u64()?;
        let string_table_offset = file.read_u64()?;

        let roots = (0..root_count)
            .map(|_| file.read_u32())
            .collect::<Result<Vec<_>>>()?;

        file.seek_noop(base + type_descriptor_offset)
            .context("Undiscovered data before type descriptor")?;

        let type_descriptors = (0..type_descriptor_count)
            .map(|_| {
                let hash = file.read_u32()?;
                let crc = file.read_u32()?;
                Ok(TypeDescriptor { hash, crc })
            })
            .collect::<Result<Vec<_>>>()?;

        if type_descriptors.first() != Some(&TypeDescriptor { hash: 0, crc: 0 }) {
            bail!("The first type descriptor should be 0")
        }

        file.seek_assert_align_up(base + string_table_offset, 16)
            .context("Undiscovered data before string table")?;

        let extern_slot_info = (0..extern_count)
            .map(|_| {
                let slot = file.read_u32()?;
                let hash = file.read_u32()?;
                let offset = file.read_u64()?;
                Ok((slot, hash, offset))
            })
            .collect::<Result<Vec<_>>>()?;

        let extern_slots = extern_slot_info
            .into_iter()
            .map(|(slot, hash, offset)| {
                file.seek_noop(base + offset)
                    .context("Undiscovered data in string table")?;
                let path = file.read_u16str()?;
                if !path.ends_with(".user") {
                    bail!("Non-USER slot string");
                }
                if hash
                    != type_descriptors
                        .get(usize::try_from(slot)?)
                        .context("slot out of bound")?
                        .hash
                {
                    bail!("slot hash mismatch")
                }
                Ok((slot, Extern { hash, path }))
            })
            .collect::<Result<HashMap<u32, Extern>>>()?;

        file.seek_assert_align_up(base + data_offset, 16)
            .context("Undiscovered data before data")?;

        let mut data = vec![];
        file.read_to_end(&mut data)?;
        
        Ok(Rsz {
            roots,
            extern_slots,
            type_descriptors,
            data,
        })
    }


    pub fn deserializev2(&self) -> Result<DeRsz> {
        let mut node_buf: Vec<NodeSlot> = vec![NodeSlot::None];
        //println!("{:?}", &self.data[0..128]);
        let mut cursor = Cursor::new(&self.data);
        let mut structs: Vec<RszValue> = Vec::new();

        for (i, &TypeDescriptor { hash, crc }) in self.type_descriptors.iter().enumerate() {
            if let Some(slot_extern) = self.extern_slots.get(&u32::try_from(i)?) {
                if slot_extern.hash != hash {
                    bail!("Extern hash mismatch")
                }
                node_buf.push(NodeSlot::Extern(slot_extern.path.clone()));
                //println!("{:?}", node_buf);
                continue;
            }

            //println!("{hash:08x}, {crc:08x}");
            let something = RszDump::parse_struct(&mut cursor, TypeDescriptor{hash, crc})?;
            //println!("{something:?}");
            structs.push(something);
        }
 
        let mut roots = Vec::new();
        for root in &self.roots {
            //println!("{root}");
            // check for object index and return that too
            match structs.get(*root as usize) {
                None => (),
                Some(obj) => roots.push(obj.to_owned())
            }
        }
        //let results = self.roots.iter().map(|root| {
        //    structs.get(*root as usize).unwrap()
        //}).collect::<Vec<RszStruct<RszType>>>();

        let mut leftover = vec![];
        cursor.read_to_end(&mut leftover)?;
        if !leftover.is_empty() {
            //bail!("Left over data {leftover:?}");
            println!("Left over data {leftover:?}");
        }

        Ok(DeRsz{
            roots,
            structs
        })
    }

    /*pub fn verify_crc(&self, crc_mismatches: &mut BTreeMap<&str, u32>, print_all: bool) {
        for td in &self.type_descriptors {
            if let Some(type_info) = RSZ_TYPE_MAP.get(&td.hash) {
                if print_all
                    || (!type_info.versions.contains_key(&td.crc) && !type_info.versions.is_empty())
                {
                    crc_mismatches.insert(type_info.symbol, td.crc);
                }
            }
        }
    }*/
}


#[derive(Debug, Serialize, Clone)]
#[allow(dead_code)]
pub enum ExternUser<T> {
    Path(String),
    Loaded(T),
}

/*impl<T: FromUser> ExternUser<T> {
    pub fn load<'a>(
        &'a mut self,
        pak: &'_ mut crate::pak::PakReader<impl Read + Seek>,
        version_hint: Option<u32>,
    ) -> Result<&'a mut T> {
        match self {
            ExternUser::Path(path) => {
                let index = pak.find_file(path)?;
                let file = pak.read_file(index)?;
                let user = crate::user::User::new(Cursor::new(file))?;
                *self = ExternUser::Loaded(user.rsz.deserialize_single(version_hint)?);
                if let ExternUser::Loaded(t) = self {
                    Ok(t)
                } else {
                    unreachable!()
                }
            }
            ExternUser::Loaded(t) => Ok(t),
        }
    }

    pub fn unwrap(&self) -> &T {
        match self {
            ExternUser::Path(_) => {
                panic!("ExternUser not loaded")
            }
            ExternUser::Loaded(t) => t,
        }
    }
}*/

/*impl<T> FieldFromRsz for ExternUser<T> {
    fn field_from_rsz(rsz: &mut RszDeserializer) -> Result<Self> {
        rsz.cursor.seek_align_up(4)?;
        let extern_path = rsz.get_extern()?.to_owned();
        Ok(ExternUser::Path(extern_path))
    }
}*/

/*impl<T> FieldFromRsz for Option<ExternUser<T>> {
    fn field_from_rsz(rsz: &mut RszDeserializer) -> Result<Self> {
        rsz.cursor.seek_align_up(4)?;
        let extern_path = rsz.get_extern_opt()?;
        Ok(extern_path.map(|p| ExternUser::Path(p.to_owned())))
    }
}*/
