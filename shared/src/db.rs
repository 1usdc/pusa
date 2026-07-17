//! Auth account DTO mirrored for JSON FFI (implementation stays in pusa-core).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthAccount {
    pub id: i64,
    pub wallet_address: String,
    pub chain_id: Option<i64>,
    pub aliyun_instance_id: Option<String>,
    pub aliyun_public_ip: Option<String>,
    pub anotherme_access_token: Option<String>,
    pub anotherme_refresh_token: Option<String>,
    pub anotherme_token_expires_at: Option<i64>,
    pub login_source: Option<String>,
    pub last_login_at: Option<i64>,
    pub created_at: i64,
}
