#!/bin/bash
set -e

# Create test directory
mkdir -p test_dir_simple/subdir

# Create test files
echo "Test file" > test_dir_simple/test.txt
echo "Subdir file" > test_dir_simple/subdir/subfile.txt

# Upload files one by one
echo "Uploading test.txt..."
cargo run -- push test_dir_simple/test.txt /tmp/gdrive_test/test_upload

echo "Creating subdirectory..."
cargo run -- files mkdir subdir --parent 1m4YMczQtDi1l1U_erBTgAkeR-5n4wPCu

echo "Uploading subfile.txt..."
cargo run -- push test_dir_simple/subdir/subfile.txt /tmp/gdrive_test/test_upload/subdir
