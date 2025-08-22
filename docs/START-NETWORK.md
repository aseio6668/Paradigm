# 🚀 Starting Your Own Paradigm Network Instance

This guide shows you how to start a Paradigm network that others can connect to.

## 🎯 Quick Start - Bootstrap Node

### 1. **Build the System**
```bash
# Build all components
./build-advanced.bat

# Or use the fast build
./build.bat
```

### 2. **Start Bootstrap Node (First Node)**
```bash
cd target\release

# Start the first node (bootstrap node)
paradigm-core.exe --port 8080 --data-dir ./bootstrap-data

# Your node will show something like:
# INFO libp2p_swarm: local_peer_id=12D3KooW...
```

**🔑 Important:** Copy the `local_peer_id` from the logs - others will need this to connect!

### 3. **Get Your Network Address**
Your bootstrap node address will be:
```
/ip4/YOUR_PUBLIC_IP/tcp/8080/p2p/12D3KooW...YourPeerID
```

**Find your IP address:**
```bash
# Windows
ipconfig

# Or check online
curl ifconfig.me
```

## 🌐 Network Configuration Options

### **Option A: Local Network (Same WiFi/LAN)**
```bash
# Start bootstrap node
paradigm-core.exe --port 8080

# Your address for others:
/ip4/192.168.1.100/tcp/8080/p2p/12D3KooW...YourPeerID
```

### **Option B: Public Internet** 
```bash
# 1. Forward port 8080 in your router
# 2. Start bootstrap node
paradigm-core.exe --port 8080

# Your address for others:
/ip4/YOUR_PUBLIC_IP/tcp/8080/p2p/12D3KooW...YourPeerID
```

### **Option C: Custom Port**
```bash
# Use different port (if 8080 is busy)
paradigm-core.exe --port 9000

# Your address:
/ip4/YOUR_IP/tcp/9000/p2p/12D3KooW...YourPeerID
```

## 👥 How Others Connect to Your Network

### **For Users Joining Your Network:**

```bash
# They run:
paradigm-core.exe --bootstrap-peers /ip4/YOUR_IP/tcp/8080/p2p/YOUR_PEER_ID

# Or multiple bootstrap nodes:
paradigm-core.exe --bootstrap-peers "addr1,addr2,addr3"
```

### **Share This Info With Others:**
```
🌟 Join My Paradigm Network!

Bootstrap Address:
/ip4/YOUR_IP/tcp/8080/p2p/YOUR_PEER_ID

To connect:
1. Download Paradigm from: [your link]
2. Run: paradigm-core.exe --bootstrap-peers /ip4/YOUR_IP/tcp/8080/p2p/YOUR_PEER_ID
3. Start earning PAR tokens!
```

## 🔧 Advanced Network Setup

### **Create network-start.bat** (Recommended)
```batch
@echo off
title Paradigm Network Bootstrap Node
echo 🌟 Starting Paradigm Network Bootstrap Node
echo ==========================================
echo.
echo This will start a Paradigm network that others can join.
echo Your network address will be displayed below.
echo.
echo ⚠️  Make sure port 8080 is open/forwarded if you want 
echo     internet users to connect.
echo.

paradigm-core.exe --port 8080 --data-dir ./network-data

pause
```

### **Port Forwarding (For Internet Access)**
1. **Router Settings:**
   - Forward **TCP port 8080** to your computer's local IP
   - Or use UPnP if available

2. **Windows Firewall:**
   ```bash
   # Allow paradigm-core through firewall
   netsh advfirewall firewall add rule name="Paradigm Node" dir=in action=allow protocol=TCP localport=8080
   ```

### **Multiple Node Setup (Advanced)**
```bash
# Node 1 (Bootstrap)
paradigm-core.exe --port 8080 --data-dir ./node1-data

# Node 2 (Connected to Node 1)  
paradigm-core.exe --port 8081 --data-dir ./node2-data --bootstrap-peers /ip4/127.0.0.1/tcp/8080/p2p/PEER_ID_OF_NODE1

# Node 3 (Connected to network)
paradigm-core.exe --port 8082 --data-dir ./node3-data --bootstrap-peers /ip4/127.0.0.1/tcp/8080/p2p/PEER_ID_OF_NODE1
```

## 🏆 Running Full Network Services

### **Complete Network with All Services:**
```bash
# Terminal 1: Core Node
paradigm-core.exe --port 8080

# Terminal 2: Start wallet service
paradigm-wallet.exe

# Terminal 3: Start contributor service
paradigm-contributor.exe
```

## 📊 Monitoring Your Network

### **Check Network Status:**
```bash
# Your node logs will show:
# ✅ Connected peers: X
# ✅ Active ML tasks: Y  
# ✅ PAR tokens distributed: Z
```

### **Network Analytics:**
- Your node automatically starts analytics on port 8081
- Visit: `http://localhost:8081/analytics` (when implemented)

## 🚨 Troubleshooting

### **No Peers Connecting?**
1. **Check firewall** - Port 8080 blocked?
2. **Check router** - Port forwarding correct?
3. **Check IP address** - Using correct public IP?
4. **Check peer ID** - Sharing the right peer ID?

### **Connection Issues:**
```bash
# Test local connection first
paradigm-core.exe --port 8081 --bootstrap-peers /ip4/127.0.0.1/tcp/8080/p2p/YOUR_PEER_ID
```

### **Multiple Networks:**
```bash
# Start separate network on different port
paradigm-core.exe --port 9000 --data-dir ./network2-data
```

## 🎉 Success Indicators

Your network is running successfully when you see:
```
✅ Paradigm node started successfully
✅ P2P network listening on port 8080
✅ Connected to 0 peers (initially)
✅ AI governance system active
✅ Ready to process ML tasks
✅ PAR token rewards enabled
```

## 📝 Network Information Template

**Share this template with potential users:**

```
🌟 JOIN OUR PARADIGM NETWORK! 🌟

Network Name: [Your Network Name]
Bootstrap Node: /ip4/[YOUR_IP]/tcp/8080/p2p/[YOUR_PEER_ID]

Features:
✅ AI-powered governance
✅ Earn PAR tokens for ML tasks  
✅ Quantum-resistant security
✅ Real-time analytics

To join:
1. Download: [your_paradigm_download_link]
2. Run: paradigm-core.exe --bootstrap-peers [YOUR_BOOTSTRAP_ADDRESS]
3. Start earning!

Questions? Contact: [your_contact]
```

---

**🚀 Your Paradigm network is now ready for others to join and start earning PAR tokens through AI/ML contributions!**