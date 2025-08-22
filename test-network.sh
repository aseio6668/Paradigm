#!/bin/bash
# Paradigm Network Test Launcher - Updated for Release Binaries
# This script demonstrates multiple clients connecting and synchronizing

echo "🚀 Starting Paradigm Network Test"
echo "=================================="

# Check if binaries exist
if [ ! -f "target/release/paradigm-core" ]; then
    echo "❌ Release binaries not found!"
    echo "Please run ./build.sh first."
    exit 1
fi

# Create test data directory
mkdir -p test-data

echo "✅ Release binaries found"
echo "🔷 Starting Main Node (Port 8080)..."
target/release/paradigm-core --port 8080 --data-dir test-data/node-main &
NODE_PID=$!
echo "Node PID: $NODE_PID"

# Wait for node to initialize
sleep 5

# Start multiple contributors
echo "🔶 Starting Contributor 1 (Fast Worker)..."
target/release/paradigm-contributor --node-address 127.0.0.1:8080 --data-dir test-data/contrib-1 &
CONTRIB1_PID=$!

sleep 3

echo "🔶 Starting Contributor 2 (Balanced Worker)..."
target/release/paradigm-contributor --node-address 127.0.0.1:8080 --data-dir test-data/contrib-2 &
CONTRIB2_PID=$!

sleep 3

echo "🔶 Starting Contributor 3 (Efficient Worker)..."
target/release/paradigm-contributor --node-address 127.0.0.1:8080 --data-dir test-data/contrib-3 &
CONTRIB3_PID=$!

echo ""
echo "✅ Network test launched successfully!"
echo ""
echo "📊 Running processes:"
echo "    - 1 Node (PID: $NODE_PID, Port 8080)"
echo "    - 3 Contributors (PIDs: $CONTRIB1_PID, $CONTRIB2_PID, $CONTRIB3_PID)"
echo ""
echo "🔍 Watch for:"
echo "    - ML task processing and PAR rewards"
echo "    - AI governance system in action"
echo "    - Network synchronization"
echo ""
echo "🛑 Press Ctrl+C to stop all processes"
echo ""

# Function to cleanup processes
cleanup() {
    echo ""
    echo "🛑 Stopping all Paradigm processes..."
    kill $NODE_PID $CONTRIB1_PID $CONTRIB2_PID $CONTRIB3_PID 2>/dev/null
    echo "✅ All processes stopped"
    exit 0
}

# Set up cleanup on script exit
trap cleanup INT TERM

# Wait for user to stop
echo "Press Enter to stop the network..."
read

cleanup