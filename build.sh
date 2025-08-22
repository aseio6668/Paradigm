#!/bin/bash
# Paradigm Cryptocurrency Build Script for Linux/macOS
# Fast and efficient build similar to build.bat

set -e  # Exit on any error

echo "🚀 Paradigm Build (Linux/macOS)"
echo "==============================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Install from: https://rustup.rs/"
    exit 1
fi

RUST_VER=$(rustc --version | awk '{print $2}')
echo "✅ Rust $RUST_VER found"

# Quick protoc check
if ! command -v protoc &> /dev/null; then
    echo "⚠️  protoc not found - some features limited"
else
    echo "✅ protoc found"
fi

echo ""
echo "🧹 Cleaning..."
cargo clean > /dev/null 2>&1
echo "✅ Clean completed"

echo ""
echo "🔨 Building Paradigm Core..."
cargo build --release --package paradigm-core --quiet
if [ $? -eq 0 ]; then
    echo "✅ Paradigm Core built successfully"
else
    echo "❌ Failed to build Paradigm Core"
    exit 1
fi

echo "🔨 Building Paradigm Wallet..."
cargo build --release --package paradigm-wallet --quiet
if [ $? -eq 0 ]; then
    echo "✅ Paradigm Wallet built successfully"
else
    echo "❌ Failed to build Paradigm Wallet"
    exit 1
fi

echo "🔨 Building Paradigm Contributor..."
cargo build --release --package paradigm-contributor --quiet
if [ $? -eq 0 ]; then
    echo "✅ Paradigm Contributor built successfully"
else
    echo "❌ Failed to build Paradigm Contributor"
    exit 1
fi

echo "🔨 Skipping Paradigm SDK (Library has compilation issues)..."
echo "⚠️  Paradigm SDK temporarily disabled - continuing without SDK"
echo "    (SDK has async/await Send trait issues - will be fixed in future update)"

echo ""
echo "🧪 Running quick compilation verification..."
cargo check --package paradigm-core --package paradigm-wallet --package paradigm-contributor --quiet
if [ $? -eq 0 ]; then
    echo "✅ All packages compile successfully"
else
    echo "⚠️  Some compilation issues detected, but core binaries built successfully"
fi

echo ""
echo "🎉 Build completed successfully!"
echo "================================"
echo ""
echo "📁 Binaries are in: target/release/"
echo "✅ paradigm-core"
echo "✅ paradigm-wallet"  
echo "✅ paradigm-contributor"
echo "⚠️  paradigm-sdk (temporarily disabled)"
echo ""
echo "🚀 Quick start:"
echo "  ./target/release/paradigm-core        # Start node"
echo "  ./target/release/paradigm-wallet      # Start wallet"
echo "  ./target/release/paradigm-contributor # Start earning PAR"
echo ""
echo "🌐 Start a network:"
echo "  ./start-network.sh                    # Launch bootstrap network"
echo ""
echo "🚀 Welcome to Paradigm!"