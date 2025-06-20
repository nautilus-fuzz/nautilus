#!/bin/bash

# Check if the script received an argument
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <bitmap_size>"
    exit 1
fi

# Assign the input parameter to bitmap_size
bitmap_size="$1"

# Update bitmap_size in config.ron
sed -i "s/bitmap_size:[[:space:]]*[0-9]\+,\?/\bitmap_size:\t\t\t\t$bitmap_size,/" config.ron

# Update bitmap_size in forksrv/src/lib.rs
# It will replace values like "1 << 16" or any integer with the new bitmap_size
sed -i "s/return (shm_id, trace_bits as \*mut \[u8; [0-9]* << [0-9]*\]);/return (shm_id, trace_bits as \*mut \[u8; $bitmap_size\]);/" forksrv/src/lib.rs
sed -i "s/return (shm_id, trace_bits as \*mut \[u8; [0-9]\+\]);/return (shm_id, trace_bits as \*mut \[u8; $bitmap_size\]);/" forksrv/src/lib.rs

echo "Updated bitmap_size to $bitmap_size in config.ron and forksrv/src/lib.rs"
