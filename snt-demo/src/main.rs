use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use inquire::{Select, Text, Confirm};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

mod snt;
mod simulation;
mod display;

use snt::*;
use simulation::*;
use display::*;

#[derive(Parser)]
#[command(name = "snt-demo")]
#[command(about = "Paradigm Symbolic Network Token Demo")]
#[command(long_about = "Experience the revolutionary SNT system - where tokens unlock living functionality instead of dead commodities")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive demonstration of the complete SNT ecosystem
    Interactive,
    /// Quick showcase of key SNT features
    Showcase,
    /// Show network statistics and SNT analytics  
    Stats,
    /// Simulate network activity over time
    Simulate {
        #[arg(short, long, default_value = "10")]
        keepers: u32,
        #[arg(short, long, default_value = "100")]
        events: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    print_banner();

    match &cli.command {
        Some(Commands::Interactive) => interactive_demo().await?,
        Some(Commands::Showcase) => showcase_demo().await?,
        Some(Commands::Stats) => stats_demo().await?,
        Some(Commands::Simulate { keepers, events }) => simulate_network(*keepers, *events).await?,
        None => interactive_demo().await?, // Default to interactive
    }

    Ok(())
}

fn print_banner() {
    println!("{}", "
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║  🔮 PARADIGM SYMBOLIC NETWORK TOKENS (SNTs) DEMO 🔮        ║
║                                                              ║
║  Experience the revolutionary alternative to traditional     ║
║  NFTs - where tokens unlock LIVING functionality instead    ║
║  of dead commodities.                                        ║
║                                                              ║
║  • Earn SNTs through network participation                   ║
║  • Watch them evolve based on your contributions            ║
║  • Use them to unlock new capabilities                      ║
║  • Participate in mythic rituals and ceremonies             ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
    ".bright_cyan());
    println!();
}

async fn interactive_demo() -> Result<()> {
    let mut network = DemoNetwork::new();
    let mut current_user = String::new();

    loop {
        if current_user.is_empty() {
            let options = vec![
                "🆕 Create New Keeper",
                "📊 View Network Stats", 
                "🔍 Browse All SNTs",
                "⚡ Simulate Network Activity",
                "❌ Exit Demo",
            ];
            
            let choice = Select::new("What would you like to do?", options).prompt()?;
            
            match choice {
                "🆕 Create New Keeper" => {
                    current_user = create_keeper_flow(&mut network).await?;
                }
                "📊 View Network Stats" => {
                    display_network_stats(&network);
                }
                "🔍 Browse All SNTs" => {
                    display_all_snts(&network);
                }
                "⚡ Simulate Network Activity" => {
                    simulate_activity(&mut network).await?;
                }
                "❌ Exit Demo" => break,
                _ => {}
            }
        } else {
            let user_snts = network.snt_system.get_holder_snts(&current_user);
            
            println!("\n{} {}", "🧙‍♂️ Welcome back,".bright_green(), current_user.bright_yellow());
            println!("{} {}", "📜 Your SNTs:".bright_blue(), user_snts.len().to_string().bright_white());
            
            let options = vec![
                "🗂️ View My SNT Collection",
                "📦 Store New Sigil", 
                "⚗️ Perform Fusion Ritual",
                "🎨 Design Custom Glyph",
                "💫 View Network Animations",
                "📈 Check My Evolution Progress",
                "🔄 Switch User",
                "❌ Exit Demo",
            ];
            
            let choice = Select::new("What would you like to do?", options).prompt()?;
            
            match choice {
                "🗂️ View My SNT Collection" => {
                    display_user_snts(&network, &current_user);
                }
                "📦 Store New Sigil" => {
                    store_sigil_flow(&mut network, &current_user).await?;
                }
                "⚗️ Perform Fusion Ritual" => {
                    fusion_ritual_flow(&mut network, &current_user).await?;
                }
                "🎨 Design Custom Glyph" => {
                    glyph_design_flow(&mut network, &current_user).await?;
                }
                "💫 View Network Animations" => {
                    display_network_animations(&network);
                }
                "📈 Check My Evolution Progress" => {
                    display_evolution_progress(&network, &current_user);
                }
                "🔄 Switch User" => {
                    current_user.clear();
                }
                "❌ Exit Demo" => break,
                _ => {}
            }
        }
        
        // Wait for user to continue
        println!("\n{}", "Press Enter to continue...".dimmed());
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }
    
    println!("\n{}", "🌟 Thanks for exploring Paradigm SNTs! The future of functional digital identity awaits. 🌟".bright_green());
    Ok(())
}

async fn create_keeper_flow(network: &mut DemoNetwork) -> Result<String> {
    println!("\n{}", "🆕 KEEPER REGISTRATION".bright_yellow().underline());
    
    let name = Text::new("Enter your keeper name:")
        .with_default("Alice")
        .prompt()?;
        
    let capacity_gb = Text::new("Storage capacity (GB):")
        .with_default("100")
        .prompt()?
        .parse::<u64>()
        .unwrap_or(100);
    
    let keeper_id = network.register_keeper(name.clone(), capacity_gb * 1024 * 1024 * 1024).await?;
    
    println!("\n{}", "✅ KEEPER SUCCESSFULLY REGISTERED!".bright_green());
    println!("{} {}", "🆔 Keeper ID:".bright_blue(), keeper_id.bright_white());
    
    // Show the automatically minted Keeper Identity SNT
    if let Some(keeper_snt) = network.snt_system.get_holder_snts(&keeper_id).first() {
        println!("\n{}", "🎁 WELCOME BONUS - SNT AUTOMATICALLY MINTED!".bright_green());
        display_snt_details(keeper_snt);
        
        // Animate keeper registration
        network.animator.trigger_keeper_registration(&keeper_id);
        display_animation_effect("Keeper heartbeat initiated", "💓");
    }
    
    Ok(keeper_id)
}

async fn store_sigil_flow(network: &mut DemoNetwork, keeper_id: &str) -> Result<()> {
    println!("\n{}", "📦 SIGIL STORAGE RITUAL".bright_yellow().underline());
    
    let file_types = vec!["📄 Document", "🖼️ Image", "🎵 Audio", "📊 Dataset", "💻 Code"];
    let file_type = Select::new("What type of data are you storing?", file_types).prompt()?;
    
    let filename = Text::new("Enter filename:")
        .with_default("my-data.txt")
        .prompt()?;
        
    let size_kb = Text::new("File size (KB):")
        .with_default("1024")
        .prompt()?
        .parse::<usize>()
        .unwrap_or(1024);

    let sigil_id = network.store_sigil(keeper_id.to_string(), filename, size_kb * 1024, file_type).await?;
    
    println!("\n{}", "✅ SIGIL STORED SUCCESSFULLY!".bright_green());
    println!("{} {}", "🔗 Sigil ID:".bright_blue(), sigil_id.bright_white());
    
    // Check for achievements and new SNTs
    let new_snts = network.check_achievements(keeper_id).await?;
    if !new_snts.is_empty() {
        println!("\n{}", "🏆 ACHIEVEMENT UNLOCKED - NEW SNT MINTED!".bright_green());
        for snt_id in new_snts {
            if let Some(snt) = network.snt_system.active_snts.get(&snt_id) {
                display_snt_details(snt);
            }
        }
    }
    
    // Show storage animation
    network.animator.trigger_sigil_storage(&sigil_id, keeper_id);
    display_animation_effect("Sigil distributed across network", "📡");
    
    Ok(())
}

async fn fusion_ritual_flow(network: &mut DemoNetwork, keeper_id: &str) -> Result<()> {
    println!("\n{}", "⚗️ FUSION RITUAL CEREMONY".bright_yellow().underline());
    
    let user_sigils = network.get_user_sigils(keeper_id);
    if user_sigils.len() < 2 {
        println!("{}", "❌ You need at least 2 sigils to perform fusion!".bright_red());
        return Ok(());
    }
    
    println!("{} {}", "📦 Available Sigils:".bright_blue(), user_sigils.len().to_string().bright_white());
    for (i, sigil) in user_sigils.iter().enumerate() {
        println!("  {}. {} ({})", (i+1).to_string().bright_yellow(), sigil.filename.as_ref().unwrap_or(&"Unknown".to_string()), format_file_size(sigil.size));
    }
    
    let fusion_count = Text::new("How many sigils to fuse? (2-5):")
        .with_default("2")
        .prompt()?
        .parse::<usize>()
        .unwrap_or(2)
        .min(5)
        .max(user_sigils.len());
    
    let ritual_types = vec!["⚗️ Synthesis", "🌙 Transmutation", "⚡ Crystallization", "🔮 Sublimation"];
    let ritual_type = Select::new("Choose fusion ritual type:", ritual_types).prompt()?;
    
    let confirm = Confirm::new("Begin the fusion ritual? (This will consume the selected sigils)")
        .with_default(true)
        .prompt()?;
        
    if !confirm {
        return Ok(());
    }
    
    println!("\n{}", "🌟 INITIATING FUSION RITUAL...".bright_magenta());
    
    // Simulate fusion process
    network.animator.trigger_fusion_ritual(fusion_count);
    display_fusion_animation(ritual_type);
    
    let tome_id = network.perform_fusion(keeper_id, fusion_count, ritual_type).await?;
    
    println!("\n{}", "✨ FUSION COMPLETED SUCCESSFULLY!".bright_green());
    println!("{} {}", "📜 Tome Created:".bright_blue(), tome_id.bright_white());
    
    // Check for Memory Anchor SNT creation
    let new_snts = network.check_achievements(keeper_id).await?;
    if !new_snts.is_empty() {
        println!("\n{}", "🏛️ MEMORY ANCHOR SNT CREATED!".bright_green());
        for snt_id in new_snts {
            if let Some(snt) = network.snt_system.active_snts.get(&snt_id) {
                display_snt_details(snt);
            }
        }
    }
    
    Ok(())
}

async fn glyph_design_flow(network: &mut DemoNetwork, keeper_id: &str) -> Result<()> {
    println!("\n{}", "🎨 GLYPH DESIGNER STUDIO".bright_yellow().underline());
    
    let elements = vec!["🔥 Fire", "💧 Water", "🌍 Earth", "💨 Air", "⚡ Lightning", "🌙 Void", "🔮 Aether"];
    let element = Select::new("Choose primary element:", elements).prompt()?;
    
    let categories = vec!["📦 Archive", "🧠 Model", "📊 Dataset", "🎯 Result", "🎬 Media"];
    let category = Select::new("Choose data category:", categories).prompt()?;
    
    let importance = vec!["✨ Trivial", "📎 Minor", "📋 Standard", "⭐ Major", "🔥 Critical", "👑 Legendary"];
    let importance_level = Select::new("Choose importance level:", importance).prompt()?;
    
    let glyph_name = Text::new("Name your custom glyph:")
        .with_default("My Custom Glyph")
        .prompt()?;
    
    let glyph_id = network.create_custom_glyph(keeper_id, glyph_name, element, category, importance_level).await?;
    
    println!("\n{}", "✅ CUSTOM GLYPH CREATED!".bright_green());
    println!("{} {}", "🎨 Glyph ID:".bright_blue(), glyph_id.bright_white());
    
    // Show glyph creation animation
    display_animation_effect("Glyph crystallizing into existence", "✨");
    
    Ok(())
}

async fn showcase_demo() -> Result<()> {
    println!("{}", "🚀 PARADIGM SNT SHOWCASE".bright_cyan().underline());
    println!();
    
    let mut network = DemoNetwork::new();
    
    // Quick demo sequence
    println!("{}", "1️⃣  Creating sample keepers...".bright_blue());
    let keeper1 = network.register_keeper("Alice".to_string(), 100 * 1024 * 1024 * 1024).await?;
    let keeper2 = network.register_keeper("Bob".to_string(), 50 * 1024 * 1024 * 1024).await?;
    println!("   {} Keepers registered with Identity SNTs!", "✅".bright_green());
    
    println!("\n{}", "2️⃣  Storing sigils and earning rewards...".bright_blue());
    network.store_sigil(keeper1.clone(), "research-data.json".to_string(), 5 * 1024 * 1024, "📊 Dataset").await?;
    network.store_sigil(keeper1.clone(), "model-weights.bin".to_string(), 50 * 1024 * 1024, "🧠 Model").await?;
    network.store_sigil(keeper2.clone(), "image-collection.zip".to_string(), 20 * 1024 * 1024, "🖼️ Image").await?;
    println!("   {} Sigils stored, Contribution SNTs minted!", "✅".bright_green());
    
    println!("\n{}", "3️⃣  Performing fusion ritual...".bright_blue());
    let tome_id = network.perform_fusion(&keeper1, 2, "⚗️ Synthesis").await?;
    println!("   {} Fusion completed, Memory Anchor SNT created!", "✅".bright_green());
    
    println!("\n{}", "4️⃣  Network statistics:".bright_blue());
    display_network_stats(&network);
    
    println!("\n{}", "5️⃣  Alice's SNT collection:".bright_blue());
    display_user_snts(&network, &keeper1);
    
    println!("\n{}", "🎉 SHOWCASE COMPLETE!".bright_green());
    println!("{}", "This demonstrates how SNTs unlock living functionality through network participation.".dimmed());
    
    Ok(())
}

async fn stats_demo() -> Result<()> {
    let network = DemoNetwork::new_with_sample_data().await?;
    
    println!("{}", "📊 NETWORK STATISTICS DASHBOARD".bright_cyan().underline());
    display_network_stats(&network);
    
    println!("\n{}", "🏆 TOP CONTRIBUTORS".bright_yellow().underline());
    display_top_contributors(&network);
    
    println!("\n{}", "📈 SNT ANALYTICS".bright_blue().underline());
    display_snt_analytics(&network);
    
    Ok(())
}

async fn simulate_network(keeper_count: u32, event_count: u32) -> Result<()> {
    println!("{}", format!("⚡ SIMULATING NETWORK ({} keepers, {} events)", keeper_count, event_count).bright_cyan().underline());
    
    let mut network = DemoNetwork::new();
    let mut simulation = NetworkSimulation::new(keeper_count);
    
    simulation.run_simulation(&mut network, event_count).await?;
    
    println!("\n{}", "📊 SIMULATION RESULTS".bright_green().underline());
    display_network_stats(&network);
    
    Ok(())
}

async fn simulate_activity(network: &mut DemoNetwork) -> Result<()> {
    let event_count = Text::new("How many events to simulate?")
        .with_default("20")
        .prompt()?
        .parse::<u32>()
        .unwrap_or(20);
    
    println!("\n{}", format!("⚡ Running {} network events...", event_count).bright_blue());
    
    let mut simulation = NetworkSimulation::new(3);
    simulation.run_simulation(network, event_count).await?;
    
    println!("{}", "✅ Simulation complete!".bright_green());
    
    Ok(())
}