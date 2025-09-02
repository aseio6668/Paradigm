use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::rewards::{RewardStatistics, RewardTransaction, RewardType};
use super::{Keeper, Sigil};
use crate::Amount;

/// Contribution Ledger - Track earnings, hosted sigils, and network contribution history
pub struct ContributionLedger {
    /// Current user/keeper ID
    pub keeper_id: String,

    /// Ledger view configuration
    pub view_config: LedgerViewConfig,

    /// Cached ledger data
    pub cached_data: Option<LedgerData>,

    /// Filters and sorting
    pub filters: LedgerFilters,

    /// Export settings
    pub export_config: ExportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerViewConfig {
    /// Time period to display
    pub time_period: TimePeriod,

    /// Metrics to highlight
    pub highlighted_metrics: Vec<MetricType>,

    /// Chart display options
    pub chart_config: ChartConfig,

    /// Table pagination
    pub pagination: PaginationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimePeriod {
    Last24Hours,
    LastWeek,
    LastMonth,
    LastYear,
    AllTime,
    Custom {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    TotalEarnings,
    StorageRewards,
    ProofBonuses,
    RetrievalBonuses,
    UptimeBonuses,
    SigilCount,
    RetrievalCount,
    Reputation,
    Uptime,
    StorageUtilization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Chart types to display
    pub chart_types: Vec<ChartType>,

    /// Color scheme for charts
    pub color_scheme: ChartColorScheme,

    /// Animation settings
    pub animations_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    /// Earnings over time (line chart)
    EarningsTimeline,

    /// Reward type breakdown (pie chart)
    RewardBreakdown,

    /// Storage utilization over time
    StorageUtilization,

    /// Reputation progression
    ReputationTrend,

    /// Sigil activity heatmap
    SigilActivity,

    /// Network contribution comparison
    NetworkComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartColorScheme {
    Mythic,        // Purples and golds
    Elements,      // Based on glyph elements
    Performance,   // Green to red gradient
    Monochrome,    // Grayscale
    Accessibility, // High contrast
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    pub items_per_page: usize,
    pub current_page: usize,
    pub show_page_size_options: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerData {
    /// Summary statistics for the selected period
    pub summary: LedgerSummary,

    /// Detailed transaction history
    pub transactions: Vec<EnhancedRewardTransaction>,

    /// Hosted sigils with performance metrics
    pub hosted_sigils: Vec<HostedSigilInfo>,

    /// Performance trends
    pub trends: PerformanceTrends,

    /// Comparative rankings
    pub rankings: NetworkRankings,

    /// Achievements and milestones
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSummary {
    /// Total PAR earned in selected period
    pub total_earned: Amount,

    /// Earnings by reward type
    pub earnings_by_type: HashMap<RewardType, Amount>,

    /// Current reputation score
    pub current_reputation: f64,

    /// Reputation change in period
    pub reputation_change: f64,

    /// Total sigils hosted
    pub total_sigils_hosted: usize,

    /// New sigils added in period
    pub new_sigils_this_period: usize,

    /// Total data retrievals served
    pub total_retrievals_served: u32,

    /// Average response time
    pub avg_response_time_ms: f64,

    /// Uptime percentage for period
    pub uptime_percentage: f64,

    /// Storage utilization
    pub storage_utilization: f64,

    /// Network contribution score
    pub contribution_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRewardTransaction {
    /// Base transaction data
    pub transaction: RewardTransaction,

    /// Formatted display strings
    pub amount_formatted: String,
    pub timestamp_formatted: String,
    pub age_formatted: String,

    /// Related sigil info (if applicable)
    pub sigil_info: Option<SigilInfo>,

    /// Transaction impact on reputation
    pub reputation_impact: f64,

    /// Visual indicators
    pub visual_indicators: Vec<TransactionIndicator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigilInfo {
    pub content_hash: String,
    pub glyph_symbol: String,
    pub glyph_name: String,
    pub size_formatted: String,
    pub importance_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionIndicator {
    /// Large reward amount
    HighValue,

    /// First time bonus
    FirstTime,

    /// Streak bonus
    Streak(u32),

    /// Performance bonus
    PerformanceBonus,

    /// Penalty applied
    Penalty,

    /// Milestone reached
    Milestone(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostedSigilInfo {
    /// Basic sigil information
    pub sigil_info: SigilInfo,

    /// When this keeper started hosting
    pub hosting_since: DateTime<Utc>,

    /// Total retrievals served for this sigil
    pub retrievals_served: u32,

    /// PAR earned from this sigil
    pub total_earned: Amount,

    /// Last retrieval timestamp
    pub last_retrieval: Option<DateTime<Utc>>,

    /// Performance metrics
    pub avg_response_time_ms: f64,
    pub success_rate: f64,

    /// Other keepers hosting this sigil
    pub other_keepers: Vec<String>,

    /// Health status
    pub health_status: SigilHealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SigilHealthStatus {
    Healthy,  // Sufficient keepers, good performance
    AtRisk,   // Few keepers or declining performance
    Critical, // Only this keeper or very poor performance
    Orphaned, // This keeper is the only host
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    /// Daily earnings for the period
    pub daily_earnings: Vec<DailyMetric>,

    /// Reputation progression
    pub reputation_history: Vec<ReputationPoint>,

    /// Storage utilization over time
    pub storage_utilization_history: Vec<StoragePoint>,

    /// Response time trends
    pub response_time_history: Vec<ResponseTimePoint>,

    /// Uptime trends
    pub uptime_history: Vec<UptimePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetric {
    pub date: DateTime<Utc>,
    pub value: f64,
    pub formatted_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationPoint {
    pub timestamp: DateTime<Utc>,
    pub reputation: f64,
    pub event: Option<String>, // What caused the change
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePoint {
    pub timestamp: DateTime<Utc>,
    pub utilization_percentage: f64,
    pub total_size_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimePoint {
    pub timestamp: DateTime<Utc>,
    pub avg_response_ms: f64,
    pub request_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimePoint {
    pub timestamp: DateTime<Utc>,
    pub uptime_percentage: f64,
    pub downtime_incidents: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRankings {
    /// Rank by total earnings
    pub earnings_rank: NetworkRank,

    /// Rank by reputation
    pub reputation_rank: NetworkRank,

    /// Rank by sigil count
    pub storage_rank: NetworkRank,

    /// Rank by uptime
    pub uptime_rank: NetworkRank,

    /// Overall contribution rank
    pub overall_rank: NetworkRank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRank {
    pub current_rank: usize,
    pub total_keepers: usize,
    pub percentile: f64,
    pub rank_change: RankChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RankChange {
    Up(usize),
    Down(usize),
    Same,
    New,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub achievement_id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub achieved_at: DateTime<Utc>,
    pub reward_amount: Option<Amount>,
    pub rarity: AchievementRarity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerFilters {
    /// Filter by reward types
    pub reward_types: Vec<RewardType>,

    /// Filter by transaction amount range
    pub amount_range: Option<(Amount, Amount)>,

    /// Filter by specific sigils
    pub sigil_hashes: Vec<String>,

    /// Show only transactions above threshold
    pub min_amount_threshold: Option<Amount>,

    /// Text search in transaction metadata
    pub search_query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Export format
    pub format: ExportFormat,

    /// What data to include
    pub include_data: Vec<ExportDataType>,

    /// Date range for export
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,

    /// Include charts and visualizations
    pub include_charts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    CSV,
    JSON,
    PDF,
    Excel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportDataType {
    TransactionHistory,
    SigilList,
    PerformanceMetrics,
    ReputationHistory,
    Achievements,
    Summary,
}

impl ContributionLedger {
    pub fn new(keeper_id: String) -> Self {
        Self {
            keeper_id,
            view_config: LedgerViewConfig::default(),
            cached_data: None,
            filters: LedgerFilters::default(),
            export_config: ExportConfig::default(),
        }
    }

    /// Generate ledger data for the current keeper
    pub async fn generate_ledger_data(
        &mut self,
        keeper: &Keeper,
        transactions: &[RewardTransaction],
        hosted_sigils: &[Sigil],
        network_stats: &super::rewards::RewardStatistics,
    ) -> Result<()> {
        let time_range = self.get_time_range();

        // Filter transactions by time range
        let filtered_transactions: Vec<_> = transactions
            .iter()
            .filter(|tx| self.is_in_time_range(tx.timestamp, &time_range))
            .collect();

        // Generate summary
        let summary = self.generate_summary(keeper, &filtered_transactions, hosted_sigils);

        // Enhance transactions
        let enhanced_transactions =
            self.enhance_transactions(&filtered_transactions, hosted_sigils);

        // Generate hosted sigil info
        let hosted_sigil_info =
            self.generate_hosted_sigil_info(hosted_sigils, &filtered_transactions);

        // Generate performance trends
        let trends = self.generate_performance_trends(keeper, &filtered_transactions);

        // Generate rankings
        let rankings = self.generate_network_rankings(keeper, network_stats);

        // Generate achievements
        let achievements = self.generate_achievements(keeper, &filtered_transactions);

        self.cached_data = Some(LedgerData {
            summary,
            transactions: enhanced_transactions,
            hosted_sigils: hosted_sigil_info,
            trends,
            rankings,
            achievements,
        });

        Ok(())
    }

    fn get_time_range(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        match &self.view_config.time_period {
            TimePeriod::Last24Hours => (now - chrono::Duration::hours(24), now),
            TimePeriod::LastWeek => (now - chrono::Duration::weeks(1), now),
            TimePeriod::LastMonth => (now - chrono::Duration::days(30), now),
            TimePeriod::LastYear => (now - chrono::Duration::days(365), now),
            TimePeriod::AllTime => (DateTime::from_timestamp(0, 0).unwrap_or(now), now),
            TimePeriod::Custom { start, end } => (*start, *end),
        }
    }

    fn is_in_time_range(
        &self,
        timestamp: DateTime<Utc>,
        range: &(DateTime<Utc>, DateTime<Utc>),
    ) -> bool {
        timestamp >= range.0 && timestamp <= range.1
    }

    fn generate_summary(
        &self,
        keeper: &Keeper,
        transactions: &[&RewardTransaction],
        hosted_sigils: &[Sigil],
    ) -> LedgerSummary {
        let total_earned = transactions.iter().map(|tx| tx.amount).sum();

        let mut earnings_by_type = HashMap::new();
        for tx in transactions {
            *earnings_by_type.entry(tx.reward_type.clone()).or_insert(0) += tx.amount;
        }

        let total_retrievals_served = transactions
            .iter()
            .filter(|tx| tx.reward_type == RewardType::RetrievalBonus)
            .count() as u32;

        LedgerSummary {
            total_earned,
            earnings_by_type,
            current_reputation: keeper.reputation,
            reputation_change: 0.0, // Would calculate from historical data
            total_sigils_hosted: hosted_sigils.len(),
            new_sigils_this_period: 0, // Would calculate from time range
            total_retrievals_served,
            avg_response_time_ms: keeper.metrics.avg_response_time_ms,
            uptime_percentage: 99.5, // Would calculate from actual uptime data
            storage_utilization: keeper.storage_utilization(),
            contribution_score: self.calculate_contribution_score(keeper, transactions),
        }
    }

    fn calculate_contribution_score(
        &self,
        keeper: &Keeper,
        transactions: &[&RewardTransaction],
    ) -> f64 {
        let base_score = keeper.reputation * 50.0;
        let activity_score = (transactions.len() as f64).min(50.0);
        let uptime_score = if keeper.is_online() { 20.0 } else { 0.0 };

        (base_score + activity_score + uptime_score).min(100.0)
    }

    fn enhance_transactions(
        &self,
        transactions: &[&RewardTransaction],
        hosted_sigils: &[Sigil],
    ) -> Vec<EnhancedRewardTransaction> {
        transactions
            .iter()
            .map(|tx| {
                let sigil_info = tx.sigil_hash.as_ref().and_then(|hash| {
                    hosted_sigils
                        .iter()
                        .find(|sigil| &sigil.content_hash == hash)
                        .map(|sigil| SigilInfo {
                            content_hash: sigil.content_hash.clone(),
                            glyph_symbol: sigil.glyph.symbol().to_string(),
                            glyph_name: sigil.glyph.name(),
                            size_formatted: self.format_bytes(sigil.size),
                            importance_level: format!("{:?}", sigil.glyph.importance),
                        })
                });

                let visual_indicators = self.determine_transaction_indicators(tx);

                EnhancedRewardTransaction {
                    transaction: (*tx).clone(),
                    amount_formatted: self.format_par_amount(tx.amount),
                    timestamp_formatted: tx.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    age_formatted: self.format_duration_since(tx.timestamp),
                    sigil_info,
                    reputation_impact: self.calculate_reputation_impact(tx),
                    visual_indicators,
                }
            })
            .collect()
    }

    fn determine_transaction_indicators(
        &self,
        tx: &RewardTransaction,
    ) -> Vec<TransactionIndicator> {
        let mut indicators = Vec::new();

        // High value transactions
        if tx.amount > 100_00000000 {
            indicators.push(TransactionIndicator::HighValue);
        }

        // Performance bonuses
        if tx.reward_type == RewardType::ProofBonus || tx.reward_type == RewardType::RetrievalBonus
        {
            indicators.push(TransactionIndicator::PerformanceBonus);
        }

        // Penalties
        if tx.reward_type == RewardType::Penalty {
            indicators.push(TransactionIndicator::Penalty);
        }

        indicators
    }

    fn calculate_reputation_impact(&self, tx: &RewardTransaction) -> f64 {
        match tx.reward_type {
            RewardType::ProofBonus => 0.01,
            RewardType::RetrievalBonus => 0.005,
            RewardType::UptimeBonus => 0.002,
            RewardType::Penalty => -0.05,
            _ => 0.0,
        }
    }

    fn generate_hosted_sigil_info(
        &self,
        hosted_sigils: &[Sigil],
        transactions: &[&RewardTransaction],
    ) -> Vec<HostedSigilInfo> {
        hosted_sigils
            .iter()
            .map(|sigil| {
                let sigil_transactions: Vec<_> = transactions
                    .iter()
                    .filter(|tx| tx.sigil_hash.as_ref() == Some(&sigil.content_hash))
                    .collect();

                let total_earned = sigil_transactions.iter().map(|tx| tx.amount).sum();
                let retrievals_served = sigil_transactions
                    .iter()
                    .filter(|tx| tx.reward_type == RewardType::RetrievalBonus)
                    .count() as u32;

                let health_status = if sigil.keepers.len() == 1 {
                    SigilHealthStatus::Orphaned
                } else if sigil.keepers.len() < 3 {
                    SigilHealthStatus::AtRisk
                } else {
                    SigilHealthStatus::Healthy
                };

                HostedSigilInfo {
                    sigil_info: SigilInfo {
                        content_hash: sigil.content_hash.clone(),
                        glyph_symbol: sigil.glyph.symbol().to_string(),
                        glyph_name: sigil.glyph.name(),
                        size_formatted: self.format_bytes(sigil.size),
                        importance_level: format!("{:?}", sigil.glyph.importance),
                    },
                    hosting_since: sigil.created_at, // Simplified - would track when this keeper started hosting
                    retrievals_served,
                    total_earned,
                    last_retrieval: sigil.retrievals.last().map(|r| r.timestamp),
                    avg_response_time_ms: 150.0, // Would calculate from actual data
                    success_rate: 0.98,
                    other_keepers: sigil.keepers.clone(),
                    health_status,
                }
            })
            .collect()
    }

    fn generate_performance_trends(
        &self,
        _keeper: &Keeper,
        _transactions: &[&RewardTransaction],
    ) -> PerformanceTrends {
        // In a real implementation, this would analyze historical data
        // For now, return placeholder trends
        PerformanceTrends {
            daily_earnings: Vec::new(),
            reputation_history: Vec::new(),
            storage_utilization_history: Vec::new(),
            response_time_history: Vec::new(),
            uptime_history: Vec::new(),
        }
    }

    fn generate_network_rankings(
        &self,
        keeper: &Keeper,
        _network_stats: &super::rewards::RewardStatistics,
    ) -> NetworkRankings {
        // Placeholder rankings - would be calculated from actual network data
        let total_keepers = 100; // Mock total

        NetworkRankings {
            earnings_rank: NetworkRank {
                current_rank: 25,
                total_keepers,
                percentile: 75.0,
                rank_change: RankChange::Up(3),
            },
            reputation_rank: NetworkRank {
                current_rank: (keeper.reputation * total_keepers as f64) as usize,
                total_keepers,
                percentile: keeper.reputation * 100.0,
                rank_change: RankChange::Same,
            },
            storage_rank: NetworkRank {
                current_rank: 30,
                total_keepers,
                percentile: 70.0,
                rank_change: RankChange::Down(2),
            },
            uptime_rank: NetworkRank {
                current_rank: 15,
                total_keepers,
                percentile: 85.0,
                rank_change: RankChange::Up(1),
            },
            overall_rank: NetworkRank {
                current_rank: 22,
                total_keepers,
                percentile: 78.0,
                rank_change: RankChange::Up(5),
            },
        }
    }

    fn generate_achievements(
        &self,
        keeper: &Keeper,
        transactions: &[&RewardTransaction],
    ) -> Vec<Achievement> {
        let mut achievements = Vec::new();

        // First earnings achievement
        if !transactions.is_empty() {
            achievements.push(Achievement {
                achievement_id: "first_earnings".to_string(),
                title: "First Rewards".to_string(),
                description: "Earned your first PAR tokens from storage contributions".to_string(),
                icon: "💰".to_string(),
                achieved_at: transactions[0].timestamp,
                reward_amount: Some(1_00000000), // 1 PAR bonus
                rarity: AchievementRarity::Common,
            });
        }

        // Reputation milestones
        if keeper.reputation >= 0.9 {
            achievements.push(Achievement {
                achievement_id: "high_reputation".to_string(),
                title: "Trusted Keeper".to_string(),
                description: "Achieved reputation score of 90% or higher".to_string(),
                icon: "🏆".to_string(),
                achieved_at: Utc::now(),
                reward_amount: Some(50_00000000), // 50 PAR bonus
                rarity: AchievementRarity::Epic,
            });
        }

        // Status achievements
        match keeper.status {
            super::KeeperStatus::Loremaster => {
                achievements.push(Achievement {
                    achievement_id: "loremaster_status".to_string(),
                    title: "🎭 Loremaster Ascension".to_string(),
                    description: "Achieved the legendary Loremaster status".to_string(),
                    icon: "🎭".to_string(),
                    achieved_at: Utc::now(),
                    reward_amount: Some(1000_00000000), // 1000 PAR bonus
                    rarity: AchievementRarity::Legendary,
                });
            }
            super::KeeperStatus::Archivist => {
                achievements.push(Achievement {
                    achievement_id: "archivist_status".to_string(),
                    title: "📚 Archivist Promotion".to_string(),
                    description: "Promoted to Archivist status for exceptional storage service"
                        .to_string(),
                    icon: "📚".to_string(),
                    achieved_at: Utc::now(),
                    reward_amount: Some(200_00000000), // 200 PAR bonus
                    rarity: AchievementRarity::Rare,
                });
            }
            _ => {}
        }

        achievements
    }

    // Utility formatting functions
    fn format_par_amount(&self, amount: Amount) -> String {
        let par = amount as f64 / 1_00000000.0;
        if par >= 1000000.0 {
            format!("{:.2}M PAR", par / 1000000.0)
        } else if par >= 1000.0 {
            format!("{:.2}K PAR", par / 1000.0)
        } else {
            format!("{:.4} PAR", par)
        }
    }

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

    fn format_duration_since(&self, timestamp: DateTime<Utc>) -> String {
        let duration = Utc::now().signed_duration_since(timestamp);

        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else {
            format!("{} minutes ago", duration.num_minutes().max(1))
        }
    }

    /// Export ledger data in the specified format
    pub async fn export_data(&self) -> Result<Vec<u8>> {
        match self.export_config.format {
            ExportFormat::JSON => self.export_json(),
            ExportFormat::CSV => self.export_csv(),
            _ => Err(anyhow::anyhow!("Export format not implemented")),
        }
    }

    fn export_json(&self) -> Result<Vec<u8>> {
        let data = self
            .cached_data
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No data to export"))?;

        let json = serde_json::to_string_pretty(data)?;
        Ok(json.into_bytes())
    }

    fn export_csv(&self) -> Result<Vec<u8>> {
        let data = self
            .cached_data
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No data to export"))?;

        let mut csv = String::new();
        csv.push_str("Date,Type,Amount,Sigil,Description\n");

        for tx in &data.transactions {
            csv.push_str(&format!(
                "{},{:?},{},{},{}\n",
                tx.timestamp_formatted,
                tx.transaction.reward_type,
                tx.amount_formatted,
                tx.sigil_info
                    .as_ref()
                    .map(|s| s.content_hash.as_str())
                    .unwrap_or(""),
                tx.transaction
                    .metadata
                    .get("description")
                    .unwrap_or(&String::new())
            ));
        }

        Ok(csv.into_bytes())
    }
}

// Default implementations

impl Default for LedgerViewConfig {
    fn default() -> Self {
        Self {
            time_period: TimePeriod::LastMonth,
            highlighted_metrics: vec![MetricType::TotalEarnings, MetricType::Reputation],
            chart_config: ChartConfig::default(),
            pagination: PaginationConfig::default(),
        }
    }
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            chart_types: vec![ChartType::EarningsTimeline, ChartType::RewardBreakdown],
            color_scheme: ChartColorScheme::Mythic,
            animations_enabled: true,
        }
    }
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            items_per_page: 20,
            current_page: 0,
            show_page_size_options: true,
        }
    }
}

impl Default for LedgerFilters {
    fn default() -> Self {
        Self {
            reward_types: Vec::new(),
            amount_range: None,
            sigil_hashes: Vec::new(),
            min_amount_threshold: None,
            search_query: String::new(),
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::JSON,
            include_data: vec![
                ExportDataType::TransactionHistory,
                ExportDataType::SigilList,
                ExportDataType::Summary,
            ],
            date_range: None,
            include_charts: false,
        }
    }
}
