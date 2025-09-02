use crate::metaspace::{DataCategory, Element, Glyph, Importance};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphDesigner {
    pub custom_glyphs: HashMap<String, CustomGlyph>,
    pub glyph_templates: Vec<GlyphTemplate>,
    pub element_combinations: HashMap<String, ElementCombination>,
    pub saved_designs: Vec<SavedDesign>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGlyph {
    pub glyph_id: String,
    pub name: String,
    pub description: String,
    pub base_glyph: Glyph,
    pub visual_properties: VisualProperties,
    pub custom_properties: HashMap<String, String>,
    pub creator: String,
    pub created_at: u64,
    pub usage_count: u64,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub element_pattern: ElementPattern,
    pub suggested_properties: HashMap<String, String>,
    pub icon: String,
    pub rarity: TemplateRarity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemplateCategory {
    Computing,
    Storage,
    Communication,
    Analytics,
    Security,
    Gaming,
    Creative,
    Scientific,
    Business,
    Personal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementPattern {
    Pure(Element),
    Dual(Element, Element),
    Triad(Element, Element, Element),
    Balanced,
    Chaotic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualProperties {
    pub color_scheme: ColorScheme,
    pub animation_style: AnimationStyle,
    pub glow_intensity: f32,
    pub particle_effects: Vec<ParticleEffect>,
    pub size_multiplier: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub gradient_direction: GradientDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GradientDirection {
    Horizontal,
    Vertical,
    Radial,
    Diagonal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationStyle {
    Static,
    Pulse,
    Rotate,
    Float,
    Shimmer,
    Ripple,
    Spiral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEffect {
    pub effect_type: ParticleType,
    pub intensity: f32,
    pub color: String,
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleType {
    Sparks,
    Mist,
    Embers,
    Lightning,
    Stars,
    Bubbles,
    Smoke,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementCombination {
    pub combination_id: String,
    pub elements: Vec<Element>,
    pub result_element: Element,
    pub fusion_requirements: FusionRequirements,
    pub unlock_conditions: Vec<UnlockCondition>,
    pub rarity_bonus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionRequirements {
    pub min_energy: u64,
    pub required_materials: HashMap<String, u32>,
    pub success_rate: f64,
    pub fusion_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockCondition {
    StorageThreshold(u64),
    ReputationLevel(u32),
    FusionCount(u32),
    TimeActive(u64),
    CustomAchievement(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedDesign {
    pub design_id: String,
    pub name: String,
    pub glyph: Glyph,
    pub visual_properties: VisualProperties,
    pub notes: String,
    pub creator: String,
    pub created_at: u64,
    pub is_favorite: bool,
}

impl GlyphDesigner {
    pub fn new() -> Self {
        let mut designer = Self {
            custom_glyphs: HashMap::new(),
            glyph_templates: Vec::new(),
            element_combinations: HashMap::new(),
            saved_designs: Vec::new(),
        };

        designer.initialize_default_templates();
        designer.initialize_element_combinations();
        designer
    }

    fn initialize_default_templates(&mut self) {
        self.glyph_templates = vec![
            GlyphTemplate {
                template_id: "datacube_basic".to_string(),
                name: "🧊 Data Cube".to_string(),
                description: "Basic storage glyph for structured data".to_string(),
                category: TemplateCategory::Storage,
                element_pattern: ElementPattern::Pure(Element::Earth),
                suggested_properties: [
                    ("durability".to_string(), "high".to_string()),
                    ("access_speed".to_string(), "medium".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                icon: "🧊".to_string(),
                rarity: TemplateRarity::Common,
            },
            GlyphTemplate {
                template_id: "flame_processor".to_string(),
                name: "🔥 Flame Processor".to_string(),
                description: "High-performance computation glyph".to_string(),
                category: TemplateCategory::Computing,
                element_pattern: ElementPattern::Dual(Element::Fire, Element::Lightning),
                suggested_properties: [
                    ("processing_power".to_string(), "extreme".to_string()),
                    ("energy_consumption".to_string(), "high".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                icon: "🔥".to_string(),
                rarity: TemplateRarity::Rare,
            },
            GlyphTemplate {
                template_id: "void_cipher".to_string(),
                name: "🌙 Void Cipher".to_string(),
                description: "Advanced encryption and privacy glyph".to_string(),
                category: TemplateCategory::Security,
                element_pattern: ElementPattern::Pure(Element::Void),
                suggested_properties: [
                    ("encryption_strength".to_string(), "military".to_string()),
                    ("visibility".to_string(), "hidden".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                icon: "🌙".to_string(),
                rarity: TemplateRarity::Epic,
            },
            GlyphTemplate {
                template_id: "aether_nexus".to_string(),
                name: "🔮 Aether Nexus".to_string(),
                description: "Meta-data and system integration glyph".to_string(),
                category: TemplateCategory::Analytics,
                element_pattern: ElementPattern::Triad(
                    Element::Aether,
                    Element::Air,
                    Element::Lightning,
                ),
                suggested_properties: [
                    ("connectivity".to_string(), "universal".to_string()),
                    ("insight_level".to_string(), "transcendent".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                icon: "🔮".to_string(),
                rarity: TemplateRarity::Legendary,
            },
            GlyphTemplate {
                template_id: "water_stream".to_string(),
                name: "💧 Water Stream".to_string(),
                description: "Flowing data and real-time processing".to_string(),
                category: TemplateCategory::Communication,
                element_pattern: ElementPattern::Dual(Element::Water, Element::Air),
                suggested_properties: [
                    ("flow_rate".to_string(), "adaptive".to_string()),
                    ("latency".to_string(), "minimal".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                icon: "💧".to_string(),
                rarity: TemplateRarity::Uncommon,
            },
            GlyphTemplate {
                template_id: "rainbow_prism".to_string(),
                name: "🌈 Rainbow Prism".to_string(),
                description: "Multi-dimensional creative glyph".to_string(),
                category: TemplateCategory::Creative,
                element_pattern: ElementPattern::Balanced,
                suggested_properties: [
                    ("creativity_boost".to_string(), "maximum".to_string()),
                    ("inspiration".to_string(), "endless".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
                icon: "🌈".to_string(),
                rarity: TemplateRarity::Legendary,
            },
        ];
    }

    fn initialize_element_combinations(&mut self) {
        let combinations = vec![
            ElementCombination {
                combination_id: "fire_water_steam".to_string(),
                elements: vec![Element::Fire, Element::Water],
                result_element: Element::Air,
                fusion_requirements: FusionRequirements {
                    min_energy: 500,
                    required_materials: HashMap::new(),
                    success_rate: 0.8,
                    fusion_time: 300,
                },
                unlock_conditions: vec![UnlockCondition::FusionCount(5)],
                rarity_bonus: 1.5,
            },
            ElementCombination {
                combination_id: "earth_lightning_crystal".to_string(),
                elements: vec![Element::Earth, Element::Lightning],
                result_element: Element::Aether,
                fusion_requirements: FusionRequirements {
                    min_energy: 1000,
                    required_materials: [("crystal_essence".to_string(), 3)]
                        .iter()
                        .cloned()
                        .collect(),
                    success_rate: 0.6,
                    fusion_time: 600,
                },
                unlock_conditions: vec![
                    UnlockCondition::ReputationLevel(10),
                    UnlockCondition::StorageThreshold(1000000),
                ],
                rarity_bonus: 2.0,
            },
            ElementCombination {
                combination_id: "void_aether_cosmos".to_string(),
                elements: vec![Element::Void, Element::Aether],
                result_element: Element::Lightning,
                fusion_requirements: FusionRequirements {
                    min_energy: 2000,
                    required_materials: [
                        ("void_essence".to_string(), 1),
                        ("aether_shard".to_string(), 2),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                    success_rate: 0.3,
                    fusion_time: 1200,
                },
                unlock_conditions: vec![
                    UnlockCondition::CustomAchievement("Void Walker".to_string()),
                    UnlockCondition::TimeActive(86400 * 30), // 30 days
                ],
                rarity_bonus: 5.0,
            },
        ];

        for combo in combinations {
            self.element_combinations
                .insert(combo.combination_id.clone(), combo);
        }
    }

    pub fn create_custom_glyph(
        &mut self,
        name: String,
        description: String,
        base_element: Element,
        category: DataCategory,
        importance: Importance,
        visual_properties: VisualProperties,
        creator: String,
        is_public: bool,
    ) -> Result<String> {
        let glyph_id = format!("custom_{}", uuid::Uuid::new_v4().simple());

        let base_glyph = Glyph {
            element: base_element,
            category,
            importance,
            properties: HashMap::new(),
        };

        let custom_glyph = CustomGlyph {
            glyph_id: glyph_id.clone(),
            name,
            description,
            base_glyph,
            visual_properties,
            custom_properties: HashMap::new(),
            creator,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            usage_count: 0,
            is_public,
        };

        self.custom_glyphs.insert(glyph_id.clone(), custom_glyph);

        Ok(glyph_id)
    }

    pub fn apply_template(
        &mut self,
        template_id: &str,
        creator: String,
        customizations: HashMap<String, String>,
    ) -> Result<String> {
        let template = self
            .glyph_templates
            .iter()
            .find(|t| t.template_id == template_id)
            .ok_or_else(|| anyhow!("Template not found"))?;

        let (base_element, category, importance) = self.template_to_glyph_params(template);
        let visual_properties = self.template_to_visual_properties(template);

        let glyph_id = self.create_custom_glyph(
            template.name.clone(),
            template.description.clone(),
            base_element,
            category,
            importance,
            visual_properties,
            creator,
            false,
        )?;

        // Apply customizations
        if let Some(custom_glyph) = self.custom_glyphs.get_mut(&glyph_id) {
            for (key, value) in customizations {
                custom_glyph.custom_properties.insert(key, value);
            }
        }

        Ok(glyph_id)
    }

    fn template_to_glyph_params(
        &self,
        template: &GlyphTemplate,
    ) -> (Element, DataCategory, Importance) {
        let element = match &template.element_pattern {
            ElementPattern::Pure(e) => e.clone(),
            ElementPattern::Dual(e1, _) => e1.clone(),
            ElementPattern::Triad(e1, _, _) => e1.clone(),
            ElementPattern::Balanced => Element::Aether,
            ElementPattern::Chaotic => Element::Void,
        };

        let category = match template.category {
            TemplateCategory::Computing => DataCategory::Code,
            TemplateCategory::Storage => DataCategory::Archive,
            TemplateCategory::Communication => DataCategory::Media,
            TemplateCategory::Analytics => DataCategory::Dataset,
            TemplateCategory::Security => DataCategory::Key,
            TemplateCategory::Gaming => DataCategory::Media,
            TemplateCategory::Creative => DataCategory::Media,
            TemplateCategory::Scientific => DataCategory::Dataset,
            TemplateCategory::Business => DataCategory::Document,
            TemplateCategory::Personal => DataCategory::Archive,
        };

        let importance = match template.rarity {
            TemplateRarity::Common => Importance::Standard,
            TemplateRarity::Uncommon => Importance::Major,
            TemplateRarity::Rare => Importance::Critical,
            TemplateRarity::Epic => Importance::Critical,
            TemplateRarity::Legendary => Importance::Legendary,
        };

        (element, category, importance)
    }

    fn template_to_visual_properties(&self, template: &GlyphTemplate) -> VisualProperties {
        let color_scheme = match template.rarity {
            TemplateRarity::Common => ColorScheme {
                primary_color: "#8B8B8B".to_string(),
                secondary_color: "#A9A9A9".to_string(),
                accent_color: "#D3D3D3".to_string(),
                gradient_direction: GradientDirection::Vertical,
            },
            TemplateRarity::Uncommon => ColorScheme {
                primary_color: "#4CAF50".to_string(),
                secondary_color: "#66BB6A".to_string(),
                accent_color: "#81C784".to_string(),
                gradient_direction: GradientDirection::Diagonal,
            },
            TemplateRarity::Rare => ColorScheme {
                primary_color: "#2196F3".to_string(),
                secondary_color: "#42A5F5".to_string(),
                accent_color: "#64B5F6".to_string(),
                gradient_direction: GradientDirection::Radial,
            },
            TemplateRarity::Epic => ColorScheme {
                primary_color: "#9C27B0".to_string(),
                secondary_color: "#AB47BC".to_string(),
                accent_color: "#BA68C8".to_string(),
                gradient_direction: GradientDirection::Horizontal,
            },
            TemplateRarity::Legendary => ColorScheme {
                primary_color: "#FF9800".to_string(),
                secondary_color: "#FFB74D".to_string(),
                accent_color: "#FFCC02".to_string(),
                gradient_direction: GradientDirection::Radial,
            },
        };

        let animation_style = match template.rarity {
            TemplateRarity::Common => AnimationStyle::Static,
            TemplateRarity::Uncommon => AnimationStyle::Pulse,
            TemplateRarity::Rare => AnimationStyle::Float,
            TemplateRarity::Epic => AnimationStyle::Shimmer,
            TemplateRarity::Legendary => AnimationStyle::Spiral,
        };

        let glow_intensity = match template.rarity {
            TemplateRarity::Common => 0.0,
            TemplateRarity::Uncommon => 0.3,
            TemplateRarity::Rare => 0.5,
            TemplateRarity::Epic => 0.8,
            TemplateRarity::Legendary => 1.0,
        };

        VisualProperties {
            color_scheme,
            animation_style,
            glow_intensity,
            particle_effects: self.get_default_particle_effects(template.rarity.clone()),
            size_multiplier: 1.0,
            opacity: 1.0,
        }
    }

    fn get_default_particle_effects(&self, rarity: TemplateRarity) -> Vec<ParticleEffect> {
        match rarity {
            TemplateRarity::Common => vec![],
            TemplateRarity::Uncommon => vec![ParticleEffect {
                effect_type: ParticleType::Sparks,
                intensity: 0.2,
                color: "#4CAF50".to_string(),
                duration: 2.0,
            }],
            TemplateRarity::Rare => vec![ParticleEffect {
                effect_type: ParticleType::Stars,
                intensity: 0.4,
                color: "#2196F3".to_string(),
                duration: 3.0,
            }],
            TemplateRarity::Epic => vec![
                ParticleEffect {
                    effect_type: ParticleType::Lightning,
                    intensity: 0.6,
                    color: "#9C27B0".to_string(),
                    duration: 1.5,
                },
                ParticleEffect {
                    effect_type: ParticleType::Mist,
                    intensity: 0.3,
                    color: "#AB47BC".to_string(),
                    duration: 4.0,
                },
            ],
            TemplateRarity::Legendary => vec![
                ParticleEffect {
                    effect_type: ParticleType::Embers,
                    intensity: 0.8,
                    color: "#FF9800".to_string(),
                    duration: 2.5,
                },
                ParticleEffect {
                    effect_type: ParticleType::Stars,
                    intensity: 0.5,
                    color: "#FFCC02".to_string(),
                    duration: 3.5,
                },
                ParticleEffect {
                    effect_type: ParticleType::Lightning,
                    intensity: 0.4,
                    color: "#FFB74D".to_string(),
                    duration: 1.0,
                },
            ],
        }
    }

    pub fn save_design(
        &mut self,
        name: String,
        glyph: Glyph,
        visual_properties: VisualProperties,
        notes: String,
        creator: String,
    ) -> String {
        let design_id = format!("design_{}", uuid::Uuid::new_v4().simple());

        let design = SavedDesign {
            design_id: design_id.clone(),
            name,
            glyph,
            visual_properties,
            notes,
            creator,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_favorite: false,
        };

        self.saved_designs.push(design);
        design_id
    }

    pub fn get_available_templates(
        &self,
        category: Option<TemplateCategory>,
    ) -> Vec<&GlyphTemplate> {
        self.glyph_templates
            .iter()
            .filter(|template| {
                category
                    .as_ref()
                    .map_or(true, |cat| &template.category == cat)
            })
            .collect()
    }

    pub fn get_unlocked_combinations(
        &self,
        user_level: u32,
        user_storage: u64,
    ) -> Vec<&ElementCombination> {
        self.element_combinations
            .values()
            .filter(|combo| {
                combo.unlock_conditions.iter().all(|condition| {
                    match condition {
                        UnlockCondition::ReputationLevel(required) => user_level >= *required,
                        UnlockCondition::StorageThreshold(required) => user_storage >= *required,
                        _ => true, // For now, assume other conditions are met
                    }
                })
            })
            .collect()
    }

    pub fn combine_elements(
        &mut self,
        elements: Vec<Element>,
        user_energy: u64,
    ) -> Result<Element> {
        let combination_key = self.find_combination_key(&elements)?;
        let combination = self
            .element_combinations
            .get(&combination_key)
            .ok_or_else(|| anyhow!("No combination found for these elements"))?;

        if user_energy < combination.fusion_requirements.min_energy {
            return Err(anyhow!("Insufficient energy for fusion"));
        }

        // Simple deterministic "randomness" based on elements
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        for element in &elements {
            element.hash(&mut hasher);
        }
        let hash = hasher.finish();
        let success_roll = (hash % 100) as f64 / 100.0;

        if success_roll <= combination.fusion_requirements.success_rate {
            Ok(combination.result_element.clone())
        } else {
            Err(anyhow!("Element combination failed"))
        }
    }

    fn find_combination_key(&self, elements: &[Element]) -> Result<String> {
        let mut sorted_elements = elements.to_vec();
        sorted_elements.sort_by_key(|e| format!("{:?}", e));

        for (key, combo) in &self.element_combinations {
            let mut combo_elements = combo.elements.clone();
            combo_elements.sort_by_key(|e| format!("{:?}", e));

            if sorted_elements == combo_elements {
                return Ok(key.clone());
            }
        }

        Err(anyhow!("No matching combination found"))
    }

    pub fn increment_usage(&mut self, glyph_id: &str) -> Result<()> {
        if let Some(glyph) = self.custom_glyphs.get_mut(glyph_id) {
            glyph.usage_count += 1;
            Ok(())
        } else {
            Err(anyhow!("Custom glyph not found"))
        }
    }

    pub fn get_popular_glyphs(&self, limit: usize) -> Vec<&CustomGlyph> {
        let mut glyphs: Vec<&CustomGlyph> = self.custom_glyphs.values().collect();
        glyphs.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        glyphs.into_iter().take(limit).collect()
    }

    pub fn get_designer_stats(&self) -> GlyphDesignerStats {
        let total_custom_glyphs = self.custom_glyphs.len();
        let public_glyphs = self.custom_glyphs.values().filter(|g| g.is_public).count();

        let total_usage = self.custom_glyphs.values().map(|g| g.usage_count).sum();

        let templates_by_category =
            self.glyph_templates
                .iter()
                .fold(HashMap::new(), |mut acc, template| {
                    *acc.entry(format!("{:?}", template.category)).or_insert(0) += 1;
                    acc
                });

        let designs_count = self.saved_designs.len();

        GlyphDesignerStats {
            total_custom_glyphs,
            public_glyphs,
            total_usage,
            templates_available: self.glyph_templates.len(),
            templates_by_category,
            saved_designs: designs_count,
            element_combinations: self.element_combinations.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphDesignerStats {
    pub total_custom_glyphs: usize,
    pub public_glyphs: usize,
    pub total_usage: u64,
    pub templates_available: usize,
    pub templates_by_category: HashMap<String, usize>,
    pub saved_designs: usize,
    pub element_combinations: usize,
}

impl Default for VisualProperties {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme {
                primary_color: "#FFFFFF".to_string(),
                secondary_color: "#CCCCCC".to_string(),
                accent_color: "#999999".to_string(),
                gradient_direction: GradientDirection::Vertical,
            },
            animation_style: AnimationStyle::Static,
            glow_intensity: 0.0,
            particle_effects: Vec::new(),
            size_multiplier: 1.0,
            opacity: 1.0,
        }
    }
}

impl TemplateRarity {
    pub fn multiplier(&self) -> f64 {
        match self {
            TemplateRarity::Common => 1.0,
            TemplateRarity::Uncommon => 1.2,
            TemplateRarity::Rare => 1.5,
            TemplateRarity::Epic => 2.0,
            TemplateRarity::Legendary => 3.0,
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            TemplateRarity::Common => "#8B8B8B",
            TemplateRarity::Uncommon => "#4CAF50",
            TemplateRarity::Rare => "#2196F3",
            TemplateRarity::Epic => "#9C27B0",
            TemplateRarity::Legendary => "#FF9800",
        }
    }
}
