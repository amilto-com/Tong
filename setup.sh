#!/usr/bin/env bash
# TONG Programming Language Setup Script (Unix)

set -euo pipefail

echo "🚀 Setting up TONG - The Ultimate Programming Language"
echo "=================================================="

if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ Rust toolchain not found. Please install Rust from https://rustup.rs and re-run this script."
    exit 1
fi

echo "Building tong (release)..."
pushd rust/tong >/dev/null
cargo build --release
popd >/dev/null

BIN="$(pwd)/rust/tong/target/release/tong"
if [ "$1" = "--global" ]; then
    echo "Creating global symlink..."
    sudo ln -sf "$BIN" /usr/local/bin/tong
    echo "✅ TONG is now available globally as 'tong'"
else
    echo "Built binary at: $BIN"
    echo "You can run: $BIN examples/hello.tong"
fi

echo ""
echo "🎯 Quick Start:"
echo "  cargo run -p tong -- ../../examples/hello.tong    # Run example"
echo "  cargo build -p tong --release                     # Build optimized binary"
echo "  tong ../../examples/hello.tong                    # After --global install"
echo ""
echo "📚 Examples available in examples/ directory"
echo "📖 See README.md for full documentation"
echo ""
echo "✨ TONG is ready for heterogeneous computing!"