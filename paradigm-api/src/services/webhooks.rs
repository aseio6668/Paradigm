use anyhow::Result;
use crate::{config::ApiConfig, error::ApiErrorType};

pub struct WebhookService {
    config: ApiConfig,
}

impl WebhookService {
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
}