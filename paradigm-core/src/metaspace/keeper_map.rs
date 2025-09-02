use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ui::KeeperDisplayData;
use super::{Keeper, KeeperStatus};

/// Keeper Map - Visualize the keeper network as a living constellation
pub struct KeeperMap {
    /// Network topology configuration
    pub topology: NetworkTopology,

    /// Visual layout engine
    pub layout: MapLayout,

    /// Current view settings
    pub view: MapView,

    /// Animation state
    pub animations: MapAnimations,

    /// Keeper positions and visual state
    pub keeper_positions: HashMap<String, KeeperPosition>,

    /// Connection graph between keepers
    pub connections: Vec<KeeperConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    /// Center of the network (usually most reputable keeper)
    pub center_keeper: Option<String>,

    /// Network clusters by region/performance
    pub clusters: Vec<KeeperCluster>,

    /// Force-directed layout parameters
    pub layout_forces: ForceParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperCluster {
    pub cluster_id: String,
    pub center_position: Position2D,
    pub radius: f32,
    pub keeper_ids: Vec<String>,
    pub cluster_type: ClusterType,
    pub health_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterType {
    /// High-performance keepers (Lorekeepers, Archivists)
    Elite,

    /// Stable, reliable keepers (Guardians)
    Core,

    /// New or recovering keepers (Apprentices)
    Emerging,

    /// Keepers with issues (Probation, Dormant)
    Troubled,

    /// Geographic region cluster
    Regional(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceParameters {
    /// Attraction between connected keepers
    pub connection_strength: f32,

    /// Repulsion between all keepers (prevents overlap)
    pub repulsion_strength: f32,

    /// Attraction to cluster centers
    pub cluster_attraction: f32,

    /// Damping factor for stability
    pub damping: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLayout {
    /// Layout algorithm type
    pub algorithm: LayoutAlgorithm,

    /// Canvas dimensions
    pub canvas_size: (f32, f32),

    /// Zoom level (1.0 = default)
    pub zoom: f32,

    /// Camera center position
    pub camera_center: Position2D,

    /// Auto-layout enabled
    pub auto_layout: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutAlgorithm {
    /// Force-directed spring layout
    ForceDirected,

    /// Hierarchical layout by reputation
    Hierarchical,

    /// Geographic layout (if location data available)
    Geographic,

    /// Circular layout with reputation rings
    Circular,

    /// Grid layout for equal spacing
    Grid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapView {
    /// What information to display on keepers
    pub display_mode: DisplayMode,

    /// Color scheme
    pub color_scheme: ColorScheme,

    /// Show connections between keepers
    pub show_connections: bool,

    /// Show sigil flow animations
    pub show_sigil_flow: bool,

    /// Show keeper status badges
    pub show_status_badges: bool,

    /// Highlight selection
    pub selected_keeper: Option<String>,

    /// Highlight keepers storing specific sigil
    pub highlighted_sigil: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayMode {
    /// Show keeper status and basic info
    Status,

    /// Show storage utilization
    Storage,

    /// Show reputation and performance
    Reputation,

    /// Show earnings and rewards
    Earnings,

    /// Show sigil count and types
    SigilCount,

    /// Minimal view with just status icons
    Minimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    /// Status-based colors (green=good, red=issues)
    Status,

    /// Reputation-based gradient (blue to gold)
    Reputation,

    /// Glyph element colors (fire=red, water=blue, etc.)
    Elements,

    /// Earnings-based (cold to hot colors)
    Earnings,

    /// Mythic theme (purples, gold, dark)
    Mythic,

    /// High contrast for accessibility
    HighContrast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapAnimations {
    /// Keeper heartbeat pulsing
    pub keeper_heartbeats: bool,

    /// Token flow between keepers
    pub token_flows: bool,

    /// Sigil distribution animations
    pub sigil_distribution: bool,

    /// Network health pulse
    pub network_pulse: bool,

    /// Keeper status change animations
    pub status_transitions: bool,

    /// Animation speed multiplier
    pub speed_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperPosition {
    /// Current position on the map
    pub position: Position2D,

    /// Target position (for smooth animations)
    pub target_position: Position2D,

    /// Movement velocity for physics simulation
    pub velocity: Vector2D,

    /// Visual size based on importance/reputation
    pub size: f32,

    /// Current color based on display mode
    pub color: Color,

    /// Status effects (pulsing, glowing, etc.)
    pub effects: Vec<VisualEffect>,

    /// Last activity timestamp for idle detection
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualEffect {
    /// Pulsing glow for active keepers
    Heartbeat { frequency: f32, intensity: f32 },

    /// Incoming/outgoing sigil flow
    SigilFlow {
        direction: FlowDirection,
        count: u32,
    },

    /// Reward distribution effect
    TokenBurst { amount: u64 },

    /// Status change effect
    StatusChange {
        from: KeeperStatus,
        to: KeeperStatus,
    },

    /// Connection activity
    NetworkActivity { connections: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowDirection {
    Incoming,
    Outgoing,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperConnection {
    pub from_keeper: String,
    pub to_keeper: String,
    pub connection_type: ConnectionType,
    pub strength: f32,
    pub activity_level: f32,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Keepers storing the same sigil
    SharedSigil(String),

    /// Geographic proximity
    Geographic,

    /// Similar performance/reputation
    PeerRelation,

    /// Sigil transfer/replication
    DataTransfer,

    /// Challenge/verification interaction
    ProofExchange,
}

impl KeeperMap {
    pub fn new() -> Self {
        Self {
            topology: NetworkTopology::default(),
            layout: MapLayout::default(),
            view: MapView::default(),
            animations: MapAnimations::default(),
            keeper_positions: HashMap::new(),
            connections: Vec::new(),
        }
    }

    /// Update the map with current keeper data
    pub async fn update_keepers(&mut self, keepers: &HashMap<String, Keeper>) -> Result<()> {
        // Update keeper positions and visual data
        for (keeper_id, keeper) in keepers {
            // Create initial position if needed
            if !self.keeper_positions.contains_key(keeper_id) {
                let initial_pos = self.create_initial_position(keeper);
                self.keeper_positions.insert(keeper_id.clone(), initial_pos);
            }

            // Get the position for updates
            if let Some(position) = self.keeper_positions.get_mut(keeper_id) {
                // Simple update without borrowing conflicts
                position.size = 16.0; // Default size for now
                position.last_activity = chrono::Utc::now();
            }
        }

        // Remove keepers that no longer exist
        self.keeper_positions
            .retain(|id, _| keepers.contains_key(id));

        // Update network topology
        self.update_network_topology(keepers).await?;

        // Update connections
        self.update_connections(keepers).await?;

        // Run layout algorithm if auto-layout is enabled
        if self.layout.auto_layout {
            self.run_layout_algorithm().await?;
        }

        Ok(())
    }

    /// Create initial position for a new keeper
    fn create_initial_position(&self, keeper: &Keeper) -> KeeperPosition {
        // Place new keepers in a spiral around the center
        let angle = (self.keeper_positions.len() as f32) * 2.3; // Golden angle for even distribution
        let radius = 100.0 + (keeper.reputation * 200.0) as f32;

        let x = self.layout.canvas_size.0 / 2.0 + angle.cos() * radius;
        let y = self.layout.canvas_size.1 / 2.0 + angle.sin() * radius;

        KeeperPosition {
            position: Position2D { x, y },
            target_position: Position2D { x, y },
            velocity: Vector2D { x: 0.0, y: 0.0 },
            size: self.calculate_keeper_size(keeper),
            color: self.calculate_keeper_color(keeper),
            effects: Vec::new(),
            last_activity: chrono::Utc::now(),
        }
    }

    /// Update keeper visual effects
    fn update_keeper_visual_effects(&self, position: &mut KeeperPosition, keeper: &Keeper) {
        position.effects.clear();

        // Add heartbeat effect for active keepers
        if self.animations.keeper_heartbeats && keeper.is_online() {
            let frequency = match keeper.status {
                KeeperStatus::Loremaster => 1.5,
                KeeperStatus::Archivist => 1.2,
                KeeperStatus::Guardian => 1.0,
                KeeperStatus::Apprentice => 0.8,
                _ => 0.5,
            };

            position.effects.push(VisualEffect::Heartbeat {
                frequency,
                intensity: keeper.reputation as f32,
            });
        }

        // Add sigil flow effect if recent activity
        if keeper.metrics.successful_retrievals > 0 {
            position.effects.push(VisualEffect::SigilFlow {
                direction: FlowDirection::Bidirectional,
                count: (keeper.metrics.successful_retrievals % 10) as u32,
            });
        }
    }

    /// Calculate visual size for a keeper based on importance
    fn calculate_keeper_size(&self, keeper: &Keeper) -> f32 {
        let base_size = match keeper.status {
            KeeperStatus::Loremaster => 24.0,
            KeeperStatus::Archivist => 20.0,
            KeeperStatus::Guardian => 16.0,
            KeeperStatus::Apprentice => 12.0,
            KeeperStatus::Probation => 10.0,
            KeeperStatus::Dormant => 8.0,
        };

        // Scale by reputation
        base_size * (0.8 + (keeper.reputation as f32) * 0.4)
    }

    /// Calculate color for a keeper based on current display mode
    fn calculate_keeper_color(&self, keeper: &Keeper) -> Color {
        match self.view.display_mode {
            DisplayMode::Status => self.status_color(keeper),
            DisplayMode::Reputation => self.reputation_color(keeper),
            DisplayMode::Storage => self.storage_color(keeper),
            DisplayMode::Earnings => self.earnings_color(keeper),
            _ => self.default_color(keeper),
        }
    }

    fn status_color(&self, keeper: &Keeper) -> Color {
        match keeper.status {
            KeeperStatus::Loremaster => Color {
                r: 255,
                g: 215,
                b: 0,
                a: 255,
            }, // Gold
            KeeperStatus::Archivist => Color {
                r: 138,
                g: 43,
                b: 226,
                a: 255,
            }, // Blue Violet
            KeeperStatus::Guardian => Color {
                r: 0,
                g: 128,
                b: 0,
                a: 255,
            }, // Green
            KeeperStatus::Apprentice => Color {
                r: 70,
                g: 130,
                b: 180,
                a: 255,
            }, // Steel Blue
            KeeperStatus::Probation => Color {
                r: 255,
                g: 140,
                b: 0,
                a: 255,
            }, // Orange
            KeeperStatus::Dormant => Color {
                r: 128,
                g: 128,
                b: 128,
                a: 255,
            }, // Gray
        }
    }

    fn reputation_color(&self, keeper: &Keeper) -> Color {
        let intensity = (keeper.reputation * 255.0) as u8;
        Color {
            r: 0,
            g: intensity,
            b: 255 - intensity,
            a: 255,
        }
    }

    fn storage_color(&self, keeper: &Keeper) -> Color {
        let utilization = keeper.storage_utilization() as u8;
        Color {
            r: utilization,
            g: 255 - utilization,
            b: 0,
            a: 255,
        }
    }

    fn earnings_color(&self, keeper: &Keeper) -> Color {
        let earnings_normalized = (keeper.total_earned as f32 / 1000_00000000.0).min(1.0); // Normalize to 1000 PAR max
        let intensity = (earnings_normalized * 255.0) as u8;
        Color {
            r: intensity,
            g: intensity / 2,
            b: 255 - intensity,
            a: 255,
        }
    }

    fn default_color(&self, keeper: &Keeper) -> Color {
        if keeper.is_online() {
            Color {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            } // Green for online
        } else {
            Color {
                r: 128,
                g: 128,
                b: 128,
                a: 255,
            } // Gray for offline
        }
    }

    /// Update network topology clusters
    async fn update_network_topology(&mut self, keepers: &HashMap<String, Keeper>) -> Result<()> {
        // Clear existing clusters
        self.topology.clusters.clear();

        // Create clusters by keeper status
        let mut status_groups: HashMap<KeeperStatus, Vec<String>> = HashMap::new();
        for (keeper_id, keeper) in keepers {
            status_groups
                .entry(keeper.status.clone())
                .or_insert_with(Vec::new)
                .push(keeper_id.clone());
        }

        // Convert status groups to clusters
        for (status, keeper_ids) in status_groups {
            if !keeper_ids.is_empty() {
                let cluster_type = match status {
                    KeeperStatus::Loremaster | KeeperStatus::Archivist => ClusterType::Elite,
                    KeeperStatus::Guardian => ClusterType::Core,
                    KeeperStatus::Apprentice => ClusterType::Emerging,
                    _ => ClusterType::Troubled,
                };

                let cluster = KeeperCluster {
                    cluster_id: format!("{:?}", status),
                    center_position: self.calculate_cluster_center(&keeper_ids),
                    radius: 80.0 + (keeper_ids.len() as f32 * 10.0),
                    keeper_ids,
                    cluster_type,
                    health_score: self.calculate_cluster_health(&status, keepers),
                };

                self.topology.clusters.push(cluster);
            }
        }

        Ok(())
    }

    fn calculate_cluster_center(&self, keeper_ids: &[String]) -> Position2D {
        if keeper_ids.is_empty() {
            return Position2D { x: 0.0, y: 0.0 };
        }

        let mut total_x = 0.0;
        let mut total_y = 0.0;
        let mut count = 0;

        for keeper_id in keeper_ids {
            if let Some(position) = self.keeper_positions.get(keeper_id) {
                total_x += position.position.x;
                total_y += position.position.y;
                count += 1;
            }
        }

        if count > 0 {
            Position2D {
                x: total_x / count as f32,
                y: total_y / count as f32,
            }
        } else {
            Position2D {
                x: self.layout.canvas_size.0 / 2.0,
                y: self.layout.canvas_size.1 / 2.0,
            }
        }
    }

    fn calculate_cluster_health(
        &self,
        status: &KeeperStatus,
        keepers: &HashMap<String, Keeper>,
    ) -> f32 {
        let cluster_keepers: Vec<_> = keepers.values().filter(|k| &k.status == status).collect();

        if cluster_keepers.is_empty() {
            return 0.0;
        }

        let avg_reputation: f32 = cluster_keepers.iter().map(|k| k.reputation).sum::<f64>() as f32
            / cluster_keepers.len() as f32;

        let online_ratio = cluster_keepers.iter().filter(|k| k.is_online()).count() as f32
            / cluster_keepers.len() as f32;

        (avg_reputation + online_ratio) / 2.0
    }

    /// Update connections between keepers
    async fn update_connections(&mut self, keepers: &HashMap<String, Keeper>) -> Result<()> {
        self.connections.clear();

        // Create connections for keepers storing the same sigils
        let mut sigil_keepers: HashMap<String, Vec<String>> = HashMap::new();

        for (keeper_id, keeper) in keepers {
            for sigil_hash in &keeper.sigils_held {
                sigil_keepers
                    .entry(sigil_hash.clone())
                    .or_insert_with(Vec::new)
                    .push(keeper_id.clone());
            }
        }

        // Create connections between keepers sharing sigils
        for (sigil_hash, keeper_ids) in sigil_keepers {
            for i in 0..keeper_ids.len() {
                for j in i + 1..keeper_ids.len() {
                    let connection = KeeperConnection {
                        from_keeper: keeper_ids[i].clone(),
                        to_keeper: keeper_ids[j].clone(),
                        connection_type: ConnectionType::SharedSigil(sigil_hash.clone()),
                        strength: 0.5, // Base strength for shared sigils
                        activity_level: 1.0,
                        last_activity: chrono::Utc::now(),
                    };

                    self.connections.push(connection);
                }
            }
        }

        Ok(())
    }

    /// Run the selected layout algorithm
    async fn run_layout_algorithm(&mut self) -> Result<()> {
        match self.layout.algorithm {
            LayoutAlgorithm::ForceDirected => self.run_force_directed_layout(),
            LayoutAlgorithm::Hierarchical => self.run_hierarchical_layout(),
            LayoutAlgorithm::Circular => self.run_circular_layout(),
            _ => Ok(()), // Other algorithms would be implemented
        }
    }

    fn run_force_directed_layout(&mut self) -> Result<()> {
        let forces = &self.topology.layout_forces;

        // Collect all positions first to avoid borrowing issues
        let positions: Vec<_> = self
            .keeper_positions
            .iter()
            .map(|(id, pos)| (id.clone(), pos.position.clone()))
            .collect();

        // Calculate forces for each keeper
        let mut force_updates: HashMap<String, (f32, f32)> = HashMap::new();

        for (keeper_id, position) in &positions {
            let mut force_x = 0.0;
            let mut force_y = 0.0;

            // Repulsion from other keepers
            for (other_id, other_position) in &positions {
                if keeper_id == other_id {
                    continue;
                }

                let dx = position.x - other_position.x;
                let dy = position.y - other_position.y;
                let distance = (dx * dx + dy * dy).sqrt().max(1.0);

                let repulsion = forces.repulsion_strength / (distance * distance);
                force_x += dx / distance * repulsion;
                force_y += dy / distance * repulsion;
            }

            force_updates.insert(keeper_id.clone(), (force_x, force_y));
        }

        // Apply the force updates
        for (keeper_id, (force_x, force_y)) in force_updates {
            if let Some(keeper_pos) = self.keeper_positions.get_mut(&keeper_id) {
                // Apply forces
                keeper_pos.velocity.x = (keeper_pos.velocity.x + force_x) * forces.damping;
                keeper_pos.velocity.y = (keeper_pos.velocity.y + force_y) * forces.damping;

                // Update position
                keeper_pos.position.x += keeper_pos.velocity.x;
                keeper_pos.position.y += keeper_pos.velocity.y;

                // Keep within bounds
                keeper_pos.position.x = keeper_pos
                    .position
                    .x
                    .clamp(50.0, self.layout.canvas_size.0 - 50.0);
                keeper_pos.position.y = keeper_pos
                    .position
                    .y
                    .clamp(50.0, self.layout.canvas_size.1 - 50.0);
            }
        }

        Ok(())
    }

    fn run_hierarchical_layout(&mut self) -> Result<()> {
        // Collect keeper IDs and calculate positions separately
        let keeper_ids: Vec<_> = self.keeper_positions.keys().cloned().collect();
        let mut keepers_with_reputation: Vec<_> = keeper_ids
            .iter()
            .map(|id| (id.clone(), self.get_keeper_reputation(id)))
            .collect();

        // Sort by reputation (descending)
        keepers_with_reputation
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Arrange in concentric circles based on reputation
        let center_x = self.layout.canvas_size.0 / 2.0;
        let center_y = self.layout.canvas_size.1 / 2.0;

        for (i, (keeper_id, _reputation)) in keepers_with_reputation.iter().enumerate() {
            let ring = i / 8; // 8 keepers per ring
            let position_in_ring = i % 8;

            let radius = 60.0 + (ring as f32 * 80.0);
            let angle = (position_in_ring as f32 / 8.0) * 2.0 * std::f32::consts::PI;

            if let Some(position) = self.keeper_positions.get_mut(keeper_id) {
                position.target_position.x = center_x + angle.cos() * radius;
                position.target_position.y = center_y + angle.sin() * radius;
            }
        }

        Ok(())
    }

    fn run_circular_layout(&mut self) -> Result<()> {
        let keeper_count = self.keeper_positions.len();
        if keeper_count == 0 {
            return Ok(());
        }

        let center_x = self.layout.canvas_size.0 / 2.0;
        let center_y = self.layout.canvas_size.1 / 2.0;
        let radius = (self.layout.canvas_size.0.min(self.layout.canvas_size.1) / 2.0) - 100.0;

        for (i, position) in self.keeper_positions.values_mut().enumerate() {
            let angle = (i as f32 / keeper_count as f32) * 2.0 * std::f32::consts::PI;
            position.target_position.x = center_x + angle.cos() * radius;
            position.target_position.y = center_y + angle.sin() * radius;
        }

        Ok(())
    }

    fn get_keeper_reputation(&self, keeper_id: &str) -> f64 {
        // In a real implementation, this would look up the keeper's reputation
        // For now, return a default value
        0.5
    }
}

// Default implementations

impl Default for NetworkTopology {
    fn default() -> Self {
        Self {
            center_keeper: None,
            clusters: Vec::new(),
            layout_forces: ForceParameters::default(),
        }
    }
}

impl Default for ForceParameters {
    fn default() -> Self {
        Self {
            connection_strength: 0.1,
            repulsion_strength: 1000.0,
            cluster_attraction: 0.05,
            damping: 0.95,
        }
    }
}

impl Default for MapLayout {
    fn default() -> Self {
        Self {
            algorithm: LayoutAlgorithm::ForceDirected,
            canvas_size: (1200.0, 800.0),
            zoom: 1.0,
            camera_center: Position2D { x: 600.0, y: 400.0 },
            auto_layout: true,
        }
    }
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            display_mode: DisplayMode::Status,
            color_scheme: ColorScheme::Status,
            show_connections: true,
            show_sigil_flow: true,
            show_status_badges: true,
            selected_keeper: None,
            highlighted_sigil: None,
        }
    }
}

impl Default for MapAnimations {
    fn default() -> Self {
        Self {
            keeper_heartbeats: true,
            token_flows: true,
            sigil_distribution: true,
            network_pulse: true,
            status_transitions: true,
            speed_multiplier: 1.0,
        }
    }
}
