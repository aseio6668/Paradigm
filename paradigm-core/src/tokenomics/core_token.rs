use crate::Address;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core token implementation with ERC-20 compatible features
/// plus advanced tokenomics for computational merit and governance
#[derive(Debug)]
pub struct CoreToken {
    /// Total supply of tokens in existence
    total_supply: u64,
    /// Balances for each address
    balances: HashMap<Address, u64>,
    /// Allowances for spending on behalf of others
    allowances: HashMap<Address, HashMap<Address, u64>>,
    /// Frozen/locked tokens per address
    frozen_balances: HashMap<Address, u64>,
    /// Token metadata
    token_info: TokenInfo,
    /// Minting history for transparency
    mint_history: Vec<MintRecord>,
    /// Burn history for deflationary mechanisms
    burn_history: Vec<BurnRecord>,
}

impl CoreToken {
    pub fn new() -> Self {
        CoreToken {
            total_supply: 0,
            balances: HashMap::new(),
            allowances: HashMap::new(),
            frozen_balances: HashMap::new(),
            token_info: TokenInfo {
                name: "Paradigm".to_string(),
                symbol: "PAR".to_string(),
                decimals: 8,
                version: "2.0".to_string(),
            },
            mint_history: Vec::new(),
            burn_history: Vec::new(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::info!("Initializing Core Token system");
        tracing::info!(
            "Token: {} ({}) with {} decimals",
            self.token_info.name,
            self.token_info.symbol,
            self.token_info.decimals
        );
        Ok(())
    }

    /// Get balance for an address
    pub fn balance_of(&self, address: &Address) -> u64 {
        *self.balances.get(address).unwrap_or(&0)
    }

    /// Get total supply
    pub fn total_supply(&self) -> u64 {
        self.total_supply
    }

    /// Get available (non-frozen) balance
    pub fn available_balance(&self, address: &Address) -> u64 {
        let total_balance = self.balance_of(address);
        let frozen = *self.frozen_balances.get(address).unwrap_or(&0);
        total_balance.saturating_sub(frozen)
    }

    /// Transfer tokens between addresses
    pub async fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        amount: u64,
    ) -> anyhow::Result<()> {
        if from == to {
            return Err(anyhow::anyhow!("Cannot transfer to same address"));
        }

        let from_balance = self.available_balance(from);
        if from_balance < amount {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }

        // Update balances
        let new_from_balance = self.balance_of(from) - amount;
        let new_to_balance = self.balance_of(to) + amount;

        self.balances.insert(from.clone(), new_from_balance);
        self.balances.insert(to.clone(), new_to_balance);

        tracing::debug!(
            "Transferred {} PAR from {} to {}",
            amount as f64 / 100_000_000.0,
            from.to_string(),
            to.to_string()
        );

        Ok(())
    }

    /// Approve spending allowance
    pub async fn approve(
        &mut self,
        owner: &Address,
        spender: &Address,
        amount: u64,
    ) -> anyhow::Result<()> {
        self.allowances
            .entry(owner.clone())
            .or_insert_with(HashMap::new)
            .insert(spender.clone(), amount);

        Ok(())
    }

    /// Transfer from approved allowance
    pub async fn transfer_from(
        &mut self,
        spender: &Address,
        from: &Address,
        to: &Address,
        amount: u64,
    ) -> anyhow::Result<()> {
        // Check allowance
        let allowance = *self
            .allowances
            .get(from)
            .and_then(|allowances| allowances.get(spender))
            .unwrap_or(&0);

        if allowance < amount {
            return Err(anyhow::anyhow!("Insufficient allowance"));
        }

        // Perform transfer
        self.transfer(from, to, amount).await?;

        // Update allowance
        let new_allowance = allowance - amount;
        self.allowances
            .get_mut(from)
            .unwrap()
            .insert(spender.clone(), new_allowance);

        Ok(())
    }

    /// Mint new tokens (for rewards)
    pub async fn mint_tokens(&mut self, to: &Address, amount: u64) -> anyhow::Result<u64> {
        if amount == 0 {
            return Err(anyhow::anyhow!("Cannot mint zero tokens"));
        }

        // Update balance
        let current_balance = self.balance_of(to);
        let new_balance = current_balance + amount;
        self.balances.insert(to.clone(), new_balance);

        // Update total supply
        self.total_supply += amount;

        // Record mint
        self.mint_history.push(MintRecord {
            to: to.clone(),
            amount,
            timestamp: Utc::now(),
            reason: MintReason::ContributionReward,
        });

        tracing::info!(
            "Minted {} PAR to {}",
            amount as f64 / 100_000_000.0,
            to.to_string()
        );

        Ok(amount)
    }

    /// Burn tokens (deflationary mechanism)
    pub async fn burn_tokens(&mut self, from: &Address, amount: u64) -> anyhow::Result<()> {
        let balance = self.available_balance(from);
        if balance < amount {
            return Err(anyhow::anyhow!("Insufficient balance to burn"));
        }

        // Update balance
        let current_balance = self.balance_of(from);
        let new_balance = current_balance - amount;
        self.balances.insert(from.clone(), new_balance);

        // Update total supply
        self.total_supply = self.total_supply.saturating_sub(amount);

        // Record burn
        self.burn_history.push(BurnRecord {
            from: from.clone(),
            amount,
            timestamp: Utc::now(),
            reason: BurnReason::Deflationary,
        });

        tracing::info!(
            "Burned {} PAR from {}",
            amount as f64 / 100_000_000.0,
            from.to_string()
        );

        Ok(())
    }

    /// Freeze tokens (for staking, governance, etc.)
    pub async fn freeze_tokens(&mut self, address: &Address, amount: u64) -> anyhow::Result<()> {
        let available = self.available_balance(address);
        if available < amount {
            return Err(anyhow::anyhow!("Insufficient available balance to freeze"));
        }

        let current_frozen = *self.frozen_balances.get(address).unwrap_or(&0);
        self.frozen_balances
            .insert(address.clone(), current_frozen + amount);

        tracing::debug!(
            "Froze {} PAR for {}",
            amount as f64 / 100_000_000.0,
            address.to_string()
        );

        Ok(())
    }

    /// Unfreeze tokens
    pub async fn unfreeze_tokens(&mut self, address: &Address, amount: u64) -> anyhow::Result<()> {
        let current_frozen = *self.frozen_balances.get(address).unwrap_or(&0);
        if current_frozen < amount {
            return Err(anyhow::anyhow!("Cannot unfreeze more than frozen amount"));
        }

        let new_frozen = current_frozen - amount;
        if new_frozen == 0 {
            self.frozen_balances.remove(address);
        } else {
            self.frozen_balances.insert(address.clone(), new_frozen);
        }

        tracing::debug!(
            "Unfroze {} PAR for {}",
            amount as f64 / 100_000_000.0,
            address.to_string()
        );

        Ok(())
    }

    /// Get token as cross-platform compute credits
    pub fn get_compute_credits(&self, address: &Address) -> ComputeCredits {
        let balance = self.available_balance(address);
        ComputeCredits {
            par_balance: balance,
            cpu_credits: balance / 1000,    // 1 PAR = 1000 CPU units
            gpu_credits: balance / 10000,   // 1 PAR = 100 GPU units
            storage_credits: balance * 10,  // 1 PAR = 10 GB storage
            bandwidth_credits: balance * 5, // 1 PAR = 5 GB bandwidth
        }
    }

    /// Get mint history for transparency
    pub fn get_mint_history(&self) -> &Vec<MintRecord> {
        &self.mint_history
    }

    /// Get burn history for transparency
    pub fn get_burn_history(&self) -> &Vec<BurnRecord> {
        &self.burn_history
    }

    /// Get token info
    pub fn get_token_info(&self) -> &TokenInfo {
        &self.token_info
    }
}

/// Token metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub version: String,
}

/// Record of token minting
#[derive(Debug, Serialize, Deserialize)]
pub struct MintRecord {
    pub to: Address,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
    pub reason: MintReason,
}

/// Record of token burning
#[derive(Debug, Serialize, Deserialize)]
pub struct BurnRecord {
    pub from: Address,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
    pub reason: BurnReason,
}

/// Reasons for minting tokens
#[derive(Debug, Serialize, Deserialize)]
pub enum MintReason {
    ContributionReward,
    InferencePayment,
    GovernanceIncentive,
    NetworkMaintenance,
    CrossPlatformBridge,
}

/// Reasons for burning tokens
#[derive(Debug, Serialize, Deserialize)]
pub enum BurnReason {
    Deflationary,
    PenaltySlashing,
    ComputeConsumption,
    BridgeTransfer,
    GovernanceDecision,
}

/// Compute credits derived from PAR tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct ComputeCredits {
    pub par_balance: u64,
    pub cpu_credits: u64,
    pub gpu_credits: u64,
    pub storage_credits: u64,
    pub bandwidth_credits: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Keypair;
    use rand::thread_rng;

    #[tokio::test]
    async fn test_core_token_basic_operations() {
        let mut token = CoreToken::new();
        token.initialize().await.unwrap();

        let keypair1 = Keypair::generate(&mut thread_rng());
        let keypair2 = Keypair::generate(&mut thread_rng());
        let addr1 = Address::from_public_key(&keypair1.public);
        let addr2 = Address::from_public_key(&keypair2.public);

        // Mint tokens
        token.mint_tokens(&addr1, 1000000000).await.unwrap(); // 10 PAR
        assert_eq!(token.balance_of(&addr1), 1000000000);
        assert_eq!(token.total_supply(), 1000000000);

        // Transfer tokens
        token.transfer(&addr1, &addr2, 500000000).await.unwrap(); // 5 PAR
        assert_eq!(token.balance_of(&addr1), 500000000);
        assert_eq!(token.balance_of(&addr2), 500000000);

        // Freeze tokens
        token.freeze_tokens(&addr1, 200000000).await.unwrap(); // 2 PAR
        assert_eq!(token.available_balance(&addr1), 300000000); // 3 PAR available
    }

    #[tokio::test]
    async fn test_compute_credits() {
        let mut token = CoreToken::new();
        let keypair = Keypair::generate(&mut thread_rng());
        let addr = Address::from_public_key(&keypair.public);

        token.mint_tokens(&addr, 1000000000).await.unwrap(); // 10 PAR
        let credits = token.get_compute_credits(&addr);

        assert_eq!(credits.par_balance, 1000000000);
        assert_eq!(credits.cpu_credits, 1000000);
        assert_eq!(credits.gpu_credits, 100000);
    }
}
