#!/bin/bash
fs=`find ./outputs/re_chunk_000.pak.sub_000 -type f -name "*.tex.*"` 
files=($fs)
for file_path in "${files[@]}"; do
    echo $file_path
    ./target/release/mhwstex -f $file_path
done

