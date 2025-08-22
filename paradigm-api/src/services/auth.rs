use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    config::ApiConfig,
    error::ApiErrorType,
    models::{LoginResponse, UserProfile, UserRole},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub user_id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub organization: Option<String>,
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
}

#[derive(Debug, Clone)]
pub struct QuotaStatus {
    pub requests_used: u64,
    pub requests_limit: u64,
    pub exceeded: bool,
    pub reset_time: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    pub key_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub permissions: Vec<String>,
    pub last_used: Option<chrono::DateTime<Utc>>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub is_active: bool,
}

pub struct AuthService {
    config: ApiConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    // In a real implementation, these would be in a database
    users: tokio::sync::RwLock<HashMap<String, UserData>>,
    api_keys: tokio::sync::RwLock<HashMap<String, ApiKeyInfo>>,
    quotas: tokio::sync::RwLock<HashMap<Uuid, QuotaStatus>>,
    blacklisted_tokens: tokio::sync::RwLock<std::collections::HashSet<String>>,
}

#[derive(Debug, Clone)]
struct UserData {
    id: Uuid,
    email: String,
    password_hash: String,
    name: String,
    organization: Option<String>,
    role: UserRole,
    created_at: chrono::DateTime<Utc>,
    last_login: Option<chrono::DateTime<Utc>>,
    is_active: bool,
}

impl AuthService {
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        
        Ok(Self {
            config: config.clone(),
            encoding_key,
            decoding_key,
            users: tokio::sync::RwLock::new(HashMap::new()),
            api_keys: tokio::sync::RwLock::new(HashMap::new()),
            quotas: tokio::sync::RwLock::new(HashMap::new()),
            blacklisted_tokens: tokio::sync::RwLock::new(std::collections::HashSet::new()),
        })
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginResponse, ApiErrorType> {
        let users = self.users.read().await;
        let user = users.get(email)
            .ok_or(ApiErrorType::InvalidCredentials)?;

        if !user.is_active {
            return Err(ApiErrorType::Forbidden);
        }

        // Verify password
        if !bcrypt::verify(password, &user.password_hash)
            .map_err(|_| ApiErrorType::InternalServerError)? {
            return Err(ApiErrorType::InvalidCredentials);
        }

        drop(users);

        // Update last login
        {
            let mut users = self.users.write().await;
            if let Some(user) = users.get_mut(email) {
                user.last_login = Some(Utc::now());
            }
        }

        // Generate tokens
        let (access_token, refresh_token) = self.generate_tokens(&user).await?;

        let profile = UserProfile {
            id: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
            organization: user.organization.clone(),
            role: user.role.clone(),
            api_key: None, // Don't include in login response
            created_at: user.created_at,
            last_login: user.last_login,
        };

        Ok(LoginResponse {
            access_token,
            refresh_token,
            expires_in: self.config.jwt_expiration_hours * 3600,
            user: profile,
        })
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
        name: &str,
        organization: Option<&str>,
    ) -> Result<UserProfile, ApiErrorType> {
        // Check if user already exists
        {
            let users = self.users.read().await;
            if users.contains_key(email) {
                return Err(ApiErrorType::AlreadyExists {
                    resource: "User".to_string(),
                });
            }
        }

        // Validate password requirements
        self.validate_password(password)?;

        // Hash password
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|_| ApiErrorType::InternalServerError)?;

        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let user_data = UserData {
            id: user_id,
            email: email.to_string(),
            password_hash,
            name: name.to_string(),
            organization: organization.map(|s| s.to_string()),
            role: UserRole::User, // Default role
            created_at: now,
            last_login: None,
            is_active: true,
        };

        // Store user
        {
            let mut users = self.users.write().await;
            users.insert(email.to_string(), user_data.clone());
        }

        // Initialize quota
        self.init_user_quota(user_id).await?;

        Ok(UserProfile {
            id: user_data.id,
            email: user_data.email,
            name: user_data.name,
            organization: user_data.organization,
            role: user_data.role,
            api_key: None,
            created_at: user_data.created_at,
            last_login: user_data.last_login,
        })
    }

    pub async fn validate_token(&self, token: &str) -> Result<UserClaims, ApiErrorType> {
        // Check if token is blacklisted
        {
            let blacklisted = self.blacklisted_tokens.read().await;
            if blacklisted.contains(token) {
                return Err(ApiErrorType::TokenExpired);
            }
        }

        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<UserClaims>(token, &self.decoding_key, &validation)
            .map_err(|_| ApiErrorType::Unauthorized)?;

        Ok(token_data.claims)
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<LoginResponse, ApiErrorType> {
        let claims = self.validate_token(refresh_token).await?;
        
        // Get user data
        let users = self.users.read().await;
        let user = users.values()
            .find(|u| u.id == claims.user_id)
            .ok_or(ApiErrorType::Unauthorized)?
            .clone();
        drop(users);

        // Generate new tokens
        let (access_token, new_refresh_token) = self.generate_tokens(&user).await?;

        // Blacklist old refresh token
        {
            let mut blacklisted = self.blacklisted_tokens.write().await;
            blacklisted.insert(refresh_token.to_string());
        }

        let profile = UserProfile {
            id: user.id,
            email: user.email,
            name: user.name,
            organization: user.organization,
            role: user.role,
            api_key: None,
            created_at: user.created_at,
            last_login: user.last_login,
        };

        Ok(LoginResponse {
            access_token,
            refresh_token: new_refresh_token,
            expires_in: self.config.jwt_expiration_hours * 3600,
            user: profile,
        })
    }

    pub async fn logout(&self, token: &str) -> Result<(), ApiErrorType> {
        // Add token to blacklist
        let mut blacklisted = self.blacklisted_tokens.write().await;
        blacklisted.insert(token.to_string());
        Ok(())
    }

    pub async fn validate_api_key(&self, api_key: &str) -> Result<ApiKeyInfo, ApiErrorType> {
        let api_keys = self.api_keys.read().await;
        let key_info = api_keys.get(api_key)
            .ok_or(ApiErrorType::Unauthorized)?;

        if !key_info.is_active {
            return Err(ApiErrorType::Unauthorized);
        }

        if let Some(expires_at) = key_info.expires_at {
            if Utc::now() > expires_at {
                return Err(ApiErrorType::TokenExpired);
            }
        }

        Ok(key_info.clone())
    }

    pub async fn create_api_key(
        &self,
        name: &str,
        permissions: &[String],
        expires_in_days: Option<u32>,
    ) -> Result<crate::routes::auth::ApiKeyResponse, ApiErrorType> {
        let key_id = Uuid::new_v4();
        let user_id = Uuid::new_v4(); // Would come from current user context
        
        // Generate API key
        let api_key = format!("par_live_{}", 
            hex::encode(&key_id.as_bytes()[..16]));

        let expires_at = expires_in_days.map(|days| 
            Utc::now() + Duration::days(days as i64));

        let key_info = ApiKeyInfo {
            key_id,
            user_id,
            name: name.to_string(),
            permissions: permissions.to_vec(),
            last_used: None,
            expires_at,
            is_active: true,
        };

        // Store API key
        {
            let mut api_keys = self.api_keys.write().await;
            api_keys.insert(api_key.clone(), key_info);
        }

        Ok(crate::routes::auth::ApiKeyResponse {
            id: key_id,
            name: name.to_string(),
            api_key,
            permissions: permissions.to_vec(),
            expires_at,
            created_at: Utc::now(),
        })
    }

    pub async fn revoke_api_key(&self, key_id: &Uuid) -> Result<(), ApiErrorType> {
        let mut api_keys = self.api_keys.write().await;
        
        // Find and deactivate the key
        for (_, key_info) in api_keys.iter_mut() {
            if key_info.key_id == *key_id {
                key_info.is_active = false;
                return Ok(());
            }
        }

        Err(ApiErrorType::NotFound {
            resource: "API Key".to_string(),
        })
    }

    pub async fn check_quota(&self, user_id: &Uuid) -> Result<QuotaStatus, ApiErrorType> {
        let quotas = self.quotas.read().await;
        let quota = quotas.get(user_id)
            .ok_or(ApiErrorType::InternalServerError)?;
        Ok(quota.clone())
    }

    pub async fn increment_usage(&self, user_id: &Uuid) -> Result<(), ApiErrorType> {
        let mut quotas = self.quotas.write().await;
        if let Some(quota) = quotas.get_mut(user_id) {
            quota.requests_used += 1;
            quota.exceeded = quota.requests_used >= quota.requests_limit;
        }
        Ok(())
    }

    // Private methods

    async fn generate_tokens(&self, user: &UserData) -> Result<(String, String), ApiErrorType> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.config.jwt_expiration_hours as i64);

        let claims = UserClaims {
            user_id: user.id,
            email: user.email.clone(),
            role: user.role.clone(),
            organization: user.organization.clone(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            sub: user.id.to_string(),
        };

        let access_token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|_| ApiErrorType::InternalServerError)?;

        // Refresh token with longer expiration
        let refresh_exp = now + Duration::days(30);
        let refresh_claims = UserClaims {
            exp: refresh_exp.timestamp() as usize,
            ..claims
        };

        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)
            .map_err(|_| ApiErrorType::InternalServerError)?;

        Ok((access_token, refresh_token))
    }

    fn validate_password(&self, password: &str) -> Result<(), ApiErrorType> {
        let req = &self.config.auth.password_requirements;

        if password.len() < req.min_length {
            return Err(ApiErrorType::ValidationFailed {
                field: "password".to_string(),
            });
        }

        if req.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(ApiErrorType::ValidationFailed {
                field: "password".to_string(),
            });
        }

        if req.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(ApiErrorType::ValidationFailed {
                field: "password".to_string(),
            });
        }

        if req.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(ApiErrorType::ValidationFailed {
                field: "password".to_string(),
            });
        }

        if req.require_symbols && !password.chars().any(|c| c.is_ascii_punctuation()) {
            return Err(ApiErrorType::ValidationFailed {
                field: "password".to_string(),
            });
        }

        Ok(())
    }

    async fn init_user_quota(&self, user_id: Uuid) -> Result<(), ApiErrorType> {
        let quota = QuotaStatus {
            requests_used: 0,
            requests_limit: 1000, // Default quota
            exceeded: false,
            reset_time: Utc::now() + Duration::days(30),
        };

        let mut quotas = self.quotas.write().await;
        quotas.insert(user_id, quota);
        Ok(())
    }
}