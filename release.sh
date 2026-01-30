#!/bin/bash
set -e

cd "$(dirname "$0")"

VERSION="0.1.0"

echo "Releasing squirreldb-sdk v${VERSION}..."

echo "Running tests..."
cargo test

echo "Building..."
cargo build --release

echo "Publishing to crates.io..."
cargo publish --allow-dirty

echo "Released squirreldb-sdk@${VERSION}"
echo ""
echo "Users can install with:"
echo "  cargo add squirreldb-sdk"
