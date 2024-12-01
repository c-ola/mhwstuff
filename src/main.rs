mod user;
mod compression;
mod msg;
mod byte_reader;
mod tex;
mod rsz;
mod file_ext;
mod align;
mod bitfield;
mod hash;
mod pak;
mod suffix;


extern crate image;

use clap::Parser;
use msg::Msg;
use tex::Tex;
use user::User;
use std::fs::{self, File};
use std::time::SystemTime;
use image::RgbaImage;
use std::path::Path;
use std::io::{Cursor, Read};
use anyhow::*;
use rsz::*;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_name: String,
}

fn main() -> Result<()> {
    let now = SystemTime::now();
    let args = Args::parse();
    println!("{:?}", args);
    if args.file_name.ends_with("msg.23") {
        let msg = Msg::new(args.file_name.clone())?;
        msg.save();
    } else if args.file_name.ends_with("user.3") {
        //let mut data = vec![];
        //let mut file = File::open(args.file_name.clone())?;
        //file.read_to_end(&mut data);
        let nodes = User::new(File::open(args.file_name.clone())?)?.rsz
            .deserialize(Some(0)).unwrap();
        for node in nodes {
            println!("{}", node.to_json().unwrap());
        }

    } else {
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
    }
    //}
    println!("Time taken: {} ms", now.elapsed().unwrap().as_millis());
    Ok(())
}
