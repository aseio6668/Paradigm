use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::sigil::Sigil;
use super::keeper::Keeper;
use super::proofs::{StorageChallenge, StorageProof, RetrievabilityProof};

/// Network protocol messages for storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageMessage {
    /// Request to store a sigil
    StoreRequest {
        sigil: Sigil,
        originator_signature: Option<String>,
    },
    
    /// Response to store request
    StoreResponse {
        success: bool,
        error: Option<String>,
        keeper_id: String,
    },
    
    /// Request to retrieve a sigil
    RetrieveRequest {
        sigil_hash: String,
        requester_id: String,
        byte_ranges: Option<Vec<(usize, usize)>>,
    },
    
    /// Response with sigil data
    RetrieveResponse {
        success: bool,
        data: Option<Vec<u8>>,
        error: Option<String>,
        response_time_ms: u64,
    },
    
    /// Storage proof challenge
    ProofChallenge {
        challenge: StorageChallenge,
    },
    
    /// Storage proof response
    ProofResponse {
        proof: StorageProof,
    },
    
    /// Keeper heartbeat and status
    Heartbeat {
        keeper: Keeper,
        sigil_count: usize,
    },
    
    /// Request keeper status
    StatusRequest {
        requester_id: String,
    },
    
    /// Keeper status response
    StatusResponse {
        keeper: Keeper,
        online: bool,
    },
    
    /// Replicate sigil to additional keepers
    ReplicationRequest {
        sigil_hash: String,
        target_keeper_count: usize,
    },
    
    /// Response to replication request
    ReplicationResponse {
        success: bool,
        new_keepers: Vec<String>,
    },
}

/// Network client for communicating with keepers
pub struct StorageNetworkClient {
    /// Connection timeout in milliseconds
    timeout_ms: u64,
}

impl StorageNetworkClient {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }
    
    /// Send a store request to a keeper
    pub async fn store_sigil(&self, keeper_address: &str, sigil: Sigil) -> Result<bool> {
        let message = StorageMessage::StoreRequest {
            sigil,
            originator_signature: None, // TODO: Implement signature verification
        };
        
        let response = self.send_message(keeper_address, message).await?;
        
        match response {
            StorageMessage::StoreResponse { success, error, .. } => {
                if success {
                    Ok(true)
                } else {
                    Err(anyhow!("Store failed: {}", error.unwrap_or_default()))
                }
            }
            _ => Err(anyhow!("Unexpected response type")),
        }
    }
    
    /// Send a retrieve request to a keeper
    pub async fn retrieve_sigil(&self, keeper_address: &str, sigil_hash: &str, requester_id: &str) -> Result<Vec<u8>> {
        let message = StorageMessage::RetrieveRequest {
            sigil_hash: sigil_hash.to_string(),
            requester_id: requester_id.to_string(),
            byte_ranges: None,
        };
        
        let response = self.send_message(keeper_address, message).await?;
        
        match response {
            StorageMessage::RetrieveResponse { success, data, error, .. } => {
                if success {
                    data.ok_or_else(|| anyhow!("No data in successful response"))
                } else {
                    Err(anyhow!("Retrieve failed: {}", error.unwrap_or_default()))
                }
            }
            _ => Err(anyhow!("Unexpected response type")),
        }
    }
    
    /// Send a proof challenge to a keeper
    pub async fn send_proof_challenge(&self, keeper_address: &str, challenge: StorageChallenge) -> Result<StorageProof> {
        let message = StorageMessage::ProofChallenge { challenge };
        
        let response = self.send_message(keeper_address, message).await?;
        
        match response {
            StorageMessage::ProofResponse { proof } => Ok(proof),
            _ => Err(anyhow!("Unexpected response type")),
        }
    }
    
    /// Get keeper status
    pub async fn get_keeper_status(&self, keeper_address: &str, requester_id: &str) -> Result<(Keeper, bool)> {
        let message = StorageMessage::StatusRequest {
            requester_id: requester_id.to_string(),
        };
        
        let response = self.send_message(keeper_address, message).await?;
        
        match response {
            StorageMessage::StatusResponse { keeper, online } => Ok((keeper, online)),
            _ => Err(anyhow!("Unexpected response type")),
        }
    }
    
    /// Send a message to a keeper and wait for response
    async fn send_message(&self, keeper_address: &str, message: StorageMessage) -> Result<StorageMessage> {
        // Connect to keeper
        let mut stream = tokio::time::timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            TcpStream::connect(keeper_address),
        ).await??;
        
        // Serialize and send message
        let message_bytes = serde_json::to_vec(&message)?;
        let message_len = message_bytes.len() as u32;
        
        stream.write_all(&message_len.to_be_bytes()).await?;
        stream.write_all(&message_bytes).await?;
        
        // Read response
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let response_len = u32::from_be_bytes(len_bytes) as usize;
        
        let mut response_bytes = vec![0u8; response_len];
        stream.read_exact(&mut response_bytes).await?;
        
        let response: StorageMessage = serde_json::from_slice(&response_bytes)?;
        Ok(response)
    }
}

/// Network server for handling storage requests as a keeper
pub struct StorageNetworkServer {
    /// Local keeper instance
    keeper: Keeper,
    
    /// Storage directory
    storage_path: String,
    
    /// Active connections
    connections: HashMap<String, TcpStream>,
}

impl StorageNetworkServer {
    pub fn new(keeper: Keeper, storage_path: String) -> Self {
        Self {
            keeper,
            storage_path,
            connections: HashMap::new(),
        }
    }
    
    /// Start the storage server
    pub async fn start(&mut self, bind_address: &str) -> Result<()> {
        let listener = TcpListener::bind(bind_address).await?;
        println!("🔗 Keeper {} listening on {}", self.keeper.keeper_id, bind_address);
        
        loop {
            let (stream, addr) = listener.accept().await?;
            println!("📡 New connection from {}", addr);
            
            // Handle connection in background task
            let mut keeper_clone = self.keeper.clone();
            let storage_path_clone = self.storage_path.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, &mut keeper_clone, &storage_path_clone).await {
                    eprintln!("❌ Connection error: {}", e);
                }
            });
        }
    }
    
    /// Handle a single connection
    async fn handle_connection(mut stream: TcpStream, keeper: &mut Keeper, _storage_path: &str) -> Result<()> {
        loop {
            // Read message length
            let mut len_bytes = [0u8; 4];
            match stream.read_exact(&mut len_bytes).await {
                Ok(_) => {},
                Err(_) => break, // Connection closed
            }
            
            let message_len = u32::from_be_bytes(len_bytes) as usize;
            if message_len > 10 * 1024 * 1024 { // 10MB limit
                return Err(anyhow!("Message too large: {} bytes", message_len));
            }
            
            // Read message data
            let mut message_bytes = vec![0u8; message_len];
            stream.read_exact(&mut message_bytes).await?;
            
            let message: StorageMessage = serde_json::from_slice(&message_bytes)?;
            let response = Self::handle_message(message, keeper).await;
            
            // Send response
            let response_bytes = serde_json::to_vec(&response)?;
            let response_len = response_bytes.len() as u32;
            
            stream.write_all(&response_len.to_be_bytes()).await?;
            stream.write_all(&response_bytes).await?;
        }
        
        Ok(())
    }
    
    /// Handle a storage protocol message
    async fn handle_message(message: StorageMessage, keeper: &mut Keeper) -> StorageMessage {
        match message {
            StorageMessage::StoreRequest { sigil, .. } => {
                let result = keeper.store_sigil(sigil).await;
                StorageMessage::StoreResponse {
                    success: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                    keeper_id: keeper.keeper_id.clone(),
                }
            }
            
            StorageMessage::RetrieveRequest { sigil_hash, requester_id, .. } => {
                let start_time = std::time::Instant::now();
                let result = keeper.retrieve_sigil(&sigil_hash).await;
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                
                // Record retrieval metrics
                keeper.record_retrieval(result.is_ok(), response_time_ms);
                
                match result {
                    Ok(data) => {
                        println!("✅ Retrieved sigil {} for {}", sigil_hash, requester_id);
                        StorageMessage::RetrieveResponse {
                            success: true,
                            data: Some(data),
                            error: None,
                            response_time_ms,
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to retrieve sigil {}: {}", sigil_hash, e);
                        StorageMessage::RetrieveResponse {
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                            response_time_ms,
                        }
                    }
                }
            }
            
            StorageMessage::ProofChallenge { challenge } => {
                let proof_result = keeper.generate_storage_proof(&challenge.sigil_hash, &challenge.challenge_data).await;
                
                match proof_result {
                    Ok(proof) => {
                        StorageMessage::ProofResponse { proof }
                    }
                    Err(_) => {
                        // Generate empty proof on error
                        StorageMessage::ProofResponse {
                            proof: StorageProof {
                                keeper_id: keeper.keeper_id.clone(),
                                sigil_hash: challenge.sigil_hash,
                                challenge: challenge.challenge_data,
                                proof_data: vec![],
                                timestamp: chrono::Utc::now(),
                            }
                        }
                    }
                }
            }
            
            StorageMessage::StatusRequest { .. } => {
                keeper.heartbeat(); // Update last seen time
                StorageMessage::StatusResponse {
                    keeper: keeper.clone(),
                    online: true,
                }
            }
            
            StorageMessage::Heartbeat { .. } => {
                keeper.heartbeat();
                StorageMessage::StatusResponse {
                    keeper: keeper.clone(),
                    online: true,
                }
            }
            
            _ => {
                // Default response for unhandled messages
                StorageMessage::StoreResponse {
                    success: false,
                    error: Some("Message type not supported".to_string()),
                    keeper_id: keeper.keeper_id.clone(),
                }
            }
        }
    }
}

/// Network discovery and management for finding keepers
pub struct KeeperDiscovery {
    /// Known keeper addresses
    known_keepers: HashMap<String, String>,
    
    /// Keeper status cache
    status_cache: HashMap<String, (Keeper, std::time::Instant)>,
}

impl KeeperDiscovery {
    pub fn new() -> Self {
        Self {
            known_keepers: HashMap::new(),
            status_cache: HashMap::new(),
        }
    }
    
    /// Add a keeper to the known list
    pub fn add_keeper(&mut self, keeper_id: String, address: String) {
        self.known_keepers.insert(keeper_id, address);
    }
    
    /// Get all known keeper addresses
    pub fn get_known_keepers(&self) -> &HashMap<String, String> {
        &self.known_keepers
    }
    
    /// Find the best keepers for storing a sigil
    pub async fn select_keepers_for_storage(&mut self, count: usize) -> Result<Vec<(String, String)>> {
        let client = StorageNetworkClient::new(5000); // 5 second timeout
        let mut available_keepers = Vec::new();
        
        // Check status of all known keepers
        for (keeper_id, address) in &self.known_keepers {
            if let Ok((keeper, online)) = client.get_keeper_status(address, "storage_engine").await {
                if online && keeper.has_capacity_for_sigil() {
                    available_keepers.push((keeper_id.clone(), address.clone(), keeper.reputation));
                }
            }
        }
        
        // Sort by reputation (highest first)
        available_keepers.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top keepers
        Ok(available_keepers
            .into_iter()
            .take(count)
            .map(|(id, addr, _)| (id, addr))
            .collect())
    }
    
    /// Ping all keepers to check availability
    pub async fn ping_all_keepers(&mut self) -> HashMap<String, bool> {
        let client = StorageNetworkClient::new(3000); // 3 second timeout
        let mut results = HashMap::new();
        
        for (keeper_id, address) in &self.known_keepers {
            let online = client.get_keeper_status(address, "ping").await.is_ok();
            results.insert(keeper_id.clone(), online);
        }
        
        results
    }
    
    /// Remove offline keepers from the known list
    pub async fn cleanup_offline_keepers(&mut self) {
        let ping_results = self.ping_all_keepers().await;
        
        let offline_keepers: Vec<String> = ping_results
            .into_iter()
            .filter(|(_, online)| !online)
            .map(|(id, _)| id)
            .collect();
        
        for keeper_id in offline_keepers {
            self.known_keepers.remove(&keeper_id);
            self.status_cache.remove(&keeper_id);
        }
    }
}