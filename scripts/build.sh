#!/bin/bash
# Fax Compiler Build Script

set -e

cd "$(dirname "$0")/.."

if [ "$1" = "--release" ]; then
    echo "Building Fax compiler in release mode..."
    cargo build --release -p faxc-drv
else
    echo "Building Fax compiler in debug mode..."
    cargo build -p faxc-drv
fi

echo "Build complete!"
