#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "Building Fax Compiler..."
cargo build --release

echo "Build complete!"
echo "Binary: target/release/faxc"