# Repo for some MH Wilds data generation stuff

Makes use of the RSZ parse from ![https://github.com/wwylele/mhrice](https://github.com/wwylele/mhrice)

Also looked at it to figure out how to read some of the file formats.

## Build

```
git clone https://github.com/c-ola/mhwstuff.git
cd mhwstuff
git submodule update --init
cargo build # optional --release flag can be set, it can make textures take 24x less time
```

## usage

.tex and .msg files
```
cargo run -- -f <file_name>
```

.user files
```
cargo run -- -d -f <path/to/game/native>
```
