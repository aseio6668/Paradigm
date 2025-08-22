#!/bin/bash

# Paradigm Network - One-Line Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/paradigm-network/paradigm/main/install.sh | bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Global variables
PARADIGM_HOME="${PARADIGM_HOME:-$HOME/.paradigm}"
PARADIGM_VERSION="${PARADIGM_VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
TEMP_DIR="/tmp/paradigm-install-$$"

# Utility functions
log() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
    exit 1
}

# Clean up on exit
cleanup() {
    rm -rf "$TEMP_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# Detect OS and architecture
detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="darwin"
            ;;
        MINGW64*|MSYS*|CYGWIN*)
            os="windows"
            ;;
        *)
            error "Unsupported operating system: $(uname -s)"
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)
            arch="amd64"
            ;;
        arm64|aarch64)
            arch="arm64"
            ;;
        armv7l)
            arch="arm"
            ;;
        *)
            error "Unsupported architecture: $(uname -m)"
            ;;
    esac
    
    echo "${os}-${arch}"
}

# Check system requirements
check_requirements() {
    log "Checking system requirements..."
    
    local missing_deps=()
    local required_commands=("curl" "tar" "git")
    
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        error "Missing required dependencies: ${missing_deps[*]}. Please install them first."
    fi
    
    # Check available disk space (need at least 2GB)
    local available_space
    if command -v df &> /dev/null; then
        available_space=$(df -BG . | awk 'NR==2 {print $4}' | sed 's/G//')
        if [ "$available_space" -lt 2 ]; then
            warn "Low disk space detected (${available_space}GB). At least 2GB recommended."
        fi
    fi
    
    log "System requirements check passed ✓"
}

# Download and install Paradigm binaries
install_binaries() {
    log "Installing Paradigm binaries..."
    
    local platform
    platform=$(detect_platform)
    
    mkdir -p "$TEMP_DIR"
    cd "$TEMP_DIR"
    
    # Download release
    local download_url="https://github.com/paradigm-network/paradigm/releases/latest/download/paradigm-${platform}.tar.gz"
    log "Downloading from: $download_url"
    
    curl -fsSL "$download_url" -o paradigm.tar.gz
    tar -xzf paradigm.tar.gz
    
    # Install binaries
    log "Installing binaries to $INSTALL_DIR..."
    
    if [ -w "$INSTALL_DIR" ]; then
        cp paradigm-core paradigm-cli paradigm-wallet "$INSTALL_DIR/"
    else
        sudo cp paradigm-core paradigm-cli paradigm-wallet "$INSTALL_DIR/"
    fi
    
    chmod +x "$INSTALL_DIR/paradigm-core" "$INSTALL_DIR/paradigm-cli" "$INSTALL_DIR/paradigm-wallet"
    
    log "Binaries installed successfully ✓"
}

# Set up Paradigm directory and configuration
setup_paradigm_home() {
    log "Setting up Paradigm home directory..."
    
    mkdir -p "$PARADIGM_HOME"/{config,data,logs,backups,keys}
    
    # Create default configuration
    cat > "$PARADIGM_HOME/config/node.toml" << 'EOF'
[node]
data_dir = "~/.paradigm/data"
log_level = "info"
enable_metrics = true
metrics_port = 9090

[network]
listen_port = 30303
max_peers = 50
discovery_enabled = true

[rpc]
enabled = true
host = "127.0.0.1"
port = 8545
cors_origins = ["http://localhost:3000"]

[websocket]
enabled = true
host = "127.0.0.1" 
port = 8546
max_connections = 100

[consensus]
enable_ml_consensus = true
validator_timeout = 30
block_time = 12

[storage]
database_url = "sqlite://~/.paradigm/data/paradigm.db"
cache_size = "1GB"
EOF

    # Create systemd service file (Linux only)
    if [[ "$(uname -s)" == "Linux" && -d "/etc/systemd/system" ]]; then
        log "Creating systemd service..."
        
        sudo tee /etc/systemd/system/paradigm.service > /dev/null << EOF
[Unit]
Description=Paradigm Network Node
After=network.target
Wants=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$PARADIGM_HOME
ExecStart=$INSTALL_DIR/paradigm-core --config $PARADIGM_HOME/config/node.toml
Restart=always
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF
        
        sudo systemctl daemon-reload
        log "Systemd service created ✓"
    fi
    
    log "Paradigm home setup complete ✓"
}

# Initialize the network
initialize_network() {
    log "Initializing Paradigm network..."
    
    # Generate node key if it doesn't exist
    if [ ! -f "$PARADIGM_HOME/keys/node_key" ]; then
        "$INSTALL_DIR/paradigm-core" keygen --output "$PARADIGM_HOME/keys/node_key"
        log "Node key generated ✓"
    fi
    
    # Initialize genesis if needed
    if [ ! -f "$PARADIGM_HOME/data/genesis.json" ]; then
        "$INSTALL_DIR/paradigm-core" init --data-dir "$PARADIGM_HOME/data"
        log "Genesis block initialized ✓"
    fi
    
    log "Network initialization complete ✓"
}

# Install Docker and Docker Compose (optional)
install_docker() {
    if command -v docker &> /dev/null; then
        log "Docker already installed, skipping..."
        return
    fi
    
    read -p "Would you like to install Docker for containerized deployment? (y/N): " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        return
    fi
    
    log "Installing Docker..."
    
    case "$(uname -s)" in
        Linux*)
            # Install Docker on Linux
            curl -fsSL https://get.docker.com -o get-docker.sh
            sudo sh get-docker.sh
            sudo usermod -aG docker "$USER"
            
            # Install Docker Compose
            sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
            sudo chmod +x /usr/local/bin/docker-compose
            
            log "Docker installed ✓"
            warn "Please log out and back in for Docker group membership to take effect."
            ;;
        Darwin*)
            warn "Please install Docker Desktop for Mac from https://docker.com/products/docker-desktop"
            ;;
        *)
            warn "Please install Docker manually for your platform"
            ;;
    esac
}

# Download Docker Compose configuration
setup_docker_compose() {
    if ! command -v docker &> /dev/null; then
        return
    fi
    
    log "Setting up Docker Compose configuration..."
    
    curl -fsSL https://raw.githubusercontent.com/paradigm-network/paradigm/main/docker-compose.yml -o "$PARADIGM_HOME/docker-compose.yml"
    curl -fsSL https://raw.githubusercontent.com/paradigm-network/paradigm/main/docker-compose.prod.yml -o "$PARADIGM_HOME/docker-compose.prod.yml"
    curl -fsSL https://raw.githubusercontent.com/paradigm-network/paradigm/main/.env.example -o "$PARADIGM_HOME/.env.example"
    
    cp "$PARADIGM_HOME/.env.example" "$PARADIGM_HOME/.env"
    
    log "Docker Compose configuration ready ✓"
}

# Set up shell integration
setup_shell_integration() {
    log "Setting up shell integration..."
    
    local shell_rc
    case "$SHELL" in
        */bash)
            shell_rc="$HOME/.bashrc"
            ;;
        */zsh)
            shell_rc="$HOME/.zshrc"
            ;;
        */fish)
            shell_rc="$HOME/.config/fish/config.fish"
            ;;
        *)
            warn "Unsupported shell: $SHELL. Please add $INSTALL_DIR to your PATH manually."
            return
            ;;
    esac
    
    # Add to PATH if not already there
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_rc"
        log "Added $INSTALL_DIR to PATH in $shell_rc"
    fi
    
    # Add Paradigm home environment variable
    echo "export PARADIGM_HOME=\"$PARADIGM_HOME\"" >> "$shell_rc"
    
    log "Shell integration complete ✓"
}

# Print installation summary
print_summary() {
    log "🎉 Paradigm Network installation completed successfully!"
    echo
    echo -e "${BLUE}Installation Summary:${NC}"
    echo "  • Binaries installed to: $INSTALL_DIR"
    echo "  • Configuration directory: $PARADIGM_HOME"
    echo "  • Node configuration: $PARADIGM_HOME/config/node.toml"
    echo
    echo -e "${BLUE}Quick Start:${NC}"
    echo "  1. Start the node:"
    echo "     paradigm-core --config $PARADIGM_HOME/config/node.toml"
    echo
    echo "  2. Or use systemd (Linux):"
    echo "     sudo systemctl enable --now paradigm"
    echo
    echo "  3. Check node status:"
    echo "     paradigm-cli status"
    echo
    echo "  4. Create a wallet:"
    echo "     paradigm-wallet create"
    echo
    
    if command -v docker &> /dev/null; then
        echo -e "${BLUE}Docker Deployment:${NC}"
        echo "  • Start with Docker Compose:"
        echo "    cd $PARADIGM_HOME && docker-compose up -d"
        echo
    fi
    
    echo -e "${BLUE}Useful Links:${NC}"
    echo "  • Documentation: https://docs.paradigm.network"
    echo "  • Discord: https://discord.gg/paradigm"  
    echo "  • GitHub: https://github.com/paradigm-network/paradigm"
    echo
    warn "Please restart your shell or run 'source ~/.bashrc' to update your PATH"
    log "Happy mining! 🚀"
}

# Main installation flow
main() {
    echo -e "${GREEN}"
    cat << 'EOF'
    ____                   __  _               
   / __ \____ __________ _/ /_(_)___ _____ ___ 
  / /_/ / __ `/ ___/ __ `/ __/ / __ `/ __ `__ \
 / ____/ /_/ / /  / /_/ / /_/ / /_/ / / / / / /
/_/    \__,_/_/   \__,_/\__/_/\__, /_/ /_/ /_/ 
                            /____/           
                 Network Installation Script
EOF
    echo -e "${NC}"
    
    log "Starting Paradigm Network installation..."
    
    # Installation steps
    check_requirements
    install_binaries
    setup_paradigm_home
    initialize_network
    install_docker
    setup_docker_compose
    setup_shell_integration
    
    print_summary
}

# Run main function
main "$@"