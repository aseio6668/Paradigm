# Paradigm Network Production Launch Guide

## 🚀 Quick Production Launch

For users wanting to quickly start a production network that others can connect to:

### 1. Launch Production Network (Host)
```bash
# Automated production setup
.\launch-production.bat

# Manual setup (if preferred)
.\genesis-init-simple.bat
# Then configure for external access
```

### 2. Others Connect to Your Network
```bash
# Connect to your network (replace with your IP)
.\contributor-connect.bat YOUR_IP_ADDRESS:8080 --enable-autopool

# Examples:
.\contributor-connect.bat 192.168.1.100:8080 --enable-autopool
.\contributor-connect.bat paradigm.yourserver.com:8080 --enable-autopool
```

### 3. Monitor Network Status
```bash
# Monitor your network
.\network-status.bat YOUR_IP_ADDRESS:8080
```

---

## 📋 Detailed Production Setup

### Prerequisites

1. **Build Release Version**:
   ```bash
   cargo build --release
   ```

2. **Network Firewall**:
   - Open port 8080 (TCP) for Paradigm network
   - Open port 8081 (TCP) for analytics dashboard (optional)

3. **System Requirements**:
   - Minimum 2GB RAM
   - 10GB+ free disk space
   - Stable internet connection

### Production Network Configurations

#### Genesis Node (Network Founder)
```toml
# production-config.toml
[network]
bind_address = "0.0.0.0:8080"          # Accept external connections
max_peers = 100                        # Support larger network
bootstrap_peers = []                   # Empty for genesis node

[node]
enable_debug_mode = false              # Production logging
enable_ddos_protection = true          # Security hardening
max_connections_per_ip = 5             # Rate limiting
```

#### Joining Nodes (Contributors)
```toml
# network-config.toml
[network]
bind_address = "0.0.0.0:8080"
max_peers = 50
bootstrap_peers = [
    "/ip4/GENESIS_NODE_IP/tcp/8080/p2p/PEER_ID"
]
```

### Network Privacy Features

The Paradigm network includes built-in privacy protections:

- **Connection Privacy**: Peer addresses hidden by default
- **Non-tracking Architecture**: Past transactions become invisible
- **Ephemeral History**: Local transaction records only
- **IP Protection**: Optional peer address obfuscation

### Security Features

#### DDoS Protection
- Rate limiting: 100 requests per minute per IP
- Connection limits: 5 concurrent connections per IP
- Timeout protection on all network operations

#### Network Hardening
- Encrypted peer-to-peer communication
- Address verification and validation
- Automatic malicious peer detection and banning

### Recommended Launch Process

#### Phase 1: Genesis Launch (Day 1)
1. Host starts genesis network with `launch-production.bat`
2. Test with local contributors to verify functionality
3. Announce network details (IP address, port)

#### Phase 2: Early Adopters (Day 2-7)
1. First external contributors connect
2. Monitor network stability with `network-status.bat`
3. Validate transaction processing and payouts

#### Phase 3: Public Network (Week 2+)
1. Open network to broader community
2. Enable analytics dashboard for transparency
3. Monitor network growth and performance

### Network Information to Share

When inviting others to your network, provide:

```
🌐 Paradigm Network Connection Details
======================================
Network Address: YOUR_IP:8080
Network Type: Public/Community
Launch Date: [DATE]

Connection Command:
.\contributor-connect.bat YOUR_IP:8080 --enable-autopool

Features Available:
✅ PAR Token Rewards
✅ Autopool Work Aggregation  
✅ Transaction Messages
✅ Wallet Integration
✅ Privacy Protection

Network Status: .\network-status.bat YOUR_IP:8080
```

### Monitoring and Maintenance

#### Daily Monitoring
- Check network status with `network-status.bat`
- Verify contributor connections are stable
- Monitor disk space and system resources

#### Weekly Maintenance
- Review network logs for issues
- Check for software updates
- Backup blockchain data from `production-data/`

#### Performance Metrics
- Block height progression
- Peer connection count
- Task processing throughput
- Transaction volume

### Troubleshooting

#### Common Issues

**Connection Refused**:
- Verify firewall settings allow port 8080
- Check bind_address is set to "0.0.0.0:8080"
- Ensure network service is running

**Slow Performance**:
- Increase `max_peers` in configuration
- Check internet bandwidth
- Monitor system resource usage

**Sync Issues**:
- Verify time synchronization between nodes
- Check network connectivity
- Restart network service if needed

#### Emergency Procedures

**Network Split**:
1. Stop all nodes
2. Identify correct chain with highest block height
3. Restart nodes with correct blockchain data

**Data Corruption**:
1. Stop affected nodes
2. Restore from backup in `production-data/`
3. Resync with network

### Community and Support

- Share network details with the community
- Provide clear connection instructions
- Monitor and respond to user issues
- Document any network-specific configurations

---

## 🎯 Production Checklist

Before launching your production network:

- [ ] Built release version with `cargo build --release`
- [ ] Configured firewall to allow port 8080
- [ ] Updated bind_address to "0.0.0.0:8080"
- [ ] Set max_peers to appropriate value (100+ for public networks)
- [ ] Disabled debug mode for production
- [ ] Tested with local contributors first
- [ ] Prepared network details for sharing
- [ ] Set up monitoring with `network-status.bat`
- [ ] Have backup/restore procedure ready
- [ ] Know your external IP address

Your Paradigm network is ready for production use! 🚀