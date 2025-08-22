# Paradigm Production Network Guide

## 🚀 **Best Method for Production Network Deployment**

For a production Paradigm cryptocurrency network that requires persistent operation, synchronization, and monitoring, use the **Production Network Launcher** system.

### ✅ **Recommended Production Setup**

#### **1. Quick Start (Recommended)**
```bash
# Windows
launch-network.bat start

# Linux/Mac
./launch-network.sh start
```

#### **2. What This Gives You**
- **Multi-node cluster**: 3 core blockchain nodes (ports 8080-8082)
- **Distributed contributors**: 5 ML contributor clients with different thread counts
- **Auto-restart capability**: Failed components automatically restart
- **Centralized logging**: All logs in `./logs/` directory
- **Process management**: PID tracking in `./pids/` directory
- **Health monitoring**: Real-time network status checking

### 🔧 **Production Network Architecture**

```
┌─────────────────────────────────────────────────────────────┐
│                    Paradigm Production Network              │
├─────────────────────────────────────────────────────────────┤
│  Core Node 1 (8080) ←→ Core Node 2 (8081) ←→ Core Node 3   │
│                           ↕                    (8082)       │
│  Contributor 1 (8 threads) ←→ P2P Network ←→ Contributors   │
│  Contributor 2 (6 threads)    Gossipsub        2,3,4,5      │
│  Contributor 3 (4 threads) ←→ mDNS Discovery ←→ (2-4 threads)│
├─────────────────────────────────────────────────────────────┤
│              Automatic Failover & Health Monitoring         │
└─────────────────────────────────────────────────────────────┘
```

### 📋 **Production Commands**

#### **Network Management**
```bash
./launch-network.sh start     # Start production network
./launch-network.sh stop      # Stop all components  
./launch-network.sh restart   # Restart entire network
./launch-network.sh status    # Show current status
```

#### **Monitoring & Maintenance**
```bash
./launch-network.sh monitor   # Auto-restart failed components
./launch-network.sh logs      # View recent logs from all components
```

#### **What Each Command Does**
- **`start`**: Launches 3 core nodes + 5 contributors with automatic synchronization
- **`monitor`**: Continuously checks health and restarts failed components
- **`status`**: Shows which nodes/contributors are running with PIDs
- **`logs`**: Displays recent log entries from all network components
- **`stop`**: Gracefully shuts down all network components

### 🖥️ **Linux Server Deployment**

#### **Full Production Installation**
```bash
# Install as system service
sudo ./install-production.sh

# Manage via systemd
sudo systemctl start paradigm-network
sudo systemctl enable paradigm-network  # Auto-start on boot
sudo systemctl status paradigm-network

# Monitor logs
sudo journalctl -u paradigm-network -f
```

#### **Production Features**
- ✅ **Systemd integration**: Service auto-starts on boot
- ✅ **Log rotation**: Automatic log management
- ✅ **Security**: Dedicated service user with restricted permissions
- ✅ **Firewall**: Automatic port configuration
- ✅ **Monitoring**: Performance and health check scripts

### 🌐 **Network Access Points**

Once started, your production network provides:

```
Node 1 API:  http://localhost:8080
Node 2 API:  http://localhost:8081  
Node 3 API:  http://localhost:8082

P2P Network: Ports 9000-9002 (auto-configured)
```

### 📊 **Network Synchronization Features**

#### **Automatic Network Sync**
- **P2P Discovery**: Nodes automatically find each other via mDNS
- **Gossipsub Protocol**: Real-time message propagation
- **Task Distribution**: ML tasks automatically distributed to contributors
- **Reward Synchronization**: PAR rewards calculated and distributed automatically

#### **Fault Tolerance**
- **Multi-node redundancy**: Network continues with 1+ nodes running
- **Auto-restart**: Failed nodes/contributors automatically restart (with monitor)
- **Data persistence**: SQLite databases maintain state across restarts
- **Network healing**: P2P connections automatically re-establish

### 🔍 **Monitoring Your Network**

#### **Real-time Status**
```bash
# Check which components are running
./launch-network.sh status

# Monitor network health (auto-restart mode)
./launch-network.sh monitor
```

#### **Log Analysis**
```bash
# View all recent logs
./launch-network.sh logs

# Check specific component logs
tail -f ./logs/core-node-1.log
tail -f ./logs/contributor-1.log
```

#### **Performance Metrics**
The network automatically tracks:
- Task processing rates
- PAR reward distribution
- P2P connection health
- CPU/memory usage
- Network synchronization status

### 🎯 **Use Cases for Production Network**

#### **Development & Testing**
- **Local development**: Full network on single machine
- **Integration testing**: Multi-node behavior testing
- **Performance testing**: Contributor load testing

#### **Production Deployment**
- **Private networks**: Enterprise or research deployments
- **Testnet operations**: Public test network hosting
- **Mining alternative**: ML-based computational contribution network

### ⚡ **Quick Network Health Check**

After starting your network, verify it's working:

```bash
# 1. Check all components are running
./launch-network.sh status

# 2. Verify network APIs are responding
curl http://localhost:8080/health
curl http://localhost:8081/health  
curl http://localhost:8082/health

# 3. Monitor task processing
tail -f ./logs/contributor-*.log
```

### 🔧 **Troubleshooting**

#### **Common Issues**
- **Port conflicts**: Change ports in launch script if 8080-8082 are taken
- **Permission errors**: Ensure script has execute permissions (`chmod +x`)
- **Build failures**: Run `cargo build --release` manually first

#### **Recovery Commands**
```bash
# Clean restart
./launch-network.sh stop
rm -rf ./logs ./pids ./data
./launch-network.sh start

# Emergency stop
pkill -f paradigm-core
pkill -f paradigm-contributor
```

---

## 🏆 **Summary: Best Production Method**

**For persistent, synchronized Paradigm networks:**

1. **Use**: `./launch-network.sh start` (or `.bat` on Windows)
2. **Monitor**: `./launch-network.sh monitor` for auto-restart capability  
3. **Deploy**: `./install-production.sh` for Linux server deployment

This gives you a production-ready, fault-tolerant, multi-node Paradigm cryptocurrency network with automatic synchronization, health monitoring, and enterprise-grade reliability.
