use anyhow::Result;
use crate::{config::ApiConfig, error::ApiErrorType};

pub struct MLTaskService {
    config: ApiConfig,
}

impl MLTaskService {
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
}