/*use core::fmt;
use std::io::{Cursor, Read, Result, Seek};

use crate::{byte_reader::BytesFile, rsz::Rsz};

#[derive(Debug)]
pub struct UserChild {
    pub hash: u32,
    pub name: String,
}

#[derive(Debug)]
pub struct User {
    pub resource_names: Vec<String>,
    pub children: Vec<UserChild>,
    pub rsz: Rsz,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "res_names: {:?}, children: {:?}, rsz: {:?}",
            self.resource_names, self.children, self.rsz
        )
    }
}

impl User {
    pub fn new(file_name: String) -> Result<User> {
        let mut data = BytesFile::new(file_name)?;
        let magic = data.readn::<u8, 4>()?;
        if &magic != b"USR\0" {
            panic!("Invalid Magic");
        }
        let resource_count = data.read::<u32>()?;
        let child_count = data.read::<u32>()?;
        let padding = data.read::<u32>()?;
        if padding != 0 {
            panic!("Unexpected non-zero padding A: {}", padding);
        }
        let resource_list_offset = data.read::<u64>()?;
        let child_list_offset = data.read::<u64>()?;
        let rsz_offset = data.read::<u64>()?;
        println!("{resource_count} {child_count} {padding} {resource_list_offset} {child_list_offset}");

        data.index = resource_list_offset as usize;
        let resource_name_offsets = (0..resource_count)
            .map(|_| data.read::<u64>())
            .collect::<Result<Vec<_>>>()?;

        data.index = child_list_offset as usize;
        let child_info = (0..child_count)
            .map(|_| {
                let hash = data.read::<u32>()?;
                let padding = data.read::<u32>()?;
                if padding != 0 {
                    panic!("ChildInfo non-zero padding {}", padding);
                }
                let name_offset = data.read::<u64>()?;
                Ok((hash, name_offset))
            })
            .collect::<Result<Vec<_>>>()?;

        let resource_names = resource_name_offsets
            .into_iter()
            .map(|resource_name_offset| {
                let name = data.read_utf16(resource_name_offset as usize)?;
                if name.ends_with(".user") {
                    panic!("USER resource");
                }
                Ok(name)
            })
            .collect::<Result<Vec<_>>>()?;

        let children = child_info
            .into_iter()
            .map(|(hash, name_offset)| {
                let name = data.read_utf16(name_offset as usize)?;
                if !name.ends_with(".user") {
                    panic!("Non-USER child");
                }
                Ok(UserChild { hash, name })
            })
            .collect::<Result<Vec<_>>>()?;

        let rsz = Rsz::new(Cursor::new(data), rsz_offset).unwrap();
        Ok(User {
            resource_names,
            children,
            rsz,
        })
    }
}*/

use crate::file_ext::*;
use crate::rsz::*;
use anyhow::{bail, Context, Result};
use std::io::{Read, Seek};

#[derive(Debug)]
pub struct UserChild {
    pub hash: u32,
    pub name: String,
}

#[derive(Debug)]
pub struct User {
    pub resource_names: Vec<String>,
    pub children: Vec<UserChild>,
    pub rsz: Rsz,
}

impl User {
    pub fn new<F: Read + Seek>(mut file: F) -> Result<User> {
        let magic = file.read_magic()?;
        if &magic != b"USR\0" {
            bail!("Wrong magic for USER file");
        }
        let resource_count = file.read_u32()?;
        let child_count = file.read_u32()?;
        let padding = file.read_u32()?;
        if padding != 0 {
            bail!("Unexpected non-zero padding A: {}", padding);
        }
        let resource_list_offset = file.read_u64()?;
        let child_list_offset = file.read_u64()?;
        let rsz_offset = file.read_u64()?;

        file.seek_assert_align_up(resource_list_offset, 16)
            .context("Undisconvered data before resource list")?;
        let resource_name_offsets = (0..resource_count)
            .map(|_| file.read_u64())
            .collect::<Result<Vec<_>>>()?;

        file.seek_assert_align_up(child_list_offset, 16)
            .context("Undisconvered data before child list")?;
        let child_info = (0..child_count)
            .map(|_| {
                let hash = file.read_u32()?;
                let padding = file.read_u32()?;
                if padding != 0 {
                    bail!("ChildInfo non-zero padding {}", padding);
                }
                let name_offset = file.read_u64()?;
                Ok((hash, name_offset))
            })
            .collect::<Result<Vec<_>>>()?;

        let resource_names = resource_name_offsets
            .into_iter()
            .map(|resource_name_offset| {
                file.seek_noop(resource_name_offset)
                    .context("Undiscovered data in resource names")?;
                let name = file.read_u16str()?;
                if name.ends_with(".user") {
                    bail!("USER resource");
                }
                Ok(name)
            })
            .collect::<Result<Vec<_>>>()?;

        let children = child_info
            .into_iter()
            .map(|(hash, name_offset)| {
                file.seek_noop(name_offset)
                    .context("Undiscovered data in child info")?;
                let name = file.read_u16str()?;
                if !name.ends_with(".user") {
                    bail!("Non-USER child");
                }
                Ok(UserChild { hash, name })
            })
            .collect::<Result<Vec<_>>>()?;

        let rsz = Rsz::new(file, rsz_offset)?;

        Ok(User {
            resource_names,
            children,
            rsz,
        })
    }
}


