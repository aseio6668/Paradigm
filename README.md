# Paradigm (PAR) Cryptocurrency

## Overview
Paradigm is a revolutionary cryptocurrency that replaces traditional mining with machine learning-based consensus and computational contributions. The network incentivizes users to contribute computational power for ML tasks, autonomous agents, and oracle services.

## Key Features
- **No Premine**: Fair launch with AI-governed distribution
- **ML-Based Consensus**: Rewards for computational contributions instead of mining
- **Near-Instant Transactions**: Timestamp-based synchronization
- **Autonomous Governance**: AI-controlled reward distribution
- **Smart Contracts & Oracles**: Built-in support for autonomous agents
- **Evolving Network**: Self-improving through ML contributions

## Technical Specifications
- **Currency Symbol**: PAR
- **Decimal Precision**: 8 decimals
- **Initial Supply**: 8,000,000,000 PAR
- **First Year Distribution**: 1,000,000,000 PAR (AI-governed)
- **Transaction Speed**: Near-instant
- **Network Type**: Peer-to-peer with ML task coordination

## Project Structure
```
paradigm-core/          # Core blockchain implementation (Rust)
paradigm-wallet/        # GUI wallet application (Rust + egui)
paradigm-contributor/   # ML contributor client (Rust + Python)
paradigm-web/          # Web wallet interface (React/TypeScript).
paradigm-contracts/    # Smart contract engine (Rust)
paradigm-agents/       # Autonomous agent framework (Rust + ML)
paradigm-installer/    # Cross-platform installer
docs/                  # Documentation
tests/                 # Integration tests
```

## Getting Started

### Quick Start (Single Node + Contributors)
1. Install Rust (latest stable)
2. Clone the repository
3. Build the project: `cargo build --release`
4. Run the test network:
   - **Windows**: Double-click `test-network.bat`
   - **Linux/Mac**: `./test-network.sh demo`

### Manual Setup
1. Start a Paradigm node:
   ```bash
   cargo run -p paradigm-core
   ```

2. In separate terminals, start contributors:
   ```bash
   # Contributor 1 (8 threads)
   cargo run -p paradigm-contributor -- --threads 8 --verbose
   
   # Contributor 2 (4 threads) 
   cargo run -p paradigm-contributor -- --threads 4 --verbose
   
   # Contributor 3 (2 threads)
   cargo run -p paradigm-contributor -- --threads 2 --verbose
   ```

### Network Test Modes
- `./test-network.sh single` - One node + one contributor
- `./test-network.sh dual` - Two nodes + two contributors  
- `./test-network.sh network` - Three nodes + five contributors
- `./test-network.sh demo` - One node + three contributors (recommended)

### Production Network Deployment

For production environments that require persistent operation and synchronization:

#### Quick Production Start
```bash
# Windows
launch-network.bat start

# Linux/Mac  
./launch-network.sh start
```

#### Production Features
- **Multi-node cluster**: 3 core nodes with load balancing
- **Auto-restart**: Failed components automatically restart
- **Monitoring**: Real-time network health monitoring
- **Logging**: Centralized logging with rotation
- **Systemd integration**: Linux service management
- **Performance monitoring**: CPU, memory, and network metrics

#### Production Commands
```bash
# Start production network
./launch-network.sh start

# Monitor network health (auto-restart failed components)
./launch-network.sh monitor

# Check current status
./launch-network.sh status

# View recent logs
./launch-network.sh logs

# Stop network
./launch-network.sh stop

# Restart entire network
./launch-network.sh restart
```

#### Linux Server Installation
```bash
# Install as system service
sudo ./install-production.sh

# Manage via systemd
sudo systemctl start paradigm-network
sudo systemctl status paradigm-network
sudo journalctl -u paradigm-network -f
```

#### Network URLs (Production)
- **Node 1**: http://localhost:8080
- **Node 2**: http://localhost:8081  
- **Node 3**: http://localhost:8082

### What You'll See
- **Nodes**: Database initialization, P2P network startup, ML task coordination
- **Contributors**: Task processing, PAR rewards, performance metrics
- **Network Sync**: Real-time task distribution and reward synchronization
- **Production**: Automatic failover, health monitoring, performance metrics

## Contributing
Paradigm is designed to evolve through community contributions. See `CONTRIBUTING.md` for guidelines.

## License
MIT License - see LICENSE file for details
