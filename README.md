# 🚀 Paradigm Cryptocurrency

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/paradigm-network/paradigm)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Discord](https://img.shields.io/badge/discord-join-7289da)](https://discord.gg/paradigm)

**A next-generation cryptocurrency platform powered by AI governance and quantum-resistant security**

[🚀 Quick Start](#quick-start) • [📖 Documentation](docs/) • [🔧 Build](#building) • [🌐 Network](#network) • [💰 Mining](#earning-par-tokens)

---

## 🌟 What is Paradigm?

Paradigm is a revolutionary cryptocurrency featuring:

- **🤖 AI-Powered Governance** - Autonomous network decisions and smart contract optimization
- **🔒 Quantum-Resistant Security** - Future-proof cryptography and post-quantum algorithms  
- **⚡ ML Task Processing** - Earn PAR tokens by contributing machine learning compute power
- **🌐 Cross-Chain Interoperability** - Bridge to Bitcoin, Ethereum, and other networks
- **📊 Real-Time Analytics** - Comprehensive network monitoring and insights

## 🚀 Quick Start

### Windows

```batch
# Build the system
build.bat

# Start earning PAR tokens
target\release\paradigm-contributor.exe

# Start your own network
start-network.bat
```

### Linux/macOS

```bash
# Build the system
./build.sh

# Start earning PAR tokens  
./target/release/paradigm-contributor

# Start your own network
./start-network.sh
```

## 🔧 Building

### Prerequisites

- **Rust 1.75+** - [Install Rust](https://rustup.rs/)
- **Git** - For version control
- **protoc** (optional) - For gRPC features

### Build Commands

| Platform | Fast Build | Advanced Build |
|----------|------------|----------------|
| Windows | `build.bat` | `build-advanced.bat` |
| Linux/macOS | `./build.sh` | Coming soon |

**Built components:**
- `paradigm-core` - Network node
- `paradigm-wallet` - Multi-signature wallet
- `paradigm-contributor` - ML task processor  
- `paradigm-sdk` - Developer library

## 🌐 Network

### Start a Bootstrap Node

**Windows:**
```batch
start-network.bat
```

**Linux/macOS:**
```bash
./start-network.sh
```

### Join an Existing Network

```bash
# Replace with actual bootstrap node address
paradigm-core --bootstrap-peers /ip4/YOUR_IP/tcp/8080/p2p/PEER_ID
```

### Production Network

**Windows:**
```batch
launch-network.bat        # Start network
launch-network.bat status # Check status  
launch-network.bat stop   # Stop network
```

**Linux/macOS:**
```bash
./launch-network.sh        # Start network
./launch-network.sh status # Check status
./launch-network.sh stop   # Stop network
```

## 💰 Earning PAR Tokens

Start contributing ML compute power to earn PAR tokens:

```bash
# Windows
target\release\paradigm-contributor.exe

# Linux/macOS
./target/release/paradigm-contributor
```

**How it works:**
1. Your computer processes AI/ML tasks
2. You earn PAR tokens for completed work
3. AI governance system validates contributions
4. Rewards are distributed automatically

## 📁 Project Structure

```
Paradigm[CC]/
├── paradigm-core/          # Main network node
├── paradigm-wallet/        # Multi-sig wallet
├── paradigm-contributor/   # ML task processor
├── paradigm-sdk/           # Developer SDK
├── docs/                   # Documentation
├── build.bat               # Windows build script
├── build.sh                # Linux/macOS build script
├── start-network.*         # Bootstrap node launchers
└── launch-network.*        # Production network launchers
```

## 📖 Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory:

- **[Quick Start Guide](docs/QUICKSTART.md)** - Get started in 5 minutes
- **[Network Setup](docs/START-NETWORK.md)** - Launch your own network
- **[Advanced Features](docs/ADVANCED_FEATURES_DEMO.md)** - AI governance demos
- **[Developer Guide](docs/DEVELOPER_GUIDE.md)** - Build applications on Paradigm
- **[Production Deployment](docs/PRODUCTION.md)** - Enterprise deployment guide

## 🔒 Security

Paradigm implements multiple layers of security:

- **Quantum-resistant cryptography** - Future-proof against quantum computers
- **Zero-knowledge proofs** - Privacy-preserving transactions
- **Multi-signature wallets** - Enterprise-grade asset protection
- **AI-powered anomaly detection** - Real-time threat monitoring

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

## 📊 Network Stats

- **Consensus**: Proof-of-AI-Work (PoAW)
- **Block Time**: ~10 seconds  
- **Total Supply**: 21M PAR tokens
- **Decimals**: 8
- **Network**: P2P with libp2p

## 🆘 Support

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/paradigm-network/paradigm/issues)
- **Discord**: [Join our community](https://discord.gg/paradigm)
- **Email**: support@paradigm.network

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**🚀 Welcome to the future of cryptocurrency with Paradigm!**