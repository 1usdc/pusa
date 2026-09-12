//! Web 走 HTTP+SSE；桌面 native 进程内 [`shared::RuntimeContext`]。

use protocol::{
    AgentRunDetailDto, ChatModelDto, ChatTurnRequest, ConversationSummaryDto, EquippedSkillsDto,
    InstalledSkillsResponse, RoleCreateRequest, RoleDto, RoleUpdateRequest, SkillInstallResponse,
    SkillMarketResponse, SseEvent, StoredChatMessageDto,
};

/// Another-Me-Relay API 根（无尾斜杠）。用于公开的 `GET /v1/models` 等。
///
/// 优先级（与历史 `ui/src/web/relay.rs` 对齐）：
/// 1. Web：`localStorage.anotherclaw_relay_base_url`
/// 2. Web + `same-origin-api`：`{mount_prefix}/relay`（由 nginx 反代，绕开 CORS）
/// 3. 编译期 `ANOTHERME_RELAY_BASE_URL`
/// 4. 默认 `https://relay.anotherme.co`
#[cfg_attr(
    not(any(target_arch = "wasm32", feature = "native")),
    allow(dead_code)
)]
pub fn relay_base_url() -> String {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    if let Some(u) = relay_base_from_local_storage() {
        return u;
    }
    #[cfg(all(target_arch = "wasm32", feature = "web", feature = "same-origin-api"))]
    {
        let prefix = crate::web::origin::mount_prefix();
        return format!("{prefix}/relay");
    }
    #[cfg(not(all(target_arch = "wasm32", feature = "web", feature = "same-origin-api")))]
    {
        if let Some(url) = option_env!("ANOTHERME_RELAY_BASE_URL") {
            let t = url.trim();
            if !t.is_empty() {
                return t.trim_end_matches('/').to_string();
            }
        }
        "https://relay.anotherme.co".to_string()
    }
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
fn relay_base_from_local_storage() -> Option<String> {
    const LS_KEY: &str = "anotherclaw_relay_base_url";
    let w = web_sys::window()?;
    let storage = w.local_storage().ok()??;
    let url = storage.get_item(LS_KEY).ok()??;
    let t = url.trim();
    if t.is_empty() {
        return None;
    }
    Some(t.trim_end_matches('/').to_string())
}

/// 拉取可用聊天模型列表（Relay `GET /v1/models`，无需 JWT）。
///
/// 空列表视为失败，由调用方保留本地兜底。结果按 `id` 字母序稳定排序。
#[cfg(target_arch = "wasm32")]
pub async fn list_chat_models() -> anyhow::Result<Vec<ChatModelDto>> {
    use protocol::ChatModelsResponse;
    let base = relay_base_url();
    let url = format!("{}/v1/models", base.trim_end_matches('/'));
    let resp = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("models network: {e}"))?;
    if !resp.ok() {
        anyhow::bail!("models http {}", resp.status());
    }
    let body: ChatModelsResponse = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("models decode: {e}"))?;
    Ok(normalize_chat_models(body.data))
}

/// 拉取可用聊天模型列表（Relay `GET /v1/models`，无需 JWT）。
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn list_chat_models() -> anyhow::Result<Vec<ChatModelDto>> {
    use protocol::ChatModelsResponse;
    let base = relay_base_url();
    let url = format!("{}/v1/models", base.trim_end_matches('/'));
    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("models network: {e}"))?;
    if !resp.status().is_success() {
        anyhow::bail!("models http {}", resp.status());
    }
    let body: ChatModelsResponse = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("models decode: {e}"))?;
    Ok(normalize_chat_models(body.data))
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn list_chat_models() -> anyhow::Result<Vec<ChatModelDto>> {
    anyhow::bail!("当前构建未启用 `native` feature，无法拉取模型列表")
}

#[cfg(any(target_arch = "wasm32", feature = "native"))]
fn normalize_chat_models(mut models: Vec<ChatModelDto>) -> Vec<ChatModelDto> {
    models.retain(|m| !m.id.trim().is_empty());
    for m in &mut models {
        m.id = m.id.trim().to_string();
        if let Some(name) = m.display_name.as_mut() {
            let t = name.trim();
            if t.is_empty() {
                m.display_name = None;
            } else {
                *name = t.to_string();
            }
        }
    }
    models.sort_by(|a, b| a.id.cmp(&b.id));
    models
}

/// Web WASM 下 API 根地址。
///
/// 优先级：`localStorage.anotherclaw_api_base_url` →（启用 `same-origin-api` 时）**空字符串**（请求走同源 `/v1/*`，由 Nginx 反代）→
/// 编译期 `CLAW_SERVER_URL`（非空）→ 若页面 host 非 localhost/127.0.0.1 则 `{protocol}//{host}:8787` →
/// 默认 `http://127.0.0.1:8787`。
#[cfg(target_arch = "wasm32")]
pub fn api_base_url() -> String {
    #[cfg(feature = "web")]
    if let Some(u) = browser_api_base_from_local_storage() {
        return u;
    }
    #[cfg(all(feature = "web", feature = "same-origin-api"))]
    {
        // 返回 mount prefix（如 `/proxy/i-xxx`）或空串。调用方再拼 `/v1/...`
        // 时变成 `/proxy/i-xxx/v1/...`，让网关 strip prefix 后 server 命中
        // `/v1/`。直接挂 root 时退化成空串，等同于旧行为。
        return crate::web::origin::mount_prefix();
    }
    #[cfg(not(all(feature = "web", feature = "same-origin-api")))]
    if let Some(url) = option_env!("CLAW_SERVER_URL") {
        if !url.is_empty() {
            return url.to_string();
        }
    }
    #[cfg(not(all(feature = "web", feature = "same-origin-api")))]
    #[cfg(feature = "web")]
    if let Some(u) = browser_api_base_same_host_default_port() {
        return u;
    }
    #[cfg(not(all(feature = "web", feature = "same-origin-api")))]
    "http://127.0.0.1:8787".to_string()
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
fn browser_api_base_from_local_storage() -> Option<String> {
    const LS_KEY: &str = "anotherclaw_api_base_url";
    let w = web_sys::window()?;
    let storage = w.local_storage().ok()??;
    let url = storage.get_item(LS_KEY).ok()??;
    let t = url.trim();
    if t.is_empty() {
        return None;
    }
    Some(t.to_string())
}

/// 页面从局域网 IP 打开时，API 应落在同一台机器的 `8787`（需 `BIND_ADDR=0.0.0.0:8787` 启动 server）。未启用 `same-origin-api` 时使用。
///
/// Dioxus dev server 在 macOS 上可能把页面暴露为 `198.18.0.1:8080`，但本地 API server
/// 默认只监听 `127.0.0.1:8787`。这种虚拟地址不能用来推导 API host，否则聊天请求会卡在
/// 错误目标上；让它回落到默认 localhost。
#[cfg(all(
    target_arch = "wasm32",
    feature = "web",
    not(feature = "same-origin-api")
))]
fn browser_api_base_same_host_default_port() -> Option<String> {
    let w = web_sys::window()?;
    let loc = w.location();
    let host = loc.hostname().ok()?;
    let protocol = loc.protocol().ok()?;
    if host.is_empty() || host == "localhost" || host == "127.0.0.1" || host == "198.18.0.1" {
        return None;
    }
    Some(format!("{protocol}//{host}:8787"))
}

/// 执行一轮流式对话，`on_event` 在主流程线程被同步调用（可安全更新 Dioxus `Signal`）。
///
/// `abort`：传入 [`web_sys::AbortSignal`] 时，中止 fetch 可停止生成（与发送区「暂停」联动）。
///
/// API Key 与 Base URL 由 pusa server 从数据库或环境变量 `OPENAI_API_KEY` 读取。
#[cfg(target_arch = "wasm32")]
pub async fn run_chat_turn<F>(
    req: ChatTurnRequest,
    abort: Option<&web_sys::AbortSignal>,
    mut on_event: F,
) -> anyhow::Result<()>
where
    F: FnMut(SseEvent),
{
    let base = api_base_url();
    crate::chat::transport_wasm::post_chat_stream(
        &base,
        req,
        abort,
        None,
        None,
        &mut on_event,
    )
    .await
}

/// 非 web feature 的 wasm 构建（理论上不会发生，因 `web` 在 `default-features` 内，
/// 但保留兜底以免上游裁剪 features 时编译失败）。
#[cfg(all(target_arch = "wasm32", not(feature = "web")))]
async fn resolve_chat_credentials_for_request() -> (Option<String>, Option<String>) {
    (None, None)
}

#[cfg(target_arch = "wasm32")]
pub async fn list_conversations() -> anyhow::Result<Vec<ConversationSummaryDto>> {
    let base = api_base_url();
    crate::chat::transport_wasm::fetch_conversations(&base).await
}

#[cfg(target_arch = "wasm32")]
pub async fn create_conversation() -> anyhow::Result<ConversationSummaryDto> {
    let base = api_base_url();
    crate::chat::transport_wasm::create_conversation(&base).await
}

#[cfg(target_arch = "wasm32")]
pub async fn update_conversation_title(
    conversation_id: String,
    title: String,
) -> anyhow::Result<ConversationSummaryDto> {
    let base = api_base_url();
    crate::chat::transport_wasm::update_conversation_title(&base, &conversation_id, &title).await
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_conversation(conversation_id: String) -> anyhow::Result<()> {
    let base = api_base_url();
    crate::chat::transport_wasm::delete_conversation(&base, &conversation_id).await
}

#[cfg(target_arch = "wasm32")]
pub async fn load_conversation_messages(
    conversation_id: String,
    limit: usize,
) -> anyhow::Result<Vec<StoredChatMessageDto>> {
    let base = api_base_url();
    crate::chat::transport_wasm::fetch_conversation_messages(&base, &conversation_id, limit).await
}

#[cfg(target_arch = "wasm32")]
pub async fn load_agent_run_detail(run_id: i64) -> anyhow::Result<AgentRunDetailDto> {
    let base = api_base_url();
    crate::chat::transport_wasm::fetch_agent_run_detail(&base, run_id).await
}

#[cfg(target_arch = "wasm32")]
pub async fn load_skill_market(query: String) -> anyhow::Result<SkillMarketResponse> {
    let base = api_base_url();
    crate::chat::transport_wasm::fetch_skill_market(&base, &query).await
}

#[cfg(target_arch = "wasm32")]
pub async fn install_skill(
    source: String,
    id: String,
    name: Option<String>,
    url: Option<String>,
) -> anyhow::Result<SkillInstallResponse> {
    let base = api_base_url();
    crate::chat::transport_wasm::install_skill(&base, &source, &id, name, url).await
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
pub async fn load_equipped_skills() -> anyhow::Result<EquippedSkillsDto> {
    let base = api_base_url();
    crate::chat::transport_wasm::fetch_equipped_skills(&base).await
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
pub async fn toggle_skill_equip(slug: String, equipped: bool) -> anyhow::Result<EquippedSkillsDto> {
    let base = api_base_url();
    crate::chat::transport_wasm::post_skill_equip_toggle(
        &base,
        &protocol::SkillEquipToggleRequest { slug, equipped },
    )
    .await
}

#[cfg(target_arch = "wasm32")]
pub async fn load_installed_skills() -> anyhow::Result<InstalledSkillsResponse> {
    let base = api_base_url();
    crate::chat::transport_wasm::fetch_installed_skills(&base).await
}

#[cfg(target_arch = "wasm32")]
pub async fn uninstall_installed_skill(slug: &str) -> anyhow::Result<()> {
    let base = api_base_url();
    crate::chat::transport_wasm::uninstall_installed_skill(&base, slug).await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn uninstall_installed_skill(slug: &str) -> anyhow::Result<()> {
    crate::desktop::agent::runtime_ctx()?
        .installed_skill_uninstall(slug)
        .await
}

#[cfg(target_arch = "wasm32")]
pub async fn load_roles() -> anyhow::Result<Vec<RoleDto>> {
    let base = api_base_url();
    crate::chat::transport_wasm::fetch_roles(&base).await
}

#[cfg(target_arch = "wasm32")]
pub async fn create_role(body: RoleCreateRequest) -> anyhow::Result<RoleDto> {
    let base = api_base_url();
    crate::chat::transport_wasm::create_role(&base, &body).await
}

#[cfg(target_arch = "wasm32")]
pub async fn update_role(id: i64, body: RoleUpdateRequest) -> anyhow::Result<RoleDto> {
    let base = api_base_url();
    crate::chat::transport_wasm::update_role(&base, id, &body).await
}

#[cfg(target_arch = "wasm32")]
pub async fn activate_role(id: i64) -> anyhow::Result<RoleDto> {
    let base = api_base_url();
    crate::chat::transport_wasm::activate_role(&base, id).await
}

#[cfg(target_arch = "wasm32")]
pub async fn delete_role(id: i64) -> anyhow::Result<()> {
    let base = api_base_url();
    crate::chat::transport_wasm::delete_role(&base, id).await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn run_chat_turn<F>(
    req: ChatTurnRequest,
    abort: std::sync::Arc<crate::desktop::agent::ChatAbort>,
    mut on_event: F,
) -> anyhow::Result<()>
where
    F: FnMut(SseEvent),
{
    crate::desktop::agent::desktop_chat_stream(req, abort, &mut on_event).await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn list_conversations() -> anyhow::Result<Vec<ConversationSummaryDto>> {
    crate::desktop::agent::runtime_ctx()?
        .conversations_list()
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn create_conversation() -> anyhow::Result<ConversationSummaryDto> {
    crate::desktop::agent::runtime_ctx()?
        .conversation_create()
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn update_conversation_title(
    conversation_id: String,
    title: String,
) -> anyhow::Result<ConversationSummaryDto> {
    crate::desktop::agent::runtime_ctx()?
        .conversation_title_update(&conversation_id, &title)
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn delete_conversation(conversation_id: String) -> anyhow::Result<()> {
    crate::desktop::agent::runtime_ctx()?
        .conversation_delete(&conversation_id)
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn load_conversation_messages(
    conversation_id: String,
    limit: usize,
) -> anyhow::Result<Vec<StoredChatMessageDto>> {
    crate::desktop::agent::runtime_ctx()?
        .conversation_messages(&conversation_id, limit)
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn load_agent_run_detail(run_id: i64) -> anyhow::Result<AgentRunDetailDto> {
    crate::desktop::agent::runtime_ctx()?
        .agent_run_detail(run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("agent run not found: {run_id}"))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn load_skill_market(query: String) -> anyhow::Result<SkillMarketResponse> {
    crate::desktop::agent::runtime_ctx()?
        .skill_market_search(Some(query))
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn install_skill(
    source: String,
    id: String,
    name: Option<String>,
    url: Option<String>,
) -> anyhow::Result<SkillInstallResponse> {
    crate::desktop::agent::runtime_ctx()?
        .skill_install(protocol::SkillInstallRequest {
            source,
            id,
            name,
            url,
        })
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn load_equipped_skills() -> anyhow::Result<EquippedSkillsDto> {
    crate::desktop::agent::runtime_ctx()?
        .equipped_skills_get()
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn toggle_skill_equip(slug: String, equipped: bool) -> anyhow::Result<EquippedSkillsDto> {
    crate::desktop::agent::runtime_ctx()?
        .equipped_skills_toggle(protocol::SkillEquipToggleRequest { slug, equipped })
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn load_installed_skills() -> anyhow::Result<InstalledSkillsResponse> {
    crate::desktop::agent::runtime_ctx()?
        .installed_skills_list()
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn load_roles() -> anyhow::Result<Vec<RoleDto>> {
    crate::desktop::agent::runtime_ctx()?.roles_list().await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn create_role(body: RoleCreateRequest) -> anyhow::Result<RoleDto> {
    crate::desktop::agent::runtime_ctx()?.role_create(body).await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn update_role(id: i64, body: RoleUpdateRequest) -> anyhow::Result<RoleDto> {
    crate::desktop::agent::runtime_ctx()?
        .role_update(id, body)
        .await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn activate_role(id: i64) -> anyhow::Result<RoleDto> {
    crate::desktop::agent::runtime_ctx()?.role_activate(id).await
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn delete_role(id: i64) -> anyhow::Result<()> {
    crate::desktop::agent::runtime_ctx()?.role_delete(id).await
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn run_chat_turn<F>(_req: ChatTurnRequest, _on_event: F) -> anyhow::Result<()>
where
    F: FnMut(SseEvent),
{
    anyhow::bail!("当前构建未启用 `native` feature，无法用桌面直连模式运行 Agent")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn list_conversations() -> anyhow::Result<Vec<ConversationSummaryDto>> {
    anyhow::bail!("当前构建未启用 `native` feature，无法读取会话")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn create_conversation() -> anyhow::Result<ConversationSummaryDto> {
    anyhow::bail!("当前构建未启用 `native` feature，无法创建会话")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn update_conversation_title(
    _conversation_id: String,
    _title: String,
) -> anyhow::Result<ConversationSummaryDto> {
    anyhow::bail!("当前构建未启用 `native` feature，无法更新会话标题")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn delete_conversation(_conversation_id: String) -> anyhow::Result<()> {
    anyhow::bail!("当前构建未启用 `native` feature，无法删除会话")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn load_conversation_messages(
    _conversation_id: String,
    _limit: usize,
) -> anyhow::Result<Vec<StoredChatMessageDto>> {
    anyhow::bail!("当前构建未启用 `native` feature，无法读取会话消息")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn load_agent_run_detail(_run_id: i64) -> anyhow::Result<AgentRunDetailDto> {
    anyhow::bail!("当前构建未启用 `native` feature，无法读取 Agent 详情")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn load_skill_market(_query: String) -> anyhow::Result<SkillMarketResponse> {
    anyhow::bail!("当前构建未启用 `native` feature，无法读取技能市场")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn install_skill(
    _source: String,
    _id: String,
    _name: Option<String>,
    _url: Option<String>,
) -> anyhow::Result<SkillInstallResponse> {
    anyhow::bail!("当前构建未启用 `native` feature，无法安装技能")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn load_equipped_skills() -> anyhow::Result<EquippedSkillsDto> {
    anyhow::bail!("当前构建未启用 `native` feature，无法读取装备技能")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn toggle_skill_equip(
    _slug: String,
    _equipped: bool,
) -> anyhow::Result<EquippedSkillsDto> {
    anyhow::bail!("当前构建未启用 `native` feature，无法切换装备技能")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn load_installed_skills() -> anyhow::Result<InstalledSkillsResponse> {
    anyhow::bail!("当前构建未启用 `native` feature，无法读取已挂载技能")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn uninstall_installed_skill(_slug: &str) -> anyhow::Result<()> {
    anyhow::bail!("当前构建未启用 `native` feature，无法卸载技能")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn load_roles() -> anyhow::Result<Vec<RoleDto>> {
    anyhow::bail!("当前构建未启用 `native` feature，无法读取角色")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn create_role(_body: RoleCreateRequest) -> anyhow::Result<RoleDto> {
    anyhow::bail!("当前构建未启用 `native` feature，无法创建角色")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn update_role(_id: i64, _body: RoleUpdateRequest) -> anyhow::Result<RoleDto> {
    anyhow::bail!("当前构建未启用 `native` feature，无法更新角色")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn activate_role(_id: i64) -> anyhow::Result<RoleDto> {
    anyhow::bail!("当前构建未启用 `native` feature，无法激活角色")
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub async fn delete_role(_id: i64) -> anyhow::Result<()> {
    anyhow::bail!("当前构建未启用 `native` feature，无法删除角色")
}
