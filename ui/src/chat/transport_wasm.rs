//! WASM：`POST /v1/chat/stream`，读取 fetch body 字节流并解析 SSE。

use futures_util::StreamExt;
use protocol::{
    AboutInfoDto, AgentRunDetailDto, ChatTurnRequest, ConversationSummaryDto, ConversationTitleBody,
    EquippedSkillsDto, InstalledSkillsResponse, RoleCreateRequest, RoleDto, RoleUpdateRequest,
    SkillEquipToggleRequest, SkillInstallRequest, SkillInstallResponse, SkillMarketResponse,
    SseEvent, StoredChatMessageDto,
};
use wasm_bindgen::JsCast;
use wasm_streams::readable::ReadableStream;

/// 将增量 UTF-8 追加到缓冲区并派发完整的 SSE `data:` JSON 事件。
fn dispatch_sse_block(block: &str, on_event: &mut impl FnMut(SseEvent)) {
    for line in block.lines() {
        let line = line.trim_end_matches('\r');
        let Some(rest) = line.strip_prefix("data:") else {
            continue;
        };
        let data = rest.trim();
        if data.is_empty() {
            continue;
        }
        if let Ok(ev) = serde_json::from_str::<SseEvent>(data) {
            on_event(ev);
        }
    }
}

pub(crate) fn drain_sse_buffer(
    buf: &mut String,
    chunk: &str,
    on_event: &mut impl FnMut(SseEvent),
) -> anyhow::Result<()> {
    buf.push_str(chunk);
    loop {
        let Some(idx) = buf.find("\n\n") else {
            break;
        };
        let block = buf[..idx].to_string();
        buf.drain(..idx + 2);
        dispatch_sse_block(&block, on_event);
    }
    Ok(())
}

/// GET `/v1/about`（无需登录）。
pub async fn load_about_info(base_url: &str) -> anyhow::Result<AboutInfoDto> {
    let url = format!("{}/v1/about", base_url.trim_end_matches('/'));
    let resp = gloo_net::http::Request::get(&url).send().await?;
    if !resp.ok() {
        anyhow::bail!("about HTTP {}", resp.status());
    }
    Ok(resp.json::<AboutInfoDto>().await?)
}

/// POST JSON，消费 `text/event-stream` 响应体。
///
/// `openai_api_key_override` / `openai_v1_base_override` 不为空时，会以
/// `x-openai-api-key` / `x-openai-base-url` 头透传给 pusa server，让 server 端
/// 跳过 DB / env 里的 key，直接用调用方传入的凭证调上游。
pub async fn post_chat_stream(
    base_url: &str,
    req: ChatTurnRequest,
    abort: Option<&web_sys::AbortSignal>,
    openai_api_key_override: Option<&str>,
    openai_v1_base_override: Option<&str>,
    on_event: &mut impl FnMut(SseEvent),
) -> anyhow::Result<()> {
    let url = format!("{}/v1/chat/stream", base_url.trim_end_matches('/'));
    let mut builder = gloo_net::http::Request::post(&url);
    builder = builder.abort_signal(abort);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    if let Some(k) = openai_api_key_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        builder = builder.header("x-openai-api-key", k);
    }
    if let Some(b) = openai_v1_base_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        builder = builder.header("x-openai-base-url", b);
    }
    let gloo_resp = match builder.json(&req)?.send().await {
        Ok(r) => r,
        Err(e) => {
            if abort.map(|s| s.aborted()).unwrap_or(false) {
                return Ok(());
            }
            // fetch send 错误一般是网络/超时；让上层做友好映射前保留原文。
            return Err(anyhow::anyhow!("chat stream send: {e}"));
        }
    };

    if !gloo_resp.ok() {
        // 不要把"chat stream HTTP 502"原样返出去——console.rs 会把错误塞进对话气泡，
        // 用 friendly_chat_error_message 把 502/503/504 都映射成"云平台暂时抖动，请稍后重试"。
        let status = gloo_resp.status();
        let raw = format!("chat stream HTTP {status}");
        let friendly = crate::chat::friendly_chat_error_message(&raw);
        anyhow::bail!("{friendly}");
    }

    let web_resp: web_sys::Response = gloo_resp.into();
    let body_js = web_resp
        .body()
        .ok_or_else(|| anyhow::anyhow!("response body null"))?;

    let readable = ReadableStream::from_raw(body_js.unchecked_into());
    let mut byte_stream = readable.into_stream();

    let mut sse_carry = String::new();

    while let Some(js_chunk) = byte_stream.next().await {
        if abort.map(|s| s.aborted()).unwrap_or(false) {
            break;
        }
        let js_chunk = match js_chunk {
            Ok(c) => c,
            Err(e) => {
                if abort.map(|s| s.aborted()).unwrap_or(false) {
                    break;
                }
                return Err(anyhow::anyhow!("stream {:?}", e));
            }
        };
        let arr: js_sys::Uint8Array = js_chunk
            .dyn_into()
            .map_err(|_| anyhow::anyhow!("chunk not Uint8Array"))?;
        let mut chunk = vec![0u8; arr.length() as usize];
        arr.copy_to(&mut chunk);

        let chunk_str = String::from_utf8_lossy(&chunk);
        drain_sse_buffer(&mut sse_carry, &chunk_str, on_event)?;
    }

    let tail = sse_carry.trim();
    if !tail.is_empty() {
        dispatch_sse_block(tail, on_event);
    }

    Ok(())
}

pub async fn fetch_conversations(base_url: &str) -> anyhow::Result<Vec<ConversationSummaryDto>> {
    let url = format!("{}/v1/chat/conversations", base_url.trim_end_matches('/'));
    let mut builder = gloo_net::http::Request::get(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("chat conversations HTTP {}", resp.status());
    }
    Ok(resp.json::<Vec<ConversationSummaryDto>>().await?)
}

pub async fn create_conversation(base_url: &str) -> anyhow::Result<ConversationSummaryDto> {
    let url = format!("{}/v1/chat/conversations", base_url.trim_end_matches('/'));
    let mut builder = gloo_net::http::Request::post(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("create conversation HTTP {}", resp.status());
    }
    Ok(resp.json::<ConversationSummaryDto>().await?)
}

pub async fn update_conversation_title(
    base_url: &str,
    conversation_id: &str,
    title: &str,
) -> anyhow::Result<ConversationSummaryDto> {
    let url = format!(
        "{}/v1/chat/conversations/{}/title",
        base_url.trim_end_matches('/'),
        conversation_id
    );
    let mut builder = gloo_net::http::Request::put(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder
        .json(&ConversationTitleBody {
            title: title.to_string(),
        })?
        .send()
        .await?;
    if !resp.ok() {
        anyhow::bail!("update conversation title HTTP {}", resp.status());
    }
    Ok(resp.json::<ConversationSummaryDto>().await?)
}

pub async fn delete_conversation(base_url: &str, conversation_id: &str) -> anyhow::Result<()> {
    let url = format!(
        "{}/v1/chat/conversations/{}",
        base_url.trim_end_matches('/'),
        conversation_id
    );
    let mut builder = gloo_net::http::Request::delete(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("delete conversation HTTP {}", resp.status());
    }
    Ok(())
}

pub async fn fetch_conversation_messages(
    base_url: &str,
    conversation_id: &str,
    limit: usize,
) -> anyhow::Result<Vec<StoredChatMessageDto>> {
    let url = format!(
        "{}/v1/chat/conversations/{}/messages?limit={}",
        base_url.trim_end_matches('/'),
        conversation_id,
        limit
    );
    let mut builder = gloo_net::http::Request::get(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("conversation messages HTTP {}", resp.status());
    }
    Ok(resp.json::<Vec<StoredChatMessageDto>>().await?)
}

pub async fn fetch_agent_run_detail(
    base_url: &str,
    run_id: i64,
) -> anyhow::Result<AgentRunDetailDto> {
    let url = format!(
        "{}/v1/chat/agent-runs/{}",
        base_url.trim_end_matches('/'),
        run_id
    );
    let mut builder = gloo_net::http::Request::get(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("agent run detail HTTP {}", resp.status());
    }
    Ok(resp.json::<AgentRunDetailDto>().await?)
}

pub async fn fetch_skill_market(
    base_url: &str,
    query: &str,
) -> anyhow::Result<SkillMarketResponse> {
    let url = if query.trim().is_empty() {
        format!("{}/v1/skills/market", base_url.trim_end_matches('/'))
    } else {
        format!(
            "{}/v1/skills/market?q={}",
            base_url.trim_end_matches('/'),
            js_sys::encode_uri_component(query.trim())
        )
    };
    let mut builder = gloo_net::http::Request::get(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("skill market HTTP {}", resp.status());
    }
    Ok(resp.json::<SkillMarketResponse>().await?)
}

pub async fn install_skill(
    base_url: &str,
    source: &str,
    id: &str,
    name: Option<String>,
    install_url: Option<String>,
) -> anyhow::Result<SkillInstallResponse> {
    let url = format!("{}/v1/skills/install", base_url.trim_end_matches('/'));
    let mut builder = gloo_net::http::Request::post(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder
        .json(&SkillInstallRequest {
            source: source.to_string(),
            id: id.to_string(),
            name,
            url: install_url,
        })?
        .send()
        .await?;
    if !resp.ok() {
        anyhow::bail!("skill install HTTP {}", resp.status());
    }
    Ok(resp.json::<SkillInstallResponse>().await?)
}

#[allow(dead_code)]
pub async fn fetch_equipped_skills(base_url: &str) -> anyhow::Result<EquippedSkillsDto> {
    let url = format!("{}/v1/skills/equipped", base_url.trim_end_matches('/'));
    let mut builder = gloo_net::http::Request::get(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("equipped skills HTTP {}", resp.status());
    }
    Ok(resp.json::<EquippedSkillsDto>().await?)
}

#[allow(dead_code)]
pub async fn post_skill_equip_toggle(
    base_url: &str,
    body: &SkillEquipToggleRequest,
) -> anyhow::Result<EquippedSkillsDto> {
    let url = format!("{}/v1/skills/equipped", base_url.trim_end_matches('/'));
    let mut builder = gloo_net::http::Request::post(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.json(body)?.send().await?;
    if !resp.ok() {
        anyhow::bail!("equip toggle HTTP {}", resp.status());
    }
    Ok(resp.json::<EquippedSkillsDto>().await?)
}

pub async fn fetch_installed_skills(base_url: &str) -> anyhow::Result<InstalledSkillsResponse> {
    let url = format!("{}/v1/skills/installed", base_url.trim_end_matches('/'));
    let mut builder = gloo_net::http::Request::get(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("installed skills HTTP {}", resp.status());
    }
    Ok(resp.json::<InstalledSkillsResponse>().await?)
}

pub async fn uninstall_installed_skill(base_url: &str, slug: &str) -> anyhow::Result<()> {
    let url = format!(
        "{}/v1/skills/installed/{}",
        base_url.trim_end_matches('/'),
        js_sys::encode_uri_component(slug)
    );
    let mut builder = gloo_net::http::Request::delete(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("skill uninstall HTTP {}", resp.status());
    }
    Ok(())
}

pub async fn fetch_roles(base_url: &str) -> anyhow::Result<Vec<RoleDto>> {
    let url = format!("{}/v1/roles", base_url.trim_end_matches('/'));
    let mut builder = gloo_net::http::Request::get(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("roles list HTTP {}", resp.status());
    }
    Ok(resp.json::<Vec<RoleDto>>().await?)
}

pub async fn create_role(base_url: &str, body: &RoleCreateRequest) -> anyhow::Result<RoleDto> {
    let url = format!("{}/v1/roles", base_url.trim_end_matches('/'));
    let mut builder = gloo_net::http::Request::post(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.json(body)?.send().await?;
    if !resp.ok() {
        anyhow::bail!("roles create HTTP {}", resp.status());
    }
    Ok(resp.json::<RoleDto>().await?)
}

pub async fn update_role(
    base_url: &str,
    id: i64,
    body: &RoleUpdateRequest,
) -> anyhow::Result<RoleDto> {
    let url = format!("{}/v1/roles/{}", base_url.trim_end_matches('/'), id);
    let mut builder = gloo_net::http::Request::put(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.json(body)?.send().await?;
    if !resp.ok() {
        anyhow::bail!("roles update HTTP {}", resp.status());
    }
    Ok(resp.json::<RoleDto>().await?)
}

pub async fn activate_role(base_url: &str, id: i64) -> anyhow::Result<RoleDto> {
    let url = format!(
        "{}/v1/roles/{}/activate",
        base_url.trim_end_matches('/'),
        id
    );
    let mut builder = gloo_net::http::Request::put(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("roles activate HTTP {}", resp.status());
    }
    Ok(resp.json::<RoleDto>().await?)
}

pub async fn delete_role(base_url: &str, id: i64) -> anyhow::Result<()> {
    let url = format!("{}/v1/roles/{}/delete", base_url.trim_end_matches('/'), id);
    let mut builder = gloo_net::http::Request::post(&url);
    if let Some(tok) = crate::web::auth::token_get() {
        builder = builder.header("Authorization", &format!("Bearer {}", tok));
    }
    let resp = builder.send().await?;
    if !resp.ok() {
        anyhow::bail!("roles delete HTTP {}", resp.status());
    }
    Ok(())
}
