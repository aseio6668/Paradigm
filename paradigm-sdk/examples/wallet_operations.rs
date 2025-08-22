//! Wallet operations example
//!
//! This example demonstrates advanced wallet functionality:
//! - Multi-signature wallets
//! - Hardware wallet integration  
//! - Wallet import/export
//! - Transaction signing

use anyhow::Result;
use paradigm_sdk::client::ParadigmClient;
use paradigm_sdk::types::{Address, Amount, PrivateKey};
use paradigm_sdk::wallet::{HardwareWallet, MultiSigWallet, SDKWallet};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🪙 Paradigm Wallet Operations Example");
    println!("====================================");

    // Create standard wallet
    println!("\n1️⃣ Creating standard wallet...");
    let wallet1 = SDKWallet::new()?;
    println!("   Address: {}", wallet1.get_address());

    // Import wallet from private key
    println!("\n2️⃣ Importing wallet from private key...");
    let private_key = PrivateKey::generate();
    let wallet2 = SDKWallet::from_private_key(private_key)?;
    println!("   Address: {}", wallet2.get_address());

    // Create multi-signature wallet
    println!("\n3️⃣ Creating multi-sig wallet...");
    let signers = vec![wallet1.get_public_key(), wallet2.get_public_key()];
    let multisig = MultiSigWallet::new(2, signers)?; // 2-of-2 multisig
    println!("   Multi-sig address: {}", multisig.get_address());

    // Connect to network
    let client = ParadigmClient::new("http://127.0.0.1:8080").await?;

    // Example: Sign transaction with multiple wallets
    println!("\n4️⃣ Creating multi-sig transaction...");
    let to_address = Address::from_string("PAR1recipient...")?;
    let amount = Amount::from_par(1.0);

    let mut tx_builder = client
        .create_transaction_builder()
        .from(&multisig.get_address())
        .to(&to_address)
        .amount(amount);

    // Sign with first wallet
    tx_builder = wallet1.sign_transaction(tx_builder)?;

    // Sign with second wallet
    tx_builder = wallet2.sign_transaction(tx_builder)?;

    // Build final transaction
    let signed_tx = tx_builder.build()?;

    println!("   ✅ Multi-sig transaction created");
    println!("   Transaction hash: {}", signed_tx.get_hash());

    // Hardware wallet example (mock implementation)
    println!("\n5️⃣ Hardware wallet integration...");
    match HardwareWallet::detect().await {
        Ok(hw_wallet) => {
            println!("   📱 Hardware wallet detected: {}", hw_wallet.get_model());
            println!("   Address: {}", hw_wallet.get_address(0)?);
        }
        Err(_) => {
            println!("   ⚠️  No hardware wallet detected");
        }
    }

    println!("\n✅ Wallet operations example completed!");

    Ok(())
}
