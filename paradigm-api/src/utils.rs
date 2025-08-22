/// Utility functions for the API
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

/// Format address for display
pub fn format_address(address: &paradigm_core::Address) -> String {
    format!("PAR{}", hex::encode(&address.0[..20]))
}

/// Parse address from string
pub fn parse_address(address_str: &str) -> Result<paradigm_core::Address, String> {
    if !address_str.starts_with("PAR") {
        return Err("Address must start with PAR".to_string());
    }
    
    let hex_part = &address_str[3..];
    if hex_part.len() != 40 {
        return Err("Invalid address length".to_string());
    }
    
    let bytes = hex::decode(hex_part)
        .map_err(|_| "Invalid hex encoding".to_string())?;
    
    let mut addr_array = [0u8; 32];
    addr_array[..20].copy_from_slice(&bytes);
    
    Ok(paradigm_core::Address(addr_array))
}

/// Format hash for display
pub fn format_hash(hash: &paradigm_core::Hash) -> String {
    format!("0x{}", hex::encode(hash))
}

/// Parse hash from string
pub fn parse_hash(hash_str: &str) -> Result<paradigm_core::Hash, String> {
    let hex_part = if hash_str.starts_with("0x") {
        &hash_str[2..]
    } else {
        hash_str
    };
    
    if hex_part.len() != 64 {
        return Err("Hash must be 32 bytes (64 hex characters)".to_string());
    }
    
    let bytes = hex::decode(hex_part)
        .map_err(|_| "Invalid hex encoding".to_string())?;
    
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&bytes);
    
    Ok(hash_array)
}

/// Convert amount to human-readable format
pub fn format_amount(amount: paradigm_core::Amount) -> String {
    let par_amount = amount as f64 / 100_000_000.0; // 8 decimal places
    format!("{:.8} PAR", par_amount)
}

/// Parse amount from human-readable format
pub fn parse_amount(amount_str: &str) -> Result<paradigm_core::Amount, String> {
    let clean_str = amount_str.trim().replace("PAR", "").trim().to_string();
    let amount_f64: f64 = clean_str.parse()
        .map_err(|_| "Invalid amount format".to_string())?;
    
    if amount_f64 < 0.0 {
        return Err("Amount cannot be negative".to_string());
    }
    
    Ok((amount_f64 * 100_000_000.0) as paradigm_core::Amount)
}

/// Generate pagination metadata
pub fn create_pagination_meta(
    total_count: u64,
    page: u32,
    page_size: u32,
) -> HashMap<String, Value> {
    let total_pages = (total_count + page_size as u64 - 1) / page_size as u64;
    
    let mut meta = HashMap::new();
    meta.insert("total_count".to_string(), Value::from(total_count));
    meta.insert("page".to_string(), Value::from(page));
    meta.insert("page_size".to_string(), Value::from(page_size));
    meta.insert("total_pages".to_string(), Value::from(total_pages));
    meta.insert("has_next".to_string(), Value::from(page < total_pages as u32));
    meta.insert("has_prev".to_string(), Value::from(page > 1));
    
    meta
}

/// Validate pagination parameters
pub fn validate_pagination(page: Option<u32>, page_size: Option<u32>) -> Result<(u32, u32), String> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    
    if page < 1 {
        return Err("Page must be at least 1".to_string());
    }
    
    if page_size < 1 || page_size > 1000 {
        return Err("Page size must be between 1 and 1000".to_string());
    }
    
    Ok((page, page_size))
}

/// Format duration for human consumption
pub fn format_duration(start: DateTime<Utc>, end: DateTime<Utc>) -> String {
    let duration = end.signed_duration_since(start);
    
    if duration.num_days() > 0 {
        format!("{} days", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes", duration.num_minutes())
    } else {
        format!("{} seconds", duration.num_seconds())
    }
}

/// Sanitize user input
pub fn sanitize_string(input: &str, max_length: usize) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".-_@".contains(*c))
        .take(max_length)
        .collect()
}

/// Generate secure random string
pub fn generate_secure_token(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Rate limiting key generation
pub fn generate_rate_limit_key(ip: &str, user_id: Option<uuid::Uuid>) -> String {
    if let Some(user_id) = user_id {
        format!("user:{}", user_id)
    } else {
        format!("ip:{}", ip)
    }
}

/// Convert between different time formats
pub fn timestamp_to_datetime(timestamp: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now())
}

pub fn datetime_to_timestamp(datetime: DateTime<Utc>) -> i64 {
    datetime.timestamp()
}

/// URL validation
pub fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

/// Email validation (basic)
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_address() {
        let address = paradigm_core::Address([1u8; 32]);
        let formatted = format_address(&address);
        assert!(formatted.starts_with("PAR"));
        assert_eq!(formatted.len(), 43); // "PAR" + 40 hex chars
    }
    
    #[test]
    fn test_parse_address() {
        let address_str = "PAR0101010101010101010101010101010101010101";
        let parsed = parse_address(address_str).unwrap();
        assert_eq!(parsed.0[0], 1);
    }
    
    #[test]
    fn test_format_amount() {
        let amount = 123_456_789; // 1.23456789 PAR
        let formatted = format_amount(amount);
        assert_eq!(formatted, "1.23456789 PAR");
    }
    
    #[test]
    fn test_parse_amount() {
        let amount_str = "1.5 PAR";
        let parsed = parse_amount(amount_str).unwrap();
        assert_eq!(parsed, 150_000_000);
    }
    
    #[test]
    fn test_pagination_validation() {
        let (page, page_size) = validate_pagination(Some(2), Some(50)).unwrap();
        assert_eq!(page, 2);
        assert_eq!(page_size, 50);
        
        assert!(validate_pagination(Some(0), Some(50)).is_err());
        assert!(validate_pagination(Some(1), Some(2000)).is_err());
    }
}