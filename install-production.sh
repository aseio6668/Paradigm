#!/bin/bash

# Paradigm Network Installation Script
# Sets up production environment for Paradigm cryptocurrency network

set -e

# Configuration
INSTALL_DIR="/opt/paradigm"
SERVICE_USER="paradigm"
DATA_DIR="/var/lib/paradigm"
LOG_DIR="/var/log/paradigm"
SERVICE_FILE="/etc/systemd/system/paradigm-network.service"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                 Paradigm Network Installer                  ║${NC}"
    echo -e "${BLUE}║               Production Environment Setup                   ║${NC}"
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

check_root() {
    if [ "$EUID" -ne 0 ]; then
        print_error "This script must be run as root"
        exit 1
    fi
}

install_dependencies() {
    print_status "Installing system dependencies..."
    
    # Update package lists
    apt-get update
    
    # Install required packages
    apt-get install -y \
        curl \
        build-essential \
        pkg-config \
        libssl-dev \
        sqlite3 \
        libsqlite3-dev \
        git \
        systemd \
        logrotate
    
    # Install Rust for the service user
    print_status "Installing Rust toolchain..."
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
}

create_user() {
    print_status "Creating service user '$SERVICE_USER'..."
    
    if ! id "$SERVICE_USER" &>/dev/null; then
        useradd --system --create-home --shell /bin/bash "$SERVICE_USER"
        usermod -aG systemd-journal "$SERVICE_USER"
    else
        print_warning "User '$SERVICE_USER' already exists"
    fi
}

setup_directories() {
    print_status "Setting up directories..."
    
    # Create installation directory
    mkdir -p "$INSTALL_DIR"
    chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
    
    # Create data directory
    mkdir -p "$DATA_DIR"
    chown "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR"
    chmod 750 "$DATA_DIR"
    
    # Create log directory
    mkdir -p "$LOG_DIR"
    chown "$SERVICE_USER:$SERVICE_USER" "$LOG_DIR"
    chmod 750 "$LOG_DIR"
}

install_paradigm() {
    print_status "Installing Paradigm network..."
    
    # Copy current directory to installation location
    cp -r . "$INSTALL_DIR/"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR"
    
    # Make scripts executable
    chmod +x "$INSTALL_DIR/launch-network.sh"
    chmod +x "$INSTALL_DIR/test-network.sh"
    
    # Build the project as the service user
    print_status "Building Paradigm (this may take a while)..."
    sudo -u "$SERVICE_USER" bash -c "cd $INSTALL_DIR && cargo build --release"
}

install_service() {
    print_status "Installing systemd service..."
    
    # Copy service file
    cp paradigm-network.service "$SERVICE_FILE"
    
    # Update paths in service file
    sed -i "s|/opt/paradigm|$INSTALL_DIR|g" "$SERVICE_FILE"
    sed -i "s|User=paradigm|User=$SERVICE_USER|g" "$SERVICE_FILE"
    sed -i "s|Group=paradigm|Group=$SERVICE_USER|g" "$SERVICE_FILE"
    
    # Reload systemd
    systemctl daemon-reload
    
    # Enable service
    systemctl enable paradigm-network.service
}

setup_logrotate() {
    print_status "Setting up log rotation..."
    
    cat > /etc/logrotate.d/paradigm << EOF
$LOG_DIR/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 $SERVICE_USER $SERVICE_USER
    postrotate
        systemctl reload paradigm-network.service
    endscript
}
EOF
}

setup_firewall() {
    print_status "Configuring firewall..."
    
    if command -v ufw &> /dev/null; then
        # Allow SSH
        ufw allow ssh
        
        # Allow Paradigm network ports
        ufw allow 8080:8082/tcp comment "Paradigm HTTP API"
        ufw allow 9000:9002/tcp comment "Paradigm P2P Network"
        
        # Enable firewall
        ufw --force enable
    elif command -v firewall-cmd &> /dev/null; then
        # CentOS/RHEL/Fedora
        firewall-cmd --permanent --add-port=8080-8082/tcp
        firewall-cmd --permanent --add-port=9000-9002/tcp
        firewall-cmd --reload
    else
        print_warning "No firewall detected. Please manually configure firewall rules."
    fi
}

create_monitoring_scripts() {
    print_status "Creating monitoring scripts..."
    
    # Health check script
    cat > "$INSTALL_DIR/health-check.sh" << 'EOF'
#!/bin/bash

# Paradigm Network Health Check
# Returns 0 if healthy, 1 if unhealthy

TIMEOUT=10
NODES=(8080 8081 8082)

for port in "${NODES[@]}"; do
    if ! curl -s --max-time $TIMEOUT "http://localhost:$port/health" > /dev/null; then
        echo "Node on port $port is unhealthy"
        exit 1
    fi
done

echo "All nodes healthy"
exit 0
EOF

    chmod +x "$INSTALL_DIR/health-check.sh"
    chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/health-check.sh"
    
    # Performance monitor script
    cat > "$INSTALL_DIR/monitor-performance.sh" << 'EOF'
#!/bin/bash

# Paradigm Network Performance Monitor
LOG_FILE="/var/log/paradigm/performance.log"

while true; do
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    # Check CPU and memory usage
    cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | sed 's/%us,//')
    memory_usage=$(free | grep Mem | awk '{printf "%.1f", $3/$2 * 100.0}')
    
    # Check disk usage
    disk_usage=$(df /var/lib/paradigm | tail -1 | awk '{print $5}' | sed 's/%//')
    
    # Check network connections
    connections=$(netstat -an | grep -E ':(8080|8081|8082|9000|9001|9002)' | wc -l)
    
    echo "$timestamp CPU: ${cpu_usage}% MEM: ${memory_usage}% DISK: ${disk_usage}% CONN: $connections" >> "$LOG_FILE"
    
    sleep 60
done
EOF

    chmod +x "$INSTALL_DIR/monitor-performance.sh"
    chown "$SERVICE_USER:$SERVICE_USER" "$INSTALL_DIR/monitor-performance.sh"
}

print_success() {
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                   Installation Complete!                    ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}Paradigm Network has been installed successfully!${NC}"
    echo ""
    echo -e "${BLUE}Service Management:${NC}"
    echo "  Start network:    sudo systemctl start paradigm-network"
    echo "  Stop network:     sudo systemctl stop paradigm-network"
    echo "  Check status:     sudo systemctl status paradigm-network"
    echo "  View logs:        sudo journalctl -u paradigm-network -f"
    echo ""
    echo -e "${BLUE}Manual Management:${NC}"
    echo "  cd $INSTALL_DIR"
    echo "  sudo -u $SERVICE_USER ./launch-network.sh start"
    echo "  sudo -u $SERVICE_USER ./launch-network.sh status"
    echo ""
    echo -e "${BLUE}Network URLs (after starting):${NC}"
    echo "  Node 1: http://localhost:8080"
    echo "  Node 2: http://localhost:8081"
    echo "  Node 3: http://localhost:8082"
    echo ""
    echo -e "${BLUE}Data Locations:${NC}"
    echo "  Installation: $INSTALL_DIR"
    echo "  Data:         $DATA_DIR"
    echo "  Logs:         $LOG_DIR"
    echo ""
    echo -e "${YELLOW}Next Steps:${NC}"
    echo "1. Review configuration in $INSTALL_DIR"
    echo "2. Start the service: sudo systemctl start paradigm-network"
    echo "3. Monitor logs: sudo journalctl -u paradigm-network -f"
    echo "4. Set up monitoring and alerting"
}

# Main installation process
main() {
    print_header
    
    check_root
    install_dependencies
    create_user
    setup_directories
    install_paradigm
    install_service
    setup_logrotate
    setup_firewall
    create_monitoring_scripts
    
    print_success
}

# Run main function
main "$@"
