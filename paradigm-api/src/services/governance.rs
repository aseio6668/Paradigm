use anyhow::Result;
use crate::{config::ApiConfig, error::ApiErrorType};

pub struct GovernanceService {
    config: ApiConfig,
}

impl GovernanceService {
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
}