//! Smart contract interaction and management
//!
//! This module provides tools for deploying, interacting with, and managing
//! smart contracts on the Paradigm blockchain.

use crate::client::{BlockNumber, ParadigmClient};
use crate::error::{ErrorExt, ParadigmError, Result};
use crate::types::*;
use crate::wallet::{TransactionBuilder, Wallet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Contract ABI function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiFunction {
    /// Function name
    pub name: String,
    /// Function inputs
    pub inputs: Vec<AbiParameter>,
    /// Function outputs
    pub outputs: Vec<AbiParameter>,
    /// Function state mutability
    pub state_mutability: StateMutability,
    /// Function type
    pub function_type: FunctionType,
    /// Whether function is payable
    pub payable: bool,
}

/// Contract ABI parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Internal type
    pub internal_type: Option<String>,
    /// Components for complex types
    pub components: Option<Vec<AbiParameter>>,
}

/// Function state mutability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StateMutability {
    Pure,
    View,
    NonPayable,
    Payable,
}

/// Function type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FunctionType {
    Function,
    Constructor,
    Receive,
    Fallback,
}

/// Contract ABI event definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiEvent {
    /// Event name
    pub name: String,
    /// Event inputs
    pub inputs: Vec<AbiEventInput>,
    /// Whether event is anonymous
    pub anonymous: bool,
}

/// Contract ABI event input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiEventInput {
    /// Input name
    pub name: String,
    /// Input type
    pub param_type: String,
    /// Whether input is indexed
    pub indexed: bool,
    /// Internal type
    pub internal_type: Option<String>,
}

/// Complete contract ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAbi {
    /// Contract functions
    pub functions: HashMap<String, AbiFunction>,
    /// Contract events
    pub events: HashMap<String, AbiEvent>,
    /// Contract constructor
    pub constructor: Option<AbiFunction>,
    /// Receive function
    pub receive: Option<AbiFunction>,
    /// Fallback function
    pub fallback: Option<AbiFunction>,
}

impl ContractAbi {
    /// Create empty ABI
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            events: HashMap::new(),
            constructor: None,
            receive: None,
            fallback: None,
        }
    }

    /// Load ABI from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        let abi_items: Vec<serde_json::Value> = serde_json::from_str(json)
            .map_err(|e| ParadigmError::Abi(format!("Invalid ABI JSON: {}", e)))?;

        let mut abi = Self::new();

        for item in abi_items {
            match item.get("type").and_then(|t| t.as_str()) {
                Some("function") => {
                    let function: AbiFunction = serde_json::from_value(item)
                        .map_err(|e| ParadigmError::Abi(format!("Invalid function ABI: {}", e)))?;
                    abi.functions.insert(function.name.clone(), function);
                }
                Some("event") => {
                    let event: AbiEvent = serde_json::from_value(item)
                        .map_err(|e| ParadigmError::Abi(format!("Invalid event ABI: {}", e)))?;
                    abi.events.insert(event.name.clone(), event);
                }
                Some("constructor") => {
                    let constructor: AbiFunction = serde_json::from_value(item).map_err(|e| {
                        ParadigmError::Abi(format!("Invalid constructor ABI: {}", e))
                    })?;
                    abi.constructor = Some(constructor);
                }
                Some("receive") => {
                    let receive: AbiFunction = serde_json::from_value(item)
                        .map_err(|e| ParadigmError::Abi(format!("Invalid receive ABI: {}", e)))?;
                    abi.receive = Some(receive);
                }
                Some("fallback") => {
                    let fallback: AbiFunction = serde_json::from_value(item)
                        .map_err(|e| ParadigmError::Abi(format!("Invalid fallback ABI: {}", e)))?;
                    abi.fallback = Some(fallback);
                }
                _ => {} // Skip unknown types
            }
        }

        Ok(abi)
    }

    /// Get function by name
    pub fn get_function(&self, name: &str) -> Option<&AbiFunction> {
        self.functions.get(name)
    }

    /// Get event by name
    pub fn get_event(&self, name: &str) -> Option<&AbiEvent> {
        self.events.get(name)
    }

    /// List all function names
    pub fn function_names(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }

    /// List all event names
    pub fn event_names(&self) -> Vec<&String> {
        self.events.keys().collect()
    }
}

impl Default for ContractAbi {
    fn default() -> Self {
        Self::new()
    }
}

/// Contract bytecode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractBytecode {
    /// Deployment bytecode
    pub bytecode: Vec<u8>,
    /// Runtime bytecode
    pub runtime_bytecode: Vec<u8>,
    /// Source map
    pub source_map: Option<String>,
    /// Deployed source map
    pub deployed_source_map: Option<String>,
}

impl ContractBytecode {
    /// Create new bytecode
    pub fn new(bytecode: Vec<u8>) -> Self {
        Self {
            bytecode,
            runtime_bytecode: Vec::new(),
            source_map: None,
            deployed_source_map: None,
        }
    }

    /// Load bytecode from hex string
    pub fn from_hex(hex: &str) -> Result<Self> {
        let bytecode = hex::decode(hex.trim_start_matches("0x"))
            .map_err(|e| ParadigmError::InvalidHex(format!("Invalid bytecode hex: {}", e)))?;

        Ok(Self::new(bytecode))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.bytecode))
    }
}

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    /// Contract name
    pub name: String,
    /// Contract version
    pub version: String,
    /// Compiler version
    pub compiler_version: String,
    /// Source code hash
    pub source_hash: Option<Hash>,
    /// License
    pub license: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: SystemTime,
}

impl ContractMetadata {
    /// Create new metadata
    pub fn new(name: String, version: String, compiler_version: String) -> Self {
        Self {
            name,
            version,
            compiler_version,
            source_hash: None,
            license: None,
            author: None,
            description: None,
            tags: Vec::new(),
            created_at: SystemTime::now(),
        }
    }
}

/// Contract deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeployment {
    /// Deployment transaction hash
    pub transaction_hash: Hash,
    /// Contract address
    pub address: Address,
    /// Block number where deployed
    pub block_number: u64,
    /// Gas used for deployment
    pub gas_used: u64,
    /// Deployment timestamp
    pub deployed_at: SystemTime,
    /// Deployer address
    pub deployer: Address,
}

/// Smart contract representation
#[derive(Debug, Clone)]
pub struct Contract {
    /// Contract ID
    pub id: Uuid,
    /// Contract address
    pub address: Address,
    /// Contract ABI
    pub abi: ContractAbi,
    /// Contract bytecode
    pub bytecode: Option<ContractBytecode>,
    /// Contract metadata
    pub metadata: ContractMetadata,
    /// Deployment information
    pub deployment: Option<ContractDeployment>,
    /// Client for blockchain interaction
    client: Arc<ParadigmClient>,
}

impl Contract {
    /// Create a new contract instance
    pub fn new(
        address: Address,
        abi: ContractAbi,
        client: Arc<ParadigmClient>,
        metadata: ContractMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            address,
            abi,
            bytecode: None,
            metadata,
            deployment: None,
            client,
        }
    }

    /// Create contract from deployment
    pub fn from_deployment(
        deployment: ContractDeployment,
        abi: ContractAbi,
        bytecode: ContractBytecode,
        client: Arc<ParadigmClient>,
        metadata: ContractMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            address: deployment.address.clone(),
            abi,
            bytecode: Some(bytecode),
            metadata,
            deployment: Some(deployment),
            client,
        }
    }

    /// Get contract address
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Get contract ABI
    pub fn abi(&self) -> &ContractAbi {
        &self.abi
    }

    /// Create a function call
    pub fn function(&self, name: &str) -> Result<ContractCall> {
        let function = self
            .abi
            .get_function(name)
            .ok_or_else(|| ParadigmError::Contract(format!("Function '{}' not found", name)))?;

        Ok(ContractCall::new(
            self.address.clone(),
            function.clone(),
            self.client.clone(),
        ))
    }

    /// Get contract code
    pub async fn get_code(&self) -> Result<Vec<u8>> {
        // This would need to be implemented in the client
        // For now, return empty vector
        Ok(Vec::new())
    }

    /// Check if contract exists at address
    pub async fn exists(&self) -> Result<bool> {
        let code = self.get_code().await?;
        Ok(!code.is_empty())
    }

    /// Get contract storage at slot
    pub async fn get_storage(&self, slot: &Hash) -> Result<Hash> {
        // This would need to be implemented in the client
        Ok(Hash::default())
    }

    /// Parse event logs from transaction receipt
    pub fn parse_logs(&self, receipt: &TransactionReceipt) -> Result<Vec<ParsedEvent>> {
        let mut parsed_events = Vec::new();

        for log in &receipt.logs {
            if log.address == self.address {
                for (_, event) in &self.abi.events {
                    if let Ok(parsed) = self.parse_log(log, event) {
                        parsed_events.push(parsed);
                        break;
                    }
                }
            }
        }

        Ok(parsed_events)
    }

    /// Parse a single log entry
    fn parse_log(&self, log: &Log, event: &AbiEvent) -> Result<ParsedEvent> {
        // Simplified log parsing - real implementation would decode based on ABI
        Ok(ParsedEvent {
            event_name: event.name.clone(),
            address: log.address.clone(),
            topics: log.topics.clone(),
            data: log.data.clone(),
            block_number: log.block_number,
            transaction_hash: log.transaction_hash.clone(),
            log_index: log.log_index,
        })
    }
}

/// Parsed contract event
#[derive(Debug, Clone)]
pub struct ParsedEvent {
    pub event_name: String,
    pub address: Address,
    pub topics: Vec<Hash>,
    pub data: Vec<u8>,
    pub block_number: u64,
    pub transaction_hash: Hash,
    pub log_index: u64,
}

/// Contract function call builder
#[derive(Debug)]
pub struct ContractCall {
    /// Contract address
    address: Address,
    /// Function definition
    function: AbiFunction,
    /// Function parameters
    params: Vec<serde_json::Value>,
    /// Client for blockchain interaction
    client: Arc<ParadigmClient>,
    /// Gas limit override
    gas_limit: Option<u64>,
    /// Gas price override
    gas_price: Option<Amount>,
    /// Value to send with call
    value: Amount,
}

impl ContractCall {
    /// Create a new contract call
    pub fn new(address: Address, function: AbiFunction, client: Arc<ParadigmClient>) -> Self {
        Self {
            address,
            function,
            params: Vec::new(),
            client,
            gas_limit: None,
            gas_price: None,
            value: Amount::zero(),
        }
    }

    /// Add parameter to function call
    pub fn param<T: Serialize>(mut self, value: T) -> Result<Self> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| ParadigmError::Abi(format!("Failed to serialize parameter: {}", e)))?;
        self.params.push(json_value);
        Ok(self)
    }

    /// Set gas limit for call
    pub fn gas(mut self, gas: u64) -> Self {
        self.gas_limit = Some(gas);
        self
    }

    /// Set gas price for call
    pub fn gas_price(mut self, price: Amount) -> Self {
        self.gas_price = Some(price);
        self
    }

    /// Set value to send with call (for payable functions)
    pub fn value(mut self, value: Amount) -> Self {
        self.value = value;
        self
    }

    /// Execute call as a view function (no transaction)
    pub async fn call(&self) -> Result<serde_json::Value> {
        if self.function.state_mutability != StateMutability::View
            && self.function.state_mutability != StateMutability::Pure
        {
            return Err(ParadigmError::Contract(
                "Function is not a view function - use send() for state-changing calls".to_string(),
            ));
        }

        let call_data = self.encode_call_data()?;

        // Create call transaction for simulation
        let call_tx = Transaction {
            to: Some(self.address.clone()),
            input: call_data,
            value: Amount::zero(),
            gas: self.gas_limit.unwrap_or(1000000),
            ..Default::default()
        };

        // This would need eth_call implementation in client
        // For now, return mock response
        Ok(serde_json::Value::String("0x".to_string()))
    }

    /// Execute call as a transaction (state-changing)
    pub async fn send(&self, wallet: &Wallet, account_id: &Uuid) -> Result<Hash> {
        if self.function.state_mutability == StateMutability::View
            || self.function.state_mutability == StateMutability::Pure
        {
            return Err(ParadigmError::Contract(
                "Function is a view function - use call() for read-only calls".to_string(),
            ));
        }

        let call_data = self.encode_call_data()?;

        let mut tx_builder = TransactionBuilder::new()
            .with_client(self.client.clone())
            .to(self.address.clone())
            .data(call_data)
            .value(self.value.clone());

        if let Some(gas) = self.gas_limit {
            tx_builder = tx_builder.gas(gas);
        }

        if let Some(gas_price) = &self.gas_price {
            tx_builder = tx_builder.gas_price(gas_price.clone());
        }

        let signed_tx = tx_builder.build_and_sign(wallet, account_id).await?;
        let tx_data = signed_tx.to_bytes()?;

        self.client.send_raw_transaction(&tx_data).await
    }

    /// Estimate gas for the function call
    pub async fn estimate_gas(&self) -> Result<u64> {
        let call_data = self.encode_call_data()?;

        let tx = Transaction {
            to: Some(self.address.clone()),
            input: call_data,
            value: self.value.clone(),
            gas: 1000000, // High limit for estimation
            ..Default::default()
        };

        self.client.estimate_gas(&tx).await
    }

    /// Encode function call data
    fn encode_call_data(&self) -> Result<Vec<u8>> {
        // Simplified encoding - real implementation would use proper ABI encoding
        let mut data = Vec::new();

        // Function selector (first 4 bytes of keccak256 hash of function signature)
        let signature = self.create_function_signature();
        let mut hasher = sha2::Sha256::new();
        hasher.update(signature.as_bytes());
        let hash = hasher.finalize();
        data.extend_from_slice(&hash[..4]);

        // Encode parameters (simplified)
        for param in &self.params {
            let param_bytes = self.encode_parameter(param)?;
            data.extend_from_slice(&param_bytes);
        }

        Ok(data)
    }

    /// Create function signature string
    fn create_function_signature(&self) -> String {
        let param_types: Vec<String> = self
            .function
            .inputs
            .iter()
            .map(|param| param.param_type.clone())
            .collect();

        format!("{}({})", self.function.name, param_types.join(","))
    }

    /// Encode a single parameter (simplified)
    fn encode_parameter(&self, param: &serde_json::Value) -> Result<Vec<u8>> {
        // Simplified parameter encoding
        match param {
            serde_json::Value::String(s) => {
                if s.starts_with("0x") {
                    hex::decode(&s[2..])
                        .map_err(|e| ParadigmError::Abi(format!("Invalid hex parameter: {}", e)))
                } else {
                    Ok(s.as_bytes().to_vec())
                }
            }
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_u64() {
                    Ok(i.to_be_bytes().to_vec())
                } else {
                    Err(ParadigmError::Abi("Invalid number parameter".to_string()))
                }
            }
            serde_json::Value::Bool(b) => Ok(vec![if *b { 1 } else { 0 }]),
            _ => Err(ParadigmError::Abi("Unsupported parameter type".to_string())),
        }
    }
}

/// Contract builder for deploying new contracts
#[derive(Debug)]
pub struct ContractBuilder {
    /// Contract bytecode
    bytecode: ContractBytecode,
    /// Contract ABI
    abi: ContractAbi,
    /// Constructor parameters
    constructor_params: Vec<serde_json::Value>,
    /// Contract metadata
    metadata: ContractMetadata,
    /// Client for deployment
    client: Arc<ParadigmClient>,
    /// Gas limit override
    gas_limit: Option<u64>,
    /// Gas price override
    gas_price: Option<Amount>,
    /// Value to send with deployment
    value: Amount,
}

impl ContractBuilder {
    /// Create a new contract builder
    pub fn new(
        bytecode: ContractBytecode,
        abi: ContractAbi,
        metadata: ContractMetadata,
        client: Arc<ParadigmClient>,
    ) -> Self {
        Self {
            bytecode,
            abi,
            constructor_params: Vec::new(),
            metadata,
            client,
            gas_limit: None,
            gas_price: None,
            value: Amount::zero(),
        }
    }

    /// Add constructor parameter
    pub fn constructor_param<T: Serialize>(mut self, value: T) -> Result<Self> {
        let json_value = serde_json::to_value(value).map_err(|e| {
            ParadigmError::Abi(format!("Failed to serialize constructor parameter: {}", e))
        })?;
        self.constructor_params.push(json_value);
        Ok(self)
    }

    /// Set gas limit for deployment
    pub fn gas(mut self, gas: u64) -> Self {
        self.gas_limit = Some(gas);
        self
    }

    /// Set gas price for deployment
    pub fn gas_price(mut self, price: Amount) -> Self {
        self.gas_price = Some(price);
        self
    }

    /// Set value to send with deployment
    pub fn value(mut self, value: Amount) -> Self {
        self.value = value;
        self
    }

    /// Deploy the contract
    pub async fn deploy(&self, wallet: &Wallet, account_id: &Uuid) -> Result<Contract> {
        let deployment_data = self.create_deployment_data()?;

        let mut tx_builder = TransactionBuilder::new()
            .with_client(self.client.clone())
            .data(deployment_data)
            .value(self.value.clone());

        if let Some(gas) = self.gas_limit {
            tx_builder = tx_builder.gas(gas);
        }

        if let Some(gas_price) = &self.gas_price {
            tx_builder = tx_builder.gas_price(gas_price.clone());
        }

        let signed_tx = tx_builder.build_and_sign(wallet, account_id).await?;
        let tx_data = signed_tx.to_bytes()?;

        let tx_hash = self.client.send_raw_transaction(&tx_data).await?;

        // Wait for deployment confirmation
        let receipt = self.client.wait_for_confirmation(&tx_hash, 1).await?;

        let contract_address = receipt
            .contract_address
            .ok_or_else(|| ParadigmError::Contract("No contract address in receipt".to_string()))?;

        let deployment = ContractDeployment {
            transaction_hash: tx_hash,
            address: contract_address.clone(),
            block_number: receipt.block_number,
            gas_used: receipt.gas_used,
            deployed_at: SystemTime::now(),
            deployer: signed_tx.from,
        };

        Ok(Contract::from_deployment(
            deployment,
            self.abi.clone(),
            self.bytecode.clone(),
            self.client.clone(),
            self.metadata.clone(),
        ))
    }

    /// Estimate gas for deployment
    pub async fn estimate_gas(&self) -> Result<u64> {
        let deployment_data = self.create_deployment_data()?;

        let tx = Transaction {
            to: None, // Deployment transaction
            input: deployment_data,
            value: self.value.clone(),
            gas: 10000000, // High limit for estimation
            ..Default::default()
        };

        self.client.estimate_gas(&tx).await
    }

    /// Create deployment transaction data
    fn create_deployment_data(&self) -> Result<Vec<u8>> {
        let mut data = self.bytecode.bytecode.clone();

        // Encode constructor parameters if any
        if !self.constructor_params.is_empty() {
            if let Some(constructor) = &self.abi.constructor {
                let encoded_params = self.encode_constructor_params(constructor)?;
                data.extend_from_slice(&encoded_params);
            }
        }

        Ok(data)
    }

    /// Encode constructor parameters
    fn encode_constructor_params(&self, constructor: &AbiFunction) -> Result<Vec<u8>> {
        // Simplified constructor parameter encoding
        let mut encoded = Vec::new();

        for param in &self.constructor_params {
            let param_bytes = self.encode_parameter(param)?;
            encoded.extend_from_slice(&param_bytes);
        }

        Ok(encoded)
    }

    /// Encode a single parameter (simplified)
    fn encode_parameter(&self, param: &serde_json::Value) -> Result<Vec<u8>> {
        // Simplified parameter encoding (same as ContractCall)
        match param {
            serde_json::Value::String(s) => {
                if s.starts_with("0x") {
                    hex::decode(&s[2..])
                        .map_err(|e| ParadigmError::Abi(format!("Invalid hex parameter: {}", e)))
                } else {
                    Ok(s.as_bytes().to_vec())
                }
            }
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_u64() {
                    Ok(i.to_be_bytes().to_vec())
                } else {
                    Err(ParadigmError::Abi("Invalid number parameter".to_string()))
                }
            }
            serde_json::Value::Bool(b) => Ok(vec![if *b { 1 } else { 0 }]),
            _ => Err(ParadigmError::Abi("Unsupported parameter type".to_string())),
        }
    }
}

/// Contract manager for organizing multiple contracts
#[derive(Debug)]
pub struct ContractManager {
    contracts: Arc<RwLock<HashMap<Uuid, Contract>>>,
    contracts_by_address: Arc<RwLock<HashMap<Address, Uuid>>>,
    contract_registry: Arc<RwLock<HashMap<String, Vec<Uuid>>>>, // Name -> Contract IDs
}

impl ContractManager {
    /// Create a new contract manager
    pub fn new() -> Self {
        Self {
            contracts: Arc::new(RwLock::new(HashMap::new())),
            contracts_by_address: Arc::new(RwLock::new(HashMap::new())),
            contract_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add contract to manager
    pub async fn add_contract(&self, contract: Contract) -> Uuid {
        let contract_id = contract.id;
        let contract_address = contract.address.clone();
        let contract_name = contract.metadata.name.clone();

        // Store contract
        self.contracts.write().await.insert(contract_id, contract);

        // Index by address
        self.contracts_by_address
            .write()
            .await
            .insert(contract_address, contract_id);

        // Index by name
        self.contract_registry
            .write()
            .await
            .entry(contract_name)
            .or_insert_with(Vec::new)
            .push(contract_id);

        contract_id
    }

    /// Get contract by ID
    pub async fn get_contract(&self, id: &Uuid) -> Option<Contract> {
        self.contracts.read().await.get(id).cloned()
    }

    /// Get contract by address
    pub async fn get_contract_by_address(&self, address: &Address) -> Option<Contract> {
        let contracts_by_address = self.contracts_by_address.read().await;
        if let Some(id) = contracts_by_address.get(address) {
            self.contracts.read().await.get(id).cloned()
        } else {
            None
        }
    }

    /// Get contracts by name
    pub async fn get_contracts_by_name(&self, name: &str) -> Vec<Contract> {
        let registry = self.contract_registry.read().await;
        if let Some(ids) = registry.get(name) {
            let contracts = self.contracts.read().await;
            ids.iter()
                .filter_map(|id| contracts.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// List all contracts
    pub async fn list_contracts(&self) -> Vec<Contract> {
        self.contracts.read().await.values().cloned().collect()
    }

    /// Remove contract
    pub async fn remove_contract(&self, id: &Uuid) -> Option<Contract> {
        let contracts = &mut *self.contracts.write().await;
        if let Some(contract) = contracts.remove(id) {
            // Remove from address index
            self.contracts_by_address
                .write()
                .await
                .remove(&contract.address);

            // Remove from name registry
            let mut registry = self.contract_registry.write().await;
            if let Some(ids) = registry.get_mut(&contract.metadata.name) {
                ids.retain(|&contract_id| contract_id != *id);
                if ids.is_empty() {
                    registry.remove(&contract.metadata.name);
                }
            }

            Some(contract)
        } else {
            None
        }
    }

    /// Get contract statistics
    pub async fn get_statistics(&self) -> ContractManagerStats {
        let contracts = self.contracts.read().await;
        let registry = self.contract_registry.read().await;

        ContractManagerStats {
            total_contracts: contracts.len(),
            unique_names: registry.len(),
            deployed_contracts: contracts
                .values()
                .filter(|c| c.deployment.is_some())
                .count(),
            contracts_with_source: contracts.values().filter(|c| c.bytecode.is_some()).count(),
        }
    }
}

impl Default for ContractManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Contract manager statistics
#[derive(Debug, Clone)]
pub struct ContractManagerStats {
    pub total_contracts: usize,
    pub unique_names: usize,
    pub deployed_contracts: usize,
    pub contracts_with_source: usize,
}

/// Utility functions for contract operations
pub mod utils {
    use super::*;

    /// Load contract ABI from file
    pub async fn load_abi_from_file(path: &std::path::Path) -> Result<ContractAbi> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ParadigmError::Io(e))?;
        ContractAbi::from_json(&content)
    }

    /// Load contract bytecode from file
    pub async fn load_bytecode_from_file(path: &std::path::Path) -> Result<ContractBytecode> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ParadigmError::Io(e))?;
        ContractBytecode::from_hex(content.trim())
    }

    /// Calculate contract address for deployment
    pub fn calculate_contract_address(deployer: &Address, nonce: u64) -> Result<Address> {
        // Simplified contract address calculation
        // Real implementation would use RLP encoding and Keccak256
        let mut hasher = sha2::Sha256::new();
        hasher.update(deployer.as_bytes());
        hasher.update(nonce.to_be_bytes());
        let hash = hasher.finalize();

        Address::from_bytes(hash[12..32].try_into().map_err(|_| Error::InvalidAddress("Invalid address length".to_string()))?)
    }

    /// Create standard ERC20 token ABI
    pub fn erc20_abi() -> ContractAbi {
        // Simplified ERC20 ABI
        let mut abi = ContractAbi::new();

        // Add transfer function
        abi.functions.insert(
            "transfer".to_string(),
            AbiFunction {
                name: "transfer".to_string(),
                inputs: vec![
                    AbiParameter {
                        name: "to".to_string(),
                        param_type: "address".to_string(),
                        internal_type: None,
                        components: None,
                    },
                    AbiParameter {
                        name: "amount".to_string(),
                        param_type: "uint256".to_string(),
                        internal_type: None,
                        components: None,
                    },
                ],
                outputs: vec![AbiParameter {
                    name: "".to_string(),
                    param_type: "bool".to_string(),
                    internal_type: None,
                    components: None,
                }],
                state_mutability: StateMutability::NonPayable,
                function_type: FunctionType::Function,
                payable: false,
            },
        );

        // Add balanceOf function
        abi.functions.insert(
            "balanceOf".to_string(),
            AbiFunction {
                name: "balanceOf".to_string(),
                inputs: vec![AbiParameter {
                    name: "account".to_string(),
                    param_type: "address".to_string(),
                    internal_type: None,
                    components: None,
                }],
                outputs: vec![AbiParameter {
                    name: "".to_string(),
                    param_type: "uint256".to_string(),
                    internal_type: None,
                    components: None,
                }],
                state_mutability: StateMutability::View,
                function_type: FunctionType::Function,
                payable: false,
            },
        );

        abi
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[test]
    fn test_contract_abi_creation() {
        let abi = ContractAbi::new();
        assert!(abi.functions.is_empty());
        assert!(abi.events.is_empty());
    }

    #[test]
    fn test_contract_bytecode() {
        let bytecode = ContractBytecode::from_hex("0x608060405234801561001057600080fd5b50");
        assert!(bytecode.is_ok());

        let bytecode = bytecode.unwrap();
        assert!(!bytecode.bytecode.is_empty());
    }

    #[test]
    fn test_contract_metadata() {
        let metadata = ContractMetadata::new(
            "TestContract".to_string(),
            "1.0.0".to_string(),
            "0.8.19".to_string(),
        );

        assert_eq!(metadata.name, "TestContract");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.compiler_version, "0.8.19");
    }

    #[tokio::test]
    async fn test_contract_manager() {
        let manager = ContractManager::new();

        let client = Arc::new(ParadigmClient::with_config(ClientConfig::default()).unwrap());
        let contract = Contract::new(
            Address::default(),
            ContractAbi::new(),
            client,
            ContractMetadata::new("Test".to_string(), "1.0".to_string(), "0.8.0".to_string()),
        );

        let id = manager.add_contract(contract).await;
        let retrieved = manager.get_contract(&id).await;
        assert!(retrieved.is_some());

        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_contracts, 1);
    }

    #[test]
    fn test_erc20_abi() {
        let abi = utils::erc20_abi();
        assert!(abi.get_function("transfer").is_some());
        assert!(abi.get_function("balanceOf").is_some());
    }
}
