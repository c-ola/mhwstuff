pub mod bc7;

extern crate image;

use bc7::{
    Bc1Unorm, Bc3Unorm, Bc4Unorm, Bc5Unorm, BitField, R8G8B8A8Unorm, R8G8Unorm, R8Unorm, TexCodec
};
use byteorder::{self, LittleEndian, ReadBytesExt};
use clap::Parser;
use core::str;
use std::time::SystemTime;
use image::RgbaImage;
use std::path::Path;
use std::{
    fmt,
    fs::{self},
    io::Result,
    u16,
};

#[derive(Debug)]
struct BytesFile {
    data: Vec<u8>,
    pub index: usize,
}

impl BytesFile {
    fn new(file_name: String) -> Result<BytesFile> {
        let data = fs::read(file_name)?;
        Ok(BytesFile { data, index: 0 })
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

    pub fn seek(&mut self, num: usize) {
        self.index = num;
    }
}

trait ReadBytesTyped: Sized {
    fn read(file: &mut BytesFile) -> Result<Self>;
    fn readn<const N: usize>(file: &mut BytesFile) -> Result<[Self; N]>;
}

impl ReadBytesTyped for u64 {
    fn read(file: &mut BytesFile) -> Result<u64> {
        let mut data = &file.data[file.index..file.index + 8];
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
        let res = (&file.data[file.index..file.index + 4]).read_u32::<LittleEndian>()?;
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
        let res = (&file.data[file.index..file.index + 2]).read_u16::<LittleEndian>()?;
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

#[derive(Debug, Clone)]
struct Tex {
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

impl Tex {
    fn new(file_name: String) -> Result<Tex> {
        let mut data = BytesFile::new(file_name)?;

        let magic = data.readn::<u8, 4>()?;
        let m = ['T', 'E', 'X', '\0'];
        for i in 0..4 {
            if magic[i] != m[i] as u8 {
                panic!("Invalid Magic");
            }
        }
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
        let _x = data.read_bytes_to_vec(6);
        println!("x0: {_x:?}");

        impl fmt::Display for TexInfo {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "\tOffset: {:#010x}, pitch: {:#010x}, Len: {:#010x}",
                    self.offset, self.pitch, self.len
                )
            }
        }

        let mut tex_infos = Vec::new();

        for i in 0..tex_count {
            for _j in 0..mipmap_count {
                println!("texture_{i}-{_j}");
                let offset = data.read::<u64>()? as usize;
                let pitch = data.read::<u32>()? as usize;
                let len = data.read::<u32>()? as usize;

                tex_infos.push(TexInfo { offset, pitch, len });
                println!("{}", tex_infos[_j as usize]);
            }
        }

        let textures = tex_infos
            .clone()
            .into_iter()
            .map(|tex_info| {
                println!("{tex_info:?}");
                data.seek(tex_info.offset);
                let mut buf: Vec<u8> = Vec::new();
                let _rows = tex_info.len/tex_info.pitch;
                //for _ in 0..rows {
                buf.extend(data.read_bytes_to_vec(tex_info.len).unwrap());
                    //buf.extend(vec![0; tex_info.pitch])
                //}
                buf
            })
            .collect::<Vec<_>>();
        let tex = Tex {
            width: width as u32,
            height: height as u32,
            format,
            layout,
            tex_infos,
            textures: textures.clone(),
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
            0x62 | 0x63 => bc7::Bc7Unorm::decode_image(
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
    let now = SystemTime::now();
    let args = Args::parse();
    let tex = Tex::new(args.file_name.clone())?;
    //println!("{tex:?}");
    //for i in 0..tex.tex_infos.len() {
    let rgba = tex.to_rgba(0)?;
    println!("{}", rgba.data.len());
    let name = format!("{}_{}.png", args.file_name, 0);
    println!("saving to {name}");
    let _ = image::save_buffer(
        &Path::new(&name),
        &rgba.data,
        rgba.width,
        rgba.height,
        image::ExtendedColorType::Rgba8,
    );

    //}
    println!("Time taken: {} ms", now.elapsed().unwrap().as_millis());
    Ok(())
}
