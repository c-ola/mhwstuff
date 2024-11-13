pub mod bc7;

extern crate image;

use core::str;
use std::path::Path;
use std::{fmt, fs::{self}, io::Result, u16};
use bc7::{bc7_decompress_block, BitField, TexCodec};
use byteorder::{self, LittleEndian, ReadBytesExt};
use clap::Parser;
use image::RgbaImage;

#[derive(Debug)]
struct Tex {
    width: u32,
    height: u32,
    format: u32,
    layout: u32,
    textures: Vec<Vec<u8>>,
}

#[derive(Debug)]
struct BytesFile {
    data: Vec<u8>,
    pub index: usize,
}

enum SizeType {
    U8(usize),
    U16(usize),
    U32(usize),
    U64(usize)
}

impl BytesFile {
    fn new(file_name: String) -> Result<BytesFile> {
        let data = fs::read(file_name)?;
        Ok(BytesFile {
            data,
            index: 0
        })
    }

    pub fn read<T: ReadBytesTyped>(&mut self) -> Result<T> {
        T::read(self)
    }

    pub fn readn<T: ReadBytesTyped, const N: usize>(&mut self) -> Result<[T; N]> {
        T::readn::<N>(self)
    }
    
    pub fn read_bytes_to_vec(&mut self, num: usize) -> Result<Vec<u8>> {
        let mut data = vec![0; num];
        for i in 0..num {
            let byte = u8::read(self)?;
            data[i] = byte;
        }
        Ok(data)
    }

    pub fn readnvec(&mut self, sizes: Vec<SizeType>) -> Result<Vec<u64>>{
        let mut res = Vec::new();
        for size_type in sizes {
            match size_type {
                SizeType::U8(n) => readnvec::<u8>(self, n)?.iter().for_each(|x| res.push(*x as u64)),
                SizeType::U16(n) => readnvec::<u16>(self, n)?.iter().for_each(|x| res.push(*x as u64)),
                SizeType::U32(n) => readnvec::<u32>(self, n)?.iter().for_each(|x| res.push(*x as u64)),
                SizeType::U64(n) => readnvec::<u64>(self, n)?.iter().for_each(|x| res.push(*x as u64))
            };
        }
        Ok(res)
    }

    pub fn seek(&mut self, num: usize) {
        self.index = num;
    }
}

fn readnvec<T: ReadBytesTyped>(file: &mut BytesFile, num: usize) -> Result<Vec<T>> {
    let mut data = Vec::new();
    for _ in 0..num {
        data.push(file.read::<T>()?);
    }
    Ok(data)
}

trait ReadBytesTyped: Sized {
    fn read(file: &mut BytesFile) -> Result<Self>;
    fn readn<const N: usize>(file: &mut BytesFile) -> Result<[Self; N]>;
}

impl ReadBytesTyped for u64 {
    fn read(file: &mut BytesFile) -> Result<u64> {
        let mut data = &file.data[file.index..file.index+8];
        let res = data.read_u64::<LittleEndian>()?;
        file.seek(file.index + 8);
        Ok(res)
    }
    
    fn readn<const N: usize>(file: &mut BytesFile) -> Result<[u64; N]> {
        let mut data = [0u64; N];
        for i in 0..N {
            data[i] = file.read::<u64>()?;
        }
        Ok(data)
    }
}

impl ReadBytesTyped for u32 {
    fn read(file: &mut BytesFile) -> Result<u32> {
        let res = (&file.data[file.index..file.index+4]).read_u32::<LittleEndian>()?;
        file.seek(file.index + 4);
        Ok(res)
    }
    
    fn readn<const N: usize>(file: &mut BytesFile) -> Result<[u32; N]> {
        let mut data = [0u32; N];
        for i in 0..N {
            data[i] = file.read::<u32>()?;
        }
        Ok(data)
    }
}

impl ReadBytesTyped for u16 {
    fn read(file: &mut BytesFile) -> Result<u16> {
        let res = (&file.data[file.index..file.index+2]).read_u16::<LittleEndian>()?;
        file.seek(file.index + 2);
        Ok(res)
    }

    fn readn<const N: usize>(file: &mut BytesFile) -> Result<[u16; N]> {
        let mut data = [0u16; N];
        for i in 0..N {
            data[i] = file.read::<u16>()?;
        }
        Ok(data)
    }
}

impl ReadBytesTyped for u8 {
    fn read(file: &mut BytesFile) -> Result<u8> {
        let res = file.data[file.index];
        file.seek(file.index + 1);
        Ok(res)
    }

    fn readn<const N: usize>(file: &mut BytesFile) -> Result<[u8; N]> {
        let mut data = [0u8; N];
        for i in 0..N {
            data[i] = file.read::<u8>()?;
        }
        Ok(data)
    }
}


//      32-MAGIC, 32-VERSION, 16-w, 16-h, 16-depth, 24-texcount, 8



impl Tex {
    fn new(file_name: String) -> Result<Tex> {
        let mut data = BytesFile::new(file_name)?;
        
        let magic = data.readn::<u8, 4>()?;
        let name = str::from_utf8(&magic);
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
        let _x = data.readnvec(vec![SizeType::U16(3)])?;
        println!("x0: {_x:?}");
        
        #[derive(Clone)]
        struct TexInfo {
            offset: usize,
            pitch: usize,
            len: usize,
        }

        impl fmt::Display for TexInfo {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f,
                       "\tOffset: {:#010x}, pitch: {:#010x}, Len: {:#010x}",
                       self.offset, self.pitch, self.len)
            }
        }

        //0x24320 - 0x223a0
        let mut tex_infos = Vec::new();

        for i in 0..tex_count {
            for _j in 0..mipmap_count {
                println!("texture_{i}-{_j}");
                let offset = data.read::<u64>()? as usize;
                //let val = data.read::<u16>()? as usize;
                //println!("{val}");
                //data.index += 4;
                let pitch = data.read::<u32>()? as usize;
                //data.index += 2;
                let len = data.read::<u32>()? as usize;
                //data.index += 2;

                tex_infos.push(TexInfo {
                    offset,
                    pitch,
                    len,
                });
                println!("{}", tex_infos[_j as usize]);
            }
        }

        let big_val = if version == 240701001 {
            data.read::<u64>()? as usize
        } else {
            1
        };

        // scale = 0x100
        // len = 0x100
        // pitch = 0x100
        // p/s = 1
        // l/s = 1
        println!("{big_val:#010x}");
        let textures = tex_infos.clone()
            .into_iter()
            .enumerate()
            .map(|(index, tex_info)|{
                println!("{index}");
                let mut buf: Vec<u8> = Vec::new();
                match version {
                    240701001 => {
                        data.seek(tex_info.offset & 0xffff);
                    },
                    28 => data.seek(tex_info.offset),
                    _ => panic!("Invalid version number"),
                }
                for j in 0..tex_info.len/tex_info.pitch {
                    data.index = (tex_info.offset & 0xffff) + j * 8;
                    for i in 0..tex_info.len/tex_info.pitch {
                        //println!("{:#010x}, {:#010x}",tex_info.len/tex_info.pitch * j + i, data.index);
                        if data.index + 8 >= data.data.len() {
                            break
                        }
                        buf.extend(data.read_bytes_to_vec(8).unwrap());
                        data.index += tex_info.pitch - 8;
                    }
                }
                buf
            }).collect::<Vec<_>>();
        for t in &textures {
            print!("[");
            for byte in t {
                //print!("{byte:#04x}, ");
            }
            println!("]");
        }
        //println!("{:?}", textures);

        let tex = Tex {
            width: width as u32,
            height: height as u32,
            format,
            layout,
            textures
        };
        Ok(tex)
    }

    pub fn to_rgba(self, index: usize) -> Result<RGBAImage> {
        let texture: Vec<u8> = self.textures[index].clone();
        let swizzle = "rgba";
        let width = self.width as usize;
        let height = self.height as usize;
        let mut data = vec![0; width * height * 4];

        let writer = |x: usize, y: usize, v: [u8; 4]| {
            let i = (x + y * (width)) * 4;
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
            /*0x1C | 0x1D => R8G8B8A8Unorm::decode_image,
              0x31 => R8G8Unorm::decode_image,
              0x3D => R8Unorm::decode_image,*/
            0x47 | 0x48 => bc7::Bc1Unorm::decode_image(&texture, self.width as usize, self.height as usize, self.layout, writer),
            //0x4D | 0x4E => Bc3Unorm::decode_image,
            /*0x50 => Bc4Unorm::decode_image,
              0x53 => Bc5Unorm::decode_image,*/
            0x62 | 0x63 => bc7::Bc7Unorm::decode_image(&texture, 256, 256, self.layout, writer),
            /*0x402 | 0x403 => Astc::<4, 4>::decode_image,
            0x405 | 0x406 => Astc::<5, 4>::decode_image,
            0x408 | 0x409 => Astc::<5, 5>::decode_image,
            0x40B | 0x40C => Astc::<6, 5>::decode_image,
            0x40E | 0x40F => Astc::<6, 6>::decode_image,
            0x411 | 0x412 => Astc::<8, 5>::decode_image,
            0x414 | 0x415 => Astc::<8, 6>::decode_image,
            0x417 | 0x418 => Astc::<8, 8>::decode_image,
            0x41A | 0x41B => Astc::<10, 5>::decode_image,
            0x41D | 0x41E => Astc::<10, 6>::decode_image,
            0x420 | 0x421 => Astc::<10, 8>::decode_image,
            0x423 | 0x424 => Astc::<10, 10>::decode_image,
            0x426 | 0x427 => Astc::<12, 10>::decode_image,
            0x429 | 0x42A => Astc::<12, 12>::decode_image*/
            x => panic!("unsupported format {:08X}", x),
        };
        Ok(RGBAImage {
            data,
            width: self.width,
            height: self.height
        })
    }
}

struct RGBAImage {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let tex = Tex::new(args.file_name)?;
    //println!("{tex:?}");
    let rgba = tex.to_rgba(0)?;
    println!("{}", rgba.data.len());
    let _ = image::save_buffer(&Path::new("image.png"), &rgba.data, rgba.width, rgba.height, image::ExtendedColorType::Rgba8);
    Ok(())
}
