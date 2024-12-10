mod align;
mod bitfield;
mod byte_reader;
mod compression;
mod file_ext;
mod msg;
mod rsz;
mod suffix;
mod tex;
mod user;
mod dersz;

extern crate image;

use std::io::*;
use clap::Parser;
use msg::Msg;
use std::fs::{self, read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tex::Tex;
use user::User;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short('f'), long)]
    file_name: Option<String>,
    
    #[arg(short('r'), long)]
    root_dir: Option<String>,

    #[arg(short('l'), long)]
    list: Option<String>, 

    #[arg(short('o'), long, default_value_t = String::from("outputs"))]
    out_dir: String,

    #[arg(short('u'), long, default_value_t = false)]
    dump_user: bool,
    
    #[arg(short('m'), long, default_value_t = false)]
    dump_msg: bool,
}

fn dump_users(root_dir: PathBuf, files: Vec<&str>, subdir: &str) -> anyhow::Result<()> {
    for file in files {
        let full_file_path = root_dir.join(file);
        println!("Reading file {full_file_path:?}");
        let rsz = User::new(File::open(full_file_path)?)?.rsz;
        let res = rsz.deserializev2();
        match res {
            Ok(nodes) => {
                let file_path = Path::new(file);
                let mut output_path = PathBuf::from("outputs").join(subdir);
                output_path.push(file_path.file_name().unwrap().to_str().unwrap().to_string() + ".json");

                let json = serde_json::to_string_pretty(&nodes)?; 
                println!("Trying to save to {:?}", &output_path);
                let _ = fs::create_dir_all(output_path.parent().unwrap())?;
                let mut f = std::fs::File::create(&output_path).expect("Error Creating File");
                f.write_all(json.as_bytes())?;
                println!("Saved file");
            },
            Err(e) => {
                eprintln!("{e:?}");
                continue
            }

        }
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

fn dump_all(root_dir: PathBuf, list_file: Option<String>, do_dump_user: bool, do_dump_msg: bool) -> anyhow::Result<()> {
    if do_dump_user {
        match list_file {
            None => {
                let _ = dump_users(root_dir.clone(), vec![
                    "stm/gamedesign/common/item/itemdata.user.3",
                    "stm/gamedesign/common/item/fixitems.user.3",
                    "stm/gamedesign/common/item/itemrecipe.user.3",
                    "stm/gamedesign/common/item/autousehealthitemdata.user.3",
                    "stm/gamedesign/common/item/autousestatusitemdata.user.3",
                ], "items");
                let _ = dump_users(root_dir.clone(), vec![
                    "stm/gamedesign/common/equip/skilldata.user.3",
                    "stm/gamedesign/common/equip/skillcommondata.user.3",
                ], "skills");
            },
            Some(file_name) => {
                let list = read_to_string(&file_name).expect(format!("Could not open file {file_name}").as_str());
                let list = list.lines().collect();
                dump_users(root_dir.clone(), list, "dump")?;
            }
        }
    }
    if do_dump_msg {
        let _ = dump_msg(root_dir.join("stm/gamedesign/"));
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let now = SystemTime::now();
    let args = Args::parse();
    println!("{:?}", args);


    if args.dump_msg || args.dump_user  {
        if let Some(dir) = args.root_dir {
            let base_dir = PathBuf::from(dir.clone());
            dump_all(base_dir, args.list, args.dump_user, args.dump_msg)?;
        } else {
            println!("Please provide a directory to natives folder when using dump all");
        }
        return Ok(())
    } else if let Some(file_name) = args.file_name {
        if file_name.ends_with("msg.23") {
            let msg = Msg::new(file_name.clone())?;
            msg.save(&file_name.clone());
        } else if file_name.ends_with("user.3") {
            let re_dump = User::new(File::open(&file_name)?)?
                .rsz
                .deserializev2()
                .unwrap();

            let file_path = Path::new(&file_name);
            let mut output_path = PathBuf::from("outputs").join("user");
            output_path.push(file_path.file_name().unwrap().to_str().unwrap().to_string() + ".json");

            println!("Trying to save to {:?}", &output_path);
            let _ = fs::create_dir_all(output_path.parent().unwrap())?;
            let f = std::fs::File::create(&output_path).expect("Error Creating File");
            serde_json::to_writer_pretty(f, &re_dump)?;
            println!("Saved file");
            //println!("{}", serde_json::to_string_pretty(&re_dump)?);
        } else {
            let tex = Tex::new(file_name.clone())?;
            let rgba = tex.to_rgba(0)?;
            println!("{}", rgba.data.len());
            let name = format!("{}_{}.png", file_name, 0);
            println!("saving to {name}");
            let _ = image::save_buffer(
                &Path::new(&name),
                &rgba.data,
                rgba.width,
                rgba.height,
                image::ExtendedColorType::Rgba8,
            );
        }
    }
    println!("Time taken: {} ms", now.elapsed().unwrap().as_millis());
    Ok(())
}
