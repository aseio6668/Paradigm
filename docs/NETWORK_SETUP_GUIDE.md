# Paradigm Network Setup Guide

This comprehensive guide will help you deploy a complete Paradigm blockchain network instance from scratch.

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Production Deployment](#production-deployment)
4. [Network Configuration](#network-configuration)
5. [Monitoring Setup](#monitoring-setup)
6. [Backup & Recovery](#backup--recovery)
7. [Troubleshooting](#troubleshooting)

## 🔧 Prerequisites

### System Requirements

**Minimum Requirements (Development):**
- 4 CPU cores
- 8 GB RAM
- 100 GB SSD storage
- Ubuntu 20.04+ / RHEL 8+ / Windows Server 2019+

**Recommended Requirements (Production):**
- 16 CPU cores
- 32 GB RAM
- 1 TB NVMe SSD storage
- Load balancer
- Dedicated monitoring instance

### Software Dependencies

```bash
# Install Docker & Docker Compose
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Install Kubernetes (Production)
curl -s https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key add -
echo "deb https://apt.kubernetes.io/ kubernetes-xenial main" | sudo tee /etc/apt/sources.list.d/kubernetes.list
sudo apt update && sudo apt install -y kubectl kubelet kubeadm

# Install Rust (for building from source)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## 🚀 Quick Start

### 1. Clone Repository

```bash
git clone https://github.com/paradigm-network/paradigm.git
cd paradigm
```

### 2. Environment Setup

```bash
# Copy environment template
cp .env.example .env

# Edit configuration
nano .env
```

### 3. Build & Start Network

```bash
# Build all components
./scripts/build.sh

# Start development network
docker-compose up -d

# Check status
docker-compose ps
./scripts/health-check.sh
```

### 4. Initialize Genesis

```bash
# Create genesis block and initial accounts
./scripts/init-genesis.sh

# Fund initial accounts
./scripts/fund-accounts.sh
```

### 5. Verify Installation

```bash
# Check API health
curl http://localhost:8080/health

# Check node status
curl http://localhost:8080/api/v1/blockchain/latest-block

# View logs
docker-compose logs -f paradigm-core
```

## 🏭 Production Deployment

### Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Load Balancer │    │   API Gateway   │    │   Monitoring    │
│   (HAProxy)     │    │   (Kong/Nginx)  │    │   (Prometheus)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Paradigm Nodes │    │   API Servers   │    │    Grafana      │
│   (Core Chain)  │    │   (REST/WS)     │    │  (Dashboard)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   PostgreSQL    │    │     Redis       │    │   ELK Stack     │
│   (Metadata)    │    │   (Caching)     │    │   (Logging)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 1. Production Configuration

Create production environment file:

```bash
# production.env
ENVIRONMENT=production
NETWORK_ID=paradigm-mainnet-1
CHAIN_ID=paradigm-1

# Core Node Configuration
PARADIGM_RPC_PORT=8545
PARADIGM_P2P_PORT=30303
PARADIGM_WS_PORT=8546
PARADIGM_METRICS_PORT=9090

# API Configuration
API_PORT=8080
API_HOST=0.0.0.0
JWT_SECRET=your-super-secure-jwt-secret-change-this
RATE_LIMIT_REQUESTS_PER_MINUTE=1000
RATE_LIMIT_BURST=100

# Database Configuration
DATABASE_URL=postgresql://paradigm:secure_password@postgres:5432/paradigm_mainnet
REDIS_URL=redis://redis:6379/0

# Monitoring
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000
JAEGER_PORT=14268

# Cross-Chain Configuration
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/YOUR_INFURA_KEY
BITCOIN_RPC_URL=https://btc-mainnet.gateway.pokt.network/v1/YOUR_POKT_KEY
COSMOS_RPC_URL=https://cosmos-rpc.polkachu.com

# Security
ENABLE_TLS=true
TLS_CERT_PATH=/etc/ssl/certs/paradigm.crt
TLS_KEY_PATH=/etc/ssl/private/paradigm.key

# Performance
MAX_PEERS=100
CACHE_SIZE=1000MB
DB_CONNECTION_POOL_SIZE=20
```

### 2. Docker Production Setup

Create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  paradigm-core:
    image: paradigm/core:latest
    container_name: paradigm-core-mainnet
    restart: unless-stopped
    env_file: production.env
    ports:
      - "8545:8545"
      - "30303:30303"
      - "8546:8546"
    volumes:
      - paradigm-data:/data
      - ./config:/config:ro
      - ./ssl:/etc/ssl:ro
    networks:
      - paradigm-network
    depends_on:
      - postgres
      - redis
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8545/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      resources:
        limits:
          memory: 16G
          cpus: '8'
        reservations:
          memory: 8G
          cpus: '4'

  paradigm-api:
    image: paradigm/api:latest
    container_name: paradigm-api-mainnet
    restart: unless-stopped
    env_file: production.env
    ports:
      - "8080:8080"
    volumes:
      - ./config:/config:ro
      - ./ssl:/etc/ssl:ro
    networks:
      - paradigm-network
    depends_on:
      - paradigm-core
      - postgres
      - redis
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      replicas: 3
      resources:
        limits:
          memory: 4G
          cpus: '4'

  postgres:
    image: postgres:15-alpine
    container_name: paradigm-postgres
    restart: unless-stopped
    environment:
      POSTGRES_DB: paradigm_mainnet
      POSTGRES_USER: paradigm
      POSTGRES_PASSWORD: secure_password
      POSTGRES_INITDB_ARGS: "--encoding=UTF-8 --lc-collate=C --lc-ctype=C"
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init-db.sql:ro
    networks:
      - paradigm-network
    ports:
      - "5432:5432"
    command: >
      postgres
      -c max_connections=200
      -c shared_buffers=256MB
      -c effective_cache_size=1GB
      -c maintenance_work_mem=64MB
      -c checkpoint_completion_target=0.9
      -c wal_buffers=16MB
      -c default_statistics_target=100

  redis:
    image: redis:7-alpine
    container_name: paradigm-redis
    restart: unless-stopped
    volumes:
      - redis-data:/data
    networks:
      - paradigm-network
    ports:
      - "6379:6379"
    command: >
      redis-server
      --appendonly yes
      --maxmemory 1gb
      --maxmemory-policy allkeys-lru

  prometheus:
    image: prom/prometheus:latest
    container_name: paradigm-prometheus
    restart: unless-stopped
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - ./monitoring/alerts:/etc/prometheus/alerts:ro
      - prometheus-data:/prometheus
    networks:
      - paradigm-network
    ports:
      - "9090:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=15d'
      - '--web.enable-lifecycle'

  grafana:
    image: grafana/grafana:latest
    container_name: paradigm-grafana
    restart: unless-stopped
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin_password_change_this
      GF_INSTALL_PLUGINS: grafana-clock-panel,grafana-simple-json-datasource
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    networks:
      - paradigm-network
    ports:
      - "3000:3000"

  nginx:
    image: nginx:alpine
    container_name: paradigm-nginx
    restart: unless-stopped
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/ssl:ro
    networks:
      - paradigm-network
    ports:
      - "80:80"
      - "443:443"
    depends_on:
      - paradigm-api

networks:
  paradigm-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16

volumes:
  paradigm-data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /var/lib/paradigm/data
  postgres-data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /var/lib/paradigm/postgres
  redis-data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /var/lib/paradigm/redis
  prometheus-data:
    driver: local
  grafana-data:
    driver: local
```

### 3. Kubernetes Production Setup

Create `k8s/production/` directory with manifests:

```bash
mkdir -p k8s/production
```

## 🔧 Network Configuration

### Genesis Configuration

Create `config/genesis.json`:

```json
{
  "chain_id": "paradigm-1",
  "network_id": "paradigm-mainnet-1",
  "genesis_time": "2024-01-01T00:00:00Z",
  "initial_validators": [
    {
      "address": "PAR1234567890abcdef1234567890abcdef12345678",
      "public_key": "0x...",
      "stake": "10000000000000000",
      "commission": "0.05"
    }
  ],
  "initial_accounts": [
    {
      "address": "PAR1111111111111111111111111111111111111111",
      "balance": "1000000000000000000"
    }
  ],
  "consensus_params": {
    "block_time": 12,
    "block_size_limit": 1048576,
    "gas_limit": 30000000,
    "min_validator_stake": "1000000000000000"
  },
  "governance_params": {
    "voting_period": 604800,
    "proposal_deposit": "100000000000000",
    "quorum": "0.334",
    "threshold": "0.5"
  },
  "tokenomics": {
    "total_supply": "8000000000000000000",
    "inflation_rate": "0.07",
    "staking_rewards": "0.05",
    "ml_rewards": "0.02"
  }
}
```

### Node Configuration

Create `config/node.toml`:

```toml
[node]
node_id = "paradigm-node-1"
data_dir = "/data"
log_level = "info"
enable_metrics = true
metrics_port = 9090

[network]
listen_port = 30303
max_peers = 100
discovery_enabled = true
nat_enabled = true
bootstrap_nodes = [
    "/ip4/seed1.paradigm.network/tcp/30303/p2p/QmBootstrapNode1",
    "/ip4/seed2.paradigm.network/tcp/30303/p2p/QmBootstrapNode2"
]

[rpc]
enabled = true
host = "0.0.0.0"
port = 8545
cors_origins = ["*"]
rate_limit = 1000

[websocket]
enabled = true
host = "0.0.0.0"
port = 8546
max_connections = 1000

[consensus]
enable_ml_consensus = true
ml_task_timeout = 300
validator_timeout = 30
block_time = 12

[storage]
database_url = "postgresql://paradigm:password@postgres:5432/paradigm"
cache_size = "1GB"
enable_pruning = true
pruning_interval = 86400

[cross_chain]
ethereum_enabled = true
ethereum_rpc = "https://mainnet.infura.io/v3/YOUR_KEY"
bitcoin_enabled = true
bitcoin_rpc = "https://btc-mainnet.gateway.pokt.network/v1/YOUR_KEY"
cosmos_enabled = true
cosmos_rpc = "https://cosmos-rpc.polkachu.com"

[security]
enable_tls = true
cert_file = "/etc/ssl/certs/paradigm.crt"
key_file = "/etc/ssl/private/paradigm.key"
enable_firewall = true
allowed_ips = ["0.0.0.0/0"]
```

## 📊 Monitoring Setup

### Prometheus Configuration

Create `monitoring/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alerts/*.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  - job_name: 'paradigm-core'
    static_configs:
      - targets: ['paradigm-core:9090']
    metrics_path: /metrics
    scrape_interval: 15s

  - job_name: 'paradigm-api'
    static_configs:
      - targets: ['paradigm-api:9090']
    metrics_path: /metrics
    scrape_interval: 15s

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']
```

### Grafana Dashboards

Create monitoring dashboards in `monitoring/grafana/dashboards/`.

## 🔄 Deployment Scripts

Create comprehensive deployment scripts:

```bash
# scripts/deploy-production.sh
#!/bin/bash
set -e

echo "🚀 Starting Paradigm Network Production Deployment..."

# Create directories
sudo mkdir -p /var/lib/paradigm/{data,postgres,redis}
sudo chown -R $USER:$USER /var/lib/paradigm

# Generate SSL certificates
./scripts/generate-ssl.sh

# Initialize database
./scripts/init-database.sh

# Deploy with Docker Compose
docker-compose -f docker-compose.prod.yml up -d

# Wait for services
echo "⏳ Waiting for services to start..."
sleep 30

# Initialize genesis
./scripts/init-genesis.sh

# Health checks
./scripts/health-check.sh

echo "✅ Paradigm Network deployed successfully!"
echo "🌐 API: https://localhost:8080"
echo "📊 Grafana: http://localhost:3000"
echo "📈 Prometheus: http://localhost:9090"
```

This deployment guide provides everything needed to start a new Paradigm network instance from scratch. Would you like me to continue with the Kubernetes manifests and additional production configurations?