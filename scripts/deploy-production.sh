#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PARADIGM_ROOT="/var/lib/paradigm"
CONFIG_DIR="$PARADIGM_ROOT/config"
SSL_DIR="$PARADIGM_ROOT/ssl"
LOGS_DIR="$PARADIGM_ROOT/logs"
BACKUP_DIR="$PARADIGM_ROOT/backups"

print_banner() {
    echo -e "${BLUE}"
    echo "=================================================================="
    echo "    ____                      __  _                             "
    echo "   / __ \\____ __________ _____/ / (_)___ _____                   "
    echo "  / /_/ / __ \`/ ___/ __ \`/ __  / / / __ \`/ __ \\                  "
    echo " / ____/ /_/ / /  / /_/ / /_/ / / / /_/ / / / /                  "
    echo "/_/    \\__,_/_/   \\__,_/\\__,_/ /_/\\__, /_/ /_/                   "
    echo "                                /____/                          "
    echo "                                                                "
    echo "         Production Deployment Script v1.0.0                   "
    echo "=================================================================="
    echo -e "${NC}"
}

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

check_requirements() {
    log "Checking system requirements..."
    
    # Check if running as root or with sudo
    if [[ $EUID -eq 0 ]]; then
        error "This script should not be run as root. Please run as a regular user with sudo privileges."
    fi
    
    # Check for required commands
    local required_commands=("docker" "docker-compose" "curl" "openssl" "jq")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            error "Required command '$cmd' is not installed."
        fi
    done
    
    # Check Docker daemon
    if ! docker info &> /dev/null; then
        error "Docker daemon is not running or accessible."
    fi
    
    # Check available disk space (minimum 100GB)
    local available_space=$(df / | awk 'NR==2 {print $4}')
    local required_space=$((100 * 1024 * 1024)) # 100GB in KB
    if [[ $available_space -lt $required_space ]]; then
        error "Insufficient disk space. Minimum 100GB required."
    fi
    
    # Check available memory (minimum 16GB)
    local available_memory=$(free -m | awk 'NR==2 {print $2}')
    if [[ $available_memory -lt 16384 ]]; then
        warn "Less than 16GB RAM available. Performance may be impacted."
    fi
    
    log "✅ System requirements check passed"
}

create_directories() {
    log "Creating directory structure..."
    
    # Create main directories
    sudo mkdir -p "$PARADIGM_ROOT"/{data,config,ssl,logs,backups,postgres,redis,nginx-logs,postgres-config}
    sudo mkdir -p "$CONFIG_DIR"/{node,api,nginx,monitoring}
    sudo mkdir -p "$SSL_DIR"/{certs,private}
    sudo mkdir -p "$LOGS_DIR"/{core,api,nginx}
    sudo mkdir -p secrets
    
    # Set ownership
    sudo chown -R $USER:$USER "$PARADIGM_ROOT"
    chmod -R 755 "$PARADIGM_ROOT"
    chmod 700 "$SSL_DIR/private"
    
    log "✅ Directory structure created"
}

generate_secrets() {
    log "Generating secure secrets..."
    
    # Generate random passwords
    local postgres_password=$(openssl rand -base64 32)
    local grafana_password=$(openssl rand -base64 32)
    local jwt_secret=$(openssl rand -base64 64)
    local api_secret=$(openssl rand -base64 32)
    
    # Save secrets
    echo "$postgres_password" > secrets/postgres_password.txt
    echo "$grafana_password" > secrets/grafana_password.txt
    echo "$jwt_secret" > secrets/jwt_secret.txt
    echo "$api_secret" > secrets/api_secret.txt
    
    chmod 600 secrets/*.txt
    
    log "✅ Secrets generated and stored in secrets/"
}

generate_ssl_certificates() {
    log "Generating SSL certificates..."
    
    # Create CA private key
    openssl genrsa -out "$SSL_DIR/private/ca-key.pem" 4096
    
    # Create CA certificate
    openssl req -new -x509 -days 365 -key "$SSL_DIR/private/ca-key.pem" \
        -out "$SSL_DIR/certs/ca.pem" \
        -subj "/C=US/ST=CA/L=San Francisco/O=Paradigm Network/OU=Blockchain/CN=Paradigm CA"
    
    # Create server private key
    openssl genrsa -out "$SSL_DIR/private/paradigm.key" 4096
    
    # Create server certificate signing request
    openssl req -new -key "$SSL_DIR/private/paradigm.key" \
        -out "$SSL_DIR/certs/paradigm.csr" \
        -subj "/C=US/ST=CA/L=San Francisco/O=Paradigm Network/OU=Blockchain/CN=paradigm.network"
    
    # Create server certificate
    openssl x509 -req -days 365 -in "$SSL_DIR/certs/paradigm.csr" \
        -CA "$SSL_DIR/certs/ca.pem" -CAkey "$SSL_DIR/private/ca-key.pem" \
        -CAcreateserial -out "$SSL_DIR/certs/paradigm.crt"
    
    # Set permissions
    chmod 600 "$SSL_DIR/private"/*.pem "$SSL_DIR/private"/*.key
    chmod 644 "$SSL_DIR/certs"/*.pem "$SSL_DIR/certs"/*.crt "$SSL_DIR/certs"/*.csr
    
    log "✅ SSL certificates generated"
}

create_configuration_files() {
    log "Creating configuration files..."
    
    # Create production environment file
    cat > production.env << EOF
# Environment
ENVIRONMENT=production
NETWORK_ID=paradigm-mainnet-1
CHAIN_ID=paradigm-1

# Core Node Configuration
PARADIGM_RPC_PORT=8545
PARADIGM_P2P_PORT=30303
PARADIGM_WS_PORT=8546
PARADIGM_METRICS_PORT=9090
PARADIGM_DATA_DIR=/data
PARADIGM_LOG_LEVEL=info

# API Configuration
API_PORT=8080
API_HOST=0.0.0.0
JWT_SECRET_FILE=/run/secrets/jwt_secret
RATE_LIMIT_REQUESTS_PER_MINUTE=1000
RATE_LIMIT_BURST=100
API_TIMEOUT=30

# Database Configuration
DATABASE_URL=postgresql://paradigm:\$(cat /run/secrets/postgres_password)@postgres:5432/paradigm_mainnet
REDIS_URL=redis://redis:6379/0
DB_POOL_SIZE=20
REDIS_POOL_SIZE=10

# Cross-Chain Configuration (Update with your API keys)
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/YOUR_INFURA_KEY
BITCOIN_RPC_URL=https://btc-mainnet.gateway.pokt.network/v1/YOUR_POKT_KEY
COSMOS_RPC_URL=https://cosmos-rpc.polkachu.com

# Security
ENABLE_TLS=true
TLS_CERT_PATH=/etc/ssl/certs/paradigm.crt
TLS_KEY_PATH=/etc/ssl/private/paradigm.key
CORS_ORIGINS=https://paradigm.network,https://app.paradigm.network

# Performance
MAX_PEERS=100
CACHE_SIZE=1000MB
WORKER_THREADS=8

# Monitoring
JAEGER_ENDPOINT=http://jaeger:14268/api/traces
PROMETHEUS_ENDPOINT=http://prometheus:9090
ENABLE_METRICS=true
METRICS_INTERVAL=15

# Logging
LOG_FORMAT=json
LOG_OUTPUT=file
LOG_ROTATION=daily
LOG_RETENTION_DAYS=30
EOF

    # Create Nginx configuration
    mkdir -p nginx/conf.d
    cat > nginx/nginx.conf << 'EOF'
user nginx;
worker_processes auto;
worker_rlimit_nofile 65535;

error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 4096;
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Logging
    log_format main '\$remote_addr - \$remote_user [\$time_local] "\$request" '
                    '\$status \$body_bytes_sent "\$http_referer" '
                    '"\$http_user_agent" "\$http_x_forwarded_for" '
                    'rt=\$request_time uct="\$upstream_connect_time" '
                    'uht="\$upstream_header_time" urt="\$upstream_response_time"';

    access_log /var/log/nginx/access.log main;

    # Basic settings
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    client_max_body_size 16M;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/json;

    # Rate limiting
    limit_req_zone \$binary_remote_addr zone=api:10m rate=10r/s;
    limit_req_zone \$binary_remote_addr zone=auth:10m rate=5r/s;

    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;

    # Security headers
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Load balancing
    upstream paradigm_api {
        least_conn;
        server paradigm-api-1:8080 weight=1 max_fails=3 fail_timeout=30s;
        server paradigm-api-2:8080 weight=1 max_fails=3 fail_timeout=30s;
        server paradigm-api-3:8080 weight=1 max_fails=3 fail_timeout=30s;
        keepalive 32;
    }

    # Include additional configurations
    include /etc/nginx/conf.d/*.conf;
}
EOF

    cat > nginx/conf.d/paradigm.conf << 'EOF'
server {
    listen 80;
    server_name _;
    return 301 https://\$server_name\$request_uri;
}

server {
    listen 443 ssl http2;
    server_name paradigm.network *.paradigm.network;

    ssl_certificate /etc/ssl/certs/paradigm.crt;
    ssl_certificate_key /etc/ssl/private/paradigm.key;

    # API endpoints
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        
        proxy_pass http://paradigm_api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_cache_bypass \$http_upgrade;
        
        proxy_connect_timeout 5s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # WebSocket endpoint
    location /ws {
        proxy_pass http://paradigm_api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        
        proxy_connect_timeout 7d;
        proxy_send_timeout 7d;
        proxy_read_timeout 7d;
    }

    # Health check endpoint
    location /health {
        access_log off;
        proxy_pass http://paradigm_api;
        proxy_set_header Host \$host;
    }

    # Static content and documentation
    location / {
        root /usr/share/nginx/html;
        index index.html;
        try_files \$uri \$uri/ /index.html;
    }
}

# Monitoring endpoints (internal only)
server {
    listen 8090;
    server_name localhost;
    allow 172.20.0.0/16;
    deny all;

    location /prometheus/ {
        proxy_pass http://prometheus:9090/;
    }

    location /grafana/ {
        proxy_pass http://grafana:3000/;
    }
}
EOF

    # Create Redis configuration
    cat > config/redis.conf << 'EOF'
# Network
bind 0.0.0.0
port 6379
timeout 300

# Persistence
save 900 1
save 300 10
save 60 10000
rdbcompression yes
rdbchecksum yes

# Memory management
maxmemory 2gb
maxmemory-policy allkeys-lru

# Logging
loglevel notice
syslog-enabled yes
syslog-ident redis

# Security
requirepass \$(openssl rand -base64 32)

# Performance
tcp-keepalive 300
tcp-backlog 511
EOF

    log "✅ Configuration files created"
}

create_monitoring_config() {
    log "Creating monitoring configuration..."
    
    mkdir -p monitoring/{grafana/{dashboards,datasources},alerts}
    
    # Prometheus configuration
    cat > monitoring/prometheus.yml << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'paradigm-mainnet'
    replica: '1'

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
    scrape_timeout: 10s

  - job_name: 'paradigm-api'
    static_configs:
      - targets: 
        - 'paradigm-api-1:9090'
        - 'paradigm-api-2:9090'
        - 'paradigm-api-3:9090'
    metrics_path: /metrics
    scrape_interval: 15s

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis:6379']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'cadvisor'
    static_configs:
      - targets: ['cadvisor:8080']

  - job_name: 'nginx'
    static_configs:
      - targets: ['nginx:80']
    metrics_path: /nginx_status
EOF

    # Alert rules
    cat > monitoring/alerts/paradigm.yml << 'EOF'
groups:
  - name: paradigm_alerts
    rules:
      - alert: ParadigmNodeDown
        expr: up{job="paradigm-core"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Paradigm node is down"
          description: "Paradigm core node has been down for more than 1 minute"

      - alert: HighCPUUsage
        expr: 100 - (avg by(instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage detected"
          description: "CPU usage is above 80% for more than 5 minutes"

      - alert: HighMemoryUsage
        expr: (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100 > 85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"
          description: "Memory usage is above 85% for more than 5 minutes"

      - alert: DiskSpaceLow
        expr: (node_filesystem_size_bytes{mountpoint="/"} - node_filesystem_free_bytes{mountpoint="/"}) / node_filesystem_size_bytes{mountpoint="/"} * 100 > 85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Disk space is low"
          description: "Disk usage is above 85% for more than 5 minutes"

      - alert: DatabaseConnectionsHigh
        expr: pg_stat_database_numbackends > 180
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High number of database connections"
          description: "Database connections are above 180"
EOF

    log "✅ Monitoring configuration created"
}

pull_docker_images() {
    log "Pulling Docker images..."
    
    # Core images
    docker pull postgres:15-alpine
    docker pull redis:7-alpine
    docker pull nginx:alpine
    
    # Monitoring images
    docker pull prom/prometheus:latest
    docker pull grafana/grafana:latest
    docker pull prom/alertmanager:latest
    docker pull prom/node-exporter:latest
    docker pull prometheuscommunity/postgres-exporter:latest
    docker pull gcr.io/cadvisor/cadvisor:latest
    
    # Logging images
    docker pull docker.elastic.co/elasticsearch/elasticsearch:8.8.0
    docker pull docker.elastic.co/kibana/kibana:8.8.0
    docker pull fluent/fluent-bit:latest
    
    # Tracing
    docker pull jaegertracing/all-in-one:latest
    
    log "✅ Docker images pulled"
}

build_paradigm_images() {
    log "Building Paradigm Docker images..."
    
    # Build core node
    if [[ -f "paradigm-core/Dockerfile" ]]; then
        docker build -t paradigm/core:latest ./paradigm-core/
    else
        warn "Paradigm core Dockerfile not found, using placeholder"
        cat > Dockerfile.core << 'EOF'
FROM rust:1.75 as builder
WORKDIR /app
COPY paradigm-core/ .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/paradigm-core /usr/local/bin/
EXPOSE 8545 30303 8546 9090
ENTRYPOINT ["paradigm-core"]
EOF
        docker build -f Dockerfile.core -t paradigm/core:latest .
    fi
    
    # Build API server
    if [[ -f "paradigm-api/Dockerfile" ]]; then
        docker build -t paradigm/api:latest ./paradigm-api/
    else
        warn "Paradigm API Dockerfile not found, using placeholder"
        cat > Dockerfile.api << 'EOF'
FROM rust:1.75 as builder
WORKDIR /app
COPY paradigm-api/ .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/paradigm-api /usr/local/bin/
EXPOSE 8080 9090
ENTRYPOINT ["paradigm-api"]
EOF
        docker build -f Dockerfile.api -t paradigm/api:latest .
    fi
    
    log "✅ Paradigm images built"
}

initialize_database() {
    log "Initializing database..."
    
    # Create database initialization script
    cat > scripts/init-db.sql << 'EOF'
-- Create Paradigm mainnet database
CREATE DATABASE paradigm_mainnet;

-- Create dedicated user
CREATE USER paradigm_user WITH ENCRYPTED PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE paradigm_mainnet TO paradigm_user;

-- Connect to paradigm database
\c paradigm_mainnet;

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gin";

-- Create core tables
CREATE TABLE IF NOT EXISTS blocks (
    height BIGSERIAL PRIMARY KEY,
    hash BYTEA UNIQUE NOT NULL,
    parent_hash BYTEA NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    proposer BYTEA NOT NULL,
    transactions_count INTEGER DEFAULT 0,
    gas_used BIGINT DEFAULT 0,
    gas_limit BIGINT DEFAULT 30000000,
    size_bytes INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hash BYTEA UNIQUE NOT NULL,
    block_height BIGINT REFERENCES blocks(height),
    transaction_index INTEGER,
    from_address BYTEA NOT NULL,
    to_address BYTEA,
    amount BIGINT NOT NULL,
    fee BIGINT NOT NULL,
    gas_used BIGINT,
    gas_price BIGINT,
    nonce BIGINT NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    data BYTEA,
    timestamp TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS accounts (
    address BYTEA PRIMARY KEY,
    balance BIGINT DEFAULT 0,
    nonce BIGINT DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ml_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type VARCHAR(100) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    difficulty INTEGER DEFAULT 1,
    reward BIGINT DEFAULT 0,
    assigned_node BYTEA,
    parameters JSONB,
    result JSONB,
    created_at TIMESTAMP DEFAULT NOW(),
    started_at TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS governance_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    proposer BYTEA NOT NULL,
    proposal_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) DEFAULT 'active',
    voting_start TIMESTAMP NOT NULL,
    voting_end TIMESTAMP NOT NULL,
    yes_votes BIGINT DEFAULT 0,
    no_votes BIGINT DEFAULT 0,
    abstain_votes BIGINT DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_blocks_hash ON blocks(hash);
CREATE INDEX idx_blocks_timestamp ON blocks(timestamp);
CREATE INDEX idx_transactions_hash ON transactions(hash);
CREATE INDEX idx_transactions_block_height ON transactions(block_height);
CREATE INDEX idx_transactions_from_address ON transactions(from_address);
CREATE INDEX idx_transactions_to_address ON transactions(to_address);
CREATE INDEX idx_transactions_timestamp ON transactions(timestamp);
CREATE INDEX idx_accounts_balance ON accounts(balance);
CREATE INDEX idx_ml_tasks_status ON ml_tasks(status);
CREATE INDEX idx_ml_tasks_assigned_node ON ml_tasks(assigned_node);
CREATE INDEX idx_governance_proposals_status ON governance_proposals(status);

-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO paradigm_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO paradigm_user;
EOF

    log "✅ Database initialization script created"
}

start_services() {
    log "Starting Paradigm network services..."
    
    # Start infrastructure services first
    docker-compose -f docker-compose.prod.yml up -d postgres redis elasticsearch
    
    # Wait for databases to be ready
    log "Waiting for databases to initialize..."
    sleep 30
    
    # Start core services
    docker-compose -f docker-compose.prod.yml up -d paradigm-core
    
    # Wait for core to be ready
    log "Waiting for Paradigm core to start..."
    sleep 30
    
    # Start API services
    docker-compose -f docker-compose.prod.yml up -d paradigm-api-1 paradigm-api-2 paradigm-api-3
    
    # Start monitoring
    docker-compose -f docker-compose.prod.yml up -d prometheus grafana alertmanager
    
    # Start remaining services
    docker-compose -f docker-compose.prod.yml up -d
    
    log "✅ All services started"
}

run_health_checks() {
    log "Running health checks..."
    
    local max_attempts=30
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        log "Health check attempt $attempt/$max_attempts..."
        
        # Check core node
        if curl -f http://localhost:8545/health &>/dev/null; then
            log "✅ Paradigm core is healthy"
            break
        fi
        
        if [[ $attempt -eq $max_attempts ]]; then
            error "Health checks failed after $max_attempts attempts"
        fi
        
        sleep 10
        ((attempt++))
    done
    
    # Check API endpoints
    curl -f http://localhost:8080/health || error "API health check failed"
    log "✅ Paradigm API is healthy"
    
    # Check database
    docker-compose -f docker-compose.prod.yml exec -T postgres pg_isready -U paradigm || error "Database health check failed"
    log "✅ Database is healthy"
    
    # Check monitoring
    curl -f http://localhost:9090/-/healthy || warn "Prometheus health check failed"
    curl -f http://localhost:3000/api/health || warn "Grafana health check failed"
    
    log "✅ Health checks completed"
}

create_genesis_block() {
    log "Creating genesis block..."
    
    # This would typically interact with the paradigm-core binary
    # For now, we'll create a placeholder
    cat > config/genesis.json << 'EOF'
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
  }
}
EOF
    
    log "✅ Genesis configuration created"
}

setup_backup_system() {
    log "Setting up backup system..."
    
    cat > scripts/backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="/var/lib/paradigm/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR/$DATE"

# Backup database
docker-compose -f docker-compose.prod.yml exec -T postgres pg_dump -U paradigm paradigm_mainnet > "$BACKUP_DIR/$DATE/database.sql"

# Backup blockchain data
tar -czf "$BACKUP_DIR/$DATE/blockchain_data.tar.gz" -C /var/lib/paradigm/data .

# Backup configuration
tar -czf "$BACKUP_DIR/$DATE/config.tar.gz" -C /var/lib/paradigm/config .

# Clean old backups (keep last 7 days)
find "$BACKUP_DIR" -type d -mtime +7 -exec rm -rf {} +

echo "Backup completed: $BACKUP_DIR/$DATE"
EOF
    
    chmod +x scripts/backup.sh
    
    # Add to crontab for daily backups
    (crontab -l 2>/dev/null; echo "0 2 * * * /path/to/paradigm/scripts/backup.sh") | crontab -
    
    log "✅ Backup system configured"
}

print_summary() {
    log "Deployment completed successfully! 🎉"
    echo ""
    echo -e "${GREEN}=== Paradigm Network Status ===${NC}"
    echo -e "${BLUE}Core Node RPC:${NC}     http://localhost:8545"
    echo -e "${BLUE}Core Node WebSocket:${NC} ws://localhost:8546"
    echo -e "${BLUE}API Endpoint:${NC}      https://localhost:443"
    echo -e "${BLUE}Grafana Dashboard:${NC} http://localhost:3000"
    echo -e "${BLUE}Prometheus:${NC}        http://localhost:9090"
    echo -e "${BLUE}Kibana Logs:${NC}       http://localhost:5601"
    echo ""
    echo -e "${YELLOW}Default Credentials:${NC}"
    echo -e "${BLUE}Grafana:${NC} admin / $(cat secrets/grafana_password.txt)"
    echo ""
    echo -e "${YELLOW}Important Files:${NC}"
    echo -e "${BLUE}Configuration:${NC}     /var/lib/paradigm/config/"
    echo -e "${BLUE}SSL Certificates:${NC}  /var/lib/paradigm/ssl/"
    echo -e "${BLUE}Logs:${NC}              /var/lib/paradigm/logs/"
    echo -e "${BLUE}Backups:${NC}           /var/lib/paradigm/backups/"
    echo ""
    echo -e "${YELLOW}Next Steps:${NC}"
    echo "1. Update API keys in production.env"
    echo "2. Configure domain name and proper SSL certificates"
    echo "3. Set up monitoring alerts"
    echo "4. Review security settings"
    echo "5. Start validating on the network"
    echo ""
    echo -e "${GREEN}Documentation: https://docs.paradigm.network${NC}"
}

# Main execution
main() {
    print_banner
    
    log "Starting Paradigm Network production deployment..."
    
    check_requirements
    create_directories
    generate_secrets
    generate_ssl_certificates
    create_configuration_files
    create_monitoring_config
    initialize_database
    pull_docker_images
    build_paradigm_images
    start_services
    create_genesis_block
    run_health_checks
    setup_backup_system
    
    print_summary
}

# Run main function
main "$@"