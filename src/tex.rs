use crate::byte_reader::BytesFile;
use crate::bitfield::BitField;
use crate::compression::{
    Bc1Unorm, Bc3Unorm, Bc4Unorm, Bc5Unorm, Bc7Unorm, R8G8B8A8Unorm, R8G8Unorm, R8Unorm, TexCodec
};

use std::{fmt, io::Result};

pub struct RGBAImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Tex {
    width: u32,
    height: u32,
    format: u32,
    layout: u32,
    tex_infos: Vec<TexInfo>,
    textures: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
struct TexInfo {
    offset: usize,
    pitch: usize,
    len: usize,
}

impl fmt::Display for TexInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\tOffset: {:#010x}, pitch: {:#010x}, Len: {:#010x}",
            self.offset, self.pitch, self.len
        )
    }
}

impl Tex {
    pub fn new(file_name: String) -> Result<Tex> {
        let mut data = BytesFile::new(file_name)?;

        let magic = data.readn::<u8, 4>()?;
        let m = ['T', 'E', 'X', '\0'];
        for i in 0..4 {
            if magic[i] != m[i] as u8 {
                panic!("Invalid Magic");
            }
        }
        let name = core::str::from_utf8(&magic);
        let version = data.read::<u32>()?;
        println!("name: {:?}", name.clone());
        println!("version: {:?}", version);

        let width = data.read::<u16>()?;
        let height = data.read::<u16>()?;
        let depth = data.read::<u16>()?;
        println!("dims: {width}, {height}, {depth}");

        let counts = data.read::<u16>()?;
        let (tex_count, mipmap_count) = counts.bit_split((12, 4));
        println!("counts: {counts},texs: {tex_count}, mipmaps: {mipmap_count}");

        let format = data.read::<u32>()?;
        let layout = data.read::<u32>()?;
        println!("format: {format:#010x}, layout: {layout:#010x}");

        let _a = data.read::<u32>()?;
        let _b = data.read::<u32>()?;
        println!("idk a b: {_a:#010x}, {_b:#010x}");

        let super_dims = data.read::<u16>()?;
        println!("super_dims: {super_dims:#06x}");
        let _x = data.read_bytes_to_vec(6);
        println!("x0: {_x:?}");

        let mut tex_infos = Vec::new();
        
        let mut decompressed_size = 0; 
        for i in 0..tex_count {
            for _j in 0..mipmap_count {
                println!("texture_{i}-{_j}");
                let offset = data.read::<u64>()? as usize;
                let pitch = data.read::<u32>()? as usize;
                let len = data.read::<u32>()? as usize;
                decompressed_size += len;

                tex_infos.push(TexInfo { offset, pitch, len });
                println!("{}", tex_infos[_j as usize]);
            }
        }

        
        #[derive(Debug)]
        struct GDefSection {
            compressed_size: u32,
            offset: u32,
        }
        let mut total_size = 0;
        let gdef_sections = (0..mipmap_count * tex_count).into_iter().map(|_| {
            let compressed_size = data.read::<u32>()?;
            let offset = data.read::<u32>()?;
            total_size += compressed_size;
            Ok(GDefSection {
                compressed_size,
                offset
            })
        }).collect::<Result<Vec<_>>>()?;


        
        let base = tex_infos[0].offset + mipmap_count as usize * tex_count as usize * 8;
        println!("base {}", base);
        //let mut decomp_data = Cursor::new(decomp_data);
        //let mut out_buf = Vec::with_capacity(total_size + 32);
        let mut bytes_read = 0;
        let textures = tex_infos
            .iter()
            .enumerate()
            .map(|(i, tex_info)| {
                let section = &gdef_sections[i];
                let out_size = tex_info.len as usize;
                let in_size = section.compressed_size as usize;
                println!("{tex_info:?}");
                println!("{section:?}");
                data.index = base + section.offset as usize;
                let in_buf = data.read_bytes_to_vec(in_size).unwrap();
                let mut out_buf: Vec<u8> = Vec::new();
                out_buf.resize(out_size, 0);
                println!("in_size {}, out_size {}", in_size, out_size);
                match libdeflater::GDeflateDecompressor::gdeflate_decompress(&in_buf, &mut out_buf) {
                    Ok(x) => {
                        bytes_read += x;
                        println!("bytes read: {x}");
                    },
                    Err(e) => panic!("Error in gdeflate decompression: {e}"),
                }
                out_buf
            }).collect::<Vec<_>>();


        assert_eq!(bytes_read, decompressed_size, "Bytes read should be the same as the decompressed size");
        let tex = Tex {
            width: width as u32,
            height: height as u32,
            format,
            layout,
            tex_infos,
            textures,
        };
        Ok(tex)
    }

    pub fn to_rgba(&self, index: usize) -> Result<RGBAImage> {
        let texture: Vec<u8> = self.textures[index].clone();
        let tex_info = self.tex_infos[index].clone();
        let swizzle = "rgba";
        
        let s_pitch = tex_info.pitch as usize / 4;
        let padding = ((self.width as usize / s_pitch + 1) * s_pitch) % self.width as usize;

        let (width, height) = if padding != 0 && self.width / padding as u32 != 2 { 
            (self.width as usize + padding, self.height as usize * (self.width as usize + padding) / self.width as usize)
        } else {
            //(self.width as usize + padding, self.height as usize * (self.width as usize + padding) / self.width as usize)
            (self.width as usize, self.height as usize)
        };
        let mut data = vec![0; width as usize * height as usize * 4];

        println!("w{}, h{}, pad:{padding}", width, height);
        let writer = |x: usize, y: usize, v: [u8; 4]| {
            let i = (x + y * (width as usize)) * 4;
            let dest = &mut data[i..][..4];
            for (dest, &code) in dest.iter_mut().zip(swizzle.as_bytes()) {
                *dest = match code {
                    b'r' | b'x' => v[0],
                    b'g' | b'y' => v[1],
                    b'b' | b'z' => v[2],
                    b'a' | b'w' => v[3],
                    b'0' => 0,
                    b'1' => 255,
                    b'n' => 0,
                    _ => 0,
                }
            }

            if let Some(n) = swizzle.as_bytes().iter().position(|&c| c == b'n') {
                let mut l: f32 = dest
                    .iter()
                    .map(|&x| {
                        let x = x as f32 / 255.0 * 2.0 - 1.0;
                        x * x
                    })
                    .sum();
                if l > 1.0 {
                    l = 1.0
                }
                let z = (((1.0 - l).sqrt() + 1.0) / 2.0 * 255.0).round() as u8;
                dest[n] = z;
            }
        };
        match self.format {
            0x1C | 0x1D => R8G8B8A8Unorm::decode_image(
                &texture,
                width as usize,
                height as usize,
                self.layout,
                writer,
            ),
            0x31 => R8G8Unorm::decode_image(
                &texture,
                width as usize,
                height as usize,
                self.layout,
                writer,
            ),
            0x3D => R8Unorm::decode_image(
                &texture,
                width as usize,
                height as usize,
                self.layout,
                writer,
            ),
            0x47 | 0x48 => Bc1Unorm::decode_image(
                &texture,
                width as usize,
                height as usize,
                self.layout,
                writer,
            ),
            0x4D | 0x4E => Bc3Unorm::decode_image(
                &texture,
                width as usize,
                height as usize,
                self.layout,
                writer,
            ),
            0x50 => Bc4Unorm::decode_image(
                &texture,
                width as usize,
                height as usize,
                self.layout,
                writer,
            ),
            0x53 => Bc5Unorm::decode_image(
                &texture,
                width as usize,
                height as usize,
                self.layout,
                writer,
            ),
            0x62 | 0x63 => Bc7Unorm::decode_image(
                &texture,
                width as usize,
                height as usize,
                self.layout,
                writer,
            ),
            x => panic!("unsupported format {:08X}", x),
        };

        // remove padding around the image
        let mut data2 = vec![0; self.width as usize * self.height as usize * 4]; 
        for i in 0..self.height as usize {
            for j in 0..self.width as usize * 4 {
                data2[i * self.width as usize * 4 + j] = data[i * width * 4 + j];
            }
        }

        Ok(RGBAImage {
            data: data2,
            width: self.width as u32,
            height: self.height as u32,
        })
    }
}
