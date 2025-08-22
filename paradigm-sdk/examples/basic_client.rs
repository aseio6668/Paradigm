//! Basic client example demonstrating core Paradigm SDK functionality
//!
//! This example shows how to:
//! - Connect to a Paradigm network node
//! - Create and manage wallets
//! - Send transactions
//! - Query balances

use anyhow::Result;
use paradigm_sdk::client::ParadigmClient;
use paradigm_sdk::types::{Address, Amount};
use paradigm_sdk::wallet::SDKWallet;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the Paradigm client
    println!("🔗 Connecting to Paradigm network...");
    let client = ParadigmClient::new("http://127.0.0.1:8080").await?;

    // Create a new wallet
    println!("🪙 Creating wallet...");
    let wallet = SDKWallet::new()?;
    let address = wallet.get_address();

    println!("✅ Wallet created!");
    println!("📍 Address: {}", address);

    // Check balance
    println!("💰 Checking balance...");
    let balance = client.get_balance(&address).await?;
    println!("💸 Balance: {} PAR", balance);

    // Example of creating a transaction (commented out for safety)
    /*
    if balance > Amount::from_par(1.0) {
        let to_address = Address::from_string("PAR1example...")?;
        let amount = Amount::from_par(0.5);

        println!("📤 Sending transaction...");
        let tx_hash = client.send_transaction(&wallet, &to_address, amount).await?;
        println!("✅ Transaction sent: {}", tx_hash);
    }
    */

    Ok(())
}
