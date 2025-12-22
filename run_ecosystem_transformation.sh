#!/bin/bash
# Ecosystem transformation runner for cargo2nix
# This will apply split-decls-rs to the entire cargo2nix ecosystem (590 submodules, 6017 Cargo.toml files)

set -e

echo "🚀 Starting cargo2nix ecosystem transformation..."
echo "📊 Target: 590 submodules with 6,017 Cargo.toml files"
echo "🔧 Tool: split-decls-rs declaration splitter"

cd /mnt/data1/nix/vendor/rust/cargo2nix/submodules/split-decls-rs

# Build the tool if needed
if [ ! -f target/debug/split-decls-rs ]; then
    echo "🔨 Building split-decls-rs..."
    cargo build --bin split-decls-rs
fi

# Run the ecosystem transformation
echo "🌟 Applying declaration splitting to entire ecosystem..."
timeout 1800 ./target/debug/split-decls-rs --verbose 2>&1 | tee ecosystem_transformation.log

echo "✅ Ecosystem transformation complete!"
echo "📝 Log saved to: ecosystem_transformation.log"
echo "🎯 Ready for diagonalization matrix integration"
