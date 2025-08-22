//! Smart contract interaction example
//!
//! This example demonstrates how to:
//! - Deploy smart contracts
//! - Call contract functions
//! - Handle contract events

use anyhow::Result;
use paradigm_sdk::client::ParadigmClient;
use paradigm_sdk::contracts::{ContractBuilder, ContractInstance};
use paradigm_sdk::wallet::SDKWallet;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔗 Connecting to Paradigm network...");
    let client = ParadigmClient::new("http://127.0.0.1:8080").await?;

    // Create wallet for contract deployment
    let wallet = SDKWallet::new()?;
    println!("🪙 Using wallet: {}", wallet.get_address());

    // Example contract bytecode (mock)
    let contract_code = include_bytes!("../test-contracts/simple_storage.wasm");

    println!("📜 Deploying smart contract...");
    let contract_address = client.deploy_contract(&wallet, contract_code, &[]).await?;

    println!("✅ Contract deployed at: {}", contract_address);

    // Create contract instance
    let contract = ContractInstance::new(client.clone(), contract_address);

    // Call contract function
    println!("📞 Calling contract function...");
    let result = contract.call_function("getValue", &[]).await?;

    println!("📋 Contract returned: {:?}", result);

    // Send transaction to contract
    println!("📤 Sending transaction to contract...");
    let tx_hash = contract
        .send_transaction(&wallet, "setValue", &[42u64.into()])
        .await?;

    println!("✅ Transaction sent: {}", tx_hash);

    Ok(())
}
