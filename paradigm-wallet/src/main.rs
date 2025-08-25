// Paradigm Wallet - CLI interface with full wallet management
use anyhow::Result;
use paradigm_core::wallet_manager::WalletManager;
use paradigm_core::transaction_tester::TransactionTester;
use std::env;
use std::path::PathBuf;
use tracing_subscriber;


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
        let command_start = if args.len() > 2 && args[1] == "--wallet-file" { 3 } else { 1 };
        
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
                "stress-test" => {
                    let count = if args.len() > command_start + 1 {
                        args[command_start + 1].parse().unwrap_or(10)
                    } else {
                        10
                    };
                    run_stress_test(&mut wallet_manager, count).await?;
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
    println!("  send <from> <to> <amount>     Send PAR tokens");
    println!();
    println!("🧪 Transaction Testing:");
    println!("  test [amount] [message]       Test transaction between addresses");
    println!("  stress-test [count]           Run stress test with multiple transactions");
    println!();
    println!("❓ Help:");
    println!("  help                          Show this help message");
    println!();
    println!("Examples:");
    println!("  paradigm-wallet create my_wallet");
    println!("  paradigm-wallet list");
    println!("  paradigm-wallet balance my_wallet");
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
        println!("     💰 Balance: {:.8} PAR", entry.balance as f64 / 100_000_000.0);
        println!("     📅 Created: {}", 
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

fn import_private_key(wallet_manager: &mut WalletManager, private_key: &str, label: &str) -> Result<()> {
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
        println!("💸 Balance: {:.8} PAR", entry.balance as f64 / 100_000_000.0);
        println!("🏆 Total Earned: {:.8} PAR", entry.total_earned as f64 / 100_000_000.0);
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
    println!("💰 Total Balance: {:.8} PAR", total_balance as f64 / 100_000_000.0);
    println!("🏆 Total Earned: {:.8} PAR", total_earned as f64 / 100_000_000.0);
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
        println!("💰 Balance: {:.8} PAR", entry.balance as f64 / 100_000_000.0);
        println!("🏆 Total Earned: {:.8} PAR", entry.total_earned as f64 / 100_000_000.0);
        println!("📋 Tasks Completed: {}", entry.tasks_completed);
        println!("📅 Created: {}", 
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
            abbreviated_key,
            address
        );
    } else {
        println!("❌ Address '{}' not found!", address);
        println!("💡 Use 'paradigm-wallet list' to see available addresses");
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
    let result = tester.run_wallet_transaction_test(wallet_manager, amount, message).await?;
    
    println!("📊 Test Summary:");
    println!("Success Rate: {:.1}%", tester.get_success_rate());
    
    Ok(())
}

async fn run_stress_test(
    wallet_manager: &mut WalletManager,
    count: usize,
) -> Result<()> {
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
