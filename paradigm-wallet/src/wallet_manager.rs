use paradigm_core::{wallet::Wallet, Address, transaction::Transaction};
use anyhow::Result;
use std::path::PathBuf;
use tokio::sync::RwLock;

use crate::ui::TransactionDisplay;

/// Manages wallet operations for the GUI application
pub struct WalletManager {
    wallet: Option<Wallet>,
    data_dir: PathBuf,
}

impl WalletManager {
    pub async fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("paradigm-wallet");
        
        // Ensure directory exists
        std::fs::create_dir_all(&data_dir)?;

        Ok(WalletManager {
            wallet: None,
            data_dir,
        })
    }

    pub fn create_new_wallet(&mut self) -> Result<()> {
        let mut wallet = Wallet::new();
        
        // Initialize storage
        let db_path = self.data_dir.join("wallet.db");
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(wallet.initialize_storage(&db_path))?;
        
        self.wallet = Some(wallet);
        Ok(())
    }

    pub fn import_from_seed_phrase(&mut self, words: &[String]) -> Result<()> {
        let mut wallet = Wallet::from_seed_phrase(words)?;
        
        // Initialize storage
        let db_path = self.data_dir.join("wallet.db");
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(wallet.initialize_storage(&db_path))?;
        
        self.wallet = Some(wallet);
        Ok(())
    }

    pub fn import_from_private_key(&mut self, private_key: &[u8; 32]) -> Result<()> {
        let mut wallet = Wallet::from_private_key(private_key)?;
        
        // Initialize storage
        let db_path = self.data_dir.join("wallet.db");
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(wallet.initialize_storage(&db_path))?;
        
        self.wallet = Some(wallet);
        Ok(())
    }

    pub fn get_balance_string(&self) -> String {
        if let Some(wallet) = &self.wallet {
            format!("{:.8}", wallet.get_balance_par())
        } else {
            "0.00000000".to_string()
        }
    }

    pub fn get_address_string(&self) -> String {
        if let Some(wallet) = &self.wallet {
            wallet.get_address().to_string()
        } else {
            "No wallet loaded".to_string()
        }
    }

    pub fn get_current_address(&self) -> Option<Address> {
        self.wallet.as_ref().map(|w| w.get_address().clone())
    }

    pub async fn update_balance(&mut self, balance: u64) -> Result<()> {
        if let Some(wallet) = &mut self.wallet {
            wallet.update_balance(balance).await?;
        }
        Ok(())
    }

    pub async fn update_transactions(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        if let Some(wallet) = &mut self.wallet {
            for tx in transactions {
                wallet.add_transaction(tx).await?;
            }
        }
        Ok(())
    }

    pub fn get_transaction_history_display(&self) -> Vec<TransactionDisplay> {
        if let Some(wallet) = &self.wallet {
            wallet.get_transaction_history()
                .iter()
                .map(|tx| self.transaction_to_display(tx))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn transaction_to_display(&self, tx: &Transaction) -> TransactionDisplay {
        let wallet_address = self.get_current_address();
        
        let (direction, address) = if let Some(wallet_addr) = wallet_address {
            if tx.from == wallet_addr {
                ("Sent".to_string(), tx.to.to_string())
            } else {
                ("Received".to_string(), tx.from.to_string())
            }
        } else {
            ("Unknown".to_string(), "Unknown".to_string())
        };

        TransactionDisplay {
            id: tx.id.to_string(),
            direction,
            amount: format!("{:.8}", tx.amount as f64 / 100_000_000.0),
            fee: format!("{:.8}", tx.fee as f64 / 100_000_000.0),
            address,
            timestamp: tx.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            status: "Confirmed".to_string(), // Simplified
        }
    }

    pub fn send_transaction(
        &mut self,
        to_address: &str,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction> {
        if let Some(wallet) = &self.wallet {
            // Parse address
            let to = self.parse_address(to_address)?;
            
            // Create transaction
            let transaction = wallet.create_transaction(to, amount, fee)?;
            
            // In a real implementation, we'd broadcast this to the network
            // For now, we'll just return it
            Ok(transaction)
        } else {
            Err(anyhow::anyhow!("No wallet loaded"))
        }
    }

    fn parse_address(&self, addr_str: &str) -> Result<Address> {
        if addr_str.starts_with("PAR") && addr_str.len() == 67 {
            let hex_part = &addr_str[3..];
            let bytes = hex::decode(hex_part)?;
            if bytes.len() == 32 {
                let mut addr = [0u8; 32];
                addr.copy_from_slice(&bytes);
                Ok(Address(addr))
            } else {
                Err(anyhow::anyhow!("Invalid address length"))
            }
        } else {
            Err(anyhow::anyhow!("Invalid address format"))
        }
    }

    pub fn export_private_key(&self) -> Result<String> {
        if let Some(wallet) = &self.wallet {
            let private_key = wallet.export_private_key();
            Ok(hex::encode(private_key))
        } else {
            Err(anyhow::anyhow!("No wallet loaded"))
        }
    }

    pub fn export_seed_phrase(&self) -> Result<Vec<String>> {
        // In a real implementation, we'd derive this from the stored wallet
        // For now, return a placeholder
        Ok(Wallet::generate_seed_phrase())
    }
}
