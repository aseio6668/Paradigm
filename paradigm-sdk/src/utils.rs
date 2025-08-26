//! Utility functions and helpers
//!
//! This module provides various utility functions, formatters, validators,
//! and helper types used throughout the Paradigm SDK.

use crate::error::{ParadigmError, Result};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Formatting utilities
pub mod format {
    use super::*;

    /// Format amount with proper decimal places
    pub fn format_amount(amount: &Amount) -> String {
        let paradigm_value = amount.to_paradigm();
        if paradigm_value.fract() == 0.0 {
            format!("{:.0} PARADIGM", paradigm_value)
        } else {
            format!("{:.6} PARADIGM", paradigm_value)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
                + " PARADIGM"
        }
    }

    /// Format amount in wei
    pub fn format_wei(amount: &Amount) -> String {
        format!("{} wei", amount.wei())
    }

    /// Format hash with checksum
    pub fn format_hash(hash: &Hash) -> String {
        hash.to_string()
    }

    /// Format address with checksum
    pub fn format_address(address: &Address) -> String {
        address.to_checksum()
    }

    /// Format duration in human-readable form
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();

        if total_seconds < 60 {
            format!("{}s", total_seconds)
        } else if total_seconds < 3600 {
            let minutes = total_seconds / 60;
            let seconds = total_seconds % 60;
            if seconds == 0 {
                format!("{}m", minutes)
            } else {
                format!("{}m {}s", minutes, seconds)
            }
        } else if total_seconds < 86400 {
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            if minutes == 0 {
                format!("{}h", hours)
            } else {
                format!("{}h {}m", hours, minutes)
            }
        } else {
            let days = total_seconds / 86400;
            let hours = (total_seconds % 86400) / 3600;
            if hours == 0 {
                format!("{}d", days)
            } else {
                format!("{}d {}h", days, hours)
            }
        }
    }

    /// Format bytes in human-readable form
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }

    /// Format gas price in gwei
    pub fn format_gas_price(gas_price: &Amount) -> String {
        let gwei = gas_price.wei() as f64 / 1_000_000_000.0;
        format!("{:.2} gwei", gwei)
    }

    /// Format timestamp as human-readable date
    pub fn format_timestamp(timestamp: u64) -> String {
        let system_time = UNIX_EPOCH + Duration::from_secs(timestamp);

        // Simple formatting - in production, would use chrono or similar
        match system_time.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let seconds = duration.as_secs();
                let days = seconds / 86400;
                let hours = (seconds % 86400) / 3600;
                let minutes = (seconds % 3600) / 60;
                let secs = seconds % 60;

                format!("{} days, {:02}:{:02}:{:02}", days, hours, minutes, secs)
            }
            Err(_) => "Invalid timestamp".to_string(),
        }
    }

    /// Format percentage
    pub fn format_percentage(value: f64) -> String {
        format!("{:.2}%", value * 100.0)
    }
}

/// Validation utilities
pub mod validate {
    use super::*;

    /// Validate Paradigm address format
    pub fn address(address: &str) -> Result<()> {
        if !address.starts_with("0x") {
            return Err(ParadigmError::InvalidAddress(
                "Address must start with 0x".to_string(),
            ));
        }

        if address.len() != 42 {
            return Err(ParadigmError::InvalidAddress(
                "Address must be 42 characters long".to_string(),
            ));
        }

        if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParadigmError::InvalidAddress(
                "Address contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate hash format
    pub fn hash(hash: &str) -> Result<()> {
        if !hash.starts_with("0x") {
            return Err(ParadigmError::InvalidHash(
                "Hash must start with 0x".to_string(),
            ));
        }

        if hash.len() != 66 {
            return Err(ParadigmError::InvalidHash(
                "Hash must be 66 characters long".to_string(),
            ));
        }

        if !hash[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParadigmError::InvalidHash(
                "Hash contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate private key format
    pub fn private_key(key: &str) -> Result<()> {
        let key = if key.starts_with("0x") {
            &key[2..]
        } else {
            key
        };

        if key.len() != 64 {
            return Err(ParadigmError::InvalidKey(
                "Private key must be 32 bytes (64 hex characters)".to_string(),
            ));
        }

        if !key.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParadigmError::InvalidKey(
                "Private key contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate mnemonic phrase
    pub fn mnemonic(mnemonic: &str) -> Result<()> {
        let words: Vec<&str> = mnemonic.split_whitespace().collect();

        if ![12, 15, 18, 21, 24].contains(&words.len()) {
            return Err(ParadigmError::InvalidKey(
                "Mnemonic must have 12, 15, 18, 21, or 24 words".to_string(),
            ));
        }

        // Basic word validation - in production, would check against BIP39 wordlist
        for word in words {
            if word.is_empty() || !word.chars().all(|c| c.is_ascii_alphabetic()) {
                return Err(ParadigmError::InvalidKey(
                    "Mnemonic contains invalid words".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate amount format
    pub fn amount(amount_str: &str) -> Result<()> {
        if amount_str.is_empty() {
            return Err(ParadigmError::InvalidAmount(
                "Amount cannot be empty".to_string(),
            ));
        }

        if amount_str.parse::<f64>().is_err() {
            return Err(ParadigmError::InvalidAmount(
                "Invalid amount format".to_string(),
            ));
        }

        let value: f64 = amount_str.parse().unwrap();
        if value < 0.0 {
            return Err(ParadigmError::InvalidAmount(
                "Amount cannot be negative".to_string(),
            ));
        }

        if value > 1e18 {
            return Err(ParadigmError::InvalidAmount("Amount too large".to_string()));
        }

        Ok(())
    }

    /// Validate URL format
    pub fn url(url_str: &str) -> Result<()> {
        url::Url::parse(url_str)
            .map_err(|e| ParadigmError::Config(format!("Invalid URL: {}", e)))?;
        Ok(())
    }

    /// Validate chain ID
    pub fn chain_id(chain_id: u64) -> Result<()> {
        if chain_id == 0 {
            return Err(ParadigmError::Config("Chain ID cannot be zero".to_string()));
        }

        if chain_id > u32::MAX as u64 {
            return Err(ParadigmError::Config("Chain ID too large".to_string()));
        }

        Ok(())
    }
}

/// Conversion utilities
pub mod convert {
    use super::*;

    /// Convert hex string to bytes
    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
        let hex = if hex.starts_with("0x") {
            &hex[2..]
        } else {
            hex
        };

        hex::decode(hex)
            .map_err(|e| ParadigmError::InvalidHex(format!("Invalid hex string: {}", e)))
    }

    /// Convert bytes to hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        format!("0x{}", hex::encode(bytes))
    }

    /// Convert wei to paradigm
    pub fn wei_to_paradigm(wei: u64) -> f64 {
        wei as f64 / 1e18
    }

    /// Convert paradigm to wei
    pub fn paradigm_to_wei(paradigm: f64) -> u64 {
        (paradigm * 1e18) as u64
    }

    /// Convert gwei to wei
    pub fn gwei_to_wei(gwei: f64) -> u64 {
        (gwei * 1e9) as u64
    }

    /// Convert wei to gwei
    pub fn wei_to_gwei(wei: u64) -> f64 {
        wei as f64 / 1e9
    }

    /// Convert timestamp to SystemTime
    pub fn timestamp_to_system_time(timestamp: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(timestamp)
    }

    /// Convert SystemTime to timestamp
    pub fn system_time_to_timestamp(time: SystemTime) -> u64 {
        time.duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Convert string to fixed-size byte array
    pub fn string_to_bytes32(input: &str) -> [u8; 32] {
        let mut result = [0u8; 32];
        let bytes = input.as_bytes();
        let copy_len = bytes.len().min(32);
        result[..copy_len].copy_from_slice(&bytes[..copy_len]);
        result
    }

    /// Convert fixed-size byte array to string
    pub fn bytes32_to_string(bytes: &[u8; 32]) -> String {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(32);
        String::from_utf8_lossy(&bytes[..end]).to_string()
    }
}

/// Cryptographic utilities
pub mod crypto {
    use super::*;
    use sha2::{Digest, Sha256};

    /// Calculate keccak256 hash (simplified version using SHA256)
    pub fn keccak256(input: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        Hash::from_bytes(&result).unwrap()
    }

    /// Calculate double SHA256 hash
    pub fn double_sha256(input: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(input);
        let first_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(first_hash);
        let result = hasher.finalize();

        Hash::from_bytes(&result).unwrap()
    }

    /// Generate random bytes
    pub fn random_bytes(length: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut bytes = vec![0u8; length];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        bytes
    }

    /// Generate random hash
    pub fn random_hash() -> Hash {
        Hash::from_bytes(&random_bytes(32)).unwrap()
    }

    /// Generate random address
    pub fn random_address() -> Address {
        Address::from_bytes(random_bytes(20).try_into().unwrap())
    }

    /// Verify ECDSA signature (simplified)
    pub fn verify_signature(message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
        // Simplified verification - real implementation would use proper ECDSA
        signature.len() == 64 && public_key.len() == 33 && !message.is_empty()
    }

    /// Recover address from signature (simplified)
    pub fn recover_address(message_hash: &Hash, signature: &[u8]) -> Result<Address> {
        if signature.len() != 65 {
            return Err(ParadigmError::InvalidSignature(
                "Invalid signature length".to_string(),
            ));
        }

        // Simplified recovery - real implementation would use secp256k1
        let mut addr_bytes = [0u8; 20];
        addr_bytes[..20].copy_from_slice(&message_hash.bytes[..20]);
        Ok(Address::from_bytes(addr_bytes.try_into().map_err(|_| Error::InvalidAddress("Invalid address length".to_string()))?)?)
    }
}

/// Math utilities
pub mod math {
    use super::*;

    /// Calculate percentage
    pub fn percentage(part: u64, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (part as f64 / total as f64) * 100.0
        }
    }

    /// Calculate average
    pub fn average(values: &[u64]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<u64>() as f64 / values.len() as f64
        }
    }

    /// Calculate median
    pub fn median(values: &[u64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort();

        let len = sorted.len();
        if len % 2 == 0 {
            (sorted[len / 2 - 1] + sorted[len / 2]) as f64 / 2.0
        } else {
            sorted[len / 2] as f64
        }
    }

    /// Calculate standard deviation
    pub fn standard_deviation(values: &[u64]) -> f64 {
        if values.len() <= 1 {
            return 0.0;
        }

        let mean = average(values);
        let variance = values
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / (values.len() - 1) as f64;

        variance.sqrt()
    }

    /// Calculate compound interest
    pub fn compound_interest(principal: f64, rate: f64, time: f64, compounds_per_year: f64) -> f64 {
        principal * (1.0 + rate / compounds_per_year).powf(compounds_per_year * time)
    }

    /// Calculate simple interest
    pub fn simple_interest(principal: f64, rate: f64, time: f64) -> f64 {
        principal * (1.0 + rate * time)
    }

    /// Safe addition that prevents overflow
    pub fn safe_add(a: u64, b: u64) -> Result<u64> {
        a.checked_add(b)
            .ok_or_else(|| ParadigmError::Generic("Arithmetic overflow in addition".to_string()))
    }

    /// Safe subtraction that prevents underflow
    pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
        a.checked_sub(b).ok_or_else(|| {
            ParadigmError::Generic("Arithmetic underflow in subtraction".to_string())
        })
    }

    /// Safe multiplication that prevents overflow
    pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
        a.checked_mul(b).ok_or_else(|| {
            ParadigmError::Generic("Arithmetic overflow in multiplication".to_string())
        })
    }

    /// Safe division that handles division by zero
    pub fn safe_div(a: u64, b: u64) -> Result<u64> {
        if b == 0 {
            return Err(ParadigmError::Generic("Division by zero".to_string()));
        }
        Ok(a / b)
    }
}

/// Time utilities
pub mod time {
    use super::*;

    /// Get current timestamp
    pub fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get current timestamp in milliseconds
    pub fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Check if timestamp is in the past
    pub fn is_past(timestamp: u64) -> bool {
        timestamp < now()
    }

    /// Check if timestamp is in the future
    pub fn is_future(timestamp: u64) -> bool {
        timestamp > now()
    }

    /// Calculate time difference in seconds
    pub fn time_diff(timestamp1: u64, timestamp2: u64) -> u64 {
        timestamp1.abs_diff(timestamp2)
    }

    /// Add duration to timestamp
    pub fn add_duration(timestamp: u64, duration: Duration) -> u64 {
        timestamp + duration.as_secs()
    }

    /// Subtract duration from timestamp
    pub fn sub_duration(timestamp: u64, duration: Duration) -> u64 {
        timestamp.saturating_sub(duration.as_secs())
    }

    /// Get start of day timestamp
    pub fn start_of_day(timestamp: u64) -> u64 {
        let seconds_in_day = 86400;
        (timestamp / seconds_in_day) * seconds_in_day
    }

    /// Get end of day timestamp
    pub fn end_of_day(timestamp: u64) -> u64 {
        start_of_day(timestamp) + 86399
    }
}

/// Debug and testing utilities
pub mod debug {
    use super::*;

    /// Create debug transaction
    pub fn create_debug_transaction() -> Transaction {
        Transaction {
            hash: crypto::random_hash(),
            from: crypto::random_address(),
            to: Some(crypto::random_address()),
            value: Amount::from_paradigm(100),
            gas: 21000,
            gas_price: Amount::from_wei(20_000_000_000),
            nonce: 1,
            input: vec![],
            signature: Some(vec![0; 65]),
            block_hash: Some(crypto::random_hash()),
            block_number: 1000,
            transaction_index: Some(0),
        }
    }

    /// Create debug block
    pub fn create_debug_block() -> Block {
        Block {
            hash: crypto::random_hash(),
            parent_hash: crypto::random_hash(),
            number: 1000,
            timestamp: time::now(),
            gas_limit: 15_000_000,
            gas_used: 10_000_000,
            miner: crypto::random_address(),
            transactions: vec![create_debug_transaction()],
            state_root: crypto::random_hash(),
            transaction_root: crypto::random_hash(),
            receipts_root: crypto::random_hash(),
            difficulty: 1000000,
            total_difficulty: 1000000000,
            size: 1024,
            extra_data: vec![],
            nonce: 12345,
        }
    }

    /// Create debug account
    pub fn create_debug_account() -> (Address, Amount) {
        (crypto::random_address(), Amount::from_paradigm(1000))
    }

    /// Pretty print transaction
    pub fn pretty_print_transaction(tx: &Transaction) -> String {
        format!(
            "Transaction {{\n  Hash: {}\n  From: {}\n  To: {}\n  Value: {}\n  Gas: {}\n  Gas Price: {}\n  Nonce: {}\n}}",
            format::format_hash(&tx.hash),
            format::format_address(&tx.from),
            tx.to.as_ref().map_or("None".to_string(), |addr| format::format_address(addr)),
            format::format_amount(&tx.value),
            tx.gas,
            format::format_gas_price(&tx.gas_price),
            tx.nonce
        )
    }

    /// Pretty print block
    pub fn pretty_print_block(block: &Block) -> String {
        format!(
            "Block {{\n  Hash: {}\n  Number: {}\n  Timestamp: {}\n  Gas Used: {} / {}\n  Transactions: {}\n  Miner: {}\n}}",
            format::format_hash(&block.hash),
            block.number,
            format::format_timestamp(block.timestamp),
            block.gas_used,
            block.gas_limit,
            block.transactions.len(),
            format::format_address(&block.miner)
        )
    }
}

/// Configuration utilities
pub mod config {
    use super::*;
    use std::path::Path;

    /// Load configuration from file
    pub async fn load_from_file<T>(path: &Path) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ParadigmError::Io(e))?;

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)
                .map_err(|e| ParadigmError::Config(format!("Invalid JSON config: {}", e)))
        } else {
            // Assume TOML format
            toml::from_str(&content)
                .map_err(|e| ParadigmError::Config(format!("Invalid TOML config: {}", e)))
        }
    }

    /// Save configuration to file
    pub async fn save_to_file<T>(config: &T, path: &Path) -> Result<()>
    where
        T: Serialize,
    {
        let content = if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::to_string_pretty(config)
                .map_err(|e| ParadigmError::Config(format!("Failed to serialize JSON: {}", e)))?
        } else {
            // Assume TOML format
            toml::to_string_pretty(config)
                .map_err(|e| ParadigmError::Config(format!("Failed to serialize TOML: {}", e)))?
        };

        tokio::fs::write(path, content)
            .await
            .map_err(|e| ParadigmError::Io(e))?;

        Ok(())
    }

    /// Get default config directory
    pub fn default_config_dir() -> Result<std::path::PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| ParadigmError::Config("Could not find home directory".to_string()))?;

        Ok(home.join(".paradigm"))
    }

    /// Ensure config directory exists
    pub async fn ensure_config_dir(path: &Path) -> Result<()> {
        if !path.exists() {
            tokio::fs::create_dir_all(path)
                .await
                .map_err(|e| ParadigmError::Io(e))?;
        }
        Ok(())
    }
}

/// Rate limiting utilities
pub mod rate_limit {
    use super::*;
    use std::collections::VecDeque;

    /// Token bucket rate limiter
    #[derive(Debug)]
    pub struct TokenBucket {
        capacity: u32,
        tokens: u32,
        refill_rate: u32, // tokens per second
        last_refill: SystemTime,
    }

    impl TokenBucket {
        pub fn new(capacity: u32, refill_rate: u32) -> Self {
            Self {
                capacity,
                tokens: capacity,
                refill_rate,
                last_refill: SystemTime::now(),
            }
        }

        pub fn consume(&mut self, tokens: u32) -> bool {
            self.refill();

            if self.tokens >= tokens {
                self.tokens -= tokens;
                true
            } else {
                false
            }
        }

        fn refill(&mut self) {
            let now = SystemTime::now();
            if let Ok(elapsed) = now.duration_since(self.last_refill) {
                let tokens_to_add =
                    (elapsed.as_secs() as u32 * self.refill_rate).min(self.capacity - self.tokens);
                self.tokens += tokens_to_add;
                self.last_refill = now;
            }
        }

        pub fn available_tokens(&mut self) -> u32 {
            self.refill();
            self.tokens
        }
    }

    /// Sliding window rate limiter
    #[derive(Debug)]
    pub struct SlidingWindow {
        window_size: Duration,
        max_requests: u32,
        requests: VecDeque<SystemTime>,
    }

    impl SlidingWindow {
        pub fn new(window_size: Duration, max_requests: u32) -> Self {
            Self {
                window_size,
                max_requests,
                requests: VecDeque::new(),
            }
        }

        pub fn allow_request(&mut self) -> bool {
            self.cleanup_old_requests();

            if self.requests.len() < self.max_requests as usize {
                self.requests.push_back(SystemTime::now());
                true
            } else {
                false
            }
        }

        fn cleanup_old_requests(&mut self) {
            let cutoff = SystemTime::now() - self.window_size;
            while let Some(&front) = self.requests.front() {
                if front < cutoff {
                    self.requests.pop_front();
                } else {
                    break;
                }
            }
        }

        pub fn current_requests(&mut self) -> usize {
            self.cleanup_old_requests();
            self.requests.len()
        }
    }
}

/// Cache utilities
pub mod cache {
    use super::*;
    use std::collections::HashMap;

    /// Simple LRU cache
    #[derive(Debug)]
    pub struct LruCache<K, V> {
        capacity: usize,
        map: HashMap<K, V>,
        order: VecDeque<K>,
    }

    impl<K: Clone + Eq + std::hash::Hash, V> LruCache<K, V> {
        pub fn new(capacity: usize) -> Self {
            Self {
                capacity,
                map: HashMap::new(),
                order: VecDeque::new(),
            }
        }

        pub fn get(&mut self, key: &K) -> Option<&V> {
            if self.map.contains_key(key) {
                // Move to front
                self.order.retain(|k| k != key);
                self.order.push_front(key.clone());
                self.map.get(key)
            } else {
                None
            }
        }

        pub fn put(&mut self, key: K, value: V) {
            if self.map.contains_key(&key) {
                // Update existing
                self.map.insert(key.clone(), value);
                self.order.retain(|k| k != &key);
                self.order.push_front(key);
            } else {
                // Add new
                if self.map.len() >= self.capacity {
                    // Remove LRU
                    if let Some(lru_key) = self.order.pop_back() {
                        self.map.remove(&lru_key);
                    }
                }
                self.map.insert(key.clone(), value);
                self.order.push_front(key);
            }
        }

        pub fn len(&self) -> usize {
            self.map.len()
        }

        pub fn is_empty(&self) -> bool {
            self.map.is_empty()
        }

        pub fn clear(&mut self) {
            self.map.clear();
            self.order.clear();
        }
    }

    /// TTL cache with expiration
    #[derive(Debug)]
    pub struct TtlCache<K, V> {
        map: HashMap<K, (V, SystemTime)>,
        ttl: Duration,
    }

    impl<K: Clone + Eq + std::hash::Hash, V> TtlCache<K, V> {
        pub fn new(ttl: Duration) -> Self {
            Self {
                map: HashMap::new(),
                ttl,
            }
        }

        pub fn get(&mut self, key: &K) -> Option<&V> {
            self.cleanup_expired();
            self.map.get(key).map(|(value, _)| value)
        }

        pub fn put(&mut self, key: K, value: V) {
            let expires_at = SystemTime::now() + self.ttl;
            self.map.insert(key, (value, expires_at));
        }

        pub fn cleanup_expired(&mut self) {
            let now = SystemTime::now();
            self.map.retain(|_, (_, expires_at)| *expires_at > now);
        }

        pub fn len(&self) -> usize {
            self.map.len()
        }

        pub fn clear(&mut self) {
            self.map.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_amount() {
        let amount = Amount::from_paradigm(123);
        let formatted = format::format_amount(&amount);
        assert!(formatted.contains("123"));
        assert!(formatted.contains("PARADIGM"));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format::format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format::format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format::format_duration(Duration::from_secs(3661)), "1h 1m");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format::format_bytes(512), "512 B");
        assert_eq!(format::format_bytes(1536), "1.50 KB");
        assert_eq!(format::format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_validate_address() {
        assert!(validate::address("0x742d35Cc6634C0532925a3b8D5C9C1D2").is_err());
        assert!(validate::address("0x742d35Cc6634C0532925a3b8D5C9C1D26d28b9fF").is_ok());
        assert!(validate::address("742d35Cc6634C0532925a3b8D5C9C1D26d28b9fF").is_err());
    }

    #[test]
    fn test_validate_hash() {
        assert!(validate::hash("0xabcd").is_err());
        assert!(validate::hash(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        )
        .is_ok());
    }

    #[test]
    fn test_hex_conversion() {
        let bytes = vec![0x12, 0x34, 0x56, 0x78];
        let hex = convert::bytes_to_hex(&bytes);
        assert_eq!(hex, "0x12345678");

        let decoded = convert::hex_to_bytes(&hex).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_wei_conversion() {
        assert_eq!(convert::wei_to_paradigm(1_000_000_000_000_000_000), 1.0);
        assert_eq!(convert::paradigm_to_wei(1.5), 1_500_000_000_000_000_000);
    }

    #[test]
    fn test_math_functions() {
        assert_eq!(math::percentage(25, 100), 25.0);
        assert_eq!(math::average(&[1, 2, 3, 4, 5]), 3.0);
        assert_eq!(math::median(&[1, 2, 3, 4, 5]), 3.0);
        assert_eq!(math::median(&[1, 2, 3, 4]), 2.5);
    }

    #[test]
    fn test_safe_math() {
        assert!(math::safe_add(u64::MAX, 1).is_err());
        assert!(math::safe_sub(0, 1).is_err());
        assert!(math::safe_div(10, 0).is_err());
        assert_eq!(math::safe_add(5, 3).unwrap(), 8);
        assert_eq!(math::safe_sub(10, 3).unwrap(), 7);
        assert_eq!(math::safe_div(10, 2).unwrap(), 5);
    }

    #[test]
    fn test_token_bucket() {
        let mut bucket = rate_limit::TokenBucket::new(10, 1);
        assert!(bucket.consume(5));
        assert!(bucket.consume(5));
        assert!(!bucket.consume(1)); // Should fail, no tokens left
    }

    #[test]
    fn test_lru_cache() {
        let mut cache = cache::LruCache::new(2);
        cache.put("key1", "value1");
        cache.put("key2", "value2");
        cache.put("key3", "value3"); // Should evict key1

        assert!(cache.get(&"key1").is_none());
        assert!(cache.get(&"key2").is_some());
        assert!(cache.get(&"key3").is_some());
    }
}
