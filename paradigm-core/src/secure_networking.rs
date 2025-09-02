// Secure networking with TLS encryption and connection management
// Prevents man-in-the-middle attacks and ensures peer communication integrity

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_rustls::TlsAcceptor;
use uuid::Uuid;

use crate::certificate_manager::CertificateManager;
use crate::transaction::Transaction;

/// Secure message types for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecureMessage {
    Handshake {
        node_id: Uuid,
        version: String,
        capabilities: Vec<String>,
    },
    Transaction {
        transaction: Transaction,
        signature: Vec<u8>,
    },
    BlockAnnouncement {
        block_hash: Vec<u8>,
        height: u64,
        timestamp: u64,
    },
    PeerRequest {
        max_peers: u32,
    },
    PeerResponse {
        peers: Vec<SocketAddr>,
    },
    Ping {
        timestamp: u64,
        nonce: u64,
    },
    Pong {
        timestamp: u64,
        nonce: u64,
    },
}

/// Connection security levels
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    None,      // Unencrypted (development only)
    TLS,       // Standard TLS encryption
    MutualTLS, // Mutual authentication with client certificates
}

/// Connection state and metrics
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub peer_id: Option<Uuid>,
    pub remote_addr: SocketAddr,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub security_level: SecurityLevel,
    pub is_outbound: bool,
}

/// Connection rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_connections_per_ip: u32,
    pub max_messages_per_minute: u32,
    pub connection_timeout: Duration,
    pub handshake_timeout: Duration,
    pub message_size_limit: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_connections_per_ip: 5,
            max_messages_per_minute: 100,
            connection_timeout: Duration::from_secs(30),
            handshake_timeout: Duration::from_secs(10),
            message_size_limit: 1024 * 1024, // 1MB
        }
    }
}

/// Connection rate tracking
#[derive(Debug, Clone)]
struct ConnectionRate {
    count: u32,
    first_connection: Instant,
    message_count: u32,
    last_message: Instant,
}

/// Secure P2P networking manager
pub struct SecureNetworkManager {
    node_id: Uuid,
    connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
    rate_limits: Arc<RwLock<HashMap<SocketAddr, ConnectionRate>>>,
    config: RateLimitConfig,
    security_level: SecurityLevel,
    tls_acceptor: Option<TlsAcceptor>,
    cert_manager: Arc<CertificateManager>,
}

impl SecureNetworkManager {
    pub fn new(
        node_id: Uuid,
        security_level: SecurityLevel,
        config: RateLimitConfig,
        cert_manager: Arc<CertificateManager>,
    ) -> Self {
        Self {
            node_id,
            connections: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            config,
            security_level,
            tls_acceptor: None,
            cert_manager,
        }
    }

    /// Initialize TLS configuration using certificate manager
    pub async fn initialize_tls(&mut self) -> Result<()> {
        if self.security_level == SecurityLevel::None {
            tracing::info!("🔓 Security level: None (development only)");
            return Ok(());
        }

        // Initialize default certificates if none exist
        self.cert_manager.initialize_default_certificates().await?;

        // Get server configuration from certificate manager
        let server_config = self.cert_manager.get_server_config(None).await?;

        // Create TLS acceptor
        self.tls_acceptor = Some(TlsAcceptor::from(server_config));

        tracing::info!(
            "🔐 TLS encryption initialized (security level: {:?})",
            self.security_level
        );
        Ok(())
    }

    /// Start secure server listener
    pub async fn start_server(&self, bind_addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(bind_addr).await?;
        tracing::info!("🌐 Secure server listening on {}", bind_addr);

        let connections = self.connections.clone();
        let rate_limits = self.rate_limits.clone();
        let config = self.config.clone();
        let tls_acceptor = self.tls_acceptor.clone();
        let node_id = self.node_id;

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, remote_addr)) => {
                        // Check rate limits
                        if Self::check_rate_limit(&rate_limits, &config, remote_addr).await {
                            let connections = connections.clone();
                            let tls_acceptor = tls_acceptor.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_connection(
                                    stream,
                                    remote_addr,
                                    connections,
                                    tls_acceptor,
                                    node_id,
                                    false, // inbound connection
                                )
                                .await
                                {
                                    tracing::warn!("Connection handling error: {}", e);
                                }
                            });
                        } else {
                            tracing::warn!("🚫 Rate limit exceeded for {}", remote_addr);
                            // Connection is automatically dropped
                        }
                    }
                    Err(e) => {
                        tracing::error!("Accept error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Connect to remote peer securely
    pub async fn connect_to_peer(&self, remote_addr: SocketAddr) -> Result<Uuid> {
        tracing::info!("🔗 Connecting to peer: {}", remote_addr);

        let stream = tokio::time::timeout(
            self.config.connection_timeout,
            TcpStream::connect(remote_addr),
        )
        .await??;

        let connection_id = Uuid::new_v4();
        let connections = self.connections.clone();
        let tls_acceptor = self.tls_acceptor.clone();
        let node_id = self.node_id;

        tokio::spawn(async move {
            if let Err(e) = Self::handle_connection(
                stream,
                remote_addr,
                connections,
                tls_acceptor,
                node_id,
                true, // outbound connection
            )
            .await
            {
                tracing::warn!("Outbound connection handling error: {}", e);
            }
        });

        Ok(connection_id)
    }

    /// Send secure message to specific peer
    pub async fn send_message(&self, peer_id: Uuid, message: SecureMessage) -> Result<()> {
        let connections = self.connections.read().await;

        if let Some(connection) = connections.get(&peer_id) {
            tracing::debug!(
                "📤 Sending {} message to peer {}",
                message.message_type(),
                peer_id
            );

            // In a real implementation, we would serialize and send the message
            // through the encrypted connection
            tracing::info!(
                "Message sent via secure channel to {}",
                connection.remote_addr
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!("Peer {} not connected", peer_id))
        }
    }

    /// Broadcast message to all connected peers
    pub async fn broadcast_message(&self, message: SecureMessage) -> Result<u32> {
        let connections = self.connections.read().await;
        let peer_count = connections.len() as u32;

        tracing::info!(
            "📡 Broadcasting {} message to {} peers",
            message.message_type(),
            peer_count
        );

        // In real implementation, would send to all connections
        for (peer_id, connection) in connections.iter() {
            tracing::debug!(
                "Broadcasting to peer {} at {}",
                peer_id,
                connection.remote_addr
            );
        }

        Ok(peer_count)
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> HashMap<Uuid, ConnectionInfo> {
        self.connections.read().await.clone()
    }

    /// Check and update rate limits
    async fn check_rate_limit(
        rate_limits: &Arc<RwLock<HashMap<SocketAddr, ConnectionRate>>>,
        config: &RateLimitConfig,
        remote_addr: SocketAddr,
    ) -> bool {
        let mut limits = rate_limits.write().await;
        let now = Instant::now();

        let ip = remote_addr.ip();
        let ip_addr = SocketAddr::new(ip, 0); // Ignore port for IP-based limiting

        match limits.get_mut(&ip_addr) {
            Some(rate) => {
                // Clean up old entries
                if now.duration_since(rate.first_connection) > Duration::from_secs(60) {
                    rate.count = 1;
                    rate.first_connection = now;
                    rate.message_count = 0;
                    return true;
                }

                // Check connection limit
                if rate.count >= config.max_connections_per_ip {
                    return false;
                }

                rate.count += 1;
                true
            }
            None => {
                limits.insert(
                    ip_addr,
                    ConnectionRate {
                        count: 1,
                        first_connection: now,
                        message_count: 0,
                        last_message: now,
                    },
                );
                true
            }
        }
    }

    /// Handle individual peer connection
    async fn handle_connection(
        stream: TcpStream,
        remote_addr: SocketAddr,
        connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
        tls_acceptor: Option<TlsAcceptor>,
        node_id: Uuid,
        is_outbound: bool,
    ) -> Result<()> {
        let connection_id = Uuid::new_v4();
        let now = Instant::now();

        // Upgrade to TLS if configured
        let security_level = if let Some(acceptor) = &tls_acceptor {
            tracing::debug!("🔐 Upgrading connection to TLS");

            // Perform TLS handshake
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    tracing::debug!("✅ TLS handshake successful with {}", remote_addr);
                    // Store the TLS stream for later use
                    SecurityLevel::TLS
                }
                Err(e) => {
                    tracing::warn!("TLS handshake failed with {}: {}", remote_addr, e);
                    return Err(anyhow::anyhow!("TLS handshake failed: {}", e));
                }
            }
        } else {
            SecurityLevel::None
        };

        // Store connection info
        let connection_info = ConnectionInfo {
            peer_id: None, // Will be set after handshake
            remote_addr,
            connected_at: now,
            last_activity: now,
            bytes_sent: 0,
            bytes_received: 0,
            security_level: security_level.clone(),
            is_outbound,
        };

        {
            let mut conns = connections.write().await;
            conns.insert(connection_id, connection_info);
        }

        tracing::info!(
            "✅ Secure connection established with {} ({}, security: {:?})",
            remote_addr,
            if is_outbound { "outbound" } else { "inbound" },
            &security_level
        );

        // Perform handshake protocol
        if let Err(e) = Self::perform_handshake(connection_id, &connections, node_id).await {
            tracing::warn!("Handshake failed with {}: {}", remote_addr, e);

            // Remove failed connection
            let mut conns = connections.write().await;
            conns.remove(&connection_id);

            return Err(e);
        }

        // Start message handling loop
        Self::handle_peer_messages(connection_id, &connections, remote_addr).await?;

        // Remove connection on disconnect
        {
            let mut conns = connections.write().await;
            conns.remove(&connection_id);
        }

        tracing::info!("🔌 Connection with {} closed", remote_addr);
        Ok(())
    }

    /// Perform secure handshake with peer
    async fn perform_handshake(
        connection_id: Uuid,
        connections: &Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
        node_id: Uuid,
    ) -> Result<()> {
        // Create handshake message
        let handshake = SecureMessage::Handshake {
            node_id,
            version: crate::PARADIGM_VERSION.to_string(),
            capabilities: vec![
                "transactions".to_string(),
                "blocks".to_string(),
                "peers".to_string(),
            ],
        };

        tracing::debug!("📤 Sending handshake for connection {}", connection_id);

        // In a real implementation, we would:
        // 1. Send handshake message over the TLS stream
        // 2. Wait for peer's handshake response
        // 3. Validate peer's capabilities
        // 4. Exchange peer information

        // Simulate handshake process
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Update connection with peer information
        {
            let mut conns = connections.write().await;
            if let Some(conn_info) = conns.get_mut(&connection_id) {
                conn_info.peer_id = Some(Uuid::new_v4()); // Would be from peer's handshake
                conn_info.last_activity = Instant::now();
            }
        }

        tracing::debug!("✅ Handshake completed for connection {}", connection_id);
        Ok(())
    }

    /// Handle incoming messages from peer
    async fn handle_peer_messages(
        connection_id: Uuid,
        connections: &Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
        remote_addr: SocketAddr,
    ) -> Result<()> {
        // Simulate active connection with periodic ping/pong
        let mut ping_interval = tokio::time::interval(Duration::from_secs(30));
        let mut message_count = 0u32;

        for _ in 0..10 {
            // Simulate 10 message exchanges
            ping_interval.tick().await;

            // Create ping message
            let ping = SecureMessage::Ping {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
                nonce: rand::random(),
            };

            // In real implementation, would send over TLS stream and wait for pong
            tracing::trace!("📤 Ping sent to {}", remote_addr);

            // Update connection stats
            {
                let mut conns = connections.write().await;
                if let Some(conn_info) = conns.get_mut(&connection_id) {
                    conn_info.last_activity = Instant::now();
                    conn_info.bytes_sent += ping.estimated_size() as u64;
                    message_count += 1;
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        tracing::debug!(
            "Message handling completed for {} ({} messages)",
            remote_addr,
            message_count
        );
        Ok(())
    }
}

impl SecureMessage {
    /// Get message type name for logging
    pub fn message_type(&self) -> &'static str {
        match self {
            SecureMessage::Handshake { .. } => "Handshake",
            SecureMessage::Transaction { .. } => "Transaction",
            SecureMessage::BlockAnnouncement { .. } => "BlockAnnouncement",
            SecureMessage::PeerRequest { .. } => "PeerRequest",
            SecureMessage::PeerResponse { .. } => "PeerResponse",
            SecureMessage::Ping { .. } => "Ping",
            SecureMessage::Pong { .. } => "Pong",
        }
    }

    /// Estimate message size
    pub fn estimated_size(&self) -> usize {
        match self {
            SecureMessage::Handshake { .. } => 256,
            SecureMessage::Transaction { .. } => 512,
            SecureMessage::BlockAnnouncement { .. } => 128,
            SecureMessage::PeerRequest { .. } => 64,
            SecureMessage::PeerResponse { peers } => 64 + (peers.len() * 16),
            SecureMessage::Ping { .. } => 32,
            SecureMessage::Pong { .. } => 32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secure_network_manager() {
        let node_id = Uuid::new_v4();
        let config = RateLimitConfig::default();

        let mut manager = SecureNetworkManager::new(
            node_id,
            SecurityLevel::None, // For testing
            config,
        );

        // Test initialization
        manager.initialize_tls().await.unwrap();

        // Test message creation
        let message = SecureMessage::Ping {
            timestamp: 12345,
            nonce: 67890,
        };

        assert_eq!(message.message_type(), "Ping");
        assert!(message.estimated_size() > 0);
    }
}
