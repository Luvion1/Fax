#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "Running tests..."
cargo test --workspace
cargo test --workspace --release

echo "All tests passed!"