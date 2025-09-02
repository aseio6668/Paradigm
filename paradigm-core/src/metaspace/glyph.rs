use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A Glyph represents the symbolic nature and purpose of stored data
/// Glyphs add meaning and classification to content in the metaspace
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Glyph {
    /// Primary element type
    pub element: Element,

    /// Data category
    pub category: DataCategory,

    /// Importance level affects storage rewards
    pub importance: Importance,

    /// Custom properties for specialized use cases
    pub properties: HashMap<String, String>,
}

/// Primary elemental classification inspired by classical elements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Element {
    /// 🔥 Fire: Computation, processing, active data
    Fire,

    /// 💧 Water: Flow, streaming data, liquid assets  
    Water,

    /// 🌍 Earth: Archive, permanent storage, immutable data
    Earth,

    /// 💨 Air: Communication, messages, volatile data
    Air,

    /// ⚡ Lightning: High-priority, fast-access data
    Lightning,

    /// 🌙 Void: Unknown, encrypted, or dark data
    Void,

    /// 🔮 Aether: Meta-data, system files, protocol data
    Aether,
}

/// Data category classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DataCategory {
    /// Raw file storage
    Archive,

    /// Machine learning models
    Model,

    /// Training datasets
    Dataset,

    /// Computation results
    Result,

    /// Media files (images, video, audio)
    Media,

    /// Documents and text
    Document,

    /// Code and software
    Code,

    /// Configuration and metadata
    Config,

    /// Temporary or cache data
    Temporary,

    /// Unknown or unclassified
    Unknown,

    /// Cryptographic keys and security data
    Key,

    /// Identity and profile data
    Identity,

    /// Protocol and system data
    Protocol,
}

/// Importance level affects storage priority and rewards
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Importance {
    /// Temporary data, minimal storage requirements
    Trivial,

    /// Low priority data
    Minor,

    /// Normal priority data
    Standard,

    /// Important user data
    Major,

    /// Critical system data - highest priority
    Critical,

    /// Legendary artifacts and unique data
    Legendary,
}

impl Glyph {
    /// Create a new glyph
    pub fn new(element: Element, category: DataCategory, importance: Importance) -> Self {
        Self {
            element,
            category,
            importance,
            properties: HashMap::new(),
        }
    }

    /// Create a glyph from a simple string representation
    pub fn from_string(glyph_str: &str) -> Self {
        let parts: Vec<&str> = glyph_str.split(':').collect();

        let element = match parts.first() {
            Some(&"fire") => Element::Fire,
            Some(&"water") => Element::Water,
            Some(&"earth") => Element::Earth,
            Some(&"air") => Element::Air,
            Some(&"lightning") => Element::Lightning,
            Some(&"void") => Element::Void,
            Some(&"aether") => Element::Aether,
            _ => Element::Void,
        };

        let category = match parts.get(1) {
            Some(&"archive") => DataCategory::Archive,
            Some(&"model") => DataCategory::Model,
            Some(&"dataset") => DataCategory::Dataset,
            Some(&"result") => DataCategory::Result,
            Some(&"media") => DataCategory::Media,
            Some(&"document") => DataCategory::Document,
            Some(&"code") => DataCategory::Code,
            Some(&"config") => DataCategory::Config,
            Some(&"temp") => DataCategory::Temporary,
            _ => DataCategory::Unknown,
        };

        let importance = match parts.get(2) {
            Some(&"critical") => Importance::Critical,
            Some(&"high") => Importance::Major,
            Some(&"normal") => Importance::Standard,
            Some(&"low") => Importance::Minor,
            Some(&"ephemeral") => Importance::Trivial,
            _ => Importance::Standard,
        };

        Self::new(element, category, importance)
    }

    /// Get the multiplier for reward calculation based on importance
    pub fn importance_multiplier(&self) -> u64 {
        match self.importance {
            Importance::Critical => 5,
            Importance::Major => 3,
            Importance::Standard => 2,
            Importance::Minor => 1,
            Importance::Trivial => 1,
            Importance::Legendary => 10,
        }
    }

    /// Get storage priority (higher = more important)
    pub fn storage_priority(&self) -> u8 {
        match self.importance {
            Importance::Critical => 10,
            Importance::Major => 7,
            Importance::Standard => 5,
            Importance::Minor => 3,
            Importance::Trivial => 1,
            Importance::Legendary => 15,
        }
    }

    /// Add a custom property
    pub fn add_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    /// Get a custom property
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    /// Get the symbolic representation (emoji/unicode)
    pub fn symbol(&self) -> &str {
        match self.element {
            Element::Fire => "🔥",
            Element::Water => "💧",
            Element::Earth => "🌍",
            Element::Air => "💨",
            Element::Lightning => "⚡",
            Element::Void => "🌙",
            Element::Aether => "🔮",
        }
    }

    /// Get a descriptive name for the glyph
    pub fn name(&self) -> String {
        format!(
            "{} {} {}",
            self.element.name(),
            self.category.name(),
            self.importance.name()
        )
    }

    /// Check if this glyph represents machine learning data
    pub fn is_ml_related(&self) -> bool {
        matches!(
            self.category,
            DataCategory::Model | DataCategory::Dataset | DataCategory::Result
        )
    }

    /// Check if this glyph represents permanent storage
    pub fn is_permanent(&self) -> bool {
        matches!(self.element, Element::Earth) || matches!(self.importance, Importance::Critical)
    }

    /// Get recommended minimum keeper count based on importance
    pub fn recommended_keeper_count(&self) -> usize {
        match self.importance {
            Importance::Critical => 7,
            Importance::Major => 5,
            Importance::Standard => 3,
            Importance::Minor => 2,
            Importance::Trivial => 1,
            Importance::Legendary => 10,
        }
    }
}

impl fmt::Display for Glyph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.element, self.category, self.importance)
    }
}

impl Element {
    pub fn name(&self) -> &str {
        match self {
            Element::Fire => "Fire",
            Element::Water => "Water",
            Element::Earth => "Earth",
            Element::Air => "Air",
            Element::Lightning => "Lightning",
            Element::Void => "Void",
            Element::Aether => "Aether",
        }
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name().to_lowercase())
    }
}

impl DataCategory {
    pub fn name(&self) -> &str {
        match self {
            DataCategory::Archive => "Archive",
            DataCategory::Model => "Model",
            DataCategory::Dataset => "Dataset",
            DataCategory::Result => "Result",
            DataCategory::Media => "Media",
            DataCategory::Document => "Document",
            DataCategory::Code => "Code",
            DataCategory::Config => "Config",
            DataCategory::Temporary => "Temporary",
            DataCategory::Unknown => "Unknown",
            DataCategory::Key => "Key",
            DataCategory::Identity => "Identity",
            DataCategory::Protocol => "Protocol",
        }
    }
}

impl fmt::Display for DataCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name().to_lowercase())
    }
}

impl Importance {
    pub fn name(&self) -> &str {
        match self {
            Importance::Critical => "Critical",
            Importance::Major => "Major",
            Importance::Standard => "Standard",
            Importance::Minor => "Minor",
            Importance::Trivial => "Trivial",
            Importance::Legendary => "Legendary",
        }
    }
}

impl fmt::Display for Importance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name().to_lowercase())
    }
}

/// Glyph system manages the symbolic classification system
pub struct GlyphSystem {
    /// Predefined glyph templates
    templates: HashMap<String, Glyph>,

    /// Custom user-defined glyphs  
    custom_glyphs: HashMap<String, Glyph>,
}

impl GlyphSystem {
    pub fn new() -> Self {
        let mut system = Self {
            templates: HashMap::new(),
            custom_glyphs: HashMap::new(),
        };

        system.init_default_templates();
        system
    }

    /// Initialize common glyph templates
    fn init_default_templates(&mut self) {
        // ML/AI related glyphs
        self.templates.insert(
            "ml_model".to_string(),
            Glyph::new(Element::Fire, DataCategory::Model, Importance::Major),
        );

        self.templates.insert(
            "training_data".to_string(),
            Glyph::new(Element::Water, DataCategory::Dataset, Importance::Major),
        );

        self.templates.insert(
            "ml_result".to_string(),
            Glyph::new(
                Element::Lightning,
                DataCategory::Result,
                Importance::Standard,
            ),
        );

        // Storage related glyphs
        self.templates.insert(
            "archive".to_string(),
            Glyph::new(Element::Earth, DataCategory::Archive, Importance::Standard),
        );

        self.templates.insert(
            "backup".to_string(),
            Glyph::new(Element::Earth, DataCategory::Archive, Importance::Major),
        );

        self.templates.insert(
            "temp_cache".to_string(),
            Glyph::new(Element::Air, DataCategory::Temporary, Importance::Trivial),
        );

        // Media glyphs
        self.templates.insert(
            "media".to_string(),
            Glyph::new(Element::Water, DataCategory::Media, Importance::Standard),
        );

        self.templates.insert(
            "document".to_string(),
            Glyph::new(Element::Air, DataCategory::Document, Importance::Standard),
        );

        // System glyphs
        self.templates.insert(
            "system_config".to_string(),
            Glyph::new(Element::Aether, DataCategory::Config, Importance::Critical),
        );

        self.templates.insert(
            "encrypted".to_string(),
            Glyph::new(Element::Void, DataCategory::Unknown, Importance::Major),
        );
    }

    /// Get a glyph by template name
    pub fn get_template(&self, name: &str) -> Option<Glyph> {
        self.templates.get(name).cloned()
    }

    /// Register a custom glyph
    pub fn register_custom_glyph(&mut self, name: String, glyph: Glyph) {
        self.custom_glyphs.insert(name, glyph);
    }

    /// Get a custom glyph
    pub fn get_custom_glyph(&self, name: &str) -> Option<Glyph> {
        self.custom_glyphs.get(name).cloned()
    }

    /// List all available template names
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Suggest a glyph based on filename or metadata
    pub fn suggest_glyph(&self, filename: Option<&str>, mime_type: Option<&str>) -> Glyph {
        // Simple heuristics for glyph suggestion
        if let Some(filename) = filename {
            let lower = filename.to_lowercase();

            // ML related files
            if lower.contains("model") || lower.ends_with(".pkl") || lower.ends_with(".pt") {
                return self.get_template("ml_model").unwrap_or_default();
            }

            if lower.contains("train") || lower.contains("dataset") || lower.ends_with(".csv") {
                return self.get_template("training_data").unwrap_or_default();
            }

            // Archive files
            if lower.ends_with(".zip") || lower.ends_with(".tar") || lower.ends_with(".gz") {
                return self.get_template("archive").unwrap_or_default();
            }

            // Documents
            if lower.ends_with(".pdf") || lower.ends_with(".doc") || lower.ends_with(".txt") {
                return self.get_template("document").unwrap_or_default();
            }
        }

        if let Some(mime_type) = mime_type {
            if mime_type.starts_with("image/")
                || mime_type.starts_with("video/")
                || mime_type.starts_with("audio/")
            {
                return self.get_template("media").unwrap_or_default();
            }
        }

        // Default glyph
        Glyph::new(Element::Void, DataCategory::Unknown, Importance::Standard)
    }
}

impl Default for Glyph {
    fn default() -> Self {
        Self::new(Element::Void, DataCategory::Unknown, Importance::Standard)
    }
}
