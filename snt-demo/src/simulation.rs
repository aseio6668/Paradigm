use anyhow::Result;
use crate::snt::*;
use std::collections::HashMap;
use uuid::Uuid;

pub struct DemoNetwork {
    pub snt_system: SNTSystem,
    pub keepers: HashMap<String, Keeper>,
    pub sigils: HashMap<String, Sigil>,
    pub animator: NetworkAnimator,
    pub event_log: Vec<NetworkEvent>,
}

#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub event_type: String,
    pub actor: String,
    pub details: String,
    pub timestamp: u64,
}

pub struct NetworkAnimator {
    pub active_animations: Vec<Animation>,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub animation_type: String,
    pub target: String,
    pub effect: String,
    pub duration: u64,
}

pub struct NetworkSimulation {
    keeper_count: u32,
    event_types: Vec<String>,
}

impl DemoNetwork {
    pub fn new() -> Self {
        Self {
            snt_system: SNTSystem::new(),
            keepers: HashMap::new(),
            sigils: HashMap::new(),
            animator: NetworkAnimator::new(),
            event_log: Vec::new(),
        }
    }

    pub async fn new_with_sample_data() -> Result<Self> {
        let mut network = Self::new();
        
        // Create sample keepers
        let alice = network.register_keeper("Alice".to_string(), 100 * 1024 * 1024 * 1024).await?;
        let bob = network.register_keeper("Bob".to_string(), 50 * 1024 * 1024 * 1024).await?;
        let carol = network.register_keeper("Carol".to_string(), 200 * 1024 * 1024 * 1024).await?;
        
        // Store some sigils
        network.store_sigil(alice.clone(), "research-paper.pdf".to_string(), 5 * 1024 * 1024, "📄 Document").await?;
        network.store_sigil(alice.clone(), "ml-model.pkl".to_string(), 50 * 1024 * 1024, "🧠 Model").await?;
        network.store_sigil(bob.clone(), "dataset-v1.csv".to_string(), 20 * 1024 * 1024, "📊 Dataset").await?;
        network.store_sigil(bob.clone(), "image-gallery.zip".to_string(), 100 * 1024 * 1024, "🖼️ Image").await?;
        network.store_sigil(carol.clone(), "analysis-results.json".to_string(), 2 * 1024 * 1024, "🎯 Result").await?;
        
        // Perform some fusions
        network.perform_fusion(&alice, 2, "⚗️ Synthesis").await?;
        network.perform_fusion(&bob, 2, "🌙 Transmutation").await?;
        
        // Evolve some SNTs
        for keeper_id in [&alice, &bob, &carol] {
            for _ in 0..3 {
                network.award_evolution_points(keeper_id, 25.0).await?;
            }
        }
        
        Ok(network)
    }

    pub async fn register_keeper(&mut self, name: String, capacity: u64) -> Result<String> {
        let keeper_id = format!("keeper_{}", Uuid::new_v4().simple().to_string()[..8].to_string());
        
        let keeper = Keeper {
            keeper_id: keeper_id.clone(),
            name: name.clone(),
            capacity,
            used_storage: 0,
            reputation: 0.8,
            sigils_hosted: Vec::new(),
            total_earned: 0,
            status: KeeperStatus::Apprentice,
            created_at: current_timestamp(),
        };
        
        self.keepers.insert(keeper_id.clone(), keeper);
        
        // Mint Keeper Identity SNT
        let context = [
            ("keeper_name".to_string(), name.clone()),
            ("initial_capacity".to_string(), format!("{} GB", capacity / (1024 * 1024 * 1024))),
            ("welcome_bonus".to_string(), "10_PAR".to_string()),
        ].iter().cloned().collect();
        
        let snt_id = self.snt_system.mint_snt("keeper_identity", keeper_id.clone(), context)?;
        
        // Log event
        self.event_log.push(NetworkEvent {
            event_type: "keeper_registered".to_string(),
            actor: keeper_id.clone(),
            details: format!("{} joined as {} keeper", name, KeeperStatus::Apprentice.symbol()),
            timestamp: current_timestamp(),
        });
        
        Ok(keeper_id)
    }

    pub async fn store_sigil(&mut self, keeper_id: String, filename: String, size: usize, file_type: &str) -> Result<String> {
        let sigil_id = format!("sigil_{}", Uuid::new_v4().simple().to_string()[..8].to_string());
        
        // Determine glyph based on file type
        let element = match file_type {
            s if s.contains("Dataset") => Element::Fire,
            s if s.contains("Model") => Element::Aether,
            s if s.contains("Image") || s.contains("Audio") => Element::Water,
            s if s.contains("Document") => Element::Earth,
            s if s.contains("Code") => Element::Lightning,
            _ => Element::Air,
        };
        
        let category = DataCategory::from_string(file_type);
        let importance = if size > 50 * 1024 * 1024 { Importance::Major } else { Importance::Standard };
        
        let sigil = Sigil {
            sigil_id: sigil_id.clone(),
            filename: Some(filename.clone()),
            size,
            glyph: Glyph {
                element: element.clone(),
                category,
                importance,
                symbol: element.symbol().to_string(),
            },
            owner: keeper_id.clone(),
            stored_at: vec![keeper_id.clone()],
            created_at: current_timestamp(),
        };
        
        self.sigils.insert(sigil_id.clone(), sigil.clone());
        
        // Update keeper storage
        if let Some(keeper) = self.keepers.get_mut(&keeper_id) {
            keeper.used_storage += size as u64;
            keeper.sigils_hosted.push(sigil_id.clone());
        }
        
        // Mint Storage Contribution SNT
        let context = [
            ("sigil_id".to_string(), sigil_id.clone()),
            ("filename".to_string(), filename),
            ("size".to_string(), format_file_size(size)),
            ("file_type".to_string(), file_type.to_string()),
        ].iter().cloned().collect();
        
        self.snt_system.mint_snt("storage_contribution", keeper_id.clone(), context)?;
        
        // Award evolution points
        self.award_evolution_points(&keeper_id, 10.0).await?;
        
        // Log event
        self.event_log.push(NetworkEvent {
            event_type: "sigil_stored".to_string(),
            actor: keeper_id,
            details: format!("Stored {} ({})", sigil.filename.as_ref().unwrap_or(&"Unknown".to_string()), format_file_size(size)),
            timestamp: current_timestamp(),
        });
        
        Ok(sigil_id)
    }

    pub async fn perform_fusion(&mut self, keeper_id: &str, sigil_count: usize, ritual_type: &str) -> Result<String> {
        let tome_id = format!("tome_{}", Uuid::new_v4().simple().to_string()[..8].to_string());
        
        // Remove sigils from keeper (simulate consumption)
        if let Some(keeper) = self.keepers.get_mut(keeper_id) {
            let consumed_sigils = keeper.sigils_hosted.drain(0..sigil_count.min(keeper.sigils_hosted.len())).collect::<Vec<_>>();
            
            // Remove from sigils collection
            for sigil_id in &consumed_sigils {
                self.sigils.remove(sigil_id);
            }
        }
        
        // Create Memory Anchor SNT for significant fusions
        if sigil_count >= 3 {
            let context = [
                ("tome_id".to_string(), tome_id.clone()),
                ("ritual_type".to_string(), ritual_type.to_string()),
                ("sigil_count".to_string(), sigil_count.to_string()),
                ("fusion_quality".to_string(), "Perfect".to_string()),
            ].iter().cloned().collect();
            
            self.snt_system.mint_snt("memory_anchor", keeper_id.to_string(), context)?;
        }
        
        // Award significant evolution points for fusion
        self.award_evolution_points(keeper_id, 50.0).await?;
        
        // Check for Fusion Master achievement
        let user_snts = self.snt_system.get_holder_snts(keeper_id);
        let fusion_count = user_snts.iter()
            .filter(|snt| snt.token_type == SNTType::MemoryAnchor)
            .count();
            
        if fusion_count >= 3 && !user_snts.iter().any(|snt| snt.token_type == SNTType::FusionMaster) {
            let context = [
                ("achievement".to_string(), "fusion_master".to_string()),
                ("fusion_count".to_string(), fusion_count.to_string()),
            ].iter().cloned().collect();
            
            self.snt_system.mint_snt("fusion_master", keeper_id.to_string(), context)?;
        }
        
        // Log event
        self.event_log.push(NetworkEvent {
            event_type: "fusion_completed".to_string(),
            actor: keeper_id.to_string(),
            details: format!("Performed {} ritual with {} sigils", ritual_type, sigil_count),
            timestamp: current_timestamp(),
        });
        
        Ok(tome_id)
    }

    pub async fn create_custom_glyph(&mut self, keeper_id: &str, name: String, element: &str, category: &str, importance: &str) -> Result<String> {
        let glyph_id = format!("glyph_{}", Uuid::new_v4().simple().to_string()[..8].to_string());
        
        // Award evolution points for creativity
        self.award_evolution_points(keeper_id, 15.0).await?;
        
        // Check for Glyph Artist achievement
        self.check_glyph_artist_achievement(keeper_id).await?;
        
        // Log event
        self.event_log.push(NetworkEvent {
            event_type: "glyph_created".to_string(),
            actor: keeper_id.to_string(),
            details: format!("Designed custom glyph: {}", name),
            timestamp: current_timestamp(),
        });
        
        Ok(glyph_id)
    }

    pub async fn award_evolution_points(&mut self, keeper_id: &str, points: f64) -> Result<()> {
        let holder_snts: Vec<String> = self.snt_system.get_holder_snts(keeper_id)
            .iter()
            .map(|snt| snt.snt_id.clone())
            .collect();
        
        for snt_id in holder_snts {
            if let Ok(evolved) = self.snt_system.evolve_snt(&snt_id, points) {
                if evolved {
                    self.event_log.push(NetworkEvent {
                        event_type: "snt_evolved".to_string(),
                        actor: keeper_id.to_string(),
                        details: format!("SNT evolved to next level!"),
                        timestamp: current_timestamp(),
                    });
                }
            }
        }
        
        Ok(())
    }

    pub async fn check_achievements(&self, keeper_id: &str) -> Result<Vec<String>> {
        let mut new_snts = Vec::new();
        let user_snts = self.snt_system.get_holder_snts(keeper_id);
        
        // Check for first sigil achievement
        let storage_snts = user_snts.iter()
            .filter(|snt| snt.token_type == SNTType::StorageContribution)
            .count();
            
        if storage_snts == 1 {
            // This would mint a "first storage" achievement SNT
            // For demo simplicity, we'll just return the existing SNT IDs
            new_snts.extend(user_snts.iter().map(|snt| snt.snt_id.clone()));
        }
        
        Ok(new_snts)
    }

    async fn check_glyph_artist_achievement(&mut self, keeper_id: &str) -> Result<()> {
        let user_snts = self.snt_system.get_holder_snts(keeper_id);
        
        // Check if they already have Glyph Artist SNT
        if !user_snts.iter().any(|snt| snt.token_type == SNTType::GlyphArtist) {
            let context: HashMap<String, String> = [
                ("achievement".to_string(), "glyph_artist".to_string()),
                ("first_custom_glyph".to_string(), "true".to_string()),
            ].iter().cloned().collect();
            
            // Would mint Glyph Artist SNT here in full implementation
            // self.snt_system.mint_snt("glyph_artist", keeper_id.to_string(), context)?;
        }
        
        Ok(())
    }

    pub fn get_user_sigils(&self, keeper_id: &str) -> Vec<&Sigil> {
        self.sigils.values()
            .filter(|sigil| sigil.owner == keeper_id)
            .collect()
    }

    pub fn get_recent_events(&self, limit: usize) -> Vec<&NetworkEvent> {
        self.event_log.iter().rev().take(limit).collect()
    }
}

impl NetworkAnimator {
    pub fn new() -> Self {
        Self {
            active_animations: Vec::new(),
        }
    }

    pub fn trigger_keeper_registration(&mut self, keeper_id: &str) {
        self.active_animations.push(Animation {
            animation_type: "heartbeat".to_string(),
            target: keeper_id.to_string(),
            effect: "Keeper heartbeat initiated".to_string(),
            duration: 1000,
        });
    }

    pub fn trigger_sigil_storage(&mut self, sigil_id: &str, keeper_id: &str) {
        self.active_animations.push(Animation {
            animation_type: "storage_flow".to_string(),
            target: format!("{}→{}", sigil_id, keeper_id),
            effect: "Sigil distributed across network".to_string(),
            duration: 2000,
        });
    }

    pub fn trigger_fusion_ritual(&mut self, sigil_count: usize) {
        self.active_animations.push(Animation {
            animation_type: "fusion_spiral".to_string(),
            target: "fusion_altar".to_string(),
            effect: format!("Fusing {} sigils into transcendent tome", sigil_count),
            duration: 5000,
        });
    }

    pub fn get_active_animations(&self) -> &Vec<Animation> {
        &self.active_animations
    }
}

impl NetworkSimulation {
    pub fn new(keeper_count: u32) -> Self {
        Self {
            keeper_count,
            event_types: vec![
                "store_sigil".to_string(),
                "perform_fusion".to_string(), 
                "create_glyph".to_string(),
                "keeper_interaction".to_string(),
            ],
        }
    }

    pub async fn run_simulation(&mut self, network: &mut DemoNetwork, event_count: u32) -> Result<()> {
        // Create sample keepers if none exist
        if network.keepers.is_empty() {
            for i in 0..self.keeper_count {
                let name = format!("Keeper_{}", i + 1);
                network.register_keeper(name, (50 + i * 25) as u64 * 1024 * 1024 * 1024).await?;
            }
        }

        let keeper_ids: Vec<String> = network.keepers.keys().cloned().collect();
        
        // Run random events
        for i in 0..event_count {
            let keeper_id = &keeper_ids[i as usize % keeper_ids.len()];
            let event_type = &self.event_types[i as usize % self.event_types.len()];
            
            match event_type.as_str() {
                "store_sigil" => {
                    let filename = format!("data-file-{}.txt", i);
                    let size = 1024 * 1024 * (i % 10 + 1) as usize; // 1-10 MB
                    network.store_sigil(keeper_id.clone(), filename, size, "📄 Document").await?;
                }
                "perform_fusion" => {
                    if network.get_user_sigils(keeper_id).len() >= 2 {
                        network.perform_fusion(keeper_id, 2, "⚗️ Synthesis").await?;
                    }
                }
                "create_glyph" => {
                    let name = format!("Custom-Glyph-{}", i);
                    network.create_custom_glyph(keeper_id, name, "🔥 Fire", "📦 Archive", "📋 Standard").await?;
                }
                _ => {
                    // Generic network activity
                    network.award_evolution_points(keeper_id, 5.0).await?;
                }
            }
            
            // Small delay for realism
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        Ok(())
    }
}

pub fn format_file_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}