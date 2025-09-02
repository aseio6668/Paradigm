// Paradigm Wallet - CLI interface with full wallet management
use anyhow::Result;
use paradigm_core::transaction_tester::TransactionTester;
use paradigm_core::wallet_manager::WalletManager;
use std::env;
use std::path::PathBuf;
use tracing_subscriber;

mod network_client;
use network_client::NetworkClient;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("🪙 Paradigm Wallet v2.0.0");
    println!("Advanced Cryptocurrency Wallet CLI");
    println!("===================================");

    let args: Vec<String> = env::args().collect();

    // Get wallet file path
    let wallet_path = if args.len() > 2 && args[1] == "--wallet-file" {
        PathBuf::from(&args[2])
    } else {
        WalletManager::get_default_wallet_path()
    };

    let mut wallet_manager = WalletManager::new(wallet_path.clone())?;
    println!("💼 Using wallet file: {}", wallet_path.display());

    if args.len() > 1 {
        let command_start = if args.len() > 2 && args[1] == "--wallet-file" {
            3
        } else {
            1
        };

        if command_start < args.len() {
            match args[command_start].as_str() {
                "create" => {
                    let label = if args.len() > command_start + 1 {
                        args[command_start + 1].clone()
                    } else {
                        format!("Address {}", chrono::Utc::now().timestamp())
                    };
                    create_address(&mut wallet_manager, &label)?;
                }
                "list" => {
                    list_addresses(&wallet_manager)?;
                }
                "import" => {
                    if args.len() < command_start + 3 {
                        println!("Usage: paradigm-wallet import <private_key_hex> <label>");
                        return Ok(());
                    }
                    let private_key = &args[command_start + 1];
                    let label = &args[command_start + 2];
                    import_private_key(&mut wallet_manager, private_key, label)?;
                }
                "export" => {
                    if args.len() < command_start + 2 {
                        println!("Usage: paradigm-wallet export <address>");
                        return Ok(());
                    }
                    let address = &args[command_start + 1];
                    export_private_key(&wallet_manager, address)?;
                }
                "export-all" => {
                    export_all_keys(&wallet_manager)?;
                }
                "balance" => {
                    if args.len() < command_start + 2 {
                        println!("Usage: paradigm-wallet balance <address>");
                        return Ok(());
                    }
                    let address = &args[command_start + 1];
                    show_balance(&wallet_manager, address)?;
                }
                "info" => {
                    if args.len() < command_start + 2 {
                        show_wallet_summary(&wallet_manager)?;
                    } else {
                        let address = &args[command_start + 1];
                        show_address_info(&wallet_manager, address)?;
                    }
                }
                "test" => {
                    let amount = if args.len() > command_start + 1 {
                        args[command_start + 1].parse().ok()
                    } else {
                        None
                    };
                    let message = if args.len() > command_start + 2 {
                        Some(args[command_start + 2].as_str())
                    } else {
                        None
                    };
                    run_transaction_test(&mut wallet_manager, amount, message).await?;
                }
                "send" => {
                    if args.len() < command_start + 4 {
                        println!(
                            "Usage: paradigm-wallet send <from_address> <to_address> <amount>"
                        );
                        println!("Example: paradigm-wallet send PAR1a2b3c4... PAR5d6e7f8... 0.1");
                        return Ok(());
                    }
                    let from_address = &args[command_start + 1];
                    let to_address = &args[command_start + 2];
                    let amount_str = &args[command_start + 3];
                    let message = if args.len() > command_start + 4 {
                        Some(args[command_start + 4].clone())
                    } else {
                        None
                    };
                    send_transaction(
                        &mut wallet_manager,
                        from_address,
                        to_address,
                        amount_str,
                        message,
                    )
                    .await?;
                }
                "stress-test" => {
                    let count = if args.len() > command_start + 1 {
                        args[command_start + 1].parse().unwrap_or(10)
                    } else {
                        10
                    };
                    run_stress_test(&mut wallet_manager, count).await?;
                }
                "network-status" | "network" | "status" => {
                    check_network_status().await?;
                }
                "help" | "--help" | "-h" => {
                    print_help();
                }
                _ => {
                    println!("❌ Unknown command: {}", args[command_start]);
                    print_help();
                }
            }
        }
    }

    print_help();
    Ok(())
}

fn print_help() {
    println!("\n🪙 Paradigm Wallet CLI Commands");
    println!("=====================================");
    println!("📝 Wallet Management:");
    println!("  create [name]                 Create a new wallet");
    println!("  list                          List all wallets");
    println!("  import <name> <private_key>   Import wallet from private key");
    println!("  export <wallet_name>          Export wallet private key");
    println!("  info <wallet_name>            Show wallet details");
    println!("  set-default <wallet_name>     Set default wallet");
    println!();
    println!("💰 Balance & Transactions:");
    println!("  balance <address_or_wallet>   Check balance");
    println!("  send <from> <to> <amount> [message]  Send PAR tokens");
    println!();
    println!("🧪 Transaction Testing:");
    println!("  test [amount] [message]       Test transaction between addresses");
    println!("  stress-test [count]           Run stress test with multiple transactions");
    println!();
    println!("🌐 Network:");
    println!("  network-status                Check network status and peer connections");
    println!();
    println!("❓ Help:");
    println!("  help                          Show this help message");
    println!();
    println!("Examples:");
    println!("  paradigm-wallet create my_wallet");
    println!("  paradigm-wallet list");
    println!("  paradigm-wallet balance PAR1a2b3c4...");
    println!("  paradigm-wallet send PAR1a2b3c4... PAR5d6e7f8... 0.1");
    println!("  paradigm-wallet send PAR1a2b3c4... PAR5d6e7f8... 0.1 hello");
    println!("  paradigm-wallet test 0.001 hello");
    println!("  paradigm-wallet stress-test 50");
}

fn create_address(wallet_manager: &mut WalletManager, label: &str) -> Result<()> {
    let address = wallet_manager.add_address(label)?;

    println!("✅ Address created successfully!");
    println!("📋 Label: {}", label);
    println!("🏠 Address: {}", address);
    println!("⚠️  Keep your private key secure!");

    Ok(())
}

fn list_addresses(wallet_manager: &WalletManager) -> Result<()> {
    println!("📋 Address List");
    println!("===============");

    let addresses = wallet_manager.list_addresses();
    if addresses.is_empty() {
        println!("💤 No addresses found. Create one with: paradigm-wallet create <label>");
        return Ok(());
    }

    for (address, entry) in addresses {
        let is_default = wallet_manager.get_default_address().as_deref() == Some(address);
        let marker = if is_default { "⭐" } else { "  " };

        println!("{} 🪙 {}", marker, entry.label);
        println!("     📍 {}", entry.address);
        println!(
            "     💰 Balance: {:.8} PAR",
            entry.balance as f64 / 100_000_000.0
        );
        println!(
            "     📅 Created: {}",
            chrono::DateTime::from_timestamp(entry.created_at as i64, 0)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S")
        );
        println!();
    }

    if let Some(default) = &wallet_manager.get_default_address() {
        if let Some(entry) = wallet_manager.get_address_info(default) {
            println!("⭐ Default address: {} ({})", entry.label, default);
        }
    }

    Ok(())
}

fn import_private_key(
    wallet_manager: &mut WalletManager,
    private_key: &str,
    label: &str,
) -> Result<()> {
    println!("📥 Importing private key: {}", label);

    let address = wallet_manager.import_private_key(private_key, label)?;

    println!("✅ Private key imported successfully!");
    println!("📋 Label: {}", label);
    println!("🏠 Address: {}", address);

    Ok(())
}

fn export_private_key(wallet_manager: &WalletManager, address: &str) -> Result<()> {
    if let Some(entry) = wallet_manager.get_address_info(address) {
        println!("🔐 Exporting private key for: {}", entry.label);
        println!("⚠️  WARNING: Keep this private key secure!");
        println!("🔑 Private Key: {}", entry.private_key_hex);
        println!("🏠 Address: {}", entry.address);
    } else {
        println!("❌ Address '{}' not found!", address);
        println!("💡 Use 'paradigm-wallet list' to see available addresses");
    }

    Ok(())
}

fn export_all_keys(wallet_manager: &WalletManager) -> Result<()> {
    println!("🔐 Exporting all private keys");
    println!("⚠️  WARNING: Keep these private keys secure!");
    println!("=======================================");

    let keys = wallet_manager.export_keys();
    if keys.is_empty() {
        println!("💤 No addresses found.");
        return Ok(());
    }

    for (address, private_key, label) in keys {
        println!("📋 Label: {}", label);
        println!("🏠 Address: {}", address);
        println!("🔑 Private Key: {}", private_key);
        println!();
    }

    Ok(())
}

fn show_balance(wallet_manager: &WalletManager, address: &str) -> Result<()> {
    if let Some(entry) = wallet_manager.get_address_info(address) {
        println!("💰 Balance for: {} ({})", entry.label, entry.address);
        println!(
            "💸 Balance: {:.8} PAR",
            entry.balance as f64 / 100_000_000.0
        );
        println!(
            "🏆 Total Earned: {:.8} PAR",
            entry.total_earned as f64 / 100_000_000.0
        );
        println!("📋 Tasks Completed: {}", entry.tasks_completed);
    } else {
        println!("❌ Address '{}' not found!", address);
        println!("💡 Use 'paradigm-wallet list' to see available addresses");
    }

    Ok(())
}

fn show_wallet_summary(wallet_manager: &WalletManager) -> Result<()> {
    println!("🪙 Wallet Summary");
    println!("==================");

    let addresses = wallet_manager.list_addresses();
    if addresses.is_empty() {
        println!("💤 No addresses found. Create one with: paradigm-wallet create <label>");
        return Ok(());
    }

    let mut total_balance = 0u64;
    let mut total_earned = 0u64;
    let mut total_tasks = 0u64;

    for (_, entry) in &addresses {
        total_balance += entry.balance;
        total_earned += entry.total_earned;
        total_tasks += entry.tasks_completed;
    }

    println!("📊 Total Addresses: {}", addresses.len());
    println!(
        "💰 Total Balance: {:.8} PAR",
        total_balance as f64 / 100_000_000.0
    );
    println!(
        "🏆 Total Earned: {:.8} PAR",
        total_earned as f64 / 100_000_000.0
    );
    println!("📋 Tasks Completed: {}", total_tasks);

    if let Some(default) = &wallet_manager.get_default_address() {
        if let Some(entry) = wallet_manager.get_address_info(default) {
            println!("⭐ Default Address: {} ({})", entry.label, default);
        }
    }

    Ok(())
}

fn show_address_info(wallet_manager: &WalletManager, address: &str) -> Result<()> {
    if let Some(entry) = wallet_manager.get_address_info(address) {
        let is_default = wallet_manager.get_default_address().as_deref() == Some(address);

        println!("🪙 Address Information");
        println!("======================");
        println!("📋 Label: {}", entry.label);
        println!("🏠 Address: {}", entry.address);
        println!(
            "💰 Balance: {:.8} PAR",
            entry.balance as f64 / 100_000_000.0
        );
        println!(
            "🏆 Total Earned: {:.8} PAR",
            entry.total_earned as f64 / 100_000_000.0
        );
        println!("📋 Tasks Completed: {}", entry.tasks_completed);
        println!(
            "📅 Created: {}",
            chrono::DateTime::from_timestamp(entry.created_at as i64, 0)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S")
        );
        println!("⭐ Default: {}", if is_default { "Yes" } else { "No" });

        // Show abbreviated private key for security
        let abbreviated_key = format!(
            "{}...{}",
            &entry.private_key_hex[..8],
            &entry.private_key_hex[entry.private_key_hex.len() - 8..]
        );
        println!(
            "🔑 Private Key: {} (use 'export {}' to see full key)",
            abbreviated_key, address
        );
    } else {
        println!("❌ Address '{}' not found!", address);
        println!("💡 Use 'paradigm-wallet list' to see available addresses");
    }

    Ok(())
}

async fn check_network_status() -> Result<()> {
    println!("🌐 Paradigm Network Status Check");
    println!("================================");

    // Try different common ports
    let ports_to_check = vec![8080, 8081, 8082, 8083];
    let mut found_any = false;

    for port in ports_to_check {
        let node_url = format!("http://127.0.0.1:{}", port);
        println!("\n🔍 Checking node at {}...", node_url);

        match NetworkClient::new(&node_url).await {
            Ok(mut client) => match client.connect().await {
                Ok(()) => {
                    found_any = true;
                    println!("✅ Connected to node on port {}", port);

                    match client.get_comprehensive_network_status().await {
                        Ok(status) => {
                            println!("📊 Network Status:");
                            println!(
                                "   Node URL: {}",
                                status["node_url"].as_str().unwrap_or("unknown")
                            );
                            println!(
                                "   Health: {}",
                                status["health"]["status"].as_str().unwrap_or("unknown")
                            );
                            println!(
                                "   Network Active: {}",
                                if status["network_active"].as_bool().unwrap_or(false) {
                                    "✅ Yes"
                                } else {
                                    "❌ No"
                                }
                            );

                            let peer_count =
                                status["peer_info"]["peer_count"].as_u64().unwrap_or(0);
                            let block_height =
                                status["peer_info"]["block_height"].as_u64().unwrap_or(0);
                            let is_synchronized = status["peer_info"]["is_synchronized"]
                                .as_bool()
                                .unwrap_or(false);

                            println!("   Connected Peers: {}", peer_count);
                            println!("   Block Height: {}", block_height);
                            println!(
                                "   Synchronized: {}",
                                if is_synchronized {
                                    "✅ Yes"
                                } else {
                                    "⚠️ No peers (isolated node)"
                                }
                            );

                            let available_tasks =
                                status["tasks"]["available_count"].as_u64().unwrap_or(0);
                            let queue_size = status["tasks"]["queue_size"].as_u64().unwrap_or(0);
                            let estimated_reward =
                                status["tasks"]["estimated_reward"].as_u64().unwrap_or(0);

                            println!("   Available Tasks: {}", available_tasks);
                            println!("   Task Queue Size: {}", queue_size);
                            println!(
                                "   Total Rewards: {:.8} PAR",
                                estimated_reward as f64 / 100_000_000.0
                            );

                            if peer_count == 0 {
                                println!("⚠️  WARNING: This node has no peer connections!");
                                println!("   It's running in isolation. For a proper network:");
                                println!("   1. Start multiple nodes on different ports");
                                println!("   2. Configure peer discovery between nodes");
                            }
                        }
                        Err(e) => {
                            println!("⚠️  Could not get detailed status: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Connection failed: {}", e);
                }
            },
            Err(e) => {
                println!("❌ Could not create client: {}", e);
            }
        }
    }

    if !found_any {
        println!("\n❌ No Paradigm nodes found!");
        println!(
            "💡 Start a node with: target/release/paradigm-core.exe --enable-api --api-port 8080"
        );
    } else {
        println!("\n✅ Network status check complete");
    }

    Ok(())
}

async fn run_transaction_test(
    wallet_manager: &mut WalletManager,
    amount: Option<f64>,
    message: Option<&str>,
) -> Result<()> {
    println!("🧪 Starting Transaction Test...");
    println!("================================");

    let mut tester = TransactionTester::new();
    let result = tester
        .run_wallet_transaction_test(wallet_manager, amount, message)
        .await?;

    println!("📊 Test Summary:");
    println!("Success Rate: {:.1}%", tester.get_success_rate());

    Ok(())
}

async fn run_stress_test(wallet_manager: &mut WalletManager, count: usize) -> Result<()> {
    println!("🏋️ Starting Stress Test...");
    println!("===========================");
    println!("Running {} transactions...", count);

    let mut tester = TransactionTester::new();
    let results = tester.run_stress_test(wallet_manager, count, false).await?;

    println!("🎯 Final Results:");
    println!("Success Rate: {:.1}%", tester.get_success_rate());
    println!("Total Tests: {}", results.len());

    Ok(())
}

async fn send_transaction(
    wallet_manager: &mut WalletManager,
    from_address: &str,
    to_address: &str,
    amount_str: &str,
    message: Option<String>,
) -> Result<()> {
    use chrono::Utc;
    use ed25519_dalek::{Signer, SigningKey};
    use paradigm_core::{transaction::Transaction, Address, Amount};
    use uuid::Uuid;

    println!("💸 Sending Transaction...");
    println!("=========================");

    // Parse amount - convert from PAR to smallest unit (8 decimals)
    let amount_par: f64 = amount_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid amount format. Use decimal format like 0.1"))?;

    if amount_par <= 0.0 {
        return Err(anyhow::anyhow!("Amount must be greater than 0"));
    }

    let amount: Amount = (amount_par * 100_000_000.0) as u64;

    // Verify sender address exists in wallet
    let sender_info = wallet_manager
        .get_address_info(from_address)
        .ok_or_else(|| anyhow::anyhow!("Sender address '{}' not found in wallet", from_address))?;

    // Calculate dynamic AI-driven fee
    let estimated_fee = calculate_estimated_fee(amount, false).await?; // Not urgent by default

    // Check if sender has sufficient balance
    if sender_info.balance < amount + estimated_fee {
        return Err(anyhow::anyhow!(
            "Insufficient balance. Available: {:.8} PAR, Required: {:.8} PAR (including {:.8} PAR dynamic fee)",
            sender_info.balance as f64 / 100_000_000.0,
            (amount + estimated_fee) as f64 / 100_000_000.0,
            estimated_fee as f64 / 100_000_000.0
        ));
    }

    // Parse addresses
    let from_addr = Address::from_string(from_address)?;
    let to_addr = Address::from_string(to_address)?;

    // Get or create private key for sender
    let private_key_hex = &sender_info.private_key_hex;
    let private_key_bytes =
        hex::decode(private_key_hex).map_err(|_| anyhow::anyhow!("Invalid private key format"))?;

    let mut key_bytes = [0u8; 32];
    if private_key_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Private key must be exactly 32 bytes"));
    }
    key_bytes.copy_from_slice(&private_key_bytes);

    let signing_key = SigningKey::from_bytes(&key_bytes);

    // Validate message length (10 characters max)
    if let Some(ref msg) = message {
        if msg.len() > 10 {
            return Err(anyhow::anyhow!("Message must be 10 characters or less"));
        }
    }

    // Create transaction
    let mut transaction = Transaction {
        id: Uuid::new_v4(),
        from: from_addr,
        to: to_addr,
        amount,
        fee: estimated_fee, // AI-calculated dynamic fee
        timestamp: Utc::now(),
        signature: Vec::new(),
        nonce: sender_info.tasks_completed + 1, // Use tasks_completed + 1 as nonce
        message: message.clone(),
    };

    // Sign the transaction
    let transaction_bytes = serde_json::to_vec(&transaction)?;
    let signature = signing_key.sign(&transaction_bytes);
    transaction.signature = signature.to_bytes().to_vec();

    println!("📋 Transaction Details:");
    println!("  From: {} ({})", sender_info.label, from_address);
    println!("  To: {}", to_address);
    println!("  Amount: {:.8} PAR", amount as f64 / 100_000_000.0);
    println!(
        "  Fee: {:.8} PAR (AI-calculated)",
        estimated_fee as f64 / 100_000_000.0
    );
    println!(
        "  Total: {:.8} PAR",
        (amount + estimated_fee) as f64 / 100_000_000.0
    );
    if let Some(ref msg) = message {
        println!("  Message: {}", msg);
    }
    println!("  Transaction ID: {}", transaction.id);

    // Submit transaction to network
    println!("🌐 Connecting to network...");

    match NetworkClient::new("127.0.0.1:8080").await {
        Ok(mut client) => {
            match client.connect().await {
                Ok(()) => {
                    println!("✅ Connected to network");
                    println!("📡 Broadcasting transaction...");

                    match client.broadcast_transaction(&transaction).await {
                        Ok(tx_id) => {
                            println!("🎉 Transaction broadcast successful!");
                            println!("📝 Network Transaction ID: {}", tx_id);

                            // Update sender balance in wallet
                            wallet_manager.update_address_balance(
                                from_address,
                                sender_info.balance - amount - estimated_fee,
                            )?;
                            println!("💰 Wallet balance updated");
                        }
                        Err(e) => {
                            println!("⚠️  Network broadcast failed: {}", e);
                            println!("💾 Transaction created and signed locally");
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  Could not connect to network: {}", e);
                    println!("💾 Transaction created and signed locally");
                }
            }
        }
        Err(e) => {
            println!("⚠️  Network client error: {}", e);
            println!("💾 Transaction created and signed locally");
        }
    }

    println!("✅ Transaction created and signed successfully!");
    println!("🔐 Signature: {}", hex::encode(&transaction.signature));

    Ok(())
}

/// Calculate estimated fee using simplified AI-driven logic when not connected to full node
async fn calculate_estimated_fee(
    amount: paradigm_core::Amount,
    urgent: bool,
) -> Result<paradigm_core::Amount> {
    // Default AI governance parameters (fallback when not connected to node)
    let min_fee_percentage = 0.001; // 0.1%
    let max_fee_percentage = 0.05; // 5%

    // Base fee calculation
    let mut base_fee_percentage = min_fee_percentage;

    // Amount-based adjustments
    if amount > 1000_00000000 {
        // Large transactions (>1000 PAR) get slightly higher base fee
        base_fee_percentage *= 1.2;
    } else if amount < 1_00000000 {
        // Small transactions (<1 PAR) get reduced base fee to encourage micro-transactions
        base_fee_percentage *= 0.5;
    }

    // Simulate network congestion (in production, this would come from node metrics)
    let network_congestion = 0.2; // Assume 20% congestion
    let congestion_adjustment = network_congestion * 0.1; // Fee sensitivity

    // Urgent transactions pay premium
    let urgency_multiplier = if urgent { 2.0 } else { 1.0 };

    // Near-zero fee optimization for small amounts
    let near_zero_threshold = 10_00000000; // 10 PAR
    let near_zero_reduction = if amount < near_zero_threshold {
        // Progressive reduction for amounts under 10 PAR
        let reduction_factor = (near_zero_threshold - amount) as f64 / near_zero_threshold as f64;
        0.5 * reduction_factor // Up to 50% reduction
    } else {
        0.0
    };

    // Calculate final fee percentage
    let final_fee_percentage = ((base_fee_percentage + congestion_adjustment) * urgency_multiplier
        - near_zero_reduction)
        .max(0.0001) // Minimum 0.01%
        .min(max_fee_percentage);

    let calculated_fee = ((amount as f64) * final_fee_percentage) as paradigm_core::Amount;

    // Absolute minimum fee to prevent spam (but very small)
    let absolute_minimum = 10_000; // 0.0001 PAR
    let final_fee = calculated_fee.max(absolute_minimum);

    println!("💡 Fee calculation:");
    println!("   Base rate: {:.4}%", base_fee_percentage * 100.0);
    println!("   Network congestion: {:.1}%", network_congestion * 100.0);
    println!("   Urgency multiplier: {:.1}x", urgency_multiplier);
    if near_zero_reduction > 0.0 {
        println!(
            "   Near-zero optimization: -{:.4}%",
            near_zero_reduction * 100.0
        );
    }
    println!("   Final rate: {:.4}%", final_fee_percentage * 100.0);

    Ok(final_fee)
}
