use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use blake3::Hasher;

use crate::{Hash, Amount, Address};
use super::{ChainId, CrossChainConfig, CrossChainAsset};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetType {
    Native,
    Wrapped,
    Synthetic,
    Bridged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub asset_id: Uuid,
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub decimals: u8,
    pub total_supply: Option<u128>,
    pub max_supply: Option<u128>,
    pub asset_type: AssetType,
    pub origin_chain: ChainId,
    pub supported_chains: Vec<ChainId>,
    pub icon_url: Option<String>,
    pub website_url: Option<String>,
    pub whitepaper_url: Option<String>,
    pub is_verified: bool,
    pub risk_score: u8, // 0-100, lower is safer
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    pub asset_id: Uuid,
    pub chain_id: ChainId,
    pub address: Address,
    pub balance: u128,
    pub locked_balance: u128,
    pub pending_balance: u128,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTransfer {
    pub transfer_id: Uuid,
    pub asset_id: Uuid,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub from_address: Address,
    pub to_address: Address,
    pub amount: u128,
    pub fee: u128,
    pub status: TransferStatus,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub bridge_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub confirmations_required: u64,
    pub confirmations_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferStatus {
    Initiated,
    Pending,
    Locked,
    InTransit,
    Confirmed,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrappedAsset {
    pub wrapped_id: Uuid,
    pub original_asset_id: Uuid,
    pub origin_chain: ChainId,
    pub wrapped_chain: ChainId,
    pub contract_address: String,
    pub total_wrapped: u128,
    pub backing_reserves: u128,
    pub wrapping_fee_rate: f64,
    pub unwrapping_fee_rate: f64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPrice {
    pub asset_id: Uuid,
    pub price_usd: f64,
    pub price_btc: Option<f64>,
    pub price_eth: Option<f64>,
    pub volume_24h_usd: f64,
    pub market_cap_usd: Option<f64>,
    pub price_change_24h: f64,
    pub last_updated: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLiquidity {
    pub asset_id: Uuid,
    pub chain_id: ChainId,
    pub total_liquidity: u128,
    pub available_liquidity: u128,
    pub locked_liquidity: u128,
    pub utilization_rate: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAllocation {
    pub allocation_id: Uuid,
    pub portfolio_id: Uuid,
    pub asset_id: Uuid,
    pub target_percentage: f64,
    pub current_percentage: f64,
    pub target_amount: u128,
    pub current_amount: u128,
    pub rebalance_threshold: f64,
    pub last_rebalanced: DateTime<Utc>,
}

pub struct CrossChainAssetManager {
    asset_registry: Arc<RwLock<HashMap<Uuid, AssetMetadata>>>,
    wrapped_assets: Arc<RwLock<HashMap<Uuid, WrappedAsset>>>,
    asset_balances: Arc<RwLock<HashMap<(Address, Uuid, ChainId), AssetBalance>>>,
    pending_transfers: Arc<RwLock<HashMap<Uuid, AssetTransfer>>>,
    asset_prices: Arc<RwLock<HashMap<Uuid, AssetPrice>>>,
    asset_liquidity: Arc<RwLock<HashMap<(Uuid, ChainId), AssetLiquidity>>>,
    asset_allocations: Arc<RwLock<HashMap<Uuid, AssetAllocation>>>,
    config: CrossChainConfig,
    symbol_to_asset: Arc<RwLock<HashMap<String, Uuid>>>,
    chain_asset_mappings: Arc<RwLock<HashMap<(ChainId, String), Uuid>>>,
}

impl CrossChainAssetManager {
    pub async fn new(config: CrossChainConfig) -> Result<Self> {
        Ok(Self {
            asset_registry: Arc::new(RwLock::new(HashMap::new())),
            wrapped_assets: Arc::new(RwLock::new(HashMap::new())),
            asset_balances: Arc::new(RwLock::new(HashMap::new())),
            pending_transfers: Arc::new(RwLock::new(HashMap::new())),
            asset_prices: Arc::new(RwLock::new(HashMap::new())),
            asset_liquidity: Arc::new(RwLock::new(HashMap::new())),
            asset_allocations: Arc::new(RwLock::new(HashMap::new())),
            config,
            symbol_to_asset: Arc::new(RwLock::new(HashMap::new())),
            chain_asset_mappings: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing Cross-Chain Asset Manager...");
        
        // Register default assets
        self.register_default_assets().await?;
        
        // Start monitoring tasks
        self.start_monitoring_tasks().await?;
        
        tracing::info!("Cross-Chain Asset Manager initialized successfully");
        Ok(())
    }

    pub async fn register_asset(&self, metadata: AssetMetadata) -> Result<()> {
        let asset_id = metadata.asset_id;
        let symbol = metadata.symbol.clone();
        
        // Store asset metadata
        {
            let mut registry = self.asset_registry.write().await;
            registry.insert(asset_id, metadata.clone());
        }
        
        // Create symbol mapping
        {
            let mut symbol_map = self.symbol_to_asset.write().await;
            symbol_map.insert(symbol, asset_id);
        }
        
        // Create chain mappings for each supported chain
        {
            let mut chain_mappings = self.chain_asset_mappings.write().await;
            for chain_id in &metadata.supported_chains {
                let key = (*chain_id, metadata.symbol.clone());
                chain_mappings.insert(key, asset_id);
            }
        }
        
        tracing::info!("Registered asset: {} ({})", metadata.name, metadata.symbol);
        Ok(())
    }

    pub async fn create_wrapped_asset(
        &self,
        original_asset_id: Uuid,
        origin_chain: ChainId,
        wrapped_chain: ChainId,
        contract_address: String,
        wrapping_fee_rate: f64,
        unwrapping_fee_rate: f64,
    ) -> Result<Uuid> {
        let wrapped_id = Uuid::new_v4();
        
        let wrapped_asset = WrappedAsset {
            wrapped_id,
            original_asset_id,
            origin_chain,
            wrapped_chain,
            contract_address,
            total_wrapped: 0,
            backing_reserves: 0,
            wrapping_fee_rate,
            unwrapping_fee_rate,
            is_active: true,
            created_at: Utc::now(),
        };
        
        let mut wrapped_assets = self.wrapped_assets.write().await;
        wrapped_assets.insert(wrapped_id, wrapped_asset);
        
        tracing::info!("Created wrapped asset: {} -> {:?}", wrapped_id, wrapped_chain);
        Ok(wrapped_id)
    }

    pub async fn wrap_asset(
        &self,
        asset_id: Uuid,
        amount: u128,
        from_address: Address,
        to_address: Address,
        from_chain: ChainId,
        to_chain: ChainId,
    ) -> Result<Uuid> {
        let transfer_id = Uuid::new_v4();
        
        // Validate asset exists
        let asset = self.get_asset_metadata(&asset_id).await
            .ok_or_else(|| anyhow!("Asset not found"))?;
        
        // Check if wrapping is supported
        if !asset.supported_chains.contains(&to_chain) {
            return Err(anyhow!("Asset not supported on target chain"));
        }
        
        // Calculate fees
        let fee = self.calculate_wrapping_fee(asset_id, amount, from_chain, to_chain).await?;
        
        let transfer = AssetTransfer {
            transfer_id,
            asset_id,
            from_chain,
            to_chain,
            from_address,
            to_address,
            amount,
            fee,
            status: TransferStatus::Initiated,
            source_tx_hash: None,
            destination_tx_hash: None,
            bridge_tx_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            confirmations_required: self.config.confirmation_requirements
                .get(&from_chain).cloned().unwrap_or(6),
            confirmations_received: 0,
        };
        
        let mut pending_transfers = self.pending_transfers.write().await;
        pending_transfers.insert(transfer_id, transfer);
        
        tracing::info!("Initiated asset wrapping: {}", transfer_id);
        Ok(transfer_id)
    }

    pub async fn unwrap_asset(
        &self,
        wrapped_id: Uuid,
        amount: u128,
        from_address: Address,
        to_address: Address,
    ) -> Result<Uuid> {
        let transfer_id = Uuid::new_v4();
        
        // Get wrapped asset info
        let wrapped_asset = {
            let wrapped_assets = self.wrapped_assets.read().await;
            wrapped_assets.get(&wrapped_id).cloned()
                .ok_or_else(|| anyhow!("Wrapped asset not found"))?
        };
        
        if !wrapped_asset.is_active {
            return Err(anyhow!("Wrapped asset is not active"));
        }
        
        // Calculate fees
        let fee = (amount as f64 * wrapped_asset.unwrapping_fee_rate) as u128;
        
        let transfer = AssetTransfer {
            transfer_id,
            asset_id: wrapped_asset.original_asset_id,
            from_chain: wrapped_asset.wrapped_chain,
            to_chain: wrapped_asset.origin_chain,
            from_address,
            to_address,
            amount,
            fee,
            status: TransferStatus::Initiated,
            source_tx_hash: None,
            destination_tx_hash: None,
            bridge_tx_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            confirmations_required: self.config.confirmation_requirements
                .get(&wrapped_asset.wrapped_chain).cloned().unwrap_or(6),
            confirmations_received: 0,
        };
        
        let mut pending_transfers = self.pending_transfers.write().await;
        pending_transfers.insert(transfer_id, transfer);
        
        tracing::info!("Initiated asset unwrapping: {}", transfer_id);
        Ok(transfer_id)
    }

    pub async fn get_asset_balance(
        &self,
        address: &Address,
        asset_id: &Uuid,
        chain_id: ChainId,
    ) -> Option<AssetBalance> {
        let balances = self.asset_balances.read().await;
        balances.get(&(*address, *asset_id, chain_id)).cloned()
    }

    pub async fn update_asset_balance(
        &self,
        address: Address,
        asset_id: Uuid,
        chain_id: ChainId,
        balance: u128,
        locked_balance: u128,
        pending_balance: u128,
    ) -> Result<()> {
        let asset_balance = AssetBalance {
            asset_id,
            chain_id,
            address,
            balance,
            locked_balance,
            pending_balance,
            last_updated: Utc::now(),
        };
        
        let mut balances = self.asset_balances.write().await;
        balances.insert((address, asset_id, chain_id), asset_balance);
        
        Ok(())
    }

    pub async fn get_asset_metadata(&self, asset_id: &Uuid) -> Option<AssetMetadata> {
        let registry = self.asset_registry.read().await;
        registry.get(asset_id).cloned()
    }

    pub async fn find_asset_by_symbol(&self, symbol: &str) -> Option<Uuid> {
        let symbol_map = self.symbol_to_asset.read().await;
        symbol_map.get(symbol).cloned()
    }

    pub async fn get_supported_assets(&self, chain_id: ChainId) -> Vec<AssetMetadata> {
        let registry = self.asset_registry.read().await;
        registry.values()
            .filter(|asset| asset.supported_chains.contains(&chain_id))
            .cloned()
            .collect()
    }

    pub async fn update_asset_price(
        &self,
        asset_id: Uuid,
        price_usd: f64,
        volume_24h_usd: f64,
        price_change_24h: f64,
        source: String,
    ) -> Result<()> {
        let price = AssetPrice {
            asset_id,
            price_usd,
            price_btc: None,
            price_eth: None,
            volume_24h_usd,
            market_cap_usd: None,
            price_change_24h,
            last_updated: Utc::now(),
            source,
        };
        
        let mut prices = self.asset_prices.write().await;
        prices.insert(asset_id, price);
        
        Ok(())
    }

    pub async fn get_asset_price(&self, asset_id: &Uuid) -> Option<AssetPrice> {
        let prices = self.asset_prices.read().await;
        prices.get(asset_id).cloned()
    }

    pub async fn calculate_portfolio_value(
        &self,
        address: &Address,
        chain_ids: &[ChainId],
    ) -> Result<f64> {
        let mut total_value = 0.0;
        
        let balances = self.asset_balances.read().await;
        let prices = self.asset_prices.read().await;
        
        for (key, balance) in balances.iter() {
            let (addr, asset_id, chain_id) = key;
            
            if addr != address || !chain_ids.contains(chain_id) {
                continue;
            }
            
            if let Some(price) = prices.get(asset_id) {
                let asset_value = (balance.balance as f64 / 10_f64.powi(8)) * price.price_usd;
                total_value += asset_value;
            }
        }
        
        Ok(total_value)
    }

    pub async fn get_transfer_status(&self, transfer_id: &Uuid) -> Option<AssetTransfer> {
        let transfers = self.pending_transfers.read().await;
        transfers.get(transfer_id).cloned()
    }

    pub async fn list_pending_transfers(&self) -> Vec<AssetTransfer> {
        let transfers = self.pending_transfers.read().await;
        transfers.values().cloned().collect()
    }

    pub async fn get_asset_liquidity(
        &self,
        asset_id: &Uuid,
        chain_id: ChainId,
    ) -> Option<AssetLiquidity> {
        let liquidity = self.asset_liquidity.read().await;
        liquidity.get(&(*asset_id, chain_id)).cloned()
    }

    // Private methods

    async fn register_default_assets(&self) -> Result<()> {
        // Register PAR token
        let par_metadata = AssetMetadata {
            asset_id: Uuid::new_v4(),
            symbol: "PAR".to_string(),
            name: "Paradigm".to_string(),
            description: "Native token of the Paradigm network".to_string(),
            decimals: 8,
            total_supply: Some(8_000_000_000_00000000),
            max_supply: Some(8_000_000_000_00000000),
            asset_type: AssetType::Native,
            origin_chain: ChainId::Paradigm,
            supported_chains: vec![ChainId::Paradigm, ChainId::Ethereum, ChainId::Cosmos],
            icon_url: None,
            website_url: Some("https://paradigm.network".to_string()),
            whitepaper_url: Some("https://paradigm.network/whitepaper".to_string()),
            is_verified: true,
            risk_score: 10,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.register_asset(par_metadata).await?;
        
        // Register wrapped Ethereum
        let weth_metadata = AssetMetadata {
            asset_id: Uuid::new_v4(),
            symbol: "WETH".to_string(),
            name: "Wrapped Ethereum".to_string(),
            description: "Wrapped version of Ethereum on Paradigm".to_string(),
            decimals: 18,
            total_supply: None,
            max_supply: None,
            asset_type: AssetType::Wrapped,
            origin_chain: ChainId::Ethereum,
            supported_chains: vec![ChainId::Ethereum, ChainId::Paradigm],
            icon_url: None,
            website_url: Some("https://ethereum.org".to_string()),
            whitepaper_url: None,
            is_verified: true,
            risk_score: 15,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.register_asset(weth_metadata).await?;
        
        // Register wrapped Bitcoin
        let wbtc_metadata = AssetMetadata {
            asset_id: Uuid::new_v4(),
            symbol: "WBTC".to_string(),
            name: "Wrapped Bitcoin".to_string(),
            description: "Wrapped version of Bitcoin on Paradigm".to_string(),
            decimals: 8,
            total_supply: Some(21_000_000_00000000),
            max_supply: Some(21_000_000_00000000),
            asset_type: AssetType::Wrapped,
            origin_chain: ChainId::Bitcoin,
            supported_chains: vec![ChainId::Bitcoin, ChainId::Ethereum, ChainId::Paradigm],
            icon_url: None,
            website_url: Some("https://bitcoin.org".to_string()),
            whitepaper_url: Some("https://bitcoin.org/bitcoin.pdf".to_string()),
            is_verified: true,
            risk_score: 20,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.register_asset(wbtc_metadata).await?;
        
        Ok(())
    }

    async fn start_monitoring_tasks(&self) -> Result<()> {
        // Monitor transfer confirmations
        let pending_transfers = self.pending_transfers.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                let mut completed_transfers = Vec::new();
                {
                    let transfers = pending_transfers.read().await;
                    for (transfer_id, transfer) in transfers.iter() {
                        if transfer.confirmations_received >= transfer.confirmations_required {
                            completed_transfers.push(*transfer_id);
                        }
                    }
                }
                
                {
                    let mut transfers = pending_transfers.write().await;
                    for transfer_id in completed_transfers {
                        if let Some(transfer) = transfers.get_mut(&transfer_id) {
                            transfer.status = TransferStatus::Completed;
                            transfer.updated_at = Utc::now();
                            tracing::info!("Transfer {} completed", transfer_id);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }

    async fn calculate_wrapping_fee(
        &self,
        asset_id: Uuid,
        amount: u128,
        from_chain: ChainId,
        to_chain: ChainId,
    ) -> Result<u128> {
        // Base fee rate from config
        let base_fee_rate = self.config.bridge_fee_rates
            .get(&to_chain).cloned().unwrap_or(0.001);
        
        let fee = (amount as f64 * base_fee_rate) as u128;
        
        // Minimum fee
        let min_fee = match to_chain {
            ChainId::Ethereum => 50_000,
            ChainId::Bitcoin => 10_000,
            ChainId::Cosmos => 1_000,
            _ => 5_000,
        };
        
        Ok(fee.max(min_fee))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_asset_manager_creation() {
        let config = CrossChainConfig::default();
        let manager = CrossChainAssetManager::new(config).await;
        assert!(manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_asset_registration() {
        let config = CrossChainConfig::default();
        let manager = CrossChainAssetManager::new(config).await.unwrap();
        
        let asset_id = Uuid::new_v4();
        let metadata = AssetMetadata {
            asset_id,
            symbol: "TEST".to_string(),
            name: "Test Token".to_string(),
            description: "A test token".to_string(),
            decimals: 18,
            total_supply: Some(1_000_000_000_000_000_000_000),
            max_supply: Some(1_000_000_000_000_000_000_000),
            asset_type: AssetType::Native,
            origin_chain: ChainId::Paradigm,
            supported_chains: vec![ChainId::Paradigm],
            icon_url: None,
            website_url: None,
            whitepaper_url: None,
            is_verified: false,
            risk_score: 50,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let result = manager.register_asset(metadata).await;
        assert!(result.is_ok());
        
        let found_asset = manager.get_asset_metadata(&asset_id).await;
        assert!(found_asset.is_some());
        assert_eq!(found_asset.unwrap().symbol, "TEST");
    }
    
    #[tokio::test]
    async fn test_asset_balance_operations() {
        let config = CrossChainConfig::default();
        let manager = CrossChainAssetManager::new(config).await.unwrap();
        
        let address = Address([1u8; 32]);
        let asset_id = Uuid::new_v4();
        let chain_id = ChainId::Paradigm;
        
        let result = manager.update_asset_balance(
            address,
            asset_id,
            chain_id,
            1000,
            100,
            50,
        ).await;
        assert!(result.is_ok());
        
        let balance = manager.get_asset_balance(&address, &asset_id, chain_id).await;
        assert!(balance.is_some());
        assert_eq!(balance.unwrap().balance, 1000);
    }
}