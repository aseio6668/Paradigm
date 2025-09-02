use crate::Address;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Decentralized model hosting marketplace where contributors host models
/// and earn tokens for serving inference requests
#[derive(Debug)]
pub struct ModelHosting {
    /// Registry of available models
    model_registry: HashMap<Uuid, HostedModel>,
    /// Active hosting providers
    hosting_providers: HashMap<Address, HostingProvider>,
    /// Inference request queue
    inference_queue: HashMap<Uuid, InferenceRequest>,
    /// Load balancer for distributing requests
    load_balancer: LoadBalancer,
    /// Model performance tracker
    performance_tracker: ModelPerformanceTracker,
    /// Pricing engine for dynamic model pricing
    pricing_engine: ModelPricingEngine,
    /// Quality assurance system
    quality_assurance: QualityAssuranceSystem,
}

impl ModelHosting {
    pub fn new() -> Self {
        ModelHosting {
            model_registry: HashMap::new(),
            hosting_providers: HashMap::new(),
            inference_queue: HashMap::new(),
            load_balancer: LoadBalancer::new(),
            performance_tracker: ModelPerformanceTracker::new(),
            pricing_engine: ModelPricingEngine::new(),
            quality_assurance: QualityAssuranceSystem::new(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::info!("Initializing decentralized model hosting marketplace");

        // Initialize load balancer
        self.load_balancer.initialize().await?;

        // Initialize performance tracker
        self.performance_tracker.initialize().await?;

        // Initialize pricing engine
        self.pricing_engine.initialize().await?;

        // Initialize quality assurance
        self.quality_assurance.initialize().await?;

        tracing::info!("Model hosting marketplace initialized successfully");
        Ok(())
    }

    /// Register a new model for hosting
    pub async fn register_model(
        &mut self,
        provider: Address,
        model_spec: ModelSpec,
    ) -> anyhow::Result<Uuid> {
        // Validate model specification
        self.quality_assurance
            .validate_model_spec(&model_spec)
            .await?;

        let model_id = Uuid::new_v4();
        let hosted_model = HostedModel {
            id: model_id,
            provider: provider.clone(),
            spec: model_spec.clone(),
            status: ModelStatus::Validating,
            registered_at: Utc::now(),
            performance_metrics: ModelMetrics::default(),
            pricing: self
                .pricing_engine
                .calculate_initial_pricing(&model_spec)
                .await?,
            total_inferences: 0,
            total_earnings: 0,
            reputation_score: 0.5, // Start with neutral reputation
        };

        // Add to registry
        self.model_registry.insert(model_id, hosted_model);

        // Register provider if new
        if !self.hosting_providers.contains_key(&provider) {
            let hosting_provider = HostingProvider {
                address: provider.clone(),
                registered_at: Utc::now(),
                hosted_models: vec![model_id],
                total_inferences_served: 0,
                total_earnings: 0,
                reputation_score: 0.5,
                compute_capabilities: model_spec.compute_requirements.clone(),
                availability_zone: model_spec.availability_zone.clone(),
            };
            self.hosting_providers
                .insert(provider.clone(), hosting_provider);
        } else {
            // Add model to existing provider
            if let Some(provider_info) = self.hosting_providers.get_mut(&provider) {
                provider_info.hosted_models.push(model_id);
            }
        }

        // Start quality assurance process
        self.quality_assurance
            .start_model_validation(model_id)
            .await?;

        tracing::info!(
            "Model {} registered by provider {}",
            model_id,
            provider.to_string()
        );
        Ok(model_id)
    }

    /// Submit an inference request
    pub async fn submit_inference_request(
        &mut self,
        requester: Address,
        request_spec: InferenceRequestSpec,
    ) -> anyhow::Result<Uuid> {
        let request_id = Uuid::new_v4();

        // Find suitable models for this request
        let suitable_models = self.find_suitable_models(&request_spec).await?;

        if suitable_models.is_empty() {
            return Err(anyhow::anyhow!(
                "No suitable models available for this request"
            ));
        }

        // Select optimal model based on load balancing strategy
        let selected_model = self
            .load_balancer
            .select_model(&suitable_models, &request_spec)
            .await?;

        // Calculate pricing for this request (clone values to avoid borrow issues)
        let selected_model_id = selected_model.id;
        let selected_provider = selected_model.provider.clone();

        let pricing = self
            .pricing_engine
            .calculate_request_pricing(&selected_model, &request_spec)
            .await?;

        let inference_request = InferenceRequest {
            id: request_id,
            requester,
            model_id: selected_model_id,
            provider: selected_provider,
            spec: request_spec,
            pricing,
            status: InferenceStatus::Pending,
            submitted_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
        };

        // Add to inference queue
        self.inference_queue.insert(request_id, inference_request);

        tracing::info!(
            "Inference request {} submitted for model {}",
            request_id,
            selected_model_id
        );
        Ok(request_id)
    }

    /// Process inference request (called by hosting providers)
    pub async fn process_inference_request(
        &mut self,
        provider: &Address,
        request_id: Uuid,
        result: InferenceResult,
    ) -> anyhow::Result<u64> {
        // Validate provider is authorized for this request
        let (earnings, model_id) = if let Some(request) = self.inference_queue.get_mut(&request_id)
        {
            if request.provider != *provider {
                return Err(anyhow::anyhow!("Provider not authorized for this request"));
            }

            // Update request status
            request.status = InferenceStatus::Completed;
            request.completed_at = Some(Utc::now());
            request.result = Some(result.clone());

            // Store values we need before releasing the borrow
            let request_clone = request.clone();
            let model_id = request.model_id;

            // Release the mutable borrow by dropping the reference
            drop(request);

            // Calculate earnings for provider
            let earnings = self
                .calculate_inference_earnings(&request_clone, &result)
                .await?;

            (earnings, model_id)
        } else {
            return Err(anyhow::anyhow!("Inference request not found"));
        };

        // Update model metrics
        if let Some(model) = self.model_registry.get_mut(&model_id) {
            model.total_inferences += 1;
            model.total_earnings += earnings;
        }

        // Update performance metrics
        self.performance_tracker
            .update_model_performance(&model_id, &result)
            .await?;

        // Update provider metrics
        if let Some(provider_info) = self.hosting_providers.get_mut(provider) {
            provider_info.total_inferences_served += 1;
            provider_info.total_earnings += earnings;
        }

        tracing::info!(
            "Inference request {} completed, {} PAR earned",
            request_id,
            earnings as f64 / 100_000_000.0
        );

        Ok(earnings)
    }

    /// Find models suitable for an inference request
    async fn find_suitable_models(
        &self,
        request_spec: &InferenceRequestSpec,
    ) -> anyhow::Result<Vec<&HostedModel>> {
        let mut suitable_models = Vec::new();

        for model in self.model_registry.values() {
            if model.status == ModelStatus::Active
                && self.is_model_compatible(model, request_spec).await?
            {
                suitable_models.push(model);
            }
        }

        // Sort by reputation and performance
        suitable_models.sort_by(|a, b| {
            let score_a = a.reputation_score * a.performance_metrics.average_latency_score();
            let score_b = b.reputation_score * b.performance_metrics.average_latency_score();
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(suitable_models)
    }

    async fn is_model_compatible(
        &self,
        model: &HostedModel,
        request_spec: &InferenceRequestSpec,
    ) -> anyhow::Result<bool> {
        // Check model type compatibility
        if model.spec.model_type != request_spec.model_type {
            return Ok(false);
        }

        // Check compute requirements
        if request_spec.max_latency_ms > 0
            && model.performance_metrics.average_latency_ms > request_spec.max_latency_ms as f64
        {
            return Ok(false);
        }

        // Check availability requirements
        if let Some(required_zone) = &request_spec.preferred_availability_zone {
            if let Some(model_zone) = &model.spec.availability_zone {
                if model_zone != required_zone {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    async fn calculate_inference_earnings(
        &self,
        request: &InferenceRequest,
        result: &InferenceResult,
    ) -> anyhow::Result<u64> {
        let base_earnings = request.pricing.base_cost;

        // Apply quality multiplier based on result quality
        let quality_multiplier = if result.confidence_score > 0.9 {
            1.2
        } else if result.confidence_score > 0.7 {
            1.0
        } else {
            0.8
        };

        // Apply latency bonus/penalty
        let latency_multiplier = if result.processing_time_ms < request.spec.max_latency_ms / 2 {
            1.1 // Bonus for fast response
        } else if result.processing_time_ms > request.spec.max_latency_ms {
            0.9 // Penalty for slow response
        } else {
            1.0
        };

        let final_earnings =
            (base_earnings as f64 * quality_multiplier * latency_multiplier) as u64;
        Ok(final_earnings)
    }

    /// Get marketplace statistics
    pub fn get_marketplace_stats(&self) -> MarketplaceStats {
        let total_models = self.model_registry.len();
        let active_models = self
            .model_registry
            .values()
            .filter(|m| m.status == ModelStatus::Active)
            .count();
        let total_providers = self.hosting_providers.len();
        let total_inferences = self
            .model_registry
            .values()
            .map(|m| m.total_inferences)
            .sum();
        let total_earnings = self.model_registry.values().map(|m| m.total_earnings).sum();

        MarketplaceStats {
            total_models,
            active_models,
            total_providers,
            total_inferences,
            total_earnings,
            pending_requests: self.inference_queue.len(),
        }
    }

    /// Update model reputation based on performance
    pub async fn update_model_reputation(
        &mut self,
        model_id: Uuid,
        reputation_update: ReputationUpdate,
    ) -> anyhow::Result<()> {
        if let Some(model) = self.model_registry.get_mut(&model_id) {
            // Update reputation using weighted average
            let new_reputation = (model.reputation_score * 0.9) + (reputation_update.score * 0.1);
            model.reputation_score = new_reputation.max(0.0).min(1.0);

            // Store values before releasing borrow
            let provider_address = model.provider.clone();
            let updated_reputation = model.reputation_score;

            tracing::debug!(
                "Updated reputation for model {} to {:.3}",
                model_id,
                updated_reputation
            );
        }

        // Update provider reputation separately to avoid borrow conflicts
        if let Some(model) = self.model_registry.get(&model_id) {
            let provider_address = &model.provider;
            if let Some(provider) = self.hosting_providers.get_mut(provider_address) {
                let avg_model_reputation: f64 = provider
                    .hosted_models
                    .iter()
                    .filter_map(|id| self.model_registry.get(id))
                    .map(|m| m.reputation_score)
                    .sum::<f64>()
                    / provider.hosted_models.len() as f64;
                provider.reputation_score = avg_model_reputation;
            }
        }

        Ok(())
    }
}

/// Load balancer for distributing inference requests
#[derive(Debug)]
pub struct LoadBalancer {
    balancing_strategy: LoadBalancingStrategy,
}

impl LoadBalancer {
    pub fn new() -> Self {
        LoadBalancer {
            balancing_strategy: LoadBalancingStrategy::ReputationWeighted,
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::debug!("Initializing load balancer");
        Ok(())
    }

    pub async fn select_model<'a>(
        &self,
        suitable_models: &'a [&'a HostedModel],
        _request_spec: &InferenceRequestSpec,
    ) -> anyhow::Result<&'a HostedModel> {
        if suitable_models.is_empty() {
            return Err(anyhow::anyhow!("No models available"));
        }

        match self.balancing_strategy {
            LoadBalancingStrategy::RoundRobin => {
                // Simple round-robin selection
                Ok(suitable_models[0])
            }
            LoadBalancingStrategy::ReputationWeighted => {
                // Select based on reputation score
                let best_model = suitable_models
                    .iter()
                    .max_by(|a, b| a.reputation_score.partial_cmp(&b.reputation_score).unwrap())
                    .unwrap();
                Ok(best_model)
            }
            LoadBalancingStrategy::LatencyOptimized => {
                // Select model with best latency
                let fastest_model = suitable_models
                    .iter()
                    .min_by(|a, b| {
                        a.performance_metrics
                            .average_latency_ms
                            .partial_cmp(&b.performance_metrics.average_latency_ms)
                            .unwrap()
                    })
                    .unwrap();
                Ok(fastest_model)
            }
        }
    }
}

/// Model performance tracking system
#[derive(Debug)]
pub struct ModelPerformanceTracker {
    performance_history: HashMap<Uuid, Vec<PerformanceRecord>>,
}

impl ModelPerformanceTracker {
    pub fn new() -> Self {
        ModelPerformanceTracker {
            performance_history: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::debug!("Initializing performance tracker");
        Ok(())
    }

    pub async fn update_model_performance(
        &mut self,
        model_id: &Uuid,
        result: &InferenceResult,
    ) -> anyhow::Result<()> {
        let record = PerformanceRecord {
            timestamp: Utc::now(),
            latency_ms: result.processing_time_ms,
            confidence_score: result.confidence_score,
            accuracy: result.accuracy.unwrap_or(0.0),
            throughput: 1.0 / (result.processing_time_ms as f64 / 1000.0), // requests per second
        };

        self.performance_history
            .entry(*model_id)
            .or_insert_with(Vec::new)
            .push(record);

        // Keep only recent history (last 1000 records)
        if let Some(history) = self.performance_history.get_mut(model_id) {
            if history.len() > 1000 {
                history.drain(0..100);
            }
        }

        Ok(())
    }
}

/// Dynamic pricing engine for model inference
#[derive(Debug)]
pub struct ModelPricingEngine {
    base_pricing_rates: HashMap<ModelType, u64>,
}

impl ModelPricingEngine {
    pub fn new() -> Self {
        let mut base_rates = HashMap::new();
        base_rates.insert(ModelType::LanguageModel, 1_000_000); // 0.01 PAR per request
        base_rates.insert(ModelType::ImageGeneration, 5_000_000); // 0.05 PAR per request
        base_rates.insert(ModelType::CodeGeneration, 2_000_000); // 0.02 PAR per request
        base_rates.insert(ModelType::AudioProcessing, 3_000_000); // 0.03 PAR per request
        base_rates.insert(ModelType::VideoAnalysis, 10_000_000); // 0.1 PAR per request

        ModelPricingEngine {
            base_pricing_rates: base_rates,
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::debug!("Initializing pricing engine");
        Ok(())
    }

    pub async fn calculate_initial_pricing(
        &self,
        model_spec: &ModelSpec,
    ) -> anyhow::Result<ModelPricing> {
        let base_rate = *self
            .base_pricing_rates
            .get(&model_spec.model_type)
            .unwrap_or(&1_000_000);

        // Adjust pricing based on compute requirements
        let compute_multiplier =
            self.calculate_compute_multiplier(&model_spec.compute_requirements);

        let adjusted_rate = (base_rate as f64 * compute_multiplier) as u64;

        Ok(ModelPricing {
            base_cost_per_request: adjusted_rate,
            compute_cost_multiplier: compute_multiplier,
            quality_bonus_rate: 0.2,
            latency_penalty_rate: 0.1,
        })
    }

    pub async fn calculate_request_pricing(
        &self,
        model: &HostedModel,
        request_spec: &InferenceRequestSpec,
    ) -> anyhow::Result<RequestPricing> {
        let base_cost = model.pricing.base_cost_per_request;

        // Apply urgency multiplier
        let urgency_multiplier = if request_spec.max_latency_ms < 1000 {
            1.5 // Premium for low latency
        } else if request_spec.max_latency_ms < 5000 {
            1.2
        } else {
            1.0
        };

        // Apply complexity multiplier based on input size
        let complexity_multiplier = self.calculate_complexity_multiplier(request_spec);

        let final_cost = (base_cost as f64 * urgency_multiplier * complexity_multiplier) as u64;

        Ok(RequestPricing {
            base_cost: final_cost,
            urgency_multiplier,
            complexity_multiplier,
            total_cost: final_cost,
        })
    }

    fn calculate_compute_multiplier(&self, requirements: &ComputeRequirements) -> f64 {
        let gpu_factor = if requirements.requires_gpu { 2.0 } else { 1.0 };
        let memory_factor = (requirements.memory_gb as f64 / 8.0).max(1.0);
        let cpu_factor = (requirements.cpu_cores as f64 / 4.0).max(1.0);

        gpu_factor * memory_factor.sqrt() * cpu_factor.sqrt()
    }

    fn calculate_complexity_multiplier(&self, request_spec: &InferenceRequestSpec) -> f64 {
        // Simplified complexity calculation based on input size
        let input_size_factor = (request_spec.estimated_input_size_kb as f64 / 100.0).max(1.0);
        input_size_factor.sqrt()
    }
}

/// Quality assurance system for model validation
#[derive(Debug)]
pub struct QualityAssuranceSystem {
    validation_queue: HashMap<Uuid, ValidationProcess>,
}

impl QualityAssuranceSystem {
    pub fn new() -> Self {
        QualityAssuranceSystem {
            validation_queue: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::debug!("Initializing quality assurance system");
        Ok(())
    }

    pub async fn validate_model_spec(&self, model_spec: &ModelSpec) -> anyhow::Result<()> {
        // Basic validation checks
        if model_spec.model_name.is_empty() {
            return Err(anyhow::anyhow!("Model name cannot be empty"));
        }

        if model_spec.compute_requirements.memory_gb < 1 {
            return Err(anyhow::anyhow!("Model requires at least 1GB memory"));
        }

        if model_spec.compute_requirements.cpu_cores < 1 {
            return Err(anyhow::anyhow!("Model requires at least 1 CPU core"));
        }

        Ok(())
    }

    pub async fn start_model_validation(&mut self, model_id: Uuid) -> anyhow::Result<()> {
        let validation_process = ValidationProcess {
            model_id,
            started_at: Utc::now(),
            status: ValidationStatus::InProgress,
            test_cases_passed: 0,
            test_cases_total: 5, // Standard test suite
        };

        self.validation_queue.insert(model_id, validation_process);

        // In real implementation, this would start automated testing
        tracing::info!("Started validation process for model {}", model_id);
        Ok(())
    }
}

// Data structures

#[derive(Debug, Clone)]
pub struct ModelSpec {
    pub model_name: String,
    pub model_type: ModelType,
    pub model_version: String,
    pub description: String,
    pub compute_requirements: ComputeRequirements,
    pub supported_input_formats: Vec<String>,
    pub supported_output_formats: Vec<String>,
    pub availability_zone: Option<String>,
    pub api_endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelType {
    LanguageModel,
    ImageGeneration,
    ImageClassification,
    CodeGeneration,
    AudioProcessing,
    VideoAnalysis,
    DataAnalysis,
    ReinforcementLearning,
}

#[derive(Debug, Clone)]
pub struct ComputeRequirements {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub requires_gpu: bool,
    pub gpu_memory_gb: Option<u32>,
    pub storage_gb: u32,
    pub network_bandwidth_mbps: u32,
}

#[derive(Debug)]
pub struct HostedModel {
    pub id: Uuid,
    pub provider: Address,
    pub spec: ModelSpec,
    pub status: ModelStatus,
    pub registered_at: DateTime<Utc>,
    pub performance_metrics: ModelMetrics,
    pub pricing: ModelPricing,
    pub total_inferences: u64,
    pub total_earnings: u64,
    pub reputation_score: f64,
}

#[derive(Debug, PartialEq)]
pub enum ModelStatus {
    Validating,
    Active,
    Inactive,
    Deprecated,
    Suspended,
}

#[derive(Debug, Default)]
pub struct ModelMetrics {
    pub average_latency_ms: f64,
    pub success_rate: f64,
    pub average_confidence: f64,
    pub uptime_percentage: f64,
}

impl ModelMetrics {
    pub fn average_latency_score(&self) -> f64 {
        // Convert latency to a score (lower latency = higher score)
        (1000.0 / (self.average_latency_ms + 100.0)).min(1.0)
    }
}

#[derive(Debug)]
pub struct ModelPricing {
    pub base_cost_per_request: u64,
    pub compute_cost_multiplier: f64,
    pub quality_bonus_rate: f64,
    pub latency_penalty_rate: f64,
}

#[derive(Debug)]
pub struct HostingProvider {
    pub address: Address,
    pub registered_at: DateTime<Utc>,
    pub hosted_models: Vec<Uuid>,
    pub total_inferences_served: u64,
    pub total_earnings: u64,
    pub reputation_score: f64,
    pub compute_capabilities: ComputeRequirements,
    pub availability_zone: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InferenceRequestSpec {
    pub model_type: ModelType,
    pub input_format: String,
    pub output_format: String,
    pub max_latency_ms: u64,
    pub estimated_input_size_kb: u64,
    pub preferred_availability_zone: Option<String>,
    pub quality_requirements: QualityRequirements,
}

#[derive(Debug, Clone)]
pub struct QualityRequirements {
    pub min_confidence_score: f64,
    pub min_accuracy: Option<f64>,
    pub max_error_rate: f64,
}

#[derive(Debug, Clone)]
pub struct InferenceRequest {
    pub id: Uuid,
    pub requester: Address,
    pub model_id: Uuid,
    pub provider: Address,
    pub spec: InferenceRequestSpec,
    pub pricing: RequestPricing,
    pub status: InferenceStatus,
    pub submitted_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<InferenceResult>,
}

#[derive(Debug, Clone)]
pub struct RequestPricing {
    pub base_cost: u64,
    pub urgency_multiplier: f64,
    pub complexity_multiplier: f64,
    pub total_cost: u64,
}

#[derive(Debug, Clone)]
pub enum InferenceStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub output_data: Vec<u8>,
    pub confidence_score: f64,
    pub accuracy: Option<f64>,
    pub processing_time_ms: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    ReputationWeighted,
    LatencyOptimized,
}

#[derive(Debug)]
pub struct PerformanceRecord {
    pub timestamp: DateTime<Utc>,
    pub latency_ms: u64,
    pub confidence_score: f64,
    pub accuracy: f64,
    pub throughput: f64,
}

#[derive(Debug)]
pub struct ReputationUpdate {
    pub score: f64,
    pub feedback: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ValidationProcess {
    pub model_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub status: ValidationStatus,
    pub test_cases_passed: u32,
    pub test_cases_total: u32,
}

#[derive(Debug)]
pub enum ValidationStatus {
    InProgress,
    Passed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplaceStats {
    pub total_models: usize,
    pub active_models: usize,
    pub total_providers: usize,
    pub total_inferences: u64,
    pub total_earnings: u64,
    pub pending_requests: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    #[tokio::test]
    async fn test_model_hosting_initialization() {
        let mut model_hosting = ModelHosting::new();
        let result = model_hosting.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_model_registration() {
        let mut model_hosting = ModelHosting::new();
        model_hosting.initialize().await.unwrap();

        let keypair = SigningKey::from_bytes(&rand::random());
        let provider = Address::from_public_key(&keypair.verifying_key());

        let model_spec = ModelSpec {
            model_name: "GPT-4 Chat".to_string(),
            model_type: ModelType::LanguageModel,
            model_version: "1.0.0".to_string(),
            description: "Advanced language model for chat".to_string(),
            compute_requirements: ComputeRequirements {
                cpu_cores: 8,
                memory_gb: 16,
                requires_gpu: true,
                gpu_memory_gb: Some(24),
                storage_gb: 100,
                network_bandwidth_mbps: 1000,
            },
            supported_input_formats: vec!["text/plain".to_string()],
            supported_output_formats: vec!["text/plain".to_string()],
            availability_zone: Some("us-west-2".to_string()),
            api_endpoint: "https://api.example.com/chat".to_string(),
        };

        let model_id = model_hosting
            .register_model(provider, model_spec)
            .await
            .unwrap();
        assert!(!model_id.to_string().is_empty());

        let stats = model_hosting.get_marketplace_stats();
        assert_eq!(stats.total_models, 1);
        assert_eq!(stats.total_providers, 1);
    }

    #[tokio::test]
    async fn test_inference_request_submission() {
        let mut model_hosting = ModelHosting::new();
        model_hosting.initialize().await.unwrap();

        // First register a model
        let keypair1 = SigningKey::from_bytes(&rand::random());
        let provider = Address::from_public_key(&keypair1.verifying_key());

        let model_spec = ModelSpec {
            model_name: "Image Classifier".to_string(),
            model_type: ModelType::ImageClassification,
            model_version: "1.0.0".to_string(),
            description: "Image classification model".to_string(),
            compute_requirements: ComputeRequirements {
                cpu_cores: 4,
                memory_gb: 8,
                requires_gpu: true,
                gpu_memory_gb: Some(8),
                storage_gb: 50,
                network_bandwidth_mbps: 500,
            },
            supported_input_formats: vec!["image/jpeg".to_string()],
            supported_output_formats: vec!["application/json".to_string()],
            availability_zone: None,
            api_endpoint: "https://api.example.com/classify".to_string(),
        };

        let model_id = model_hosting
            .register_model(provider, model_spec)
            .await
            .unwrap();

        // Manually set model as active for testing
        if let Some(model) = model_hosting.model_registry.get_mut(&model_id) {
            model.status = ModelStatus::Active;
        }

        // Now submit an inference request
        let keypair2 = SigningKey::from_bytes(&rand::random());
        let requester = Address::from_public_key(&keypair2.verifying_key());

        let request_spec = InferenceRequestSpec {
            model_type: ModelType::ImageClassification,
            input_format: "image/jpeg".to_string(),
            output_format: "application/json".to_string(),
            max_latency_ms: 5000,
            estimated_input_size_kb: 500,
            preferred_availability_zone: None,
            quality_requirements: QualityRequirements {
                min_confidence_score: 0.8,
                min_accuracy: Some(0.9),
                max_error_rate: 0.05,
            },
        };

        let request_id = model_hosting
            .submit_inference_request(requester, request_spec)
            .await
            .unwrap();
        assert!(!request_id.to_string().is_empty());

        let stats = model_hosting.get_marketplace_stats();
        assert_eq!(stats.pending_requests, 1);
    }
}
