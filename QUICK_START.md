# Paradigm Network Quick Start Guide

Get a Paradigm blockchain network up and running in minutes with this streamlined guide.

## 🚀 One-Line Installation

```bash
curl -fsSL https://raw.githubusercontent.com/paradigm-network/paradigm/main/install.sh | bash
```

## 📋 Prerequisites Check

```bash
# Check system requirements
./scripts/check-requirements.sh

# Install dependencies (Ubuntu/Debian)
sudo apt update
sudo apt install -y docker.io docker-compose curl git openssl jq

# Start Docker
sudo systemctl start docker
sudo usermod -aG docker $USER
newgrp docker
```

## 🏃‍♂️ Quick Setup

### 1. Clone and Configure

```bash
# Clone repository
git clone https://github.com/paradigm-network/paradigm.git
cd paradigm

# Copy environment template
cp .env.example .env

# Generate secrets (automatic)
./scripts/generate-secrets.sh
```

### 2. Start Network (Development)

```bash
# Build and start all services
docker-compose up -d

# Initialize genesis block
./scripts/init-genesis.sh

# Check status
./scripts/health-check.sh
```

### 3. Verify Installation

```bash
# Check API health
curl http://localhost:8080/health

# Get latest block
curl http://localhost:8080/api/v1/blockchain/latest-block

# View running services
docker-compose ps
```

## 🎯 Quick Access Points

| Service | URL | Description |
|---------|-----|-------------|
| **API** | http://localhost:8080 | REST API endpoint |
| **WebSocket** | ws://localhost:8546 | Real-time updates |
| **RPC** | http://localhost:8545 | Blockchain RPC |
| **Grafana** | http://localhost:3000 | Monitoring dashboard |
| **Prometheus** | http://localhost:9090 | Metrics collection |
| **API Docs** | http://localhost:8080/docs | Interactive documentation |

## 🔑 Default Credentials

```bash
# Grafana Dashboard
Username: admin
Password: admin (change on first login)

# Database Access
Username: paradigm
Password: [auto-generated, see secrets/postgres_password.txt]

# API Keys
JWT Secret: [auto-generated, see secrets/jwt_secret.txt]
```

## 🧪 Test the Network

### Create Your First Transaction

```bash
# Using curl
curl -X POST http://localhost:8080/api/v1/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "to": "PAR1111111111111111111111111111111111111111",
    "amount": 1000000000,
    "gas_limit": 21000,
    "gas_price": 20
  }'

# Using the JavaScript SDK
npm install @paradigm/sdk
```

```javascript
import { ParadigmClient } from '@paradigm/sdk';

const client = new ParadigmClient({
  baseURL: 'http://localhost:8080'
});

// Get network stats
const stats = await client.getNetworkStats();
console.log('Network Stats:', stats);

// Create transaction
const tx = await client.createTransaction({
  to: 'PAR1111111111111111111111111111111111111111',
  amount: '1000000000', // 10 PAR (8 decimals)
  gasLimit: 21000
});
console.log('Transaction:', tx);
```

### WebSocket Real-time Updates

```javascript
import { ParadigmWebSocket } from '@paradigm/sdk';

const ws = new ParadigmWebSocket('ws://localhost:8546');

// Subscribe to new blocks
ws.subscribe({
  subscription_type: 'blocks'
});

ws.on('block_update', (block) => {
  console.log('New block:', block);
});

// Subscribe to transactions
ws.subscribe({
  subscription_type: 'transactions',
  filters: {
    address: 'PAR1111111111111111111111111111111111111111'
  }
});
```

## 🏭 Production Deployment

For production deployment, use the comprehensive setup:

```bash
# Production deployment
./scripts/deploy-production.sh

# Kubernetes deployment
kubectl apply -f k8s/production/
```

## 📊 Monitoring Dashboard

Access Grafana at http://localhost:3000 to monitor:

- **Network Performance**: Block times, transaction throughput
- **Node Health**: CPU, memory, disk usage
- **API Metrics**: Request rates, response times
- **Cross-Chain Activity**: Bridge transactions, asset transfers
- **ML Tasks**: Task completion rates, compute utilization

## 🔧 Common Commands

```bash
# View logs
docker-compose logs -f paradigm-core
docker-compose logs -f paradigm-api

# Restart services
docker-compose restart paradigm-core
docker-compose restart paradigm-api

# Update to latest version
git pull origin main
docker-compose pull
docker-compose up -d

# Backup data
./scripts/backup.sh

# Clean shutdown
docker-compose down
```

## 🐛 Troubleshooting

### Service Won't Start

```bash
# Check logs
docker-compose logs paradigm-core

# Check disk space
df -h

# Check memory
free -h

# Restart Docker
sudo systemctl restart docker
```

### API Connection Issues

```bash
# Check if API is running
curl -I http://localhost:8080/health

# Check firewall
sudo ufw status

# Check port binding
netstat -tulpn | grep 8080
```

### Database Issues

```bash
# Check database status
docker-compose exec postgres pg_isready -U paradigm

# Reset database
docker-compose down
docker volume rm paradigm_postgres-data
docker-compose up -d postgres
./scripts/init-genesis.sh
```

## 📚 Next Steps

1. **🔐 Secure Your Network**: [Security Guide](./docs/SECURITY.md)
2. **⚖️ Become a Validator**: [Validator Guide](./docs/VALIDATOR.md)  
3. **🤖 Deploy ML Tasks**: [ML Guide](./docs/ML_TASKS.md)
4. **🌉 Cross-Chain Setup**: [Cross-Chain Guide](./docs/CROSS_CHAIN.md)
5. **📱 Build Apps**: [Developer Guide](./docs/DEVELOPER_GUIDE.md)

## 🆘 Support

- **Documentation**: https://docs.paradigm.network
- **Discord**: https://discord.gg/paradigm
- **GitHub Issues**: https://github.com/paradigm-network/paradigm/issues
- **Email**: support@paradigm.network

---

**🎉 Congratulations!** You now have a fully functional Paradigm blockchain network running. Start building the future of decentralized AI and cross-chain interoperability!