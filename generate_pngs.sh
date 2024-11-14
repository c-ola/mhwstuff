#!/bin/bash
fs=`find ./outputs/natives/stm/gui/ -type f -name "*.tex.*"`
files=($fs)
for file_path in "${files[@]}"; do
    echo $file_path
    ./target/debug/mhwstex2dds -f $file_path
done

