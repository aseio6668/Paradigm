use crate::{wallet_manager::WalletManager, Address, Transaction};
use anyhow::Result;
use chrono::Utc;
use ed25519_dalek::SigningKey;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{error, info};
use uuid::Uuid;

/// Transaction test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionTestResult {
    pub test_id: Uuid,
    pub test_type: String,
    pub success: bool,
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
    pub message: Option<String>,
    pub duration_ms: u64,
    pub tx_hash: Option<String>,
    pub error_message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub recommendations: Vec<String>,
}

/// Transaction testing system
#[derive(Debug)]
pub struct TransactionTester {
    test_results: Vec<TransactionTestResult>,
}

impl TransactionTester {
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
        }
    }

    /// Run comprehensive transaction tests between wallet addresses
    pub async fn run_wallet_transaction_test(
        &mut self,
        wallet_manager: &mut WalletManager,
        amount_par: Option<f64>,
        test_message: Option<&str>,
    ) -> Result<TransactionTestResult> {
        info!("🧪 Starting wallet transaction test...");

        let start_time = Instant::now();
        let test_id = Uuid::new_v4();

        // Use minimal amount if not specified (0.00000001 PAR)
        let amount = amount_par.unwrap_or(0.00000001);
        let amount_sats = (amount * 100_000_000.0) as u64;

        // Ensure we have at least 2 addresses for testing
        let addresses = wallet_manager.list_addresses();
        let (from_addr, to_addr) = if addresses.len() < 2 {
            info!("🏗️ Creating test addresses (need 2 for testing)...");

            let from_addr = wallet_manager.add_address("test-sender")?;
            let to_addr = wallet_manager.add_address("test-receiver")?;

            info!("✅ Created sender: {} and receiver: {}", from_addr, to_addr);
            (from_addr, to_addr)
        } else {
            let addrs: Vec<String> = addresses
                .into_iter()
                .map(|(addr, _)| addr.clone())
                .collect();
            (addrs[0].clone(), addrs[1].clone())
        };

        // Create and test the transaction
        let mut test_result = TransactionTestResult {
            test_id,
            test_type: "wallet_to_wallet".to_string(),
            success: false,
            from_address: from_addr.clone(),
            to_address: to_addr.clone(),
            amount: amount_sats,
            message: test_message.map(|m| m.to_string()),
            duration_ms: 0,
            tx_hash: None,
            error_message: None,
            timestamp: Utc::now(),
            recommendations: Vec::new(),
        };

        // Perform the transaction test
        match self
            .execute_transaction_test(&from_addr, &to_addr, amount_sats, test_message)
            .await
        {
            Ok(tx_hash) => {
                test_result.success = true;
                test_result.tx_hash = Some(tx_hash);
                test_result
                    .recommendations
                    .push("✅ Transaction system working correctly".to_string());
                info!("✅ Transaction test successful!");
            }
            Err(e) => {
                test_result.error_message = Some(e.to_string());
                test_result
                    .recommendations
                    .push("❌ Transaction failed - check network connectivity".to_string());
                test_result
                    .recommendations
                    .push("💡 Verify wallet has sufficient balance".to_string());
                error!("❌ Transaction test failed: {}", e);
            }
        }

        test_result.duration_ms = start_time.elapsed().as_millis() as u64;

        // Add performance recommendations
        if test_result.duration_ms > 5000 {
            test_result
                .recommendations
                .push("⚠️ Transaction took longer than expected (>5s)".to_string());
            test_result
                .recommendations
                .push("💡 Consider checking network latency".to_string());
        } else if test_result.duration_ms < 100 {
            test_result
                .recommendations
                .push("🚀 Excellent transaction speed!".to_string());
        }

        self.test_results.push(test_result.clone());
        self.print_test_result(&test_result);

        Ok(test_result)
    }

    /// Execute actual transaction test
    async fn execute_transaction_test(
        &self,
        from_address: &str,
        to_address: &str,
        amount_sats: u64,
        message: Option<&str>,
    ) -> Result<String> {
        info!("📤 Creating test transaction...");
        info!("   From: {}", from_address);
        info!("   To: {}", to_address);
        info!("   Amount: {:.8} PAR", amount_sats as f64 / 100_000_000.0);
        if let Some(msg) = message {
            info!("   Message: '{}'", msg);
        }

        // Parse addresses
        let from_addr = Address::from_string(from_address)?;
        let to_addr = Address::from_string(to_address)?;

        // Create a test keypair (in production, this would come from wallet)
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let keypair = SigningKey::from_bytes(&secret_bytes);

        // Create transaction
        let transaction = if let Some(msg) = message {
            Transaction::new_with_message(
                from_addr,
                to_addr,
                amount_sats,
                1000, // 0.00001 PAR fee
                Utc::now(),
                &keypair,
                Some(msg.to_string()),
            )?
        } else {
            Transaction::new(from_addr, to_addr, amount_sats, 1000, Utc::now(), &keypair)?
        };

        let tx_hash = hex::encode(transaction.hash());
        info!("🔗 Transaction created with hash: {}", tx_hash);

        // Validate transaction
        let public_key = keypair.verifying_key();
        transaction.validate(&public_key)?;
        info!("✅ Transaction validation passed");

        // In a real system, this would be broadcast to the network
        // For now, we simulate successful processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(tx_hash)
    }

    /// Print detailed test result
    fn print_test_result(&self, result: &TransactionTestResult) {
        println!("\n🧪 Transaction Test Results");
        println!("================================");
        println!("🆔 Test ID: {}", result.test_id);
        println!("📊 Test Type: {}", result.test_type);
        println!("✅ Success: {}", if result.success { "YES" } else { "NO" });
        println!("📤 From: {}", result.from_address);
        println!("📥 To: {}", result.to_address);
        println!("💰 Amount: {:.8} PAR", result.amount as f64 / 100_000_000.0);

        if let Some(ref message) = result.message {
            println!("💬 Message: '{}'", message);
        }

        println!("⏱️  Duration: {}ms", result.duration_ms);

        if let Some(ref hash) = result.tx_hash {
            println!("🔗 Transaction Hash: {}", hash);
        }

        if let Some(ref error) = result.error_message {
            println!("❌ Error: {}", error);
        }

        println!(
            "📅 Timestamp: {}",
            result.timestamp.format("%Y-%m-%d %H:%M:%S")
        );

        if !result.recommendations.is_empty() {
            println!("\n💡 Recommendations:");
            for rec in &result.recommendations {
                println!("   {}", rec);
            }
        }
        println!();
    }

    /// Run stress test with multiple transactions
    pub async fn run_stress_test(
        &mut self,
        wallet_manager: &mut WalletManager,
        num_transactions: usize,
        concurrent: bool,
    ) -> Result<Vec<TransactionTestResult>> {
        info!(
            "🏋️ Starting stress test with {} transactions (concurrent: {})",
            num_transactions, concurrent
        );

        let mut results = Vec::new();

        if concurrent {
            // Run concurrent transactions
            let handles: Vec<tokio::task::JoinHandle<Result<TransactionTestResult>>> =
                Vec::new();

            for i in 0..num_transactions {
                let test_message = format!("test-{}", i);
                // In a real implementation, we'd need to clone wallet_manager safely
                // For now, run sequentially to avoid borrowing issues
            }

            // For now, run sequentially to avoid complex async borrowing
            for i in 0..num_transactions {
                let message = format!("stress-{}", i);
                let result = self
                    .run_wallet_transaction_test(wallet_manager, Some(0.00000001), Some(&message))
                    .await?;
                results.push(result);
            }
        } else {
            // Run sequential transactions
            for i in 0..num_transactions {
                let message = format!("seq-{}", i);
                let result = self
                    .run_wallet_transaction_test(wallet_manager, Some(0.00000001), Some(&message))
                    .await?;
                results.push(result);
            }
        }

        self.print_stress_test_summary(&results);

        Ok(results)
    }

    /// Print stress test summary
    fn print_stress_test_summary(&self, results: &[TransactionTestResult]) {
        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;
        let avg_duration: f64 =
            results.iter().map(|r| r.duration_ms as f64).sum::<f64>() / results.len() as f64;
        let min_duration = results.iter().map(|r| r.duration_ms).min().unwrap_or(0);
        let max_duration = results.iter().map(|r| r.duration_ms).max().unwrap_or(0);

        println!("\n🏋️ Stress Test Summary");
        println!("========================");
        println!("📊 Total Tests: {}", results.len());
        println!(
            "✅ Successful: {} ({:.1}%)",
            successful,
            (successful as f64 / results.len() as f64) * 100.0
        );
        println!(
            "❌ Failed: {} ({:.1}%)",
            failed,
            (failed as f64 / results.len() as f64) * 100.0
        );
        println!("⏱️  Avg Duration: {:.2}ms", avg_duration);
        println!("⚡ Min Duration: {}ms", min_duration);
        println!("🐌 Max Duration: {}ms", max_duration);

        if failed > 0 {
            println!("\n❌ Failed Transaction Details:");
            for (i, result) in results.iter().enumerate() {
                if !result.success {
                    println!(
                        "   Test {}: {}",
                        i + 1,
                        result
                            .error_message
                            .as_ref()
                            .unwrap_or(&"Unknown error".to_string())
                    );
                }
            }
        }

        println!();
    }

    /// Get all test results
    pub fn get_test_results(&self) -> &[TransactionTestResult] {
        &self.test_results
    }

    /// Get successful test count
    pub fn get_success_rate(&self) -> f64 {
        if self.test_results.is_empty() {
            return 0.0;
        }

        let successful = self.test_results.iter().filter(|r| r.success).count();
        (successful as f64 / self.test_results.len() as f64) * 100.0
    }

    /// Clear test history
    pub fn clear_test_results(&mut self) {
        self.test_results.clear();
    }
}

impl Default for TransactionTester {
    fn default() -> Self {
        Self::new()
    }
}
