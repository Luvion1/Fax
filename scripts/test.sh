#!/bin/bash
# Fax Compiler Test Script

set -e

cd "$(dirname "$0")/.."

echo "Running Fax compiler tests..."
cargo test --workspace

echo "Tests complete!"
