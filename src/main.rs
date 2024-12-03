mod align;
mod bitfield;
mod byte_reader;
mod compression;
mod file_ext;
mod hash;
mod msg;
mod pak;
mod rsz;
mod suffix;
mod tex;
mod user;

extern crate image;

use anyhow::*;
use clap::Parser;
use msg::Msg;
use std::fs::{self, write, File, ReadDir};
use std::io::Write;
use std::os;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tex::Tex;
use user::User;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_name: String,
    #[arg(short, long, default_value_t = false)]
    dump_all: bool,
}

fn dump_files(root_dir: PathBuf, files: Vec<&str>, subdir: &str) -> Result<()> {
    for file in files {
        let full_file_path = root_dir.join(file);
        println!("Reading file {full_file_path:?}");
        let nodes = User::new(File::open(full_file_path)?)?
            .rsz
            .deserialize(None)
            .expect("Could not read file");
        let json = nodes
            .into_iter()
            .map(|x| x.to_json().unwrap())
            .collect::<String>();

        let file_path = Path::new(file);
        let mut output_path = PathBuf::from("outputs").join(subdir);
        output_path.push(file_path.file_name().unwrap().to_str().unwrap().to_string() + ".json");

        println!("Trying to save to {:?}", &output_path);
        let _ = fs::create_dir_all(output_path.parent().unwrap())?;
        let mut f = std::fs::File::create(&output_path).expect("Error Creating File");
        f.write_all(json.as_bytes())?;
        println!("Saved file");
    }
    Ok(())
}

fn dump_msg(base_dir: PathBuf) -> Result<()> {
    let files = find_files_with_extension(base_dir, ".msg.23");

    for file in &files {
        println!("{file:?}");
        if let Result::Ok(msg) = Msg::new(file.to_str().unwrap().to_string()) {
            msg.save(file.file_name().unwrap().to_str().expect("This should not have been able to happen"));
        } else {
            eprintln!("Could not convert message file {}", file.display());
        }
    }

    Ok(())
}

fn find_files_with_extension(base_dir: PathBuf, extension: &str) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let mut paths: Vec<PathBuf> = Vec::new();
    paths.push(base_dir);
    while let Some(dir) = paths.pop() {
        if let Result::Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    paths.push(path);
                } else {
                    if let Some(x) = path.file_name().unwrap().to_str() {
                        if x.ends_with(extension) {
                            results.push(path);
                        }
                    }
                }
            }
        }
    }
    results
}

fn main() -> Result<()> {
    let now = SystemTime::now();
    let args = Args::parse();
    println!("{:?}", args);

    if args.dump_all {
        let base_dir = PathBuf::from(args.file_name.clone());
        let _ = dump_files(base_dir.clone(), vec![
            "stm/gamedesign/common/item/itemdata.user.3",
            "stm/gamedesign/common/item/fixitems.user.3",
            "stm/gamedesign/common/item/itemrecipe.user.3",
            "stm/gamedesign/common/item/autousehealthitemdata.user.3",
            "stm/gamedesign/common/item/autousestatusitemdata.user.3",
        ], "items");
        let _ = dump_files(base_dir.clone(), vec![
            "stm/gamedesign/common/equip/skilldata.user.3",
            "stm/gamedesign/common/equip/skillcommondata.user.3",
        ], "skills");
        let _ = dump_msg(base_dir.clone().join("stm/gamedesign/"));
    } else if args.file_name.ends_with("msg.23") {
        let msg = Msg::new(args.file_name.clone())?;
        msg.save(&args.file_name.clone());
    } else if args.file_name.ends_with("user.3") {
        let nodes = User::new(File::open(&args.file_name)?)?
            .rsz
            .deserialize(Some(0))
            .unwrap();
        for node in nodes {
            println!("{}", node.to_json().unwrap());
        }
    } else {
        let tex = Tex::new(args.file_name.clone())?;
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
    println!("Time taken: {} ms", now.elapsed().unwrap().as_millis());
    Ok(())
}
