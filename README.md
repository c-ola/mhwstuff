# Repo for some MH Wilds data generation stuff

Makes use of the RSZ reader, texture codecs, and some file reading stuff from ![https://github.com/wwylele/mhrice](https://github.com/wwylele/mhrice)

Also looked at it to figure out how to read some of the file formats.

Uses praydog's emulation dumper for the rsz files.

## Build

```
git clone https://github.com/c-ola/mhwstuff.git
cd mhwstuff
git submodule update --init
cargo build # optional --release flag can be set, it can make textures take 24x less time
```

## usage

Single File
```
cargo run --release -- -r <path/to/game/native> -o <output/directory> -f <path/to/file>
```

Multi File
Note: the root directory prefix gets removed from the file path when saving
```
cargo run --release -- -r <path/to/game/native> -o <output/directory> -l <list of files to process>
```
