pub mod generator;
use std::path::PathBuf;

use generator::Rsz;

fn main() {
    let _ = Rsz::gen_structs(&mut PathBuf::from("rszmhwilds.json"));
}
