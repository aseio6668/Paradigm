#!/bin/bash
# Paradigm Network Bootstrap Node (Linux/macOS)
# Start a Paradigm network that others can join

echo "🌟 Starting Paradigm Network Bootstrap Node"
echo "=========================================="
echo ""

# Check if binaries exist
if [ ! -f "target/release/paradigm-core" ]; then
    echo "❌ Paradigm binaries not found!"
    echo "Please run ./build.sh first."
    echo ""
    exit 1
fi

echo "✅ Paradigm binaries found"
echo ""

# Get local IP address for display
LOCAL_IP=$(hostname -I | awk '{print $1}' 2>/dev/null || echo "127.0.0.1")

echo "📡 Network Configuration:"
echo "    Port: 8080"
echo "    Local IP: $LOCAL_IP"
echo "    Data Directory: ./network-data"
echo ""

echo "⚠️  IMPORTANT: For internet access, make sure:"
echo "    1. Port 8080 is open in your firewall"
echo "    2. Router port forwarding is configured (if needed)"
echo ""

echo "🚀 Starting bootstrap node..."
echo "    Your network address will be displayed in the logs below."
echo "    Look for: \"local_peer_id=12D3KooW...\""
echo ""

echo "✋ Press Ctrl+C to stop the network"
echo ""

# Start the bootstrap node
cd target/release
./paradigm-core --port 8080 --data-dir ./network-data

echo ""
echo "🔻 Bootstrap node stopped"