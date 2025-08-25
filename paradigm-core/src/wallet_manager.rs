use crate::{Address, AddressExt};
use anyhow::Result;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEntry {
    pub address: String,
    pub private_key_hex: String,
    pub label: String,
    pub balance: u64,
    pub total_earned: u64,
    pub tasks_completed: u64,
    pub created_at: u64,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletFile {
    pub version: String,
    pub created_at: u64,
    pub last_used: u64,
    pub default_address: Option<String>,
    pub addresses: HashMap<String, WalletEntry>,
}

impl WalletFile {
    pub fn new() -> Self {
        let now = chrono::Utc::now().timestamp() as u64;
        Self {
            version: "1.0.0".to_string(),
            created_at: now,
            last_used: now,
            default_address: None,
            addresses: HashMap::new(),
        }
    }

    pub fn load_or_create(path: &PathBuf) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let mut wallet: WalletFile = serde_json::from_str(&content)?;
            wallet.last_used = chrono::Utc::now().timestamp() as u64;
            wallet.save(path)?;
            info!("📂 Loaded wallet file: {} ({} addresses)", path.display(), wallet.addresses.len());
            Ok(wallet)
        } else {
            let wallet = Self::new();
            wallet.save(path)?;
            info!("🔑 Created new wallet file: {}", path.display());
            Ok(wallet)
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_address(&mut self, label: &str) -> Result<String> {
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let address = Address::from_public_key(&signing_key.verifying_key());
        let address_str = address.to_string();
        let private_key_hex = hex::encode(signing_key.to_bytes());
        let now = chrono::Utc::now().timestamp() as u64;

        let entry = WalletEntry {
            address: address_str.clone(),
            private_key_hex,
            label: label.to_string(),
            balance: 0,
            total_earned: 0,
            tasks_completed: 0,
            created_at: now,
            last_updated: now,
        };

        self.addresses.insert(address_str.clone(), entry);
        
        // Set as default if it's the first address
        if self.default_address.is_none() {
            self.default_address = Some(address_str.clone());
        }

        info!("➕ Added new address: {} ({})", address_str, label);
        Ok(address_str)
    }

    pub fn get_address(&self, address: &str) -> Option<&WalletEntry> {
        self.addresses.get(address)
    }

    pub fn get_default_address(&self) -> Option<&WalletEntry> {
        if let Some(default_addr) = &self.default_address {
            self.addresses.get(default_addr)
        } else {
            None
        }
    }

    pub fn set_default_address(&mut self, address: &str) -> Result<()> {
        if self.addresses.contains_key(address) {
            self.default_address = Some(address.to_string());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Address not found in wallet"))
        }
    }

    pub fn update_balance(&mut self, address: &str, amount: u64) -> Result<()> {
        if let Some(entry) = self.addresses.get_mut(address) {
            entry.balance += amount;
            entry.total_earned += amount;
            entry.tasks_completed += 1;
            entry.last_updated = chrono::Utc::now().timestamp() as u64;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Address not found in wallet"))
        }
    }

    pub fn list_addresses(&self) -> Vec<(&String, &WalletEntry)> {
        self.addresses.iter().collect()
    }

    pub fn export_private_key(&self, address: &str) -> Result<String> {
        if let Some(entry) = self.addresses.get(address) {
            Ok(entry.private_key_hex.clone())
        } else {
            Err(anyhow::anyhow!("Address not found"))
        }
    }

    pub fn export_all_keys(&self) -> Vec<(String, String, String)> {
        self.addresses.iter()
            .map(|(addr, entry)| (addr.clone(), entry.private_key_hex.clone(), entry.label.clone()))
            .collect()
    }

    pub fn import_private_key(&mut self, private_key_hex: &str, label: &str) -> Result<String> {
        let private_key_bytes = hex::decode(private_key_hex)?;
        if private_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid private key length"));
        }
        
        let private_key_array: [u8; 32] = private_key_bytes.try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert private key"))?;
        
        let signing_key = SigningKey::from_bytes(&private_key_array);
        let address = Address::from_public_key(&signing_key.verifying_key());
        let address_str = address.to_string();
        let now = chrono::Utc::now().timestamp() as u64;

        let entry = WalletEntry {
            address: address_str.clone(),
            private_key_hex: private_key_hex.to_string(),
            label: label.to_string(),
            balance: 0,
            total_earned: 0,
            tasks_completed: 0,
            created_at: now,
            last_updated: now,
        };

        self.addresses.insert(address_str.clone(), entry);
        
        info!("📥 Imported address: {} ({})", address_str, label);
        Ok(address_str)
    }
}

pub struct WalletManager {
    wallet_file: WalletFile,
    wallet_path: PathBuf,
}

impl WalletManager {
    pub fn new(wallet_path: PathBuf) -> Result<Self> {
        let wallet_file = WalletFile::load_or_create(&wallet_path)?;
        Ok(Self {
            wallet_file,
            wallet_path,
        })
    }

    pub fn get_or_create_address(&mut self, address_hint: Option<String>, label: &str) -> Result<String> {
        match address_hint {
            Some(addr) => {
                // Verify the address exists in our wallet
                if self.wallet_file.get_address(&addr).is_some() {
                    info!("🔑 Using existing wallet address: {}", addr);
                    Ok(addr)
                } else {
                    warn!("⚠️ Specified address {} not found in wallet, creating new one", addr);
                    let new_addr = self.wallet_file.add_address(label)?;
                    self.save()?;
                    Ok(new_addr)
                }
            }
            None => {
                // Check if we have a default address
                if let Some(default_entry) = self.wallet_file.get_default_address() {
                    info!("🔑 Using default wallet address: {}", default_entry.address);
                    Ok(default_entry.address.clone())
                } else {
                    // Create a new address
                    let new_addr = self.wallet_file.add_address(label)?;
                    self.save()?;
                    Ok(new_addr)
                }
            }
        }
    }

    pub fn update_address_balance(&mut self, address: &str, amount: u64) -> Result<()> {
        self.wallet_file.update_balance(address, amount)?;
        self.save()
    }

    pub fn get_address_info(&self, address: &str) -> Option<&WalletEntry> {
        self.wallet_file.get_address(address)
    }

    pub fn list_addresses(&self) -> Vec<(&String, &WalletEntry)> {
        self.wallet_file.list_addresses()
    }

    pub fn export_keys(&self) -> Vec<(String, String, String)> {
        self.wallet_file.export_all_keys()
    }

    pub fn import_key(&mut self, private_key_hex: &str, label: &str) -> Result<String> {
        let address = self.wallet_file.import_private_key(private_key_hex, label)?;
        self.save()?;
        Ok(address)
    }

    pub fn save(&self) -> Result<()> {
        self.wallet_file.save(&self.wallet_path)
    }

    pub fn get_default_wallet_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("paradigm")
            .join("wallet.json")
    }

    pub fn add_address(&mut self, label: &str) -> Result<String> {
        let address = self.wallet_file.add_address(label)?;
        self.save()?;
        Ok(address)
    }

    pub fn get_default_address(&self) -> Option<String> {
        self.wallet_file.default_address.clone()
    }

    pub fn import_private_key(&mut self, private_key: &str, label: &str) -> Result<String> {
        let address = self.wallet_file.import_private_key(private_key, label)?;
        self.save()?;
        Ok(address)
    }

    pub fn export_private_key(&self, address: &str) -> Result<String> {
        self.wallet_file.export_private_key(address)
    }
}