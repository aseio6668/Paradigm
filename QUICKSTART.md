# Quick Start Guide

This guide helps you get started with Paradigm cryptocurrency quickly.

## Installation

### Option 1: Download Prebuilt Binaries (Recommended)
1. Download the latest release from [GitHub Releases]
2. Extract the archive
3. Run the installer

### Option 2: Build from Source
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build Paradigm
git clone https://github.com/paradigm-crypto/paradigm.git
cd paradigm
cargo build --release
```

## Quick Start

### 1. Create a Wallet
```bash
# GUI Wallet (easiest)
./paradigm-wallet

# Or via command line
./paradigm-core --create-wallet
```

### 2. Start a Node
```bash
# Regular node
./paradigm-core

# Contributor node (earns rewards)
./paradigm-core --contributor

# Contributor with custom settings
./paradigm-contributor --wallet-address PAR1234... --use-gpu
```

### 3. Send Transactions
Use the GUI wallet or CLI:
```bash
./paradigm-core send --to PAR5678... --amount 10.5 --fee 0.001
```

## Network Overview

- **Currency**: PAR (Paradigm)
- **Decimals**: 8
- **Initial Supply**: 8,000,000,000 PAR
- **Consensus**: ML-based Proof of Contribution
- **Transaction Time**: Near-instant
- **Block Time**: Timestamp-based (no traditional blocks)

## Earning Rewards

Contribute computational power for ML tasks:

1. Run a contributor node: `./paradigm-contributor --wallet-address YOUR_ADDRESS`
2. Your computer processes ML tasks automatically
3. Earn PAR rewards based on contribution quality
4. AI governance distributes rewards fairly

## Network Roles

### Regular Users
- Send/receive PAR
- Use GUI or web wallet
- Near-instant transactions

### Contributors
- Process ML tasks
- Earn PAR rewards
- Help secure the network
- Provide computational resources

### Developers
- Build on Paradigm
- Create smart contracts
- Develop autonomous agents
- Integrate with APIs

## Key Features

### ML-Based Consensus
- No energy-intensive mining
- Rewards useful computational work
- Self-improving network
- AI-governed distribution

### Fast Synchronization
- Timestamp-based data chunks
- Quick client bootstrapping
- Minimal download requirements
- Efficient network updates

### Autonomous Governance
- AI-controlled reward distribution
- Self-optimizing parameters
- Proposal-based upgrades
- Community-driven evolution

## Wallet Security

### Best Practices
- Backup your seed phrase safely
- Never share private keys
- Use hardware wallets when available
- Keep software updated

### Seed Phrase Storage
```
Store your 12-word seed phrase securely:
- Write on paper (never digital)
- Store in multiple safe locations
- Consider metal backup plates
- Never photograph or email
```

## Configuration

### Node Configuration
```toml
# paradigm.toml
[network]
port = 8080
bootstrap_peers = ["127.0.0.1:8080"]

[contributor]
max_tasks = 4
use_gpu = true
task_types = ["Oracle", "NetworkOptimization"]

[wallet]
data_dir = "./paradigm-data"
auto_sync = true
```

### Environment Variables
```bash
export PARADIGM_DATA_DIR="/path/to/data"
export PARADIGM_LOG_LEVEL="info"
export PARADIGM_NETWORK="mainnet"
```

## Troubleshooting

### Common Issues

**Q: Node won't start**
- Check firewall settings
- Ensure port 8080 is available
- Verify data directory permissions

**Q: Wallet shows zero balance**
- Wait for network synchronization
- Check network connection
- Verify wallet address

**Q: ML tasks not processing**
- Check hardware requirements
- Verify GPU drivers (if using GPU)
- Ensure sufficient memory

**Q: Slow transaction confirmation**
- Check network connectivity
- Verify sufficient fee
- Wait for network propagation

### Getting Help
- Check logs: `./paradigm-core --log-level debug`
- Join Discord: [Coming Soon]
- GitHub Issues: [Repository Link]
- Documentation: `./docs/`

## Advanced Usage

### Pool Contribution
```bash
# Join a contributor pool
./paradigm-contributor --pool-address POOL_ADDRESS --wallet-address YOUR_ADDRESS
```

### Smart Contracts
```bash
# Deploy a contract
./paradigm-core deploy-contract --file contract.wasm --gas-limit 100000
```

### API Integration
```bash
# REST API
curl http://localhost:8080/api/v1/balance/PAR1234...

# WebSocket
wscat -c ws://localhost:8080/ws
```

## Network Statistics

Monitor network health:
- Active contributors
- Task completion rate
- Network difficulty
- Reward distribution
- Transaction throughput

## Next Steps

1. **For Users**: Explore the GUI wallet features
2. **For Contributors**: Optimize your setup for maximum rewards
3. **For Developers**: Read the API documentation
4. **For Community**: Join discussions and proposals

## Resources

- [Full Documentation](docs/)
- [API Reference](docs/api.md)
- [Developer Guide](docs/development.md)
- [Network Statistics](https://stats.paradigm.network)
- [Community Discord](https://discord.gg/paradigm)

Welcome to the future of cryptocurrency! 🚀
