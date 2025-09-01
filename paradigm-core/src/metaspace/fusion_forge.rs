use crate::metaspace::{Sigil, Glyph, Element, DataCategory, Importance, Tome};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionForge {
    pub active_fusions: HashMap<String, FusionWorkbench>,
    pub fusion_templates: Vec<FusionTemplate>,
    pub completed_fusions: Vec<CompletedFusion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionWorkbench {
    pub fusion_id: String,
    pub selected_sigils: Vec<String>,
    pub target_glyph: Option<Glyph>,
    pub fusion_mode: FusionMode,
    pub energy_cost: u64,
    pub success_probability: f64,
    pub preview_tome: Option<Tome>,
    pub creator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FusionMode {
    Synthesis,
    Transmutation,
    Crystallization,
    Sublimation,
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub required_elements: Vec<Element>,
    pub required_categories: Vec<DataCategory>,
    pub min_sigils: usize,
    pub max_sigils: usize,
    pub fusion_mode: FusionMode,
    pub energy_multiplier: f64,
    pub rarity_bonus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedFusion {
    pub fusion_id: String,
    pub tome_hash: String,
    pub input_sigils: Vec<String>,
    pub fusion_mode: FusionMode,
    pub energy_consumed: u64,
    pub creator: String,
    pub timestamp: u64,
    pub fusion_quality: FusionQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FusionQuality {
    Flawed,
    Standard,
    Refined,
    Perfect,
    Transcendent,
}

impl FusionForge {
    pub fn new() -> Self {
        let mut forge = Self {
            active_fusions: HashMap::new(),
            fusion_templates: Vec::new(),
            completed_fusions: Vec::new(),
        };
        
        forge.initialize_templates();
        forge
    }

    fn initialize_templates(&mut self) {
        self.fusion_templates = vec![
            FusionTemplate {
                template_id: "synthesis_vault".to_string(),
                name: "🏛️ Synthesis Vault".to_string(),
                description: "Combine complementary sigils into a unified tome".to_string(),
                required_elements: vec![Element::Fire, Element::Water],
                required_categories: vec![],
                min_sigils: 2,
                max_sigils: 5,
                fusion_mode: FusionMode::Synthesis,
                energy_multiplier: 1.0,
                rarity_bonus: 1.2,
            },
            FusionTemplate {
                template_id: "elemental_archive".to_string(),
                name: "🌟 Elemental Archive".to_string(),
                description: "Archive all elemental variants of similar data".to_string(),
                required_elements: vec![Element::Fire, Element::Water, Element::Earth, Element::Air],
                required_categories: vec![],
                min_sigils: 4,
                max_sigils: 7,
                fusion_mode: FusionMode::Archive,
                energy_multiplier: 2.0,
                rarity_bonus: 2.5,
            },
            FusionTemplate {
                template_id: "void_transmutation".to_string(),
                name: "🌙 Void Transmutation".to_string(),
                description: "Transform disparate sigils through void essence".to_string(),
                required_elements: vec![Element::Void],
                required_categories: vec![],
                min_sigils: 3,
                max_sigils: 6,
                fusion_mode: FusionMode::Transmutation,
                energy_multiplier: 1.8,
                rarity_bonus: 3.0,
            },
            FusionTemplate {
                template_id: "lightning_crystallization".to_string(),
                name: "⚡ Lightning Crystallization".to_string(),
                description: "Rapidly fuse similar sigils into crystalline structure".to_string(),
                required_elements: vec![Element::Lightning],
                required_categories: vec![DataCategory::Media, DataCategory::Code],
                min_sigils: 2,
                max_sigils: 4,
                fusion_mode: FusionMode::Crystallization,
                energy_multiplier: 0.8,
                rarity_bonus: 1.5,
            },
            FusionTemplate {
                template_id: "aether_sublimation".to_string(),
                name: "🔮 Aether Sublimation".to_string(),
                description: "Elevate rare sigils to transcendent form".to_string(),
                required_elements: vec![Element::Aether],
                required_categories: vec![],
                min_sigils: 1,
                max_sigils: 3,
                fusion_mode: FusionMode::Sublimation,
                energy_multiplier: 5.0,
                rarity_bonus: 10.0,
            },
        ];
    }

    pub fn create_workbench(&mut self, creator: String) -> String {
        let fusion_id = Uuid::new_v4().to_string();
        let workbench = FusionWorkbench {
            fusion_id: fusion_id.clone(),
            selected_sigils: Vec::new(),
            target_glyph: None,
            fusion_mode: FusionMode::Synthesis,
            energy_cost: 0,
            success_probability: 0.0,
            preview_tome: None,
            creator,
        };
        
        self.active_fusions.insert(fusion_id.clone(), workbench);
        fusion_id
    }

    pub fn add_sigil_to_workbench(&mut self, fusion_id: &str, sigil_hash: String, sigils: &HashMap<String, Sigil>) -> Result<()> {
        let workbench = self.active_fusions.get_mut(fusion_id)
            .ok_or_else(|| anyhow!("Workbench not found"))?;

        if workbench.selected_sigils.len() >= 10 {
            return Err(anyhow!("Maximum sigils reached"));
        }

        if !sigils.contains_key(&sigil_hash) {
            return Err(anyhow!("Sigil not found"));
        }

        workbench.selected_sigils.push(sigil_hash);
        self.update_workbench_preview(fusion_id, sigils)?;
        Ok(())
    }

    pub fn remove_sigil_from_workbench(&mut self, fusion_id: &str, sigil_hash: &str, sigils: &HashMap<String, Sigil>) -> Result<()> {
        let workbench = self.active_fusions.get_mut(fusion_id)
            .ok_or_else(|| anyhow!("Workbench not found"))?;

        workbench.selected_sigils.retain(|s| s != sigil_hash);
        self.update_workbench_preview(fusion_id, sigils)?;
        Ok(())
    }

    pub fn set_fusion_mode(&mut self, fusion_id: &str, mode: FusionMode, sigils: &HashMap<String, Sigil>) -> Result<()> {
        let workbench = self.active_fusions.get_mut(fusion_id)
            .ok_or_else(|| anyhow!("Workbench not found"))?;

        workbench.fusion_mode = mode;
        self.update_workbench_preview(fusion_id, sigils)?;
        Ok(())
    }

    fn update_workbench_preview(&mut self, fusion_id: &str, sigils: &HashMap<String, Sigil>) -> Result<()> {
        let workbench = self.active_fusions.get_mut(fusion_id)
            .ok_or_else(|| anyhow!("Workbench not found"))?;

        if workbench.selected_sigils.is_empty() {
            workbench.preview_tome = None;
            workbench.energy_cost = 0;
            workbench.success_probability = 0.0;
            return Ok(());
        }

        let selected_sigils: Vec<&Sigil> = workbench.selected_sigils
            .iter()
            .filter_map(|hash| sigils.get(hash))
            .collect();

        if selected_sigils.is_empty() {
            return Err(anyhow!("No valid sigils selected"));
        }

        // Extract fusion mode to avoid borrowing conflicts
        let fusion_mode = workbench.fusion_mode.clone();
        let fusion_glyph = self.calculate_fusion_glyph(&selected_sigils, &fusion_mode);
        let (energy_cost, success_probability) = self.calculate_fusion_metrics(&selected_sigils, &fusion_mode);

        let preview_tome = Tome {
            tome_hash: format!("preview_{}", fusion_id),
            sigil_hashes: workbench.selected_sigils.clone(),
            glyph: fusion_glyph.clone(),
            originator: workbench.creator.clone(),
            filename: Some(format!("Fused_{}", selected_sigils.len())),
            mime_type: Some("application/paradigm-tome".to_string()),
            total_size: selected_sigils.iter().map(|s| s.size as usize).sum(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            access_policy: crate::metaspace::AccessPolicy::Open,
            fusion_metadata: Some(FusionMetadata {
                fusion_mode: workbench.fusion_mode.clone(),
                component_count: selected_sigils.len(),
                elemental_signature: Self::calculate_elemental_signature_static(&selected_sigils),
                fusion_quality: Self::predict_fusion_quality_static(success_probability),
            }),
        };

        workbench.target_glyph = Some(fusion_glyph);
        workbench.energy_cost = energy_cost;
        workbench.success_probability = success_probability;
        workbench.preview_tome = Some(preview_tome);

        Ok(())
    }

    fn calculate_fusion_glyph(&self, sigils: &[&Sigil], mode: &FusionMode) -> Glyph {
        let elements: Vec<&Element> = sigils.iter().map(|s| &s.glyph.element).collect();
        let categories: Vec<&DataCategory> = sigils.iter().map(|s| &s.glyph.category).collect();
        
        let dominant_element = match mode {
            FusionMode::Synthesis => self.find_complementary_element(&elements),
            FusionMode::Transmutation => Element::Void,
            FusionMode::Crystallization => Element::Lightning,
            FusionMode::Sublimation => Element::Aether,
            FusionMode::Archive => self.find_most_common_element(&elements),
        };

        let fusion_category = self.find_most_common_category(&categories);
        let fusion_importance = self.calculate_fusion_importance(sigils, mode);

        Glyph {
            element: dominant_element,
            category: fusion_category,
            importance: fusion_importance,
            properties: HashMap::new(),
        }
    }

    fn find_complementary_element(&self, elements: &[&Element]) -> Element {
        let element_counts = elements.iter().fold(HashMap::new(), |mut acc, &elem| {
            *acc.entry(elem).or_insert(0) += 1;
            acc
        });

        match element_counts.len() {
            1 => elements[0].clone(),
            2 => {
                if element_counts.contains_key(&Element::Fire) && element_counts.contains_key(&Element::Water) {
                    Element::Aether
                } else if element_counts.contains_key(&Element::Earth) && element_counts.contains_key(&Element::Air) {
                    Element::Lightning
                } else {
                    Element::Void
                }
            }
            _ => Element::Aether,
        }
    }

    fn find_most_common_element(&self, elements: &[&Element]) -> Element {
        elements.iter()
            .fold(HashMap::new(), |mut acc, &elem| {
                *acc.entry(elem).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(elem, _)| elem.clone())
            .unwrap_or(Element::Void)
    }

    fn find_most_common_category(&self, categories: &[&DataCategory]) -> DataCategory {
        categories.iter()
            .fold(HashMap::new(), |mut acc, &cat| {
                *acc.entry(cat).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(cat, _)| cat.clone())
            .unwrap_or(DataCategory::Code)
    }

    fn calculate_fusion_importance(&self, sigils: &[&Sigil], mode: &FusionMode) -> Importance {
        let importance_values: Vec<u32> = sigils.iter().map(|s| match s.glyph.importance {
            Importance::Trivial => 1,
            Importance::Minor => 2,
            Importance::Standard => 3,
            Importance::Major => 4,
            Importance::Critical => 5,
            Importance::Legendary => 6,
        }).collect();

        let avg_importance = importance_values.iter().sum::<u32>() as f64 / importance_values.len() as f64;
        let mode_bonus = match mode {
            FusionMode::Synthesis => 0.2,
            FusionMode::Archive => 0.5,
            FusionMode::Transmutation => 0.8,
            FusionMode::Crystallization => 0.3,
            FusionMode::Sublimation => 2.0,
        };

        let final_value = avg_importance + mode_bonus;
        
        match final_value as u32 {
            1 => Importance::Trivial,
            2 => Importance::Minor,
            3 => Importance::Standard,
            4 => Importance::Major,
            5 => Importance::Critical,
            _ => Importance::Legendary,
        }
    }

    fn calculate_fusion_metrics(&self, sigils: &[&Sigil], mode: &FusionMode) -> (u64, f64) {
        let base_energy = sigils.len() as u64 * 100;
        let size_factor = sigils.iter().map(|s| s.size as usize).sum::<usize>() as u64 / 1024;
        
        let mode_multiplier = match mode {
            FusionMode::Synthesis => 1.0,
            FusionMode::Archive => 2.0,
            FusionMode::Transmutation => 1.8,
            FusionMode::Crystallization => 0.8,
            FusionMode::Sublimation => 5.0,
        };

        let energy_cost = ((base_energy + size_factor) as f64 * mode_multiplier) as u64;
        
        let base_success = 0.95 - (sigils.len() as f64 * 0.05);
        let compatibility_bonus = self.calculate_compatibility_bonus(sigils);
        let success_probability = (base_success + compatibility_bonus).min(0.99).max(0.1);

        (energy_cost, success_probability)
    }

    fn calculate_compatibility_bonus(&self, sigils: &[&Sigil]) -> f64 {
        if sigils.len() < 2 { return 0.0; }

        let mut element_diversity = 0.0;
        let mut category_harmony = 0.0;

        let elements: Vec<&Element> = sigils.iter().map(|s| &s.glyph.element).collect();
        let unique_elements = elements.iter().collect::<std::collections::HashSet<_>>().len();
        element_diversity = match unique_elements {
            1 => 0.1,
            2 => 0.2,
            3..=4 => 0.15,
            _ => 0.05,
        };

        let categories: Vec<&DataCategory> = sigils.iter().map(|s| &s.glyph.category).collect();
        let unique_categories = categories.iter().collect::<std::collections::HashSet<_>>().len();
        category_harmony = if unique_categories <= 2 { 0.1 } else { -0.05 };

        element_diversity + category_harmony
    }

    fn calculate_elemental_signature_static(sigils: &[&Sigil]) -> ElementalSignature {
        let mut signature = ElementalSignature {
            fire: 0, water: 0, earth: 0, air: 0, lightning: 0, void: 0, aether: 0,
        };

        for sigil in sigils {
            match sigil.glyph.element {
                Element::Fire => signature.fire += 1,
                Element::Water => signature.water += 1,
                Element::Earth => signature.earth += 1,
                Element::Air => signature.air += 1,
                Element::Lightning => signature.lightning += 1,
                Element::Void => signature.void += 1,
                Element::Aether => signature.aether += 1,
            }
        }

        signature
    }

    fn predict_fusion_quality_static(success_probability: f64) -> FusionQuality {
        match success_probability {
            p if p >= 0.95 => FusionQuality::Transcendent,
            p if p >= 0.85 => FusionQuality::Perfect,
            p if p >= 0.7 => FusionQuality::Refined,
            p if p >= 0.5 => FusionQuality::Standard,
            _ => FusionQuality::Flawed,
        }
    }

    pub fn execute_fusion(&mut self, fusion_id: &str, creator_energy: u64, sigils: &mut HashMap<String, Sigil>) -> Result<CompletedFusion> {
        let workbench = self.active_fusions.remove(fusion_id)
            .ok_or_else(|| anyhow!("Workbench not found"))?;

        if creator_energy < workbench.energy_cost {
            return Err(anyhow!("Insufficient energy for fusion"));
        }

        if workbench.selected_sigils.is_empty() {
            return Err(anyhow!("No sigils selected for fusion"));
        }

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        workbench.fusion_id.hash(&mut hasher);
        let hash = hasher.finish();
        let random_roll: f64 = (hash % 10000) as f64 / 10000.0;
        let fusion_succeeded = random_roll < workbench.success_probability;

        if !fusion_succeeded {
            return Err(anyhow!("Fusion failed - sigils remain intact"));
        }

        let tome = workbench.preview_tome
            .ok_or_else(|| anyhow!("No preview tome available"))?;

        let fusion_quality = self.predict_fusion_quality(workbench.success_probability);

        for sigil_hash in &workbench.selected_sigils {
            sigils.remove(sigil_hash);
        }

        let completed_fusion = CompletedFusion {
            fusion_id: fusion_id.to_string(),
            tome_hash: tome.tome_hash.clone(),
            input_sigils: workbench.selected_sigils.clone(),
            fusion_mode: workbench.fusion_mode,
            energy_consumed: workbench.energy_cost,
            creator: workbench.creator,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            fusion_quality,
        };

        self.completed_fusions.push(completed_fusion.clone());

        Ok(completed_fusion)
    }

    pub fn get_available_templates(&self, selected_sigils: &[&Sigil]) -> Vec<&FusionTemplate> {
        self.fusion_templates.iter()
            .filter(|template| self.template_matches_selection(template, selected_sigils))
            .collect()
    }

    fn template_matches_selection(&self, template: &FusionTemplate, sigils: &[&Sigil]) -> bool {
        if sigils.len() < template.min_sigils || sigils.len() > template.max_sigils {
            return false;
        }

        if !template.required_elements.is_empty() {
            let sigil_elements: std::collections::HashSet<_> = 
                sigils.iter().map(|s| &s.glyph.element).collect();
            
            let has_required_elements = template.required_elements.iter()
                .all(|req_elem| sigil_elements.contains(&req_elem));
            
            if !has_required_elements {
                return false;
            }
        }

        if !template.required_categories.is_empty() {
            let sigil_categories: std::collections::HashSet<_> = 
                sigils.iter().map(|s| &s.glyph.category).collect();
            
            let has_required_categories = template.required_categories.iter()
                .any(|req_cat| sigil_categories.contains(&req_cat));
            
            if !has_required_categories {
                return false;
            }
        }

        true
    }

    pub fn get_fusion_history(&self, creator: Option<&str>) -> Vec<&CompletedFusion> {
        self.completed_fusions.iter()
            .filter(|fusion| creator.map_or(true, |c| fusion.creator == c))
            .collect()
    }

    pub fn get_workbench_stats(&self) -> FusionForgeStats {
        FusionForgeStats {
            active_workbenches: self.active_fusions.len(),
            completed_fusions: self.completed_fusions.len(),
            total_energy_consumed: self.completed_fusions.iter()
                .map(|f| f.energy_consumed)
                .sum(),
            success_rate: if self.completed_fusions.is_empty() {
                0.0
            } else {
                self.completed_fusions.len() as f64 / 
                (self.completed_fusions.len() + self.active_fusions.len()) as f64
            },
            quality_distribution: self.calculate_quality_distribution(),
        }
    }

    fn calculate_quality_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for fusion in &self.completed_fusions {
            let quality_str = match fusion.fusion_quality {
                FusionQuality::Flawed => "Flawed",
                FusionQuality::Standard => "Standard", 
                FusionQuality::Refined => "Refined",
                FusionQuality::Perfect => "Perfect",
                FusionQuality::Transcendent => "Transcendent",
            };
            *distribution.entry(quality_str.to_string()).or_insert(0) += 1;
        }
        distribution
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionMetadata {
    pub fusion_mode: FusionMode,
    pub component_count: usize,
    pub elemental_signature: ElementalSignature,
    pub fusion_quality: FusionQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementalSignature {
    pub fire: usize,
    pub water: usize,
    pub earth: usize,
    pub air: usize,
    pub lightning: usize,
    pub void: usize,
    pub aether: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionForgeStats {
    pub active_workbenches: usize,
    pub completed_fusions: usize,
    pub total_energy_consumed: u64,
    pub success_rate: f64,
    pub quality_distribution: HashMap<String, usize>,
}

impl FusionMode {
    pub fn description(&self) -> &'static str {
        match self {
            FusionMode::Synthesis => "Harmoniously combine complementary sigils",
            FusionMode::Transmutation => "Transform sigils through void essence",
            FusionMode::Crystallization => "Rapidly fuse similar elements",
            FusionMode::Sublimation => "Elevate rare sigils to transcendent form", 
            FusionMode::Archive => "Create comprehensive collections",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FusionMode::Synthesis => "⚗️",
            FusionMode::Transmutation => "🌙",
            FusionMode::Crystallization => "⚡",
            FusionMode::Sublimation => "🔮",
            FusionMode::Archive => "🏛️",
        }
    }
}

impl FusionQuality {
    pub fn multiplier(&self) -> f64 {
        match self {
            FusionQuality::Flawed => 0.8,
            FusionQuality::Standard => 1.0,
            FusionQuality::Refined => 1.3,
            FusionQuality::Perfect => 1.8,
            FusionQuality::Transcendent => 3.0,
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            FusionQuality::Flawed => "#8B4513",
            FusionQuality::Standard => "#C0C0C0", 
            FusionQuality::Refined => "#4169E1",
            FusionQuality::Perfect => "#9370DB",
            FusionQuality::Transcendent => "#FFD700",
        }
    }
}