#!/bin/bash

# Simple test to check if output directory generation works
echo "Testing output directory generation..."

# Create a simple test crate structure
mkdir -p test_crate/src
echo 'pub fn hello() { println!("Hello, world!"); }
pub struct TestStruct { pub field: i32 }
pub enum TestEnum { A, B }' > test_crate/src/lib.rs

echo '[package]
name = "test_crate"
version = "0.1.0"
edition = "2021"' > test_crate/Cargo.toml

echo "Created test crate structure"
echo "Contents of test_crate/src/lib.rs:"
cat test_crate/src/lib.rs

# Test our CratePaths generation
echo "Testing CratePaths generation..."
