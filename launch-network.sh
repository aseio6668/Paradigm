#!/bin/bash

# Paradigm Network Launcher
# Production-ready network deployment with monitoring and synchronization

set -e

# Configuration
NETWORK_NAME="paradigm-mainnet"
LOG_DIR="./logs"
PID_DIR="./pids"
DATA_DIR="./data"
CONFIG_DIR="./config"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create directories
mkdir -p "$LOG_DIR" "$PID_DIR" "$DATA_DIR" "$CONFIG_DIR"

print_header() {
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                    Paradigm Network Launcher                ║${NC}"
    echo -e "${BLUE}║                  Production Network Manager                  ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if process is running
is_running() {
    local pid_file="$1"
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            return 0
        else
            rm -f "$pid_file"
            return 1
        fi
    fi
    return 1
}

# Start core node
start_core_node() {
    local node_id="$1"
    local port="${2:-8080}"
    local p2p_port="${3:-9000}"
    
    local pid_file="$PID_DIR/core-node-$node_id.pid"
    local log_file="$LOG_DIR/core-node-$node_id.log"
    
    if is_running "$pid_file"; then
        print_warning "Core node $node_id already running (PID: $(cat $pid_file))"
        return 0
    fi
    
    print_status "Starting core node $node_id (HTTP: $port, P2P: $p2p_port)..."
    
    # Start core node in background with environment variables
    RUST_LOG=info PARADIGM_PORT="$port" PARADIGM_DATA_DIR="$DATA_DIR/node-$node_id" \
    cargo run -p paradigm-core -- --port "$port" \
        > "$log_file" 2>&1 &
    
    local pid=$!
    echo $pid > "$pid_file"
    
    # Wait for startup
    sleep 3
    
    if is_running "$pid_file"; then
        print_status "Core node $node_id started successfully (PID: $pid)"
        return 0
    else
        print_error "Failed to start core node $node_id"
        return 1
    fi
}

# Start contributor
start_contributor() {
    local contrib_id="$1"
    local threads="${2:-4}"
    local node_port="${3:-8080}"
    
    local pid_file="$PID_DIR/contributor-$contrib_id.pid"
    local log_file="$LOG_DIR/contributor-$contrib_id.log"
    
    if is_running "$pid_file"; then
        print_warning "Contributor $contrib_id already running (PID: $(cat $pid_file))"
        return 0
    fi
    
    print_status "Starting contributor $contrib_id ($threads threads, connecting to port $node_port)..."
    
    # Start contributor in background with environment variables
    RUST_LOG=info PARADIGM_NODE_URL="http://127.0.0.1:$node_port" \
    cargo run -p paradigm-contributor -- --threads "$threads" --verbose \
        > "$log_file" 2>&1 &
    
    local pid=$!
    echo $pid > "$pid_file"
    
    # Wait for startup
    sleep 2
    
    if is_running "$pid_file"; then
        print_status "Contributor $contrib_id started successfully (PID: $pid)"
        return 0
    else
        print_error "Failed to start contributor $contrib_id"
        return 1
    fi
}

# Stop process
stop_process() {
    local name="$1"
    local pid_file="$2"
    
    if is_running "$pid_file"; then
        local pid=$(cat "$pid_file")
        print_status "Stopping $name (PID: $pid)..."
        
        kill "$pid"
        
        # Wait for graceful shutdown
        for i in {1..10}; do
            if ! kill -0 "$pid" 2>/dev/null; then
                break
            fi
            sleep 1
        done
        
        # Force kill if still running
        if kill -0 "$pid" 2>/dev/null; then
            print_warning "Force killing $name..."
            kill -9 "$pid"
        fi
        
        rm -f "$pid_file"
        print_status "$name stopped"
    else
        print_warning "$name not running"
    fi
}

# Show network status
show_status() {
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                      Network Status                         ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    echo -e "${GREEN}Core Nodes:${NC}"
    for pid_file in "$PID_DIR"/core-node-*.pid; do
        if [ -f "$pid_file" ]; then
            local node_id=$(basename "$pid_file" .pid | sed 's/core-node-//')
            if is_running "$pid_file"; then
                local pid=$(cat "$pid_file")
                echo -e "  ✅ Node $node_id (PID: $pid)"
            else
                echo -e "  ❌ Node $node_id (stopped)"
            fi
        fi
    done
    
    echo ""
    echo -e "${GREEN}Contributors:${NC}"
    for pid_file in "$PID_DIR"/contributor-*.pid; do
        if [ -f "$pid_file" ]; then
            local contrib_id=$(basename "$pid_file" .pid | sed 's/contributor-//')
            if is_running "$pid_file"; then
                local pid=$(cat "$pid_file")
                echo -e "  ✅ Contributor $contrib_id (PID: $pid)"
            else
                echo -e "  ❌ Contributor $contrib_id (stopped)"
            fi
        fi
    done
    
    echo ""
    echo -e "${GREEN}Log Files:${NC}"
    if [ -d "$LOG_DIR" ]; then
        ls -la "$LOG_DIR"/*.log 2>/dev/null || echo "  No log files found"
    fi
}

# Monitor network health
monitor_network() {
    print_status "Starting network monitoring..."
    
    while true; do
        clear
        show_status
        
        # Check if any core nodes failed
        local failed_nodes=()
        for pid_file in "$PID_DIR"/core-node-*.pid; do
            if [ -f "$pid_file" ]; then
                local node_id=$(basename "$pid_file" .pid | sed 's/core-node-//')
                if ! is_running "$pid_file"; then
                    failed_nodes+=("$node_id")
                fi
            fi
        done
        
        # Restart failed nodes
        for node_id in "${failed_nodes[@]}"; do
            print_warning "Restarting failed core node $node_id..."
            start_core_node "$node_id"
        done
        
        # Check if any contributors failed
        local failed_contributors=()
        for pid_file in "$PID_DIR"/contributor-*.pid; do
            if [ -f "$pid_file" ]; then
                local contrib_id=$(basename "$pid_file" .pid | sed 's/contributor-//')
                if ! is_running "$pid_file"; then
                    failed_contributors+=("$contrib_id")
                fi
            fi
        done
        
        # Restart failed contributors
        for contrib_id in "${failed_contributors[@]}"; do
            print_warning "Restarting failed contributor $contrib_id..."
            start_contributor "$contrib_id"
        done
        
        echo ""
        echo -e "${BLUE}Press Ctrl+C to stop monitoring${NC}"
        sleep 10
    done
}

# Cleanup function
cleanup() {
    echo ""
    print_status "Shutting down network..."
    
    # Stop all contributors
    for pid_file in "$PID_DIR"/contributor-*.pid; do
        if [ -f "$pid_file" ]; then
            local contrib_id=$(basename "$pid_file" .pid | sed 's/contributor-//')
            stop_process "Contributor $contrib_id" "$pid_file"
        fi
    done
    
    # Stop all core nodes
    for pid_file in "$PID_DIR"/core-node-*.pid; do
        if [ -f "$pid_file" ]; then
            local node_id=$(basename "$pid_file" .pid | sed 's/core-node-//')
            stop_process "Core Node $node_id" "$pid_file"
        fi
    done
    
    print_status "Network shutdown complete"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Main execution
case "${1:-start}" in
    "start")
        print_header
        print_status "Building Paradigm network..."
        cargo build --release
        
        print_status "Starting production network..."
        
        # Start core nodes
        start_core_node "1" "8080" "9000"
        sleep 2
        start_core_node "2" "8081" "9001"
        sleep 2
        start_core_node "3" "8082" "9002"
        
        # Wait for nodes to initialize
        sleep 5
        
        # Start contributors
        start_contributor "1" "8" "8080"
        start_contributor "2" "6" "8081"
        start_contributor "3" "4" "8082"
        start_contributor "4" "4" "8080"
        start_contributor "5" "2" "8081"
        
        print_status "Network started successfully!"
        show_status
        
        echo ""
        echo -e "${GREEN}Network URLs:${NC}"
        echo -e "  Node 1: http://127.0.0.1:8080"
        echo -e "  Node 2: http://127.0.0.1:8081"
        echo -e "  Node 3: http://127.0.0.1:8082"
        echo ""
        echo -e "${BLUE}Commands:${NC}"
        echo -e "  ./launch-network.sh monitor    # Monitor network health"
        echo -e "  ./launch-network.sh status     # Show current status"
        echo -e "  ./launch-network.sh stop       # Stop the network"
        echo -e "  ./launch-network.sh logs       # Show recent logs"
        ;;
        
    "monitor")
        print_header
        monitor_network
        ;;
        
    "status")
        show_status
        ;;
        
    "stop")
        cleanup
        ;;
        
    "logs")
        print_status "Recent network logs:"
        echo ""
        for log_file in "$LOG_DIR"/*.log; do
            if [ -f "$log_file" ]; then
                echo -e "${GREEN}=== $(basename "$log_file") ===${NC}"
                tail -n 20 "$log_file"
                echo ""
            fi
        done
        ;;
        
    "restart")
        print_status "Restarting network..."
        $0 stop
        sleep 3
        $0 start
        ;;
        
    *)
        echo "Usage: $0 {start|stop|status|monitor|logs|restart}"
        echo ""
        echo "Commands:"
        echo "  start    - Start the production network"
        echo "  stop     - Stop all network components"
        echo "  status   - Show current network status"
        echo "  monitor  - Monitor and auto-restart failed components"
        echo "  logs     - Show recent logs from all components"
        echo "  restart  - Restart the entire network"
        exit 1
        ;;
esac
