use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
/// Real-time Network Analytics Dashboard
/// Provides comprehensive monitoring, metrics collection, and visualization
/// for the Paradigm tokenomics network
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{ContributionType, EconomicParameters, NetworkState};
use crate::{Address, ParadigmError};

pub type Result<T> = std::result::Result<T, ParadigmError>;

/// Main network analytics dashboard
#[derive(Debug)]
pub struct NetworkAnalyticsDashboard {
    /// Real-time metrics collector
    metrics_collector: MetricsCollector,
    /// Performance monitor
    performance_monitor: PerformanceMonitor,
    /// Economic health tracker
    economic_tracker: EconomicHealthTracker,
    /// Alert system
    alert_system: AlertSystem,
    /// Visualization engine
    visualization_engine: VisualizationEngine,
    /// Data retention manager
    data_retention: DataRetentionManager,
}

impl NetworkAnalyticsDashboard {
    pub fn new() -> Self {
        Self {
            metrics_collector: MetricsCollector::new(),
            performance_monitor: PerformanceMonitor::new(),
            economic_tracker: EconomicHealthTracker::new(),
            alert_system: AlertSystem::new(),
            visualization_engine: VisualizationEngine::new(),
            data_retention: DataRetentionManager::new(),
        }
    }

    /// Initialize the analytics dashboard
    pub async fn initialize(&mut self) -> Result<()> {
        self.metrics_collector.initialize().await?;
        self.performance_monitor.initialize().await?;
        self.economic_tracker.initialize().await?;
        self.alert_system.initialize().await?;
        self.visualization_engine.initialize().await?;
        self.data_retention.initialize().await?;

        println!("Network Analytics Dashboard initialized successfully");
        Ok(())
    }

    /// Update dashboard with new network state
    pub async fn update_network_state(&mut self, network_state: &NetworkState) -> Result<()> {
        // Collect metrics
        self.metrics_collector
            .collect_network_metrics(network_state)
            .await?;

        // Monitor performance
        self.performance_monitor
            .update_performance_metrics(network_state)
            .await?;

        // Track economic health
        self.economic_tracker
            .analyze_economic_health(network_state)
            .await?;

        // Check for alerts
        self.alert_system.check_thresholds(network_state).await?;

        // Update visualizations
        self.visualization_engine
            .update_charts(network_state)
            .await?;

        // Manage data retention
        self.data_retention.manage_historical_data().await?;

        Ok(())
    }

    /// Record contribution event
    pub async fn record_contribution(
        &mut self,
        contributor: &Address,
        contribution_type: ContributionType,
        value: u64,
    ) -> Result<()> {
        let event = ContributionEvent {
            id: Uuid::new_v4(),
            contributor: contributor.clone(),
            contribution_type,
            value,
            timestamp: Utc::now(),
        };

        self.metrics_collector
            .record_contribution_event(event)
            .await?;
        Ok(())
    }

    /// Get real-time dashboard data
    pub async fn get_dashboard_data(&self) -> Result<DashboardData> {
        let current_metrics = self.metrics_collector.get_current_metrics().await?;
        let performance_data = self.performance_monitor.get_performance_summary().await?;
        let economic_health = self.economic_tracker.get_health_indicators().await?;
        let active_alerts = self.alert_system.get_active_alerts().await?;
        let chart_data = self.visualization_engine.get_chart_data().await?;

        Ok(DashboardData {
            timestamp: Utc::now(),
            metrics: current_metrics,
            performance: performance_data,
            economic_health,
            alerts: active_alerts,
            charts: chart_data,
        })
    }

    /// Generate analytics report
    pub async fn generate_report(&self, timeframe: TimeFrame) -> Result<AnalyticsReport> {
        let historical_data = self
            .data_retention
            .get_historical_data(timeframe.clone())
            .await?;
        let trends = self.analyze_trends(&historical_data).await?;
        let insights = self.generate_insights(&trends).await?;

        Ok(AnalyticsReport {
            timeframe,
            generated_at: Utc::now(),
            summary: self.generate_summary(&historical_data).await?,
            trends,
            insights: insights.clone(),
            recommendations: self.generate_recommendations(&insights).await?,
        })
    }

    async fn analyze_trends(&self, data: &[HistoricalDataPoint]) -> Result<TrendAnalysis> {
        // Implement trend analysis logic
        let mut token_supply_trend = Vec::new();
        let mut participation_trend = Vec::new();
        let mut performance_trend = Vec::new();

        for point in data {
            token_supply_trend.push((point.timestamp, point.metrics.total_supply as f64));
            participation_trend.push((point.timestamp, point.metrics.active_participants as f64));
            performance_trend.push((point.timestamp, point.performance.throughput));
        }

        Ok(TrendAnalysis {
            token_supply_trend: self.calculate_trend_direction(&token_supply_trend),
            participation_trend: self.calculate_trend_direction(&participation_trend),
            performance_trend: self.calculate_trend_direction(&performance_trend),
            volatility_index: self.calculate_volatility(&token_supply_trend),
        })
    }

    fn calculate_trend_direction(&self, data: &[(DateTime<Utc>, f64)]) -> TrendDirection {
        if data.len() < 2 {
            return TrendDirection::Stable;
        }

        let start_value = data.first().unwrap().1;
        let end_value = data.last().unwrap().1;
        let change_percent = ((end_value - start_value) / start_value) * 100.0;

        if change_percent > 5.0 {
            TrendDirection::Increasing
        } else if change_percent < -5.0 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    fn calculate_volatility(&self, data: &[(DateTime<Utc>, f64)]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let values: Vec<f64> = data.iter().map(|(_, v)| *v).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

        variance.sqrt() / mean
    }

    async fn generate_insights(&self, trends: &TrendAnalysis) -> Result<Vec<NetworkInsight>> {
        let mut insights = Vec::new();

        // Token supply insights
        match trends.token_supply_trend {
            TrendDirection::Increasing => {
                insights.push(NetworkInsight {
                    category: InsightCategory::Tokenomics,
                    severity: InsightSeverity::Info,
                    message: "Token supply is increasing, indicating network growth".to_string(),
                    recommendation: "Monitor inflation rate to ensure sustainability".to_string(),
                });
            }
            TrendDirection::Decreasing => {
                insights.push(NetworkInsight {
                    category: InsightCategory::Tokenomics,
                    severity: InsightSeverity::Warning,
                    message: "Token supply is decreasing, potential deflationary pressure"
                        .to_string(),
                    recommendation: "Consider adjusting burn rate or increasing rewards"
                        .to_string(),
                });
            }
            _ => {}
        }

        // Volatility insights
        if trends.volatility_index > 0.2 {
            insights.push(NetworkInsight {
                category: InsightCategory::Stability,
                severity: InsightSeverity::Warning,
                message: format!(
                    "High volatility detected: {:.2}%",
                    trends.volatility_index * 100.0
                ),
                recommendation: "Consider implementing stability mechanisms".to_string(),
            });
        }

        Ok(insights)
    }

    async fn generate_summary(&self, data: &[HistoricalDataPoint]) -> Result<ReportSummary> {
        if data.is_empty() {
            return Ok(ReportSummary::default());
        }

        let latest = data.last().unwrap();
        let earliest = data.first().unwrap();

        Ok(ReportSummary {
            period_start: earliest.timestamp,
            period_end: latest.timestamp,
            total_data_points: data.len(),
            average_participants: data
                .iter()
                .map(|d| d.metrics.active_participants as f64)
                .sum::<f64>()
                / data.len() as f64,
            average_throughput: data.iter().map(|d| d.performance.throughput).sum::<f64>()
                / data.len() as f64,
            total_contributions: data
                .iter()
                .map(|d| d.metrics.total_contributions)
                .sum::<u64>(),
        })
    }

    async fn generate_recommendations(&self, insights: &[NetworkInsight]) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        let warning_count = insights
            .iter()
            .filter(|i| {
                matches!(
                    i.severity,
                    InsightSeverity::Warning | InsightSeverity::Critical
                )
            })
            .count();

        if warning_count > 0 {
            recommendations.push(
                "Review network health indicators and consider parameter adjustments".to_string(),
            );
        }

        let tokenomics_issues = insights
            .iter()
            .filter(|i| matches!(i.category, InsightCategory::Tokenomics))
            .count();

        if tokenomics_issues > 0 {
            recommendations
                .push("Analyze tokenomics parameters and consider optimization".to_string());
        }

        recommendations.push("Continue monitoring network metrics for emerging trends".to_string());

        Ok(recommendations)
    }
}

/// Metrics collection system
#[derive(Debug)]
pub struct MetricsCollector {
    current_metrics: Arc<RwLock<NetworkMetrics>>,
    contribution_events: Arc<RwLock<VecDeque<ContributionEvent>>>,
    metric_history: Arc<RwLock<VecDeque<TimestampedMetrics>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            current_metrics: Arc::new(RwLock::new(NetworkMetrics::default())),
            contribution_events: Arc::new(RwLock::new(VecDeque::new())),
            metric_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Initialize metrics collection
        println!("Metrics collector initialized");
        Ok(())
    }

    pub async fn collect_network_metrics(&mut self, network_state: &NetworkState) -> Result<()> {
        let metrics = NetworkMetrics {
            total_supply: network_state.total_supply,
            active_participants: network_state.active_participants,
            transaction_volume: network_state.transaction_volume,
            transaction_throughput: network_state.transaction_throughput,
            uptime_percentage: network_state.uptime_percentage,
            avg_consensus_time: network_state.avg_consensus_time,
            error_rate: network_state.error_rate,
            resource_utilization: network_state.resource_utilization,
            governance_participation_rate: network_state.governance_participation_rate,
            total_contributions: 0, // This would be calculated from contribution events
            timestamp: Utc::now(),
        };

        // Update current metrics
        {
            let mut current = self.current_metrics.write().await;
            *current = metrics.clone();
        }

        // Add to history
        {
            let mut history = self.metric_history.write().await;
            history.push_back(TimestampedMetrics {
                metrics,
                timestamp: Utc::now(),
            });

            // Keep last 1000 entries
            if history.len() > 1000 {
                history.pop_front();
            }
        }

        Ok(())
    }

    pub async fn record_contribution_event(&mut self, event: ContributionEvent) -> Result<()> {
        let mut events = self.contribution_events.write().await;
        events.push_back(event);

        // Keep last 500 events
        if events.len() > 500 {
            events.pop_front();
        }

        Ok(())
    }

    pub async fn get_current_metrics(&self) -> Result<NetworkMetrics> {
        let metrics = self.current_metrics.read().await;
        Ok(metrics.clone())
    }
}

/// Performance monitoring system
#[derive(Debug)]
pub struct PerformanceMonitor {
    performance_data: Arc<RwLock<PerformanceData>>,
    latency_history: Arc<RwLock<VecDeque<f64>>>,
    throughput_history: Arc<RwLock<VecDeque<f64>>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            performance_data: Arc::new(RwLock::new(PerformanceData::default())),
            latency_history: Arc::new(RwLock::new(VecDeque::new())),
            throughput_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        println!("Performance monitor initialized");
        Ok(())
    }

    pub async fn update_performance_metrics(&mut self, network_state: &NetworkState) -> Result<()> {
        let performance = PerformanceData {
            throughput: network_state.transaction_throughput,
            latency: network_state.avg_consensus_time,
            error_rate: network_state.error_rate,
            uptime: network_state.uptime_percentage,
            resource_utilization: network_state.resource_utilization,
            timestamp: Utc::now(),
        };

        // Update current performance
        {
            let mut current = self.performance_data.write().await;
            *current = performance.clone();
        }

        // Update histories
        {
            let mut latency_hist = self.latency_history.write().await;
            latency_hist.push_back(performance.latency);
            if latency_hist.len() > 100 {
                latency_hist.pop_front();
            }
        }

        {
            let mut throughput_hist = self.throughput_history.write().await;
            throughput_hist.push_back(performance.throughput);
            if throughput_hist.len() > 100 {
                throughput_hist.pop_front();
            }
        }

        Ok(())
    }

    pub async fn get_performance_summary(&self) -> Result<PerformanceData> {
        let data = self.performance_data.read().await;
        Ok(data.clone())
    }
}

/// Economic health tracking system
#[derive(Debug)]
pub struct EconomicHealthTracker {
    health_indicators: Arc<RwLock<EconomicHealthIndicators>>,
    health_history: Arc<RwLock<VecDeque<EconomicHealthIndicators>>>,
}

impl EconomicHealthTracker {
    pub fn new() -> Self {
        Self {
            health_indicators: Arc::new(RwLock::new(EconomicHealthIndicators::default())),
            health_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        println!("Economic health tracker initialized");
        Ok(())
    }

    pub async fn analyze_economic_health(&mut self, network_state: &NetworkState) -> Result<()> {
        let indicators = EconomicHealthIndicators {
            token_velocity: network_state.token_velocity,
            inflation_rate: network_state.inflation_rate,
            wealth_concentration: network_state.wealth_concentration_gini,
            participation_rate: network_state.governance_participation_rate,
            treasury_health: self.calculate_treasury_health(network_state),
            liquidity_score: self.calculate_liquidity_score(network_state),
            stability_index: self.calculate_stability_index(network_state),
            timestamp: Utc::now(),
        };

        // Update current indicators
        {
            let mut current = self.health_indicators.write().await;
            *current = indicators.clone();
        }

        // Add to history
        {
            let mut history = self.health_history.write().await;
            history.push_back(indicators);
            if history.len() > 200 {
                history.pop_front();
            }
        }

        Ok(())
    }

    fn calculate_treasury_health(&self, network_state: &NetworkState) -> f64 {
        let treasury_ratio =
            network_state.treasury_balance as f64 / network_state.total_supply as f64;
        // Healthy treasury should be 5-15% of total supply
        if treasury_ratio >= 0.05 && treasury_ratio <= 0.15 {
            1.0
        } else if treasury_ratio < 0.05 {
            treasury_ratio / 0.05 // Scale down if too low
        } else {
            0.15 / treasury_ratio // Scale down if too high
        }
    }

    fn calculate_liquidity_score(&self, network_state: &NetworkState) -> f64 {
        // Higher token velocity indicates better liquidity
        // Normal velocity range is 2-6
        let velocity = network_state.token_velocity;
        if velocity >= 2.0 && velocity <= 6.0 {
            1.0
        } else if velocity < 2.0 {
            velocity / 2.0
        } else {
            6.0 / velocity
        }
    }

    fn calculate_stability_index(&self, network_state: &NetworkState) -> f64 {
        // Combine multiple factors for stability
        let inflation_stability = if network_state.inflation_rate <= 0.1 {
            1.0
        } else {
            0.1 / network_state.inflation_rate
        };
        let error_stability = 1.0 - network_state.error_rate;
        let uptime_stability = network_state.uptime_percentage;

        (inflation_stability + error_stability + uptime_stability) / 3.0
    }

    pub async fn get_health_indicators(&self) -> Result<EconomicHealthIndicators> {
        let indicators = self.health_indicators.read().await;
        Ok(indicators.clone())
    }
}

/// Alert system for threshold monitoring
#[derive(Debug)]
pub struct AlertSystem {
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    active_alerts: Arc<RwLock<Vec<Alert>>>,
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
}

impl AlertSystem {
    pub fn new() -> Self {
        Self {
            alert_rules: Arc::new(RwLock::new(Vec::new())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Set up default alert rules
        let default_rules = vec![
            AlertRule {
                id: Uuid::new_v4(),
                name: "High Error Rate".to_string(),
                condition: AlertCondition::ErrorRateExceeds(0.05),
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                id: Uuid::new_v4(),
                name: "Low Uptime".to_string(),
                condition: AlertCondition::UptimeBelow(0.95),
                severity: AlertSeverity::Critical,
                enabled: true,
            },
            AlertRule {
                id: Uuid::new_v4(),
                name: "High Inflation".to_string(),
                condition: AlertCondition::InflationExceeds(0.15),
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                id: Uuid::new_v4(),
                name: "Low Participation".to_string(),
                condition: AlertCondition::ParticipationBelow(0.1),
                severity: AlertSeverity::Info,
                enabled: true,
            },
        ];

        {
            let mut rules = self.alert_rules.write().await;
            *rules = default_rules;
        }

        println!("Alert system initialized with default rules");
        Ok(())
    }

    pub async fn check_thresholds(&mut self, network_state: &NetworkState) -> Result<()> {
        let rules = self.alert_rules.read().await;
        let mut new_alerts = Vec::new();

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            let triggered = match &rule.condition {
                AlertCondition::ErrorRateExceeds(threshold) => {
                    network_state.error_rate > *threshold
                }
                AlertCondition::UptimeBelow(threshold) => {
                    network_state.uptime_percentage < *threshold
                }
                AlertCondition::InflationExceeds(threshold) => {
                    network_state.inflation_rate > *threshold
                }
                AlertCondition::ParticipationBelow(threshold) => {
                    network_state.governance_participation_rate < *threshold
                }
                AlertCondition::ThroughputBelow(threshold) => {
                    network_state.transaction_throughput < *threshold
                }
            };

            if triggered {
                let alert = Alert {
                    id: Uuid::new_v4(),
                    rule_id: rule.id,
                    message: format!("Alert: {}", rule.name),
                    severity: rule.severity.clone(),
                    timestamp: Utc::now(),
                    acknowledged: false,
                };

                new_alerts.push(alert);
            }
        }

        if !new_alerts.is_empty() {
            // Add to active alerts
            {
                let mut active = self.active_alerts.write().await;
                active.extend(new_alerts.clone());
            }

            // Add to history
            {
                let mut history = self.alert_history.write().await;
                for alert in new_alerts {
                    history.push_back(alert);
                }

                // Keep last 1000 alerts
                while history.len() > 1000 {
                    history.pop_front();
                }
            }
        }

        Ok(())
    }

    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        let alerts = self.active_alerts.read().await;
        Ok(alerts.clone())
    }

    pub async fn acknowledge_alert(&mut self, alert_id: Uuid) -> Result<()> {
        let mut active = self.active_alerts.write().await;
        if let Some(alert) = active.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
        }
        Ok(())
    }
}

/// Visualization engine for charts and graphs
#[derive(Debug)]
pub struct VisualizationEngine {
    chart_data: Arc<RwLock<ChartDataCollection>>,
}

impl VisualizationEngine {
    pub fn new() -> Self {
        Self {
            chart_data: Arc::new(RwLock::new(ChartDataCollection::default())),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        println!("Visualization engine initialized");
        Ok(())
    }

    pub async fn update_charts(&mut self, network_state: &NetworkState) -> Result<()> {
        let timestamp = Utc::now();

        let mut charts = self.chart_data.write().await;

        // Update time series charts
        charts
            .token_supply_chart
            .push((timestamp, network_state.total_supply as f64));
        charts
            .participation_chart
            .push((timestamp, network_state.active_participants as f64));
        charts
            .throughput_chart
            .push((timestamp, network_state.transaction_throughput));
        charts
            .uptime_chart
            .push((timestamp, network_state.uptime_percentage * 100.0));

        // Keep last 100 data points for each chart
        if charts.token_supply_chart.len() > 100 {
            charts.token_supply_chart.remove(0);
        }
        if charts.participation_chart.len() > 100 {
            charts.participation_chart.remove(0);
        }
        if charts.throughput_chart.len() > 100 {
            charts.throughput_chart.remove(0);
        }
        if charts.uptime_chart.len() > 100 {
            charts.uptime_chart.remove(0);
        }

        Ok(())
    }

    pub async fn get_chart_data(&self) -> Result<ChartDataCollection> {
        let data = self.chart_data.read().await;
        Ok(data.clone())
    }
}

/// Data retention management
#[derive(Debug)]
pub struct DataRetentionManager {
    historical_data: Arc<RwLock<VecDeque<HistoricalDataPoint>>>,
    retention_policy: RetentionPolicy,
}

impl DataRetentionManager {
    pub fn new() -> Self {
        Self {
            historical_data: Arc::new(RwLock::new(VecDeque::new())),
            retention_policy: RetentionPolicy::default(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        println!("Data retention manager initialized");
        Ok(())
    }

    pub async fn manage_historical_data(&mut self) -> Result<()> {
        let mut data = self.historical_data.write().await;

        // Remove data older than retention period
        let cutoff_time = Utc::now() - Duration::days(self.retention_policy.days_to_keep as i64);

        while let Some(front) = data.front() {
            if front.timestamp < cutoff_time {
                data.pop_front();
            } else {
                break;
            }
        }

        Ok(())
    }

    pub async fn get_historical_data(
        &self,
        timeframe: TimeFrame,
    ) -> Result<Vec<HistoricalDataPoint>> {
        let data = self.historical_data.read().await;
        let cutoff_time = match timeframe {
            TimeFrame::LastHour => Utc::now() - Duration::hours(1),
            TimeFrame::LastDay => Utc::now() - Duration::days(1),
            TimeFrame::LastWeek => Utc::now() - Duration::weeks(1),
            TimeFrame::LastMonth => Utc::now() - Duration::days(30),
        };

        let filtered_data: Vec<HistoricalDataPoint> = data
            .iter()
            .filter(|point| point.timestamp >= cutoff_time)
            .cloned()
            .collect();

        Ok(filtered_data)
    }
}

// Data structures for the analytics system

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub total_supply: u64,
    pub active_participants: u64,
    pub transaction_volume: u64,
    pub transaction_throughput: f64,
    pub uptime_percentage: f64,
    pub avg_consensus_time: f64,
    pub error_rate: f64,
    pub resource_utilization: f64,
    pub governance_participation_rate: f64,
    pub total_contributions: u64,
    pub timestamp: DateTime<Utc>,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            total_supply: 0,
            active_participants: 0,
            transaction_volume: 0,
            transaction_throughput: 0.0,
            uptime_percentage: 1.0,
            avg_consensus_time: 0.0,
            error_rate: 0.0,
            resource_utilization: 0.0,
            governance_participation_rate: 0.0,
            total_contributions: 0,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionEvent {
    pub id: Uuid,
    pub contributor: Address,
    pub contribution_type: ContributionType,
    pub value: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedMetrics {
    pub metrics: NetworkMetrics,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceData {
    pub throughput: f64,
    pub latency: f64,
    pub error_rate: f64,
    pub uptime: f64,
    pub resource_utilization: f64,
    pub timestamp: DateTime<Utc>,
}

impl Default for PerformanceData {
    fn default() -> Self {
        Self {
            throughput: 0.0,
            latency: 0.0,
            error_rate: 0.0,
            uptime: 1.0,
            resource_utilization: 0.0,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicHealthIndicators {
    pub token_velocity: f64,
    pub inflation_rate: f64,
    pub wealth_concentration: f64,
    pub participation_rate: f64,
    pub treasury_health: f64,
    pub liquidity_score: f64,
    pub stability_index: f64,
    pub timestamp: DateTime<Utc>,
}

impl Default for EconomicHealthIndicators {
    fn default() -> Self {
        Self {
            token_velocity: 3.0,
            inflation_rate: 0.05,
            wealth_concentration: 0.3,
            participation_rate: 0.2,
            treasury_health: 1.0,
            liquidity_score: 1.0,
            stability_index: 1.0,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    ErrorRateExceeds(f64),
    UptimeBelow(f64),
    InflationExceeds(f64),
    ParticipationBelow(f64),
    ThroughputBelow(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataCollection {
    pub token_supply_chart: Vec<(DateTime<Utc>, f64)>,
    pub participation_chart: Vec<(DateTime<Utc>, f64)>,
    pub throughput_chart: Vec<(DateTime<Utc>, f64)>,
    pub uptime_chart: Vec<(DateTime<Utc>, f64)>,
}

impl Default for ChartDataCollection {
    fn default() -> Self {
        Self {
            token_supply_chart: Vec::new(),
            participation_chart: Vec::new(),
            throughput_chart: Vec::new(),
            uptime_chart: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub timestamp: DateTime<Utc>,
    pub metrics: NetworkMetrics,
    pub performance: PerformanceData,
    pub economic_health: EconomicHealthIndicators,
    pub alerts: Vec<Alert>,
    pub charts: ChartDataCollection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeFrame {
    LastHour,
    LastDay,
    LastWeek,
    LastMonth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub timeframe: TimeFrame,
    pub generated_at: DateTime<Utc>,
    pub summary: ReportSummary,
    pub trends: TrendAnalysis,
    pub insights: Vec<NetworkInsight>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_data_points: usize,
    pub average_participants: f64,
    pub average_throughput: f64,
    pub total_contributions: u64,
}

impl Default for ReportSummary {
    fn default() -> Self {
        Self {
            period_start: Utc::now(),
            period_end: Utc::now(),
            total_data_points: 0,
            average_participants: 0.0,
            average_throughput: 0.0,
            total_contributions: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub token_supply_trend: TrendDirection,
    pub participation_trend: TrendDirection,
    pub performance_trend: TrendDirection,
    pub volatility_index: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInsight {
    pub category: InsightCategory,
    pub severity: InsightSeverity,
    pub message: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightCategory {
    Tokenomics,
    Performance,
    Governance,
    Security,
    Stability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct HistoricalDataPoint {
    pub timestamp: DateTime<Utc>,
    pub metrics: NetworkMetrics,
    pub performance: PerformanceData,
    pub economic_health: EconomicHealthIndicators,
}

#[derive(Debug)]
pub struct RetentionPolicy {
    pub days_to_keep: u32,
    pub max_data_points: usize,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            days_to_keep: 30,       // Keep 30 days of data
            max_data_points: 10000, // Maximum 10k data points
        }
    }
}
