use colored::*;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use crate::snt::*;
use crate::simulation::*;
use std::thread;
use std::time::Duration;

pub fn display_snt_details(snt: &SNT) {
    println!("{}", format!("╭─ {} ─╮", snt.glyph.symbol).bright_cyan());
    println!("│ {:<20} │", format!("{} {}", snt.token_type.name(), snt.glyph.symbol).bright_white());
    println!("│ {:<20} │", format!("Level {}", snt.evolution_level).bright_yellow());
    println!("│ {:<20} │", format!("Progress: {:.1}%", snt.evolution_progress).dimmed());
    println!("│ {:<20} │", format!("Element: {}", snt.glyph.element.symbol()).bright_blue());
    println!("│ {:<20} │", format!("Rarity: {:?}", snt.glyph.importance).bright_magenta());
    if !snt.narrative_fragments.is_empty() {
        println!("│ {:<20} │", "Recent Memory:".dimmed());
        println!("│ {:<20} │", snt.narrative_fragments.last().unwrap_or(&"None".to_string()).italic());
    }
    println!("{}", "╰────────────────────╯".bright_cyan());
}

pub fn display_network_stats(network: &DemoNetwork) {
    let stats = network.snt_system.get_stats();
    
    let mut table = Table::new();
    table.load_preset(UTF8_FULL)
         .set_header(vec!["📊 Network Statistics", "Value"]);

    table.add_row(vec!["🔗 Active Keepers", &network.keepers.len().to_string()]);
    table.add_row(vec!["📦 Total Sigils", &network.sigils.len().to_string()]);
    table.add_row(vec!["🎯 SNTs Minted", &stats.total_snts.to_string()]);
    table.add_row(vec!["👥 Unique Holders", &stats.unique_holders.to_string()]);
    table.add_row(vec!["⬆️ Avg Evolution Level", &format!("{:.1}", stats.average_evolution_level)]);
    
    let total_storage: u64 = network.keepers.values().map(|k| k.used_storage).sum();
    table.add_row(vec!["💾 Total Storage Used", &format_file_size(total_storage as usize)]);
    
    let total_capacity: u64 = network.keepers.values().map(|k| k.capacity).sum();
    let utilization = if total_capacity > 0 { (total_storage as f64 / total_capacity as f64) * 100.0 } else { 0.0 };
    table.add_row(vec!["📈 Network Utilization", &format!("{:.1}%", utilization)]);
    
    table.add_row(vec!["⚡ Recent Events", &network.event_log.len().to_string()]);

    println!("{}", table);
}

pub fn display_all_snts(network: &DemoNetwork) {
    let stats = network.snt_system.get_stats();
    
    println!("\n{}", "🔮 ALL NETWORK SNTs".bright_cyan().underline());
    
    let mut table = Table::new();
    table.load_preset(UTF8_FULL)
         .set_header(vec!["Type", "Count", "Percentage"]);

    for (snt_type, count) in &stats.type_distribution {
        let percentage = (*count as f64 / stats.total_snts as f64) * 100.0;
        table.add_row(vec![
            snt_type,
            &count.to_string(),
            &format!("{:.1}%", percentage)
        ]);
    }
    
    println!("{}", table);
    
    // Show recent activity
    println!("\n{}", "⚡ RECENT NETWORK ACTIVITY".bright_blue().underline());
    for event in network.get_recent_events(5) {
        let time_ago = format!("{}s ago", chrono::Utc::now().timestamp() as u64 - event.timestamp);
        println!("{} {} {} {}", 
                 event.event_type.bright_yellow(),
                 event.actor.bright_blue(),
                 event.details.white(),
                 time_ago.dimmed());
    }
}

pub fn display_user_snts(network: &DemoNetwork, keeper_id: &str) {
    let user_snts = network.snt_system.get_holder_snts(keeper_id);
    let keeper = network.keepers.get(keeper_id);
    
    if let Some(keeper) = keeper {
        println!("\n{}", format!("🧙‍♂️ {}'s MYSTICAL COLLECTION", keeper.name).bright_cyan().underline());
        println!("{} {} | {} {} | {} {}", 
                "Status:".dimmed(), keeper.status.symbol(),
                "Reputation:".dimmed(), format!("{:.2}", keeper.reputation).bright_green(),
                "Storage:".dimmed(), format!("{}/{}", 
                    format_file_size(keeper.used_storage as usize),
                    format_file_size(keeper.capacity as usize)
                ).bright_blue()
        );
    }
    
    if user_snts.is_empty() {
        println!("{}", "No SNTs found. Participate in the network to earn your first tokens!".dimmed());
        return;
    }
    
    println!("\n{}", format!("📜 {} Symbolic Network Tokens:", user_snts.len()).bright_white());
    
    for (i, snt) in user_snts.iter().enumerate() {
        println!("\n{} {}", format!("{}.", i + 1).bright_yellow(), snt.token_type.name().bright_white());
        display_snt_details(snt);
    }
    
    // Show evolution progress summary
    let total_levels: u32 = user_snts.iter().map(|snt| snt.evolution_level).sum();
    let avg_progress: f64 = user_snts.iter().map(|snt| snt.evolution_progress).sum::<f64>() / user_snts.len() as f64;
    
    println!("\n{}", "📈 EVOLUTION SUMMARY".bright_green().underline());
    println!("{} {} | {} {:.1}%", 
            "Total Levels:".dimmed(), total_levels.to_string().bright_white(),
            "Average Progress:".dimmed(), avg_progress);
}

pub fn display_top_contributors(network: &DemoNetwork) {
    let mut keepers: Vec<_> = network.keepers.values().collect();
    keepers.sort_by(|a, b| b.total_earned.cmp(&a.total_earned));
    
    let mut table = Table::new();
    table.load_preset(UTF8_FULL)
         .set_header(vec!["Rank", "Keeper", "Status", "Reputation", "Storage", "SNTs"]);

    for (i, keeper) in keepers.iter().take(5).enumerate() {
        let snt_count = network.snt_system.get_holder_snts(&keeper.keeper_id).len();
        table.add_row(vec![
            &format!("#{}", i + 1),
            &keeper.name,
            keeper.status.symbol(),
            &format!("{:.2}", keeper.reputation),
            &format_file_size(keeper.used_storage as usize),
            &snt_count.to_string()
        ]);
    }
    
    println!("{}", table);
}

pub fn display_snt_analytics(network: &DemoNetwork) {
    let stats = network.snt_system.get_stats();
    
    let mut evolution_levels = std::collections::HashMap::new();
    for snt in network.snt_system.active_snts.values() {
        *evolution_levels.entry(snt.evolution_level).or_insert(0) += 1;
    }
    
    println!("📊 Evolution Level Distribution:");
    for level in 1..=5 {
        let count = evolution_levels.get(&level).unwrap_or(&0);
        let bar = "█".repeat(*count).bright_blue();
        println!("  Level {}: {} ({})", level, bar, count);
    }
    
    println!("\n🎭 SNT Type Distribution:");
    for (snt_type, count) in &stats.type_distribution {
        let bar = "▓".repeat(*count).bright_green();
        println!("  {}: {} ({})", snt_type, bar, count);
    }
}

pub fn display_evolution_progress(network: &DemoNetwork, keeper_id: &str) {
    let user_snts = network.snt_system.get_holder_snts(keeper_id);
    
    println!("\n{}", "📈 YOUR EVOLUTION JOURNEY".bright_green().underline());
    
    for snt in user_snts {
        let progress_bar_width = 20;
        let filled = ((snt.evolution_progress / 100.0) * progress_bar_width as f64) as usize;
        let empty = progress_bar_width - filled;
        
        let bar = format!("{}{}",
                         "█".repeat(filled).bright_green(),
                         "░".repeat(empty).dimmed());
        
        println!("{} {} [{}] {:.1}%",
                snt.glyph.symbol,
                snt.token_type.name().bright_white(),
                bar,
                snt.evolution_progress);
        
        if snt.evolution_level > 1 {
            println!("  {} Achieved level {} through network participation",
                    "🌟".bright_yellow(),
                    snt.evolution_level);
        }
        
        println!();
    }
    
    let next_rewards = vec![
        "Level 5: Glyph importance upgrade",
        "Level 10: Advanced permissions",
        "Level 15: Ritual leadership access",
        "Level 20: Network governance rights"
    ];
    
    println!("{}", "🎯 UPCOMING REWARDS".bright_blue());
    for reward in next_rewards {
        println!("  • {}", reward.dimmed());
    }
}

pub fn display_network_animations(network: &DemoNetwork) {
    println!("\n{}", "💫 NETWORK ANIMATIONS & EFFECTS".bright_magenta().underline());
    
    let animations = network.animator.get_active_animations();
    
    if animations.is_empty() {
        println!("{}", "Network is calm... waiting for activity to create visual effects.".dimmed());
        return;
    }
    
    for animation in animations {
        println!("{} {} {}",
                animation.animation_type.bright_cyan(),
                "→".bright_white(),
                animation.effect.bright_yellow());
    }
    
    // Simulate some live effects
    let effects = vec![
        ("🔮", "Keeper heartbeats synchronizing across the network"),
        ("📡", "Sigil distribution flowing through quantum channels"),
        ("⚡", "Evolution energy cascading through token holders"),
        ("🌟", "Memory anchors crystallizing into permanent lore"),
    ];
    
    println!("\n{}", "✨ LIVE NETWORK EFFECTS:".bright_white());
    for (symbol, description) in effects {
        println!("   {} {}", symbol, description.bright_white());
        thread::sleep(Duration::from_millis(300));
    }
}

pub fn display_animation_effect(description: &str, symbol: &str) {
    println!("\n{}", "✨ NETWORK EFFECT".bright_magenta());
    
    // Animated effect display
    for i in 0..3 {
        print!("\r{} {}{}",
               symbol,
               description.bright_white(),
               ".".repeat(i + 1).dimmed());
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        thread::sleep(Duration::from_millis(500));
    }
    println!(" ✅");
}

pub fn display_fusion_animation(ritual_type: &str) {
    let phases = vec![
        "🌟 Gathering sigil essences...",
        "⚡ Channeling elemental energies...", 
        "🔮 Weaving quantum entanglements...",
        "✨ Crystallizing new reality...",
        "🎉 Fusion complete!"
    ];
    
    println!("\n{} {}", "🔥 FUSION RITUAL:".bright_red(), ritual_type.bright_yellow());
    
    for phase in phases {
        println!("   {}", phase.bright_white());
        thread::sleep(Duration::from_millis(800));
    }
}

impl SNTType {
    pub fn name(&self) -> String {
        match self {
            SNTType::KeeperIdentity => "🛡️ Keeper Identity".to_string(),
            SNTType::StorageContribution => "📦 Storage Contribution".to_string(),
            SNTType::MemoryAnchor => "📜 Memory Anchor".to_string(),
            SNTType::FusionMaster => "⚗️ Fusion Master".to_string(),
            SNTType::GlyphArtist => "🎨 Glyph Artist".to_string(),
            SNTType::CommunityBond => "🤝 Community Bond".to_string(),
        }
    }
}