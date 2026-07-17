//! Open façade: same Rust API as before, backed by closed `libpusa_core` cdylib.

mod ffi;

pub mod db;

pub use protocol;

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{broadcast, mpsc};

use protocol::StrategyEvent;

pub const ERR_MODEL_REQUIRED: &str = "model_required";

pub fn require_model_id(model: &str) -> Result<&str> {
    let trimmed = model.trim();
    if trimmed.is_empty() {
        anyhow::bail!("{ERR_MODEL_REQUIRED}: 请在请求中指定 model（模型 ID 不能为空）");
    }
    Ok(trimmed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyFrequencyInferDetail {
    pub seconds: i64,
    pub openai_v1_base_used: String,
    pub llm_http_requested: bool,
    pub http_status: Option<u16>,
    pub raw_assistant_text: Option<String>,
    pub extracted_integer: Option<i64>,
    pub trace: String,
}

#[derive(Clone)]
pub struct RuntimeContext {
    handle: Arc<ffi::Handle>,
    strategy_tx: broadcast::Sender<StrategyEvent>,
}

impl RuntimeContext {
    pub fn open(path: impl AsRef<Path>, openai_api_key: Option<String>) -> Result<Self> {
        let handle = ffi::open(path.as_ref(), openai_api_key)?;
        let (strategy_tx, _) = broadcast::channel(256);
        let tx = strategy_tx.clone();
        ffi::start_event_pump(&handle, move |json| {
            if let Ok(ev) = serde_json::from_str::<StrategyEvent>(json) {
                let _ = tx.send(ev);
            }
        });
        Ok(Self {
            handle,
            strategy_tx,
        })
    }

    pub fn subscribe_strategy_events(&self) -> broadcast::Receiver<StrategyEvent> {
        self.strategy_tx.subscribe()
    }

    async fn call_json_async<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        method: &'static str,
        args: serde_json::Value,
    ) -> Result<T> {
        let h = Arc::clone(&self.handle);
        tokio::task::spawn_blocking(move || ffi::call_json(&h, method, args))
            .await?
    }

    async fn call_json_opt_async<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        method: &'static str,
        args: serde_json::Value,
    ) -> Result<Option<T>> {
        let h = Arc::clone(&self.handle);
        tokio::task::spawn_blocking(move || ffi::call_json_opt(&h, method, args))
            .await?
    }

    async fn call_unit_async(&self, method: &'static str, args: serde_json::Value) -> Result<()> {
        let h = Arc::clone(&self.handle);
        tokio::task::spawn_blocking(move || ffi::call_unit(&h, method, args))
            .await?
    }

    pub async fn server_public_ip(&self) -> Result<String> {
        self.call_json_async("server_public_ip", ffi::args_empty()).await
    }

    pub fn openai_key_configured(&self) -> bool {
        ffi::call_json::<bool>(&self.handle, "openai_key_configured", ffi::args_empty())
            .unwrap_or(false)
    }

    pub async fn llm_config_get(&self) -> Result<protocol::LlmConfigDto> {
        self.call_json_async("llm_config_get", ffi::args_empty()).await
    }

    pub async fn llm_config_upsert(
        &self,
        body: protocol::LlmConfigUpsertBody,
    ) -> Result<protocol::LlmConfigDto> {
        self.call_json_async("llm_config_upsert", json!({ "body": body }))
            .await
    }

    pub async fn persona_get(&self) -> Result<String> {
        self.call_json_async("persona_get", ffi::args_empty()).await
    }

    pub async fn persona_set(&self, text: &str) -> Result<()> {
        self.call_unit_async("persona_set", json!({ "text": text })).await
    }

    pub async fn plugin_ai_search(
        &self,
        query: &str,
        model: &str,
    ) -> Result<Vec<protocol::PluginAiSearchItemDto>> {
        self.call_json_async(
            "plugin_ai_search",
            json!({ "query": query, "model": model }),
        )
        .await
    }

    pub async fn plugin_smart_ui_generate(
        &self,
        project_context: &str,
        model: &str,
    ) -> Result<protocol::PluginSmartUiDto> {
        self.call_json_async(
            "plugin_smart_ui_generate",
            json!({ "project_context": project_context, "model": model }),
        )
        .await
    }

    pub async fn persona_polish(&self, text: &str, model: &str) -> Result<String> {
        self.call_json_async(
            "persona_polish",
            json!({ "text": text, "model": model }),
        )
        .await
    }

    pub async fn strategy_semantic_parse_stream(
        &self,
        req: protocol::StrategySemanticParseRequest,
        tx: mpsc::UnboundedSender<protocol::SseEvent>,
    ) -> Result<()> {
        let h = Arc::clone(&self.handle);
        tokio::task::spawn_blocking(move || {
            ffi::call_stream(
                &h,
                "strategy_semantic_parse_stream",
                json!({ "req": req }),
                move |js| {
                    if let Ok(ev) = serde_json::from_str::<protocol::SseEvent>(js) {
                        let _ = tx.send(ev);
                    }
                },
            )
        })
        .await?
    }

    pub async fn roles_list(&self) -> Result<Vec<protocol::RoleDto>> {
        self.call_json_async("roles_list", ffi::args_empty()).await
    }

    pub async fn role_create(&self, req: protocol::RoleCreateRequest) -> Result<protocol::RoleDto> {
        self.call_json_async("role_create", json!({ "req": req })).await
    }

    pub async fn role_update(
        &self,
        role_id: i64,
        req: protocol::RoleUpdateRequest,
    ) -> Result<protocol::RoleDto> {
        self.call_json_async("role_update", json!({ "role_id": role_id, "req": req }))
            .await
    }

    pub async fn role_activate(&self, role_id: i64) -> Result<protocol::RoleDto> {
        self.call_json_async("role_activate", json!({ "role_id": role_id }))
            .await
    }

    pub async fn role_delete(&self, role_id: i64) -> Result<()> {
        self.call_unit_async("role_delete", json!({ "role_id": role_id }))
            .await
    }

    pub async fn conversation_create(&self) -> Result<protocol::ConversationSummaryDto> {
        self.call_json_async("conversation_create", ffi::args_empty())
            .await
    }

    pub async fn conversation_title_update(
        &self,
        conversation_id: &str,
        title: &str,
    ) -> Result<protocol::ConversationSummaryDto> {
        self.call_json_async(
            "conversation_title_update",
            json!({ "conversation_id": conversation_id, "title": title }),
        )
        .await
    }

    pub async fn conversation_delete(&self, conversation_id: &str) -> Result<()> {
        self.call_unit_async(
            "conversation_delete",
            json!({ "conversation_id": conversation_id }),
        )
        .await
    }

    pub async fn conversations_list(&self) -> Result<Vec<protocol::ConversationSummaryDto>> {
        self.call_json_async("conversations_list", ffi::args_empty())
            .await
    }

    pub async fn conversation_messages(
        &self,
        conversation_id: &str,
        limit: usize,
    ) -> Result<Vec<protocol::StoredChatMessageDto>> {
        self.call_json_async(
            "conversation_messages",
            json!({ "conversation_id": conversation_id, "limit": limit }),
        )
        .await
    }

    pub async fn conversation_recent_messages(
        &self,
        conversation_id: &str,
        limit: usize,
    ) -> Result<Vec<(String, String)>> {
        self.call_json_async(
            "conversation_recent_messages",
            json!({ "conversation_id": conversation_id, "limit": limit }),
        )
        .await
    }

    pub async fn agent_run_detail(
        &self,
        run_id: i64,
    ) -> Result<Option<protocol::AgentRunDetailDto>> {
        self.call_json_opt_async("agent_run_detail", json!({ "run_id": run_id }))
            .await
    }

    pub async fn auth_any_wallet_exists(&self) -> Result<bool> {
        self.call_json_async("auth_any_wallet_exists", ffi::args_empty())
            .await
    }

    pub async fn auth_get_by_wallet(&self, wallet: &str) -> Result<Option<db::AuthAccount>> {
        self.call_json_opt_async("auth_get_by_wallet", json!({ "wallet": wallet }))
            .await
    }

    pub async fn auth_upsert_wallet(
        &self,
        wallet: &str,
        chain_id: Option<i64>,
        aliyun: Option<(String, String)>,
        token: Option<(String, String, i64)>,
        login_source: Option<&str>,
    ) -> Result<db::AuthAccount> {
        let aliyun_json = aliyun.map(|(a, b)| json!([a, b]));
        let token_json = token.map(|(a, b, c)| json!([a, b, c]));
        self.call_json_async(
            "auth_upsert_wallet",
            json!({
                "wallet": wallet,
                "chain_id": chain_id,
                "aliyun": aliyun_json,
                "token": token_json,
                "login_source": login_source,
            }),
        )
        .await
    }

    pub async fn auth_clear_tokens(&self, wallet: &str) -> Result<()> {
        self.call_unit_async("auth_clear_tokens", json!({ "wallet": wallet }))
            .await
    }

    pub async fn skill_market_search(
        &self,
        query: Option<String>,
    ) -> Result<protocol::SkillMarketResponse> {
        self.call_json_async("skill_market_search", json!({ "query": query }))
            .await
    }

    pub async fn skill_install(
        &self,
        req: protocol::SkillInstallRequest,
    ) -> Result<protocol::SkillInstallResponse> {
        self.call_json_async("skill_install", json!({ "req": req }))
            .await
    }

    pub async fn equipped_skills_get(&self) -> Result<protocol::EquippedSkillsDto> {
        self.call_json_async("equipped_skills_get", ffi::args_empty())
            .await
    }

    pub async fn equipped_skills_toggle(
        &self,
        body: protocol::SkillEquipToggleRequest,
    ) -> Result<protocol::EquippedSkillsDto> {
        self.call_json_async("equipped_skills_toggle", json!({ "body": body }))
            .await
    }

    pub async fn installed_skills_list(&self) -> Result<protocol::InstalledSkillsResponse> {
        self.call_json_async("installed_skills_list", ffi::args_empty())
            .await
    }

    pub async fn installed_skill_uninstall(&self, slug: &str) -> Result<()> {
        self.call_unit_async("installed_skill_uninstall", json!({ "slug": slug }))
            .await
    }

    pub async fn strategies_list(&self) -> Result<Vec<protocol::StrategyDto>> {
        self.call_json_async("strategies_list", ffi::args_empty()).await
    }

    pub async fn strategy_create(
        &self,
        req: protocol::StrategyCreateRequest,
    ) -> Result<protocol::StrategyDto> {
        self.call_json_async("strategy_create", json!({ "req": req }))
            .await
    }

    pub async fn strategy_update(
        &self,
        strategy_id: i64,
        req: protocol::StrategyUpdateRequest,
    ) -> Result<protocol::StrategyDto> {
        self.call_json_async(
            "strategy_update",
            json!({ "strategy_id": strategy_id, "req": req }),
        )
        .await
    }

    pub async fn strategy_delete(&self, strategy_id: i64) -> Result<()> {
        self.call_unit_async("strategy_delete", json!({ "strategy_id": strategy_id }))
            .await
    }

    pub async fn due_running_strategies(&self, now_ts: i64) -> Result<Vec<protocol::StrategyDto>> {
        self.call_json_async("due_running_strategies", json!({ "now_ts": now_ts }))
            .await
    }

    pub async fn strategy_runs_list(&self, limit: usize) -> Result<Vec<protocol::StrategyRunDto>> {
        self.call_json_async("strategy_runs_list", json!({ "limit": limit }))
            .await
    }

    pub async fn mark_strategy_run_started(&self, strategy_id: i64, now_ts: i64) -> Result<()> {
        self.call_unit_async(
            "mark_strategy_run_started",
            json!({ "strategy_id": strategy_id, "now_ts": now_ts }),
        )
        .await
    }

    pub async fn claim_strategy_run_lock(&self, strategy_id: i64, now_ts: i64) -> Result<bool> {
        self.call_json_async(
            "claim_strategy_run_lock",
            json!({ "strategy_id": strategy_id, "now_ts": now_ts }),
        )
        .await
    }

    pub async fn create_strategy_run(&self, strategy_id: i64, now_ts: i64) -> Result<i64> {
        self.call_json_async(
            "create_strategy_run",
            json!({ "strategy_id": strategy_id, "now_ts": now_ts }),
        )
        .await
    }

    pub async fn complete_strategy_run(
        &self,
        strategy_run_id: i64,
        now_ts: i64,
        ok: bool,
        error: &str,
        summary: &str,
    ) -> Result<()> {
        self.call_unit_async(
            "complete_strategy_run",
            json!({
                "strategy_run_id": strategy_run_id,
                "now_ts": now_ts,
                "ok": ok,
                "error": error,
                "summary": summary,
            }),
        )
        .await
    }

    pub async fn last_agent_run_status_for_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<Option<String>> {
        self.call_json_opt_async(
            "last_agent_run_status_for_conversation",
            json!({ "conversation_id": conversation_id }),
        )
        .await
    }

    pub async fn mark_strategy_run_finished(
        &self,
        strategy_id: i64,
        now_ts: i64,
        ok: bool,
        detail: &str,
    ) -> Result<()> {
        self.call_unit_async(
            "mark_strategy_run_finished",
            json!({
                "strategy_id": strategy_id,
                "now_ts": now_ts,
                "ok": ok,
                "detail": detail,
            }),
        )
        .await
    }

    pub async fn credential_upsert(
        &self,
        req: protocol::CredentialUpsertRequest,
    ) -> Result<protocol::CredentialStatusDto> {
        self.call_json_async("credential_upsert", json!({ "req": req }))
            .await
    }

    pub async fn credential_status_get(
        &self,
        exchange: &str,
    ) -> Result<protocol::CredentialStatusDto> {
        self.call_json_async("credential_status_get", json!({ "exchange": exchange }))
            .await
    }

    pub async fn credential_delete(&self, exchange: &str) -> Result<protocol::CredentialStatusDto> {
        self.call_json_async("credential_delete", json!({ "exchange": exchange }))
            .await
    }

    pub async fn live_assets(&self, exchange: &str) -> Result<protocol::LiveAssetsDashboardDto> {
        self.call_json_async("live_assets", json!({ "exchange": exchange }))
            .await
    }

    pub async fn live_asset_trade_history(
        &self,
        exchange: &str,
    ) -> Result<protocol::LiveAssetTradeHistoryDto> {
        self.call_json_async(
            "live_asset_trade_history",
            json!({ "exchange": exchange }),
        )
        .await
    }

    pub async fn credential_envs_for_working_dir(
        &self,
        working_dir: &str,
    ) -> Result<Vec<(String, String)>> {
        self.call_json_async(
            "credential_envs_for_working_dir",
            json!({ "working_dir": working_dir }),
        )
        .await
    }

    pub async fn run_chat_turn(
        &self,
        req: protocol::ChatTurnRequest,
        tx: mpsc::UnboundedSender<protocol::SseEvent>,
    ) -> Result<()> {
        self.run_chat_turn_with_client_headers(req, tx, None, None)
            .await
    }

    pub async fn run_chat_turn_with_client_headers(
        &self,
        req: protocol::ChatTurnRequest,
        tx: mpsc::UnboundedSender<protocol::SseEvent>,
        client_api_key: Option<String>,
        client_openai_v1_base: Option<String>,
    ) -> Result<()> {
        let h = Arc::clone(&self.handle);
        tokio::task::spawn_blocking(move || {
            ffi::call_stream(
                &h,
                "run_chat_turn_with_client_headers",
                json!({
                    "req": req,
                    "client_api_key": client_api_key,
                    "client_openai_v1_base": client_openai_v1_base,
                }),
                move |js| {
                    if let Ok(ev) = serde_json::from_str::<protocol::SseEvent>(js) {
                        let _ = tx.send(ev);
                    }
                },
            )
        })
        .await?
    }

    pub async fn infer_strategy_frequency_seconds_with_ai(&self, prompt: &str, model: &str) -> i64 {
        self.call_json_async::<i64>(
            "infer_strategy_frequency_seconds_with_ai",
            json!({ "prompt": prompt, "model": model }),
        )
        .await
        .unwrap_or(300)
    }

    pub async fn infer_strategy_frequency_seconds_with_ai_detail(
        &self,
        prompt: &str,
        model: &str,
    ) -> StrategyFrequencyInferDetail {
        self.call_json_async(
            "infer_strategy_frequency_seconds_with_ai_detail",
            json!({ "prompt": prompt, "model": model }),
        )
        .await
        .unwrap_or(StrategyFrequencyInferDetail {
            seconds: 300,
            openai_v1_base_used: String::new(),
            llm_http_requested: false,
            http_status: None,
            raw_assistant_text: None,
            extracted_integer: None,
            trace: "facade fallback".into(),
        })
    }
}

pub mod strategy_scheduler {
    use super::RuntimeContext;

    pub fn spawn_strategy_scheduler(ctx: RuntimeContext) {
        super::ffi::spawn_scheduler(&ctx.handle);
    }
}

pub mod tools {
    pub mod file {
        use std::ffi::OsString;
        use std::path::PathBuf;

        pub fn project_root() -> PathBuf {
            super::super::ffi::project_root()
        }

        pub fn enriched_path() -> OsString {
            super::super::ffi::enriched_path()
        }
    }
}
