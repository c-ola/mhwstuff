mod align;
mod bitfield;
mod byte_reader;
mod compression;
mod file_ext;
mod msg;
mod rsz;
mod tex;
mod user;
mod dersz;

extern crate image;

use std::io::*;
use anyhow::anyhow;
use clap::Parser;
use msg::Msg;
use std::fs::{self, read_to_string,File};
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
}

fn construct_paths(file: String, prefix: Option<String>, out_dir_base: String, preserve_structure: bool) -> Result<(PathBuf, PathBuf)> {
    let full_file_path = match prefix {
        Some(ref prefix) => Path::new(&prefix).join(&file),
        None => PathBuf::from(&file),
    };
    let output_path = PathBuf::from(out_dir_base).join(
        if preserve_structure {
            match prefix {
                Some(ref prefix) => {
                    let file = Path::new(&full_file_path);
                    file.strip_prefix(prefix).unwrap().to_str().unwrap()
                }
                None => &file
            }
        } else {
            let file = Path::new(&file);
            let path = file.file_name().unwrap().to_str().unwrap();
            path
        }
    );

    Ok((full_file_path, output_path))
}

enum FileType {
    Msg(u32),
    User(u32),
    Tex(u32),
    Unknown
}

fn get_file_ext(file_name: String) -> Result<FileType> {
    let split = file_name.split('.').collect::<Vec<_>>();

    let version = match u32::from_str_radix(split[split.len() - 1], 10) {
        Ok(val) => val,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("{e}"))),
    };

    let file_type = match split.get(split.len() - 2) {
        Some(ext) => {
            match *ext {
                "user" => FileType::User(version),
                "msg" => FileType::Msg(version),
                "tex" => FileType::Tex(version),
                _ => FileType::Unknown
            }
        },
        None => {
            FileType::Unknown
        }
    };

    Ok(file_type)
}

fn dump_file(file_path: PathBuf, output_path: PathBuf) -> anyhow::Result<()> {
    //output_path.set_file_name(file_path.file_name().unwrap().to_str().unwrap().to_string() + ".json");
    let file_name = match file_path.file_name() {
        Some(file_name) => file_name,
        None => {
            return Err(anyhow!("Path does not contain file"));
        }
    };
    let file_type = get_file_ext(file_name.to_string_lossy().to_string())?;
    let res = match file_type {
        FileType::Msg(_v) => {
            let mut output_path = output_path.clone();
            output_path.set_file_name(output_path.file_name().unwrap().to_str().unwrap().to_string() + ".json");
            let msg = Msg::new(file_path.to_string_lossy().to_string())?;

            println!("Trying to save to {:?}", &output_path);
            let _ = fs::create_dir_all(output_path.parent().unwrap())?;
            let mut f = std::fs::File::create(&output_path).expect("Error Creating File");
            msg.save(&mut f);
            println!("Saved file");
            Ok(())
        },
        FileType::User(_v) => {
            let rsz = User::new(File::open(file_path.clone())?)?.rsz;
            let res = rsz.deserializev2();
            let res = match res {
                Ok(nodes) => {
                    let mut output_path = output_path.clone();
                    output_path.set_file_name(output_path.file_name().unwrap().to_str().unwrap().to_string() + ".json");
                    //output_path.push(file_path.file_name().unwrap().to_str().unwrap().to_string() + ".json");

                    let json_res = serde_json::to_string_pretty(&nodes); 
                    match json_res {
                        Ok(json) => {
                            let _ = fs::create_dir_all(output_path.parent().unwrap())?;
                            let mut f = std::fs::File::create(&output_path).expect("Error Creating File");
                            f.write_all(json.as_bytes())?;
                            println!("[INFO] Saved File {:?}", &output_path);
                            Ok(())
                        },
                        Err(e) => {
                            Err(anyhow!("File: {file_path:?} Reason: {e:?}"))
                        }
                    }
                },
                Err(e) => {
                    Err(anyhow!("File: {file_path:?} Reason: {e:?}"))
                }
            };
            res
            //Ok(())
        },
        FileType::Tex(_v) => {
            let file_name = file_name.to_string_lossy().to_string();
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
            Ok(())
        },
        FileType::Unknown => return Err(anyhow!("Unknown File Type")),
    };
    res
}

#[allow(dead_code)]
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

fn dump_all(root_dir: Option<String>, out_dir: String, list_file: String) -> anyhow::Result<()> {
    let list = read_to_string(&list_file).expect(format!("Could not open file {list_file}").as_str());
    let list: Vec<&str> = list.lines().collect();
    for file in list {
        let paths = construct_paths(file.to_string(), root_dir.clone(), out_dir.clone(), true);
        let (file_path, output_path) = match paths {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[ERROR] Could not create file path and output path {e}");
                continue
            }
        };
        match dump_file(file_path, output_path) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("[ERROR] Error dumping file {e}");
                continue
            }
        };
    }
    Ok(())
}



fn main() -> anyhow::Result<()> {
    let now = SystemTime::now();
    let args = Args::parse();
    println!("{:#?}", args);
    
    match args.list {
        Some(list) => {
            dump_all(args.root_dir, args.out_dir, list)?;
        }, 
        None => match args.file_name {
            Some(file_name) => {
                let (file_path, output_path) = construct_paths(file_name.clone(), args.root_dir.clone(), args.out_dir.clone(), false)?;
                dump_file(file_path, output_path)?;
            },
            None => println!("Must provide file name"),
        }
    }
    println!("Time taken: {} ms", now.elapsed().unwrap().as_millis());
    Ok(())
}
