use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{ApiErrorType, ApiResult},
    models::{
        ApiResponse, PaginatedResponse, PaginationParams,
        CreateTransactionRequest, TransactionResponse, SendTransactionRequest,
    },
    services::ApiServices,
};
use paradigm_core::{Hash, Address};

/// Transaction routes
pub fn router(services: Arc<ApiServices>) -> Router {
    Router::new()
        .route("/transactions", get(list_transactions))
        .route("/transactions", post(create_transaction))
        .route("/transactions/send", post(send_signed_transaction))
        .route("/transactions/:hash", get(get_transaction))
        .route("/transactions/:hash/receipt", get(get_transaction_receipt))
        .route("/transactions/estimate-fee", post(estimate_transaction_fee))
        .route("/transactions/batch", post(create_batch_transaction))
        .route("/addresses/:address/transactions", get(get_address_transactions))
        .route("/addresses/:address/nonce", get(get_address_nonce))
        .with_state(services)
}

/// List recent transactions
async fn list_transactions(
    State(services): State<Arc<ApiServices>>,
    Query(pagination): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<PaginatedResponse<TransactionResponse>>>> {
    pagination.validate()
        .map_err(|e| ApiErrorType::ValidationFailed { 
            field: e.field_errors().keys().next().unwrap_or(&"unknown").to_string() 
        })?;

    let page = pagination.page.unwrap_or(1);
    let page_size = pagination.page_size.unwrap_or(20);

    // Mock transactions
    let transactions = vec![
        TransactionResponse {
            hash: [1u8; 32],
            from: Address([2u8; 32]),
            to: Address([3u8; 32]),
            amount: 1000_00000000, // 1000 PAR
            fee: 1_00000000,       // 1 PAR
            gas_used: Some(21000),
            gas_price: Some(20),
            nonce: 42,
            block_hash: Some([4u8; 32]),
            block_height: Some(12345),
            transaction_index: Some(0),
            status: crate::models::TransactionStatus::Confirmed,
            timestamp: chrono::Utc::now(),
            confirmations: 6,
            data: None,
        },
        TransactionResponse {
            hash: [5u8; 32],
            from: Address([6u8; 32]),
            to: Address([7u8; 32]),
            amount: 500_00000000,  // 500 PAR
            fee: 1_00000000,       // 1 PAR
            gas_used: Some(25000),
            gas_price: Some(20),
            nonce: 43,
            block_hash: None,
            block_height: None,
            transaction_index: None,
            status: crate::models::TransactionStatus::Pending,
            timestamp: chrono::Utc::now(),
            confirmations: 0,
            data: Some(vec![0x12, 0x34, 0x56]),
        },
    ];

    let total_count = 1000; // Mock total
    let total_pages = (total_count + page_size - 1) / page_size;

    let paginated = PaginatedResponse {
        items: transactions,
        total_count: total_count as u64,
        page,
        page_size,
        total_pages,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    Ok(Json(ApiResponse::success(paginated)))
}

/// Create a new transaction
async fn create_transaction(
    State(services): State<Arc<ApiServices>>,
    Json(request): Json<CreateTransactionRequest>,
) -> ApiResult<Json<ApiResponse<TransactionResponse>>> {
    request.validate()
        .map_err(|e| ApiErrorType::ValidationFailed { 
            field: e.field_errors().keys().next().unwrap_or(&"unknown").to_string() 
        })?;

    let transaction = services.blockchain_service()
        .create_transaction(request)
        .await?;

    Ok(Json(ApiResponse::success(transaction)))
}

/// Send a signed transaction
async fn send_signed_transaction(
    State(services): State<Arc<ApiServices>>,
    Json(request): Json<SendTransactionRequest>,
) -> ApiResult<Json<ApiResponse<TransactionResponse>>> {
    let transaction = services.blockchain_service()
        .send_signed_transaction(&request.signed_transaction)
        .await?;

    Ok(Json(ApiResponse::success(transaction)))
}

/// Get transaction by hash
async fn get_transaction(
    State(services): State<Arc<ApiServices>>,
    Path(hash): Path<String>,
) -> ApiResult<Json<ApiResponse<TransactionResponse>>> {
    let hash_bytes = hex::decode(&hash)
        .map_err(|_| ApiErrorType::InvalidRequest { 
            message: "Invalid transaction hash format".to_string() 
        })?;

    if hash_bytes.len() != 32 {
        return Err(ApiErrorType::InvalidRequest { 
            message: "Transaction hash must be 32 bytes".to_string() 
        });
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes);

    let transaction = services.blockchain_service()
        .get_transaction(&hash_array)
        .await?
        .ok_or(ApiErrorType::NotFound { 
            resource: "Transaction".to_string() 
        })?;

    Ok(Json(ApiResponse::success(transaction)))
}

/// Get transaction receipt
async fn get_transaction_receipt(
    State(services): State<Arc<ApiServices>>,
    Path(hash): Path<String>,
) -> ApiResult<Json<ApiResponse<TransactionReceipt>>> {
    let hash_bytes = hex::decode(&hash)
        .map_err(|_| ApiErrorType::InvalidRequest { 
            message: "Invalid transaction hash format".to_string() 
        })?;

    if hash_bytes.len() != 32 {
        return Err(ApiErrorType::InvalidRequest { 
            message: "Transaction hash must be 32 bytes".to_string() 
        });
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes);

    let receipt = services.blockchain_service()
        .get_transaction_receipt(&hash_array)
        .await?
        .ok_or(ApiErrorType::NotFound { 
            resource: "Transaction receipt".to_string() 
        })?;

    Ok(Json(ApiResponse::success(receipt)))
}

/// Estimate transaction fee
async fn estimate_transaction_fee(
    State(services): State<Arc<ApiServices>>,
    Json(request): Json<CreateTransactionRequest>,
) -> ApiResult<Json<ApiResponse<FeeEstimate>>> {
    request.validate()
        .map_err(|e| ApiErrorType::ValidationFailed { 
            field: e.field_errors().keys().next().unwrap_or(&"unknown").to_string() 
        })?;

    let estimate = services.blockchain_service()
        .estimate_transaction_fee(&request)
        .await?;

    Ok(Json(ApiResponse::success(estimate)))
}

/// Create batch transaction
async fn create_batch_transaction(
    State(services): State<Arc<ApiServices>>,
    Json(requests): Json<Vec<CreateTransactionRequest>>,
) -> ApiResult<Json<ApiResponse<BatchTransactionResponse>>> {
    // Validate all requests
    for request in &requests {
        request.validate()
            .map_err(|e| ApiErrorType::ValidationFailed { 
                field: e.field_errors().keys().next().unwrap_or(&"unknown").to_string() 
            })?;
    }

    if requests.len() > 100 {
        return Err(ApiErrorType::InvalidRequest { 
            message: "Batch size cannot exceed 100 transactions".to_string() 
        });
    }

    let batch_response = services.blockchain_service()
        .create_batch_transaction(requests)
        .await?;

    Ok(Json(ApiResponse::success(batch_response)))
}

/// Get transactions for a specific address
async fn get_address_transactions(
    State(services): State<Arc<ApiServices>>,
    Path(address): Path<String>,
    Query(pagination): Query<AddressTransactionParams>,
) -> ApiResult<Json<ApiResponse<PaginatedResponse<TransactionResponse>>>> {
    pagination.validate()
        .map_err(|e| ApiErrorType::ValidationFailed { 
            field: e.field_errors().keys().next().unwrap_or(&"unknown").to_string() 
        })?;

    // Parse address
    let address_bytes = if address.starts_with("PAR") {
        // Parse Paradigm address format
        hex::decode(&address[3..])
            .map_err(|_| ApiErrorType::InvalidAddress { 
                address: address.clone() 
            })?
    } else {
        return Err(ApiErrorType::InvalidAddress { 
            address: address.clone() 
        });
    };

    if address_bytes.len() != 20 {
        return Err(ApiErrorType::InvalidAddress { 
            address: address.clone() 
        });
    }

    let mut addr_array = [0u8; 32];
    addr_array[..20].copy_from_slice(&address_bytes);
    let addr = Address(addr_array);

    let transactions = services.blockchain_service()
        .get_address_transactions(&addr, &pagination)
        .await?;

    Ok(Json(ApiResponse::success(transactions)))
}

/// Get nonce for an address
async fn get_address_nonce(
    State(services): State<Arc<ApiServices>>,
    Path(address): Path<String>,
) -> ApiResult<Json<ApiResponse<NonceResponse>>> {
    // Parse address (similar to above)
    let address_bytes = if address.starts_with("PAR") {
        hex::decode(&address[3..])
            .map_err(|_| ApiErrorType::InvalidAddress { 
                address: address.clone() 
            })?
    } else {
        return Err(ApiErrorType::InvalidAddress { 
            address: address.clone() 
        });
    };

    let mut addr_array = [0u8; 32];
    addr_array[..20].copy_from_slice(&address_bytes);
    let addr = Address(addr_array);

    let nonce = services.blockchain_service()
        .get_address_nonce(&addr)
        .await?;

    Ok(Json(ApiResponse::success(NonceResponse {
        address: addr,
        nonce,
        pending_nonce: nonce + 1, // Mock pending nonce
    })))
}

// Supporting types
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TransactionReceipt {
    pub transaction_hash: Hash,
    pub block_hash: Hash,
    pub block_height: u64,
    pub transaction_index: u32,
    pub gas_used: u64,
    pub gas_price: u64,
    pub status: bool, // true for success, false for failure
    pub logs: Vec<TransactionLog>,
    pub cumulative_gas_used: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TransactionLog {
    pub address: Address,
    pub topics: Vec<Hash>,
    pub data: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FeeEstimate {
    pub gas_estimate: u64,
    pub gas_price: u64,
    pub total_fee: u64,
    pub confidence: f64, // 0.0 to 1.0
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BatchTransactionResponse {
    pub batch_id: Uuid,
    pub transactions: Vec<TransactionResponse>,
    pub total_fee: u64,
    pub status: BatchStatus,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum BatchStatus {
    Pending,
    Submitted,
    Confirmed,
    PartiallyFailed,
    Failed,
}

#[derive(serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct AddressTransactionParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub transaction_type: Option<String>, // "sent", "received", "all"
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NonceResponse {
    pub address: Address,
    pub nonce: u64,
    pub pending_nonce: u64,
}