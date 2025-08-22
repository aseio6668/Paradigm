use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;
use paradigm_core::{Address, Hash, Amount};

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: DateTime<Utc>,
    pub request_id: Uuid,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            request_id: Uuid::new_v4(),
        }
    }
    
    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
            request_id: Uuid::new_v4(),
        }
    }
}

/// API error details
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Paginated response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Pagination parameters
#[derive(Debug, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, max = 1000))]
    pub page_size: Option<u32>,
    #[validate(range(min = 1))]
    pub page: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page_size: Some(20),
            page: Some(1),
        }
    }
}

// Authentication Models

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user: UserProfile,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 2))]
    pub name: String,
    pub organization: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub organization: Option<String>,
    pub role: UserRole,
    pub api_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserRole {
    User,
    Developer,
    Enterprise,
    Admin,
}

// Blockchain Models

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockResponse {
    pub hash: Hash,
    pub height: u64,
    pub timestamp: DateTime<Utc>,
    pub parent_hash: Hash,
    pub transaction_count: u32,
    pub transactions: Vec<TransactionSummary>,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub hash: Hash,
    pub from: Address,
    pub to: Address,
    pub amount: Amount,
    pub fee: Amount,
    pub status: TransactionStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

// Transaction Models

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateTransactionRequest {
    pub to: Address,
    #[validate(range(min = 1))]
    pub amount: Amount,
    pub data: Option<Vec<u8>>,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<u64>,
    pub nonce: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub hash: Hash,
    pub from: Address,
    pub to: Address,
    pub amount: Amount,
    pub fee: Amount,
    pub gas_used: Option<u64>,
    pub gas_price: Option<u64>,
    pub nonce: u64,
    pub block_hash: Option<Hash>,
    pub block_height: Option<u64>,
    pub transaction_index: Option<u32>,
    pub status: TransactionStatus,
    pub timestamp: DateTime<Utc>,
    pub confirmations: u32,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SendTransactionRequest {
    pub signed_transaction: String, // Hex-encoded signed transaction
}

// Account Models

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResponse {
    pub address: Address,
    pub balance: Amount,
    pub nonce: u64,
    pub transaction_count: u64,
    pub created_at: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub address: Address,
    pub balance: Amount,
    pub pending_balance: Amount,
    pub locked_balance: Amount,
    pub block_height: u64,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAccountRequest {
    pub name: Option<String>,
    pub initial_balance: Option<Amount>,
}

// ML Task Models

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct MLTaskRequest {
    #[validate(length(min = 1))]
    pub task_type: String,
    pub parameters: serde_json::Value,
    pub data_source: Option<String>,
    pub priority: Option<TaskPriority>,
    #[validate(range(min = 1, max = 10))]
    pub difficulty: Option<u32>,
    pub reward: Option<Amount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MLTaskResponse {
    pub task_id: Uuid,
    pub task_type: String,
    pub status: TaskStatus,
    pub progress: f64,
    pub result: Option<serde_json::Value>,
    pub assigned_node: Option<Address>,
    pub reward: Amount,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

// Cross-Chain Models

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CrossChainTransferRequest {
    pub from_chain: String,
    pub to_chain: String,
    pub asset: String,
    #[validate(range(min = 1))]
    pub amount: Amount,
    pub recipient: String, // Address on destination chain
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossChainResponse {
    pub transfer_id: Uuid,
    pub from_chain: String,
    pub to_chain: String,
    pub asset: String,
    pub amount: Amount,
    pub recipient: String,
    pub status: CrossChainStatus,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub confirmations: u32,
    pub required_confirmations: u32,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CrossChainStatus {
    Initiated,
    Pending,
    Confirmed,
    Completed,
    Failed,
    Cancelled,
}

// Governance Models

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateProposalRequest {
    #[validate(length(min = 10, max = 200))]
    pub title: String,
    #[validate(length(min = 50, max = 5000))]
    pub description: String,
    pub proposal_type: ProposalType,
    pub voting_period_hours: Option<u64>,
    #[validate(range(min = 1000))]
    pub initial_deposit: Amount,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProposalType {
    ParameterChange { parameter: String, new_value: String },
    ProtocolUpgrade { version: String },
    TreasurySpending { recipient: Address, amount: Amount, purpose: String },
    Other { details: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProposalResponse {
    pub proposal_id: Uuid,
    pub title: String,
    pub description: String,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub voting_start: DateTime<Utc>,
    pub voting_end: DateTime<Utc>,
    pub yes_votes: Amount,
    pub no_votes: Amount,
    pub abstain_votes: Amount,
    pub total_votes: Amount,
    pub quorum: f64,
    pub pass_threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct VoteRequest {
    pub proposal_id: Uuid,
    pub option: VoteOption,
    pub voting_power: Option<Amount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
    NoWithVeto,
}

// Analytics Models

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatsResponse {
    pub total_transactions: u64,
    pub total_accounts: u64,
    pub current_block_height: u64,
    pub average_block_time: f64,
    pub transactions_per_second: f64,
    pub active_validators: u32,
    pub total_staked: Amount,
    pub market_cap: Option<f64>,
    pub price_usd: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionStatsRequest {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub granularity: Option<String>, // hour, day, week, month
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionStatsResponse {
    pub period: String,
    pub transaction_count: u64,
    pub total_volume: Amount,
    pub average_fee: Amount,
    pub unique_addresses: u64,
}

// Webhook Models

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateWebhookRequest {
    #[validate(url)]
    pub url: String,
    pub events: Vec<WebhookEvent>,
    pub secret: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WebhookEvent {
    TransactionConfirmed,
    BlockCreated,
    MLTaskCompleted,
    ProposalCreated,
    CrossChainTransfer,
    AccountCreated,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub webhook_id: Uuid,
    pub url: String,
    pub events: Vec<WebhookEvent>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub success_count: u64,
    pub failure_count: u64,
}

// WebSocket Models

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: WebSocketMessageType,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WebSocketMessageType {
    BlockUpdate,
    TransactionUpdate,
    BalanceUpdate,
    MLTaskUpdate,
    ProposalUpdate,
    PriceUpdate,
    SystemAlert,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    pub subscription_type: SubscriptionType,
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SubscriptionType {
    Blocks,
    Transactions { address: Option<Address> },
    Balances { address: Address },
    MLTasks,
    Proposals,
    Prices { assets: Vec<String> },
    All,
}