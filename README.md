# 🚀 Paradigm Cryptocurrency

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/paradigm-network/paradigm)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Security](https://img.shields.io/badge/security-enterprise-green)](#security)
[![Discord](https://img.shields.io/badge/discord-join-7289da)](https://discord.gg/paradigm)

**A next-generation AI-governed cryptocurrency with enterprise-grade security and near-zero transaction fees**

[🚀 Quick Start](#quick-start) • [📖 Documentation](docs/) • [🔧 Build](#building) • [🌐 Network](#network-setup) • [💰 Wallet](#wallet-usage) • [🛡️ Security](#security-features)

---

## 🌟 What Makes Paradigm Revolutionary?

### 🤖 **AI-Powered Governance**
- **Autonomous Decision Engine**: Self-governing network that adapts to market conditions
- **Neural Consensus System**: Distributed AI validation for maximum security
- **Predictive Governance**: Anticipates network needs and auto-adjusts parameters
- **Dynamic Fee Optimization**: AI-calculated near-zero fees (as low as 0.01% for small transactions)

### 🛡️ **Enterprise-Grade Security**
- **TLS/mTLS Encryption**: Military-grade secure communications
- **Hardware Security Module (HSM)**: Support for AWS CloudHSM, Azure HSM, YubiKey, Ledger
- **Multi-Signature Treasury Wallets**: Threshold signatures with weighted voting
- **Formal Transaction Validation**: Comprehensive rule-based verification with risk scoring
- **DDoS Protection**: Advanced rate limiting and threat detection

### ⚡ **Performance & Efficiency**
- **Near-Zero Fees**: Micro-transactions cost as little as 0.0001 PAR
- **High Throughput**: Optimized storage and parallel processing
- **Real-Time Analytics**: Live network monitoring and insights
- **Cross-Chain Compatibility**: Bridge to Bitcoin, Ethereum, and other networks

---

## 🚀 Quick Start

### 1. **Build the System**

**Windows:**
```batch
# Fast build (recommended)
build.bat

# Advanced build with all features
build-advanced.bat
```

**Linux/macOS:**
```bash
# Install dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build system
cargo build --release
```

### 2. **Start Your Network**

**Create Genesis Network:**
```bash
# Windows
paradigm-core.exe --data-dir ./my-network --genesis genesis-config.toml --enable-api

# Linux/macOS  
./paradigm-core --data-dir ./my-network --genesis genesis-config.toml --enable-api
```

**Join Existing Network:**
```bash
# Replace YOUR_IP with bootstrap node IP
paradigm-core --data-dir ./my-data --addnode BOOTSTRAP_IP:8080
```

### 3. **Use the Wallet**

```bash
# Create wallet
paradigm-wallet create my_wallet

# Check balance
paradigm-wallet balance PAR1a2b3c4...

# Send PAR (AI-optimized fees)
paradigm-wallet send PAR1from... PAR1to... 0.1

# Advanced: Multi-signature treasury
paradigm-wallet create-multisig "Treasury" 3 5  # 3-of-5 signatures required
```

---

## 🔧 Building

### **Prerequisites**
- **Rust 1.75+** - [Install Rust](https://rustup.rs/)
- **Git** - For version control  
- **protoc** (optional) - For gRPC features

### **Build Commands**

| Platform | Command | Description |
|----------|---------|-------------|
| **Windows** | `build.bat` | Fast build with core features |
| **Windows** | `build-advanced.bat` | Full feature build |
| **Linux/macOS** | `cargo build --release` | Standard build |
| **All** | `cargo build --all-features` | Complete build |

### **What Gets Built**

| Binary | Purpose | Key Features |
|--------|---------|--------------|
| `paradigm-core` | Network node | AI governance, HSM support, API server |
| `paradigm-wallet` | Wallet CLI | Multi-sig, dynamic fees, hardware wallet support |
| `paradigm-contributor` | ML processor | Earn PAR tokens through compute contribution |

---

## 🌐 Network Setup

### **1. Bootstrap Node (Network Creator)**

```bash
# Create new network from genesis
paradigm-core \
  --data-dir ./genesis-data \
  --genesis genesis-config.toml \
  --enable-api \
  --api-port 8080 \
  --port 8080
```

### **2. Peer Nodes (Network Joiners)**

```bash
# Connect to existing network
paradigm-core \
  --data-dir ./peer-data \
  --addnode BOOTSTRAP_IP:8080 \
  --enable-api \
  --api-port 8080
```

### **3. Network Configuration**

**Router Setup (for external connections):**
- Open port `8080` (or your chosen port) for TCP traffic
- Configure port forwarding: External:8080 → Internal:8080

**Firewall Configuration:**
```bash
# Linux (ufw)
sudo ufw allow 8080/tcp

# Windows Firewall
# Allow paradigm-core.exe through Windows Defender Firewall
```

**Advanced Peer Discovery:**
```bash
# Multiple bootstrap peers
paradigm-core --addnode "192.168.1.100:8080;192.168.1.101:8080"

# Load peers from file  
paradigm-core --addnodefile peers.txt

# Use peer file format:
# peers.txt:
# 192.168.1.100:8080
# 192.168.1.101:8080
```

---

## 💰 Wallet Usage

### **Basic Operations**

```bash
# Create new wallet
paradigm-wallet create my_personal_wallet

# List all addresses in wallet
paradigm-wallet list

# Check balance (shows AI-optimized fee estimations)
paradigm-wallet balance PAR1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0

# Send with AI fee optimization  
paradigm-wallet send \
  PAR1from... \
  PAR1to... \
  0.1 \
  "Hello Paradigm"
```

### **Advanced Features**

**Multi-Signature Treasury:**
```bash
# Create 3-of-5 multi-signature treasury wallet
paradigm-wallet create-multisig \
  "Company Treasury" \
  --threshold 3 \
  --signers cfo_key.pub,cto_key.pub,ceo_key.pub,board1.pub,board2.pub

# Propose transaction (requires multiple approvals)
paradigm-wallet propose-tx \
  --wallet treasury_id \
  --to PAR1recipient... \
  --amount 1000.0 \
  --message "Q4 Bonus Distribution"

# Sign pending transaction
paradigm-wallet sign-tx --transaction-id tx_123... --signer-key cfo_key.priv

# Execute when threshold met
paradigm-wallet execute-tx --transaction-id tx_123...
```

**AI Fee Analysis:**
```bash
# Get detailed fee breakdown
paradigm-wallet estimate-fee 10.5
# Output:
# 💡 Fee calculation:
#    Base rate: 0.1000%
#    Network congestion: 20.0%
#    Urgency multiplier: 1.0x
#    Near-zero optimization: -0.2500%
#    Final rate: 0.0750% (0.007875 PAR)
```

---

## 🛡️ Security Features

### **1. TLS/mTLS Encryption**
- **Development**: No encryption (local testing only)
- **Standard**: TLS encryption for secure communications
- **Enterprise**: Mutual TLS with certificate-based authentication

```bash
# Start with enterprise security
paradigm-core --security-level MutualTLS --enable-api
```

### **2. Hardware Security Module (HSM) Integration**

**Supported HSM Types:**
- AWS CloudHSM
- Azure Key Vault HSM  
- PKCS#11 hardware modules
- YubiKey hardware tokens
- Ledger hardware wallets
- Software HSM (development/testing)

```toml
# network-config.toml
[hsm]
enabled = true
hsm_type = "AWS_CloudHSM"
connection_string = "cluster-xyz.cloudhsm.region.amazonaws.com:2225"
credentials = "/path/to/hsm-credentials.json"
```

### **3. Multi-Signature Treasury Wallets**

**Features:**
- **Threshold Signatures**: M-of-N approval requirements
- **Weighted Voting**: Different signature weights per signer
- **Role-Based Access**: Treasury Manager, Emergency Recovery, etc.
- **Time-Limited Proposals**: Auto-expiring transaction proposals
- **Audit Trail**: Complete signature and approval history

### **4. AI-Driven Dynamic Fee System**

**Near-Zero Fee Optimization:**
- **Micro-transactions**: As low as 0.01% for amounts <1 PAR
- **Small transactions**: 50% reduction for amounts <10 PAR  
- **Network health bonuses**: Fees reduced when network is healthy
- **Anti-spam protection**: Absolute minimum 0.0001 PAR

**How It Works:**
1. AI analyzes network congestion, contributor health, and transaction patterns
2. Dynamically adjusts fees between 0.1% - 5% based on conditions
3. Provides near-zero fees for small transactions to encourage adoption
4. Balances user affordability with contributor sustainability

---

## 📊 Network Statistics

| Metric | Value | Notes |
|--------|-------|-------|
| **Consensus** | Proof-of-Work + AI Validation | Hybrid security model |
| **Block Time** | ~10 seconds | AI-optimized for network conditions |
| **Transaction Finality** | 1-3 confirmations | Fast settlement |
| **Fee Range** | 0.01% - 5% | AI-optimized, often <0.1% |
| **Minimum Fee** | 0.0001 PAR | Anti-spam protection |
| **Total Supply** | 1 Billion PAR | Fixed supply, 8 decimal places |
| **Address Format** | PAR1... | Bech32-style encoding |

---

## 📁 Project Structure

```
Paradigm[CC]/
├── paradigm-core/              # 🏗️ Main network node
│   ├── src/
│   │   ├── ai/                 # 🤖 AI governance systems
│   │   ├── tokenomics/         # 💰 Economic models
│   │   ├── fee_calculation.rs  # 📊 Dynamic fee system
│   │   ├── secure_networking.rs # 🔒 TLS/mTLS encryption
│   │   ├── hsm_manager.rs      # 🔐 Hardware security
│   │   ├── multisig_treasury.rs # 🏦 Multi-sig wallets
│   │   └── ...
│   └── Cargo.toml
├── paradigm-wallet/            # 💼 Multi-signature wallet
├── paradigm-contributor/       # ⚡ ML task processor
├── paradigm-sdk/              # 🔧 Developer library
├── docs/                      # 📖 Documentation
│   ├── DEVELOPER_GUIDE_ADVANCED.md  # 🚀 Complete dev guide
│   ├── API.md                 # 📡 API reference
│   └── ...
├── build.bat                  # 🔨 Windows build script
├── paradigm-network.py        # 🐍 Modern network launcher
└── README.md                  # 📄 This file
```

---

## 📖 Documentation

### **Quick References**
- **[🚀 Quick Start Guide](docs/QUICKSTART.md)** - Get running in 5 minutes
- **[🔧 Advanced Developer Guide](docs/DEVELOPER_GUIDE_ADVANCED.md)** - Complete technical reference
- **[🌐 Network Setup Guide](docs/NETWORK_SETUP_GUIDE.md)** - Production deployment
- **[🛡️ Security Guide](docs/SECURITY.md)** - Enterprise security setup

### **API Documentation**
- **[📡 REST API Reference](docs/API.md)** - HTTP endpoints and WebSocket
- **[🔗 Integration Examples](docs/INTEGRATION.md)** - Payment & DeFi integration
- **[🧪 Testing Guide](docs/TESTING.md)** - Unit, integration, and load tests

### **Operations**
- **[🚀 Production Deployment](docs/PRODUCTION.md)** - Docker, Kubernetes, monitoring
- **[📊 Network Analytics](docs/ANALYTICS.md)** - Monitoring and insights
- **[🔧 Contributing Guide](docs/CONTRIBUTING.md)** - How to contribute

---

## 🤖 AI Governance Features

### **Autonomous Network Management**
- **Parameter Optimization**: AI adjusts fees, block times, and network settings
- **Threat Detection**: Real-time monitoring for attacks and anomalies
- **Performance Scaling**: Auto-scaling based on network load
- **Economic Balancing**: Maintains healthy token economics

### **Neural Consensus System**
- **Distributed Validation**: Multiple AI nodes validate transactions
- **Confidence Scoring**: Each validation includes confidence metrics
- **Adaptive Security**: Security level adjusts based on transaction risk
- **Anti-Gaming**: Prevents manipulation of consensus decisions

### **Predictive Governance**
- **Market Analysis**: Predicts network needs based on usage patterns
- **Proactive Adjustments**: Makes changes before issues arise
- **Community Input**: Balances AI decisions with human governance
- **Transparency**: All AI decisions are logged and auditable

---

## 🔗 Integration Examples

### **Simple Payment Integration**
```python
import requests

# Send PAR with AI-optimized fees
response = requests.post('http://localhost:8080/api/v1/transactions', json={
    'from': 'PAR1sender...',
    'to': 'PAR1recipient...',
    'amount': 1000000000,  # 10 PAR in smallest units
    'message': 'API payment'
})

print(f"Transaction ID: {response.json()['transaction_id']}")
print(f"AI-optimized fee: {response.json()['fee']} PAR")
```

### **Multi-Signature Treasury Integration**
```javascript
// Create corporate treasury with 3-of-5 signatures
const treasury = await fetch('/api/v1/treasury/create', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
        name: 'Corporate Treasury',
        type: 'MainTreasury',
        threshold: 3,
        signers: [
            { name: 'CFO', role: 'TreasuryManager', weight: 2 },
            { name: 'CEO', role: 'TreasuryManager', weight: 2 },
            { name: 'CTO', role: 'TechnicalLead', weight: 1 },
            { name: 'Board-1', role: 'EmergencyRecovery', weight: 1 },
            { name: 'Board-2', role: 'AuditOversight', weight: 1 }
        ]
    })
});
```

---

## 🆘 Support & Community

### **Getting Help**
- **📖 Documentation**: [docs/](docs/) directory
- **🐛 Issues**: [GitHub Issues](https://github.com/paradigm-network/paradigm/issues)
- **💬 Discord**: [Join Developer Community](https://discord.gg/paradigm)
- **📧 Email**: support@paradigm.network

### **Development Community**  
- **🔧 Contributing**: See [CONTRIBUTING.md](docs/CONTRIBUTING.md)
- **🧪 Testing**: Help test new features
- **📝 Documentation**: Improve guides and tutorials
- **🌍 Translation**: Translate to your language

### **Enterprise Support**
- **🏢 Enterprise**: enterprise@paradigm.network
- **🔒 Security**: security@paradigm.network  
- **🚀 Partnerships**: partnerships@paradigm.network

---

## 📈 Roadmap

### **Q1 2024** ✅
- [x] AI-powered dynamic fee system
- [x] Enterprise security (TLS/mTLS, HSM, Multi-sig)
- [x] Neural consensus implementation
- [x] Performance optimizations

### **Q2 2024** 🔄
- [ ] Cross-chain bridges (Bitcoin, Ethereum)
- [ ] Mobile wallet applications
- [ ] Governance UI dashboard
- [ ] Advanced analytics platform

### **Q3 2024** 📋
- [ ] Smart contract platform
- [ ] DeFi protocol integration  
- [ ] Quantum-resistant upgrades
- [ ] Enterprise partnerships

### **Q4 2024** 📋
- [ ] Mainnet launch preparation
- [ ] Regulatory compliance framework
- [ ] Global node deployment
- [ ] Community governance transition

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

**Key Points:**
- ✅ Free to use, modify, and distribute
- ✅ Commercial use permitted
- ✅ Private use permitted  
- ⚠️ No warranty provided
- ⚠️ License must be included in copies

---

## 🏆 Achievements

- 🥇 **First AI-Governed Cryptocurrency** with neural consensus
- 🛡️ **Enterprise-Grade Security** with HSM and multi-signature support
- ⚡ **Near-Zero Transaction Fees** starting at 0.01%
- 🤖 **Autonomous Network Management** with predictive governance
- 🔗 **Cross-Chain Compatibility** with Bitcoin and Ethereum bridges

---

**🚀 Welcome to the future of AI-powered cryptocurrency with Paradigm!**

*Built with ❤️ by the Paradigm Core Team*