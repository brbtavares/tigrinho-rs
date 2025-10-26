use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpinRequest {
    pub client_seed: String,
    pub bet: f64,
    pub lines: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpinResponse {
    pub server_seed_hash: String,
    pub nonce: u64,
    pub reels: Vec<Vec<u8>>, // indices of symbols
    pub payout: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerifyResponse {
    pub server_seed_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminSetParamsRequest {
    pub rtp_target: f64,
    pub paytable: Vec<PaytableEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaytableEntry {
    pub symbol: u8,
    pub count: u8,
    pub payout_multiplier: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpinLogEntry {
    pub id: i64,
    pub ts: DateTime<Utc>,
    pub client_seed: String,
    pub nonce: i64,
    pub server_seed_hash: String,
    pub result_reels: Vec<Vec<u8>>,
    pub payout: f64,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("internal server error")]
    Internal,
}

pub type ApiResult<T> = Result<T, ApiError>;
