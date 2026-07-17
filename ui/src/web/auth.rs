//! Web：已移除钱包登录；`token_get` 恒为 `None`，请求不再携带 Bearer。

/// 已移除登录：请求不再携带 Bearer token。
#[cfg(feature = "web")]
pub fn token_get() -> Option<String> {
    None
}

#[cfg(not(feature = "web"))]
pub fn token_get() -> Option<String> {
    None
}
