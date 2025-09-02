use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Keeper, MetaspaceStats, Sigil};
use crate::Amount;

/// Main UI state and controller for the Mythic Metaspace Dashboard
pub struct MetaspaceUI {
    /// Current view mode
    pub view_mode: ViewMode,

    /// Selected filters
    pub filters: UIFilters,

    /// Animation settings
    pub animations: AnimationSettings,

    /// Theme configuration
    pub theme: UITheme,

    /// Real-time data cache for performance
    pub data_cache: UIDataCache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewMode {
    /// Main dashboard overview
    Overview,

    /// Browse sigils by glyph, type, contributor
    SigilExplorer,

    /// Visualize keeper network as living map
    KeeperMap,

    /// View earnings, hosted sigils, retrieval logs
    ContributionLedger,

    /// Combine sigils into bundles/vaults
    FusionForge,

    /// Create custom glyphs and symbolic tags
    GlyphDesigner,

    /// Network health and activity monitoring
    NetworkPulse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIFilters {
    /// Filter by glyph elements (Fire, Water, Earth, etc.)
    pub elements: Vec<String>,

    /// Filter by data categories
    pub categories: Vec<String>,

    /// Filter by importance levels
    pub importance_levels: Vec<String>,

    /// Filter by keeper status
    pub keeper_status: Vec<String>,

    /// Search text
    pub search_query: String,

    /// Date range filter
    pub date_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSettings {
    /// Enable flowing particle effects for token transfers
    pub token_flow_animation: bool,

    /// Enable keeper heartbeat pulsing
    pub keeper_heartbeats: bool,

    /// Enable sigil fusion animations
    pub sigil_fusion_effects: bool,

    /// Enable network pulse visualization
    pub network_pulse: bool,

    /// Animation speed multiplier (0.5 = slow, 2.0 = fast)
    pub animation_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UITheme {
    /// Mystical dark theme with glowing elements
    MythicDark,

    /// Arcane terminal theme
    ArcaneTerminal,

    /// Crystal interface theme
    CrystalInterface,

    /// Classic clean theme
    Classic,

    /// High contrast accessibility theme
    HighContrast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIDataCache {
    /// Cached metaspace statistics
    pub metaspace_stats: Option<MetaspaceStats>,

    /// Last update timestamp
    pub last_update: chrono::DateTime<chrono::Utc>,

    /// Cache TTL in seconds
    pub ttl_seconds: u64,

    /// Cached sigil data for explorer
    pub sigil_cache: HashMap<String, SigilDisplayData>,

    /// Cached keeper data for map
    pub keeper_cache: HashMap<String, KeeperDisplayData>,
}

/// Display-optimized sigil data for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigilDisplayData {
    pub content_hash: String,
    pub glyph_symbol: String,
    pub glyph_name: String,
    pub size_formatted: String,
    pub age_formatted: String,
    pub keeper_count: usize,
    pub retrieval_count: u32,
    pub originator_short: String,
    pub importance_color: String,
    pub dna_string: String,
}

/// Display-optimized keeper data for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperDisplayData {
    pub keeper_id: String,
    pub keeper_id_short: String,
    pub status_icon: String,
    pub status_name: String,
    pub reputation_percentage: u8,
    pub uptime_formatted: String,
    pub storage_used_percentage: u8,
    pub sigil_count: usize,
    pub total_earned_formatted: String,
    pub response_time_avg: String,
    pub location_estimate: Option<NetworkLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLocation {
    pub region: String,
    pub estimated_lat: f64,
    pub estimated_lng: f64,
}

impl MetaspaceUI {
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::Overview,
            filters: UIFilters::default(),
            animations: AnimationSettings::default(),
            theme: UITheme::MythicDark,
            data_cache: UIDataCache::new(),
        }
    }

    /// Switch to a different view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Update filters and refresh data
    pub fn update_filters(&mut self, filters: UIFilters) {
        self.filters = filters;
        // Trigger data refresh
        self.invalidate_cache();
    }

    /// Convert raw sigil data to display format
    pub fn format_sigil_for_display(&self, sigil: &Sigil) -> SigilDisplayData {
        SigilDisplayData {
            content_hash: sigil.content_hash.clone(),
            glyph_symbol: sigil.glyph.symbol().to_string(),
            glyph_name: sigil.glyph.name(),
            size_formatted: Self::format_bytes(sigil.size),
            age_formatted: Self::format_duration_since(sigil.created_at),
            keeper_count: sigil.keepers.len(),
            retrieval_count: sigil.retrievals.len() as u32,
            originator_short: Self::shorten_id(&sigil.originator),
            importance_color: Self::importance_to_color(&sigil.glyph.importance),
            dna_string: sigil.get_dna_string(),
        }
    }

    /// Convert raw keeper data to display format
    pub fn format_keeper_for_display(&self, keeper: &Keeper) -> KeeperDisplayData {
        KeeperDisplayData {
            keeper_id: keeper.keeper_id.clone(),
            keeper_id_short: Self::shorten_id(&keeper.keeper_id),
            status_icon: keeper.status.to_string(),
            status_name: format!("{:?}", keeper.status),
            reputation_percentage: (keeper.reputation * 100.0) as u8,
            uptime_formatted: Self::format_duration(keeper.uptime_hours()),
            storage_used_percentage: keeper.storage_utilization() as u8,
            sigil_count: keeper.sigils_held.len(),
            total_earned_formatted: Self::format_par_amount(keeper.total_earned),
            response_time_avg: format!("{}ms", keeper.metrics.avg_response_time_ms as u32),
            location_estimate: Self::estimate_location(&keeper.network_address),
        }
    }

    /// Generate dashboard overview data
    pub fn generate_overview_data(&self, stats: &MetaspaceStats) -> OverviewData {
        OverviewData {
            total_sigils: stats.total_sigils,
            active_keepers: stats.active_keepers,
            total_keepers: stats.total_keepers,
            reward_pool_par: Self::format_par_amount(stats.reward_pool_balance),
            pending_rewards_par: Self::format_par_amount(stats.pending_rewards_total),
            network_health_score: Self::calculate_network_health(stats),
            recent_activity: self.generate_activity_feed(),
            top_keepers: self.generate_top_keepers_list(),
            glyph_distribution: self.generate_glyph_distribution(),
        }
    }

    /// Generate activity feed for dashboard
    fn generate_activity_feed(&self) -> Vec<ActivityItem> {
        // In a real implementation, this would query recent events
        vec![
            ActivityItem {
                icon: "🔥".to_string(),
                message: "New ML model stored with Fire glyph".to_string(),
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(5),
                activity_type: ActivityType::SigilStored,
            },
            ActivityItem {
                icon: "🛡️".to_string(),
                message: "Keeper_abc123 promoted to Guardian status".to_string(),
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(12),
                activity_type: ActivityType::KeeperPromotion,
            },
            ActivityItem {
                icon: "💰".to_string(),
                message: "50 PAR distributed to 5 keepers".to_string(),
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(18),
                activity_type: ActivityType::RewardDistribution,
            },
        ]
    }

    /// Generate top keepers leaderboard
    fn generate_top_keepers_list(&self) -> Vec<TopKeeperEntry> {
        // Mock data - in reality this would query actual keeper stats
        vec![
            TopKeeperEntry {
                keeper_id: "keeper_loremaster_01".to_string(),
                status_icon: "🎭".to_string(),
                reputation: 0.98,
                total_earned: 15000_00000000,
                sigil_count: 1250,
            },
            TopKeeperEntry {
                keeper_id: "keeper_archivist_07".to_string(),
                status_icon: "📚".to_string(),
                reputation: 0.95,
                total_earned: 12500_00000000,
                sigil_count: 980,
            },
            TopKeeperEntry {
                keeper_id: "keeper_guardian_23".to_string(),
                status_icon: "🛡️".to_string(),
                reputation: 0.89,
                total_earned: 8900_00000000,
                sigil_count: 750,
            },
        ]
    }

    /// Generate glyph distribution chart data
    fn generate_glyph_distribution(&self) -> Vec<GlyphDistributionEntry> {
        vec![
            GlyphDistributionEntry {
                element: "Fire".to_string(),
                symbol: "🔥".to_string(),
                count: 450,
                percentage: 30,
            },
            GlyphDistributionEntry {
                element: "Earth".to_string(),
                symbol: "🌍".to_string(),
                count: 380,
                percentage: 25,
            },
            GlyphDistributionEntry {
                element: "Water".to_string(),
                symbol: "💧".to_string(),
                count: 290,
                percentage: 19,
            },
            GlyphDistributionEntry {
                element: "Air".to_string(),
                symbol: "💨".to_string(),
                count: 220,
                percentage: 15,
            },
            GlyphDistributionEntry {
                element: "Lightning".to_string(),
                symbol: "⚡".to_string(),
                count: 105,
                percentage: 7,
            },
            GlyphDistributionEntry {
                element: "Void".to_string(),
                symbol: "🌙".to_string(),
                count: 45,
                percentage: 3,
            },
            GlyphDistributionEntry {
                element: "Aether".to_string(),
                symbol: "🔮".to_string(),
                count: 15,
                percentage: 1,
            },
        ]
    }

    // Utility formatting functions
    fn format_bytes(bytes: usize) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if size < 10.0 && unit_index > 0 {
            format!("{:.1} {}", size, UNITS[unit_index])
        } else {
            format!("{:.0} {}", size, UNITS[unit_index])
        }
    }

    fn format_par_amount(amount: Amount) -> String {
        let par = amount as f64 / 1_00000000.0;
        if par >= 1000000.0 {
            format!("{:.1}M PAR", par / 1000000.0)
        } else if par >= 1000.0 {
            format!("{:.1}K PAR", par / 1000.0)
        } else if par >= 1.0 {
            format!("{:.2} PAR", par)
        } else {
            format!("{:.4} PAR", par)
        }
    }

    fn format_duration_since(timestamp: chrono::DateTime<chrono::Utc>) -> String {
        let duration = chrono::Utc::now().signed_duration_since(timestamp);
        Self::format_duration_chrono(duration)
    }

    fn format_duration(hours: f64) -> String {
        if hours >= 24.0 {
            format!("{:.1} days", hours / 24.0)
        } else if hours >= 1.0 {
            format!("{:.1} hours", hours)
        } else {
            format!("{:.0} minutes", hours * 60.0)
        }
    }

    fn format_duration_chrono(duration: chrono::Duration) -> String {
        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes ago", duration.num_minutes())
        } else {
            "Just now".to_string()
        }
    }

    fn shorten_id(id: &str) -> String {
        if id.len() > 12 {
            format!("{}...{}", &id[0..6], &id[id.len() - 4..])
        } else {
            id.to_string()
        }
    }

    fn importance_to_color(importance: &super::glyph::Importance) -> String {
        match importance {
            super::glyph::Importance::Critical => "#ff4444".to_string(),
            super::glyph::Importance::Major => "#ff8800".to_string(),
            super::glyph::Importance::Standard => "#00aa00".to_string(),
            super::glyph::Importance::Minor => "#888888".to_string(),
            super::glyph::Importance::Trivial => "#444444".to_string(),
            super::glyph::Importance::Legendary => "#ff00ff".to_string(),
        }
    }

    fn calculate_network_health(stats: &MetaspaceStats) -> u8 {
        // Simple health calculation - in reality this would be more sophisticated
        let base_score = if stats.total_keepers > 0 {
            (stats.active_keepers * 100 / stats.total_keepers) as u8
        } else {
            0
        };

        let sigil_factor = if stats.total_sigils > 100 {
            100
        } else {
            stats.total_sigils
        } as u8;
        let proof_factor = if stats.total_proofs > 50 {
            100
        } else {
            (stats.total_proofs * 2) as u8
        };

        ((base_score + sigil_factor + proof_factor) / 3).min(100)
    }

    fn estimate_location(address: &str) -> Option<NetworkLocation> {
        // In a real implementation, this would use GeoIP or network topology
        // For now, return mock data
        Some(NetworkLocation {
            region: "Unknown".to_string(),
            estimated_lat: 0.0,
            estimated_lng: 0.0,
        })
    }

    fn invalidate_cache(&mut self) {
        self.data_cache.last_update =
            chrono::Utc::now() - chrono::Duration::seconds(self.data_cache.ttl_seconds as i64 + 1);
    }
}

// Data structures for UI components

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewData {
    pub total_sigils: usize,
    pub active_keepers: usize,
    pub total_keepers: usize,
    pub reward_pool_par: String,
    pub pending_rewards_par: String,
    pub network_health_score: u8,
    pub recent_activity: Vec<ActivityItem>,
    pub top_keepers: Vec<TopKeeperEntry>,
    pub glyph_distribution: Vec<GlyphDistributionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub icon: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub activity_type: ActivityType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    SigilStored,
    SigilRetrieved,
    KeeperJoined,
    KeeperPromotion,
    RewardDistribution,
    ProofChallenge,
    NetworkEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopKeeperEntry {
    pub keeper_id: String,
    pub status_icon: String,
    pub reputation: f64,
    pub total_earned: Amount,
    pub sigil_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphDistributionEntry {
    pub element: String,
    pub symbol: String,
    pub count: usize,
    pub percentage: u8,
}

// Default implementations

impl Default for UIFilters {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            categories: Vec::new(),
            importance_levels: Vec::new(),
            keeper_status: Vec::new(),
            search_query: String::new(),
            date_range: None,
        }
    }
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            token_flow_animation: true,
            keeper_heartbeats: true,
            sigil_fusion_effects: true,
            network_pulse: true,
            animation_speed: 1.0,
        }
    }
}

impl UIDataCache {
    pub fn new() -> Self {
        Self {
            metaspace_stats: None,
            last_update: chrono::Utc::now() - chrono::Duration::hours(1), // Force initial update
            ttl_seconds: 30,
            sigil_cache: HashMap::new(),
            keeper_cache: HashMap::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        now.signed_duration_since(self.last_update).num_seconds() > self.ttl_seconds as i64
    }

    pub fn update(&mut self, stats: MetaspaceStats) {
        self.metaspace_stats = Some(stats);
        self.last_update = chrono::Utc::now();
    }
}
