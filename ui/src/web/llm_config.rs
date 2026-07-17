//! Web：OpenAI API Key 与兼容端点 Base URL 由服务端数据库保存。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

use gloo_net::http::Request;

/// 默认 OpenAI 官方 API 根路径（须含 `/v1`）。
pub const DEFAULT_OPENAI_V1_BASE: &str = "https://api.openai.com/v1";

pub async fn fetch_llm_config() -> Option<protocol::LlmConfigDto> {
    let base_owned = crate::chat::api_base_url();
    let base = base_owned.trim_end_matches('/');
    let url = format!("{base}/v1/llm/config");
    let resp = Request::get(&url).send().await.ok()?;
    if !resp.ok() {
        return None;
    }
    resp.json::<protocol::LlmConfigDto>().await.ok()
}

pub async fn save_llm_config(
    api_key: &str,
    openai_v1_base: &str,
) -> Option<protocol::LlmConfigDto> {
    save_llm_config_full(api_key, openai_v1_base, None, false, false).await
}

/// 保存时若输入框为空，**显式**把 DB 字段清空（写入空字符串覆盖旧值）。
pub async fn save_llm_config_clearing(
    api_key: &str,
    openai_v1_base: &str,
    clear_api_key: bool,
    clear_openai_v1_base: bool,
) -> Option<protocol::LlmConfigDto> {
    save_llm_config_full(
        api_key,
        openai_v1_base,
        None,
        clear_api_key,
        clear_openai_v1_base,
    )
    .await
}

async fn save_llm_config_full(
    api_key: &str,
    openai_v1_base: &str,
    prefer_custom_key: Option<bool>,
    clear_api_key: bool,
    clear_openai_v1_base: bool,
) -> Option<protocol::LlmConfigDto> {
    let base_owned = crate::chat::api_base_url();
    let base = base_owned.trim_end_matches('/');
    let url = format!("{base}/v1/llm/config");
    let body = protocol::LlmConfigUpsertBody {
        api_key: api_key.trim().to_string(),
        openai_v1_base: openai_v1_base.trim().to_string(),
        prefer_custom_key,
        clear_api_key,
        clear_openai_v1_base,
    };
    let resp = Request::put(&url)
        .json(&body)
        .ok()?
        .send()
        .await
        .ok()?;
    if !resp.ok() {
        return None;
    }
    resp.json::<protocol::LlmConfigDto>().await.ok()
}

/// 查询服务端是否已通过环境变量配置 OpenAI；网络失败时返回 `None`。
pub async fn fetch_server_openai_configured() -> Option<bool> {
    let base_owned = crate::chat::api_base_url();
    let base = base_owned.trim_end_matches('/');
    let url = format!("{base}/v1/llm/status");
    let resp = Request::get(&url).send().await.ok()?;
    if !resp.ok() {
        return None;
    }
    let body: protocol::LlmStatusBody = resp.json().await.ok()?;
    Some(body.server_openai_configured)
}
