use anyhow::Result;
use crate::{config::ApiConfig, error::ApiErrorType};

pub struct CrossChainService {
    config: ApiConfig,
}

impl CrossChainService {
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
}