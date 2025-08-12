#!/bin/bash

# Paradigm Cryptocurrency Build Script
# This script builds all components of the Paradigm cryptocurrency system

set -e  # Exit on any error

echo "🚀 Building Paradigm Cryptocurrency System"
echo "=========================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust found: $(rustc --version)"

# Check for required system dependencies
echo "🔍 Checking system dependencies..."

# Check for protobuf compiler
if ! command -v protoc &> /dev/null; then
    echo "⚠️  protoc not found. Installing..."
    case "$OSTYPE" in
        "darwin"*)
            if command -v brew &> /dev/null; then
                brew install protobuf
            else
                echo "❌ Please install protobuf manually or install Homebrew"
                exit 1
            fi
            ;;
        "linux-gnu"*)
            if command -v apt-get &> /dev/null; then
                sudo apt-get update
                sudo apt-get install -y protobuf-compiler libprotobuf-dev
            elif command -v yum &> /dev/null; then
                sudo yum install -y protobuf-compiler protobuf-devel
            else
                echo "❌ Please install protobuf-compiler manually"
                exit 1
            fi
            ;;
        "msys"*)
            echo "ℹ️  On Windows, please install protobuf manually or use vcpkg"
            ;;
    esac
fi

# Check for SQLite
if ! command -v sqlite3 &> /dev/null; then
    echo "⚠️  SQLite3 not found. Installing..."
    case "$OSTYPE" in
        "darwin"*)
            brew install sqlite3
            ;;
        "linux-gnu"*)
            if command -v apt-get &> /dev/null; then
                sudo apt-get install -y sqlite3 libsqlite3-dev
            elif command -v yum &> /dev/null; then
                sudo yum install -y sqlite sqlite-devel
            fi
            ;;
    esac
fi

echo "✅ System dependencies checked"

# Create build directory
BUILD_DIR="target/paradigm-release"
mkdir -p "$BUILD_DIR"

echo "🔨 Building Paradigm Core..."
cargo build --release --package paradigm-core
if [ $? -eq 0 ]; then
    echo "✅ Paradigm Core built successfully"
    cp target/release/paradigm-core "$BUILD_DIR/"
else
    echo "❌ Failed to build Paradigm Core"
    exit 1
fi

echo "🔨 Building Paradigm Wallet..."
cargo build --release --package paradigm-wallet
if [ $? -eq 0 ]; then
    echo "✅ Paradigm Wallet built successfully"
    cp target/release/paradigm-wallet "$BUILD_DIR/"
else
    echo "❌ Failed to build Paradigm Wallet"
    exit 1
fi

echo "🔨 Building Paradigm Contributor..."
cargo build --release --package paradigm-contributor
if [ $? -eq 0 ]; then
    echo "✅ Paradigm Contributor built successfully"
    cp target/release/paradigm-contributor "$BUILD_DIR/"
else
    echo "❌ Failed to build Paradigm Contributor"
    exit 1
fi

# Run tests
echo "🧪 Running tests..."
cargo test --release --all
if [ $? -eq 0 ]; then
    echo "✅ All tests passed"
else
    echo "⚠️  Some tests failed, but continuing..."
fi

# Create configuration files
echo "📝 Creating configuration files..."

cat > "$BUILD_DIR/paradigm.toml" << EOF
# Paradigm Cryptocurrency Configuration

[network]
# Network port for node communication
port = 8080

# Bootstrap peers (comma-separated)
bootstrap_peers = []

# Network ID
network_id = "paradigm-mainnet"

[node]
# Data directory for blockchain data
data_dir = "./paradigm-data"

# Log level (trace, debug, info, warn, error)
log_level = "info"

# Maximum number of connections
max_connections = 50

[contributor]
# Maximum concurrent ML tasks
max_tasks = 4

# Use GPU acceleration if available
use_gpu = true

# Accepted task types (empty means all)
task_types = []

# Minimum task difficulty to accept (1-10)
min_difficulty = 1

[wallet]
# Auto-sync with network
auto_sync = true

# Sync interval in seconds
sync_interval = 60

# GUI theme (light, dark, auto)
theme = "auto"
EOF

cat > "$BUILD_DIR/start-node.sh" << 'EOF'
#!/bin/bash
echo "Starting Paradigm Node..."
./paradigm-core --config paradigm.toml
EOF

cat > "$BUILD_DIR/start-contributor.sh" << 'EOF'
#!/bin/bash
echo "Starting Paradigm Contributor..."
echo "Please provide your wallet address:"
read -p "Wallet Address: " WALLET_ADDR
./paradigm-contributor --wallet-address "$WALLET_ADDR" --use-gpu
EOF

cat > "$BUILD_DIR/start-wallet.sh" << 'EOF'
#!/bin/bash
echo "Starting Paradigm Wallet..."
./paradigm-wallet
EOF

# Make scripts executable
chmod +x "$BUILD_DIR"/*.sh

# Create README for the build
cat > "$BUILD_DIR/README.txt" << EOF
Paradigm Cryptocurrency - Release Build
======================================

This directory contains the complete Paradigm cryptocurrency system.

Files:
- paradigm-core: Main blockchain node
- paradigm-wallet: GUI wallet application  
- paradigm-contributor: ML contributor client
- paradigm.toml: Configuration file
- start-*.sh: Startup scripts

Quick Start:
1. For regular users: Run ./start-wallet.sh
2. To run a node: Run ./start-node.sh
3. To contribute ML power: Run ./start-contributor.sh

Documentation:
- See QUICKSTART.md in the main repository
- Visit: https://paradigm.network/docs

Requirements:
- Windows 10+, macOS 10.15+, or Linux
- 4GB RAM minimum, 8GB recommended
- 10GB free disk space
- Internet connection

GPU Support:
- NVIDIA GPUs with CUDA support recommended for contributors
- AMD GPUs with OpenCL support also work
- CPU-only contribution is supported

Support:
- Discord: https://discord.gg/paradigm
- GitHub: https://github.com/paradigm-crypto/paradigm
- Email: support@paradigm.network

License: MIT
EOF

# Create version info
echo "$(git describe --tags --always --dirty 2>/dev/null || echo 'v0.1.0-dev')" > "$BUILD_DIR/VERSION"
echo "Built on: $(date)" >> "$BUILD_DIR/VERSION"
echo "Platform: $(uname -s)-$(uname -m)" >> "$BUILD_DIR/VERSION"

# Copy license and documentation
cp LICENSE "$BUILD_DIR/" 2>/dev/null || true
cp QUICKSTART.md "$BUILD_DIR/" 2>/dev/null || true
cp CONTRIBUTING.md "$BUILD_DIR/" 2>/dev/null || true

echo ""
echo "🎉 Build completed successfully!"
echo "📁 Release files are in: $BUILD_DIR"
echo ""
echo "To get started:"
echo "  cd $BUILD_DIR"
echo "  ./start-wallet.sh     # For users"
echo "  ./start-node.sh       # For node operators"
echo "  ./start-contributor.sh # For ML contributors"
echo ""
echo "📚 See QUICKSTART.md for detailed instructions"
echo "🚀 Welcome to the future of cryptocurrency!"
