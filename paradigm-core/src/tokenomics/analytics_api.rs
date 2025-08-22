use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
/// Web API interface for the Network Analytics Dashboard
/// Provides REST endpoints for accessing real-time network metrics,
/// historical data, and analytical insights
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{AlertSeverity, AnalyticsReport, DashboardData, TimeFrame, TokenomicsSystem};
use crate::{Address, ParadigmError};

pub type Result<T> = std::result::Result<T, ParadigmError>;

/// Analytics API server
#[derive(Debug)]
pub struct AnalyticsAPI {
    tokenomics_system: Arc<RwLock<TokenomicsSystem>>,
    api_config: APIConfig,
}

impl AnalyticsAPI {
    pub fn new(tokenomics_system: Arc<RwLock<TokenomicsSystem>>) -> Self {
        Self {
            tokenomics_system,
            api_config: APIConfig::default(),
        }
    }

    /// Initialize the analytics API server
    pub async fn initialize(&mut self) -> Result<()> {
        println!(
            "Analytics API server initializing on port {}",
            self.api_config.port
        );
        // In a real implementation, this would start the web server
        println!("Analytics API server initialized successfully");
        Ok(())
    }

    /// Start the API server
    pub async fn start_server(&self) -> Result<()> {
        println!("Starting Analytics API server...");
        // In a real implementation, this would start the HTTP server
        // For now, we'll just demonstrate the API endpoints
        self.print_available_endpoints();
        Ok(())
    }

    fn print_available_endpoints(&self) {
        println!("\n=== Available Analytics API Endpoints ===");
        println!("GET  /api/v1/dashboard              - Real-time dashboard data");
        println!("GET  /api/v1/metrics/current        - Current network metrics");
        println!("GET  /api/v1/metrics/historical     - Historical metrics data");
        println!("GET  /api/v1/performance             - Performance metrics");
        println!("GET  /api/v1/health                  - Economic health indicators");
        println!("GET  /api/v1/alerts                  - Active alerts");
        println!("GET  /api/v1/charts/token-supply     - Token supply chart data");
        println!("GET  /api/v1/charts/participation    - Participation chart data");
        println!("GET  /api/v1/charts/throughput       - Throughput chart data");
        println!("GET  /api/v1/reports/generate        - Generate analytics report");
        println!("POST /api/v1/alerts/acknowledge      - Acknowledge an alert");
        println!("GET  /api/v1/status                  - API health status");
        println!("==========================================\n");
    }

    /// Get real-time dashboard data
    /// GET /api/v1/dashboard
    pub async fn get_dashboard(&self) -> Result<APIResponse<DashboardData>> {
        let system = self.tokenomics_system.read().await;
        match system.get_dashboard_data().await {
            Ok(data) => Ok(APIResponse::success(data)),
            Err(e) => Ok(APIResponse::error(format!(
                "Failed to get dashboard data: {}",
                e
            ))),
        }
    }

    /// Get current network metrics
    /// GET /api/v1/metrics/current
    pub async fn get_current_metrics(&self) -> Result<APIResponse<NetworkMetricsResponse>> {
        let mut system = self.tokenomics_system.write().await;

        // Update analytics first
        if let Err(e) = system.update_network_analytics().await {
            return Ok(APIResponse::error(format!(
                "Failed to update analytics: {}",
                e
            )));
        }

        match system.get_dashboard_data().await {
            Ok(data) => {
                let response = NetworkMetricsResponse {
                    timestamp: data.timestamp,
                    total_supply: data.metrics.total_supply,
                    active_participants: data.metrics.active_participants,
                    transaction_throughput: data.metrics.transaction_throughput,
                    uptime_percentage: data.metrics.uptime_percentage,
                    error_rate: data.metrics.error_rate,
                    governance_participation_rate: data.metrics.governance_participation_rate,
                };
                Ok(APIResponse::success(response))
            }
            Err(e) => Ok(APIResponse::error(format!("Failed to get metrics: {}", e))),
        }
    }

    /// Get historical metrics data
    /// GET /api/v1/metrics/historical?timeframe=last_day
    pub async fn get_historical_metrics(
        &self,
        timeframe: TimeFrame,
    ) -> Result<APIResponse<HistoricalMetricsResponse>> {
        let system = self.tokenomics_system.read().await;
        match system.generate_analytics_report(timeframe).await {
            Ok(report) => {
                let response = HistoricalMetricsResponse {
                    timeframe: report.timeframe,
                    period_start: report.summary.period_start,
                    period_end: report.summary.period_end,
                    total_data_points: report.summary.total_data_points,
                    average_participants: report.summary.average_participants,
                    average_throughput: report.summary.average_throughput,
                    trends: HistoricalTrends {
                        token_supply_trend: report.trends.token_supply_trend,
                        participation_trend: report.trends.participation_trend,
                        performance_trend: report.trends.performance_trend,
                        volatility_index: report.trends.volatility_index,
                    },
                };
                Ok(APIResponse::success(response))
            }
            Err(e) => Ok(APIResponse::error(format!(
                "Failed to get historical data: {}",
                e
            ))),
        }
    }

    /// Get performance metrics
    /// GET /api/v1/performance
    pub async fn get_performance_metrics(&self) -> Result<APIResponse<PerformanceMetricsResponse>> {
        let system = self.tokenomics_system.read().await;
        match system.get_dashboard_data().await {
            Ok(data) => {
                let response = PerformanceMetricsResponse {
                    timestamp: data.performance.timestamp,
                    throughput: data.performance.throughput,
                    latency: data.performance.latency,
                    error_rate: data.performance.error_rate,
                    uptime: data.performance.uptime,
                    resource_utilization: data.performance.resource_utilization,
                };
                Ok(APIResponse::success(response))
            }
            Err(e) => Ok(APIResponse::error(format!(
                "Failed to get performance metrics: {}",
                e
            ))),
        }
    }

    /// Get economic health indicators
    /// GET /api/v1/health
    pub async fn get_health_indicators(&self) -> Result<APIResponse<HealthIndicatorsResponse>> {
        let system = self.tokenomics_system.read().await;
        match system.get_dashboard_data().await {
            Ok(data) => {
                let response = HealthIndicatorsResponse {
                    timestamp: data.economic_health.timestamp,
                    token_velocity: data.economic_health.token_velocity,
                    inflation_rate: data.economic_health.inflation_rate,
                    wealth_concentration: data.economic_health.wealth_concentration,
                    participation_rate: data.economic_health.participation_rate,
                    treasury_health: data.economic_health.treasury_health,
                    liquidity_score: data.economic_health.liquidity_score,
                    stability_index: data.economic_health.stability_index,
                    overall_health_score: self.calculate_overall_health(&data.economic_health),
                };
                Ok(APIResponse::success(response))
            }
            Err(e) => Ok(APIResponse::error(format!(
                "Failed to get health indicators: {}",
                e
            ))),
        }
    }

    /// Get active alerts
    /// GET /api/v1/alerts
    pub async fn get_active_alerts(&self) -> Result<APIResponse<AlertsResponse>> {
        let system = self.tokenomics_system.read().await;
        match system.get_dashboard_data().await {
            Ok(data) => {
                let critical_count = data
                    .alerts
                    .iter()
                    .filter(|a| matches!(a.severity, AlertSeverity::Critical))
                    .count();
                let warning_count = data
                    .alerts
                    .iter()
                    .filter(|a| matches!(a.severity, AlertSeverity::Warning))
                    .count();
                let info_count = data
                    .alerts
                    .iter()
                    .filter(|a| matches!(a.severity, AlertSeverity::Info))
                    .count();

                let response = AlertsResponse {
                    total_alerts: data.alerts.len(),
                    critical_alerts: critical_count,
                    warning_alerts: warning_count,
                    info_alerts: info_count,
                    alerts: data
                        .alerts
                        .into_iter()
                        .map(|alert| AlertSummary {
                            id: alert.id,
                            message: alert.message,
                            severity: alert.severity,
                            timestamp: alert.timestamp,
                            acknowledged: alert.acknowledged,
                        })
                        .collect(),
                };
                Ok(APIResponse::success(response))
            }
            Err(e) => Ok(APIResponse::error(format!("Failed to get alerts: {}", e))),
        }
    }

    /// Generate analytics report
    /// GET /api/v1/reports/generate?timeframe=last_week
    pub async fn generate_report(
        &self,
        timeframe: TimeFrame,
    ) -> Result<APIResponse<AnalyticsReportResponse>> {
        let system = self.tokenomics_system.read().await;
        match system.generate_analytics_report(timeframe).await {
            Ok(report) => {
                let response = AnalyticsReportResponse {
                    report_id: Uuid::new_v4(),
                    timeframe: report.timeframe,
                    generated_at: report.generated_at,
                    summary: ReportSummaryResponse {
                        period_start: report.summary.period_start,
                        period_end: report.summary.period_end,
                        total_data_points: report.summary.total_data_points,
                        average_participants: report.summary.average_participants,
                        average_throughput: report.summary.average_throughput,
                        total_contributions: report.summary.total_contributions,
                    },
                    insights_count: report.insights.len(),
                    top_insights: report
                        .insights
                        .into_iter()
                        .take(5)
                        .map(|insight| InsightSummary {
                            category: insight.category,
                            severity: insight.severity,
                            message: insight.message,
                            recommendation: insight.recommendation,
                        })
                        .collect(),
                    recommendations: report.recommendations,
                };
                Ok(APIResponse::success(response))
            }
            Err(e) => Ok(APIResponse::error(format!(
                "Failed to generate report: {}",
                e
            ))),
        }
    }

    /// Get token supply chart data
    /// GET /api/v1/charts/token-supply
    pub async fn get_token_supply_chart(&self) -> Result<APIResponse<ChartDataResponse>> {
        let system = self.tokenomics_system.read().await;
        match system.get_dashboard_data().await {
            Ok(data) => {
                let chart_data: Vec<ChartPoint> = data
                    .charts
                    .token_supply_chart
                    .into_iter()
                    .map(|(timestamp, value)| ChartPoint { timestamp, value })
                    .collect();

                let response = ChartDataResponse {
                    chart_type: "token_supply".to_string(),
                    title: "Token Supply Over Time".to_string(),
                    data_points: chart_data.len(),
                    data: chart_data,
                };
                Ok(APIResponse::success(response))
            }
            Err(e) => Ok(APIResponse::error(format!(
                "Failed to get chart data: {}",
                e
            ))),
        }
    }

    /// Acknowledge an alert
    /// POST /api/v1/alerts/acknowledge
    pub async fn acknowledge_alert(
        &self,
        request: AcknowledgeAlertRequest,
    ) -> Result<APIResponse<()>> {
        // In a real implementation, this would call the alert system
        println!(
            "Alert {} acknowledged by user {}",
            request.alert_id,
            request.user_id.unwrap_or_else(|| "anonymous".to_string())
        );
        Ok(APIResponse::success(()))
    }

    /// Get API health status
    /// GET /api/v1/status
    pub async fn get_api_status(&self) -> Result<APIResponse<APIStatusResponse>> {
        let response = APIStatusResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            uptime: "24h 15m 30s".to_string(), // In real implementation, calculate actual uptime
            timestamp: Utc::now(),
            endpoints_available: 11,
            last_data_update: Utc::now(), // In real implementation, get from analytics system
        };
        Ok(APIResponse::success(response))
    }

    fn calculate_overall_health(&self, health: &super::EconomicHealthIndicators) -> f64 {
        // Calculate weighted average of health indicators
        let weights = [0.2, 0.15, 0.15, 0.15, 0.1, 0.1, 0.15]; // Sum = 1.0
        let values = [
            health.token_velocity / 6.0,                // Normalize to 0-1 scale
            1.0 - health.inflation_rate.min(0.2) / 0.2, // Lower inflation is better
            1.0 - health.wealth_concentration,          // Lower concentration is better
            health.participation_rate,
            health.treasury_health,
            health.liquidity_score,
            health.stability_index,
        ];

        weights
            .iter()
            .zip(values.iter())
            .map(|(w, v)| w * v)
            .sum::<f64>()
            .min(1.0)
            .max(0.0)
    }
}

// API Response types

#[derive(Debug, Serialize, Deserialize)]
pub struct APIResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> APIResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMetricsResponse {
    pub timestamp: DateTime<Utc>,
    pub total_supply: u64,
    pub active_participants: u64,
    pub transaction_throughput: f64,
    pub uptime_percentage: f64,
    pub error_rate: f64,
    pub governance_participation_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalMetricsResponse {
    pub timeframe: TimeFrame,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_data_points: usize,
    pub average_participants: f64,
    pub average_throughput: f64,
    pub trends: HistoricalTrends,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalTrends {
    pub token_supply_trend: super::network_analytics::TrendDirection,
    pub participation_trend: super::network_analytics::TrendDirection,
    pub performance_trend: super::network_analytics::TrendDirection,
    pub volatility_index: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetricsResponse {
    pub timestamp: DateTime<Utc>,
    pub throughput: f64,
    pub latency: f64,
    pub error_rate: f64,
    pub uptime: f64,
    pub resource_utilization: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthIndicatorsResponse {
    pub timestamp: DateTime<Utc>,
    pub token_velocity: f64,
    pub inflation_rate: f64,
    pub wealth_concentration: f64,
    pub participation_rate: f64,
    pub treasury_health: f64,
    pub liquidity_score: f64,
    pub stability_index: f64,
    pub overall_health_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertsResponse {
    pub total_alerts: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub info_alerts: usize,
    pub alerts: Vec<AlertSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertSummary {
    pub id: Uuid,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsReportResponse {
    pub report_id: Uuid,
    pub timeframe: TimeFrame,
    pub generated_at: DateTime<Utc>,
    pub summary: ReportSummaryResponse,
    pub insights_count: usize,
    pub top_insights: Vec<InsightSummary>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSummaryResponse {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_data_points: usize,
    pub average_participants: f64,
    pub average_throughput: f64,
    pub total_contributions: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsightSummary {
    pub category: super::network_analytics::InsightCategory,
    pub severity: super::network_analytics::InsightSeverity,
    pub message: String,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartDataResponse {
    pub chart_type: String,
    pub title: String,
    pub data_points: usize,
    pub data: Vec<ChartPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcknowledgeAlertRequest {
    pub alert_id: Uuid,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APIStatusResponse {
    pub status: String,
    pub version: String,
    pub uptime: String,
    pub timestamp: DateTime<Utc>,
    pub endpoints_available: u32,
    pub last_data_update: DateTime<Utc>,
}

#[derive(Debug)]
pub struct APIConfig {
    pub port: u16,
    pub host: String,
    pub enable_cors: bool,
    pub rate_limit_requests_per_minute: u32,
}

impl Default for APIConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            enable_cors: true,
            rate_limit_requests_per_minute: 100,
        }
    }
}
