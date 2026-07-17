//! JSON / SSE 负载类型：Web、Desktop（protocol）、Server、Shared 共用。

use serde::{Deserialize, Serialize};

/// 单条对话消息（请求体内）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessageDto {
    pub role: String,
    pub content: String,
    /// 可选图片（`data:image/...;base64,...`），供视觉模型走多模态 content。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<String>,
}

/// 客户端发起一轮补全（可附带历史与系统提示）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTurnRequest {
    /// 会话 ID，默认 `default`。
    #[serde(default)]
    pub conversation_id: Option<String>,
    pub messages: Vec<ChatMessageDto>,
    #[serde(default)]
    pub system: Option<String>,
    /// LLM 模型 ID（必填，由前端选择后传入）。
    pub model: String,
    /// Agent 工具循环最大步数；缺省为 12（普通聊天）。自动化策略等可传更大值。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_agent_steps: Option<u32>,
}

/// 会话列表中的摘要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationSummaryDto {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: i64,
    pub last_message_preview: String,
}

/// PUT `/v1/chat/conversations/:id/title` 正文。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTitleBody {
    pub title: String,
}

/// 已持久化的聊天消息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredChatMessageDto {
    pub id: i64,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_run_id: Option<i64>,
}

/// Agent 运行详情。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRunDetailDto {
    pub id: i64,
    pub conversation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_message_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistant_message_id: Option<i64>,
    pub status: String,
    pub model: String,
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_duration_ms: Option<i64>,
    pub steps: Vec<AgentRunStepDetailDto>,
}

/// 单轮 Agent 循环详情。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRunStepDetailDto {
    pub id: i64,
    pub index: usize,
    pub phase: String,
    pub model_output: String,
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    pub tools: Vec<AgentToolCallDetailDto>,
}

/// 单个工具调用详情。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentToolCallDetailDto {
    pub id: i64,
    pub tool_call_id: String,
    pub name: String,
    pub args: String,
    pub result: String,
    pub summary: String,
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
}

/// SSE `data:` 行 JSON（与 UI 事件一一对应）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    AgentStepStart {
        index: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step_id: Option<i64>,
    },
    AgentThinking {
        index: usize,
    },
    AgentObserving {
        index: usize,
    },
    AgentStepDone {
        index: usize,
        #[serde(default)]
        model_output: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<i64>,
    },
    AgentFinalizing,
    /// 当前 Agent 步的模型输出增量（仅思考面板）。
    ThinkingDelta {
        step_index: usize,
        text: String,
    },
    /// 最终回复增量（主聊天气泡）。
    AnswerDelta {
        text: String,
    },
    /// 追加增量正文（兼容旧客户端；新实现请用 ThinkingDelta / AnswerDelta）。
    Delta {
        text: String,
    },
    ToolStart {
        id: String,
        name: String,
        #[serde(default)]
        args: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step_index: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tool_db_id: Option<i64>,
    },
    ToolResult {
        id: String,
        summary: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<i64>,
    },
    TurnPersisted {
        conversation_id: String,
        user_message_id: i64,
        assistant_message_id: i64,
        agent_run_id: i64,
    },
    Done,
    Error {
        message: String,
    },
}

/// GET/PUT `/v1/persona` 正文。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaBody {
    pub system_prompt: String,
}

/// POST `/v1/persona/polish` 请求体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaPolishRequest {
    pub system_prompt: String,
    /// LLM 模型 ID（必填，由前端选择后传入）。
    pub model: String,
}

/// POST `/v1/persona/polish` 响应体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaPolishResponse {
    pub system_prompt: String,
}

/// POST `/v1/plugins/ai-search` 请求体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAiSearchRequest {
    pub query: String,
    /// LLM 模型 ID（必填，由前端选择后传入）。
    pub model: String,
}

/// AI 搜索到的单个开源应用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginAiSearchItemDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub git_url: String,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

/// POST `/v1/plugins/ai-search` 响应体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAiSearchResponse {
    pub items: Vec<PluginAiSearchItemDto>,
}

/// 智能 UI 中的可执行动作（启动项目 / 跑脚本等）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSmartUiActionDto {
    pub id: String,
    pub label: String,
    /// 在插件根目录执行的 shell 命令（如 `npm start`、`cargo run`）。
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
    /// 长驻进程（dev server 等）为 true：后台启动，不阻塞等待退出。
    #[serde(default)]
    pub detached: bool,
}

/// 插件目录内 `.pusa-smart-ui.json` / LLM 生成的可视化操作面板。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSmartUiDto {
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub actions: Vec<PluginSmartUiActionDto>,
    #[serde(default)]
    pub generated_at: Option<i64>,
}

/// 角色实体（多 persona）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleDto {
    pub id: i64,
    pub name: String,
    pub system_prompt: String,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// POST `/v1/roles` 正文。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleCreateRequest {
    pub name: String,
    pub system_prompt: String,
}

/// PUT `/v1/roles/:id` 正文（部分更新）；删除角色另见 `DELETE /v1/roles/:id` 与 `POST /v1/roles/:id/delete`（后者供 Web 规避 405）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleUpdateRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
}

/// GET `/v1/auth/status`：reown 模式下恒返回 `true`，仅作探活；具体登录走客户端直连
/// Another-Me-LLM 的 `wallet/challenge` → `wallet/verify` 流程。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatusBody {
    pub registered: bool,
}

/// GET `/v1/auth/me`：透传 Another-Me-LLM 后端的 `/api/cloud/auth/me` 响应，描述当前
/// access token 所属账户与登录场景。`login_source == "ecs"` 时附带 `instance_id` /
/// `access_ip`；`"desktop"` 表示桌面端登录（来源 IP 不命中任何已纳管 ECS）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMeDto {
    /// 上游 `accountId`，形如 `wallet:0x…`。
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// 上游 session 主键。
    #[serde(rename = "sessionId", default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// 登录场景：`"ecs"` 或 `"desktop"`。
    #[serde(rename = "loginSource", default, skip_serializing_if = "Option::is_none")]
    pub login_source: Option<String>,
    /// 阿里云 ECS instance id（仅 `login_source == "ecs"` 时存在）。
    #[serde(rename = "instanceId", default, skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// 阿里云 ECS 公网入口 IP（仅 `login_source == "ecs"` 时存在）。
    #[serde(rename = "accessIp", default, skip_serializing_if = "Option::is_none")]
    pub access_ip: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub perms: Vec<String>,
    /// access token 过期 ISO-8601 时间。
    #[serde(rename = "expiresAt", default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// 上游 `cloud_users` 文档（含 walletAddress 等字段），作为透明 JSON 直接转发。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<serde_json::Value>,
}

/// GET `/v1/about`：本机/部署信息（版本来自根目录 `.version` 或 `VERSION_FILE`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AboutInfoDto {
    pub version: String,
    pub runtime_os: String,
    pub runtime_arch: String,
}

/// GET `/v1/system/server-ip`：当前部署服务器的公网 IP（由后端调 ip.sb 查询并缓存）。
///
/// 用于交易所 API 弹窗向用户提示"将该 IP 添加到对应 API 密钥的白名单"。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerIpDto {
    pub ip: String,
}

/// GET `/v1/llm/status`：服务端是否已配置 OpenAI（环境变量或数据库）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStatusBody {
    pub server_openai_configured: bool,
}

/// 聊天模型选择器的一项。
///
/// 数据来自 Another-Me-Relay 的 OpenAI 兼容接口 `GET /v1/models`
///（形如 `{ "data": [ { "id": "gpt-5.4", ... } ] }`）。
/// 鉴权版 `GET /v1/me/models` 还会带 `display_name`（admin 在
/// `/admin/model_prices` 录入的描述）；公开列表通常没有该字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatModelDto {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// `GET …/v1/models` 响应外壳。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatModelsResponse {
    #[serde(default)]
    pub data: Vec<ChatModelDto>,
}

/// GET `/v1/llm/config`：当前 LLM 配置（来自服务端数据库；已登录用户可读回已存密钥）。
///
/// `prefer_custom_key` 用来持久化「设置弹窗 → 自定义 API Key」switch 的开关状态。
/// 单纯靠 `api_key` 与平台 token prefix 比对会出现死循环：用户关掉 switch 但 `api_key`
/// 仍是自定义值时，下次进入设置会再次被推断成「自定义」。新增此字段后，UI 的初始
/// switch 状态完全由本字段决定，与 `api_key` 的内容解耦。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfigDto {
    pub api_key_configured: bool,
    /// 已保存的 API Key；未配置时为空字符串。
    pub api_key: String,
    pub openai_v1_base: String,
    /// 设置弹窗 switch 状态：true=自定义模式，false=自动（平台分配）模式。
    /// 旧后端不返回此字段时按 `false`（默认自动模式）反序列化。
    #[serde(default)]
    pub prefer_custom_key: bool,
}

/// PUT `/v1/llm/config`：更新服务端数据库中的 LLM 配置。
///
/// `prefer_custom_key` 为 `None` 时表示「不触动 switch 偏好」，仅写入 `api_key` /
/// `openai_v1_base`。前端在 toggle 切换时显式传 `Some(true/false)`，常规保存（点
/// 「保存」按钮）传 `None` 以免误覆盖。
///
/// `clear_api_key` / `clear_openai_v1_base`：显式声明「我要清空这个字段」。后端的
/// 写入规则是：
/// - 字段非空 → 直接写入；
/// - 字段为空 + `clear_*=true` → 把 DB 里的旧值真正覆盖为空字符串（下次读出来
///   等价于"无配置"）；
/// - 字段为空 + `clear_*=false`（**默认**） → 维持「空=no-op」的兼容语义，DB 旧值
///   保留不动。这条默认行为保护「toggle switch 偏好时不动 api_key/base」等只更新
///   一部分字段的调用路径（典型如 `save_llm_prefer_custom_key`）。
///
/// 旧前端不传这两个字段时按 `false` 反序列化，与历史行为一致。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfigUpsertBody {
    pub api_key: String,
    pub openai_v1_base: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefer_custom_key: Option<bool>,
    #[serde(default, skip_serializing_if = "is_false_default")]
    pub clear_api_key: bool,
    #[serde(default, skip_serializing_if = "is_false_default")]
    pub clear_openai_v1_base: bool,
}

#[inline]
fn is_false_default(b: &bool) -> bool {
    !*b
}

/// POST `/v1/auth/register`、POST `/v1/auth/login` 请求体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentialsBody {
    pub username: String,
    pub password: String,
}

/// 注册或登录成功后返回 JWT。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokenBody {
    pub token: String,
}

/// 技能市场来源。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillRegistryDto {
    pub id: String,
    pub name: String,
    pub homepage: String,
}

/// 技能市场卡片。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillMarketItemDto {
    pub source: String,
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_url: Option<String>,
    /// 技能封面图 URL（绝对地址）；缺省或空串时 UI 使用默认占位图标。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    pub downloads: i64,
    pub stars: i64,
    pub version: String,
    pub installed: bool,
    pub installable: bool,
}

/// GET `/v1/skills/market` 查询参数。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillMarketQuery {
    #[serde(default)]
    pub q: Option<String>,
}

/// GET `/v1/skills/market` 响应。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillMarketResponse {
    pub registries: Vec<SkillRegistryDto>,
    pub items: Vec<SkillMarketItemDto>,
    pub warnings: Vec<String>,
}

/// POST `/v1/skills/install` 正文。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillInstallRequest {
    pub source: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// POST `/v1/skills/install` 响应。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillInstallResponse {
    pub ok: bool,
    pub name: String,
    pub path: String,
    pub message: String,
}

/// GET `/v1/skills/equipped` 与装备操作后的响应：已标记为优先进入 Agent 检索的技能目录名（与 `skills/<slug>` 对应）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquippedSkillsDto {
    pub slugs: Vec<String>,
}

/// POST `/v1/skills/equipped` 正文：切换单个技能的「装备」状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillEquipToggleRequest {
    pub slug: String,
    pub equipped: bool,
}

/// 本地已挂载技能（仅技能市场下载目录，与项目 `skills/` 无关）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledSkillDto {
    pub slug: String,
    pub title: String,
    pub description: String,
    /// `http(s)://...` 或同源 API 路径（如 `/v1/skills/installed/{slug}/icon`）；缺省或空串时用占位图标。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

/// GET `/v1/skills/installed` 响应。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledSkillsResponse {
    pub skills: Vec<InstalledSkillDto>,
}

/// POST `/v1/credentials`：保存指定交易所凭证。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialUpsertRequest {
    pub exchange: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binance_api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binance_secret_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_api_key: Option<String>,
}

/// GET `/v1/credentials/:exchange`：仅返回是否已配置与更新时间（不返回明文）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialStatusDto {
    pub exchange: String,
    pub configured: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}

fn default_strategy_llm_model() -> String {
    "gpt-5.5".to_string()
}

/// 自动化策略（交易 / 雷达）持久化实体。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyDto {
    pub id: i64,
    pub view: String,
    pub exchange: String,
    pub title: String,
    pub prompt: String,
    pub frequency_seconds: i64,
    pub status: String,
    pub last_event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_run_at: Option<i64>,
    /// 该策略调度执行时使用的 LLM 模型 ID。
    #[serde(default = "default_strategy_llm_model")]
    pub model: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 策略执行记录（用于执行看板）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyRunDto {
    pub id: i64,
    pub strategy_id: i64,
    pub strategy_title: String,
    pub strategy_view: String,
    pub strategy_exchange: String,
    pub status: String,
    pub started_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<i64>,
    pub error: String,
    pub summary: String,
}

/// 策略事件总线消息：`shared` 在 mutation 时广播，server 通过 SSE 转给 UI、desktop 直接订阅。
///
/// UI 收到事件即可有选择地拉取最新 `strategies` / `strategy_runs`，无需再做 30 秒整页轮询。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum StrategyEvent {
    /// 策略 CRUD（创建 / 更新 / 删除）发生。UI 应重新拉 `strategies`。
    StrategyChanged {
        strategy_id: i64,
        ts: i64,
    },
    /// 调度器为某策略开了一次 run。UI 应重新拉 `strategies` 与 `strategy_runs`。
    RunStarted {
        strategy_id: i64,
        strategy_run_id: i64,
        ts: i64,
    },
    /// run 完成（成功或失败）。UI 应重新拉 `strategies` 与 `strategy_runs`。
    RunFinished {
        strategy_id: i64,
        strategy_run_id: i64,
        ok: bool,
        ts: i64,
    },
    /// 订阅端检测到事件丢失（broadcast Lagged 等），通知 UI 做一次全量同步。
    Resync {
        ts: i64,
    },
}

/// POST `/v1/strategies/semantic-parse`（SSE）：根据构建器配置与用户描述流式生成策略提示词。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategySemanticParseRequest {
    /// `auto`（自动交易）或 `radar`（信号雷达）。
    pub view: String,
    pub exchange: String,
    pub scope_pairs: Vec<String>,
    pub frequency_seconds: i64,
    /// 执行模型 ID（与构建器「执行模型」一致）。
    pub model: String,
    /// 用户在大文本框中的自然语言描述（可为空）。
    #[serde(default)]
    pub user_input: String,
}

/// POST `/v1/strategies`：创建策略。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyCreateRequest {
    pub view: String,
    pub exchange: String,
    pub title: String,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_seconds: Option<i64>,
    pub status: String,
    pub last_event: String,
    /// 该策略调度执行时使用的 LLM 模型 ID（必填）。
    pub model: String,
}

/// PUT `/v1/strategies/:id`：部分更新策略。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyUpdateRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// 实盘资产列表中的单行。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveAssetRowDto {
    pub symbol: String,
    pub title: String,
    pub subtitle: String,
    pub value_usd: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pnl_pct: Option<f64>,
}

/// 实盘资产看板数据。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveAssetsDashboardDto {
    pub exchange: String,
    pub total_usd: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_pnl_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_pnl_pct: Option<f64>,
    pub rows: Vec<LiveAssetRowDto>,
    pub updated_at: i64,
}

/// 实盘资产看板「交易历史」列表中的单行。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveAssetTradeRowDto {
    pub market: String,
    pub symbol: String,
    pub side: String,
    pub price: f64,
    pub qty: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_qty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realized_pnl: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_asset: Option<String>,
    pub time: i64,
}

/// 实盘资产看板「交易历史」数据。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveAssetTradeHistoryDto {
    pub exchange: String,
    pub spot_rows: Vec<LiveAssetTradeRowDto>,
    pub futures_rows: Vec<LiveAssetTradeRowDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spot_note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub futures_note: Option<String>,
    pub updated_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_events_round_trip_with_metadata() {
        let ev = SseEvent::ToolResult {
            id: "call-1".into(),
            summary: "ok".into(),
            duration_ms: Some(42),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let decoded: SseEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, ev);
    }

    #[test]
    fn thinking_and_answer_delta_round_trip() {
        let thinking = SseEvent::ThinkingDelta {
            step_index: 2,
            text: "分析中…".into(),
        };
        let thinking_json = serde_json::to_string(&thinking).unwrap();
        assert_eq!(
            serde_json::from_str::<SseEvent>(&thinking_json).unwrap(),
            thinking
        );

        let done = SseEvent::AgentStepDone {
            index: 1,
            model_output: "step text".into(),
            duration_ms: Some(99),
        };
        let done_json = serde_json::to_string(&done).unwrap();
        assert_eq!(serde_json::from_str::<SseEvent>(&done_json).unwrap(), done);
    }

    #[test]
    fn agent_run_detail_round_trip() {
        let detail = AgentRunDetailDto {
            id: 1,
            conversation_id: "default".into(),
            user_message_id: Some(10),
            assistant_message_id: Some(11),
            status: "completed".into(),
            model: "gpt-4o-mini".into(),
            created_at: 100,
            completed_at: Some(101),
            total_duration_ms: Some(1200),
            steps: vec![AgentRunStepDetailDto {
                id: 2,
                index: 1,
                phase: "done".into(),
                model_output: "thinking".into(),
                created_at: 100,
                completed_at: Some(101),
                duration_ms: Some(1200),
                tools: vec![AgentToolCallDetailDto {
                    id: 3,
                    tool_call_id: "call-1".into(),
                    name: "read_file".into(),
                    args: "{\"path\":\"README.md\"}".into(),
                    result: "content".into(),
                    summary: "content".into(),
                    created_at: 100,
                    completed_at: Some(101),
                    duration_ms: Some(250),
                }],
            }],
        };
        let json = serde_json::to_string(&detail).unwrap();
        let decoded: AgentRunDetailDto = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, detail);
    }
}
