#!/bin/bash

fs=`find /mnt/windows/Users/nikol/programming/REtool/wilds/re_chunk_000.pak.sub_000/natives/stm/gui/ -type f -name "*.tex.*"`
files=($fs)

for file_path in "${files[@]}"; do
    echo $file_path
   ./mhwstex/mhwstex $file_path
done
