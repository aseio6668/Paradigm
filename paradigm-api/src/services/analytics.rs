use anyhow::Result;
use crate::{config::ApiConfig, error::ApiErrorType};

pub struct AnalyticsService {
    config: ApiConfig,
}

impl AnalyticsService {
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
}