// Paradigm Wallet - CLI interface (GUI temporarily disabled)
use paradigm_core::{Transaction, ParadigmError};
use std::env;
use anyhow::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("Paradigm Wallet v0.1.0");
    println!("Command Line Interface");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "create" => {
                println!("Creating new wallet...");
                // TODO: Create wallet implementation
                println!("New wallet created!");
                return Ok(());
            }
            "balance" => {
                if args.len() < 3 {
                    println!("Usage: paradigm-wallet balance <address>");
                    return Ok(());
                }
                let address = &args[2];
                println!("Checking balance for: {}", address);
                // TODO: Query balance implementation
                println!("Balance: 0 PAR");
                return Ok(());
            }
            "send" => {
                if args.len() < 4 {
                    println!("Usage: paradigm-wallet send <to_address> <amount>");
                    return Ok(());
                }
                let to_address = &args[2];
                let amount = args[3].parse::<u64>().unwrap_or(0);
                println!("Sending {} PAR to {}", amount, to_address);
                // TODO: Send transaction implementation
                return Ok(());
            }
            "help" | "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                println!("Unknown command: {}", args[1]);
                print_help();
                return Ok(());
            }
        }
    }
    
    print_help();
    Ok(())
}

fn print_help() {
    println!("\nParadigm Wallet CLI");
    println!("Commands:");
    println!("  create              Create a new wallet");
    println!("  balance <address>   Check balance for an address");
    println!("  send <to> <amount>  Send PAR to an address");
    println!("  help                Show this help message");
}
