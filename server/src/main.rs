//! claw-rs HTTP API：Web 端通过此进程调用 [`shared::RuntimeContext`]。
#![forbid(unsafe_code)]

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::http::Method;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use protocol::{
    AboutInfoDto, AgentRunDetailDto, ChatTurnRequest,
    ConversationSummaryDto, ConversationTitleBody, CredentialStatusDto, CredentialUpsertRequest,
    EquippedSkillsDto, InstalledSkillsResponse, LiveAssetTradeHistoryDto, LiveAssetsDashboardDto, LlmConfigDto,
    LlmConfigUpsertBody, LlmCredentialUpsertBody, LlmStatusBody, PersonaBody, PersonaPolishRequest, PersonaPolishResponse,
    RoleCreateRequest, RoleDto, RoleUpdateRequest,
    ServerIpDto, SkillEquipToggleRequest, SkillInstallRequest, SkillInstallResponse,
    SkillMarketQuery, SkillMarketResponse, SseEvent,
    StoredChatMessageDto, StrategyCreateRequest, StrategyDto, StrategyEvent, StrategyRunDto,
    StrategySemanticParseRequest, StrategyUpdateRequest,
};
use serde::Deserialize;
use shared::{RuntimeContext, ERR_MODEL_REQUIRED};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, instrument, warn};

/// claw-rs server 共享上下文。
#[derive(Clone)]
struct AppState {
    ctx: RuntimeContext,
}

#[derive(Debug, Deserialize)]
struct MessagesQuery {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct StrategyRunsQuery {
    limit: Option<usize>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./data/server.db".into());
    let openai = std::env::var("OPENAI_API_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty());
    std::fs::create_dir_all(
        PathBuf::from(&db_path)
            .parent()
            .unwrap_or(std::path::Path::new(".")),
    )?;

    let ctx = RuntimeContext::open(&db_path, openai.clone())?;

    info!(
        db_path = %db_path,
        openai_configured = openai.is_some(),
        "claw-rs-api-server starting"
    );

    let state = AppState { ctx };
    shared::strategy_scheduler::spawn_strategy_scheduler(state.ctx.clone());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/about", get(about_info))
        .route("/v1/llm/status", get(llm_status))
        .route("/v1/llm/config", get(llm_config_get).put(llm_config_put))
        .route("/v1/llm/credentials", post(llm_credential_upsert))
        .route("/v1/llm/credentials/:id", delete(llm_credential_delete))
        .route("/v1/llm/credentials/:id/activate", post(llm_credential_activate))
        .route("/v1/persona", get(persona_get).put(persona_put))
        .route("/v1/persona/polish", post(persona_polish))
        .route("/v1/roles", get(roles_list).post(roles_create))
        .route("/v1/roles/:id", put(roles_update).delete(roles_delete))
        /* Web WASM 用 POST 规避部分宿主对 DELETE 的 405；与 `transport_wasm::delete_role` 对齐 */
        .route("/v1/roles/:id/delete", post(roles_delete))
        .route("/v1/roles/:id/activate", put(roles_activate))
        .route(
            "/v1/chat/conversations",
            get(chat_conversations_list).post(chat_conversation_create),
        )
        .route(
            "/v1/chat/conversations/:id/messages",
            get(chat_conversation_messages),
        )
        .route(
            "/v1/chat/conversations/:id",
            delete(chat_conversation_delete),
        )
        .route(
            "/v1/chat/conversations/:id/title",
            put(chat_conversation_title_update),
        )
        .route("/v1/chat/agent-runs/:id", get(chat_agent_run_detail))
        .route("/v1/chat/stream", post(chat_stream))
        .route("/v1/system/server-ip", get(system_server_ip))
        .route("/v1/skills/market", get(skills_market))
        .route("/v1/skills/install", post(skills_install))
        .route(
            "/v1/skills/equipped",
            get(skills_equipped_get).post(skills_equipped_post),
        )
        .route(
            "/v1/skills/installed/:slug",
            delete(skills_installed_delete),
        )
        .route("/v1/skills/installed", get(skills_installed_get))
        .route("/v1/credentials", post(credentials_upsert))
        .route(
            "/v1/credentials/:exchange",
            get(credentials_get).delete(credentials_delete),
        )
        .route("/v1/assets/:exchange", get(assets_get))
        .route("/v1/assets/:exchange/trades", get(asset_trades_get))
        .route(
            "/v1/strategies/semantic-parse",
            post(strategies_semantic_parse),
        )
        .route(
            "/v1/strategies",
            get(strategies_list).post(strategies_create),
        )
        .route(
            "/v1/strategies/:id",
            put(strategies_update).delete(strategies_delete),
        )
        .route("/v1/strategy-runs", get(strategy_runs_list))
        .route("/v1/strategies/events", get(strategy_events_sse))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8787".into())
        .parse()?;
    info!("listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

fn read_deploy_version() -> String {
    if let Ok(path) = std::env::var("VERSION_FILE") {
        if let Ok(raw) = std::fs::read_to_string(&path) {
            let v = raw.trim();
            if !v.is_empty() {
                // 纯版本字符串（历史兼容）或 Cargo.toml
                if v.contains("[package]") {
                    if let Some(parsed) = parse_cargo_package_version(v) {
                        return parsed;
                    }
                } else {
                    return v.to_string();
                }
            }
        }
    }
    parse_cargo_package_version(include_str!("../../desktop/Cargo.toml"))
        .unwrap_or_else(|| "unknown".to_string())
}

fn parse_cargo_package_version(toml: &str) -> Option<String> {
    let mut in_package = false;
    for line in toml.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some(rest) = line.strip_prefix("version") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let v = rest.trim().trim_matches('"').trim();
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

async fn about_info() -> Json<AboutInfoDto> {
    Json(AboutInfoDto {
        version: read_deploy_version(),
        runtime_os: std::env::consts::OS.to_string(),
        runtime_arch: std::env::consts::ARCH.to_string(),
    })
}

/// GET `/v1/system/server-ip`：返回当前部署服务器公网 IP（10 分钟内存缓存）。
///
/// 用于「配置 API」弹窗提示用户把该 IP 加到交易所 API 密钥的 IP 白名单。
/// 上游探测失败统一返回 502，前端据此展示降级提示。
#[instrument(skip(state))]
async fn system_server_ip(
    State(state): State<AppState>,
) -> Result<Json<ServerIpDto>, StatusCode> {
    match state.ctx.server_public_ip().await {
        Ok(ip) => Ok(Json(ServerIpDto { ip })),
        Err(e) => {
            warn!(error = %e, "server_public_ip 查询失败");
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

#[instrument(skip(state))]
async fn llm_status(State(state): State<AppState>) -> Result<Json<LlmStatusBody>, StatusCode> {
    let db_cfg = state
        .ctx
        .llm_config_get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(LlmStatusBody {
        server_openai_configured: db_cfg.api_key_configured,
    }))
}

#[instrument(skip(state))]
async fn llm_config_get(State(state): State<AppState>) -> Result<Json<LlmConfigDto>, StatusCode> {
    let cfg = state
        .ctx
        .llm_config_get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(cfg))
}

#[instrument(skip(state))]
async fn llm_config_put(
    State(state): State<AppState>,
    Json(body): Json<LlmConfigUpsertBody>,
) -> Result<Json<LlmConfigDto>, StatusCode> {
    let cfg = state
        .ctx
        .llm_config_upsert(body)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(cfg))
}

#[instrument(skip(state))]
async fn llm_credential_upsert(
    State(state): State<AppState>,
    Json(body): Json<LlmCredentialUpsertBody>,
) -> Result<Json<LlmConfigDto>, StatusCode> {
    let cfg = state
        .ctx
        .llm_credential_upsert(body)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(cfg))
}

#[instrument(skip(state))]
async fn llm_credential_delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<LlmConfigDto>, StatusCode> {
    let cfg = state
        .ctx
        .llm_credential_delete(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(cfg))
}

#[instrument(skip(state))]
async fn llm_credential_activate(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<LlmConfigDto>, StatusCode> {
    let cfg = state
        .ctx
        .llm_credential_activate(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(cfg))
}

#[instrument(skip(state))]
async fn persona_get(State(state): State<AppState>) -> Result<Json<PersonaBody>, StatusCode> {
    let text = state
        .ctx
        .persona_get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(PersonaBody {
        system_prompt: text,
    }))
}

#[instrument(skip(state))]
async fn persona_put(
    State(state): State<AppState>,
    Json(body): Json<PersonaBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .ctx
        .persona_set(&body.system_prompt)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[instrument(skip(state))]
async fn persona_polish(
    State(state): State<AppState>,
    Json(body): Json<PersonaPolishRequest>,
) -> Result<Json<PersonaPolishResponse>, (StatusCode, Json<serde_json::Value>)> {
    match state.ctx.persona_polish(&body.system_prompt, &body.model).await {
        Ok(polished) => Ok(Json(PersonaPolishResponse {
            system_prompt: polished,
        })),
        Err(e) => {
            let msg = e.to_string();
            warn!(%msg, model = %body.model, "persona_polish failed");
            let status = if msg.contains(ERR_MODEL_REQUIRED)
                || msg.contains("openai_api_key_missing")
                || msg.contains("persona_prompt_empty")
            {
                StatusCode::BAD_REQUEST
            } else if msg.contains("上游 API HTTP") {
                StatusCode::BAD_GATEWAY
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((
                status,
                Json(serde_json::json!({ "error": msg })),
            ))
        }
    }
}

#[instrument(skip(state))]
async fn roles_list(State(state): State<AppState>) -> Result<Json<Vec<RoleDto>>, StatusCode> {
    let roles = state
        .ctx
        .roles_list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(roles))
}

#[instrument(skip(state, body))]
async fn roles_create(
    State(state): State<AppState>,
    Json(body): Json<RoleCreateRequest>,
) -> Result<Json<RoleDto>, StatusCode> {
    let role = state
        .ctx
        .role_create(body)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(role))
}

#[instrument(skip(state, body))]
async fn roles_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<RoleUpdateRequest>,
) -> Result<Json<RoleDto>, StatusCode> {
    let role = state.ctx.role_update(id, body).await.map_err(|e| {
        if e.to_string().contains("role not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    Ok(Json(role))
}

#[instrument(skip(state))]
async fn roles_activate(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<RoleDto>, StatusCode> {
    let role = state.ctx.role_activate(id).await.map_err(|e| {
        if e.to_string().contains("role not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    Ok(Json(role))
}

#[instrument(skip(state))]
async fn roles_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.ctx.role_delete(id).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("role not found") {
            StatusCode::NOT_FOUND
        } else if msg.contains("role_delete:last_role") {
            StatusCode::CONFLICT
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[instrument(skip(state))]
async fn chat_conversations_list(
    State(state): State<AppState>,
) -> Result<Json<Vec<ConversationSummaryDto>>, StatusCode> {
    let conversations = state
        .ctx
        .conversations_list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(conversations))
}

#[instrument(skip(state))]
async fn chat_conversation_create(
    State(state): State<AppState>,
) -> Result<Json<ConversationSummaryDto>, StatusCode> {
    let conversation = state
        .ctx
        .conversation_create()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(conversation))
}

#[instrument(skip(state))]
async fn chat_conversation_messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<Vec<StoredChatMessageDto>>, StatusCode> {
    let messages = state
        .ctx
        .conversation_messages(&id, query.limit.unwrap_or(200).min(500))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(messages))
}

#[instrument(skip(state, body))]
async fn chat_conversation_title_update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ConversationTitleBody>,
) -> Result<Json<ConversationSummaryDto>, StatusCode> {
    let conversation = state
        .ctx
        .conversation_title_update(&id, &body.title)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(conversation))
}

#[instrument(skip(state))]
async fn chat_conversation_delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .ctx
        .conversation_delete(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[instrument(skip(state))]
async fn chat_agent_run_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<AgentRunDetailDto>, StatusCode> {
    let detail = state
        .ctx
        .agent_run_detail(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(detail))
}

#[instrument(skip(state))]
async fn skills_market(
    State(state): State<AppState>,
    Query(query): Query<SkillMarketQuery>,
) -> Result<Json<SkillMarketResponse>, StatusCode> {
    let market = state
        .ctx
        .skill_market_search(query.q)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(Json(market))
}

#[instrument(skip(state, body))]
async fn skills_install(
    State(state): State<AppState>,
    Json(body): Json<SkillInstallRequest>,
) -> Result<Json<SkillInstallResponse>, StatusCode> {
    let installed = state
        .ctx
        .skill_install(body)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(Json(installed))
}

async fn skills_equipped_get(
    State(state): State<AppState>,
) -> Result<Json<EquippedSkillsDto>, StatusCode> {
    let dto = state
        .ctx
        .equipped_skills_get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto))
}

async fn skills_equipped_post(
    State(state): State<AppState>,
    Json(body): Json<SkillEquipToggleRequest>,
) -> Result<Json<EquippedSkillsDto>, StatusCode> {
    let dto = state
        .ctx
        .equipped_skills_toggle(body)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto))
}

async fn skills_installed_get(
    State(state): State<AppState>,
) -> Result<Json<InstalledSkillsResponse>, StatusCode> {
    let dto = state
        .ctx
        .installed_skills_list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto))
}

async fn skills_installed_delete(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .ctx
        .installed_skill_uninstall(&slug)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("无效的技能") || msg.contains("不存在")
            {
                (StatusCode::BAD_REQUEST, msg)
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        })?;
    Ok(StatusCode::NO_CONTENT)
}

async fn credentials_upsert(
    State(state): State<AppState>,
    Json(body): Json<CredentialUpsertRequest>,
) -> Result<Json<CredentialStatusDto>, (StatusCode, String)> {
    let dto = state.ctx.credential_upsert(body).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("不支持的交易所") || msg.contains("不能为空") {
            (StatusCode::BAD_REQUEST, msg)
        } else if msg.contains("base64 解析失败") || msg.contains("长度必须为 32") {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "cred:master_key_invalid".to_string(),
            )
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, msg)
        }
    })?;
    Ok(Json(dto))
}

async fn credentials_get(
    State(state): State<AppState>,
    Path(exchange): Path<String>,
) -> Result<Json<CredentialStatusDto>, (StatusCode, String)> {
    let dto = state
        .ctx
        .credential_status_get(&exchange)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("不支持的交易所") {
                (StatusCode::BAD_REQUEST, msg)
            } else if msg.contains("base64 解析失败") || msg.contains("长度必须为 32") {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "cred:master_key_invalid".to_string(),
                )
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        })?;
    Ok(Json(dto))
}

async fn credentials_delete(
    State(state): State<AppState>,
    Path(exchange): Path<String>,
) -> Result<Json<CredentialStatusDto>, (StatusCode, String)> {
    let dto = state.ctx.credential_delete(&exchange).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("不支持的交易所") {
            (StatusCode::BAD_REQUEST, msg)
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, msg)
        }
    })?;
    Ok(Json(dto))
}

async fn assets_get(
    State(state): State<AppState>,
    Path(exchange): Path<String>,
) -> Result<Json<LiveAssetsDashboardDto>, (StatusCode, String)> {
    let dto = state.ctx.live_assets(&exchange).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("不支持的交易所")
            || msg.contains("尚未配置凭证")
            || msg.contains("缺少凭证字段")
            || msg.contains("401")
            || msg.contains("Unauthorized")
            || msg.contains("Invalid")
            || msg.contains("签名")
        {
            (StatusCode::BAD_REQUEST, msg)
        } else {
            (StatusCode::BAD_GATEWAY, msg)
        }
    })?;
    Ok(Json(dto))
}

async fn asset_trades_get(
    State(state): State<AppState>,
    Path(exchange): Path<String>,
) -> Result<Json<LiveAssetTradeHistoryDto>, (StatusCode, String)> {
    let dto = state.ctx.live_asset_trade_history(&exchange).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("不支持的交易所")
            || msg.contains("尚未配置凭证")
            || msg.contains("缺少凭证字段")
            || msg.contains("401")
            || msg.contains("Unauthorized")
            || msg.contains("Invalid")
            || msg.contains("签名")
        {
            (StatusCode::BAD_REQUEST, msg)
        } else {
            (StatusCode::BAD_GATEWAY, msg)
        }
    })?;
    Ok(Json(dto))
}

async fn strategies_semantic_parse(
    State(state): State<AppState>,
    Json(body): Json<StrategySemanticParseRequest>,
) -> Sse<UnboundedReceiverStream<Result<Event, Infallible>>> {
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event, Infallible>>();
    let ctx = state.ctx.clone();

    tokio::spawn(async move {
        let (sse_tx, mut sse_rx) = tokio::sync::mpsc::unbounded_channel::<SseEvent>();
        tokio::spawn(async move {
            let _ = ctx.strategy_semantic_parse_stream(body, sse_tx).await;
        });

        while let Some(ev) = sse_rx.recv().await {
            let data = serde_json::to_string(&ev).unwrap_or_else(|_| "{}".to_string());
            if event_tx.send(Ok(Event::default().data(data))).is_err() {
                break;
            }
        }
    });

    Sse::new(UnboundedReceiverStream::new(event_rx)).keep_alive(KeepAlive::default())
}

async fn strategies_list(
    State(state): State<AppState>,
) -> Result<Json<Vec<StrategyDto>>, StatusCode> {
    let list = state
        .ctx
        .strategies_list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(list))
}

async fn strategies_create(
    State(state): State<AppState>,
    Json(body): Json<StrategyCreateRequest>,
) -> Result<Json<StrategyDto>, (StatusCode, Json<serde_json::Value>)> {
    match state.ctx.strategy_create(body).await {
        Ok(created) => Ok(Json(created)),
        Err(e) => {
            let msg = e.to_string();
            let status = if msg.contains(ERR_MODEL_REQUIRED) {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((status, Json(serde_json::json!({ "error": msg }))))
        }
    }
}

async fn strategies_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<StrategyUpdateRequest>,
) -> Result<Json<StrategyDto>, StatusCode> {
    let updated = state.ctx.strategy_update(id, body).await.map_err(|e| {
        if e.to_string().contains("strategy not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    Ok(Json(updated))
}

async fn strategies_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.ctx.strategy_delete(id).await.map_err(|e| {
        if e.to_string().contains("strategy not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn strategy_runs_list(
    State(state): State<AppState>,
    Query(q): Query<StrategyRunsQuery>,
) -> Result<Json<Vec<StrategyRunDto>>, StatusCode> {
    let limit = q.limit.unwrap_or(50).clamp(1, 500);
    let list = state
        .ctx
        .strategy_runs_list(limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(list))
}

/// GET `/v1/strategies/events`：SSE 推送策略事件，UI 据此触发增量拉取，替代 30s 整页轮询。
///
/// 工作流：把 `RuntimeContext::subscribe_strategy_events()` 的 broadcast 接收端转成 SSE。
/// 若接收端落后（broadcast `Lagged`），转发一条 `StrategyEvent::Resync` 让客户端做一次全量拉取。
async fn strategy_events_sse(
    State(state): State<AppState>,
) -> Sse<UnboundedReceiverStream<Result<Event, Infallible>>> {
    let mut rx = state.ctx.subscribe_strategy_events();
    let (event_tx, event_rx) =
        tokio::sync::mpsc::unbounded_channel::<Result<Event, Infallible>>();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(ev) => {
                    let data = serde_json::to_string(&ev).unwrap_or_else(|_| "{}".to_string());
                    if event_tx.send(Ok(Event::default().data(data))).is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_n)) => {
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;
                    let resync = StrategyEvent::Resync { ts };
                    let data = serde_json::to_string(&resync).unwrap_or_else(|_| "{}".to_string());
                    if event_tx.send(Ok(Event::default().data(data))).is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
    Sse::new(UnboundedReceiverStream::new(event_rx)).keep_alive(KeepAlive::default())
}

fn optional_header_string(headers: &HeaderMap, name: &'static str) -> Option<String> {
    headers
        .get(name)?
        .to_str()
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

async fn chat_stream(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChatTurnRequest>,
) -> Sse<UnboundedReceiverStream<Result<Event, Infallible>>> {
    let client_openai_key = optional_header_string(&headers, "x-openai-api-key");
    let client_openai_base = optional_header_string(&headers, "x-openai-base-url");
    // 默认模式下 wasm 客户端会每次 chat 都从 relay `/v1/me/api-credentials` 拉
    // 新鲜的 sk-xxx + base 通过这两个头注入；自定义模式则两者都缺省。打点
    // `from_client` 字段方便 deploy / 凭证轮换后排查 401 是 server 还在用陈旧 DB
    // key 还是 relay 端拒识。不打印 key 明文，只看「有没有」。
    let client_key_present = client_openai_key.is_some();
    let client_base_present = client_openai_base.is_some();
    tracing::debug!(
        client_openai_key_present = client_key_present,
        client_openai_base_present = client_base_present,
        "chat_stream got request"
    );
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<Result<Event, Infallible>>();
    let ctx = state.ctx.clone();

    tokio::spawn(async move {
        let (sse_tx, mut sse_rx) = tokio::sync::mpsc::unbounded_channel::<SseEvent>();
        let run_ctx = ctx.clone();
        tokio::spawn(async move {
            let _ = run_ctx
                .run_chat_turn_with_client_headers(
                    req,
                    sse_tx,
                    client_openai_key,
                    client_openai_base,
                )
                .await;
        });

        while let Some(ev) = sse_rx.recv().await {
            let data = serde_json::to_string(&ev).unwrap_or_else(|_| "{}".to_string());
            if event_tx.send(Ok(Event::default().data(data))).is_err() {
                break;
            }
        }
    });

    Sse::new(UnboundedReceiverStream::new(event_rx)).keep_alive(KeepAlive::default())
}
