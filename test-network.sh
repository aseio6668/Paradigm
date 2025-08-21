#!/bin/bash
# Paradigm Network Test Launcher
# This script demonstrates multiple clients connecting and synchronizing

echo "🚀 Starting Paradigm Network Test"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to start a node
start_node() {
    local port=$1
    local data_dir=$2
    local node_name=$3
    
    echo -e "${BLUE}🔷 Starting Node: $node_name (Port: $port)${NC}"
    
    # Create separate data directory for each node
    mkdir -p "./test-data/$data_dir"
    
    # Start the node with specific port and data directory
    cargo run -p paradigm-core -- \
        --port $port \
        --data-dir "./test-data/$data_dir" \
        --node-name "$node_name" \
        > "./test-data/$data_dir/node.log" 2>&1 &
        
    echo $! > "./test-data/$data_dir/node.pid"
    echo -e "${GREEN}✅ Node $node_name started (PID: $(cat ./test-data/$data_dir/node.pid))${NC}"
}

# Function to start a contributor
start_contributor() {
    local node_address=$1
    local contributor_name=$2
    local threads=$3
    
    echo -e "${YELLOW}🔶 Starting Contributor: $contributor_name -> $node_address${NC}"
    
    # Start contributor connecting to specific node
    cargo run -p paradigm-contributor -- \
        --node-address $node_address \
        --threads $threads \
        --verbose \
        > "./test-data/contributor-$contributor_name.log" 2>&1 &
        
    echo $! > "./test-data/contributor-$contributor_name.pid"
    echo -e "${GREEN}✅ Contributor $contributor_name started (PID: $(cat ./test-data/contributor-$contributor_name.pid))${NC}"
}

# Function to monitor logs
monitor_network() {
    echo -e "${BLUE}📊 Network Activity Monitor${NC}"
    echo "Press Ctrl+C to stop monitoring..."
    
    # Monitor all log files
    tail -f ./test-data/*/node.log ./test-data/contributor-*.log 2>/dev/null | \
    while read line; do
        case "$line" in
            *"node.log"*) echo -e "${BLUE}[NODE]${NC} $line" ;;
            *"contributor"*) echo -e "${YELLOW}[CONTRIBUTOR]${NC} $line" ;;
            *) echo "$line" ;;
        esac
    done
}

# Function to stop all processes
cleanup() {
    echo -e "\n${RED}🛑 Stopping all Paradigm processes...${NC}"
    
    # Kill all nodes and contributors
    for pidfile in ./test-data/*/*.pid ./test-data/contributor-*.pid; do
        if [ -f "$pidfile" ]; then
            pid=$(cat "$pidfile")
            if kill -0 "$pid" 2>/dev/null; then
                echo "Stopping process $pid"
                kill "$pid" 2>/dev/null
            fi
            rm -f "$pidfile"
        fi
    done
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
    exit 0
}

# Set up cleanup on script exit
trap cleanup EXIT INT TERM

# Main execution
case "${1:-demo}" in
    "single")
        echo "🔷 Single Node + Single Contributor Test"
        start_node 8080 "node1" "MainNode"
        sleep 3
        start_contributor "127.0.0.1:8080" "worker1" 4
        monitor_network
        ;;
        
    "dual")
        echo "🔷 Dual Node + Dual Contributor Test"
        start_node 8080 "node1" "Node-Alpha"
        start_node 8081 "node2" "Node-Beta"
        sleep 5
        start_contributor "127.0.0.1:8080" "alpha-worker" 4
        start_contributor "127.0.0.1:8081" "beta-worker" 4
        monitor_network
        ;;
        
    "network")
        echo "🔷 Full Network Test (3 Nodes + 5 Contributors)"
        start_node 8080 "node1" "Genesis"
        start_node 8081 "node2" "Alpha"
        start_node 8082 "node3" "Beta"
        sleep 7
        start_contributor "127.0.0.1:8080" "genesis-worker1" 2
        start_contributor "127.0.0.1:8080" "genesis-worker2" 2
        start_contributor "127.0.0.1:8081" "alpha-worker1" 3
        start_contributor "127.0.0.1:8082" "beta-worker1" 3
        start_contributor "127.0.0.1:8082" "beta-worker2" 4
        monitor_network
        ;;
        
    "demo"|*)
        echo "🔷 Demo Mode: Single Node + Multiple Contributors"
        echo "This demonstrates multiple contributors connecting to one node"
        start_node 8080 "demo-node" "DemoNode"
        sleep 3
        start_contributor "127.0.0.1:8080" "fast-worker" 8
        sleep 2
        start_contributor "127.0.0.1:8080" "balanced-worker" 4
        sleep 2
        start_contributor "127.0.0.1:8080" "efficient-worker" 2
        monitor_network
        ;;
esac
