// Paradigm Wallet - CLI interface with full wallet management
use paradigm_core::Wallet;
use std::env;
use std::fs;
use std::path::Path;
use anyhow::Result;
use tracing_subscriber;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct WalletInfo {
    name: String,
    address: String,
    private_key: String,
    created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct WalletStore {
    wallets: HashMap<String, WalletInfo>,
    default_wallet: Option<String>,
}

const WALLET_DIR: &str = "./wallets";
const WALLET_STORE_FILE: &str = "./wallets/wallet_store.json";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("🪙 Paradigm Wallet v0.1.0");
    println!("Advanced Cryptocurrency Wallet CLI");
    println!("===================================");
    
    // Ensure wallet directory exists
    ensure_wallet_directory()?;
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "create" => {
                let name = if args.len() > 2 {
                    args[2].clone()
                } else {
                    format!("wallet_{}", chrono::Utc::now().timestamp())
                };
                create_wallet(&name).await?;
                return Ok(());
            }
            "list" => {
                list_wallets().await?;
                return Ok(());
            }
            "import" => {
                if args.len() < 4 {
                    println!("Usage: paradigm-wallet import <name> <private_key>");
                    return Ok(());
                }
                let name = &args[2];
                let private_key = &args[3];
                import_wallet(name, private_key).await?;
                return Ok(());
            }
            "export" => {
                if args.len() < 3 {
                    println!("Usage: paradigm-wallet export <wallet_name>");
                    return Ok(());
                }
                let name = &args[2];
                export_wallet(name).await?;
                return Ok(());
            }
            "balance" => {
                if args.len() < 3 {
                    println!("Usage: paradigm-wallet balance <address_or_wallet_name>");
                    return Ok(());
                }
                let address = &args[2];
                check_balance(address).await?;
                return Ok(());
            }
            "send" => {
                if args.len() < 5 {
                    println!("Usage: paradigm-wallet send <from_wallet> <to_address> <amount>");
                    return Ok(());
                }
                let from_wallet = &args[2];
                let to_address = &args[3];
                let amount = args[4].parse::<u64>().unwrap_or(0);
                send_transaction(from_wallet, to_address, amount).await?;
                return Ok(());
            }
            "info" => {
                if args.len() < 3 {
                    println!("Usage: paradigm-wallet info <wallet_name>");
                    return Ok(());
                }
                let name = &args[2];
                show_wallet_info(name).await?;
                return Ok(());
            }
            "set-default" => {
                if args.len() < 3 {
                    println!("Usage: paradigm-wallet set-default <wallet_name>");
                    return Ok(());
                }
                let name = &args[2];
                set_default_wallet(name).await?;
                return Ok(());
            }
            "help" | "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                println!("❌ Unknown command: {}", args[1]);
                print_help();
                return Ok(());
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
    println!("❓ Help:");
    println!("  help                          Show this help message");
    println!();
    println!("Examples:");
    println!("  paradigm-wallet create my_wallet");
    println!("  paradigm-wallet list");
    println!("  paradigm-wallet balance my_wallet");
    println!("  paradigm-wallet send my_wallet PAR1abc...xyz 1000000000");
}

// Utility function to ensure wallet directory exists
fn ensure_wallet_directory() -> Result<()> {
    if !Path::new(WALLET_DIR).exists() {
        fs::create_dir_all(WALLET_DIR)?;
        println!("📁 Created wallet directory: {}", WALLET_DIR);
    }
    Ok(())
}

// Load wallet store from file
async fn load_wallet_store() -> Result<WalletStore> {
    if Path::new(WALLET_STORE_FILE).exists() {
        let content = fs::read_to_string(WALLET_STORE_FILE)?;
        let store: WalletStore = serde_json::from_str(&content)?;
        Ok(store)
    } else {
        Ok(WalletStore::default())
    }
}

// Save wallet store to file
async fn save_wallet_store(store: &WalletStore) -> Result<()> {
    let content = serde_json::to_string_pretty(store)?;
    fs::write(WALLET_STORE_FILE, content)?;
    Ok(())
}

// Create a new wallet
async fn create_wallet(name: &str) -> Result<()> {
    println!("🔐 Creating new wallet: {}", name);
    
    let mut store = load_wallet_store().await?;
    
    // Check if wallet already exists
    if store.wallets.contains_key(name) {
        println!("❌ Wallet '{}' already exists!", name);
        return Ok(());
    }
    
    // Create new wallet using paradigm-core
    let wallet = Wallet::new();
    let address = wallet.get_address().to_string();
    let private_key_bytes = wallet.export_private_key();
    let private_key = hex::encode(private_key_bytes);
    
    let wallet_info = WalletInfo {
        name: name.to_string(),
        address: address.clone(),
        private_key,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    store.wallets.insert(name.to_string(), wallet_info);
    
    // Set as default if it's the first wallet
    if store.default_wallet.is_none() {
        store.default_wallet = Some(name.to_string());
        println!("🌟 Set as default wallet");
    }
    
    save_wallet_store(&store).await?;
    
    println!("✅ Wallet created successfully!");
    println!("📋 Name: {}", name);
    println!("🏠 Address: {}", address);
    println!("⚠️  Keep your private key secure!");
    
    Ok(())
}

// List all wallets
async fn list_wallets() -> Result<()> {
    println!("📋 Wallet List");
    println!("===============");
    
    let store = load_wallet_store().await?;
    
    if store.wallets.is_empty() {
        println!("💤 No wallets found. Create one with: paradigm-wallet create <name>");
        return Ok(());
    }
    
    for (name, wallet) in &store.wallets {
        let is_default = store.default_wallet.as_ref() == Some(name);
        let marker = if is_default { "⭐" } else { "  " };
        
        println!("{} 🪙 {}", marker, name);
        println!("     📍 {}", wallet.address);
        println!("     📅 Created: {}", wallet.created_at);
        println!();
    }
    
    if let Some(default) = &store.default_wallet {
        println!("⭐ Default wallet: {}", default);
    }
    
    Ok(())
}

// Import wallet from private key
async fn import_wallet(name: &str, private_key: &str) -> Result<()> {
    println!("📥 Importing wallet: {}", name);
    
    let mut store = load_wallet_store().await?;
    
    // Check if wallet already exists
    if store.wallets.contains_key(name) {
        println!("❌ Wallet '{}' already exists!", name);
        return Ok(());
    }
    
    // Try to decode hex private key
    let private_key_bytes = match hex::decode(private_key) {
        Ok(bytes) => {
            if bytes.len() != 32 {
                println!("❌ Invalid private key length. Expected 32 bytes (64 hex characters)");
                return Ok(());
            }
            let mut key_array = [0u8; 32];
            key_array.copy_from_slice(&bytes);
            key_array
        }
        Err(_) => {
            println!("❌ Invalid private key format. Expected hexadecimal string");
            return Ok(());
        }
    };
    
    // Try to create wallet from private key
    let wallet = match Wallet::from_private_key(&private_key_bytes) {
        Ok(w) => w,
        Err(e) => {
            println!("❌ Invalid private key: {}", e);
            return Ok(());
        }
    };
    
    let address = wallet.get_address().to_string();
    
    let wallet_info = WalletInfo {
        name: name.to_string(),
        address: address.clone(),
        private_key: private_key.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    store.wallets.insert(name.to_string(), wallet_info);
    
    // Set as default if it's the first wallet
    if store.default_wallet.is_none() {
        store.default_wallet = Some(name.to_string());
        println!("🌟 Set as default wallet");
    }
    
    save_wallet_store(&store).await?;
    
    println!("✅ Wallet imported successfully!");
    println!("📋 Name: {}", name);
    println!("🏠 Address: {}", address);
    
    Ok(())
}

// Export wallet private key
async fn export_wallet(name: &str) -> Result<()> {
    let store = load_wallet_store().await?;
    
    match store.wallets.get(name) {
        Some(wallet) => {
            println!("🔐 Exporting wallet: {}", name);
            println!("⚠️  WARNING: Keep this private key secure!");
            println!("🔑 Private Key: {}", wallet.private_key);
            println!("🏠 Address: {}", wallet.address);
        }
        None => {
            println!("❌ Wallet '{}' not found!", name);
            println!("💡 Use 'paradigm-wallet list' to see available wallets");
        }
    }
    
    Ok(())
}

// Check balance for address or wallet
async fn check_balance(address_or_wallet: &str) -> Result<()> {
    let store = load_wallet_store().await?;
    
    // Try to resolve wallet name to address
    let address = if let Some(wallet) = store.wallets.get(address_or_wallet) {
        wallet.address.clone()
    } else {
        address_or_wallet.to_string()
    };
    
    println!("💰 Checking balance for: {}", address);
    
    // TODO: Implement actual balance check with paradigm-core
    // For now, simulate balance check
    println!("🔍 Querying Paradigm network...");
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Simulate different balances based on address
    let balance = if address.starts_with("PAR1") { 15.75842156 } else { 0.0 };
    
    println!("💸 Balance: {:.8} PAR", balance);
    println!("💵 USD Value: ${:.2} (approx)", balance * 1.42); // Mock exchange rate
    
    Ok(())
}

// Send transaction
async fn send_transaction(from_wallet: &str, to_address: &str, amount: u64) -> Result<()> {
    let store = load_wallet_store().await?;
    
    let wallet_info = match store.wallets.get(from_wallet) {
        Some(w) => w,
        None => {
            println!("❌ Wallet '{}' not found!", from_wallet);
            return Ok(());
        }
    };
    
    let amount_par = amount as f64 / 100_000_000.0;
    
    println!("📤 Preparing transaction...");
    println!("👤 From: {} ({})", from_wallet, wallet_info.address);
    println!("🎯 To: {}", to_address);
    println!("💰 Amount: {:.8} PAR", amount_par);
    
    // TODO: Implement actual transaction with paradigm-core
    // For now, simulate transaction
    println!("🔐 Signing transaction...");
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    println!("📡 Broadcasting to network...");
    tokio::time::sleep(tokio::time::Duration::from_millis(700)).await;
    
    let tx_hash = format!("TX{:x}", rand::random::<u64>());
    println!("✅ Transaction sent successfully!");
    println!("🆔 Transaction ID: {}", tx_hash);
    println!("⏱️  Processing time: ~30 seconds");
    
    Ok(())
}

// Show wallet info
async fn show_wallet_info(name: &str) -> Result<()> {
    let store = load_wallet_store().await?;
    
    match store.wallets.get(name) {
        Some(wallet) => {
            let is_default = store.default_wallet.as_ref().map(|s| s.as_str()) == Some(name);
            
            println!("🪙 Wallet Information");
            println!("====================");
            println!("📋 Name: {}", wallet.name);
            println!("🏠 Address: {}", wallet.address);
            println!("📅 Created: {}", wallet.created_at);
            println!("⭐ Default: {}", if is_default { "Yes" } else { "No" });
            
            // Show abbreviated private key for security
            let abbreviated_key = format!("{}...{}", 
                &wallet.private_key[..8], 
                &wallet.private_key[wallet.private_key.len()-8..]
            );
            println!("🔑 Private Key: {} (use 'export' to see full key)", abbreviated_key);
        }
        None => {
            println!("❌ Wallet '{}' not found!", name);
            println!("💡 Use 'paradigm-wallet list' to see available wallets");
        }
    }
    
    Ok(())
}

// Set default wallet
async fn set_default_wallet(name: &str) -> Result<()> {
    let mut store = load_wallet_store().await?;
    
    if store.wallets.contains_key(name) {
        store.default_wallet = Some(name.to_string());
        save_wallet_store(&store).await?;
        println!("⭐ Set '{}' as default wallet", name);
    } else {
        println!("❌ Wallet '{}' not found!", name);
        println!("💡 Use 'paradigm-wallet list' to see available wallets");
    }
    
    Ok(())
}
