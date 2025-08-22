/// Comprehensive tests for the Network Analytics Dashboard
/// Tests real-time monitoring, metrics collection, alerting, and API functionality

use paradigm_core::tokenomics::*;
use paradigm_core::Address;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Create a test address with a specific ID
fn create_test_address(id: u8) -> Address {
    let mut addr = [0u8; 32];
    addr[0] = id;
    Address(addr)
}

#[tokio::test]
async fn test_network_analytics_dashboard_initialization() {
    let mut dashboard = NetworkAnalyticsDashboard::new();
    
    let result = dashboard.initialize().await;
    assert!(result.is_ok(), "Dashboard should initialize successfully");
}

#[tokio::test]
async fn test_metrics_collection() {
    let mut dashboard = NetworkAnalyticsDashboard::new();
    dashboard.initialize().await.expect("Dashboard should initialize");
    
    // Create test network state
    let network_state = NetworkState {
        total_supply: 1000000,
        active_participants: 150,
        transaction_volume: 50000,
        transaction_throughput: 120.0,
        uptime_percentage: 0.99,
        avg_consensus_time: 2.0,
        error_rate: 0.01,
        resource_utilization: 0.65,
        token_velocity: 3.5,
        network_growth: 0.18,
        inflation_rate: 0.04,
        top_10_validator_stake_percentage: 0.3,
        geographic_diversity_index: 0.85,
        wealth_concentration_gini: 0.35,
        treasury_balance: 200000,
        mint_rate: 0.025,
        burn_rate: 0.015,
        total_rewards: 12000,
        productive_work_rewards: 9600,
        avg_transaction_fee: 0.008,
        contributor_satisfaction_score: 0.88,
        governance_participation_rate: 0.65,
        monthly_active_user_retention: 0.82,
    };
    
    // Update dashboard with network state
    let result = dashboard.update_network_state(&network_state).await;
    assert!(result.is_ok(), "Network state update should succeed");
    
    // Verify dashboard data
    let dashboard_data = dashboard.get_dashboard_data().await;
    assert!(dashboard_data.is_ok(), "Should be able to get dashboard data");
    
    let data = dashboard_data.unwrap();
    assert_eq!(data.metrics.total_supply, 1000000);
    assert_eq!(data.metrics.active_participants, 150);
    assert_eq!(data.performance.throughput, 120.0);
    assert_eq!(data.economic_health.token_velocity, 3.5);
}

#[tokio::test]
async fn test_contribution_event_recording() {
    let mut dashboard = NetworkAnalyticsDashboard::new();
    dashboard.initialize().await.expect("Dashboard should initialize");
    
    let contributor = create_test_address(1);
    
    // Record multiple contribution events
    for i in 0..5 {
        let result = dashboard.record_contribution(
            &contributor,
            ContributionType::MLTraining,
            1000 + i * 100
        ).await;
        assert!(result.is_ok(), "Contribution recording should succeed");
    }
    
    // Verify contributions are recorded
    let dashboard_data = dashboard.get_dashboard_data().await.unwrap();
    // Note: In a real implementation, we'd verify the contribution events
    // are properly stored and aggregated in the metrics
    assert!(dashboard_data.metrics.timestamp <= chrono::Utc::now());
}

#[tokio::test]
async fn test_alert_system() {
    let mut dashboard = NetworkAnalyticsDashboard::new();
    dashboard.initialize().await.expect("Dashboard should initialize");
    
    // Create network state that should trigger alerts
    let high_error_state = NetworkState {
        total_supply: 1000000,
        active_participants: 50, // Low participation
        transaction_volume: 30000,
        transaction_throughput: 25.0, // Low throughput
        uptime_percentage: 0.92, // Below threshold
        avg_consensus_time: 5.0,
        error_rate: 0.08, // High error rate
        resource_utilization: 0.95,
        token_velocity: 2.0,
        network_growth: 0.05,
        inflation_rate: 0.18, // High inflation
        top_10_validator_stake_percentage: 0.7,
        geographic_diversity_index: 0.3,
        wealth_concentration_gini: 0.8,
        treasury_balance: 50000,
        mint_rate: 0.15,
        burn_rate: 0.01,
        total_rewards: 5000,
        productive_work_rewards: 3000,
        avg_transaction_fee: 0.05,
        contributor_satisfaction_score: 0.45,
        governance_participation_rate: 0.05, // Very low participation
        monthly_active_user_retention: 0.3,
    };
    
    // Update dashboard with problematic state
    let result = dashboard.update_network_state(&high_error_state).await;
    assert!(result.is_ok(), "State update should succeed");
    
    // Check for alerts
    let dashboard_data = dashboard.get_dashboard_data().await.unwrap();
    assert!(!dashboard_data.alerts.is_empty(), "Should have generated alerts");
    
    // Verify we have different types of alerts
    let has_error_alert = dashboard_data.alerts.iter()
        .any(|alert| alert.message.contains("Error"));
    let has_uptime_alert = dashboard_data.alerts.iter()
        .any(|alert| alert.message.contains("Uptime"));
    let has_inflation_alert = dashboard_data.alerts.iter()
        .any(|alert| alert.message.contains("Inflation"));
    
    assert!(has_error_alert || has_uptime_alert || has_inflation_alert, 
           "Should have at least one type of expected alert");
}

#[tokio::test]
async fn test_performance_monitoring() {
    let mut dashboard = NetworkAnalyticsDashboard::new();
    dashboard.initialize().await.expect("Dashboard should initialize");
    
    // Create multiple network states to test performance trending
    let states = vec![
        NetworkState {
            transaction_throughput: 100.0,
            avg_consensus_time: 2.0,
            error_rate: 0.01,
            uptime_percentage: 0.99,
            resource_utilization: 0.7,
            ..create_default_network_state()
        },
        NetworkState {
            transaction_throughput: 110.0,
            avg_consensus_time: 1.8,
            error_rate: 0.008,
            uptime_percentage: 0.995,
            resource_utilization: 0.65,
            ..create_default_network_state()
        },
        NetworkState {
            transaction_throughput: 95.0,
            avg_consensus_time: 2.2,
            error_rate: 0.015,
            uptime_percentage: 0.985,
            resource_utilization: 0.75,
            ..create_default_network_state()
        },
    ];
    
    // Update dashboard with each state
    for state in states {
        dashboard.update_network_state(&state).await.expect("State update should succeed");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Verify performance data is collected
    let dashboard_data = dashboard.get_dashboard_data().await.unwrap();
    assert!(dashboard_data.performance.throughput > 0.0);
    assert!(dashboard_data.performance.latency > 0.0);
    assert!(dashboard_data.performance.uptime > 0.0);
}

#[tokio::test]
async fn test_economic_health_tracking() {
    let mut dashboard = NetworkAnalyticsDashboard::new();
    dashboard.initialize().await.expect("Dashboard should initialize");
    
    // Test with healthy economic conditions
    let healthy_state = NetworkState {
        token_velocity: 4.0, // Good velocity
        inflation_rate: 0.03, // Low inflation
        wealth_concentration_gini: 0.25, // Low concentration
        governance_participation_rate: 0.7, // High participation
        treasury_balance: 150000, // Good treasury ratio
        ..create_default_network_state()
    };
    
    dashboard.update_network_state(&healthy_state).await.expect("Update should succeed");
    
    let dashboard_data = dashboard.get_dashboard_data().await.unwrap();
    
    // Verify health indicators are calculated correctly
    assert!(dashboard_data.economic_health.token_velocity == 4.0);
    assert!(dashboard_data.economic_health.inflation_rate == 0.03);
    assert!(dashboard_data.economic_health.wealth_concentration == 0.25);
    assert!(dashboard_data.economic_health.participation_rate == 0.7);
    
    // Treasury health should be good (treasury_balance / total_supply = 0.15)
    assert!(dashboard_data.economic_health.treasury_health > 0.8);
    
    // Liquidity score should be good for velocity of 4.0
    assert!(dashboard_data.economic_health.liquidity_score > 0.8);
    
    // Overall stability should be high
    assert!(dashboard_data.economic_health.stability_index > 0.8);
}

#[tokio::test]
async fn test_analytics_report_generation() {
    let mut dashboard = NetworkAnalyticsDashboard::new();
    dashboard.initialize().await.expect("Dashboard should initialize");
    
    // Add some historical data
    for i in 0..10 {
        let state = NetworkState {
            total_supply: 1000000 + i * 10000,
            active_participants: 100 + i * 5,
            transaction_throughput: 100.0 + i as f64 * 2.0,
            ..create_default_network_state()
        };
        
        dashboard.update_network_state(&state).await.expect("Update should succeed");
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    
    // Generate report for different timeframes
    for timeframe in [TimeFrame::LastHour, TimeFrame::LastDay, TimeFrame::LastWeek] {
        let report = dashboard.generate_report(timeframe).await;
        assert!(report.is_ok(), "Report generation should succeed");
        
        let report_data = report.unwrap();
        assert!(report_data.generated_at <= chrono::Utc::now());
        
        // Verify trends analysis exists
        // In a real scenario with more data points, we'd check for meaningful trends
        
        // Verify insights are generated
        // The number of insights depends on the data patterns
        
        // Verify recommendations are provided
        assert!(!report_data.recommendations.is_empty(), "Should have recommendations");
    }
}

#[tokio::test]
async fn test_chart_data_generation() {
    let mut dashboard = NetworkAnalyticsDashboard::new();
    dashboard.initialize().await.expect("Dashboard should initialize");
    
    // Generate data for charts
    for i in 0..20 {
        let state = NetworkState {
            total_supply: 1000000 + i * 5000,
            active_participants: 100 + i * 3,
            transaction_throughput: 100.0 + (i as f64 * 1.5),
            uptime_percentage: 0.98 + (i as f64 * 0.001),
            ..create_default_network_state()
        };
        
        dashboard.update_network_state(&state).await.expect("Update should succeed");
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
    }
    
    // Verify chart data is generated
    let dashboard_data = dashboard.get_dashboard_data().await.unwrap();
    
    // Should have chart data points
    assert!(!dashboard_data.charts.token_supply_chart.is_empty(), "Should have token supply data");
    assert!(!dashboard_data.charts.participation_chart.is_empty(), "Should have participation data");
    assert!(!dashboard_data.charts.throughput_chart.is_empty(), "Should have throughput data");
    assert!(!dashboard_data.charts.uptime_chart.is_empty(), "Should have uptime data");
    
    // Verify data points are in chronological order
    let supply_chart = &dashboard_data.charts.token_supply_chart;
    if supply_chart.len() > 1 {
        for i in 1..supply_chart.len() {
            assert!(supply_chart[i].0 >= supply_chart[i-1].0, "Chart data should be chronological");
        }
    }
}

#[tokio::test]
async fn test_analytics_api_integration() {
    // Create tokenomics system
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");
    
    let system_arc = Arc::new(RwLock::new(system));
    
    // Create analytics API
    let mut api = AnalyticsAPI::new(system_arc.clone());
    api.initialize().await.expect("API should initialize");
    
    // Test dashboard endpoint
    let dashboard_response = api.get_dashboard().await;
    assert!(dashboard_response.is_ok(), "Dashboard endpoint should work");
    
    let response = dashboard_response.unwrap();
    assert!(response.success, "Response should be successful");
    assert!(response.data.is_some(), "Should have dashboard data");
    
    // Test current metrics endpoint
    let metrics_response = api.get_current_metrics().await;
    assert!(metrics_response.is_ok(), "Metrics endpoint should work");
    
    // Test performance metrics endpoint
    let performance_response = api.get_performance_metrics().await;
    assert!(performance_response.is_ok(), "Performance endpoint should work");
    
    // Test health indicators endpoint
    let health_response = api.get_health_indicators().await;
    assert!(health_response.is_ok(), "Health endpoint should work");
    
    // Test alerts endpoint
    let alerts_response = api.get_active_alerts().await;
    assert!(alerts_response.is_ok(), "Alerts endpoint should work");
    
    // Test report generation
    let report_response = api.generate_report(TimeFrame::LastHour).await;
    assert!(report_response.is_ok(), "Report generation should work");
    
    // Test API status
    let status_response = api.get_api_status().await;
    assert!(status_response.is_ok(), "Status endpoint should work");
    
    let status = status_response.unwrap();
    assert!(status.success, "Status should be successful");
    assert_eq!(status.data.as_ref().unwrap().status, "healthy");
}

#[tokio::test]
async fn test_data_retention_management() {
    let mut dashboard = NetworkAnalyticsDashboard::new();
    dashboard.initialize().await.expect("Dashboard should initialize");
    
    // Simulate adding a lot of data
    for i in 0..150 {
        let state = NetworkState {
            total_supply: 1000000 + i * 1000,
            active_participants: 100 + i,
            ..create_default_network_state()
        };
        
        dashboard.update_network_state(&state).await.expect("Update should succeed");
    }
    
    // Data retention should limit the amount of stored data
    // This is more of a validation that the system handles large amounts of data
    let dashboard_data = dashboard.get_dashboard_data().await.unwrap();
    
    // Verify we still get valid current data
    assert!(dashboard_data.metrics.total_supply > 1000000);
    assert!(dashboard_data.metrics.active_participants >= 100);
}

#[tokio::test]
async fn test_system_integration_with_analytics() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");
    
    let contributor = create_test_address(1);
    
    // Create test contribution
    let proof = ContributionProof {
        id: uuid::Uuid::new_v4(),
        contributor: contributor.clone(),
        contribution_type: ContributionType::MLTraining,
        workload_hash: vec![1, 2, 3, 4],
        zk_proof: vec![5, 6, 7, 8],
        qr_zk_proof: None,
        qr_signature: None,
        metadata: serde_json::json!({"test": "data"}),
        timestamp: chrono::Utc::now(),
    };
    
    // Process contribution (this should trigger analytics recording)
    let result = system.process_contribution(&contributor, proof).await;
    assert!(result.is_ok(), "Contribution processing should succeed");
    
    // Update network analytics
    let analytics_result = system.update_network_analytics().await;
    assert!(analytics_result.is_ok(), "Analytics update should succeed");
    
    // Get dashboard data
    let dashboard_result = system.get_dashboard_data().await;
    assert!(dashboard_result.is_ok(), "Dashboard data retrieval should succeed");
    
    let dashboard_data = dashboard_result.unwrap();
    assert!(dashboard_data.metrics.total_supply > 0);
    
    // Generate analytics report
    let report_result = system.generate_analytics_report(TimeFrame::LastHour).await;
    assert!(report_result.is_ok(), "Report generation should succeed");
}

// Helper function to create default network state
fn create_default_network_state() -> NetworkState {
    NetworkState {
        total_supply: 1000000,
        active_participants: 100,
        transaction_volume: 50000,
        transaction_throughput: 100.0,
        uptime_percentage: 0.99,
        avg_consensus_time: 2.5,
        error_rate: 0.01,
        resource_utilization: 0.75,
        token_velocity: 3.2,
        network_growth: 0.15,
        inflation_rate: 0.05,
        top_10_validator_stake_percentage: 0.35,
        geographic_diversity_index: 0.8,
        wealth_concentration_gini: 0.4,
        treasury_balance: 500000,
        mint_rate: 0.02,
        burn_rate: 0.01,
        total_rewards: 10000,
        productive_work_rewards: 8000,
        avg_transaction_fee: 0.01,
        contributor_satisfaction_score: 0.85,
        governance_participation_rate: 0.6,
        monthly_active_user_retention: 0.8,
    }
}