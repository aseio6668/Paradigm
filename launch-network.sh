#!/bin/bash
# Paradigm Production Network Launcher
# Streamlined production-ready network deployment

echo "🌟 Paradigm Production Network Launcher"
echo "====================================="

# Check if binaries exist
if [ ! -f "target/release/paradigm-core" ]; then
    echo "❌ Release binaries not found!"
    echo "Please run ./build.sh first."
    exit 1
fi

# Configuration
DATA_DIR="production-data"
LOG_DIR="production-logs"

# Create directories
mkdir -p "$DATA_DIR" "$LOG_DIR"

case "$1" in
    "stop")
        echo "🛑 Stopping Paradigm Network..."
        pkill -f paradigm-core 2>/dev/null || true
        pkill -f paradigm-contributor 2>/dev/null || true
        pkill -f paradigm-wallet 2>/dev/null || true
        echo "✅ Network stopped"
        exit 0
        ;;
    "status")
        echo "📊 Paradigm Network Status"
        echo "========================"
        echo ""
        echo "🔷 Core Nodes:"
        if pgrep -f paradigm-core > /dev/null; then
            echo "    ✅ Core nodes running"
        else
            echo "    ❌ No core nodes running"
        fi
        echo ""
        echo "🔶 Contributors:"
        if pgrep -f paradigm-contributor > /dev/null; then
            echo "    ✅ Contributors mining PAR"
        else
            echo "    ❌ No contributors running"
        fi
        echo ""
        echo "💰 Wallets:"
        if pgrep -f paradigm-wallet > /dev/null; then
            echo "    ✅ Wallets active"
        else
            echo "    ⚠️  No wallets running"
        fi
        exit 0
        ;;
    "help")
        echo "Paradigm Production Network Launcher"
        echo "==================================="
        echo ""
        echo "Usage: ./launch-network.sh [command]"
        echo ""
        echo "Commands:"
        echo "  (none)   Start production network"
        echo "  stop     Stop all network components"
        echo "  status   Show current network status"
        echo "  help     Show this help message"
        echo ""
        echo "Examples:"
        echo "  ./launch-network.sh        # Start network"
        echo "  ./launch-network.sh stop   # Stop network"
        echo "  ./launch-network.sh status # Check status"
        exit 0
        ;;
esac

echo "✅ Starting Paradigm Production Network..."
echo ""

# Start core nodes
echo "🔷 Starting Core Node 1 (Bootstrap - Port 8080)..."
target/release/paradigm-core --port 8080 --data-dir $DATA_DIR/node-1 &
NODE1_PID=$!

sleep 3

echo "🔷 Starting Core Node 2 (Port 8081)..."
target/release/paradigm-core --port 8081 --data-dir $DATA_DIR/node-2 --bootstrap-peers /ip4/127.0.0.1/tcp/8080 &
NODE2_PID=$!

sleep 3

echo "🔷 Starting Core Node 3 (Port 8082)..."
target/release/paradigm-core --port 8082 --data-dir $DATA_DIR/node-3 --bootstrap-peers /ip4/127.0.0.1/tcp/8080 &
NODE3_PID=$!

# Wait for network to establish
echo "⏳ Waiting for network to establish..."
sleep 8

# Start contributors
echo "🔶 Starting Contributors (PAR Token Miners)..."
target/release/paradigm-contributor --data-dir $DATA_DIR/contrib-1 &
CONTRIB1_PID=$!
sleep 2

target/release/paradigm-contributor --data-dir $DATA_DIR/contrib-2 &
CONTRIB2_PID=$!
sleep 2

target/release/paradigm-contributor --data-dir $DATA_DIR/contrib-3 &
CONTRIB3_PID=$!

echo ""
echo "🎉 Production Network Started Successfully!"
echo "=========================================="
echo ""
echo "🌐 Network URLs:"
echo "    Node 1 (Bootstrap): http://127.0.0.1:8080"
echo "    Node 2: http://127.0.0.1:8081"
echo "    Node 3: http://127.0.0.1:8082"
echo ""
echo "💰 PAR Mining Active:"
echo "    - 3 Contributors earning PAR tokens"
echo "    - AI governance system online"
echo "    - Quantum-resistant security enabled"
echo ""
echo "📊 Process IDs:"
echo "    Nodes: $NODE1_PID, $NODE2_PID, $NODE3_PID"
echo "    Contributors: $CONTRIB1_PID, $CONTRIB2_PID, $CONTRIB3_PID"
echo ""
echo "📊 Commands:"
echo "    ./launch-network.sh status    # Show network status"
echo "    ./launch-network.sh stop      # Stop all components"
echo ""
echo "🛑 To stop individual processes: kill [PID]"
echo "✋ To stop entire network: ./launch-network.sh stop"
echo ""

# Function to cleanup processes
cleanup() {
    echo ""
    echo "🛑 Stopping Paradigm Network..."
    kill $NODE1_PID $NODE2_PID $NODE3_PID $CONTRIB1_PID $CONTRIB2_PID $CONTRIB3_PID 2>/dev/null || true
    echo "✅ Network stopped"
    exit 0
}

# Set up cleanup on script exit
trap cleanup INT TERM

# Wait for user to stop
echo "Press Enter to stop the network or Ctrl+C..."
read

cleanup