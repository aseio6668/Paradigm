use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Sigil, Glyph, Element, DataCategory, Importance};
use super::ui::{UIFilters, SigilDisplayData};

/// Sigil Explorer - Browse and discover sigils by symbolic meaning
pub struct SigilExplorer {
    /// Current search and filter state
    pub filters: ExplorerFilters,
    
    /// Search results
    pub results: Vec<SigilSearchResult>,
    
    /// Pagination state
    pub pagination: PaginationState,
    
    /// Sorting configuration
    pub sort: SortConfig,
    
    /// View mode (grid, list, tree)
    pub view_mode: ExplorerViewMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerFilters {
    /// Text search across content, originators, metadata
    pub search_query: String,
    
    /// Filter by glyph elements
    pub elements: Vec<Element>,
    
    /// Filter by data categories
    pub categories: Vec<DataCategory>,
    
    /// Filter by importance levels
    pub importance_levels: Vec<Importance>,
    
    /// Filter by file size range (bytes)
    pub size_range: Option<(usize, usize)>,
    
    /// Filter by age (hours)
    pub age_range: Option<(u32, u32)>,
    
    /// Filter by keeper count range
    pub keeper_count_range: Option<(usize, usize)>,
    
    /// Filter by retrieval count
    pub retrieval_count_range: Option<(u32, u32)>,
    
    /// Filter by specific originators
    pub originators: Vec<String>,
    
    /// Show only sigils with custom properties
    pub has_custom_properties: Option<bool>,
    
    /// Filter by access policy
    pub access_policy: Option<AccessPolicyFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPolicyFilter {
    PublicOnly,
    PrivateOnly,
    RestrictedOnly,
    PaidOnly,
    StakedOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigilSearchResult {
    /// Display-formatted sigil data
    pub sigil_data: SigilDisplayData,
    
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: f32,
    
    /// Match highlights for UI
    pub match_highlights: Vec<MatchHighlight>,
    
    /// Related sigils (by glyph similarity)
    pub related_sigils: Vec<String>, // content hashes
    
    /// Availability status
    pub availability: AvailabilityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchHighlight {
    /// Field that matched (content_hash, originator, metadata, etc.)
    pub field: String,
    
    /// Start position of match
    pub start: usize,
    
    /// Length of match
    pub length: usize,
    
    /// Type of match
    pub match_type: MatchType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Fuzzy,
    Prefix,
    Contains,
    Glyph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityStatus {
    /// Number of keepers currently hosting
    pub keeper_count: usize,
    
    /// Minimum keepers required for good availability
    pub required_keepers: usize,
    
    /// Estimated retrieval time
    pub estimated_retrieval_ms: u64,
    
    /// Health status
    pub health: AvailabilityHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AvailabilityHealth {
    Excellent, // > 5 keepers
    Good,      // 3-5 keepers
    Fair,      // 2 keepers
    Poor,      // 1 keeper
    Unavailable, // 0 keepers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationState {
    pub current_page: usize,
    pub page_size: usize,
    pub total_results: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortConfig {
    pub field: SortField,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortField {
    Relevance,
    CreatedAt,
    Size,
    KeeperCount,
    RetrievalCount,
    Importance,
    Originator,
    GlyphElement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExplorerViewMode {
    /// Grid of sigil cards with glyphs and metadata
    Grid,
    
    /// Detailed list view with full information
    List,
    
    /// Hierarchical tree organized by glyphs
    Tree,
    
    /// Graph view showing relationships
    Graph,
    
    /// Timeline view organized by creation date
    Timeline,
}

impl SigilExplorer {
    pub fn new() -> Self {
        Self {
            filters: ExplorerFilters::default(),
            results: Vec::new(),
            pagination: PaginationState::default(),
            sort: SortConfig::default(),
            view_mode: ExplorerViewMode::Grid,
        }
    }
    
    /// Perform search with current filters
    pub async fn search(&mut self, sigils: &HashMap<String, Sigil>) -> Result<()> {
        let mut results = Vec::new();
        
        for (hash, sigil) in sigils {
            if self.matches_filters(sigil) {
                let relevance = self.calculate_relevance(sigil);
                let highlights = self.find_highlights(sigil);
                let related = self.find_related_sigils(sigil, sigils);
                let availability = self.assess_availability(sigil);
                
                results.push(SigilSearchResult {
                    sigil_data: self.convert_to_display_data(sigil),
                    relevance_score: relevance,
                    match_highlights: highlights,
                    related_sigils: related,
                    availability,
                });
            }
        }
        
        // Sort results
        self.sort_results(&mut results);
        
        // Update pagination
        self.update_pagination(results.len());
        
        // Apply pagination
        let start = self.pagination.current_page * self.pagination.page_size;
        let end = (start + self.pagination.page_size).min(results.len());
        self.results = results[start..end].to_vec();
        
        Ok(())
    }
    
    /// Check if a sigil matches current filters
    fn matches_filters(&self, sigil: &Sigil) -> bool {
        // Text search
        if !self.filters.search_query.is_empty() {
            let query_lower = self.filters.search_query.to_lowercase();
            let searchable_text = format!("{} {} {}", 
                sigil.content_hash, 
                sigil.originator, 
                sigil.glyph.name()
            ).to_lowercase();
            
            if !searchable_text.contains(&query_lower) {
                return false;
            }
        }
        
        // Element filter
        if !self.filters.elements.is_empty() && !self.filters.elements.contains(&sigil.glyph.element) {
            return false;
        }
        
        // Category filter
        if !self.filters.categories.is_empty() && !self.filters.categories.contains(&sigil.glyph.category) {
            return false;
        }
        
        // Importance filter
        if !self.filters.importance_levels.is_empty() && !self.filters.importance_levels.contains(&sigil.glyph.importance) {
            return false;
        }
        
        // Size range filter
        if let Some((min_size, max_size)) = self.filters.size_range {
            if sigil.size < min_size || sigil.size > max_size {
                return false;
            }
        }
        
        // Age range filter
        if let Some((min_hours, max_hours)) = self.filters.age_range {
            let age_hours = chrono::Utc::now()
                .signed_duration_since(sigil.created_at)
                .num_hours() as u32;
            if age_hours < min_hours || age_hours > max_hours {
                return false;
            }
        }
        
        // Keeper count filter
        if let Some((min_keepers, max_keepers)) = self.filters.keeper_count_range {
            let keeper_count = sigil.keepers.len();
            if keeper_count < min_keepers || keeper_count > max_keepers {
                return false;
            }
        }
        
        // Retrieval count filter
        if let Some((min_retrievals, max_retrievals)) = self.filters.retrieval_count_range {
            let retrieval_count = sigil.retrievals.len() as u32;
            if retrieval_count < min_retrievals || retrieval_count > max_retrievals {
                return false;
            }
        }
        
        // Originator filter
        if !self.filters.originators.is_empty() && !self.filters.originators.contains(&sigil.originator) {
            return false;
        }
        
        // Custom properties filter
        if let Some(require_custom) = self.filters.has_custom_properties {
            let has_custom = !sigil.metadata.is_empty();
            if require_custom && !has_custom || !require_custom && has_custom {
                return false;
            }
        }
        
        true
    }
    
    /// Calculate relevance score for search ranking
    fn calculate_relevance(&self, sigil: &Sigil) -> f32 {
        let mut score = 0.0f32;
        
        // Base score from importance
        score += match sigil.glyph.importance {
            Importance::Critical => 1.0,
            Importance::Major => 0.8,
            Importance::Standard => 0.6,
            Importance::Minor => 0.4,
            Importance::Trivial => 0.2,
            Importance::Legendary => 1.0,
        };
        
        // Bonus for keeper count (redundancy)
        score += (sigil.keepers.len() as f32 * 0.1).min(0.5);
        
        // Bonus for retrieval activity
        score += (sigil.retrievals.len() as f32 * 0.05).min(0.3);
        
        // Text relevance (if searching)
        if !self.filters.search_query.is_empty() {
            let query_lower = self.filters.search_query.to_lowercase();
            
            // Exact hash match gets highest score
            if sigil.content_hash.to_lowercase().contains(&query_lower) {
                score += 2.0;
            }
            
            // Originator match
            if sigil.originator.to_lowercase().contains(&query_lower) {
                score += 0.5;
            }
            
            // Glyph name match
            if sigil.glyph.name().to_lowercase().contains(&query_lower) {
                score += 0.3;
            }
        }
        
        score.min(1.0)
    }
    
    /// Find text highlights for search matches
    fn find_highlights(&self, sigil: &Sigil) -> Vec<MatchHighlight> {
        let mut highlights = Vec::new();
        
        if !self.filters.search_query.is_empty() {
            let query = &self.filters.search_query.to_lowercase();
            
            // Check content hash
            if let Some(pos) = sigil.content_hash.to_lowercase().find(query) {
                highlights.push(MatchHighlight {
                    field: "content_hash".to_string(),
                    start: pos,
                    length: query.len(),
                    match_type: MatchType::Contains,
                });
            }
            
            // Check originator
            if let Some(pos) = sigil.originator.to_lowercase().find(query) {
                highlights.push(MatchHighlight {
                    field: "originator".to_string(),
                    start: pos,
                    length: query.len(),
                    match_type: MatchType::Contains,
                });
            }
            
            // Check glyph
            if sigil.glyph.name().to_lowercase().contains(query) {
                highlights.push(MatchHighlight {
                    field: "glyph".to_string(),
                    start: 0,
                    length: sigil.glyph.name().len(),
                    match_type: MatchType::Glyph,
                });
            }
        }
        
        highlights
    }
    
    /// Find related sigils by glyph similarity
    fn find_related_sigils(&self, sigil: &Sigil, all_sigils: &HashMap<String, Sigil>) -> Vec<String> {
        let mut related = Vec::new();
        
        for (hash, other_sigil) in all_sigils {
            if hash == &sigil.content_hash {
                continue; // Skip self
            }
            
            let similarity = self.calculate_glyph_similarity(&sigil.glyph, &other_sigil.glyph);
            if similarity > 0.5 {
                related.push(hash.clone());
            }
            
            // Limit to 5 related sigils
            if related.len() >= 5 {
                break;
            }
        }
        
        related
    }
    
    /// Calculate similarity between two glyphs
    fn calculate_glyph_similarity(&self, glyph1: &Glyph, glyph2: &Glyph) -> f32 {
        let mut similarity = 0.0f32;
        
        // Same element = high similarity
        if glyph1.element == glyph2.element {
            similarity += 0.5;
        }
        
        // Same category = medium similarity
        if glyph1.category == glyph2.category {
            similarity += 0.3;
        }
        
        // Same importance = low similarity
        if glyph1.importance == glyph2.importance {
            similarity += 0.2;
        }
        
        similarity.min(1.0)
    }
    
    /// Assess availability status of a sigil
    fn assess_availability(&self, sigil: &Sigil) -> AvailabilityStatus {
        let keeper_count = sigil.keepers.len();
        let required_keepers = sigil.glyph.recommended_keeper_count();
        
        let health = match keeper_count {
            0 => AvailabilityHealth::Unavailable,
            1 => AvailabilityHealth::Poor,
            2 => AvailabilityHealth::Fair,
            3..=5 => AvailabilityHealth::Good,
            _ => AvailabilityHealth::Excellent,
        };
        
        let estimated_retrieval_ms = match health {
            AvailabilityHealth::Excellent => 100,
            AvailabilityHealth::Good => 250,
            AvailabilityHealth::Fair => 500,
            AvailabilityHealth::Poor => 1000,
            AvailabilityHealth::Unavailable => u64::MAX,
        };
        
        AvailabilityStatus {
            keeper_count,
            required_keepers,
            estimated_retrieval_ms,
            health,
        }
    }
    
    fn convert_to_display_data(&self, sigil: &Sigil) -> SigilDisplayData {
        SigilDisplayData {
            content_hash: sigil.content_hash.clone(),
            glyph_symbol: sigil.glyph.symbol().to_string(),
            glyph_name: sigil.glyph.name(),
            size_formatted: self.format_bytes(sigil.size),
            age_formatted: self.format_age(sigil.created_at),
            keeper_count: sigil.keepers.len(),
            retrieval_count: sigil.retrievals.len() as u32,
            originator_short: self.shorten_id(&sigil.originator),
            importance_color: self.importance_to_color(&sigil.glyph.importance),
            dna_string: sigil.get_dna_string(),
        }
    }
    
    /// Sort results based on current sort configuration
    fn sort_results(&self, results: &mut Vec<SigilSearchResult>) {
        match self.sort.field {
            SortField::Relevance => {
                results.sort_by(|a, b| {
                    let cmp = b.relevance_score.partial_cmp(&a.relevance_score).unwrap();
                    if self.sort.direction == SortDirection::Ascending { cmp.reverse() } else { cmp }
                });
            }
            SortField::CreatedAt => {
                results.sort_by(|a, b| {
                    let cmp = a.sigil_data.age_formatted.cmp(&b.sigil_data.age_formatted);
                    if self.sort.direction == SortDirection::Ascending { cmp } else { cmp.reverse() }
                });
            }
            SortField::Size => {
                results.sort_by(|a, b| {
                    let cmp = a.sigil_data.size_formatted.cmp(&b.sigil_data.size_formatted);
                    if self.sort.direction == SortDirection::Ascending { cmp } else { cmp.reverse() }
                });
            }
            SortField::KeeperCount => {
                results.sort_by(|a, b| {
                    let cmp = a.availability.keeper_count.cmp(&b.availability.keeper_count);
                    if self.sort.direction == SortDirection::Ascending { cmp } else { cmp.reverse() }
                });
            }
            SortField::RetrievalCount => {
                results.sort_by(|a, b| {
                    let cmp = a.sigil_data.retrieval_count.cmp(&b.sigil_data.retrieval_count);
                    if self.sort.direction == SortDirection::Ascending { cmp } else { cmp.reverse() }
                });
            }
            _ => {} // Other sort fields would be implemented similarly
        }
    }
    
    fn update_pagination(&mut self, total_results: usize) {
        self.pagination.total_results = total_results;
        self.pagination.total_pages = (total_results + self.pagination.page_size - 1) / self.pagination.page_size;
        
        // Ensure current page is valid
        if self.pagination.current_page >= self.pagination.total_pages && self.pagination.total_pages > 0 {
            self.pagination.current_page = self.pagination.total_pages - 1;
        }
    }
    
    // Utility functions (similar to UI module)
    fn format_bytes(&self, bytes: usize) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.1} {}", size, UNITS[unit_index])
    }
    
    fn format_age(&self, timestamp: chrono::DateTime<chrono::Utc>) -> String {
        let duration = chrono::Utc::now().signed_duration_since(timestamp);
        
        if duration.num_days() > 0 {
            format!("{} days", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours", duration.num_hours())
        } else {
            format!("{} minutes", duration.num_minutes().max(1))
        }
    }
    
    fn shorten_id(&self, id: &str) -> String {
        if id.len() > 12 {
            format!("{}...{}", &id[0..6], &id[id.len()-4..])
        } else {
            id.to_string()
        }
    }
    
    fn importance_to_color(&self, importance: &Importance) -> String {
        match importance {
            Importance::Critical => "#ff4444".to_string(),
            Importance::Major => "#ff8800".to_string(),
            Importance::Standard => "#00aa00".to_string(),
            Importance::Minor => "#888888".to_string(),
            Importance::Trivial => "#444444".to_string(),
            Importance::Legendary => "#ff00ff".to_string(),
        }
    }
}

// Default implementations

impl Default for ExplorerFilters {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            elements: Vec::new(),
            categories: Vec::new(),
            importance_levels: Vec::new(),
            size_range: None,
            age_range: None,
            keeper_count_range: None,
            retrieval_count_range: None,
            originators: Vec::new(),
            has_custom_properties: None,
            access_policy: None,
        }
    }
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            current_page: 0,
            page_size: 20,
            total_results: 0,
            total_pages: 0,
        }
    }
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            field: SortField::Relevance,
            direction: SortDirection::Descending,
        }
    }
}