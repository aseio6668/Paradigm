# 🚀 Paradigm Genesis Chain Workflows

This document explains the three different ways to work with genesis chains in the Paradigm project, now integrated directly into the build target directories.

## 📂 Directory Structure

```
paradigm/
├── genesis.bat                     👈 Universal launcher (NEW!)
├── build-genesis-only.bat          👈 Build script with genesis integration
└── target/
    ├── debug/
    │   ├── genesis-init.bat         👈 Development genesis script
    │   ├── paradigm-core.exe        👈 Debug executables
    │   ├── paradigm-contributor.exe
    │   └── dev-genesis/             👈 Created when run
    │       ├── network-config.toml
    │       ├── genesis-config.toml
    │       └── genesis-data/
    ├── release/
    │   ├── genesis-init.bat         👈 Production genesis script  
    │   ├── paradigm-core.exe        👈 Release executables
    │   ├── paradigm-contributor.exe
    │   └── prod-genesis/            👈 Created when run
    │       ├── network-config.toml
    │       ├── genesis-config.toml
    │       └── genesis-data/
    └── paradigm-release/            👈 Legacy packaged release
        ├── genesis-chain.bat
        └── [all components]
```

## 🎯 Three Genesis Workflows

### **1. Development Workflow** ⚡ (Recommended for development)

**Use when:** Rapid development, testing new features, debugging

```bash
# Method A: Quick universal launcher
./genesis.bat
# Choose option 1 for development

# Method B: Direct development launch
cd target/debug
./genesis-init.bat

# Method C: Build + launch in one step
cargo build --bin paradigm-core --bin paradigm-contributor
cd target/debug && ./genesis-init.bat
```

**Features:**
- ✅ Fast debug builds
- ✅ Verbose logging for debugging
- ✅ Quick iteration cycle
- ✅ Creates `dev-genesis/` directory
- ✅ Includes development peer templates
- ✅ Restart script: `restart-dev-genesis.bat`

### **2. Production Workflow** 🚀 (Recommended for mainnet)

**Use when:** Launching production networks, performance matters

```bash
# Method A: Quick universal launcher
./genesis.bat  
# Choose option 2 for production

# Method B: Direct production launch
cd target/release
./genesis-init.bat

# Method C: Build + launch for production
cargo build --release --bin paradigm-core --bin paradigm-contributor
cd target/release && ./genesis-init.bat
```

**Features:**
- ✅ Optimized release builds
- ✅ Enhanced security settings
- ✅ Higher peer limits (100 vs 50)
- ✅ Production-ready configuration
- ✅ Creates `prod-genesis/` directory
- ✅ Includes backup script: `backup-production-chain.bat`
- ✅ Connection helper: `connect-to-production.bat`

### **3. Legacy Workflow** 📦 (Pre-packaged releases)

**Use when:** Distributing to end users, sharing complete packages

```bash
# Method A: Quick universal launcher
./genesis.bat
# Choose option 3 for legacy

# Method B: Build complete package
./build-genesis-only.bat
cd target/paradigm-release
./genesis-chain.bat

# Method C: Root launcher
./launch-genesis.bat
```

**Features:**
- ✅ Self-contained package
- ✅ Complete documentation
- ✅ Multiple connection examples
- ✅ Ready for distribution

## 💡 CLI Features - Addnode Support

All workflows now support the new `--addnode` and `--addnodefile` functionality:

### **Command Line Peers:**
```bash
# Single peer
paradigm-core.exe --addnode "192.168.1.100:8080"

# Multiple peers (semicolon separated)
paradigm-core.exe --addnode "peer1.com:8080;peer2.com:8081;peer3.com"

# Mixed with/without ports (uses default 8080)
paradigm-core.exe --addnode "peer1.com:8080;peer2.com;peer3.com:8081"
```

### **Peer Files:**
```bash
# Load from file
paradigm-core.exe --addnodefile peers.txt

# Combined approach
paradigm-core.exe --addnode "urgent.peer.com:8080" --addnodefile peers.txt
```

### **Peer File Format:**
```
# Production Network Peers
192.168.1.100:8080
203.0.113.45:8080  
198.51.100.123        # Uses default port 8080
my-node.example.com:8081
# Comments start with #
```

## 🔧 Management Commands

### **Development Management:**
```bash
cd target/debug/dev-genesis
./restart-dev-genesis.bat        # Quick restart
./test-dev-addnode.bat          # Test peer connections
```

### **Production Management:**
```bash
cd target/release/prod-genesis
./restart-production-genesis.bat  # Quick restart
./backup-production-chain.bat    # Create backup
./connect-to-production.bat      # Help others connect
```

## 🌐 Network Connection Examples

### **Joining Existing Networks:**

**Development Network:**
```bash
# Connect to development genesis node
cd target/debug
./paradigm-core.exe --addnode "dev-genesis-ip:8080" --data-dir ./my-dev-node
```

**Production Network:**  
```bash
# Connect to production genesis node
cd target/release  
./paradigm-core.exe --addnode "production-genesis-ip:8080" --data-dir ./my-prod-node
```

**Using Peer Files:**
```bash
# Create peers.txt with known network nodes
cd target/release
echo "mainnet-node1.paradigm.network:8080" > my-peers.txt  
echo "mainnet-node2.paradigm.network:8080" >> my-peers.txt
./paradigm-core.exe --addnodefile my-peers.txt --data-dir ./my-mainnet-node
```

## ⚡ Quick Reference

| Task | Command |
|------|---------|
| **Start development chain** | `cd target/debug && genesis-init.bat` |
| **Start production chain** | `cd target/release && genesis-init.bat` |  
| **Universal launcher** | `./genesis.bat` |
| **Join dev network** | `target/debug/paradigm-core.exe --addnode "IP:8080"` |
| **Join prod network** | `target/release/paradigm-core.exe --addnode "IP:8080"` |
| **Load peer file** | `paradigm-core.exe --addnodefile peers.txt` |
| **Build for development** | `cargo build --bin paradigm-core` |
| **Build for production** | `cargo build --release --bin paradigm-core` |

## 🎯 Recommended Workflow

**For Development:**
1. `cargo build --bin paradigm-core --bin paradigm-contributor`  
2. `cd target/debug && genesis-init.bat`
3. Iterate, rebuild, test

**For Production Launch:**
1. `cargo build --release --bin paradigm-core --bin paradigm-contributor`
2. `cd target/release && genesis-init.bat`  
3. Share connection info with network participants
4. Use backup scripts regularly

**For Distribution:**
1. `./build-genesis-only.bat`
2. Share entire `target/paradigm-release/` directory
3. Recipients run `genesis-chain.bat` or use connection scripts

---

🎉 **Your genesis chain workflow is now fully integrated into the build system!**