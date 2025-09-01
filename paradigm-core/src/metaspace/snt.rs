use crate::metaspace::{Glyph, Element, DataCategory, Importance, Keeper, KeeperStatus};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Symbolic Network Token - A living identity anchor that unlocks functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNT {
    /// Unique token identifier
    pub snt_id: String,
    
    /// Symbolic representation and meaning
    pub glyph: Glyph,
    
    /// Current holder of this SNT
    pub holder: String,
    
    /// SNT classification type
    pub token_type: SNTType,
    
    /// What this SNT unlocks or enables
    pub permissions: Vec<AccessPermission>,
    
    /// Modular identity components this represents
    pub identity_facets: Vec<IdentityFacet>,
    
    /// Story fragments and memory anchors
    pub narrative_fragments: Vec<NarrativeFragment>,
    
    /// Executable protocol functions
    pub protocol_triggers: Vec<ProtocolTrigger>,
    
    /// Community roles and ritual participation
    pub community_roles: Vec<CommunityRole>,
    
    /// Dynamic properties that evolve over time
    pub evolution_state: EvolutionState,
    
    /// Creation timestamp and lineage
    pub created_at: u64,
    pub parent_snt: Option<String>,
    pub generation: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SNTType {
    /// Represents keeper roles and reputation
    KeeperIdentity,
    
    /// Unlocks access to specific resources
    AccessKey,
    
    /// Commemorates significant network events
    MemoryAnchor,
    
    /// Encodes contribution history and achievements
    ContributionBadge,
    
    /// Grants ritual participation rights
    CeremonialToken,
    
    /// Represents modular protocol functions
    ProtocolModule,
    
    /// Evolving narrative elements
    StoryFragment,
    
    /// Guild membership and community roles
    CommunityBond,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPermission {
    /// What resource this unlocks
    pub resource_type: ResourceType,
    
    /// Specific resource identifiers
    pub resource_ids: Vec<String>,
    
    /// URL endpoints this enables access to
    pub url_endpoints: Vec<String>,
    
    /// Conditional access rules
    pub conditions: Vec<AccessCondition>,
    
    /// Expiration or renewal requirements
    pub validity: ValidityRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    SigilCollection,
    TomeArchive,
    FusionForge,
    KeeperNetwork,
    ProofChallenge,
    RewardPool,
    GlyphDesigner,
    NetworkAnimator,
    CustomAPI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessCondition {
    MinReputationLevel(u32),
    RequiredSNTs(Vec<String>),
    TimeWindow(u64, u64),
    NetworkParticipation(u32),
    StorageContribution(u64),
    FusionCompletions(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidityRule {
    Permanent,
    TimeExpiry(u64),
    ActivityBased(u32),
    EvolutionTrigger(String),
    CommunityConsensus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityFacet {
    /// What aspect of identity this represents
    pub facet_type: IdentityType,
    
    /// Dynamic URL pointing to current state
    pub living_url: String,
    
    /// Symbolic representation
    pub symbol: String,
    
    /// Current metrics or properties
    pub properties: HashMap<String, f64>,
    
    /// Evolution history
    pub evolution_log: Vec<EvolutionEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentityType {
    StorageContributor,
    FusionMaster,
    GlyphArtist,
    NetworkKeeper,
    ProofChallenger,
    CommunityLeader,
    ProtocolInnovator,
    MythicStoryteller,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeFragment {
    /// Story segment identifier
    pub fragment_id: String,
    
    /// What event or moment this commemorates
    pub event_type: EventType,
    
    /// Dynamic URL hosting the story content
    pub story_url: String,
    
    /// Connected narrative elements
    pub linked_fragments: Vec<String>,
    
    /// Timestamp and context
    pub timestamp: u64,
    pub context: HashMap<String, String>,
    
    /// Community annotations
    pub annotations: Vec<CommunityAnnotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    FirstSigilStored,
    MajorFusion,
    NetworkMilestone,
    CommunityRitual,
    ProtocolUpgrade,
    ConflictResolution,
    CollaborativeCreation,
    SystemEmergence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolTrigger {
    /// Function identifier
    pub trigger_id: String,
    
    /// What protocol action this enables
    pub action_type: ProtocolAction,
    
    /// Executable logic or API endpoint
    pub execution_url: String,
    
    /// Required parameters
    pub parameters: HashMap<String, String>,
    
    /// Trigger conditions
    pub conditions: Vec<TriggerCondition>,
    
    /// Effect on network state
    pub effects: Vec<NetworkEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolAction {
    InitiateHeartbeat,
    TriggerProofChallenge,
    StartFusionRitual,
    UpdateNetworkState,
    CreateAnimation,
    DistributeRewards,
    ValidateStorage,
    SynchronizeKeepers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    TimeInterval(u64),
    NetworkEvent(String),
    UserAction(String),
    ThresholdReached(String, f64),
    CommunityVote(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityRole {
    /// Role identifier and title
    pub role_id: String,
    pub title: String,
    
    /// Symbolic representation
    pub symbol: String,
    pub element_affinity: Element,
    
    /// Responsibilities and permissions
    pub responsibilities: Vec<String>,
    pub governance_weight: f64,
    
    /// Ritual participation rights
    pub rituals: Vec<RitualParticipation>,
    
    /// Community URL endpoints
    pub community_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RitualParticipation {
    /// Ritual type and frequency
    pub ritual_type: RitualType,
    pub participation_level: ParticipationLevel,
    
    /// Required other participants
    pub required_roles: Vec<String>,
    
    /// Seasonal or time-based rules
    pub timing_rules: Vec<TimingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RitualType {
    NetworkHarmonization,
    SeasonalAlignment,
    FusionCeremony,
    KeeperInitiation,
    ConflictMediation,
    ProtocolEvolution,
    CommunityGathering,
    MythicStorytelling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipationLevel {
    Observer,
    Participant,
    Facilitator,
    Conductor,
    HighPriest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionState {
    /// Current evolution level
    pub level: u32,
    
    /// Progress towards next evolution
    pub evolution_progress: f64,
    
    /// Evolution requirements
    pub evolution_requirements: Vec<EvolutionRequirement>,
    
    /// Historical evolution events
    pub evolution_history: Vec<EvolutionEvent>,
    
    /// Potential evolution paths
    pub evolution_paths: Vec<EvolutionPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRequirement {
    pub requirement_type: RequirementType,
    pub target_value: f64,
    pub current_value: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementType {
    StorageContributed,
    FusionsCompleted,
    CommunityVotes,
    TimeHeld,
    RitualParticipations,
    NetworkEvents,
    CollaborativeActions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEvent {
    pub timestamp: u64,
    pub evolution_type: EvolutionType,
    pub description: String,
    pub properties_changed: HashMap<String, (f64, f64)>, // old_value, new_value
    pub community_witnesses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionType {
    LevelUp,
    GlyphTransformation,
    PermissionExpansion,
    RoleEvolution,
    CommunityRecognition,
    ProtocolIntegration,
    NarrativeExpansion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionPath {
    pub path_id: String,
    pub name: String,
    pub description: String,
    pub requirements: Vec<EvolutionRequirement>,
    pub resulting_changes: HashMap<String, String>,
    pub community_vote_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityAnnotation {
    pub author: String,
    pub timestamp: u64,
    pub annotation_type: AnnotationType,
    pub content: String,
    pub votes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    Context,
    Interpretation,
    Connection,
    Tribute,
    Question,
    Evolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingRule {
    pub rule_type: TimingType,
    pub parameters: HashMap<String, u64>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimingType {
    Daily,
    Weekly,
    Monthly,
    Seasonal,
    Lunar,
    NetworkEvent,
    CommunityDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEffect {
    pub effect_type: EffectType,
    pub magnitude: f64,
    pub duration: Option<u64>,
    pub affected_entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectType {
    ReputationChange,
    PermissionGrant,
    ResourceAccess,
    AnimationTrigger,
    CommunityNotification,
    ProtocolModification,
    NarrativeUpdate,
}

/// SNT Management System
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNTSystem {
    /// All active SNTs in the network
    pub active_snts: HashMap<String, SNT>,
    
    /// SNT templates for common patterns
    pub snt_templates: Vec<SNTTemplate>,
    
    /// Evolution rules and patterns
    pub evolution_rules: Vec<EvolutionRule>,
    
    /// Community governance for SNTs
    pub governance: SNTGovernance,
    
    /// Active rituals and ceremonies
    pub active_rituals: HashMap<String, ActiveRitual>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNTTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub token_type: SNTType,
    pub default_glyph: Glyph,
    pub base_permissions: Vec<AccessPermission>,
    pub evolution_potential: Vec<EvolutionPath>,
    pub ritual_eligibility: Vec<RitualType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRule {
    pub rule_id: String,
    pub applicable_types: Vec<SNTType>,
    pub trigger_conditions: Vec<TriggerCondition>,
    pub evolution_effects: Vec<EvolutionEffect>,
    pub community_validation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEffect {
    pub effect_type: String,
    pub magnitude: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNTGovernance {
    pub voting_power_calculation: VotingPowerRule,
    pub proposal_types: Vec<ProposalType>,
    pub consensus_mechanisms: Vec<ConsensusRule>,
    pub community_moderators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VotingPowerRule {
    EqualPower,
    ReputationWeighted,
    SNTTypeWeighted(HashMap<SNTType, f64>),
    TimeHeldWeighted,
    CombinedMetrics(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    SNTCreation,
    EvolutionRule,
    RitualDesign,
    CommunityRole,
    ProtocolChange,
    NarrativeCanon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRule {
    pub rule_name: String,
    pub required_threshold: f64,
    pub minimum_participants: u32,
    pub time_window: u64,
    pub applicable_proposals: Vec<ProposalType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRitual {
    pub ritual_id: String,
    pub ritual_type: RitualType,
    pub participants: HashMap<String, ParticipationLevel>,
    pub start_time: u64,
    pub expected_duration: u64,
    pub ritual_state: RitualState,
    pub outcomes: Vec<RitualOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RitualState {
    Preparing,
    Active,
    Culminating,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RitualOutcome {
    pub outcome_type: OutcomeType,
    pub affected_snts: Vec<String>,
    pub network_effects: Vec<NetworkEffect>,
    pub narrative_fragments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutcomeType {
    SNTEvolution,
    PermissionGrant,
    CommunityBonding,
    ProtocolActivation,
    NarrativeExpansion,
    NetworkHarmonization,
}

impl SNTSystem {
    pub fn new() -> Self {
        let mut system = Self {
            active_snts: HashMap::new(),
            snt_templates: Vec::new(),
            evolution_rules: Vec::new(),
            governance: SNTGovernance {
                voting_power_calculation: VotingPowerRule::ReputationWeighted,
                proposal_types: vec![
                    ProposalType::SNTCreation,
                    ProposalType::EvolutionRule,
                    ProposalType::RitualDesign,
                ],
                consensus_mechanisms: Vec::new(),
                community_moderators: Vec::new(),
            },
            active_rituals: HashMap::new(),
        };
        
        system.initialize_default_templates();
        system.initialize_evolution_rules();
        system
    }

    fn initialize_default_templates(&mut self) {
        self.snt_templates = vec![
            SNTTemplate {
                template_id: "keeper_identity".to_string(),
                name: "🛡️ Keeper Identity".to_string(),
                description: "Represents your role and reputation as a network keeper".to_string(),
                token_type: SNTType::KeeperIdentity,
                default_glyph: Glyph {
                    element: Element::Earth,
                    category: DataCategory::Identity,
                    importance: Importance::Major,
                    properties: HashMap::new(),
                },
                base_permissions: vec![
                    AccessPermission {
                        resource_type: ResourceType::KeeperNetwork,
                        resource_ids: vec!["*".to_string()],
                        url_endpoints: vec!["/api/keeper/status".to_string()],
                        conditions: vec![],
                        validity: ValidityRule::Permanent,
                    }
                ],
                evolution_potential: vec![
                    EvolutionPath {
                        path_id: "apprentice_to_guardian".to_string(),
                        name: "Guardian Ascension".to_string(),
                        description: "Evolve from Apprentice to Guardian keeper".to_string(),
                        requirements: vec![
                            EvolutionRequirement {
                                requirement_type: RequirementType::StorageContributed,
                                target_value: 1000.0,
                                current_value: 0.0,
                                description: "Store 1GB of network data".to_string(),
                            }
                        ],
                        resulting_changes: [
                            ("keeper_level".to_string(), "Guardian".to_string()),
                            ("permissions_expanded".to_string(), "true".to_string()),
                        ].iter().cloned().collect(),
                        community_vote_required: false,
                    }
                ],
                ritual_eligibility: vec![RitualType::KeeperInitiation, RitualType::NetworkHarmonization],
            },
            
            SNTTemplate {
                template_id: "fusion_master".to_string(),
                name: "⚗️ Fusion Master".to_string(),
                description: "Unlocks advanced fusion capabilities and ritual leadership".to_string(),
                token_type: SNTType::CeremonialToken,
                default_glyph: Glyph {
                    element: Element::Aether,
                    category: DataCategory::Protocol,
                    importance: Importance::Critical,
                    properties: HashMap::new(),
                },
                base_permissions: vec![
                    AccessPermission {
                        resource_type: ResourceType::FusionForge,
                        resource_ids: vec!["advanced_rituals".to_string()],
                        url_endpoints: vec!["/api/fusion/master".to_string()],
                        conditions: vec![AccessCondition::FusionCompletions(10)],
                        validity: ValidityRule::ActivityBased(30),
                    }
                ],
                evolution_potential: vec![],
                ritual_eligibility: vec![RitualType::FusionCeremony],
            },

            SNTTemplate {
                template_id: "memory_anchor".to_string(),
                name: "📜 Memory Anchor".to_string(),
                description: "Commemorates significant network events and milestones".to_string(),
                token_type: SNTType::MemoryAnchor,
                default_glyph: Glyph {
                    element: Element::Void,
                    category: DataCategory::Archive,
                    importance: Importance::Legendary,
                    properties: HashMap::new(),
                },
                base_permissions: vec![
                    AccessPermission {
                        resource_type: ResourceType::TomeArchive,
                        resource_ids: vec!["historical_events".to_string()],
                        url_endpoints: vec!["/api/history/events".to_string()],
                        conditions: vec![],
                        validity: ValidityRule::Permanent,
                    }
                ],
                evolution_potential: vec![],
                ritual_eligibility: vec![RitualType::MythicStorytelling],
            },
        ];
    }

    fn initialize_evolution_rules(&mut self) {
        self.evolution_rules = vec![
            EvolutionRule {
                rule_id: "keeper_progression".to_string(),
                applicable_types: vec![SNTType::KeeperIdentity],
                trigger_conditions: vec![
                    TriggerCondition::ThresholdReached("storage_contributed".to_string(), 1000.0),
                ],
                evolution_effects: vec![
                    EvolutionEffect {
                        effect_type: "level_increase".to_string(),
                        magnitude: 1.0,
                        description: "Advance keeper level".to_string(),
                    }
                ],
                community_validation_required: false,
            },
            
            EvolutionRule {
                rule_id: "fusion_mastery".to_string(),
                applicable_types: vec![SNTType::ContributionBadge],
                trigger_conditions: vec![
                    TriggerCondition::ThresholdReached("fusions_completed".to_string(), 25.0),
                ],
                evolution_effects: vec![
                    EvolutionEffect {
                        effect_type: "ceremonial_access".to_string(),
                        magnitude: 1.0,
                        description: "Grant fusion ritual leadership".to_string(),
                    }
                ],
                community_validation_required: true,
            },
        ];
    }

    /// Mint a new SNT based on network events or achievements
    pub fn mint_snt(
        &mut self,
        template_id: &str,
        holder: String,
        trigger_event: String,
        custom_properties: HashMap<String, String>,
    ) -> Result<String> {
        let template = self.snt_templates.iter()
            .find(|t| t.template_id == template_id)
            .ok_or_else(|| anyhow!("SNT template not found"))?;

        let snt_id = format!("snt_{}", uuid::Uuid::new_v4().simple());
        
        let mut snt = SNT {
            snt_id: snt_id.clone(),
            glyph: template.default_glyph.clone(),
            holder,
            token_type: template.token_type.clone(),
            permissions: template.base_permissions.clone(),
            identity_facets: Vec::new(),
            narrative_fragments: vec![
                NarrativeFragment {
                    fragment_id: format!("origin_{}", snt_id),
                    event_type: EventType::SystemEmergence,
                    story_url: format!("/api/snt/{}/origin", snt_id),
                    linked_fragments: Vec::new(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    context: custom_properties.clone(),
                    annotations: Vec::new(),
                }
            ],
            protocol_triggers: Vec::new(),
            community_roles: Vec::new(),
            evolution_state: EvolutionState {
                level: 1,
                evolution_progress: 0.0,
                evolution_requirements: template.evolution_potential.first()
                    .map(|path| path.requirements.clone())
                    .unwrap_or_default(),
                evolution_history: Vec::new(),
                evolution_paths: template.evolution_potential.clone(),
            },
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            parent_snt: None,
            generation: 1,
        };

        // Add trigger event to custom properties
        snt.glyph.properties.insert("trigger_event".to_string(), trigger_event);
        for (key, value) in custom_properties {
            snt.glyph.properties.insert(key, value);
        }

        self.active_snts.insert(snt_id.clone(), snt);
        
        Ok(snt_id)
    }

    /// Check if holder has access to a resource via their SNTs
    pub fn check_access(&self, holder: &str, resource: &ResourceType, resource_id: &str) -> bool {
        self.active_snts.values()
            .filter(|snt| snt.holder == holder)
            .any(|snt| {
                snt.permissions.iter().any(|perm| {
                    perm.resource_type == *resource && 
                    (perm.resource_ids.contains(&resource_id.to_string()) || 
                     perm.resource_ids.contains(&"*".to_string()))
                })
            })
    }

    /// Evolve an SNT based on network activity
    pub fn evolve_snt(&mut self, snt_id: &str, activity_data: HashMap<String, f64>) -> Result<Vec<EvolutionEvent>> {
        let snt = self.active_snts.get_mut(snt_id)
            .ok_or_else(|| anyhow!("SNT not found"))?;

        let mut evolution_events = Vec::new();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check evolution requirements
        for requirement in &mut snt.evolution_state.evolution_requirements {
            if let Some(&new_value) = activity_data.get(&format!("{:?}", requirement.requirement_type).to_lowercase()) {
                let old_value = requirement.current_value;
                requirement.current_value = new_value.max(old_value);
                
                if requirement.current_value >= requirement.target_value {
                    // Trigger evolution
                    evolution_events.push(EvolutionEvent {
                        timestamp: current_time,
                        evolution_type: EvolutionType::LevelUp,
                        description: format!("Completed requirement: {}", requirement.description),
                        properties_changed: [(
                            requirement.requirement_type.clone().into(),
                            (old_value, new_value)
                        )].iter().cloned().collect(),
                        community_witnesses: Vec::new(),
                    });
                }
            }
        }

        // Add evolution events to history
        snt.evolution_state.evolution_history.extend(evolution_events.clone());

        Ok(evolution_events)
    }

    /// Get all SNTs for a specific holder
    pub fn get_holder_snts(&self, holder: &str) -> Vec<&SNT> {
        self.active_snts.values()
            .filter(|snt| snt.holder == holder)
            .collect()
    }

    /// Initiate a community ritual
    pub fn initiate_ritual(
        &mut self,
        ritual_type: RitualType,
        initiator: String,
        participants: Vec<String>,
    ) -> Result<String> {
        let ritual_id = format!("ritual_{}", uuid::Uuid::new_v4().simple());
        
        let ritual = ActiveRitual {
            ritual_id: ritual_id.clone(),
            ritual_type,
            participants: participants.into_iter()
                .map(|p| (p, ParticipationLevel::Participant))
                .chain(std::iter::once((initiator, ParticipationLevel::Facilitator)))
                .collect(),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            expected_duration: 3600, // 1 hour default
            ritual_state: RitualState::Preparing,
            outcomes: Vec::new(),
        };

        self.active_rituals.insert(ritual_id.clone(), ritual);
        Ok(ritual_id)
    }

    pub fn get_snt_stats(&self) -> SNTStats {
        let mut type_distribution = HashMap::new();
        let mut evolution_levels = HashMap::new();
        
        for snt in self.active_snts.values() {
            *type_distribution.entry(format!("{:?}", snt.token_type)).or_insert(0) += 1;
            *evolution_levels.entry(snt.evolution_state.level).or_insert(0) += 1;
        }

        SNTStats {
            total_snts: self.active_snts.len(),
            type_distribution,
            evolution_levels,
            active_rituals: self.active_rituals.len(),
            total_holders: self.active_snts.values()
                .map(|snt| &snt.holder)
                .collect::<std::collections::HashSet<_>>()
                .len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SNTStats {
    pub total_snts: usize,
    pub type_distribution: HashMap<String, usize>,
    pub evolution_levels: HashMap<u32, usize>,
    pub active_rituals: usize,
    pub total_holders: usize,
}

impl From<RequirementType> for String {
    fn from(req_type: RequirementType) -> String {
        match req_type {
            RequirementType::StorageContributed => "storage_contributed".to_string(),
            RequirementType::FusionsCompleted => "fusions_completed".to_string(),
            RequirementType::CommunityVotes => "community_votes".to_string(),
            RequirementType::TimeHeld => "time_held".to_string(),
            RequirementType::RitualParticipations => "ritual_participations".to_string(),
            RequirementType::NetworkEvents => "network_events".to_string(),
            RequirementType::CollaborativeActions => "collaborative_actions".to_string(),
        }
    }
}

// Add PartialEq for comparison operations
impl PartialEq for ResourceType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}