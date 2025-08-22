use anyhow::Result;
use crate::{config::ApiConfig, error::ApiErrorType};

pub struct WebSocketService {
    config: ApiConfig,
}

impl WebSocketService {
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
}