use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Symbolic Network Token - Living functional identity anchors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNT {
    pub snt_id: String,
    pub holder: String,
    pub token_type: SNTType,
    pub glyph: Glyph,
    pub created_at: u64,
    pub evolution_level: u32,
    pub evolution_progress: f64,
    pub permissions: Vec<Permission>,
    pub properties: HashMap<String, String>,
    pub narrative_fragments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SNTType {
    KeeperIdentity,
    StorageContribution,
    MemoryAnchor,
    FusionMaster,
    GlyphArtist,
    CommunityBond,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub element: Element,
    pub category: DataCategory,
    pub importance: Importance,
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Element {
    Fire,    // 🔥 Active data, hot storage
    Water,   // 💧 Flowing data, streams
    Earth,   // 🌍 Stable data, archives
    Air,     // 💨 Ephemeral data, temporary
    Lightning, // ⚡ Fast data, real-time
    Void,    // 🌙 Hidden data, encrypted
    Aether,  // 🔮 Transcendent data, AI models
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataCategory {
    Archive,   // Raw file storage
    Model,     // ML models
    Dataset,   // Training data
    Result,    // Computation results
    Media,     // Images, video, audio
    Document,  // Text files
    Code,      // Software
    Identity,  // Profile data
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Importance {
    Trivial,
    Minor,
    Standard,
    Major,
    Critical,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource_type: String,
    pub access_level: AccessLevel,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Read,
    Write,
    Execute,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sigil {
    pub sigil_id: String,
    pub filename: Option<String>,
    pub size: usize,
    pub glyph: Glyph,
    pub owner: String,
    pub stored_at: Vec<String>, // Keeper IDs
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keeper {
    pub keeper_id: String,
    pub name: String,
    pub capacity: u64,
    pub used_storage: u64,
    pub reputation: f64,
    pub sigils_hosted: Vec<String>,
    pub total_earned: u64,
    pub status: KeeperStatus,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeeperStatus {
    Apprentice,  // New keeper
    Guardian,    // Established keeper
    Archivist,   // Senior keeper
    Loremaster,  // Master keeper
}

/// Main SNT system managing all tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNTSystem {
    pub active_snts: HashMap<String, SNT>,
    pub snt_templates: Vec<SNTTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNTTemplate {
    pub template_id: String,
    pub name: String,
    pub token_type: SNTType,
    pub default_glyph: Glyph,
    pub base_permissions: Vec<Permission>,
}

impl SNTSystem {
    pub fn new() -> Self {
        let mut system = Self {
            active_snts: HashMap::new(),
            snt_templates: Vec::new(),
        };
        system.initialize_templates();
        system
    }

    fn initialize_templates(&mut self) {
        self.snt_templates = vec![
            SNTTemplate {
                template_id: "keeper_identity".to_string(),
                name: "🛡️ Keeper Identity".to_string(),
                token_type: SNTType::KeeperIdentity,
                default_glyph: Glyph {
                    element: Element::Earth,
                    category: DataCategory::Identity,
                    importance: Importance::Major,
                    symbol: "🛡️".to_string(),
                },
                base_permissions: vec![
                    Permission {
                        resource_type: "network_access".to_string(),
                        access_level: AccessLevel::Read,
                        conditions: vec!["active_keeper".to_string()],
                    }
                ],
            },
            SNTTemplate {
                template_id: "storage_contribution".to_string(),
                name: "📦 Storage Contribution".to_string(),
                token_type: SNTType::StorageContribution,
                default_glyph: Glyph {
                    element: Element::Fire,
                    category: DataCategory::Archive,
                    importance: Importance::Standard,
                    symbol: "📦".to_string(),
                },
                base_permissions: vec![],
            },
            SNTTemplate {
                template_id: "memory_anchor".to_string(),
                name: "📜 Memory Anchor".to_string(),
                token_type: SNTType::MemoryAnchor,
                default_glyph: Glyph {
                    element: Element::Void,
                    category: DataCategory::Archive,
                    importance: Importance::Legendary,
                    symbol: "📜".to_string(),
                },
                base_permissions: vec![],
            },
            SNTTemplate {
                template_id: "fusion_master".to_string(),
                name: "⚗️ Fusion Master".to_string(),
                token_type: SNTType::FusionMaster,
                default_glyph: Glyph {
                    element: Element::Aether,
                    category: DataCategory::Result,
                    importance: Importance::Critical,
                    symbol: "⚗️".to_string(),
                },
                base_permissions: vec![
                    Permission {
                        resource_type: "fusion_forge".to_string(),
                        access_level: AccessLevel::Execute,
                        conditions: vec!["ritual_participant".to_string()],
                    }
                ],
            },
        ];
    }

    pub fn mint_snt(&mut self, template_id: &str, holder: String, context: HashMap<String, String>) -> Result<String> {
        let template = self.snt_templates.iter()
            .find(|t| t.template_id == template_id)
            .ok_or_else(|| anyhow!("Template not found"))?;

        let snt_id = format!("snt_{}", Uuid::new_v4().simple());
        
        let snt = SNT {
            snt_id: snt_id.clone(),
            holder: holder.clone(),
            token_type: template.token_type.clone(),
            glyph: template.default_glyph.clone(),
            created_at: current_timestamp(),
            evolution_level: 1,
            evolution_progress: 0.0,
            permissions: template.base_permissions.clone(),
            properties: context,
            narrative_fragments: vec![
                format!("Born from the network's recognition of {}'s contribution", holder)
            ],
        };

        self.active_snts.insert(snt_id.clone(), snt);
        Ok(snt_id)
    }

    pub fn evolve_snt(&mut self, snt_id: &str, activity_points: f64) -> Result<bool> {
        let snt = self.active_snts.get_mut(snt_id)
            .ok_or_else(|| anyhow!("SNT not found"))?;

        snt.evolution_progress += activity_points;
        
        // Check if ready to evolve
        let evolution_threshold = (snt.evolution_level as f64 * 100.0) + 50.0;
        
        if snt.evolution_progress >= evolution_threshold {
            snt.evolution_level += 1;
            snt.evolution_progress = 0.0;
            
            // Add evolution narrative
            snt.narrative_fragments.push(
                format!("Evolved to level {} through dedicated network participation", snt.evolution_level)
            );
            
            // Potentially upgrade glyph importance
            if snt.evolution_level >= 5 && snt.glyph.importance == Importance::Standard {
                snt.glyph.importance = Importance::Major;
            } else if snt.evolution_level >= 10 && snt.glyph.importance == Importance::Major {
                snt.glyph.importance = Importance::Critical;
            }
            
            return Ok(true);
        }
        
        Ok(false)
    }

    pub fn get_holder_snts(&self, holder: &str) -> Vec<&SNT> {
        self.active_snts.values()
            .filter(|snt| snt.holder == holder)
            .collect()
    }

    pub fn get_stats(&self) -> SNTStats {
        let mut type_counts = HashMap::new();
        let mut total_evolution_levels = 0;
        
        for snt in self.active_snts.values() {
            *type_counts.entry(format!("{:?}", snt.token_type)).or_insert(0) += 1;
            total_evolution_levels += snt.evolution_level;
        }

        SNTStats {
            total_snts: self.active_snts.len(),
            type_distribution: type_counts,
            average_evolution_level: if self.active_snts.is_empty() { 
                0.0 
            } else { 
                total_evolution_levels as f64 / self.active_snts.len() as f64 
            },
            unique_holders: self.active_snts.values()
                .map(|snt| &snt.holder)
                .collect::<std::collections::HashSet<_>>()
                .len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SNTStats {
    pub total_snts: usize,
    pub type_distribution: HashMap<String, usize>,
    pub average_evolution_level: f64,
    pub unique_holders: usize,
}

impl Element {
    pub fn symbol(&self) -> &'static str {
        match self {
            Element::Fire => "🔥",
            Element::Water => "💧",
            Element::Earth => "🌍",
            Element::Air => "💨",
            Element::Lightning => "⚡",
            Element::Void => "🌙",
            Element::Aether => "🔮",
        }
    }

    pub fn from_string(s: &str) -> Element {
        match s {
            "🔥 Fire" => Element::Fire,
            "💧 Water" => Element::Water,
            "🌍 Earth" => Element::Earth,
            "💨 Air" => Element::Air,
            "⚡ Lightning" => Element::Lightning,
            "🌙 Void" => Element::Void,
            "🔮 Aether" => Element::Aether,
            _ => Element::Earth,
        }
    }
}

impl DataCategory {
    pub fn from_string(s: &str) -> DataCategory {
        match s {
            "📦 Archive" => DataCategory::Archive,
            "🧠 Model" => DataCategory::Model,
            "📊 Dataset" => DataCategory::Dataset,
            "🎯 Result" => DataCategory::Result,
            "🎬 Media" => DataCategory::Media,
            "📄 Document" | _ => DataCategory::Document,
        }
    }
}

impl Importance {
    pub fn from_string(s: &str) -> Importance {
        match s {
            "✨ Trivial" => Importance::Trivial,
            "📎 Minor" => Importance::Minor,
            "📋 Standard" => Importance::Standard,
            "⭐ Major" => Importance::Major,
            "🔥 Critical" => Importance::Critical,
            "👑 Legendary" => Importance::Legendary,
            _ => Importance::Standard,
        }
    }

    pub fn multiplier(&self) -> f64 {
        match self {
            Importance::Trivial => 0.5,
            Importance::Minor => 0.8,
            Importance::Standard => 1.0,
            Importance::Major => 1.5,
            Importance::Critical => 2.0,
            Importance::Legendary => 3.0,
        }
    }
}

impl KeeperStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            KeeperStatus::Apprentice => "🔰",
            KeeperStatus::Guardian => "🛡️",
            KeeperStatus::Archivist => "📚",
            KeeperStatus::Loremaster => "🎭",
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}