#!/bin/bash
set -e

cd "$(dirname "$0")"

VERSION=$(grep '^version' Cargo.toml | head -1 | awk -F'"' '{print $2}')

echo "Building squirreldb Rust SDK v${VERSION}..."
cargo build --release

echo "Running tests..."
cargo test

echo "Running clippy..."
cargo clippy -- -D warnings

echo "Dry run publish..."
cargo publish --dry-run

read -p "Publish squirreldb v${VERSION} to crates.io? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  cargo publish
  echo "Published squirreldb@${VERSION} to crates.io"
else
  echo "Aborted"
fi
