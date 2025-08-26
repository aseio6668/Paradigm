// Core types for the Paradigm SDK

use crate::error::{ParadigmError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Paradigm address type (20 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    bytes: [u8; 20],
}

impl Address {
    /// Create address from bytes
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Self { bytes }
    }

    /// Create address from hex string
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        if hex.len() != 40 {
            return Err(ParadigmError::InvalidAddress("Invalid length".to_string()));
        }

        let bytes = hex::decode(hex).map_err(|e| ParadigmError::InvalidAddress(e.to_string()))?;

        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&bytes);
        Ok(Self::from_bytes(addr_bytes))
    }

    /// Get address as bytes
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.bytes
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.bytes))
    }

    /// Check if address is zero
    pub fn is_zero(&self) -> bool {
        self.bytes == [0u8; 20]
    }

    /// Create zero address
    pub fn zero() -> Self {
        Self::from_bytes([0u8; 20])
    }

    /// Create address from public key
    pub fn from_public_key(public_key: &[u8]) -> Result<Self> {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(public_key);
        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&hash[12..32]);
        Ok(Address::from_bytes(addr_bytes))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for Address {
    type Err = ParadigmError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_hex(s)
    }
}

impl Default for Address {
    fn default() -> Self {
        Self::zero()
    }
}

/// Hash type (32 bytes)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash {
    bytes: [u8; 32],
}

impl Hash {
    /// Create hash from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Create hash from hex string
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        if hex.len() != 64 {
            return Err(ParadigmError::InvalidHash("Invalid length".to_string()));
        }

        let bytes = hex::decode(hex).map_err(|e| ParadigmError::InvalidHash(e.to_string()))?;

        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&bytes);
        Ok(Self::from_bytes(hash_bytes))
    }

    /// Get hash as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.bytes))
    }

    /// Check if hash is zero
    pub fn is_zero(&self) -> bool {
        self.bytes == [0u8; 32]
    }

    /// Create zero hash
    pub fn zero() -> Self {
        Self::from_bytes([0u8; 32])
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for Hash {
    type Err = ParadigmError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_hex(s)
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self::zero()
    }
}

/// Signature type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    bytes: Vec<u8>,
    signature_type: SignatureType,
    pub r: Vec<u8>,
    pub s: Vec<u8>,
    pub recovery_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureType {
    Ed25519,
    Secp256k1,
    BLS,
}

impl Signature {
    /// Create signature from bytes
    pub fn new(bytes: Vec<u8>, signature_type: SignatureType) -> Self {
        let (r, s) = if bytes.len() >= 64 {
            (bytes[..32].to_vec(), bytes[32..64].to_vec())
        } else {
            (vec![0; 32], vec![0; 32])
        };
        Self {
            bytes,
            signature_type,
            r,
            s,
            recovery_id: 0,
        }
    }

    /// Get signature bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get signature type
    pub fn signature_type(&self) -> &SignatureType {
        &self.signature_type
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.bytes))
    }
}

/// Amount type for representing token amounts
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Amount {
    wei: u64,
}

impl Amount {
    /// Create amount from wei
    pub fn from_wei(wei: u64) -> Self {
        Self { wei }
    }

    /// Create amount from Paradigm tokens
    pub fn from_paradigm(paradigm: f64) -> Self {
        let wei = (paradigm * 10_f64.powi(crate::constants::NATIVE_TOKEN_DECIMALS as i32)) as u64;
        Self::from_wei(wei)
    }

    /// Get amount in wei
    pub fn value(&self) -> u64 {
        self.wei
    }

    /// Get amount in wei (alias for compatibility)
    pub fn wei(&self) -> u64 {
        self.wei
    }

    /// Get amount in Paradigm tokens
    pub fn to_paradigm(&self) -> f64 {
        self.wei as f64 / 10_f64.powi(crate::constants::NATIVE_TOKEN_DECIMALS as i32)
    }

    /// Check if amount is zero
    pub fn is_zero(&self) -> bool {
        self.wei == 0
    }

    /// Create zero amount
    pub fn zero() -> Self {
        Self::from_wei(0)
    }

    /// Add amounts
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.wei.checked_add(other.wei).map(Self::from_wei)
    }

    /// Subtract amounts
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.wei.checked_sub(other.wei).map(Self::from_wei)
    }

    /// Multiply amount by scalar
    pub fn checked_mul(self, scalar: u64) -> Option<Self> {
        self.wei.checked_mul(scalar).map(Self::from_wei)
    }

    /// Divide amount by scalar
    pub fn checked_div(self, scalar: u64) -> Option<Self> {
        if scalar == 0 {
            None
        } else {
            Some(Self::from_wei(self.wei / scalar))
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} PARADIGM", self.to_paradigm())
    }
}

impl std::ops::Add for Amount {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.checked_add(other).expect("Amount overflow")
    }
}

impl std::ops::Sub for Amount {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.checked_sub(other).expect("Amount underflow")
    }
}

/// Balance information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    /// Available balance
    pub available: Amount,

    /// Locked/staked balance
    pub locked: Amount,

    /// Pending balance (unconfirmed)
    pub pending: Amount,
}

impl Balance {
    /// Create new balance
    pub fn new(available: Amount, locked: Amount, pending: Amount) -> Self {
        Self {
            available,
            locked,
            pending,
        }
    }

    /// Get total balance
    pub fn total(&self) -> Amount {
        self.available + self.locked + self.pending
    }

    /// Check if balance is zero
    pub fn is_zero(&self) -> bool {
        self.total().is_zero()
    }

    /// Create zero balance
    pub fn zero() -> Self {
        Self::new(Amount::zero(), Amount::zero(), Amount::zero())
    }
}

/// Fee information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fee {
    /// Base fee
    pub base_fee: Amount,

    /// Priority fee (tip)
    pub priority_fee: Amount,

    /// Maximum fee
    pub max_fee: Amount,

    /// Gas limit
    pub gas_limit: u64,
}

impl Fee {
    /// Create new fee
    pub fn new(base_fee: Amount, priority_fee: Amount, max_fee: Amount, gas_limit: u64) -> Self {
        Self {
            base_fee,
            priority_fee,
            max_fee,
            gas_limit,
        }
    }

    /// Calculate total fee
    pub fn total(&self) -> Amount {
        Amount::from_wei((self.base_fee.value() + self.priority_fee.value()) * self.gas_limit)
    }

    /// Create simple fee
    pub fn simple(gas_price: Amount, gas_limit: u64) -> Self {
        let total_fee = Amount::from_wei(gas_price.value() * gas_limit);
        Self::new(gas_price, Amount::zero(), total_fee, gas_limit)
    }
}

/// Token information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Token address (None for native token)
    pub address: Option<Address>,

    /// Token symbol
    pub symbol: String,

    /// Token name
    pub name: String,

    /// Token decimals
    pub decimals: u8,

    /// Total supply (if known)
    pub total_supply: Option<Amount>,

    /// Token metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl TokenInfo {
    /// Create new token info
    pub fn new(address: Option<Address>, symbol: String, name: String, decimals: u8) -> Self {
        Self {
            address,
            symbol,
            name,
            decimals,
            total_supply: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Check if this is the native token
    pub fn is_native(&self) -> bool {
        self.address.is_none()
    }

    /// Get native token info
    pub fn native() -> Self {
        Self::new(
            None,
            crate::constants::NATIVE_TOKEN_SYMBOL.to_string(),
            "Paradigm".to_string(),
            crate::constants::NATIVE_TOKEN_DECIMALS,
        )
    }
}

/// Transaction type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction hash
    pub hash: Hash,

    /// Sender address
    pub from: Address,

    /// Recipient address (None for contract creation)
    pub to: Option<Address>,

    /// Transaction value
    pub value: Amount,

    /// Transaction data
    pub data: Vec<u8>,

    /// Transaction input (alias for data)
    pub input: Vec<u8>,

    /// Gas limit
    pub gas_limit: u64,

    /// Gas used (alias for gas_limit)
    pub gas: u64,

    /// Gas price
    pub gas_price: Amount,

    /// Nonce
    pub nonce: u64,

    /// Chain ID
    pub chain_id: u64,

    /// Signature
    pub signature: Option<Signature>,

    /// Block number (None if pending)
    pub block_number: Option<u64>,

    /// Transaction index in block
    pub transaction_index: Option<u32>,

    /// Block hash (None if pending)
    pub block_hash: Option<Hash>,

    /// Transaction status
    pub status: TransactionStatus,
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction is pending in mempool
    Pending,

    /// Transaction is included in a block
    Included,

    /// Transaction is confirmed
    Confirmed { confirmations: u32 },

    /// Transaction failed
    Failed { reason: String },

    /// Transaction was replaced
    Replaced { by: Hash },
}

impl Transaction {
    /// Create new transaction
    pub fn new(
        from: Address,
        to: Option<Address>,
        value: Amount,
        data: Vec<u8>,
        gas_limit: u64,
        gas_price: Amount,
        nonce: u64,
        chain_id: u64,
    ) -> Self {
        // Calculate transaction hash (simplified)
        let hash = Hash::from_bytes([0u8; 32]); // Would calculate actual hash

        Self {
            hash,
            from,
            to,
            value,
            data: data.clone(),
            input: data,
            gas_limit,
            gas: gas_limit,
            gas_price,
            nonce,
            chain_id,
            signature: None,
            block_number: None,
            transaction_index: None,
            block_hash: None,
            status: TransactionStatus::Pending,
        }
    }

    /// Check if transaction is contract creation
    pub fn is_contract_creation(&self) -> bool {
        self.to.is_none()
    }

    /// Check if transaction is confirmed
    pub fn is_confirmed(&self) -> bool {
        matches!(self.status, TransactionStatus::Confirmed { .. })
    }

    /// Check if transaction failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, TransactionStatus::Failed { .. })
    }

    /// Get confirmation count
    pub fn confirmations(&self) -> u32 {
        match self.status {
            TransactionStatus::Confirmed { confirmations } => confirmations,
            _ => 0,
        }
    }

    /// Calculate transaction hash
    pub fn hash(&self) -> Hash {
        self.hash.clone()
    }

    /// Convert transaction to bytes for signing/transmission
    pub fn to_bytes(&self) -> Result<Vec<u8>, crate::error::ParadigmError> {
        serde_json::to_vec(self)
            .map_err(|e| crate::error::ParadigmError::Serialization(e.to_string()))
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            hash: Hash::default(),
            from: Address::default(),
            to: None,
            value: Amount::zero(),
            data: Vec::new(),
            input: Vec::new(),
            gas_limit: 21000,
            gas: 21000,
            gas_price: Amount::from_wei(20_000_000_000),
            nonce: 0,
            chain_id: 1,
            signature: None,
            block_number: None,
            transaction_index: None,
            block_hash: None,
            status: TransactionStatus::Pending,
        }
    }
}

/// Block type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    /// Block hash
    pub hash: Hash,

    /// Parent block hash
    pub parent_hash: Hash,

    /// Block number
    pub number: u64,

    /// Block timestamp
    pub timestamp: u64,

    /// Block author/miner
    pub author: Address,

    /// Block miner (alias for author)
    pub miner: Address,

    /// Transactions in block
    pub transactions: Vec<Hash>,

    /// Transaction receipts
    pub receipts: Vec<TransactionReceipt>,

    /// State root
    pub state_root: Hash,

    /// Transactions root
    pub transactions_root: Hash,

    /// Transaction root (alias)
    pub transaction_root: Hash,

    /// Receipts root
    pub receipts_root: Hash,

    /// Gas used
    pub gas_used: u64,

    /// Gas limit
    pub gas_limit: u64,

    /// Extra data
    pub extra_data: Vec<u8>,

    /// Block difficulty
    pub difficulty: u64,

    /// Total difficulty
    pub total_difficulty: u64,

    /// Block size in bytes
    pub size: u64,

    /// Block nonce
    pub nonce: u64,
}

impl Block {
    /// Check if block is empty
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Get transaction count
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// Calculate gas utilization percentage
    pub fn gas_utilization(&self) -> f64 {
        if self.gas_limit == 0 {
            0.0
        } else {
            (self.gas_used as f64 / self.gas_limit as f64) * 100.0
        }
    }
}

/// Transaction receipt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction hash
    pub transaction_hash: Hash,

    /// Block hash
    pub block_hash: Hash,

    /// Block number
    pub block_number: u64,

    /// Transaction index
    pub transaction_index: u32,

    /// Gas used
    pub gas_used: u64,

    /// Success status
    pub success: bool,

    /// Contract address (if contract creation)
    pub contract_address: Option<Address>,

    /// Event logs
    pub logs: Vec<Log>,

    /// Error message (if failed)
    pub error: Option<String>,
}

/// Event log
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Log {
    /// Contract address that emitted the log
    pub address: Address,

    /// Log topics
    pub topics: Vec<Hash>,

    /// Log data
    pub data: Vec<u8>,

    /// Block number
    pub block_number: u64,

    /// Transaction hash
    pub transaction_hash: Hash,

    /// Log index
    pub log_index: u32,
}

/// Network information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Chain ID
    pub chain_id: u64,

    /// Network name
    pub name: String,

    /// Native currency
    pub native_currency: TokenInfo,

    /// RPC URLs
    pub rpc_urls: Vec<String>,

    /// Block explorer URLs
    pub explorer_urls: Vec<String>,

    /// Network type
    pub network_type: NetworkType,
}

/// Network type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Devnet,
    Private,
}

/// Key pair for cryptographic operations
#[derive(Debug, Clone)]
pub struct KeyPair {
    /// Public key
    pub public_key: Vec<u8>,

    /// Private key (should be handled securely)
    pub private_key: Vec<u8>,

    /// Key type
    pub key_type: KeyType,
}

/// Key type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    Ed25519,
    Secp256k1,
    BLS,
}

impl KeyPair {
    /// Create new key pair
    pub fn new(public_key: Vec<u8>, private_key: Vec<u8>, key_type: KeyType) -> Self {
        Self {
            public_key,
            private_key,
            key_type,
        }
    }

    /// Generate random key pair
    pub fn generate(key_type: KeyType) -> Result<Self> {
        match key_type {
            KeyType::Ed25519 => {
                use ed25519_dalek::{SigningKey, VerifyingKey};
                use rand::rngs::OsRng;

                let signing_key = SigningKey::generate(&mut OsRng);
                let verifying_key = VerifyingKey::from(&signing_key);

                Ok(Self::new(
                    verifying_key.as_bytes().to_vec(),
                    signing_key.as_bytes().to_vec(),
                    KeyType::Ed25519,
                ))
            }
            _ => Err(ParadigmError::UnsupportedKeyType(format!("{:?}", key_type))),
        }
    }

    /// Get address from public key
    pub fn address(&self) -> Result<Address> {
        // Simplified address derivation
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&self.public_key);
        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&hash[12..32]);
        Ok(Address::from_bytes(addr_bytes))
    }

    /// Sign data
    pub fn sign(&self, data: &[u8]) -> Result<Signature> {
        match self.key_type {
            KeyType::Ed25519 => {
                use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey};

                let signing_key =
                    SigningKey::from_bytes(&self.private_key.clone().try_into().map_err(|_| {
                        ParadigmError::InvalidKey("Invalid private key length".to_string())
                    })?);

                let signature = signing_key.sign(data);
                Ok(Signature::new(
                    signature.to_bytes().to_vec(),
                    SignatureType::Ed25519,
                ))
            }
            _ => Err(ParadigmError::UnsupportedKeyType(format!(
                "{:?}",
                self.key_type
            ))),
        }
    }

    /// Verify signature
    pub fn verify(&self, data: &[u8], signature: &Signature) -> Result<bool> {
        match (self.key_type.clone(), signature.signature_type()) {
            (KeyType::Ed25519, SignatureType::Ed25519) => {
                use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};

                let verifying_key =
                    VerifyingKey::from_bytes(&self.public_key.clone().try_into().map_err(
                        |_| ParadigmError::InvalidKey("Invalid public key length".to_string()),
                    )?)
                    .map_err(|e| ParadigmError::InvalidKey(e.to_string()))?;

                let sig = Ed25519Signature::from_bytes(&signature.as_bytes().try_into().map_err(
                    |_| ParadigmError::InvalidSignature("Invalid signature length".to_string()),
                )?);

                Ok(verifying_key.verify(data, &sig).is_ok())
            }
            _ => Err(ParadigmError::UnsupportedKeyType(
                "Mismatched key and signature types".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address() {
        let addr_str = "0x1234567890123456789012345678901234567890";
        let addr = Address::from_hex(addr_str).unwrap();
        assert_eq!(addr.to_hex(), addr_str);
        assert!(!addr.is_zero());

        let zero_addr = Address::zero();
        assert!(zero_addr.is_zero());
    }

    #[test]
    fn test_hash() {
        let hash_str = "0x1234567890123456789012345678901234567890123456789012345678901234";
        let hash = Hash::from_hex(hash_str).unwrap();
        assert_eq!(hash.to_hex(), hash_str);
        assert!(!hash.is_zero());
    }

    #[test]
    fn test_amount() {
        let amount = Amount::from_paradigm(1.5);
        assert_eq!(amount.to_paradigm(), 1.5);

        let wei_amount = Amount::from_wei(1000);
        assert_eq!(wei_amount.value(), 1000);

        // Test arithmetic
        let sum = amount + wei_amount;
        assert_eq!(sum.value(), amount.value() + wei_amount.value());
    }

    #[test]
    fn test_balance() {
        let balance = Balance::new(
            Amount::from_paradigm(10.0),
            Amount::from_paradigm(5.0),
            Amount::from_paradigm(1.0),
        );

        assert_eq!(balance.total().to_paradigm(), 16.0);
        assert!(!balance.is_zero());
    }

    #[test]
    fn test_keypair() {
        let keypair = KeyPair::generate(KeyType::Ed25519).unwrap();
        let address = keypair.address().unwrap();

        let data = b"test message";
        let signature = keypair.sign(data).unwrap();
        assert!(keypair.verify(data, &signature).unwrap());

        // Wrong data should fail verification
        let wrong_data = b"wrong message";
        assert!(!keypair.verify(wrong_data, &signature).unwrap());
    }

    #[test]
    fn test_transaction() {
        let from = Address::from_hex("0x1234567890123456789012345678901234567890").unwrap();
        let to = Address::from_hex("0x0987654321098765432109876543210987654321").unwrap();

        let tx = Transaction::new(
            from,
            Some(to),
            Amount::from_paradigm(1.0),
            vec![],
            21000,
            Amount::from_wei(20_000_000_000), // 20 Gwei
            0,
            1,
        );

        assert!(!tx.is_contract_creation());
        assert!(!tx.is_confirmed());
        assert_eq!(tx.confirmations(), 0);
    }
}
