use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAnimator {
    pub active_animations: HashMap<String, Animation>,
    pub animation_config: AnimationConfig,
    pub particle_systems: Vec<ParticleSystem>,
    pub visual_effects: Vec<VisualEffect>,
    pub heartbeat_tracker: HeartbeatTracker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub animation_id: String,
    pub animation_type: AnimationType,
    pub target_id: String,
    pub start_time: u64,
    pub duration: f32,
    pub progress: f32,
    pub is_looping: bool,
    pub easing_function: EasingFunction,
    pub properties: AnimationProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationType {
    KeeperHeartbeat,
    SigilFlow,
    TokenTransfer,
    ProofChallenge,
    NetworkPulse,
    FusionSpiral,
    GlyphResonance,
    DataStream,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Back,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationProperties {
    pub position: Option<Position3D>,
    pub scale: Option<f32>,
    pub rotation: Option<f32>,
    pub opacity: Option<f32>,
    pub color: Option<String>,
    pub glow_intensity: Option<f32>,
    pub custom_properties: HashMap<String, f32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    pub global_speed_multiplier: f32,
    pub enable_particle_effects: bool,
    pub enable_glow_effects: bool,
    pub enable_sound_effects: bool,
    pub max_concurrent_animations: usize,
    pub performance_mode: PerformanceMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMode {
    Ultra,
    High,
    Medium,
    Low,
    Minimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystem {
    pub system_id: String,
    pub system_type: ParticleSystemType,
    pub emitter_position: Position3D,
    pub particle_count: u32,
    pub emission_rate: f32,
    pub particle_lifetime: f32,
    pub velocity: Velocity3D,
    pub color_gradient: ColorGradient,
    pub size_curve: SizeCurve,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleSystemType {
    DataSparks,
    EnergyMotes,
    TokenGlow,
    NetworkFlow,
    KeeperAura,
    SigilTrails,
    FusionBurst,
    GlyphResonance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Velocity3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub variance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGradient {
    pub start_color: String,
    pub end_color: String,
    pub keyframes: Vec<ColorKeyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorKeyframe {
    pub time: f32,
    pub color: String,
    pub alpha: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeCurve {
    pub start_size: f32,
    pub end_size: f32,
    pub curve_points: Vec<CurvePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurvePoint {
    pub time: f32,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualEffect {
    pub effect_id: String,
    pub effect_type: VisualEffectType,
    pub target_id: String,
    pub intensity: f32,
    pub duration: f32,
    pub is_active: bool,
    pub parameters: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualEffectType {
    GlowPulse,
    ColorShift,
    RippleWave,
    LightningBolt,
    EnergyField,
    HolographicShimmer,
    QuantumFlicker,
    AuraExpansion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatTracker {
    pub keeper_heartbeats: HashMap<String, HeartbeatData>,
    pub network_pulse_interval: f32,
    pub last_network_pulse: u64,
    pub synchronization_phase: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatData {
    pub keeper_id: String,
    pub last_beat: u64,
    pub beat_interval: f32,
    pub health_status: HealthStatus,
    pub pulse_strength: f32,
    pub rhythm_pattern: RhythmPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Optimal,
    Healthy,
    Stressed,
    Critical,
    Dormant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RhythmPattern {
    Steady,
    Syncopated,
    Irregular,
    Accelerating,
    Decelerating,
}

impl NetworkAnimator {
    pub fn new() -> Self {
        Self {
            active_animations: HashMap::new(),
            animation_config: AnimationConfig::default(),
            particle_systems: Vec::new(),
            visual_effects: Vec::new(),
            heartbeat_tracker: HeartbeatTracker::new(),
        }
    }

    pub fn start_keeper_heartbeat(&mut self, keeper_id: String, position: Position3D) -> String {
        let animation_id = format!("heartbeat_{}", keeper_id);
        
        let animation = Animation {
            animation_id: animation_id.clone(),
            animation_type: AnimationType::KeeperHeartbeat,
            target_id: keeper_id.clone(),
            start_time: self.current_timestamp(),
            duration: 2.0, // 2 second heartbeat cycle
            progress: 0.0,
            is_looping: true,
            easing_function: EasingFunction::EaseInOut,
            properties: AnimationProperties {
                position: Some(position),
                scale: Some(1.0),
                rotation: None,
                opacity: Some(1.0),
                color: Some("#4CAF50".to_string()),
                glow_intensity: Some(0.5),
                custom_properties: [
                    ("pulse_strength".to_string(), 1.0),
                    ("beat_frequency".to_string(), 0.5),
                ].iter().cloned().collect(),
            },
        };

        self.active_animations.insert(animation_id.clone(), animation);

        // Start keeper heartbeat tracking
        self.heartbeat_tracker.keeper_heartbeats.insert(keeper_id.clone(), HeartbeatData {
            keeper_id,
            last_beat: self.current_timestamp(),
            beat_interval: 2.0,
            health_status: HealthStatus::Healthy,
            pulse_strength: 1.0,
            rhythm_pattern: RhythmPattern::Steady,
        });

        animation_id
    }

    pub fn animate_sigil_flow(&mut self, from_position: Position3D, to_position: Position3D, sigil_id: String) -> String {
        let animation_id = format!("flow_{}", sigil_id);
        
        let animation = Animation {
            animation_id: animation_id.clone(),
            animation_type: AnimationType::SigilFlow,
            target_id: sigil_id,
            start_time: self.current_timestamp(),
            duration: 3.0,
            progress: 0.0,
            is_looping: false,
            easing_function: EasingFunction::EaseInOut,
            properties: AnimationProperties {
                position: Some(from_position),
                scale: Some(0.8),
                rotation: Some(0.0),
                opacity: Some(0.9),
                color: Some("#2196F3".to_string()),
                glow_intensity: Some(0.7),
                custom_properties: [
                    ("target_x".to_string(), to_position.x),
                    ("target_y".to_string(), to_position.y),
                    ("target_z".to_string(), to_position.z),
                    ("trail_length".to_string(), 10.0),
                ].iter().cloned().collect(),
            },
        };

        self.active_animations.insert(animation_id.clone(), animation);

        // Create particle trail
        self.create_sigil_trail_particles(from_position, to_position);

        animation_id
    }

    pub fn animate_token_transfer(&mut self, from_keeper: String, to_keeper: String, amount: u64) -> String {
        let animation_id = format!("token_{}_{}", from_keeper, self.current_timestamp());
        
        let animation = Animation {
            animation_id: animation_id.clone(),
            animation_type: AnimationType::TokenTransfer,
            target_id: format!("{}_{}", from_keeper, to_keeper),
            start_time: self.current_timestamp(),
            duration: 2.5,
            progress: 0.0,
            is_looping: false,
            easing_function: EasingFunction::EaseOut,
            properties: AnimationProperties {
                position: None,
                scale: Some(1.0),
                rotation: Some(0.0),
                opacity: Some(1.0),
                color: Some("#FFD700".to_string()),
                glow_intensity: Some(0.8),
                custom_properties: [
                    ("token_amount".to_string(), amount as f32),
                    ("beam_intensity".to_string(), (amount as f32).log10().max(1.0)),
                ].iter().cloned().collect(),
            },
        };

        self.active_animations.insert(animation_id.clone(), animation);
        animation_id
    }

    pub fn animate_proof_challenge(&mut self, challenger_id: String, keeper_id: String) -> String {
        let animation_id = format!("proof_{}_{}", challenger_id, keeper_id);
        
        let animation = Animation {
            animation_id: animation_id.clone(),
            animation_type: AnimationType::ProofChallenge,
            target_id: keeper_id.clone(),
            start_time: self.current_timestamp(),
            duration: 5.0,
            progress: 0.0,
            is_looping: false,
            easing_function: EasingFunction::Linear,
            properties: AnimationProperties {
                position: None,
                scale: Some(1.2),
                rotation: Some(0.0),
                opacity: Some(1.0),
                color: Some("#FF5722".to_string()),
                glow_intensity: Some(1.0),
                custom_properties: [
                    ("challenge_intensity".to_string(), 1.5),
                    ("scanning_speed".to_string(), 2.0),
                ].iter().cloned().collect(),
            },
        };

        self.active_animations.insert(animation_id.clone(), animation);

        // Create scanning visual effect
        self.create_proof_scanning_effect(keeper_id);

        animation_id
    }

    pub fn animate_network_pulse(&mut self) -> String {
        let animation_id = format!("network_pulse_{}", self.current_timestamp());
        
        let animation = Animation {
            animation_id: animation_id.clone(),
            animation_type: AnimationType::NetworkPulse,
            target_id: "network".to_string(),
            start_time: self.current_timestamp(),
            duration: 4.0,
            progress: 0.0,
            is_looping: true,
            easing_function: EasingFunction::EaseInOut,
            properties: AnimationProperties {
                position: None,
                scale: Some(1.0),
                rotation: Some(0.0),
                opacity: Some(0.3),
                color: Some("#9C27B0".to_string()),
                glow_intensity: Some(0.4),
                custom_properties: [
                    ("pulse_radius".to_string(), 0.0),
                    ("max_radius".to_string(), 1000.0),
                    ("wave_speed".to_string(), 250.0),
                ].iter().cloned().collect(),
            },
        };

        self.active_animations.insert(animation_id.clone(), animation);

        // Update heartbeat tracker
        self.heartbeat_tracker.last_network_pulse = self.current_timestamp();

        animation_id
    }

    pub fn animate_fusion_ritual(&mut self, sigil_positions: Vec<Position3D>, fusion_center: Position3D) -> String {
        let animation_id = format!("fusion_{}", self.current_timestamp());
        
        let animation = Animation {
            animation_id: animation_id.clone(),
            animation_type: AnimationType::FusionSpiral,
            target_id: "fusion_ritual".to_string(),
            start_time: self.current_timestamp(),
            duration: 8.0,
            progress: 0.0,
            is_looping: false,
            easing_function: EasingFunction::EaseIn,
            properties: AnimationProperties {
                position: Some(fusion_center),
                scale: Some(0.1),
                rotation: Some(0.0),
                opacity: Some(1.0),
                color: Some("#E91E63".to_string()),
                glow_intensity: Some(1.2),
                custom_properties: [
                    ("spiral_speed".to_string(), 2.0),
                    ("energy_buildup".to_string(), 0.0),
                    ("sigil_count".to_string(), sigil_positions.len() as f32),
                ].iter().cloned().collect(),
            },
        };

        self.active_animations.insert(animation_id.clone(), animation);

        // Create multiple particle systems for the fusion
        self.create_fusion_particle_systems(sigil_positions, fusion_center);

        animation_id
    }

    pub fn animate_glyph_resonance(&mut self, glyph_id: String, resonance_strength: f32) -> String {
        let animation_id = format!("resonance_{}", glyph_id);
        
        let animation = Animation {
            animation_id: animation_id.clone(),
            animation_type: AnimationType::GlyphResonance,
            target_id: glyph_id,
            start_time: self.current_timestamp(),
            duration: resonance_strength * 3.0, // Stronger resonance lasts longer
            progress: 0.0,
            is_looping: true,
            easing_function: EasingFunction::Elastic,
            properties: AnimationProperties {
                position: None,
                scale: Some(1.0 + resonance_strength * 0.3),
                rotation: Some(0.0),
                opacity: Some(0.8 + resonance_strength * 0.2),
                color: Some("#FF9800".to_string()),
                glow_intensity: Some(resonance_strength),
                custom_properties: [
                    ("resonance_frequency".to_string(), resonance_strength * 2.0),
                    ("harmonic_waves".to_string(), (resonance_strength * 5.0).ceil()),
                ].iter().cloned().collect(),
            },
        };

        self.active_animations.insert(animation_id.clone(), animation);
        animation_id
    }

    fn create_sigil_trail_particles(&mut self, from: Position3D, to: Position3D) {
        let system = ParticleSystem {
            system_id: format!("sigil_trail_{}", self.current_timestamp()),
            system_type: ParticleSystemType::SigilTrails,
            emitter_position: from,
            particle_count: 50,
            emission_rate: 20.0,
            particle_lifetime: 2.0,
            velocity: Velocity3D {
                x: (to.x - from.x) / 3.0,
                y: (to.y - from.y) / 3.0,
                z: (to.z - from.z) / 3.0,
                variance: 0.1,
            },
            color_gradient: ColorGradient {
                start_color: "#2196F3".to_string(),
                end_color: "#03DAC6".to_string(),
                keyframes: vec![
                    ColorKeyframe { time: 0.0, color: "#2196F3".to_string(), alpha: 1.0 },
                    ColorKeyframe { time: 0.5, color: "#00BCD4".to_string(), alpha: 0.8 },
                    ColorKeyframe { time: 1.0, color: "#03DAC6".to_string(), alpha: 0.0 },
                ],
            },
            size_curve: SizeCurve {
                start_size: 2.0,
                end_size: 0.1,
                curve_points: vec![
                    CurvePoint { time: 0.0, value: 2.0 },
                    CurvePoint { time: 0.3, value: 3.0 },
                    CurvePoint { time: 1.0, value: 0.1 },
                ],
            },
            is_active: true,
        };

        self.particle_systems.push(system);
    }

    fn create_proof_scanning_effect(&mut self, keeper_id: String) {
        let effect = VisualEffect {
            effect_id: format!("scan_{}", keeper_id),
            effect_type: VisualEffectType::RippleWave,
            target_id: keeper_id,
            intensity: 0.8,
            duration: 3.0,
            is_active: true,
            parameters: [
                ("ripple_speed".to_string(), 200.0),
                ("ripple_count".to_string(), 3.0),
                ("scan_color_r".to_string(), 1.0),
                ("scan_color_g".to_string(), 0.2),
                ("scan_color_b".to_string(), 0.0),
            ].iter().cloned().collect(),
        };

        self.visual_effects.push(effect);
    }

    fn create_fusion_particle_systems(&mut self, sigil_positions: Vec<Position3D>, center: Position3D) {
        // Energy convergence particles from each sigil to center
        for (i, pos) in sigil_positions.iter().enumerate() {
            let system = ParticleSystem {
                system_id: format!("fusion_energy_{}", i),
                system_type: ParticleSystemType::EnergyMotes,
                emitter_position: pos.clone(),
                particle_count: 100,
                emission_rate: 30.0,
                particle_lifetime: 3.0,
                velocity: Velocity3D {
                    x: (center.x - pos.x) / 3.0,
                    y: (center.y - pos.y) / 3.0,
                    z: (center.z - pos.z) / 3.0,
                    variance: 0.2,
                },
                color_gradient: ColorGradient {
                    start_color: "#E91E63".to_string(),
                    end_color: "#FF9800".to_string(),
                    keyframes: vec![
                        ColorKeyframe { time: 0.0, color: "#E91E63".to_string(), alpha: 0.8 },
                        ColorKeyframe { time: 0.7, color: "#FF5722".to_string(), alpha: 1.0 },
                        ColorKeyframe { time: 1.0, color: "#FF9800".to_string(), alpha: 0.3 },
                    ],
                },
                size_curve: SizeCurve {
                    start_size: 1.5,
                    end_size: 4.0,
                    curve_points: vec![
                        CurvePoint { time: 0.0, value: 1.5 },
                        CurvePoint { time: 0.8, value: 3.0 },
                        CurvePoint { time: 1.0, value: 4.0 },
                    ],
                },
                is_active: true,
            };

            self.particle_systems.push(system);
        }

        // Central fusion burst
        let burst_system = ParticleSystem {
            system_id: format!("fusion_burst_{}", self.current_timestamp()),
            system_type: ParticleSystemType::FusionBurst,
            emitter_position: center,
            particle_count: 200,
            emission_rate: 50.0,
            particle_lifetime: 2.0,
            velocity: Velocity3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                variance: 50.0, // High variance for explosion effect
            },
            color_gradient: ColorGradient {
                start_color: "#FFFFFF".to_string(),
                end_color: "#9C27B0".to_string(),
                keyframes: vec![
                    ColorKeyframe { time: 0.0, color: "#FFFFFF".to_string(), alpha: 1.0 },
                    ColorKeyframe { time: 0.2, color: "#FF9800".to_string(), alpha: 0.9 },
                    ColorKeyframe { time: 0.6, color: "#E91E63".to_string(), alpha: 0.7 },
                    ColorKeyframe { time: 1.0, color: "#9C27B0".to_string(), alpha: 0.0 },
                ],
            },
            size_curve: SizeCurve {
                start_size: 0.5,
                end_size: 8.0,
                curve_points: vec![
                    CurvePoint { time: 0.0, value: 0.5 },
                    CurvePoint { time: 0.1, value: 6.0 },
                    CurvePoint { time: 0.4, value: 8.0 },
                    CurvePoint { time: 1.0, value: 2.0 },
                ],
            },
            is_active: true,
        };

        self.particle_systems.push(burst_system);
    }

    pub fn update_animations(&mut self, delta_time: f32) -> Vec<AnimationEvent> {
        let mut events = Vec::new();
        let current_time = self.current_timestamp();
        let mut completed_animations = Vec::new();

        for (animation_id, animation) in &mut self.active_animations {
            let elapsed_time = (current_time - animation.start_time) as f32 / 1000.0;
            animation.progress = (elapsed_time / animation.duration).min(1.0);

            // Apply easing
            let eased_progress = NetworkAnimator::apply_easing_static(animation.progress, &animation.easing_function);

            // Update animation properties based on progress
            NetworkAnimator::update_animation_properties_static(animation, eased_progress);

            // Check if animation is complete
            if animation.progress >= 1.0 {
                if animation.is_looping {
                    animation.progress = 0.0;
                    animation.start_time = current_time;
                } else {
                    completed_animations.push(animation_id.clone());
                    events.push(AnimationEvent {
                        event_type: AnimationEventType::AnimationComplete,
                        animation_id: animation_id.clone(),
                        target_id: animation.target_id.clone(),
                        data: HashMap::new(),
                    });
                }
            }
        }

        // Remove completed animations
        for animation_id in completed_animations {
            self.active_animations.remove(&animation_id);
        }

        // Update particle systems
        self.update_particle_systems(delta_time);

        // Update visual effects
        self.update_visual_effects(delta_time);

        // Update heartbeats
        self.update_heartbeats(current_time, &mut events);

        events
    }

    fn apply_easing_static(progress: f32, easing: &EasingFunction) -> f32 {
        match easing {
            EasingFunction::Linear => progress,
            EasingFunction::EaseIn => progress * progress,
            EasingFunction::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
            EasingFunction::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - 2.0 * (1.0 - progress) * (1.0 - progress)
                }
            },
            EasingFunction::Bounce => {
                if progress < 0.36364 {
                    7.5625 * progress * progress
                } else if progress < 0.72727 {
                    let p = progress - 0.54545;
                    7.5625 * p * p + 0.75
                } else if progress < 0.90909 {
                    let p = progress - 0.81818;
                    7.5625 * p * p + 0.9375
                } else {
                    let p = progress - 0.95454;
                    7.5625 * p * p + 0.984375
                }
            },
            EasingFunction::Elastic => {
                if progress <= 0.0 { return 0.0; }
                if progress >= 1.0 { return 1.0; }
                let p = 0.3;
                let s = p / 4.0;
                (2.0_f32).powf(-10.0 * progress) * ((progress - s) * (2.0 * std::f32::consts::PI) / p).sin() + 1.0
            },
            EasingFunction::Back => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * progress * progress * progress - c1 * progress * progress
            },
        }
    }

    fn update_animation_properties_static(animation: &mut Animation, eased_progress: f32) {
        match animation.animation_type {
            AnimationType::KeeperHeartbeat => {
                let pulse_strength = animation.properties.custom_properties.get("pulse_strength").unwrap_or(&1.0);
                let beat_cycle = (eased_progress * 2.0 * std::f32::consts::PI).sin().abs();
                animation.properties.scale = Some(1.0 + beat_cycle * 0.1 * pulse_strength);
                animation.properties.glow_intensity = Some(0.3 + beat_cycle * 0.4 * pulse_strength);
            },
            AnimationType::SigilFlow => {
                if let Some(pos) = &mut animation.properties.position {
                    let target_x = animation.properties.custom_properties.get("target_x").unwrap_or(&pos.x);
                    let target_y = animation.properties.custom_properties.get("target_y").unwrap_or(&pos.y);
                    let target_z = animation.properties.custom_properties.get("target_z").unwrap_or(&pos.z);
                    
                    let start_x = pos.x;
                    let start_y = pos.y;
                    let start_z = pos.z;
                    
                    pos.x = start_x + (target_x - start_x) * eased_progress;
                    pos.y = start_y + (target_y - start_y) * eased_progress;
                    pos.z = start_z + (target_z - start_z) * eased_progress;
                }
                animation.properties.rotation = Some(eased_progress * 720.0); // Two full rotations
            },
            AnimationType::NetworkPulse => {
                let max_radius = animation.properties.custom_properties.get("max_radius").unwrap_or(&1000.0);
                animation.properties.custom_properties.insert("pulse_radius".to_string(), eased_progress * max_radius);
                animation.properties.opacity = Some(0.5 * (1.0 - eased_progress));
            },
            AnimationType::FusionSpiral => {
                animation.properties.scale = Some(0.1 + eased_progress * 2.0);
                animation.properties.rotation = Some(eased_progress * 1440.0); // Four full rotations
                animation.properties.glow_intensity = Some(eased_progress * 2.0);
                animation.properties.custom_properties.insert("energy_buildup".to_string(), eased_progress);
            },
            _ => {
                // Default behavior for other animation types
                if animation.properties.scale.is_some() {
                    let base_scale = 1.0;
                    animation.properties.scale = Some(base_scale + (eased_progress - 0.5).abs() * 0.1);
                }
            },
        }
    }

    fn update_particle_systems(&mut self, delta_time: f32) {
        self.particle_systems.retain_mut(|system| {
            if !system.is_active {
                return false;
            }

            // Simple particle system lifecycle management
            system.particle_lifetime -= delta_time;
            system.is_active = system.particle_lifetime > 0.0;
            
            system.is_active
        });
    }

    fn update_visual_effects(&mut self, delta_time: f32) {
        self.visual_effects.retain_mut(|effect| {
            if !effect.is_active {
                return false;
            }

            effect.duration -= delta_time;
            effect.is_active = effect.duration > 0.0;
            
            effect.is_active
        });
    }

    fn update_heartbeats(&mut self, current_time: u64, events: &mut Vec<AnimationEvent>) {
        for (keeper_id, heartbeat) in &mut self.heartbeat_tracker.keeper_heartbeats {
            let time_since_beat = (current_time - heartbeat.last_beat) as f32 / 1000.0;
            
            if time_since_beat >= heartbeat.beat_interval {
                heartbeat.last_beat = current_time;
                
                events.push(AnimationEvent {
                    event_type: AnimationEventType::KeeperHeartbeat,
                    animation_id: format!("heartbeat_{}", keeper_id),
                    target_id: keeper_id.clone(),
                    data: [
                        ("pulse_strength".to_string(), heartbeat.pulse_strength),
                        ("health_score".to_string(), NetworkAnimator::health_status_to_score_static(&heartbeat.health_status)),
                    ].iter().cloned().collect(),
                });
            }
        }

        // Network synchronization pulse
        let time_since_network_pulse = (current_time - self.heartbeat_tracker.last_network_pulse) as f32 / 1000.0;
        if time_since_network_pulse >= self.heartbeat_tracker.network_pulse_interval {
            self.animate_network_pulse();
            events.push(AnimationEvent {
                event_type: AnimationEventType::NetworkPulse,
                animation_id: "network_pulse".to_string(),
                target_id: "network".to_string(),
                data: HashMap::new(),
            });
        }
    }

    fn health_status_to_score_static(status: &HealthStatus) -> f32 {
        match status {
            HealthStatus::Optimal => 1.0,
            HealthStatus::Healthy => 0.8,
            HealthStatus::Stressed => 0.6,
            HealthStatus::Critical => 0.3,
            HealthStatus::Dormant => 0.0,
        }
    }

    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    pub fn get_animation_stats(&self) -> AnimationStats {
        AnimationStats {
            active_animations: self.active_animations.len(),
            active_particle_systems: self.particle_systems.iter().filter(|s| s.is_active).count(),
            active_visual_effects: self.visual_effects.iter().filter(|e| e.is_active).count(),
            active_heartbeats: self.heartbeat_tracker.keeper_heartbeats.len(),
            performance_mode: self.animation_config.performance_mode.clone(),
        }
    }
}

impl HeartbeatTracker {
    pub fn new() -> Self {
        Self {
            keeper_heartbeats: HashMap::new(),
            network_pulse_interval: 10.0, // 10 second network pulses
            last_network_pulse: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            synchronization_phase: 0.0,
        }
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            global_speed_multiplier: 1.0,
            enable_particle_effects: true,
            enable_glow_effects: true,
            enable_sound_effects: false,
            max_concurrent_animations: 100,
            performance_mode: PerformanceMode::High,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationEvent {
    pub event_type: AnimationEventType,
    pub animation_id: String,
    pub target_id: String,
    pub data: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationEventType {
    AnimationStart,
    AnimationComplete,
    KeeperHeartbeat,
    NetworkPulse,
    FusionComplete,
    ProofChallenge,
    TokenTransfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationStats {
    pub active_animations: usize,
    pub active_particle_systems: usize,
    pub active_visual_effects: usize,
    pub active_heartbeats: usize,
    pub performance_mode: PerformanceMode,
}