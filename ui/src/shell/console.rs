//! 主界面骨架，视觉与结构对齐 `UI-html/AnotherClaw/index.html` 中的 AI 控制台布局。

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use dioxus::events::FormEvent;
use dioxus::html::input_data::MouseButton;
use dioxus::html::point_interaction::ModifiersInteraction;
use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::BsStopFill;
#[cfg(all(target_arch = "wasm32", feature = "web"))]
use dioxus_free_icons::icons::bs_icons::{
    BsGear, BsLayoutSidebar, BsLayoutSidebarInset, BsLayoutSidebarInsetReverse,
    BsLayoutSidebarReverse,
};
use dioxus_free_icons::icons::ld_icons::{
    LdArrowUp, LdCheck, LdChevronDown, LdChevronUp, LdCopy, LdDownload, LdEye, LdGlobe,
    LdMessageCircle, LdPaperclip, LdPencil, LdPlus, LdSave, LdScrollText, LdSearch,
    LdShieldCheck, LdSparkles, LdTrash2, LdWandSparkles, LdX,
};
#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
use dioxus_free_icons::icons::ld_icons::LdChevronsUp;
use dioxus_free_icons::Icon;
use crate::icons::{
    PiMagicWandBold, PiMagicWandFill, RiApps2Fill, RiApps2Line, RiHomeSmile2Fill, RiHomeSmile2Line,
    TbFile, TbFileFilled,
};
#[cfg(all(target_arch = "wasm32", feature = "web"))]
use crate::icons::{
    VscLayoutPanel, VscLayoutPanelOff, VscLayoutSidebarLeftDock, VscLayoutSidebarRightDock,
};
use keyboard_types::{Key, Modifiers};

use crate::chat::{
    activate_role, attachment_from_file_data, create_conversation, create_role,
    delete_conversation, delete_role, install_skill, list_chat_models, list_conversations,
    load_agent_run_detail, load_conversation_messages, load_equipped_skills,
    load_installed_skills, load_roles, load_skill_market, run_chat_turn, toggle_skill_equip,
    uninstall_installed_skill, update_conversation_title, update_role, ChatMarkdownBody,
    ChatPendingAttachment, ChatThinkingHydrator, ChatThinkingPanel, StepPhase,
    ThinkingStatus, ToolStatus, TraceFileOpen, UiAgentThinking, UiChatMessage, UiThinkingStep,
    UiThinkingTool,
};
use crate::persona;
#[cfg(all(target_arch = "wasm32", feature = "web"))]
use crate::web::llm_config;
#[cfg(all(target_arch = "wasm32", feature = "web"))]
use crate::web::password_field::{AcPasswordInput, PasswordFieldStyle};
use protocol::{
    AgentRunDetailDto, ChatMessageDto, ChatTurnRequest, ConversationSummaryDto,
    InstalledSkillDto, RoleCreateRequest, RoleDto, RoleUpdateRequest, SkillMarketItemDto,
    SkillRegistryDto, SseEvent, StoredChatMessageDto,
};

/// 聊天栏顶部"模型选择"下拉的一项。
///
/// 数据来源：启动时用 [`fallback_chat_models`] 兜底；随后 `list_chat_models()`
/// 拉取 Relay 公开接口 `GET /v1/models`（web / desktop 均经 facade）。
/// - `id`：模型 ID，例如 `gpt-5.4`；既是按钮主标题（大写化后展示），也是
///   `chat_model` signal 存的值（与上游 `/v1/chat/completions` 的 `model`
///   参数一致）。
/// - `display_name`：可选副标题（鉴权版 `/v1/me/models` 的 admin 描述）；
///   公开列表通常为空，UI 不渲染副标题元素。
#[derive(Debug, Clone)]
struct ChatModelEntry {
    id: String,
    display_name: Option<String>,
}

/// API 失败或返回空列表时保留的本地默认，避免 picker 空白。
fn fallback_chat_models() -> Vec<ChatModelEntry> {
    vec![
        ChatModelEntry {
            id: DEFAULT_CHAT_MODEL_ID.into(),
            display_name: Some("稳定平衡".into()),
        },
        ChatModelEntry {
            id: "gpt-5.5".into(),
            display_name: Some("最新推理".into()),
        },
    ]
}

/// `search_market` 返回的按源前缀警告（见 `shared` 的 `skills` 模块）；非开发者模式隐藏底层错误串。
fn skill_market_warning_for_ui(warning: &str, developer_mode: bool) -> String {
    if developer_mode {
        return warning.to_string();
    }
    if warning.starts_with("ClawHub: ") {
        return "ClawHub：连接失败".to_string();
    }
    if warning.starts_with("SkillsMP: ") {
        return "SkillsMP：连接失败".to_string();
    }
    if warning.starts_with("腾讯 SkillHub: ") {
        return "腾讯 SkillHub：连接失败".to_string();
    }
    "技能市场：连接失败".to_string()
}

/// 整次市场请求失败时的顶层错误；非开发者模式不展示 anyhow / URL。
fn skill_market_fatal_error_for_ui(message: &str, developer_mode: bool) -> String {
    if developer_mode {
        return message.to_string();
    }
    "无法加载技能市场（连接失败）".to_string()
}

/// 侧栏「创建新角色」：在已有 `新角色01`…`新角色99` 中取最大序号 +1；忽略「新角色 2322」等非两位序号旧名。
fn next_new_role_display_name(roles: &[RoleDto]) -> String {
    const PREFIX: &str = "新角色";
    let mut max_n: u32 = 0;
    for r in roles {
        let name = r.name.trim();
        let Some(rest) = name.strip_prefix(PREFIX) else {
            continue;
        };
        let num_part = rest.trim_start();
        if num_part.len() == 2 && num_part.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(n) = num_part.parse::<u32>() {
                max_n = max_n.max(n);
            }
        }
    }
    let next = max_n.saturating_add(1);
    if next <= 99 {
        format!("新角色{:02}", next)
    } else {
        format!("新角色{}", next)
    }
}

#[derive(Clone, PartialEq)]
struct SkillMarketCardVm {
    item: SkillMarketItemDto,
    source_label: &'static str,
    downloads: String,
    stars: String,
    install_key: String,
}

/// 中间栏内容标签（角色设定 / 技能详情 / 应用详情 / 文件等可并存、可关闭）。
#[derive(Clone, PartialEq)]
enum CenterTabKind {
    Role,
    SkillInstalled { slug: String },
    SkillMarket { key: String },
    Plugin { key: String },
    File { path: String },
    /// 工具轨迹中某次编辑的前后对比（不写入操作缓存）。
    FileDiff {
        path: String,
        old_text: String,
        new_text: String,
    },
}

#[derive(Clone, PartialEq)]
struct CenterTab {
    id: String,
    title: String,
    kind: CenterTabKind,
}

/// 操作缓存：上次打开的中间栏标签 + 选中项（Web `localStorage` / 桌面 `center_tabs.json`）。
#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
struct CenterTabsCache {
    #[serde(default)]
    tabs: Vec<CenterTabCacheEntry>,
    #[serde(default)]
    active_id: Option<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct CenterTabCacheEntry {
    id: String,
    title: String,
    #[serde(flatten)]
    kind: CenterTabKindCache,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CenterTabKindCache {
    Role,
    SkillInstalled { slug: String },
    SkillMarket { key: String },
    Plugin { key: String },
    File { path: String },
}

impl CenterTabKindCache {
    fn try_from_kind(kind: &CenterTabKind) -> Option<Self> {
        match kind {
            CenterTabKind::Role => Some(Self::Role),
            CenterTabKind::SkillInstalled { slug } => Some(Self::SkillInstalled {
                slug: slug.clone(),
            }),
            CenterTabKind::SkillMarket { key } => Some(Self::SkillMarket { key: key.clone() }),
            CenterTabKind::Plugin { key } => Some(Self::Plugin { key: key.clone() }),
            CenterTabKind::File { path } => Some(Self::File { path: path.clone() }),
            CenterTabKind::FileDiff { .. } => None,
        }
    }
}

impl From<CenterTabKindCache> for CenterTabKind {
    fn from(kind: CenterTabKindCache) -> Self {
        match kind {
            CenterTabKindCache::Role => Self::Role,
            CenterTabKindCache::SkillInstalled { slug } => Self::SkillInstalled { slug },
            CenterTabKindCache::SkillMarket { key } => Self::SkillMarket { key },
            CenterTabKindCache::Plugin { key } => Self::Plugin { key },
            CenterTabKindCache::File { path } => Self::File { path },
        }
    }
}

impl CenterTabCacheEntry {
    fn try_from_tab(tab: &CenterTab) -> Option<Self> {
        Some(Self {
            id: tab.id.clone(),
            title: tab.title.clone(),
            kind: CenterTabKindCache::try_from_kind(&tab.kind)?,
        })
    }
}

impl From<CenterTabCacheEntry> for CenterTab {
    fn from(entry: CenterTabCacheEntry) -> Self {
        Self {
            id: entry.id,
            title: entry.title,
            kind: CenterTabKind::from(entry.kind),
        }
    }
}

/// 启动时从操作缓存恢复的中间栏会话（空 = 无缓存，保持首页空态）。
#[derive(Clone)]
struct CenterSessionRestore {
    tabs: Vec<CenterTab>,
    active_id: Option<String>,
    selected_skill_key: Option<String>,
    selected_plugin_key: Option<String>,
    selected_file_path: Option<String>,
}

fn read_center_tabs_cache_raw() -> Option<String> {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        return crate::web::prefs::center_tabs_get();
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        return crate::desktop::files::center_tabs_cache_get();
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        None
    }
}

fn write_center_tabs_cache_raw(raw: &str) {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        crate::web::prefs::center_tabs_set(raw);
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::files::center_tabs_cache_set(raw);
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        let _ = raw;
    }
}

fn load_center_session_restore() -> CenterSessionRestore {
    let empty = CenterSessionRestore {
        tabs: Vec::new(),
        active_id: None,
        selected_skill_key: None,
        selected_plugin_key: None,
        selected_file_path: None,
    };
    let Some(raw) = read_center_tabs_cache_raw() else {
        return empty;
    };
    let Ok(cache) = serde_json::from_str::<CenterTabsCache>(&raw) else {
        return empty;
    };
    if cache.tabs.is_empty() {
        return empty;
    }
    let tabs: Vec<CenterTab> = cache.tabs.into_iter().map(CenterTab::from).collect();
    let active_id = cache
        .active_id
        .filter(|id| tabs.iter().any(|t| t.id == *id))
        .or_else(|| tabs.first().map(|t| t.id.clone()));
    let active_kind = center_active_kind(&tabs, &active_id);
    let selected_skill_key = active_kind
        .as_ref()
        .and_then(center_tab_skill_key);
    let selected_plugin_key = active_kind
        .as_ref()
        .and_then(center_tab_plugin_key)
        .map(str::to_string);
    let selected_file_path = active_kind
        .as_ref()
        .and_then(center_tab_file_path)
        .map(str::to_string);
    CenterSessionRestore {
        tabs,
        active_id,
        selected_skill_key,
        selected_plugin_key,
        selected_file_path,
    }
}

fn persist_center_session(tabs: &[CenterTab], active_id: &Option<String>) {
    let cache_tabs: Vec<CenterTabCacheEntry> = tabs
        .iter()
        .filter_map(CenterTabCacheEntry::try_from_tab)
        .collect();
    if cache_tabs.is_empty() {
        write_center_tabs_cache_raw("");
        return;
    }
    let active_id = active_id
        .clone()
        .filter(|id| cache_tabs.iter().any(|t| t.id == *id));
    let cache = CenterTabsCache {
        tabs: cache_tabs,
        active_id,
    };
    if let Ok(raw) = serde_json::to_string(&cache) {
        write_center_tabs_cache_raw(&raw);
    }
}

const CENTER_TAB_ROLE_ID: &str = "role";

impl CenterTab {
    fn role() -> Self {
        Self {
            id: CENTER_TAB_ROLE_ID.to_string(),
            title: "角色设定".into(),
            kind: CenterTabKind::Role,
        }
    }

    fn skill_installed(slug: String, title: String) -> Self {
        let id = format!("skill:installed:{slug}");
        Self {
            id,
            title,
            kind: CenterTabKind::SkillInstalled { slug },
        }
    }

    fn skill_market(key: String, title: String) -> Self {
        let id = format!("skill:market:{key}");
        Self {
            id,
            title,
            kind: CenterTabKind::SkillMarket { key },
        }
    }

    fn plugin(key: String, title: String) -> Self {
        let id = format!("plugin:{key}");
        Self {
            id,
            title,
            kind: CenterTabKind::Plugin { key },
        }
    }

    fn file(path: String) -> Self {
        let title = super::files::fs_file_title(&path);
        let id = format!("file:{path}");
        Self {
            id,
            title,
            kind: CenterTabKind::File { path },
        }
    }

    fn file_diff(tool_id: String, path: String, old_text: String, new_text: String) -> Self {
        let name = super::files::fs_file_title(&path);
        let id = format!("file-diff:{tool_id}");
        Self {
            id,
            title: format!("编辑 · {name}"),
            kind: CenterTabKind::FileDiff {
                path,
                old_text,
                new_text,
            },
        }
    }
}

fn center_open_or_focus(tabs: &mut Vec<CenterTab>, active_id: &mut Option<String>, tab: CenterTab) {
    if let Some(existing) = tabs.iter_mut().find(|t| t.id == tab.id) {
        existing.title = tab.title;
        existing.kind = tab.kind;
        *active_id = Some(existing.id.clone());
        return;
    }
    *active_id = Some(tab.id.clone());
    tabs.push(tab);
}

fn center_close_tab(
    tabs: &mut Vec<CenterTab>,
    active_id: &mut Option<String>,
    id: &str,
) -> Option<CenterTab> {
    let idx = tabs.iter().position(|t| t.id == id)?;
    let removed = tabs.remove(idx);
    if active_id.as_deref() == Some(id) {
        *active_id = if tabs.is_empty() {
            None
        } else {
            Some(tabs[idx.min(tabs.len().saturating_sub(1))].id.clone())
        };
    }
    Some(removed)
}

fn center_active_kind(tabs: &[CenterTab], active_id: &Option<String>) -> Option<CenterTabKind> {
    let id = active_id.as_deref()?;
    tabs.iter().find(|t| t.id == id).map(|t| t.kind.clone())
}

fn center_tab_skill_key(kind: &CenterTabKind) -> Option<String> {
    match kind {
        CenterTabKind::SkillInstalled { slug } => Some(slug.clone()),
        CenterTabKind::SkillMarket { key } => Some(key.clone()),
        CenterTabKind::Role
        | CenterTabKind::Plugin { .. }
        | CenterTabKind::File { .. }
        | CenterTabKind::FileDiff { .. } => None,
    }
}

fn center_tab_plugin_key(kind: &CenterTabKind) -> Option<&str> {
    match kind {
        CenterTabKind::Plugin { key } => Some(key.as_str()),
        _ => None,
    }
}

fn center_tab_file_path(kind: &CenterTabKind) -> Option<&str> {
    match kind {
        CenterTabKind::File { path } => Some(path.as_str()),
        _ => None,
    }
}

/// 状态栏路径：普通文件或编辑对比标签。
fn center_tab_display_path(kind: &CenterTabKind) -> Option<&str> {
    match kind {
        CenterTabKind::File { path } | CenterTabKind::FileDiff { path, .. } => Some(path.as_str()),
        _ => None,
    }
}

/// 切换工作区根后刷新文件树信号，并切到「文件」侧栏。
fn apply_opened_project_root(
    root: String,
    mut fs_root_path: Signal<String>,
    mut fs_expanded: Signal<HashSet<String>>,
    mut fs_children_cache: Signal<
        HashMap<String, Result<Vec<super::files::FsEntryDto>, String>>,
    >,
    mut fs_selected_path: Signal<Option<String>>,
    mut fs_create_mode: Signal<Option<super::files::FsCreateKind>>,
    mut fs_create_name: Signal<String>,
    mut fs_create_parent: Signal<Option<String>>,
    mut fs_notice: Signal<Option<String>>,
    mut recent_project_dirs: Signal<Vec<String>>,
    mut active_tab: Signal<&'static str>,
    mut center_tabs: Signal<Vec<CenterTab>>,
    mut active_center_id: Signal<Option<String>>,
    mut home_logo_menu_open: Signal<bool>,
    mut plugin_refresh_tick: Signal<u64>,
) {
    use std::path::Path;
    fs_root_path.set(root.clone());
    fs_expanded.set(HashSet::new());
    fs_children_cache.set(HashMap::new());
    fs_selected_path.set(None);
    fs_create_mode.set(None);
    fs_create_name.set(String::new());
    fs_create_parent.set(None);
    fs_notice.set(None);
    recent_project_dirs.set(super::files::fs_recent_project_dirs());
    active_tab.set("files");
    home_logo_menu_open.set(false);
    center_tabs.with_mut(|tabs| {
        tabs.retain(|tab| match &tab.kind {
            CenterTabKind::File { path } | CenterTabKind::FileDiff { path, .. } => {
                Path::new(path).starts_with(Path::new(&root))
            }
            _ => true,
        });
    });
    let tabs = center_tabs();
    let active_ok = active_center_id()
        .as_ref()
        .is_some_and(|id| tabs.iter().any(|t| t.id == *id));
    if !active_ok {
        active_center_id.set(tabs.first().map(|t| t.id.clone()));
    }
    // 切根后重扫「我的应用」（例如从 monorepo 进入 application/{app}）。
    plugin_refresh_tick.with_mut(|n| *n = n.wrapping_add(1));
}

fn is_primary_modifier(mods: Modifiers) -> bool {
    mods.contains(Modifiers::SUPER)
        || mods.contains(Modifiers::CONTROL)
        || mods.contains(Modifiers::META)
}

fn is_mod_char_shortcut(e: &KeyboardEvent, ch: &str) -> bool {
    let Key::Character(c) = e.key() else {
        return false;
    };
    c.eq_ignore_ascii_case(ch) && is_primary_modifier(e.modifiers())
}

/// 新建 `extension/` 插件后：切到文件侧栏，展开并选中该目录；成功提示走全局 toast。
fn apply_created_extension_plugin(
    plugin_path: String,
    fs_root_path: Signal<String>,
    mut fs_expanded: Signal<HashSet<String>>,
    mut fs_children_cache: Signal<
        HashMap<String, Result<Vec<super::files::FsEntryDto>, String>>,
    >,
    mut fs_selected_path: Signal<Option<String>>,
    toast: super::toast::ToastCtx,
    mut active_tab: Signal<&'static str>,
    mut fs_section_open: Signal<bool>,
) {
    use std::path::Path;
    let root = fs_root_path();
    let extension_dir = Path::new(&root).join("extension");
    let extension_dir_s = extension_dir.to_string_lossy().into_owned();
    fs_expanded.with_mut(|open| {
        open.insert(root.clone());
        open.insert(extension_dir_s.clone());
    });
    fs_children_cache.with_mut(|cache| {
        super::files::fs_cache_reload(cache, &root);
        super::files::fs_cache_reload(cache, &extension_dir_s);
        super::files::fs_cache_reload(cache, &plugin_path);
    });
    fs_selected_path.set(Some(plugin_path.clone()));
    fs_section_open.set(true);
    active_tab.set("files");
    // 相对工作区路径，避免绝对路径在 toast 里折成「已创建插件：/」+ 下一行。
    let display = Path::new(&plugin_path)
        .strip_prefix(Path::new(&root))
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| plugin_path.replace('\\', "/"));
    toast.success(format!("已创建插件：{display}"));
}

/// 新建 `application/` 应用后：将该目录设为当前项目并打开；刷新应用列表。
fn apply_created_local_application(
    app_path: String,
    fs_root_path: Signal<String>,
    fs_expanded: Signal<HashSet<String>>,
    fs_children_cache: Signal<
        HashMap<String, Result<Vec<super::files::FsEntryDto>, String>>,
    >,
    fs_selected_path: Signal<Option<String>>,
    fs_create_mode: Signal<Option<super::files::FsCreateKind>>,
    fs_create_name: Signal<String>,
    fs_create_parent: Signal<Option<String>>,
    fs_notice: Signal<Option<String>>,
    recent_project_dirs: Signal<Vec<String>>,
    active_tab: Signal<&'static str>,
    center_tabs: Signal<Vec<CenterTab>>,
    active_center_id: Signal<Option<String>>,
    home_logo_menu_open: Signal<bool>,
    mut fs_section_open: Signal<bool>,
    toast: super::toast::ToastCtx,
    mut plugin_refresh_tick: Signal<u64>,
) {
    use std::path::Path;
    let display = Path::new(&app_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(app_path.as_str())
        .to_string();
    match super::files::fs_set_workspace_root(&app_path) {
        Ok(root) => {
            apply_opened_project_root(
                root,
                fs_root_path,
                fs_expanded,
                fs_children_cache,
                fs_selected_path,
                fs_create_mode,
                fs_create_name,
                fs_create_parent,
                fs_notice,
                recent_project_dirs,
                active_tab,
                center_tabs,
                active_center_id,
                home_logo_menu_open,
                plugin_refresh_tick,
            );
            fs_section_open.set(true);
            toast.success(format!("已创建并打开项目：{display}"));
        }
        Err(e) => {
            fs_section_open.set(true);
            plugin_refresh_tick.with_mut(|n| *n = n.wrapping_add(1));
            toast.error(format!("已创建应用，但打开项目失败：{e}"));
        }
    }
}

fn fs_forget_open_file(
    path: &str,
    drafts: &mut HashMap<String, String>,
    baselines: &mut HashMap<String, String>,
    dirty: &mut HashSet<String>,
    load_errors: &mut HashMap<String, String>,
) {
    drafts.remove(path);
    baselines.remove(path);
    dirty.remove(path);
    load_errors.remove(path);
}

/// 聊天点文件：显示侧栏 FILES、展开祖先目录、选中路径、打开中间栏。
/// 返回解析后的绝对/工作区路径；空串表示无效。
fn reveal_and_select_workspace_file(
    path: &str,
    mut show_sidebar: Signal<bool>,
    mut sidebar_shutting_down: Signal<bool>,
    mut show_center: Signal<bool>,
    mut center_shutting_down: Signal<bool>,
    mut active_tab: Signal<&'static str>,
    mut fs_section_open: Signal<bool>,
    fs_root_path: Signal<String>,
    mut fs_expanded: Signal<HashSet<String>>,
    mut fs_children_cache: Signal<
        HashMap<String, Result<Vec<super::files::FsEntryDto>, String>>,
    >,
    mut fs_selected_path: Signal<Option<String>>,
    mut fs_save_notice: Signal<Option<String>>,
) -> String {
    let resolved = super::files::fs_resolve_tool_path(path);
    let resolved = if resolved.is_empty() {
        path.trim().to_string()
    } else {
        resolved
    };
    if resolved.is_empty() {
        return String::new();
    }
    let root = fs_root_path();
    fs_children_cache.with_mut(|cache| {
        fs_expanded.with_mut(|open| {
            super::files::fs_reveal_in_tree(&root, &resolved, open, cache);
        });
    });
    fs_selected_path.set(Some(resolved.clone()));
    fs_save_notice.set(None);
    fs_section_open.set(true);
    active_tab.set("files");
    if sidebar_shutting_down() {
        sidebar_shutting_down.set(false);
    }
    show_sidebar.set(true);
    if center_shutting_down() {
        center_shutting_down.set(false);
    }
    show_center.set(true);
    resolved
}

/// 聊天消息列表滚动容器（`scrollTop` / `scrollHeight`）。
const AC_CHAT_THREAD_DOM_ID: &str = "ac-chat-thread";
/// 视为“仍在底部附近”的阈值；仅此时流式更新自动跟随到底部。
#[cfg(target_arch = "wasm32")]
const AC_CHAT_THREAD_STICKY_BOTTOM_PX: i32 = 72;

/// 侧栏默认宽度（像素），与 `main.css` 中 `.ac-sidebar` 一致。
const AC_SIDEBAR_DEFAULT_WIDTH_PX: f64 = 280.0;
/// 侧栏拖曳最小宽度（默认的约 2/3）；不再因拖窄而自动完全收起。
const AC_SIDEBAR_MIN_WIDTH_PX: f64 = AC_SIDEBAR_DEFAULT_WIDTH_PX * 2.0 / 3.0;
const AC_SIDEBAR_MAX_WIDTH_PX: f64 = AC_SIDEBAR_DEFAULT_WIDTH_PX;
/// 拖曳越过阈值后，侧栏宽度收至 0 的过渡时长（与 `.ac-sidebar-shutting` 的 CSS 一致）。
const AC_SIDEBAR_AUTO_COLLAPSE_ANIM_MS: u32 = 500;
/// 中间栏收起过渡时长（与 `.ac-center` / `.ac-chat` 的 CSS 一致）。
const AC_CENTER_AUTO_COLLAPSE_ANIM_MS: u32 = 220;

/// 对话区默认/最小/最大宽度（像素），与 `main.css` 中 `.ac-chat` 一致。
const AC_CHAT_DEFAULT_WIDTH_PX: f64 = 380.0;
const AC_CHAT_MIN_WIDTH_PX: f64 = 280.0;
const AC_CHAT_MAX_WIDTH_PX: f64 = 560.0;

/// 右侧会话历史栏：默认 / 可拖上限（与 `main.css` 一致）。
/// 最窄档见 [`chat_history_closed_width_px`]：必须跟 rem 基准走，写死 36px 在
/// 12–14px rem 下会多出数像素给标题，出现半个字残影。
const AC_CHAT_HISTORY_DEFAULT_WIDTH_PX: f64 = 256.0;
const AC_CHAT_HISTORY_MIN_WIDTH_PX: f64 = 140.0;
const AC_CHAT_HISTORY_MAX_WIDTH_PX: f64 = 360.0;
/// 接近最窄时吸附到纯图标宽，避免停在半个字中间态（滞回：展开须拖出该带）。
const AC_CHAT_HISTORY_SNAP_ZONE_PX: f64 = 20.0;

/// 与 `main.css` `html` rem 分档一致（12 / 14 / 16 / 17）。
fn root_font_size_px() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(rem) = document_root_font_size_px() {
            return rem;
        }
    }
    let w = console_layout_width_px();
    if w >= 2880.0 {
        17.0
    } else if w >= 2200.0 {
        16.0
    } else if w >= 1600.0 {
        14.0
    } else {
        12.0
    }
}

#[cfg(target_arch = "wasm32")]
fn document_root_font_size_px() -> Option<f64> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let el = document.document_element()?;
    let style = window.get_computed_style(&el).ok()??;
    let value = style.get_property_value("font-size").ok()?;
    let px = value.trim().trim_end_matches("px").trim().parse::<f64>().ok()?;
    (px.is_finite() && px > 0.0).then_some(px)
}

/// 最窄图标栏（px）：左右 inset 0.25rem + 图标 1.75rem + border-box 左边框 1px。
/// 与 item `padding` / `gap` 同值；`is-rail` 类彻底藏标题。
fn chat_history_closed_width_px() -> f64 {
    (2.25 * root_font_size_px() + 1.0).round()
}

/// 将拖拽宽度吸附到纯图标档（带滞回，避免卡在半露字）。
fn chat_history_snap_width(raw: f64, closed: f64, from_rail: bool) -> f64 {
    let raw = raw.clamp(closed, AC_CHAT_HISTORY_MAX_WIDTH_PX);
    let snap_at = closed + AC_CHAT_HISTORY_SNAP_ZONE_PX;
    if from_rail {
        // 从纯图标展开：必须拖出吸附带才离开，否则仍锁在 closed。
        if raw <= snap_at {
            closed
        } else {
            raw
        }
    } else if raw <= snap_at {
        closed
    } else {
        raw
    }
}

/// 中间栏底部终端默认/最小/最大高度（像素）。
const AC_TERMINAL_DEFAULT_HEIGHT_PX: f64 = 240.0;
const AC_TERMINAL_MIN_HEIGHT_PX: f64 = 120.0;
const AC_TERMINAL_MAX_HEIGHT_PX: f64 = 480.0;

#[cfg(target_arch = "wasm32")]
fn console_layout_width_px() -> f64 {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|v| v.as_f64())
        .filter(|w| w.is_finite() && *w > 0.0)
        .unwrap_or(1200.0)
}

#[cfg(not(target_arch = "wasm32"))]
fn console_layout_width_px() -> f64 {
    1200.0
}

/// 对话区拉到最大宽度时，中间栏剩余宽度（视为中间栏最小宽度）。
fn center_width_at_chat_max(show_sidebar: bool, sidebar_width: f64) -> f64 {
    let sidebar_w = if show_sidebar {
        sidebar_width
    } else {
        0.0
    };
    (console_layout_width_px() - sidebar_w - AC_CHAT_MAX_WIDTH_PX).max(0.0)
}

/// 中间栏最小宽度下再左拖该值的 1/3 即自动收起中间栏。
fn center_collapse_overshoot_px(show_sidebar: bool, sidebar_width: f64) -> f64 {
    center_width_at_chat_max(show_sidebar, sidebar_width) / 3.0
}

/// 当前 slug 是否在「已装备」列表中（与服务器 `EQUIPPED_SKILLS_KEY` 语义一致）。
/// 兼容全路径 key（如 `binance/spot`）与 basename（`spot`）的双向匹配。
fn skill_slug_equipped(equipped_slugs: &[String], slug: &str) -> bool {
    let slug_lc = slug.to_ascii_lowercase();
    let slug_base = slug_lc
        .rsplit('/')
        .next()
        .unwrap_or(slug_lc.as_str())
        .to_string();
    equipped_slugs.iter().any(|s| {
        let s_lc = s.to_ascii_lowercase();
        if s_lc == slug_lc {
            return true;
        }
        let s_base = s_lc.rsplit('/').next().unwrap_or(s_lc.as_str());
        s_base == slug_base.as_str()
    })
}

fn skill_equip_button_label(slugs: &[String], slug: &str) -> &'static str {
    if skill_slug_equipped(slugs, slug) {
        "忘却"
    } else {
        "学习"
    }
}

fn skill_source_label(source: &str) -> &'static str {
    match source {
        "clawhub" => "ClawHub",
        "skillsmp" => "SkillsMP",
        "tencent-skillhub" => "腾讯 SkillHub",
        _ => "技能市场",
    }
}

/// 技能市场下载站（与 `shared::skills::registries` 对齐）；API 未返回前作兜底。
fn default_skill_market_stations() -> Vec<SkillRegistryDto> {
    vec![
        SkillRegistryDto {
            id: "clawhub".into(),
            name: "ClawHub".into(),
            homepage: "https://clawhub.ai".into(),
        },
        SkillRegistryDto {
            id: "skillsmp".into(),
            name: "SkillsMP".into(),
            homepage: "https://skillsmp.com".into(),
        },
        SkillRegistryDto {
            id: "tencent-skillhub".into(),
            name: "腾讯 SkillHub".into(),
            homepage: "https://skillhub.tencent.com".into(),
        },
    ]
}

fn skill_station_warning_matches(warning: &str, station_id: &str) -> bool {
    match station_id {
        "clawhub" => warning.starts_with("ClawHub"),
        "skillsmp" => warning.starts_with("SkillsMP"),
        "tencent-skillhub" => warning.starts_with("腾讯 SkillHub"),
        _ => true,
    }
}

fn skill_metric(value: i64) -> String {
    if value >= 10_000 {
        format!("{:.1}w", value as f64 / 10_000.0)
    } else {
        value.to_string()
    }
}

/// 用户未做过任何选择时的兜底 chat 模型 ID（也用于服务端模型列表清空的极端情况）。
const DEFAULT_CHAT_MODEL_ID: &str = "gpt-5.4";

/// 聊天栏 chat_model 信号的初始值。
///
/// Web：优先复用 `localStorage` 里上次保存的选择；桌面 / 移动端无 prefs 存储，
/// 直接用 [`DEFAULT_CHAT_MODEL_ID`]。这样浏览器刷新或重开标签页后仍能记住模型。
fn initial_chat_model() -> String {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        if let Some(saved) = crate::web::prefs::chat_model_get() {
            return saved;
        }
    }
    DEFAULT_CHAT_MODEL_ID.to_string()
}

/// 将当前 chat 模型 ID 持久化到 `localStorage`（仅 web 生效，其它平台空实现）。
#[cfg_attr(not(all(target_arch = "wasm32", feature = "web")), allow(unused_variables))]
fn persist_chat_model(model_id: &str) {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        crate::web::prefs::chat_model_set(model_id);
    }
}

const AC_SIDEBAR_AUTO_COLLAPSE_YIELD_MS: u32 = 20;

/// 实盘资产「资产明细」每页条数（OKX 估值分项 + 持仓较多时分页展示）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum ConsoleResizeKind {
    Sidebar,
    Chat,
    ChatHistory,
    Terminal,
}

/// 当前消息线程是否仍贴近底部。
#[cfg(target_arch = "wasm32")]
fn chat_thread_is_near_bottom() -> bool {
    use wasm_bindgen::JsCast;
    let Some(w) = web_sys::window() else {
        return true;
    };
    let Some(doc) = w.document() else {
        return true;
    };
    let Some(el) = doc.get_element_by_id(AC_CHAT_THREAD_DOM_ID) else {
        return true;
    };
    let Ok(he) = el.dyn_into::<web_sys::HtmlElement>() else {
        return true;
    };
    let distance = he.scroll_height() - he.scroll_top() - he.client_height();
    distance <= AC_CHAT_THREAD_STICKY_BOTTOM_PX
}

#[cfg(not(target_arch = "wasm32"))]
fn chat_thread_is_near_bottom() -> bool {
    true
}

/// 在下一微任务将消息线程滚到底部（发送后、流式增量与切换会话时跟随最新消息）。
#[cfg(target_arch = "wasm32")]
fn schedule_chat_thread_scroll_bottom() {
    use wasm_bindgen::JsCast;
    wasm_bindgen_futures::spawn_local(async {
        let _ = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(
            &wasm_bindgen::JsValue::UNDEFINED,
        ))
        .await;
        let Some(w) = web_sys::window() else {
            return;
        };
        let Some(doc) = w.document() else {
            return;
        };
        let Some(el) = doc.get_element_by_id(AC_CHAT_THREAD_DOM_ID) else {
            return;
        };
        let Ok(he) = el.dyn_into::<web_sys::HtmlElement>() else {
            return;
        };
        he.set_scroll_top(he.scroll_height());
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn schedule_chat_thread_scroll_bottom() {}

#[cfg(target_arch = "wasm32")]
async fn ui_sleep_ms(ms: u32) {
    gloo_timers::future::TimeoutFuture::new(ms).await;
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
async fn ui_sleep_ms(ms: u32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
async fn ui_sleep_ms(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

const PET_PNG: Asset = asset!("/assets/images/pet.png");
const LOGO_PNG: Asset = asset!("/assets/images/Pusa.png");

/// Web：未配置服务端密钥且本地无 Key 时，在发送前弹出填写 OpenAI 凭证（非 Web 目标为空实现）。
#[cfg(all(target_arch = "wasm32", feature = "web"))]
#[component]
fn OpenAiSetupModal(
    mut modal_visible: Signal<bool>,
    mut api_key: Signal<String>,
    mut base: Signal<String>,
    mut llm_server_openai: Signal<Option<bool>>,
) -> Element {
    rsx! {
        if modal_visible() {
            div { class: "ac-api-modal-backdrop",
                div { class: "ac-api-modal",
                    h2 { class: "ac-api-modal-title", "配置 OpenAI" }
                    p { class: "ac-api-modal-desc",
                        "服务端未设置 OpenAI 配置。请填写 API Key 与兼容端点的 Base URL（须以 /v1 结尾）；配置将保存到服务端数据库。"
                    }
                    div { class: "ac-api-modal-field",
                        label { class: "ac-auth-label", "API Key"
                            AcPasswordInput {
                                placeholder: "sk-…",
                                value: api_key,
                                variant: PasswordFieldStyle::Modal,
                            }
                        }
                    }
                    div { class: "ac-api-modal-field",
                        label { class: "ac-auth-label", "API Base URL"
                            input {
                                r#type: "text",
                                class: "ac-api-modal-input",
                                placeholder: "https://api.openai.com/v1",
                                value: "{base()}",
                                oninput: move |e| base.set(e.value()),
                            }
                        }
                        p { class: "ac-api-modal-help", "留空 Base 则使用官方 https://api.openai.com/v1" }
                    }
                    div { class: "ac-api-modal-actions",
                        button {
                            r#type: "button",
                            class: "ac-api-modal-cancel",
                            onclick: move |_| modal_visible.set(false),
                            "取消"
                        }
                        button {
                            r#type: "button",
                            class: "ac-api-modal-submit",
                            onclick: move |_| {
                                let k = api_key().trim().to_string();
                                if k.is_empty() {
                                    return;
                                }
                                let b = base().trim().to_string();
                                let base_final = if b.is_empty() {
                                    llm_config::DEFAULT_OPENAI_V1_BASE.to_string()
                                } else {
                                    b
                                };
                                spawn(async move {
                                    if llm_config::save_llm_config(&k, &base_final).await.is_some() {
                                        modal_visible.set(false);
                                        llm_server_openai.set(Some(true));
                                    } else if let Some(v) = llm_config::fetch_server_openai_configured().await {
                                        llm_server_openai.set(Some(v));
                                    }
                                });
                            },
                            "保存并继续"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
#[component]
#[allow(unused_variables)]
fn OpenAiSetupModal(
    mut modal_visible: Signal<bool>,
    mut api_key: Signal<String>,
    mut base: Signal<String>,
    mut llm_server_openai: Signal<Option<bool>>,
) -> Element {
    rsx! {}
}

fn welcome_chat_messages() -> Vec<UiChatMessage> {
    vec![UiChatMessage::Assistant {
        id: None,
        content: "您好！我是 Pusa，有什么问题都可以问我。".into(),
        agent_run_id: None,
        thinking: UiAgentThinking::idle(),
    }]
}

fn stored_message_to_ui(message: StoredChatMessageDto) -> Option<UiChatMessage> {
    match message.role.as_str() {
        "user" => Some(UiChatMessage::User {
            id: Some(message.id),
            content: message.content,
            attachments: Vec::new(),
        }),
        "assistant" => Some(UiChatMessage::Assistant {
            id: Some(message.id),
            content: message.content,
            agent_run_id: message.agent_run_id,
            thinking: UiAgentThinking::idle(),
        }),
        _ => None,
    }
}

fn stored_messages_to_ui(messages: Vec<StoredChatMessageDto>) -> Vec<UiChatMessage> {
    let out = messages
        .into_iter()
        .filter_map(stored_message_to_ui)
        .collect::<Vec<_>>();
    if out.is_empty() {
        welcome_chat_messages()
    } else {
        out
    }
}

fn visible_conversation_title(title: &str) -> Option<String> {
    let title = title.trim();
    if title.is_empty() || title == "新会话" || title == "默认会话" {
        None
    } else {
        Some(title.to_string())
    }
}

/// 会话历史右键菜单：目标会话 + 视口坐标（`position: fixed`，绕过面板 overflow）。
#[derive(Clone, PartialEq)]
struct ChatHistoryCtxMenu {
    conversation_id: String,
    x: f64,
    y: f64,
}

/// 删除会话后刷新列表：若删的是当前会话则切到下一项或新建；否则只更新列表。
fn spawn_delete_conversation_and_refresh(
    id: String,
    mut conversations: Signal<Vec<ConversationSummaryDto>>,
    mut active_conversation_id: Signal<String>,
    mut chat_messages: Signal<Vec<UiChatMessage>>,
    mut chat_scroll_bottom_request: Signal<u64>,
    mut chat_title_editing: Signal<bool>,
    mut chat_title_editing_id: Signal<Option<String>>,
    mut chat_title_draft: Signal<String>,
    mut chat_history_error: Signal<Option<String>>,
) {
    spawn(async move {
        let was_active = active_conversation_id() == id;
        match delete_conversation(id).await {
            Ok(()) => match list_conversations().await {
                Ok(list) => {
                    if was_active {
                        if let Some(first) = list.first().cloned() {
                            active_conversation_id.set(first.id.clone());
                            conversations.set(list);
                            match load_conversation_messages(first.id, 200).await {
                                Ok(messages) => {
                                    chat_messages.set(stored_messages_to_ui(messages));
                                    chat_scroll_bottom_request += 1;
                                }
                                Err(_) => chat_messages.set(welcome_chat_messages()),
                            }
                        } else {
                            match create_conversation().await {
                                Ok(created) => {
                                    active_conversation_id.set(created.id.clone());
                                    conversations.set(vec![created]);
                                    chat_messages.set(welcome_chat_messages());
                                }
                                Err(e) => chat_history_error.set(Some(e.to_string())),
                            }
                        }
                    } else {
                        conversations.set(list);
                    }
                    chat_title_editing.set(false);
                    chat_title_editing_id.set(None);
                    chat_title_draft.set(String::new());
                    chat_history_error.set(None);
                }
                Err(e) => chat_history_error.set(Some(e.to_string())),
            },
            Err(e) => chat_history_error.set(Some(e.to_string())),
        }
    });
}

fn format_duration_ms(duration_ms: Option<i64>) -> String {
    let Some(ms) = duration_ms else {
        return "计时中".into();
    };
    if ms < 1000 {
        format!("{ms} ms")
    } else {
        format!("{:.1} s", ms as f64 / 1000.0)
    }
}

fn upsert_thinking_step_index(thinking: &mut UiAgentThinking, index: usize) -> usize {
    if let Some(pos) = thinking.steps.iter().position(|s| s.index == index) {
        return pos;
    }
    thinking.steps.push(UiThinkingStep {
        index,
        phase: StepPhase::Thinking,
        model_text: String::new(),
        tools: Vec::new(),
        duration_ms: None,
    });
    thinking.steps.len() - 1
}

fn with_last_assistant<F>(msgs: &mut [UiChatMessage], f: F)
where
    F: FnOnce(&mut String, &mut UiAgentThinking),
{
    if let Some(UiChatMessage::Assistant {
        content, thinking, ..
    }) = msgs
        .iter_mut()
        .rev()
        .find(|m| matches!(m, UiChatMessage::Assistant { .. }))
    {
        f(content, thinking);
    }
}

/// 将单条 SSE 事件应用到聊天消息（流式增量与落库后的消息 id）。
fn apply_chat_sse_event(
    ev: SseEvent,
    mut chat_messages: Signal<Vec<UiChatMessage>>,
    err_text: &mut Option<String>,
) {
    match ev {
        SseEvent::AgentStepStart { index, .. } => {
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |_, thinking| {
                thinking.status = ThinkingStatus::Running;
                thinking.expanded = true;
                let _ = upsert_thinking_step_index(thinking, index);
            });
            chat_messages.set(msgs);
        }
        SseEvent::AgentThinking { index } => {
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |_, thinking| {
                let pos = upsert_thinking_step_index(thinking, index);
                thinking.steps[pos].phase = StepPhase::Thinking;
            });
            chat_messages.set(msgs);
        }
        SseEvent::ThinkingDelta { step_index, text } => {
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |_, thinking| {
                let pos = upsert_thinking_step_index(thinking, step_index);
                thinking.steps[pos].model_text.push_str(&text);
            });
            chat_messages.set(msgs);
        }
        SseEvent::AnswerDelta { text } => {
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |content, _thinking| {
                content.push_str(&text);
            });
            chat_messages.set(msgs);
        }
        SseEvent::Delta { text } => {
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |content, thinking| {
                content.push_str(&text);
                if let Some(step) = thinking.steps.last_mut() {
                    step.model_text.push_str(&text);
                }
            });
            chat_messages.set(msgs);
        }
        SseEvent::AgentObserving { index } => {
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |_, thinking| {
                let pos = upsert_thinking_step_index(thinking, index);
                thinking.steps[pos].phase = StepPhase::Observing;
            });
            chat_messages.set(msgs);
        }
        SseEvent::ToolStart {
            id,
            name,
            args,
            step_index,
            ..
        } => {
            let id = id.clone();
            let name = name.clone();
            let args = args.clone();
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |_, thinking| {
                let index = step_index.unwrap_or_else(|| thinking.steps.len().max(1));
                let pos = upsert_thinking_step_index(thinking, index);
                let step = &mut thinking.steps[pos];
                step.phase = StepPhase::Observing;
                if let Some(tool) = step.tools.iter_mut().find(|t| t.id == id.as_str()) {
                    if !name.is_empty() {
                        tool.name = name;
                    }
                    if !args.is_empty() {
                        tool.args = args;
                    }
                    tool.status = ToolStatus::Running;
                } else {
                    step.tools.push(UiThinkingTool {
                        id,
                        name,
                        args,
                        summary: None,
                        status: ToolStatus::Running,
                    });
                }
            });
            chat_messages.set(msgs);
        }
        SseEvent::ToolResult { id, summary, .. } => {
            let id = id.clone();
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |_, thinking| {
                for step in thinking.steps.iter_mut() {
                    if let Some(tool) = step.tools.iter_mut().find(|t| t.id == id) {
                        tool.summary = Some(summary.clone());
                        tool.status = ToolStatus::Done;
                    }
                }
            });
            chat_messages.set(msgs);
        }
        SseEvent::AgentStepDone {
            index,
            model_output,
            duration_ms,
        } => {
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |_, thinking| {
                let pos = upsert_thinking_step_index(thinking, index);
                let step = &mut thinking.steps[pos];
                if !model_output.is_empty() {
                    step.model_text = model_output.clone();
                }
                step.phase = StepPhase::Done;
                step.duration_ms = duration_ms;
            });
            chat_messages.set(msgs);
        }
        SseEvent::AgentFinalizing => {
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |_, thinking| {
                thinking.status = ThinkingStatus::Done;
                thinking.expanded = true;
                if thinking.total_duration_ms.is_none() {
                    let sum: i64 = thinking.steps.iter().filter_map(|s| s.duration_ms).sum();
                    if sum > 0 {
                        thinking.total_duration_ms = Some(sum);
                    }
                }
            });
            chat_messages.set(msgs);
        }
        SseEvent::TurnPersisted {
            user_message_id,
            assistant_message_id,
            agent_run_id,
            ..
        } => {
            let mut msgs = chat_messages();
            for msg in msgs.iter_mut().rev() {
                if let UiChatMessage::Assistant {
                    id,
                    agent_run_id: run_id,
                    thinking,
                    ..
                } = msg
                {
                    if id.is_none() {
                        *id = Some(assistant_message_id);
                        *run_id = Some(agent_run_id);
                        thinking.status = ThinkingStatus::Done;
                        thinking.expanded = true;
                        break;
                    }
                }
            }
            for msg in msgs.iter_mut().rev() {
                if let UiChatMessage::User { id, .. } = msg {
                    if id.is_none() {
                        *id = Some(user_message_id);
                        break;
                    }
                }
            }
            chat_messages.set(msgs);
        }
        SseEvent::Error { message } => {
            *err_text = Some(crate::chat::friendly_chat_error_message(&message));
            let mut msgs = chat_messages();
            with_last_assistant(&mut msgs, |_, thinking| {
                thinking.status = ThinkingStatus::Error;
                thinking.expanded = true;
            });
            chat_messages.set(msgs);
        }
        SseEvent::Done => {}
    }
}

fn copy_text_to_clipboard(text: String) {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            let _ = clipboard.write_text(&text);
        }
    }

    #[cfg(not(all(target_arch = "wasm32", feature = "web")))]
    {
        let _ = text;
    }
}

fn build_agent_run_full_text(detail: &AgentRunDetailDto) -> String {
    let mut out = String::new();
    out.push_str("Agent 运行详情\n");
    out.push_str(&format!("模型: {}\n", detail.model));
    out.push_str(&format!("状态: {}\n", detail.status));
    out.push_str(&format!(
        "总耗时: {}\n\n",
        format_duration_ms(detail.total_duration_ms)
    ));

    for step in &detail.steps {
        out.push_str(&format!(
            "步骤 #{} | {} | {}\n",
            step.index,
            step.phase,
            format_duration_ms(step.duration_ms)
        ));
        if !step.model_output.trim().is_empty() {
            out.push_str("模型输出:\n");
            out.push_str(step.model_output.trim());
            out.push('\n');
        }
        for tool in &step.tools {
            out.push_str(&format!(
                "\n工具: {} | {}\n",
                tool.name,
                format_duration_ms(tool.duration_ms)
            ));
            out.push_str("参数:\n");
            out.push_str(tool.args.trim());
            out.push('\n');
            out.push_str("完整返回:\n");
            out.push_str(tool.result.trim());
            out.push('\n');
        }
        out.push('\n');
    }

    out
}

/// 聊天草稿输入（Web / 桌面共用）。
///
/// 使用受控 `value`，确保发送后 `chat_draft` 清空会立即同步到输入框。
/// `input_epoch` 在发送 / 快捷填充时递增，用于强制刷新输入节点。
///
/// **Enter** 发送（与发送按钮同逻辑）；**Shift+Enter** 换行；IME 组合中不发送。
/// **粘贴**：若剪贴板含图片，走附件管线（见 `on_paste_images`）；纯文本仍默认粘贴。
#[component]
fn ChatComposerField(
    mut chat_draft: Signal<String>,
    input_epoch: u64,
    disabled: bool,
    on_enter_send: EventHandler<()>,
    on_paste_images: EventHandler<ClipboardEvent>,
    #[props(default = 4)] rows: u32,
) -> Element {
    rsx! {
        textarea {
            key: "chat-composer-{input_epoch}",
            class: "ac-chat-input-field",
            placeholder: "向它下达指令…",
            value: "{chat_draft()}",
            disabled: if disabled { true },
            rows: "{rows}",
            oninput: move |e| {
                chat_draft.set(e.value());
            },
            onpaste: move |e: ClipboardEvent| {
                on_paste_images.call(e);
            },
            onkeydown: move |e: KeyboardEvent| {
                if e.key() != Key::Enter {
                    return;
                }
                if e.modifiers().contains(Modifiers::SHIFT) {
                    return;
                }
                if e.is_composing() {
                    return;
                }
                if disabled {
                    return;
                }
                e.prevent_default();
                on_enter_send.call(());
            },
        }
    }
}

/// Web：嵌入右侧聊天栏顶部的窗口控制条（侧栏开关、设置）；依赖父级 [`crate::web::ShellChromeCtx`]。
#[cfg(all(target_arch = "wasm32", feature = "web"))]
#[component]
fn WebShellTitlebar(
    mut show_sidebar: Signal<bool>,
    mut sidebar_width: Signal<f64>,
    mut show_center: Signal<bool>,
    mut show_terminal: Signal<bool>,
    mut show_chat: Signal<bool>,
    mut chat_history_open: Signal<bool>,
    mut sidebar_shutting_down: Signal<bool>,
) -> Element {
    let ctx = use_context::<crate::web::ShellChromeCtx>();
    let mut show_settings_modal = ctx.show_settings_modal;
    let mut show_about_modal = ctx.show_about_modal;
    let mut show_titlebar_settings_menu = ctx.show_titlebar_settings_menu;
    // 设置 API Key 入口：在设置中配置 OpenAI 兼容接口凭证。
    let _developer_mode = ctx.developer_mode;
    let sidebar_switch_on = show_sidebar() && !sidebar_shutting_down();
    let chat_switch_on = show_chat();
    let center_switch_on = show_center();
    let sidebar_title = if sidebar_switch_on {
        "隐藏左边栏"
    } else {
        "显示左边栏"
    };
    let center_title = if center_switch_on {
        "聊天栏往左覆盖中间栏"
    } else {
        "聊天栏往右收缩拉出中间栏"
    };
    let terminal_title = if show_terminal() {
        "隐藏终端"
    } else {
        "显示终端"
    };
    let chat_title = if chat_switch_on {
        "隐藏右边聊天栏"
    } else {
        "显示右边聊天栏"
    };
    let titlebar_reveal_class = if !sidebar_switch_on && !chat_switch_on {
        "ac-web-titlebar-reveal ac-web-titlebar-reveal--dock is-pinned"
    } else {
        "ac-web-titlebar-reveal ac-web-titlebar-reveal--dock"
    };

    rsx! {
        div { class: "{titlebar_reveal_class}",
            div { class: "ac-web-titlebar",
                div { class: "ac-web-titlebar-actions",
                    div { class: "ac-web-titlebar-switch-wrap",
                        button {
                            r#type: "button",
                            class: "ac-web-titlebar-switch-icon-outside",
                            title: "{sidebar_title}",
                            aria_label: "{sidebar_title}",
                            aria_expanded: sidebar_switch_on,
                            onclick: move |_| {
                                if show_sidebar() {
                                    if sidebar_shutting_down() {
                                        return;
                                    }
                                    sidebar_shutting_down.set(true);
                                    spawn(async move {
                                        ui_sleep_ms(AC_SIDEBAR_AUTO_COLLAPSE_YIELD_MS).await;
                                        sidebar_width.set(0.0);
                                        ui_sleep_ms(AC_SIDEBAR_AUTO_COLLAPSE_ANIM_MS).await;
                                        show_sidebar.set(false);
                                        sidebar_width.set(AC_SIDEBAR_DEFAULT_WIDTH_PX);
                                        sidebar_shutting_down.set(false);
                                    });
                                } else {
                                    show_sidebar.set(true);
                                }
                            },
                            if sidebar_switch_on {
                                Icon { icon: BsLayoutSidebar, width: 14, height: 14, fill: "currentColor", class: "ac-web-titlebar-icon" }
                            } else {
                                Icon { icon: BsLayoutSidebarInset, width: 14, height: 14, fill: "currentColor", class: "ac-web-titlebar-icon" }
                            }
                        }
                    }
                    div { class: "ac-web-titlebar-switch-wrap",
                        button {
                            r#type: "button",
                            class: "ac-web-titlebar-switch-icon-outside",
                            title: "{center_title}",
                            aria_label: "{center_title}",
                            aria_expanded: center_switch_on,
                            disabled: !chat_switch_on,
                            onclick: move |_| {
                                if show_chat() {
                                    show_center.toggle();
                                }
                            },
                            if center_switch_on {
                                Icon { icon: VscLayoutSidebarLeftDock, width: 14, height: 14, fill: "currentColor", class: "ac-web-titlebar-icon" }
                            } else {
                                Icon { icon: VscLayoutSidebarRightDock, width: 14, height: 14, fill: "currentColor", class: "ac-web-titlebar-icon" }
                            }
                        }
                    }
                }
                div { class: "ac-web-titlebar-spacer" }
                // 右侧：终端 → 右边聊天栏显隐 → 设置（会话历史靠拖宽手柄）
                div { class: "ac-web-titlebar-actions ac-web-titlebar-actions-end",
                    div { class: "ac-web-titlebar-switch-wrap",
                        button {
                            r#type: "button",
                            class: "ac-web-titlebar-switch-icon-outside",
                            title: "{terminal_title}",
                            aria_label: "{terminal_title}",
                            aria_expanded: show_terminal(),
                            onclick: move |_| show_terminal.toggle(),
                            if show_terminal() {
                                Icon { icon: VscLayoutPanelOff, width: 14, height: 14, fill: "currentColor", class: "ac-web-titlebar-icon" }
                            } else {
                                Icon { icon: VscLayoutPanel, width: 14, height: 14, fill: "currentColor", class: "ac-web-titlebar-icon" }
                            }
                        }
                    }
                    div { class: "ac-web-titlebar-switch-wrap",
                        button {
                            r#type: "button",
                            class: "ac-web-titlebar-switch-icon-outside",
                            title: "{chat_title}",
                            aria_label: "{chat_title}",
                            aria_expanded: chat_switch_on,
                            onclick: move |_| {
                                let next = !show_chat();
                                show_chat.set(next);
                                if !next {
                                    chat_history_open.set(false);
                                }
                            },
                            if chat_switch_on {
                                Icon { icon: BsLayoutSidebarReverse, width: 14, height: 14, fill: "currentColor", class: "ac-web-titlebar-icon" }
                            } else {
                                Icon { icon: BsLayoutSidebarInsetReverse, width: 14, height: 14, fill: "currentColor", class: "ac-web-titlebar-icon" }
                            }
                        }
                    }
                    div { class: "ac-web-titlebar-settings-wrap",
                        button {
                            r#type: "button",
                            class: if show_titlebar_settings_menu() {
                                "ac-web-titlebar-btn is-menu-open"
                            } else {
                                "ac-web-titlebar-btn"
                            },
                            title: "设置",
                            aria_expanded: show_titlebar_settings_menu(),
                            aria_haspopup: "menu",
                            onclick: move |_| show_titlebar_settings_menu.toggle(),
                            Icon { icon: BsGear, width: 16, height: 16, fill: "currentColor", class: "ac-web-titlebar-icon" }
                        }
                        if show_titlebar_settings_menu() {
                            div {
                                class: "ac-web-titlebar-menu-backdrop",
                                onclick: move |_| show_titlebar_settings_menu.set(false),
                            }
                            div { class: "ac-web-titlebar-menu", role: "menu",
                                button {
                                    r#type: "button",
                                    role: "menuitem",
                                    class: "ac-web-titlebar-menu__item",
                                    onclick: move |_| {
                                        show_titlebar_settings_menu.set(false);
                                        show_settings_modal.set(true);
                                    },
                                    "设置 API Key"
                                }
                                button {
                                    r#type: "button",
                                    role: "menuitem",
                                    class: "ac-web-titlebar-menu__item",
                                    onclick: move |_| {
                                        show_titlebar_settings_menu.set(false);
                                        show_about_modal.set(true);
                                    },
                                    "关于本机"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 非 Web 或 mobile 构建：不占位。
#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
#[component]
#[allow(unused_variables)]
fn WebShellTitlebar(
    mut show_sidebar: Signal<bool>,
    mut sidebar_width: Signal<f64>,
    mut show_center: Signal<bool>,
    mut show_terminal: Signal<bool>,
    mut show_chat: Signal<bool>,
    mut chat_history_open: Signal<bool>,
    mut sidebar_shutting_down: Signal<bool>,
) -> Element {
    rsx! {}
}

/// 三栏：侧栏（角色） / 中间（配置 Tab） / 右侧（聊天占位）。
#[component]
pub fn Console(
    mut show_sidebar: Signal<bool>,
    mut show_center: Signal<bool>,
    mut show_terminal: Signal<bool>,
    mut show_chat: Signal<bool>,
    mut chat_history_open: Signal<bool>,
    mut active_file_path: Signal<Option<String>>,
) -> Element {
    let toast = super::toast::use_toast();
    let mut chat_width = use_signal(|| AC_CHAT_DEFAULT_WIDTH_PX);
    // 对话区已顶到最大宽度后，累计左拖超出量（用于触发中间栏自动收起）。
    let mut chat_resize_overshoot = use_signal(|| 0.0f64);
    // 会话历史栏宽度（连续驱动布局；默认最窄图标栏，拖宽后标题渐进露出）。
    let mut chat_history_width = use_signal(chat_history_closed_width_px);
    // 拖曳会话历史时的实时宽度（避免每帧从信号重读产生抖动）。
    let mut chat_history_drag_width = use_signal(|| None::<f64>);
    let mut sidebar_width = use_signal(|| AC_SIDEBAR_DEFAULT_WIDTH_PX);
    // 拖曳触发阈值后先播宽度收起动画，再 `show_sidebar = false`。
    let mut sidebar_shutting_down = use_signal(|| false);
    // 中间栏关闭时先播回缩动画，再隐藏。
    let mut center_shutting_down = use_signal(|| false);
    let mut terminal_height = use_signal(|| AC_TERMINAL_DEFAULT_HEIGHT_PX);
    // 最大化前高度；再次点击「最大化」时恢复。
    let mut terminal_pre_maximize_height = use_signal(|| None::<f64>);
    let mut active_resize = use_signal(|| None::<ConsoleResizeKind>);
    let mut last_pointer_x = use_signal(|| None::<f64>);
    let mut last_pointer_y = use_signal(|| None::<f64>);
    let mut roles_loaded_gate = use_signal(|| false);
    let mut role_list = use_signal(Vec::<RoleDto>::new);
    let mut role_loading = use_signal(|| false);
    let mut role_error = use_signal(|| None::<String>);
    let mut editing_role_id = use_signal(|| None::<i64>);
    let mut editing_role_name = use_signal(|| "还没有名字呢".to_string());
    let mut active_tab = use_signal(|| "role");
    // 有操作缓存则恢复上次打开的中间栏标签；否则保持空列表 → 首页空态。
    let restored_center = use_hook(load_center_session_restore);
    let mut center_tabs = use_signal({
        let tabs = restored_center.tabs.clone();
        move || tabs
    });
    let mut active_center_id = use_signal({
        let id = restored_center.active_id.clone();
        move || id
    });
    // 技能侧栏默认「我的技能」（installed），与应用侧栏「我的应用」对齐。
    let mut skill_view = use_signal(|| "installed");
    let fs_root_path = use_signal(super::files::fs_workspace_root);
    let fs_section_open = use_signal(|| true);
    let fs_expanded = use_signal(HashSet::<String>::new);
    let fs_children_cache =
        use_signal(HashMap::<String, Result<Vec<super::files::FsEntryDto>, String>>::new);
    let mut fs_selected_path = use_signal({
        let path = restored_center.selected_file_path.clone();
        move || path
    });
    let fs_create_mode = use_signal(|| None::<super::files::FsCreateKind>);
    let fs_create_name = use_signal(String::new);
    let fs_create_parent = use_signal(|| None::<String>);
    let mut fs_notice = use_signal(|| None::<String>);
    let mut recent_project_dirs = use_signal(super::files::fs_recent_project_dirs);
    let mut home_logo_menu_open = use_signal(|| false);
    let mut fs_drafts = use_signal(HashMap::<String, String>::new);
    let mut fs_baselines = use_signal(HashMap::<String, String>::new);
    let mut fs_dirty = use_signal(HashSet::<String>::new);
    let mut fs_load_errors = use_signal(HashMap::<String, String>::new);
    let mut fs_save_notice = use_signal(|| None::<String>);
    let fs_just_written = use_signal(HashSet::<String>::new);
    let mut skill_search = use_signal(String::new);
    let mut skill_market_query = use_signal(String::new);
    let mut skill_market_search_epoch = use_signal(|| 0_u64);
    let mut skill_market_items = use_signal(Vec::<SkillMarketItemDto>::new);
    // None：尚未以当前搜索串成功拉取市场；Some(q)：已成功加载 trim 后的关键词 q。
    let mut skill_market_loaded_query = use_signal(|| None::<String>);
    let mut skill_market_loading = use_signal(|| false);
    let mut skill_market_error = use_signal(|| None::<String>);
    let mut skill_market_warnings = use_signal(Vec::<String>::new);
    let mut skill_market_stations = use_signal(default_skill_market_stations);
    let mut skill_market_station = use_signal(|| "clawhub".to_string());
    let mut skill_installing = use_signal(|| None::<String>);
    // 用户已装备（学习中）的技能 slug 列表，与服务器 `EQUIPPED_SKILLS_KEY` 一一对应；
    // 默认空 = 没有任何技能被装备到 Agent 高优先级（用户需点「学习」激活）。
    let mut skill_equipped_slugs = use_signal(Vec::<String>::new);
    let mut skill_equip_busy = use_signal(|| None::<String>);
    let mut skill_uninstall_busy = use_signal(|| None::<String>);
    let mut skill_installed_list = use_signal(Vec::<InstalledSkillDto>::new);
    let mut skill_installed_loading = use_signal(|| false);
    let mut skill_installed_error = use_signal(|| None::<String>);
    // 侧栏技能列表选中项：已安装用 slug，市场用 `source:id`。
    let mut selected_skill_key = use_signal({
        let key = restored_center.selected_skill_key.clone();
        move || key
    });
    // 应用市场：侧栏选中 key + 卡片目录（供中间栏详情查找）。
    let mut selected_plugin_key = use_signal({
        let key = restored_center.selected_plugin_key.clone();
        move || key
    });
    let plugin_catalog = use_signal(Vec::<super::plugins::PluginCard>::new);
    let plugin_refresh_tick = use_signal(|| 0_u64);
    // 删除角色确认：`(id, display_name)`，替代原生 `confirm`
    let mut delete_role_confirm = use_signal(|| None::<(i64, String)>);
    // 首页空态「创建插件」：输入插件名后脚手架到工作区 `extension/`
    let mut create_plugin_open = use_signal(|| false);
    let mut create_plugin_name = use_signal(String::new);
    let mut create_plugin_error = use_signal(|| None::<String>);
    // 首页空态「创建应用」：输入应用名后脚手架到工作区 `application/`
    let mut create_app_open = use_signal(|| false);
    let mut create_app_name = use_signal(String::new);
    let mut create_app_error = use_signal(|| None::<String>);
    // Web：服务端是否已配置 OpenAI；`None` 表示尚未拉取或失败。
    #[allow(unused_mut)]
    let mut llm_server_openai = use_signal(|| None::<bool>);
    #[allow(unused_mut)]
    let mut show_llm_modal = use_signal(|| false);
    #[allow(unused_mut)]
    let mut llm_modal_api_key = use_signal(String::new);
    #[allow(unused_mut)]
    let mut llm_modal_base = use_signal(String::new);

    let mut persona_prompt = use_signal(|| persona::DEFAULT_SYSTEM_PROMPT.to_string());
    // 角色切换 / 润色完成后递增，用于 textarea `key` 换挂载以同步受控 `value`。
    let mut persona_textarea_epoch = use_signal(|| 0_u64);
    let mut persona_load_gate = use_signal(|| false);
    let mut persona_polishing = use_signal(|| false);
    let mut persona_polish_hint = use_signal(|| None::<String>);
    let mut conversations = use_signal(Vec::<ConversationSummaryDto>::new);
    let mut active_conversation_id = use_signal(|| "default".to_string());
    let mut chat_history_loaded_gate = use_signal(|| false);
    let mut chat_history_error = use_signal(|| None::<String>);
    // 会话历史右键菜单（展开/收起栏共用）
    let mut chat_history_ctx_menu = use_signal(|| None::<ChatHistoryCtxMenu>);
    let mut chat_title_editing = use_signal(|| false);
    let mut chat_title_editing_id = use_signal(|| None::<String>);
    let mut chat_title_draft = use_signal(String::new);
    let mut chat_title_saving = use_signal(|| false);
    let mut selected_agent_run_detail = use_signal(|| None::<AgentRunDetailDto>);
    let mut agent_detail_loading = use_signal(|| false);
    let mut agent_detail_error = use_signal(|| None::<String>);
    let mut agent_detail_copy_notice = use_signal(|| None::<String>);
    let mut chat_messages = use_signal(welcome_chat_messages);
    // 默认不“吸底”：用户可以自由滚动历史；仅在 AI 正在回答（chat_busy = true）期间，
    // 下方 use_effect 会自动把它打开实现流式跟随；回答结束自动关闭。
    let mut chat_should_stick_bottom = use_signal(|| false);
    // 一次性「置底请求」计数器：每次会话切换 / 历史加载完成后递增，
    // 配套 use_effect 监听变化并在 dioxus 完成 commit 之后 schedule 一次滚到底；
    // 与 `chat_should_stick_bottom` 解耦，不会触发持续吸底。
    let mut chat_scroll_bottom_request = use_signal(|| 0_u64);
    let mut chat_draft = use_signal(|| String::new());
    let mut chat_input_epoch = use_signal(|| 0_u64);
    let mut chat_busy = use_signal(|| false);
    let mut chat_model = use_signal(initial_chat_model);
    let mut chat_attachments = use_signal(|| Vec::<ChatPendingAttachment>::new());
    let mut chat_attachment_seq = use_signal(|| 1_u64);
    // Desktop WebView：dioxus 剪贴板事件不含文件，挂 JS capture 钩子读 clipboardData。
    #[cfg(not(target_arch = "wasm32"))]
    use_effect(move || {
        spawn(async move {
            let mut eval = dioxus::document::eval(crate::chat::desktop_paste::INSTALL_HOOK_JS);
            loop {
                let Ok(payload) =
                    eval.recv::<crate::chat::desktop_paste::PastePayload>().await
                else {
                    break;
                };
                if let Some(t) = payload.text {
                    chat_draft.with_mut(|draft| {
                        if !draft.is_empty() && !draft.ends_with('\n') && !draft.ends_with(' ') {
                            draft.push(' ');
                        }
                        draft.push_str(&t);
                    });
                }
                for img in payload.images {
                    let id = chat_attachment_seq();
                    chat_attachment_seq.set(id + 1);
                    if let Some(att) =
                        crate::chat::desktop_paste::attachment_from_paste_image(id, img)
                    {
                        chat_attachments.with_mut(|list| list.push(att));
                    }
                }
            }
        });
    });
    let mut chat_model_sheet_open = use_signal(|| false);
    // 聊天栏模型选择列表：先塞本地兜底，再异步用 Relay `GET /v1/models` 覆盖。
    let mut chat_models = use_signal(fallback_chat_models);
    let mut chat_models_load_hint = use_signal(|| None::<String>);
    use_hook(|| {
        spawn(async move {
            match list_chat_models().await {
                Ok(dtos) if !dtos.is_empty() => {
                    let entries: Vec<ChatModelEntry> = dtos
                        .into_iter()
                        .map(|m| ChatModelEntry {
                            id: m.id,
                            display_name: m.display_name,
                        })
                        .collect();
                    chat_models.set(entries);
                    chat_models_load_hint.set(None);
                }
                Ok(_) => {
                    // 空列表：保留兜底，轻提示。
                    chat_models_load_hint.set(Some("模型列表为空，已使用本地默认".into()));
                }
                Err(_) => {
                    chat_models_load_hint.set(Some("模型列表暂时不可用，已使用本地默认".into()));
                }
            }
        });
    });
    #[cfg(target_arch = "wasm32")]
    let chat_stream_abort =
        use_signal(|| std::rc::Rc::new(std::cell::RefCell::new(None::<web_sys::AbortController>)));

    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    let developer_mode = use_context::<crate::web::ShellChromeCtx>().developer_mode;
    #[cfg(not(all(target_arch = "wasm32", feature = "web")))]
    let developer_mode = use_signal(|| false);

    // Reconcile：若当前选中的 chat_model 不在最新模型列表里（譬如服务端把
    // 旧模型下架），自动回落到列表第一项。`chat_models` 通过 .read() 触发
    // dioxus 依赖追踪，列表 update 后会自动重跑。空列表（首屏未拉到 + 默
    // 认兜底也被清空的极端情况）下保留当前值不动，避免误清。
    use_effect(move || {
        let models = chat_models.read();
        if models.is_empty() {
            return;
        }
        let cur = chat_model();
        if !models.iter().any(|m| m.id == cur) {
            let fallback = models[0].id.clone();
            // 同步写入 prefs：避免下次刷新时还原成已下架的旧值再被回落，
            // 让用户在 UI 看到"忽然换了一个"。
            persist_chat_model(&fallback);
            chat_model.set(fallback);
        }
    });

    // 中间栏标签操作缓存：有标签则写入；清空后清除缓存，下次启动仍进首页空态。
    use_effect(move || {
        let tabs = center_tabs();
        let active = active_center_id();
        persist_center_session(&tabs, &active);
    });

    // 底部状态栏：选中文件 / 编辑对比标签时展示路径。
    use_effect(move || {
        let path = center_active_kind(&center_tabs(), &active_center_id())
            .as_ref()
            .and_then(center_tab_display_path)
            .map(str::to_string);
        active_file_path.set(path);
    });

    let skill_market_cards = use_memo(move || {
        let q = skill_search().trim().to_ascii_lowercase();
        skill_market_items()
            .into_iter()
            .filter(|item| {
                if q.is_empty() {
                    return true;
                }
                item.name.to_ascii_lowercase().contains(&q)
                    || item.description.to_ascii_lowercase().contains(&q)
                    || item.author.to_ascii_lowercase().contains(&q)
            })
            .map(|item| {
                let source_label = skill_source_label(&item.source);
                let downloads = skill_metric(item.downloads);
                let stars = skill_metric(item.stars);
                let install_key = format!("{}:{}", item.source, item.id);
                SkillMarketCardVm {
                    item,
                    source_label,
                    downloads,
                    stars,
                    install_key,
                }
            })
            .collect::<Vec<_>>()
    });

    let skill_market_station_cards = use_memo(move || {
        let station = skill_market_station();
        skill_market_cards()
            .into_iter()
            .filter(|card| card.item.source == station)
            .collect::<Vec<_>>()
    });

    let skill_market_station_warnings = use_memo(move || {
        let station = skill_market_station();
        skill_market_warnings()
            .into_iter()
            .filter(|w| skill_station_warning_matches(w, &station))
            .collect::<Vec<_>>()
    });

    let skill_installed_filtered = use_memo(move || {
        let q = skill_search().trim().to_ascii_lowercase();
        skill_installed_list()
            .into_iter()
            .filter(|inst| {
                if q.is_empty() {
                    return true;
                }
                inst.title.to_ascii_lowercase().contains(&q)
                    || inst.description.to_ascii_lowercase().contains(&q)
                    || inst.slug.to_ascii_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    });

    use_effect(move || {
        if persona_load_gate() {
            return;
        }
        persona_load_gate.set(true);
        spawn(async move {
            if let Ok(p) = persona::load_persona().await {
                persona_prompt.set(p);
                persona_textarea_epoch.with_mut(|n| *n += 1);
            }
        });
    });

    use_effect(move || {
        if roles_loaded_gate() {
            return;
        }
        roles_loaded_gate.set(true);
        role_loading.set(true);
        spawn(async move {
            match load_roles().await {
                Ok(list) => {
                    let active = list.iter().find(|r| r.is_active).cloned();
                    if let Some(active_role) = active {
                        persona_prompt.set(active_role.system_prompt.clone());
                        editing_role_name.set(active_role.name.clone());
                        editing_role_id.set(Some(active_role.id));
                        persona_textarea_epoch.with_mut(|n| *n += 1);
                    }
                    role_list.set(list);
                    role_error.set(None);
                }
                Err(e) => {
                    role_error.set(Some(e.to_string()));
                }
            }
            role_loading.set(false);
        });
    });

    use_effect(move || {
        if chat_history_loaded_gate() {
            return;
        }
        chat_history_loaded_gate.set(true);
        spawn(async move {
            match list_conversations().await {
                Ok(list) => {
                    let active_id = list
                        .first()
                        .map(|c| c.id.clone())
                        .unwrap_or_else(|| "default".to_string());
                    conversations.set(list);
                    active_conversation_id.set(active_id.clone());
                    match load_conversation_messages(active_id, 200).await {
                        Ok(messages) => {
                            chat_messages.set(stored_messages_to_ui(messages));
                            chat_history_error.set(None);
                            chat_scroll_bottom_request += 1;
                        }
                        Err(e) => {
                            chat_messages.set(welcome_chat_messages());
                            chat_history_error.set(Some(e.to_string()));
                        }
                    }
                }
                Err(e) => {
                    chat_messages.set(welcome_chat_messages());
                    chat_history_error.set(Some(e.to_string()));
                }
            }
        });
    });

    use_effect(move || {
        #[cfg(all(target_arch = "wasm32", feature = "web"))]
        {
            spawn(async move {
                if let Some(v) = llm_config::fetch_server_openai_configured().await {
                    llm_server_openai.set(Some(v));
                }
            });
        }
    });

    use_effect(move || {
        if active_tab() != "skills" || skill_view() != "market" {
            return;
        }
        let _search_epoch = skill_market_search_epoch();
        let query = skill_market_query().trim().to_string();
        if skill_market_loaded_query() == Some(query.clone()) {
            return;
        }
        skill_market_loading.set(true);
        skill_market_error.set(None);
        spawn(async move {
            let dev = developer_mode();
            match load_skill_market(query.clone()).await {
                Ok(market) => {
                    if !market.registries.is_empty() {
                        skill_market_stations.set(market.registries.clone());
                        let current = skill_market_station();
                        if !market.registries.iter().any(|r| r.id == current) {
                            if let Some(first) = market.registries.first() {
                                skill_market_station.set(first.id.clone());
                            }
                        }
                    }
                    skill_market_items.set(market.items);
                    skill_market_warnings.set(
                        market
                            .warnings
                            .iter()
                            .map(|w| skill_market_warning_for_ui(w, dev))
                            .collect(),
                    );
                    skill_market_error.set(None);
                    skill_market_loaded_query.set(Some(query));
                }
                Err(e) => {
                    skill_market_error
                        .set(Some(skill_market_fatal_error_for_ui(&e.to_string(), dev)));
                    skill_market_warnings.set(Vec::new());
                    skill_market_loaded_query.set(None);
                }
            }
            skill_market_loading.set(false);
        });
    });

    use_effect(move || {
        if active_tab() != "skills" || skill_view() != "installed" {
            return;
        }
        skill_installed_loading.set(true);
        skill_installed_error.set(None);
        spawn(async move {
            // 已装备列表：让按钮 label / is-on 状态正确反映服务端真实状态。
            // 失败回退为空数组（视为全部未装备），保持与之前注释一致。
            match load_equipped_skills().await {
                Ok(resp) => skill_equipped_slugs.set(resp.slugs),
                Err(_) => skill_equipped_slugs.set(Vec::new()),
            }
            match load_installed_skills().await {
                Ok(resp) => {
                    skill_installed_list.set(resp.skills);
                    skill_installed_error.set(None);
                }
                Err(e) => {
                    skill_installed_list.set(Vec::new());
                    skill_installed_error.set(Some(e.to_string()));
                }
            }
            skill_installed_loading.set(false);
        });
    });

    use_effect(move || {
        let _ = chat_messages();
        if chat_should_stick_bottom() {
            schedule_chat_thread_scroll_bottom();
        }
    });

    // 回答开始（chat_busy: false → true）自动开启吸底；回答结束（true → false）自动关闭。
    // 走 effect 而不是在每条 chat_busy.set(false) 处手动同步，新增的退出路径也无需补代码。
    use_effect(move || {
        chat_should_stick_bottom.set(chat_busy());
    });

    // 监听一次性「置底请求」：每次递增都在 dioxus 完成本帧 commit 之后调一次 schedule，
    // 内部走一个 microtask 取 DOM scroll_height，此时新一批消息已经渲染完成，能稳定置底。
    use_effect(move || {
        let _ = chat_scroll_bottom_request();
        schedule_chat_thread_scroll_bottom();
    });

    let submit_chat = Rc::new(RefCell::new(move || {
        if chat_busy() {
            return;
        }
        let text = chat_draft().trim().to_string();
        let pending = chat_attachments();
        if text.is_empty() && pending.is_empty() {
            return;
        }
        chat_busy.set(true);
        chat_draft.set(String::new());
        chat_attachments.set(Vec::new());
        chat_input_epoch += 1;

        let image_urls: Vec<String> = pending
            .iter()
            .filter_map(|a| a.image_data_url.clone())
            .collect();
        let non_image_names: Vec<String> = pending
            .iter()
            .filter(|a| a.image_data_url.is_none())
            .map(|a| a.name.clone())
            .collect();
        let mut send_text = text.clone();
        if !non_image_names.is_empty() {
            let names = non_image_names.join(", ");
            if send_text.is_empty() {
                send_text = format!("[附件: {names}]");
            } else {
                send_text.push_str(&format!("\n\n[附件: {names}]"));
            }
        }
        // 纯图无文案时给模型一个简短提示，避免空 text part。
        if send_text.is_empty() && !image_urls.is_empty() {
            send_text = "请查看附图。".into();
        }

        let mut dto: Vec<ChatMessageDto> = chat_messages()
            .iter()
            .filter_map(|m| match m {
                UiChatMessage::User { content, .. } => Some(ChatMessageDto {
                    role: "user".into(),
                    content: content.clone(),
                    images: Vec::new(),
                }),
                UiChatMessage::Assistant {
                    id,
                    content,
                    agent_run_id,
                    ..
                } => {
                    if content.trim().is_empty() || (id.is_none() && agent_run_id.is_none()) {
                        None
                    } else {
                        Some(ChatMessageDto {
                            role: "assistant".into(),
                            content: content.clone(),
                            images: Vec::new(),
                        })
                    }
                }
            })
            .collect();
        dto.push(ChatMessageDto {
            role: "user".into(),
            content: send_text.clone(),
            images: image_urls,
        });

        let mut ui_msgs = chat_messages();
        ui_msgs.push(UiChatMessage::User {
            id: None,
            content: send_text.clone(),
            attachments: pending,
        });
        ui_msgs.push(UiChatMessage::Assistant {
            id: None,
            content: String::new(),
            agent_run_id: None,
            thinking: UiAgentThinking::running(),
        });
        chat_messages.set(ui_msgs);

        let model_for_turn = chat_model().trim().to_string();
        if model_for_turn.is_empty() {
            chat_busy.set(false);
            return;
        }
        let system = persona_prompt();
        let conversation_id = active_conversation_id();
        spawn(async move {
            let req = ChatTurnRequest {
                conversation_id: Some(conversation_id),
                messages: dto,
                system: Some(system),
                model: model_for_turn,
                max_agent_steps: None,
            };

            let mut err_text: Option<String> = None;

            #[cfg(target_arch = "wasm32")]
            let (run_result, user_aborted) = {
                let ac = match web_sys::AbortController::new() {
                    Ok(c) => c,
                    Err(_) => {
                        chat_busy.set(false);
                        return;
                    }
                };
                *chat_stream_abort().borrow_mut() = Some(ac.clone());
                let sig = ac.signal();
                let r = run_chat_turn(req, Some(&sig), |ev| {
                    apply_chat_sse_event(ev, chat_messages, &mut err_text);
                })
                .await;
                let aborted = sig.aborted();
                *chat_stream_abort().borrow_mut() = None;
                (r, aborted)
            };

            #[cfg(not(target_arch = "wasm32"))]
            let (run_result, user_aborted) = {
                let r = run_chat_turn(req, |ev| {
                    apply_chat_sse_event(ev, chat_messages, &mut err_text);
                })
                .await;
                (r, false)
            };

            if user_aborted {
                // 中止流式输出：丢弃本轮未完成的助手气泡，不写入「已停止」文案。
                let mut msgs = chat_messages();
                if matches!(msgs.last(), Some(UiChatMessage::Assistant { .. })) {
                    msgs.pop();
                }
                chat_messages.set(msgs);
            }

            if let Err(e) = run_result {
                if !user_aborted && err_text.is_none() {
                    err_text = Some(crate::chat::friendly_chat_error_message(&e.to_string()));
                }
            }
            if let Some(msg) = err_text {
                let mut msgs = chat_messages();
                if let Some(UiChatMessage::Assistant {
                    content, thinking, ..
                }) = msgs.last_mut()
                {
                    if content.is_empty() {
                        *content = format!("❌ {}", msg);
                    } else {
                        content.push_str(&format!("\n❌ {}", msg));
                    }
                    thinking.dismissed = true;
                }
                chat_messages.set(msgs);
            }

            if let Ok(list) = list_conversations().await {
                conversations.set(list);
            }
            chat_busy.set(false);
        });
    }));

    let enter_send_chat = {
        let submit_chat = submit_chat.clone();
        move |_| (*submit_chat.borrow_mut())()
    };

    let on_composer_paste = move |e: ClipboardEvent| {
        #[cfg(target_arch = "wasm32")]
        {
            use crate::chat::wasm_paste;
            let Some(dt) = wasm_paste::clipboard_data_from_event(&e) else {
                return;
            };
            let (images, text) = wasm_paste::extract_clipboard_images(&dt);
            if images.is_empty() {
                return;
            }
            // 有图片时拦截默认粘贴，避免把图片当乱码文本插入；文本则手动并入草稿。
            e.prevent_default();
            if let Some(t) = text {
                chat_draft.with_mut(|draft| {
                    if !draft.is_empty() && !draft.ends_with('\n') && !draft.ends_with(' ') {
                        draft.push(' ');
                    }
                    draft.push_str(&t);
                });
            }
            spawn(async move {
                for (name, mime, file) in images {
                    let Some(bytes) = wasm_paste::read_web_file_bytes(&file).await else {
                        continue;
                    };
                    let id = chat_attachment_seq();
                    chat_attachment_seq.set(id + 1);
                    let att = crate::chat::attachment_from_bytes(id, name, mime, bytes);
                    chat_attachments.with_mut(|list| list.push(att));
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Desktop：由 `desktop_paste` JS capture 钩子处理（见上方 use_effect）。
            let _ = e;
        }
    };

    let send_or_pause_chat = {
        let submit_chat = submit_chat.clone();
        #[cfg(target_arch = "wasm32")]
        let chat_stream_abort = chat_stream_abort;
        move |_| {
            if chat_busy() {
                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(c) = chat_stream_abort().borrow().as_ref() {
                        c.abort();
                    }
                }
            } else {
                (*submit_chat.borrow_mut())();
            }
        }
    };

    let console_class = if active_resize().is_some() {
        "ac-console-root ac-console-root-resizing"
    } else {
        "ac-console-root"
    };
    let sidebar_class = if !show_sidebar() {
        "ac-sidebar premium-sidebar ac-sidebar-hidden".to_string()
    } else if sidebar_shutting_down() {
        "ac-sidebar premium-sidebar ac-sidebar-shutting".to_string()
    } else {
        "ac-sidebar premium-sidebar".to_string()
    };
    let center_class = if show_center() && center_shutting_down() {
        "ac-center premium-panel ac-center-shutting"
    } else if show_center() {
        "ac-center premium-panel"
    } else {
        "ac-center premium-panel ac-center-hidden"
    };
    // 收起动画期间即让对话区撑满，避免中间栏缩小时右侧仍固定宽度留下空白。
    let center_panel_open = show_center() && !center_shutting_down();
    let chat_class = if !show_chat() {
        "ac-chat premium-chat ac-chat-hidden".to_string()
    } else if !show_sidebar() && !center_panel_open {
        #[cfg(all(target_arch = "wasm32", feature = "web"))]
        {
            "ac-chat premium-chat ac-chat-expanded".to_string()
        }
        #[cfg(not(all(target_arch = "wasm32", feature = "web")))]
        {
            "ac-chat premium-chat ac-chat-expanded ac-chat-titlebar-spaced".to_string()
        }
    } else if center_panel_open {
        "ac-chat premium-chat".to_string()
    } else {
        "ac-chat premium-chat ac-chat-expanded".to_string()
    };
    let chat_width_style = if !show_chat() {
        String::new()
    } else if center_panel_open {
        format!("width: {:.0}px;", chat_width())
    } else {
        String::new()
    };
    let chat_history_width_style = {
        let closed = chat_history_closed_width_px();
        let w = chat_history_drag_width()
            .unwrap_or_else(|| chat_history_width())
            .clamp(closed, AC_CHAT_HISTORY_MAX_WIDTH_PX);
        // 最小档用 rem 锁定「仅图标」真实宽度；略宽后改回 px，标题靠 overflow 渐进露出。
        if w <= closed + 0.5 {
            "width: calc(2.25rem + 1px); max-width: calc(2.25rem + 1px); min-width: calc(2.25rem + 1px);"
                .to_string()
        } else {
            let w = w.round();
            format!("width: {w:.0}px; max-width: {w:.0}px; min-width: {w:.0}px;")
        }
    };
    let chat_history_is_rail = {
        let closed = chat_history_closed_width_px();
        let w = chat_history_drag_width()
            .unwrap_or_else(|| chat_history_width());
        w <= closed + 0.5
    };
    let chat_history_panel_class = if chat_history_is_rail {
        "ac-chat-history-panel scrollbar-hide is-rail"
    } else {
        "ac-chat-history-panel scrollbar-hide"
    };
    let sidebar_width_style = if show_sidebar() {
        let w = sidebar_width();
        format!("width: {w:.0}px; flex: 0 0 {w:.0}px;")
    } else {
        String::new()
    };
    let terminal_height_style = if show_terminal() {
        format!("height: {:.0}px;", terminal_height())
    } else {
        String::new()
    };
    // 模型 sheet 打开时在侧栏加类，配合 CSS 解除 overflow 裁切
    let chat_aside_class = format!(
        "{}{}",
        chat_class,
        if chat_model_sheet_open() {
            " ac-chat-menus-open"
        } else {
            ""
        }
    );

    rsx! {
        div {
            class: "{console_class}",
            onkeydown: move |e: KeyboardEvent| {
                if !center_tabs().is_empty()
                    || !super::files::fs_available()
                    || create_plugin_open()
                    || create_app_open()
                {
                    return;
                }
                if is_mod_char_shortcut(&e, "o") {
                    e.prevent_default();
                    match super::files::fs_open_project_directory_dialog() {
                        Ok(Some(root)) => {
                            apply_opened_project_root(
                                root,
                                fs_root_path,
                                fs_expanded,
                                fs_children_cache,
                                fs_selected_path,
                                fs_create_mode,
                                fs_create_name,
                                fs_create_parent,
                                fs_notice,
                                recent_project_dirs,
                                active_tab,
                                center_tabs,
                                active_center_id,
                                home_logo_menu_open,
                                plugin_refresh_tick,
                            );
                        }
                        Ok(None) => {}
                        Err(err) => fs_notice.set(Some(err)),
                    }
                } else if is_mod_char_shortcut(&e, "j") {
                    e.prevent_default();
                    create_app_name.set(String::new());
                    create_app_error.set(None);
                    create_app_open.set(true);
                } else if is_mod_char_shortcut(&e, "e") {
                    e.prevent_default();
                    create_plugin_name.set(String::new());
                    create_plugin_error.set(None);
                    create_plugin_open.set(true);
                }
            },
            onmousemove: move |event| {
                let Some(kind) = active_resize() else {
                    return;
                };
                let current_x = event.data.client_coordinates().x;
                let current_y = event.data.client_coordinates().y;
                match kind {
                    ConsoleResizeKind::Chat => {
                        let Some(previous_x) = last_pointer_x() else {
                            last_pointer_x.set(Some(current_x));
                            return;
                        };
                        if !show_chat() || !show_center() || center_shutting_down() {
                            active_resize.set(None);
                            last_pointer_x.set(None);
                            chat_resize_overshoot.set(0.0);
                            return;
                        }
                        let delta = previous_x - current_x;
                        let raw_next = chat_width() + delta;
                        let collapse_threshold =
                            center_collapse_overshoot_px(show_sidebar(), sidebar_width());
                        if raw_next > AC_CHAT_MAX_WIDTH_PX && collapse_threshold > 0.0 {
                            let extra = raw_next - AC_CHAT_MAX_WIDTH_PX;
                            let total_overshoot = chat_resize_overshoot() + extra;
                            if total_overshoot >= collapse_threshold {
                                active_resize.set(None);
                                last_pointer_x.set(None);
                                chat_resize_overshoot.set(0.0);
                                center_shutting_down.set(true);
                                spawn(async move {
                                    ui_sleep_ms(AC_CENTER_AUTO_COLLAPSE_ANIM_MS).await;
                                    show_center.set(false);
                                    center_shutting_down.set(false);
                                });
                                return;
                            }
                            chat_resize_overshoot.set(total_overshoot);
                            chat_width.set(AC_CHAT_MAX_WIDTH_PX);
                        } else {
                            chat_resize_overshoot.set(0.0);
                            chat_width.set(
                                raw_next.clamp(AC_CHAT_MIN_WIDTH_PX, AC_CHAT_MAX_WIDTH_PX),
                            );
                        }
                        last_pointer_x.set(Some(current_x));
                    }
                    ConsoleResizeKind::Sidebar => {
                        let Some(previous_x) = last_pointer_x() else {
                            last_pointer_x.set(Some(current_x));
                            return;
                        };
                        if !show_sidebar() || sidebar_shutting_down() {
                            active_resize.set(None);
                            last_pointer_x.set(None);
                            return;
                        }
                        let next_w = sidebar_width() + current_x - previous_x;
                        let clamped =
                            next_w.clamp(AC_SIDEBAR_MIN_WIDTH_PX, AC_SIDEBAR_MAX_WIDTH_PX);
                        sidebar_width.set(clamped);
                        last_pointer_x.set(Some(current_x));
                    }
                    ConsoleResizeKind::ChatHistory => {
                        let Some(previous_x) = last_pointer_x() else {
                            last_pointer_x.set(Some(current_x));
                            return;
                        };
                        // 左边缘：往左拖加宽，往右拖变窄；接近最窄时吸附到纯图标档。
                        let delta = previous_x - current_x;
                        let current_w = chat_history_drag_width()
                            .unwrap_or_else(|| chat_history_width());
                        let closed = chat_history_closed_width_px();
                        let from_rail = current_w <= closed + 0.5;
                        let next_w =
                            chat_history_snap_width(current_w + delta, closed, from_rail);
                        chat_history_drag_width.set(Some(next_w));
                        chat_history_width.set(next_w);
                        chat_history_open.set(next_w > closed + 0.5);
                        last_pointer_x.set(Some(current_x));
                    }
                    ConsoleResizeKind::Terminal => {
                        let Some(previous_y) = last_pointer_y() else {
                            last_pointer_y.set(Some(current_y));
                            return;
                        };
                        if !show_terminal() {
                            active_resize.set(None);
                            last_pointer_y.set(None);
                            return;
                        }
                        // 向上拖增高，向下拖降低。
                        let next_h = terminal_height() + (previous_y - current_y);
                        terminal_height.set(next_h.clamp(
                            AC_TERMINAL_MIN_HEIGHT_PX,
                            AC_TERMINAL_MAX_HEIGHT_PX,
                        ));
                        // 手动拖曳后取消「最大化前高度」记忆。
                        terminal_pre_maximize_height.set(None);
                        last_pointer_y.set(Some(current_y));
                    }
                }
            },
            onmouseup: move |_| {
                // 松手时若停在吸附带内，强制落到纯图标宽。
                if matches!(active_resize(), Some(ConsoleResizeKind::ChatHistory)) {
                    let closed = chat_history_closed_width_px();
                    let w = chat_history_drag_width()
                        .unwrap_or_else(|| chat_history_width());
                    let snapped = chat_history_snap_width(w, closed, false);
                    chat_history_width.set(snapped);
                    chat_history_open.set(snapped > closed + 0.5);
                }
                active_resize.set(None);
                last_pointer_x.set(None);
                last_pointer_y.set(None);
                chat_resize_overshoot.set(0.0);
                chat_history_drag_width.set(None);
            },
            onmouseleave: move |_| {
                if matches!(active_resize(), Some(ConsoleResizeKind::ChatHistory)) {
                    let closed = chat_history_closed_width_px();
                    let w = chat_history_drag_width()
                        .unwrap_or_else(|| chat_history_width());
                    let snapped = chat_history_snap_width(w, closed, false);
                    chat_history_width.set(snapped);
                    chat_history_open.set(snapped > closed + 0.5);
                }
                active_resize.set(None);
                last_pointer_x.set(None);
                last_pointer_y.set(None);
                chat_resize_overshoot.set(0.0);
                chat_history_drag_width.set(None);
            },
            // Web：聊天栏收起后栏内标题栏不可点，在顶层放一条固定恢复条。
            if !show_chat() {
                div { class: "ac-web-titlebar-floating",
                    WebShellTitlebar {
                        show_sidebar,
                        sidebar_width,
                        show_center,
                        show_terminal,
                        show_chat,
                        chat_history_open,
                        sidebar_shutting_down,
                    }
                }
            }
            aside { class: "{sidebar_class}", style: "{sidebar_width_style}",
                    div { class: "ac-sidebar-scroll scrollbar-hide",
                        nav { class: "ac-sidebar-nav",
                            button {
                                r#type: "button",
                                class: if active_tab() == "role" {
                                    "ac-sidebar-nav-item is-active"
                                } else {
                                    "ac-sidebar-nav-item"
                                },
                                title: "角色设定",
                                aria_label: "角色设定",
                                onclick: move |_| {
                                    active_tab.set("role");
                                    center_tabs.with_mut(|tabs| {
                                        active_center_id.with_mut(|active| {
                                            center_open_or_focus(tabs, active, CenterTab::role());
                                        });
                                    });
                                },
                                if active_tab() == "role" {
                                    Icon { icon: RiHomeSmile2Fill, width: 16, height: 16, fill: "currentColor", class: "ac-sidebar-nav-icon" }
                                } else {
                                    Icon { icon: RiHomeSmile2Line, width: 16, height: 16, fill: "currentColor", class: "ac-sidebar-nav-icon" }
                                }
                            }
                            button {
                                r#type: "button",
                                class: if active_tab() == "files" {
                                    "ac-sidebar-nav-item is-active"
                                } else {
                                    "ac-sidebar-nav-item"
                                },
                                title: "文件",
                                aria_label: "文件",
                                onclick: move |_| {
                                    active_tab.set("files");
                                },
                                if active_tab() == "files" {
                                    Icon { icon: TbFileFilled, width: 16, height: 16, fill: "currentColor", class: "ac-sidebar-nav-icon" }
                                } else {
                                    Icon { icon: TbFile, width: 16, height: 16, fill: "currentColor", class: "ac-sidebar-nav-icon" }
                                }
                            }
                            button {
                                r#type: "button",
                                class: if active_tab() == "skills" {
                                    "ac-sidebar-nav-item is-active"
                                } else {
                                    "ac-sidebar-nav-item"
                                },
                                title: "技能库",
                                aria_label: "技能库",
                                onclick: move |_| {
                                    active_tab.set("skills");
                                },
                                if active_tab() == "skills" {
                                    Icon { icon: PiMagicWandFill, width: 16, height: 16, fill: "currentColor", class: "ac-sidebar-nav-icon" }
                                } else {
                                    Icon { icon: PiMagicWandBold, width: 16, height: 16, fill: "currentColor", class: "ac-sidebar-nav-icon" }
                                }
                            }
                            button {
                                r#type: "button",
                                class: if active_tab() == "plugins" {
                                    "ac-sidebar-nav-item is-active"
                                } else {
                                    "ac-sidebar-nav-item"
                                },
                                title: "插件市场",
                                aria_label: "插件市场",
                                onclick: move |_| {
                                    active_tab.set("plugins");
                                },
                                if active_tab() == "plugins" {
                                    Icon { icon: RiApps2Fill, width: 16, height: 16, fill: "currentColor", class: "ac-sidebar-nav-icon" }
                                } else {
                                    Icon { icon: RiApps2Line, width: 16, height: 16, fill: "currentColor", class: "ac-sidebar-nav-icon" }
                                }
                            }
                        }
                        if active_tab() == "skills" {
                            div { class: "ac-sidebar-skills",
                                div { class: "ac-sidebar-skills-search",
                                    Icon {
                                        icon: LdSearch,
                                        width: 14,
                                        height: 14,
                                        fill: "currentColor",
                                        class: "ac-sidebar-skills-search-icon",
                                    }
                                    input {
                                        r#type: "search",
                                        value: "{skill_search()}",
                                        placeholder: "搜索技能...",
                                        oninput: move |e| skill_search.set(e.value()),
                                        onkeydown: move |e: KeyboardEvent| {
                                            if e.key() != Key::Enter {
                                                return;
                                            }
                                            e.prevent_default();
                                            if skill_view() == "market" && !skill_market_loading() {
                                                skill_market_query.set(skill_search().trim().to_string());
                                                skill_market_loaded_query.set(None);
                                                skill_market_search_epoch.with_mut(|epoch| *epoch += 1);
                                            }
                                        },
                                    }
                                }
                                div { class: "ac-sidebar-skills-toggle",
                                    button {
                                        r#type: "button",
                                        class: if skill_view() == "installed" {
                                            "ac-sidebar-skills-toggle-btn is-active"
                                        } else {
                                            "ac-sidebar-skills-toggle-btn"
                                        },
                                        onclick: move |_| {
                                            skill_view.set("installed");
                                            selected_skill_key.set(None);
                                        },
                                        "我的技能"
                                    }
                                    button {
                                        r#type: "button",
                                        class: if skill_view() == "market" {
                                            "ac-sidebar-skills-toggle-btn is-active"
                                        } else {
                                            "ac-sidebar-skills-toggle-btn"
                                        },
                                        onclick: move |_| {
                                            skill_view.set("market");
                                            selected_skill_key.set(None);
                                        },
                                        "技能市场"
                                        span { class: "ac-skills-hot", "HOT" }
                                    }
                                }
                                if skill_view() == "market" {
                                    div { class: "ac-sidebar-skills-stations",
                                        for station in skill_market_stations() {
                                            {
                                                let station_id = station.id.clone();
                                                let station_id_click = station.id.clone();
                                                let station_name = station.name.clone();
                                                let is_active = skill_market_station() == station_id;
                                                rsx! {
                                                    button {
                                                        r#type: "button",
                                                        class: if is_active {
                                                            "ac-sidebar-skills-station-btn is-active"
                                                        } else {
                                                            "ac-sidebar-skills-station-btn"
                                                        },
                                                        title: "{station_name}",
                                                        onclick: move |_| {
                                                            skill_market_station.set(station_id_click.clone());
                                                            selected_skill_key.set(None);
                                                        },
                                                        "{station_name}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if skill_view() == "installed" {
                                    if skill_installed_loading() {
                                        div { class: "ac-skills-list",
                                            for _ in 0..4 {
                                                div { class: "ac-skills-list-row ac-skills-list-row-skeleton",
                                                    div { class: "ac-skills-list-icon ac-skeleton-base" }
                                                    div { class: "ac-skills-list-body",
                                                        div { class: "ac-skills-list-skel-title ac-skeleton-base" }
                                                        div { class: "ac-skills-list-skel-desc ac-skeleton-base" }
                                                    }
                                                }
                                            }
                                        }
                                    } else if let Some(err) = skill_installed_error() {
                                        div { class: "ac-sidebar-skills-banner is-error", "{err}" }
                                    } else if skill_installed_filtered().is_empty() {
                                        div { class: "ac-sidebar-skills-empty",
                                            if skill_installed_list().is_empty() {
                                                "暂无已学习技能，去「技能市场」安装。"
                                            } else {
                                                "无匹配技能"
                                            }
                                        }
                                    } else {
                                        div { class: "ac-skills-section-header",
                                            span { "我的技能" }
                                        }
                                        div { class: "ac-skills-list",
                                            for inst in skill_installed_filtered() {
                                                {
                                                    let slug = inst.slug.clone();
                                                    let title = inst.title.clone();
                                                    let desc = inst.description.clone();
                                                    let key = slug.clone();
                                                    let key_select = key.clone();
                                                    let slug_click = slug.clone();
                                                    let slug_remove = slug.clone();
                                                    let equipped = skill_slug_equipped(
                                                        &skill_equipped_slugs(),
                                                        slug.as_str(),
                                                    );
                                                    let is_selected = selected_skill_key() == Some(key.clone());
                                                    rsx! {
                                                        div {
                                                            class: if is_selected {
                                                                "ac-skills-list-row is-selected"
                                                            } else {
                                                                "ac-skills-list-row"
                                                            },
                                                            onclick: move |_| {
                                                                selected_skill_key.set(Some(key_select.clone()));
                                                                let tab = CenterTab::skill_installed(
                                                                    key_select.clone(),
                                                                    title.clone(),
                                                                );
                                                                center_tabs.with_mut(|tabs| {
                                                                    active_center_id.with_mut(|active| {
                                                                        center_open_or_focus(tabs, active, tab);
                                                                    });
                                                                });
                                                            },
                                                            div { class: "ac-skills-list-icon ac-skill-icon-blue",
                                                                Icon { icon: LdGlobe, width: 16, height: 16, fill: "currentColor", class: "ac-skill-card-icon" }
                                                            }
                                                            div { class: "ac-skills-list-body",
                                                                div { class: "ac-skills-list-title", "{title}" }
                                                                div { class: "ac-skills-list-desc", "{desc}" }
                                                                div { class: "ac-skills-list-meta", "{slug}" }
                                                            }
                                                            div { class: "ac-skills-list-actions",
                                                                button {
                                                                    r#type: "button",
                                                                    class: if skill_slug_equipped(&skill_equipped_slugs(), slug.as_str()) {
                                                                        "ac-skills-list-action is-on"
                                                                    } else {
                                                                        "ac-skills-list-action"
                                                                    },
                                                                    title: "{skill_equip_button_label(&skill_equipped_slugs(), slug.as_str())}",
                                                                    disabled: skill_equip_busy() == Some(slug.clone()),
                                                                    onclick: move |evt| {
                                                                        evt.stop_propagation();
                                                                        let s = slug_click.clone();
                                                                        let equip = !skill_slug_equipped(&skill_equipped_slugs(), &s);
                                                                        skill_equip_busy.set(Some(s.clone()));
                                                                        skill_installed_error.set(None);
                                                                        spawn(async move {
                                                                            match toggle_skill_equip(s.clone(), equip).await {
                                                                                Ok(resp) => skill_equipped_slugs.set(resp.slugs),
                                                                                Err(err) => {
                                                                                    skill_installed_error.set(Some(err.to_string()));
                                                                                }
                                                                            }
                                                                            skill_equip_busy.set(None);
                                                                        });
                                                                    },
                                                                    Icon { icon: LdShieldCheck, width: 13, height: 13, fill: "currentColor", class: "ac-skill-action-icon" }
                                                                }
                                                                button {
                                                                    r#type: "button",
                                                                    class: "ac-skills-list-action is-danger",
                                                                    disabled: equipped
                                                                        || skill_uninstall_busy() == Some(slug.clone()),
                                                                    title: if !equipped {
                                                                        "移除"
                                                                    } else {
                                                                        "请先忘却后再移除"
                                                                    },
                                                                    onclick: move |evt| {
                                                                        evt.stop_propagation();
                                                                        if equipped {
                                                                            return;
                                                                        }
                                                                        let s = slug_remove.clone();
                                                                        skill_uninstall_busy.set(Some(s.clone()));
                                                                        skill_installed_error.set(None);
                                                                        spawn(async move {
                                                                            match uninstall_installed_skill(&s).await {
                                                                                Ok(()) => {
                                                                                    skill_installed_list.with_mut(|list| {
                                                                                        list.retain(|item| {
                                                                                            !item.slug.eq_ignore_ascii_case(&s)
                                                                                        });
                                                                                    });
                                                                                    skill_equipped_slugs.with_mut(|slugs| {
                                                                                        slugs.retain(|item| {
                                                                                            !item.eq_ignore_ascii_case(&s)
                                                                                        });
                                                                                    });
                                                                                    if selected_skill_key() == Some(s.clone()) {
                                                                                        selected_skill_key.set(None);
                                                                                    }
                                                                                    let tab_id = format!("skill:installed:{s}");
                                                                                    center_tabs.with_mut(|tabs| {
                                                                                        active_center_id.with_mut(|active| {
                                                                                            let _ = center_close_tab(
                                                                                                tabs,
                                                                                                active,
                                                                                                &tab_id,
                                                                                            );
                                                                                        });
                                                                                    });
                                                                                }
                                                                                Err(e) => {
                                                                                    skill_installed_error.set(Some(e.to_string()));
                                                                                }
                                                                            }
                                                                            skill_uninstall_busy.set(None);
                                                                        });
                                                                    },
                                                                    Icon { icon: LdTrash2, width: 13, height: 13, fill: "currentColor", class: "ac-skill-action-icon" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    if skill_market_loading() {
                                        div { class: "ac-skills-list",
                                            for _ in 0..5 {
                                                div { class: "ac-skills-list-row ac-skills-list-row-skeleton",
                                                    div { class: "ac-skills-list-icon ac-skeleton-base" }
                                                    div { class: "ac-skills-list-body",
                                                        div { class: "ac-skills-list-skel-title ac-skeleton-base" }
                                                        div { class: "ac-skills-list-skel-desc ac-skeleton-base" }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        if let Some(err) = skill_market_error() {
                                            div { class: "ac-sidebar-skills-banner is-error", "{err}" }
                                        }
                                        if !skill_market_station_warnings().is_empty() {
                                            div { class: "ac-sidebar-skills-banner is-warn",
                                                for warning in skill_market_station_warnings() {
                                                    span { "{warning}" }
                                                }
                                            }
                                        }
                                        if skill_market_station_cards().is_empty() && skill_market_error().is_none() {
                                            div { class: "ac-sidebar-skills-empty",
                                                if skill_market_items().is_empty() {
                                                    "输入关键词搜索技能市场"
                                                } else {
                                                    "当前下载站无匹配技能"
                                                }
                                            }
                                        } else if !skill_market_station_cards().is_empty() {
                                            div { class: "ac-skills-section-header",
                                                span { "{skill_source_label(&skill_market_station())}" }
                                            }
                                            div { class: "ac-skills-list",
                                                for card in skill_market_station_cards() {
                                                    {
                                                        let item = card.item.clone();
                                                        let item_name = item.name.clone();
                                                        let item_name_install = item.name.clone();
                                                        let source_label = card.source_label;
                                                        let downloads = card.downloads.clone();
                                                        let stars = card.stars.clone();
                                                        let install_key = card.install_key.clone();
                                                        let button_install_key = install_key.clone();
                                                        let key_select = install_key.clone();
                                                        let is_selected = selected_skill_key() == Some(install_key.clone());
                                                        rsx! {
                                                            div {
                                                                class: if is_selected {
                                                                    "ac-skills-list-row is-selected"
                                                                } else {
                                                                    "ac-skills-list-row"
                                                                },
                                                                onclick: move |_| {
                                                                    selected_skill_key.set(Some(key_select.clone()));
                                                                    let tab = CenterTab::skill_market(
                                                                        key_select.clone(),
                                                                        item_name.clone(),
                                                                    );
                                                                    center_tabs.with_mut(|tabs| {
                                                                        active_center_id.with_mut(|active| {
                                                                            center_open_or_focus(tabs, active, tab);
                                                                        });
                                                                    });
                                                                },
                                                                div { class: "ac-skills-list-icon ac-skill-icon-blue",
                                                                    Icon { icon: LdGlobe, width: 16, height: 16, fill: "currentColor", class: "ac-skill-card-icon" }
                                                                }
                                                                div { class: "ac-skills-list-body",
                                                                    div { class: "ac-skills-list-title", "{item.name}" }
                                                                    div { class: "ac-skills-list-desc", "{item.description}" }
                                                                    div { class: "ac-skills-list-meta", "{source_label} · {item.author} · ↓{downloads} · ★{stars}" }
                                                                }
                                                                div { class: "ac-skills-list-actions",
                                                                    if item.installed {
                                                                        span { class: "ac-skills-list-badge", "已装" }
                                                                    } else if !item.installable {
                                                                        span { class: "ac-skills-list-badge is-muted", "不可用" }
                                                                    } else {
                                                                        button {
                                                                            r#type: "button",
                                                                            class: "ac-skills-list-action",
                                                                            title: if skill_installing() == Some(install_key.clone()) {
                                                                                "安装中"
                                                                            } else {
                                                                                "学习 / 安装"
                                                                            },
                                                                            disabled: skill_installing().is_some(),
                                                                            onclick: move |evt| {
                                                                                evt.stop_propagation();
                                                                                let source = item.source.clone();
                                                                                let id = item.id.clone();
                                                                                let name = Some(item_name_install.clone());
                                                                                let url = item.install_url.clone();
                                                                                let key = button_install_key.clone();
                                                                                skill_installing.set(Some(key));
                                                                                spawn(async move {
                                                                                    match install_skill(source, id, name, url).await {
                                                                                        Ok(done) => {
                                                                                            toast.success(done.message);
                                                                                            skill_market_loaded_query.set(None);
                                                                                            if let Ok(resp) = load_installed_skills().await {
                                                                                                skill_installed_list.set(resp.skills);
                                                                                            }
                                                                                        }
                                                                                        Err(e) => {
                                                                                            toast.error(format!("安装失败：{e}"));
                                                                                        }
                                                                                    }
                                                                                    skill_installing.set(None);
                                                                                });
                                                                            },
                                                                            Icon { icon: LdDownload, width: 13, height: 13, fill: "currentColor", class: "ac-skill-action-icon" }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else if active_tab() == "plugins" {
                            super::plugins::SidebarPluginMarket {
                                selected_key: selected_plugin_key,
                                catalog: plugin_catalog,
                                refresh_tick: plugin_refresh_tick,
                                on_open: move |(key, title): (String, String)| {
                                    if center_shutting_down() {
                                        center_shutting_down.set(false);
                                    }
                                    show_center.set(true);
                                    selected_plugin_key.set(Some(key.clone()));
                                    center_tabs.with_mut(|tabs| {
                                        active_center_id.with_mut(|active| {
                                            center_open_or_focus(
                                                tabs,
                                                active,
                                                CenterTab::plugin(key, title),
                                            );
                                        });
                                    });
                                },
                            }
                        } else if active_tab() == "files" {
                            super::files::SidebarFileExplorer {
                                root_path: fs_root_path(),
                                section_open: fs_section_open,
                                expanded: fs_expanded,
                                children_cache: fs_children_cache,
                                selected_path: fs_selected_path,
                                create_mode: fs_create_mode,
                                create_name: fs_create_name,
                                create_parent: fs_create_parent,
                                notice: fs_notice,
                                highlighted_paths: fs_just_written,
                                on_open_file: move |path: String| {
                                    fs_selected_path.set(Some(path.clone()));
                                    fs_save_notice.set(None);
                                    if center_shutting_down() {
                                        center_shutting_down.set(false);
                                    }
                                    show_center.set(true);
                                    center_tabs.with_mut(|tabs| {
                                        active_center_id.with_mut(|active| {
                                            center_open_or_focus(tabs, active, CenterTab::file(path));
                                        });
                                    });
                                },
                            }
                        } else {
                            if role_loading() {
                                div { class: "ac-skills-state", "正在加载角色..." }
                            } else if let Some(err) = role_error() {
                                div { class: "ac-skills-state ac-skills-state-error", "{err}" }
                            } else {
                                for role in role_list() {
                                    {
                                        let role_id = role.id;
                                        let role_name = role.name.clone();
                                        let role_name_delete = role.name.clone();
                                        let role_prompt = role.system_prompt.clone();
                                        let can_delete_this_role = role_list().len() > 1;
                                        rsx! {
                                            div {
                                                class: if role.is_active { "sidebar-agent is-active ac-agent-card" } else { "sidebar-agent ac-agent-card" },
                                                div { class: "ac-agent-card-toolbar",
                                                    button {
                                                        r#type: "button",
                                                        class: "ac-agent-select-hit",
                                                        onclick: move |_| {
                                                            if role.is_active {
                                                                editing_role_id.set(Some(role_id));
                                                                editing_role_name.set(role_name.clone());
                                                                persona_prompt.set(role_prompt.clone());
                                                                persona_textarea_epoch.with_mut(|n| *n += 1);
                                                                active_tab.set("role");
                                                                center_tabs.with_mut(|tabs| {
                                                                    active_center_id.with_mut(|active| {
                                                                        center_open_or_focus(tabs, active, CenterTab::role());
                                                                    });
                                                                });
                                                                return;
                                                            }
                                                            role_loading.set(true);
                                                            spawn(async move {
                                                                match activate_role(role_id).await {
                                                                    Ok(active_role) => {
                                                                        role_list.with_mut(|items| {
                                                                            for r in items.iter_mut() {
                                                                                r.is_active = r.id == active_role.id;
                                                                                if r.id == active_role.id {
                                                                                    r.name = active_role.name.clone();
                                                                                    r.system_prompt = active_role.system_prompt.clone();
                                                                                }
                                                                            }
                                                                        });
                                                                        editing_role_id.set(Some(active_role.id));
                                                                        editing_role_name.set(active_role.name.clone());
                                                                        persona_prompt.set(active_role.system_prompt);
                                                                        persona_textarea_epoch.with_mut(|n| *n += 1);
                                                                        if let Ok(equipped) = load_equipped_skills().await {
                                                                            skill_equipped_slugs.set(equipped.slugs);
                                                                        } else {
                                                                            skill_equipped_slugs.set(Vec::new());
                                                                        }
                                                                        role_error.set(None);
                                                                        active_tab.set("role");
                                                                        center_tabs.with_mut(|tabs| {
                                                                            active_center_id.with_mut(|active| {
                                                                                center_open_or_focus(tabs, active, CenterTab::role());
                                                                            });
                                                                        });
                                                                    }
                                                                    Err(e) => role_error.set(Some(e.to_string())),
                                                                }
                                                                role_loading.set(false);
                                                            });
                                                        },
                                                        div { class: "ac-agent-row",
                                                            div { class: "ac-agent-avatar",
                                                                img {
                                                                    src: PET_PNG,
                                                                    alt: "",
                                                                    class: "ac-agent-avatar-img",
                                                                }
                                                            }
                                                            div { class: "ac-agent-meta",
                                                                h4 { class: "ac-agent-name", "{role.name}" }
                                                            }
                                                        }
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "ac-agent-delete",
                                                        title: if can_delete_this_role { "删除角色" } else { "至少保留一个角色" },
                                                        disabled: !can_delete_this_role,
                                                        onclick: move |evt| {
                                                            evt.stop_propagation();
                                                            if !can_delete_this_role {
                                                                return;
                                                            }
                                                            delete_role_confirm
                                                                .set(Some((role_id, role_name_delete.clone())));
                                                        },
                                                        Icon {
                                                            icon: LdTrash2,
                                                            width: 15,
                                                            height: 15,
                                                            fill: "currentColor",
                                                            class: "ac-agent-delete-icon",
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "ac-btn-new-agent",
                                onclick: move |_| {
                                    role_loading.set(true);
                                    let default_name = next_new_role_display_name(&role_list());
                                    let default_prompt = persona::DEFAULT_SYSTEM_PROMPT.to_string();
                                    spawn(async move {
                                        match create_role(RoleCreateRequest {
                                            name: default_name,
                                            system_prompt: default_prompt,
                                        })
                                        .await
                                        {
                                            Ok(created) => {
                                                match activate_role(created.id).await {
                                                    Ok(active_role) => {
                                                        role_list.with_mut(|items| {
                                                            if !items.iter().any(|r| r.id == created.id) {
                                                                items.insert(0, created.clone());
                                                            }
                                                            for r in items.iter_mut() {
                                                                r.is_active = r.id == active_role.id;
                                                                if r.id == active_role.id {
                                                                    r.name = active_role.name.clone();
                                                                    r.system_prompt = active_role.system_prompt.clone();
                                                                }
                                                            }
                                                        });
                                                        editing_role_id.set(Some(active_role.id));
                                                        editing_role_name.set(active_role.name.clone());
                                                        persona_prompt.set(active_role.system_prompt);
                                                        persona_textarea_epoch.with_mut(|n| *n += 1);
                                                        skill_equipped_slugs.set(Vec::new());
                                                        role_error.set(None);
                                                        active_tab.set("role");
                                                        center_tabs.with_mut(|tabs| {
                                                            active_center_id.with_mut(|active| {
                                                                center_open_or_focus(tabs, active, CenterTab::role());
                                                            });
                                                        });
                                                    }
                                                    Err(e) => role_error.set(Some(e.to_string())),
                                                }
                                            }
                                            Err(e) => role_error.set(Some(e.to_string())),
                                        }
                                        role_loading.set(false);
                                    });
                                },
                                Icon {
                                    icon: LdPlus,
                                    width: 14,
                                    height: 14,
                                    fill: "currentColor",
                                    class: "ac-btn-icon",
                                }
                                "创建新角色"
                            }
                        }
                    }
                    if show_sidebar() {
                        div {
                            class: "ac-sidebar-resizer",
                            title: "拖动调整侧栏宽度；缩小过多将收起侧栏",
                            onmousedown: move |event| {
                                active_resize.set(Some(ConsoleResizeKind::Sidebar));
                                last_pointer_x.set(Some(event.data.client_coordinates().x));
                            },
                        }
                    }
                }

            div { class: "{center_class}",
                if !center_tabs().is_empty() {
                    div { class: "ac-center-tabs", role: "tablist",
                        for tab in center_tabs() {
                            {
                                let tab_id = tab.id.clone();
                                let tab_id_close = tab.id.clone();
                                let tab_id_middle = tab.id.clone();
                                let is_active = active_center_id().as_deref() == Some(tab.id.as_str());
                                let title = tab.title.clone();
                                let kind_focus = tab.kind.clone();
                                let file_dirty = center_tab_file_path(&tab.kind)
                                    .is_some_and(|p| fs_dirty().contains(p));
                                rsx! {
                                    div {
                                        key: "{tab_id}",
                                        class: if is_active {
                                            "ac-center-tab is-active"
                                        } else {
                                            "ac-center-tab"
                                        },
                                        role: "tab",
                                        aria_selected: is_active,
                                        title: "{title}",
                                        onmousedown: move |e| {
                                            if e.trigger_button() != Some(MouseButton::Auxiliary) {
                                                return;
                                            }
                                            e.prevent_default();
                                            let removed = center_tabs.with_mut(|tabs| {
                                                active_center_id.with_mut(|active| {
                                                    center_close_tab(tabs, active, &tab_id_middle)
                                                })
                                            });
                                            if let Some(removed) = removed {
                                                if let Some(key) = center_tab_skill_key(&removed.kind) {
                                                    if selected_skill_key().as_deref() == Some(key.as_str()) {
                                                        selected_skill_key.set(None);
                                                    }
                                                }
                                                if let Some(key) = center_tab_plugin_key(&removed.kind) {
                                                    if selected_plugin_key().as_deref() == Some(key) {
                                                        selected_plugin_key.set(None);
                                                    }
                                                }
                                                if let Some(path) = center_tab_file_path(&removed.kind) {
                                                    let path = path.to_string();
                                                    fs_drafts.with_mut(|m| {
                                                        fs_baselines.with_mut(|b| {
                                                            fs_dirty.with_mut(|d| {
                                                                fs_load_errors.with_mut(|e| {
                                                                    fs_forget_open_file(&path, m, b, d, e);
                                                                });
                                                            });
                                                        });
                                                    });
                                                    fs_save_notice.set(None);
                                                }
                                            }
                                            if let Some(next) = center_active_kind(&center_tabs(), &active_center_id()) {
                                                if let Some(key) = center_tab_skill_key(&next) {
                                                    selected_skill_key.set(Some(key));
                                                }
                                                if let Some(key) = center_tab_plugin_key(&next) {
                                                    selected_plugin_key.set(Some(key.to_string()));
                                                }
                                            }
                                        },
                                        onclick: move |_| {
                                            active_center_id.set(Some(tab_id.clone()));
                                            if let Some(key) = center_tab_skill_key(&kind_focus) {
                                                selected_skill_key.set(Some(key));
                                            }
                                            if let Some(key) = center_tab_plugin_key(&kind_focus) {
                                                selected_plugin_key.set(Some(key.to_string()));
                                            }
                                            if let Some(path) = center_tab_display_path(&kind_focus) {
                                                fs_selected_path.set(Some(path.to_string()));
                                                fs_save_notice.set(None);
                                            }
                                        },
                                        if file_dirty {
                                            span { class: "ac-center-tab-dirty", title: "未保存", "●" }
                                        }
                                        span { class: "ac-center-tab-title", "{title}" }
                                        button {
                                            r#type: "button",
                                            class: "ac-center-tab-close",
                                            title: "关闭标签页",
                                            aria_label: "关闭标签页",
                                            onclick: move |e| {
                                                e.stop_propagation();
                                                let removed = center_tabs.with_mut(|tabs| {
                                                    active_center_id.with_mut(|active| {
                                                        center_close_tab(tabs, active, &tab_id_close)
                                                    })
                                                });
                                                if let Some(removed) = removed {
                                                    if let Some(key) = center_tab_skill_key(&removed.kind) {
                                                        if selected_skill_key().as_deref() == Some(key.as_str()) {
                                                            selected_skill_key.set(None);
                                                        }
                                                    }
                                                    if let Some(key) = center_tab_plugin_key(&removed.kind) {
                                                        if selected_plugin_key().as_deref() == Some(key) {
                                                            selected_plugin_key.set(None);
                                                        }
                                                    }
                                                    if let Some(path) = center_tab_file_path(&removed.kind) {
                                                        let path = path.to_string();
                                                        fs_drafts.with_mut(|m| {
                                                            fs_baselines.with_mut(|b| {
                                                                fs_dirty.with_mut(|d| {
                                                                    fs_load_errors.with_mut(|e| {
                                                                        fs_forget_open_file(&path, m, b, d, e);
                                                                    });
                                                                });
                                                            });
                                                        });
                                                        fs_save_notice.set(None);
                                                    }
                                                }
                                                if let Some(next) = center_active_kind(&center_tabs(), &active_center_id()) {
                                                    if let Some(key) = center_tab_skill_key(&next) {
                                                        selected_skill_key.set(Some(key));
                                                    }
                                                    if let Some(key) = center_tab_plugin_key(&next) {
                                                        selected_plugin_key.set(Some(key.to_string()));
                                                    }
                                                }
                                            },
                                            Icon {
                                                icon: LdX,
                                                width: 12,
                                                height: 12,
                                                fill: "currentColor",
                                                class: "ac-center-tab-close-icon",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "ac-main-body premium-card",
                    {
                        let active_kind = center_active_kind(&center_tabs(), &active_center_id());
                        match active_kind {
                            Some(CenterTabKind::SkillInstalled { slug: key }) => rsx! {
                        section { class: "ac-skills-detail-pane",
                            {
                                        if let Some(inst) = skill_installed_list()
                                            .into_iter()
                                            .find(|s| s.slug == key)
                                        {
                                            let slug = inst.slug.clone();
                                            let title = inst.title.clone();
                                            let desc = inst.description.clone();
                                            let slug_click = slug.clone();
                                            let slug_remove = slug.clone();
                                            let equipped = skill_slug_equipped(
                                                &skill_equipped_slugs(),
                                                slug.as_str(),
                                            );
                                            rsx! {
                                                div { class: "ac-skills-detail",
                                                    div { class: "ac-skills-detail-head",
                                                        div { class: "ac-skills-detail-icon ac-skill-icon-blue",
                                                            Icon { icon: LdGlobe, width: 28, height: 28, fill: "currentColor", class: "ac-skill-card-icon" }
                                                        }
                                                        div { class: "ac-skills-detail-titles",
                                                            h2 { "{title}" }
                                                            p { class: "ac-skills-detail-slug", "{slug}" }
                                                        }
                                                    }
                                                    p { class: "ac-skills-detail-desc", "{desc}" }
                                                    div { class: "ac-skills-detail-actions",
                                                        button {
                                                            r#type: "button",
                                                            class: if skill_slug_equipped(&skill_equipped_slugs(), slug.as_str()) {
                                                                "ac-installed-equip is-on"
                                                            } else {
                                                                "ac-installed-equip"
                                                            },
                                                            disabled: skill_equip_busy() == Some(slug.clone()),
                                                            onclick: move |_| {
                                                                let s = slug_click.clone();
                                                                let equip = !skill_slug_equipped(&skill_equipped_slugs(), &s);
                                                                skill_equip_busy.set(Some(s.clone()));
                                                                skill_installed_error.set(None);
                                                                spawn(async move {
                                                                    match toggle_skill_equip(s.clone(), equip).await {
                                                                        Ok(resp) => skill_equipped_slugs.set(resp.slugs),
                                                                        Err(err) => {
                                                                            skill_installed_error.set(Some(err.to_string()));
                                                                        }
                                                                    }
                                                                    skill_equip_busy.set(None);
                                                                });
                                                            },
                                                            Icon { icon: LdShieldCheck, width: 15, height: 15, fill: "currentColor", class: "ac-skill-action-icon" }
                                                            "{skill_equip_button_label(&skill_equipped_slugs(), slug.as_str())}"
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            class: "ac-installed-action",
                                                            disabled: equipped
                                                                || skill_uninstall_busy() == Some(slug.clone()),
                                                            title: if !equipped {
                                                                "删除本地技能文件"
                                                            } else {
                                                                "请先点击「忘却」后再移除"
                                                            },
                                                            onclick: move |_| {
                                                                if equipped {
                                                                    return;
                                                                }
                                                                let s = slug_remove.clone();
                                                                skill_uninstall_busy.set(Some(s.clone()));
                                                                skill_installed_error.set(None);
                                                                spawn(async move {
                                                                    match uninstall_installed_skill(&s).await {
                                                                        Ok(()) => {
                                                                            skill_installed_list.with_mut(|list| {
                                                                                list.retain(|item| {
                                                                                    !item.slug.eq_ignore_ascii_case(&s)
                                                                                });
                                                                            });
                                                                            skill_equipped_slugs.with_mut(|slugs| {
                                                                                slugs.retain(|item| {
                                                                                    !item.eq_ignore_ascii_case(&s)
                                                                                });
                                                                            });
                                                                            selected_skill_key.set(None);
                                                                            let tab_id = format!("skill:installed:{s}");
                                                                            center_tabs.with_mut(|tabs| {
                                                                                active_center_id.with_mut(|active| {
                                                                                    let _ = center_close_tab(
                                                                                        tabs,
                                                                                        active,
                                                                                        &tab_id,
                                                                                    );
                                                                                });
                                                                            });
                                                                        }
                                                                        Err(e) => {
                                                                            skill_installed_error.set(Some(e.to_string()));
                                                                        }
                                                                    }
                                                                    skill_uninstall_busy.set(None);
                                                                });
                                                            },
                                                            Icon { icon: LdTrash2, width: 15, height: 15, fill: "currentColor", class: "ac-skill-action-icon" }
                                                            "移除"
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                div { class: "ac-skills-detail-empty",
                                                    Icon { icon: LdWandSparkles, width: 40, height: 40, fill: "currentColor", class: "ac-installed-empty-icon" }
                                                    h3 { "技能库" }
                                                    p { "该技能已不存在或已卸载。" }
                                                }
                                            }
                                        }
                            }
                        }
                            },
                            Some(CenterTabKind::SkillMarket { key }) => rsx! {
                        section { class: "ac-skills-detail-pane",
                            {
                                    if let Some(card) = skill_market_cards()
                                        .into_iter()
                                        .find(|c| c.install_key == key)
                                    {
                                        let item = card.item.clone();
                                        let source_label = card.source_label;
                                        let downloads = card.downloads.clone();
                                        let stars = card.stars.clone();
                                        let install_key = card.install_key.clone();
                                        let button_install_key = install_key.clone();
                                        rsx! {
                                            div { class: "ac-skills-detail",
                                                div { class: "ac-skills-detail-head",
                                                    div { class: "ac-skills-detail-icon ac-skill-icon-blue",
                                                        Icon { icon: LdGlobe, width: 28, height: 28, fill: "currentColor", class: "ac-skill-card-icon" }
                                                    }
                                                    div { class: "ac-skills-detail-titles",
                                                        span { class: "ac-skill-source", "{source_label}" }
                                                        h2 { "{item.name}" }
                                                    }
                                                }
                                                p { class: "ac-skills-detail-desc", "{item.description}" }
                                                div { class: "ac-skill-market-meta",
                                                    span { "作者 {item.author}" }
                                                    span { "下载 {downloads}" }
                                                    span { "星标 {stars}" }
                                                    span { "版本 {item.version}" }
                                                }
                                                div { class: "ac-skills-detail-actions",
                                                    if item.installed {
                                                        button { r#type: "button", class: "ac-skill-installed-btn", disabled: true, "已安装" }
                                                    } else if !item.installable {
                                                        button { r#type: "button", class: "ac-skill-installed-btn", disabled: true, "暂不支持下载" }
                                                    } else {
                                                        button {
                                                            r#type: "button",
                                                            class: "ac-skill-install-btn",
                                                            disabled: skill_installing().is_some(),
                                                            onclick: move |_| {
                                                                let source = item.source.clone();
                                                                let id = item.id.clone();
                                                                let name = Some(item.name.clone());
                                                                let url = item.install_url.clone();
                                                                let key = button_install_key.clone();
                                                                skill_installing.set(Some(key));
                                                                spawn(async move {
                                                                    match install_skill(source, id, name, url).await {
                                                                        Ok(done) => {
                                                                            toast.success(done.message);
                                                                            skill_market_loaded_query.set(None);
                                                                            if let Ok(resp) = load_installed_skills().await {
                                                                                skill_installed_list.set(resp.skills);
                                                                            }
                                                                        }
                                                                        Err(e) => {
                                                                            toast.error(format!("安装失败：{e}"));
                                                                        }
                                                                    }
                                                                    skill_installing.set(None);
                                                                });
                                                            },
                                                            Icon { icon: LdDownload, width: 15, height: 15, fill: "currentColor", class: "ac-skill-action-icon" }
                                                            if skill_installing() == Some(install_key) {
                                                                "安装中"
                                                            } else {
                                                                "一键下载"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        rsx! {
                                            div { class: "ac-skills-detail-empty",
                                                Icon { icon: LdWandSparkles, width: 40, height: 40, fill: "currentColor", class: "ac-installed-empty-icon" }
                                                h3 { "技能市场" }
                                                p { "该技能已不在当前搜索结果中，可重新搜索后打开。" }
                                            }
                                        }
                                    }
                            }
                        }
                            },
                            Some(CenterTabKind::Plugin { key }) => rsx! {
                                super::plugins::PluginDetailPane {
                                    key: "{key}",
                                    plugin_key: key,
                                    catalog: plugin_catalog,
                                    refresh_tick: plugin_refresh_tick,
                                    on_uninstalled: move |key: String| {
                                        selected_plugin_key.set(None);
                                        let tab_id = format!("plugin:{key}");
                                        center_tabs.with_mut(|tabs| {
                                            active_center_id.with_mut(|active| {
                                                let _ = center_close_tab(tabs, active, &tab_id);
                                            });
                                        });
                                    },
                                }
                            },
                            Some(CenterTabKind::File { path }) => rsx! {
                                super::files::FileEditorPane {
                                    key: "{path}",
                                    path,
                                    drafts: fs_drafts,
                                    baselines: fs_baselines,
                                    dirty: fs_dirty,
                                    load_errors: fs_load_errors,
                                    save_notice: fs_save_notice,
                                }
                            },
                            Some(CenterTabKind::FileDiff {
                                path,
                                old_text,
                                new_text,
                            }) => {
                                let open_path = path.clone();
                                let can_open = super::files::fs_available();
                                rsx! {
                                    super::files::FileEditDiffPane {
                                        key: "diff:{path}",
                                        path: path.clone(),
                                        old_text,
                                        new_text,
                                        can_open_current: can_open,
                                        on_open_current: move |_| {
                                            let resolved = reveal_and_select_workspace_file(
                                                &open_path,
                                                show_sidebar,
                                                sidebar_shutting_down,
                                                show_center,
                                                center_shutting_down,
                                                active_tab,
                                                fs_section_open,
                                                fs_root_path,
                                                fs_expanded,
                                                fs_children_cache,
                                                fs_selected_path,
                                                fs_save_notice,
                                            );
                                            if resolved.is_empty() {
                                                return;
                                            }
                                            center_tabs.with_mut(|tabs| {
                                                active_center_id.with_mut(|active| {
                                                    center_open_or_focus(
                                                        tabs,
                                                        active,
                                                        CenterTab::file(resolved),
                                                    );
                                                });
                                            });
                                        },
                                    }
                                }
                            },
                            Some(CenterTabKind::Role) => rsx! {
                        section { class: "ac-role-panel",
                            div { class: "ac-role-header",
                                div { class: "ac-role-heading",
                                    div { class: "ac-role-avatar",
                                        img {
                                            src: PET_PNG,
                                            alt: "",
                                            class: "ac-role-avatar-img",
                                        }
                                    }
                                    div { class: "ac-role-heading-text",
                                        input {
                                            class: "ac-role-name-input",
                                            r#type: "text",
                                            placeholder: "角色名称",
                                            value: "{editing_role_name()}",
                                            oninput: move |e| editing_role_name.set(e.value()),
                                        }
                                        p { class: "ac-role-subtitle", "正在严格遵循以下人格提示词为您工作。" }
                                    }
                                }
                            }
                            div { class: "ac-role-divider" }
                            div { class: "ac-persona-prompt-card",
                                div { class: "ac-persona-prompt-header",
                                    div { class: "ac-persona-prompt-title",
                                        Icon {
                                            icon: LdScrollText,
                                            width: 18,
                                            height: 18,
                                            fill: "currentColor",
                                            class: "ac-persona-prompt-icon",
                                        }
                                        "人格提示词"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "ac-persona-magic-btn",
                                        disabled: persona_polishing(),
                                        onclick: move |_| {
                                            if persona_polishing() {
                                                return;
                                            }
                                            let current = persona_prompt().trim().to_string();
                                            if current.is_empty() {
                                                persona_polish_hint.set(Some("请先填写人格提示词，再使用 AI 润色。".into()));
                                                return;
                                            }
                                            persona_polishing.set(true);
                                            persona_polish_hint.set(None);
                                            let model_for_polish = chat_model();
                                            spawn(async move {
                                                #[cfg(all(target_arch = "wasm32", feature = "web"))]
                                                {
                                                    let configured = match llm_config::fetch_llm_config().await {
                                                        Some(cfg) => {
                                                            llm_server_openai.set(Some(cfg.api_key_configured));
                                                            cfg.api_key_configured
                                                        }
                                                        None => llm_server_openai().unwrap_or(false),
                                                    };
                                                    if !configured {
                                                        llm_modal_api_key.set(String::new());
                                                        llm_modal_base.set(llm_config::DEFAULT_OPENAI_V1_BASE.to_string());
                                                        show_llm_modal.set(true);
                                                        persona_polishing.set(false);
                                                        return;
                                                    }
                                                }

                                                let model_trim = model_for_polish.trim().to_string();
                                                if model_trim.is_empty() {
                                                    persona_polish_hint.set(Some("请先在聊天栏选择模型，再使用 AI 润色。".into()));
                                                    persona_polishing.set(false);
                                                    return;
                                                }
                                                match persona::polish_persona_remote(
                                                    &current,
                                                    &model_trim,
                                                )
                                                .await
                                                {
                                                    Ok(polished) => {
                                                        persona_prompt.set(polished);
                                                        persona_textarea_epoch.with_mut(|n| *n += 1);
                                                        persona_polish_hint.set(Some("AI 润色完成，可继续调整后保存人格。".into()));
                                                    }
                                                    Err(e) => {
                                                        persona_polish_hint.set(Some(format!("AI 润色失败：{e}")));
                                                    }
                                                }
                                                persona_polishing.set(false);
                                            });
                                        },
                                        Icon {
                                            icon: LdSparkles,
                                            width: 16,
                                            height: 16,
                                            fill: "currentColor",
                                            class: "ac-persona-prompt-icon",
                                        }
                                        if persona_polishing() {
                                            "润色中..."
                                        } else {
                                            "AI 魔法润色"
                                        }
                                    }
                                }
                                // 受控 `value`：与聊天输入框一致，确保「AI 魔法润色」读到的是当前输入，而非脱节的 initial_value。
                                textarea {
                                    key: "persona-prompt-{persona_textarea_epoch()}",
                                    class: "ac-persona-textarea",
                                    value: "{persona_prompt()}",
                                    oninput: move |e| persona_prompt.set(e.value()),
                                }
                                if let Some(hint) = persona_polish_hint() {
                                    p {
                                        class: if hint == "AI 润色完成，可继续调整后保存人格。"
                                            || hint == "人格设定已保存。"
                                        {
                                            "ac-settings-saved ac-settings-saved--ok"
                                        } else {
                                            "ac-settings-saved"
                                        },
                                        "{hint}"
                                    }
                                }
                            }
                            div { class: "ac-role-actions",
                                button {
                                    r#type: "button",
                                    class: "ac-role-btn ac-role-btn-secondary",
                                    onclick: move |_| {
                                        show_sidebar.set(false);
                                        show_center.set(false);
                                        show_chat.set(true);
                                    },
                                    Icon {
                                        icon: LdMessageCircle,
                                        width: 18,
                                        height: 18,
                                        fill: "currentColor",
                                        class: "ac-role-btn-icon",
                                    }
                                    "去对话"
                                }
                                button {
                                    r#type: "button",
                                    class: "ac-role-btn ac-role-btn-primary",
                                    onclick: move |_| {
                                        let text = persona_prompt();
                                        let role_name = editing_role_name();
                                        let role_id = editing_role_id();
                                        spawn(async move {
                                            if let Some(id) = role_id {
                                                match update_role(
                                                    id,
                                                    RoleUpdateRequest {
                                                        name: Some(role_name.clone()),
                                                        system_prompt: Some(text.clone()),
                                                    },
                                                )
                                                .await
                                                {
                                                    Ok(updated) => {
                                                        role_list.with_mut(|items| {
                                                            for r in items.iter_mut() {
                                                                if r.id == updated.id {
                                                                    *r = updated.clone();
                                                                }
                                                            }
                                                        });
                                                        if role_list().iter().any(|r| r.id == updated.id && r.is_active)
                                                        {
                                                            let _ = persona::save_persona_remote(&text).await;
                                                        }
                                                        persona_polish_hint.set(Some("人格设定已保存。".into()));
                                                        role_error.set(None);
                                                    }
                                                    Err(e) => role_error.set(Some(e.to_string())),
                                                }
                                            } else {
                                                let _ = persona::save_persona_remote(&text).await;
                                                persona_polish_hint.set(Some("人格设定已保存。".into()));
                                            }
                                        });
                                    },
                                    Icon {
                                        icon: LdSave,
                                        width: 18,
                                        height: 18,
                                        fill: "currentColor",
                                        class: "ac-role-btn-icon",
                                    }
                                    "保存人格"
                                }
                            }
                        }
                            },
                            None => rsx! {
                                div { class: "ac-center-empty",
                                    div { class: "ac-center-empty-logo-wrap",
                                        img {
                                            src: LOGO_PNG,
                                            alt: "Pusa",
                                            class: "ac-center-empty-logo",
                                            oncontextmenu: move |evt| {
                                                evt.prevent_default();
                                                if super::files::fs_available() {
                                                    home_logo_menu_open.set(true);
                                                }
                                            },
                                        }
                                        if home_logo_menu_open() && super::files::fs_available() {
                                            div {
                                                class: "ac-center-empty-logo-menu-backdrop",
                                                onclick: move |_| home_logo_menu_open.set(false),
                                            }
                                            div {
                                                class: "ac-center-empty-logo-menu",
                                                role: "menu",
                                                button {
                                                    r#type: "button",
                                                    class: "ac-center-empty-logo-menu-item",
                                                    role: "menuitem",
                                                    onclick: move |_| {
                                                        home_logo_menu_open.set(false);
                                                        match super::files::fs_open_project_directory_dialog() {
                                                            Ok(Some(root)) => {
                                                                apply_opened_project_root(
                                                                    root,
                                                                    fs_root_path,
                                                                    fs_expanded,
                                                                    fs_children_cache,
                                                                    fs_selected_path,
                                                                    fs_create_mode,
                                                                    fs_create_name,
                                                                    fs_create_parent,
                                                                    fs_notice,
                                                                    recent_project_dirs,
                                                                    active_tab,
                                                                    center_tabs,
                                                                    active_center_id,
                                                                    home_logo_menu_open,
                                                                    plugin_refresh_tick,
                                                                );
                                                            }
                                                            Ok(None) => {}
                                                            Err(e) => fs_notice.set(Some(e)),
                                                        }
                                                    },
                                                    "打开项目"
                                                }
                                                if !recent_project_dirs().is_empty() {
                                                    div { class: "ac-center-empty-logo-menu-sep", role: "separator" }
                                                    for dir in recent_project_dirs() {
                                                        {
                                                            let dir_path = dir.clone();
                                                            let dir_label = super::files::fs_root_display_name(&dir);
                                                            let dir_title = dir.clone();
                                                            rsx! {
                                                                button {
                                                                    r#type: "button",
                                                                    class: "ac-center-empty-logo-menu-item",
                                                                    role: "menuitem",
                                                                    title: "{dir_title}",
                                                                    onclick: move |_| {
                                                                        home_logo_menu_open.set(false);
                                                                        match super::files::fs_set_workspace_root(&dir_path) {
                                                                            Ok(root) => {
                                                                                apply_opened_project_root(
                                                                                    root,
                                                                                    fs_root_path,
                                                                                    fs_expanded,
                                                                                    fs_children_cache,
                                                                                    fs_selected_path,
                                                                                    fs_create_mode,
                                                                                    fs_create_name,
                                                                                    fs_create_parent,
                                                                                    fs_notice,
                                                                                    recent_project_dirs,
                                                                                    active_tab,
                                                                                    center_tabs,
                                                                                    active_center_id,
                                                                                    home_logo_menu_open,
                                                                                    plugin_refresh_tick,
                                                                                );
                                                                            }
                                                                            Err(e) => {
                                                                                recent_project_dirs.set(
                                                                                    super::files::fs_recent_project_dirs(),
                                                                                );
                                                                                fs_notice.set(Some(e));
                                                                            }
                                                                        }
                                                                    },
                                                                    "{dir_label}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    p { class: "ac-center-empty-intro", "万千世界，皆为梦幻泡影" }
                                    if super::files::fs_available() {
                                        div {
                                            class: "ac-center-empty-actions",
                                            tabindex: "0",
                                            onkeydown: move |e: KeyboardEvent| {
                                                if create_plugin_open() || create_app_open() {
                                                    return;
                                                }
                                                if is_mod_char_shortcut(&e, "o") {
                                                    e.prevent_default();
                                                    match super::files::fs_open_project_directory_dialog() {
                                                        Ok(Some(root)) => {
                                                            apply_opened_project_root(
                                                                root,
                                                                fs_root_path,
                                                                fs_expanded,
                                                                fs_children_cache,
                                                                fs_selected_path,
                                                                fs_create_mode,
                                                                fs_create_name,
                                                                fs_create_parent,
                                                                fs_notice,
                                                                recent_project_dirs,
                                                                active_tab,
                                                                center_tabs,
                                                                active_center_id,
                                                                home_logo_menu_open,
                                                                plugin_refresh_tick,
                                                            );
                                                        }
                                                        Ok(None) => {}
                                                        Err(err) => fs_notice.set(Some(err)),
                                                    }
                                                } else if is_mod_char_shortcut(&e, "j") {
                                                    e.prevent_default();
                                                    create_app_name.set(String::new());
                                                    create_app_error.set(None);
                                                    create_app_open.set(true);
                                                } else if is_mod_char_shortcut(&e, "e") {
                                                    e.prevent_default();
                                                    create_plugin_name.set(String::new());
                                                    create_plugin_error.set(None);
                                                    create_plugin_open.set(true);
                                                }
                                            },
                                            button {
                                                r#type: "button",
                                                class: "ac-center-empty-open-btn",
                                                title: "打开项目（⌘O）",
                                                onclick: move |_| {
                                                    match super::files::fs_open_project_directory_dialog() {
                                                        Ok(Some(root)) => {
                                                            apply_opened_project_root(
                                                                root,
                                                                fs_root_path,
                                                                fs_expanded,
                                                                fs_children_cache,
                                                                fs_selected_path,
                                                                fs_create_mode,
                                                                fs_create_name,
                                                                fs_create_parent,
                                                                fs_notice,
                                                                recent_project_dirs,
                                                                active_tab,
                                                                center_tabs,
                                                                active_center_id,
                                                                home_logo_menu_open,
                                                                plugin_refresh_tick,
                                                            );
                                                        }
                                                        Ok(None) => {}
                                                        Err(e) => fs_notice.set(Some(e)),
                                                    }
                                                },
                                                span { class: "ac-center-empty-open-btn-label", "打开项目" }
                                                span { class: "ac-center-empty-open-btn-keys", aria_hidden: "true",
                                                    kbd { "⌘" }
                                                    kbd { "O" }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "ac-center-empty-open-btn",
                                                title: "创建应用（⌘J）",
                                                onclick: move |_| {
                                                    create_app_name.set(String::new());
                                                    create_app_error.set(None);
                                                    create_app_open.set(true);
                                                },
                                                span { class: "ac-center-empty-open-btn-label", "创建应用" }
                                                span { class: "ac-center-empty-open-btn-keys", aria_hidden: "true",
                                                    kbd { "⌘" }
                                                    kbd { "J" }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "ac-center-empty-open-btn",
                                                title: "创建插件（⌘E）",
                                                onclick: move |_| {
                                                    create_plugin_name.set(String::new());
                                                    create_plugin_error.set(None);
                                                    create_plugin_open.set(true);
                                                },
                                                span { class: "ac-center-empty-open-btn-label", "创建插件" }
                                                span { class: "ac-center-empty-open-btn-keys", aria_hidden: "true",
                                                    kbd { "⌘" }
                                                    kbd { "E" }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                        }
                    }
                }
                if show_terminal() {
                    div {
                        style: "{terminal_height_style}",
                        class: "ac-terminal-panel-host",
                        {
                            #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
                            {
                                rsx! {
                                    crate::desktop::terminal::DesktopTerminal {
                                        show_terminal,
                                        on_resize_start: move |event: MouseEvent| {
                                            active_resize.set(Some(ConsoleResizeKind::Terminal));
                                            last_pointer_y.set(Some(event.data.client_coordinates().y));
                                        },
                                        on_toggle_maximize: move |_| {
                                            let current = terminal_height();
                                            if (current - AC_TERMINAL_MAX_HEIGHT_PX).abs() < 0.5 {
                                                let restore = terminal_pre_maximize_height()
                                                    .unwrap_or(AC_TERMINAL_DEFAULT_HEIGHT_PX)
                                                    .clamp(
                                                        AC_TERMINAL_MIN_HEIGHT_PX,
                                                        AC_TERMINAL_MAX_HEIGHT_PX,
                                                    );
                                                terminal_height.set(restore);
                                                terminal_pre_maximize_height.set(None);
                                            } else {
                                                terminal_pre_maximize_height.set(Some(current));
                                                terminal_height.set(AC_TERMINAL_MAX_HEIGHT_PX);
                                            }
                                        },
                                    }
                                }
                            }
                            #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
                            {
                                rsx! {
                                    div { class: "ac-terminal-panel",
                                        div { class: "ac-terminal-toolbar",
                                            span { class: "ac-terminal-toolbar-title", "终端" }
                                            div { class: "ac-terminal-toolbar-actions",
                                                button {
                                                    r#type: "button",
                                                    class: "ac-terminal-toolbar-btn",
                                                    title: "最大化终端",
                                                    aria_label: "最大化终端",
                                                    onclick: move |_| {
                                                        let current = terminal_height();
                                                        if (current - AC_TERMINAL_MAX_HEIGHT_PX).abs() < 0.5 {
                                                            let restore = terminal_pre_maximize_height()
                                                                .unwrap_or(AC_TERMINAL_DEFAULT_HEIGHT_PX)
                                                                .clamp(
                                                                    AC_TERMINAL_MIN_HEIGHT_PX,
                                                                    AC_TERMINAL_MAX_HEIGHT_PX,
                                                                );
                                                            terminal_height.set(restore);
                                                            terminal_pre_maximize_height.set(None);
                                                        } else {
                                                            terminal_pre_maximize_height.set(Some(current));
                                                            terminal_height.set(AC_TERMINAL_MAX_HEIGHT_PX);
                                                        }
                                                    },
                                                    Icon {
                                                        icon: LdChevronsUp,
                                                        width: 14,
                                                        height: 14,
                                                        fill: "currentColor",
                                                    }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "ac-terminal-toolbar-btn",
                                                    title: "关闭终端",
                                                    aria_label: "关闭终端",
                                                    onclick: move |_| show_terminal.set(false),
                                                    Icon {
                                                        icon: LdX,
                                                        width: 14,
                                                        height: 14,
                                                        fill: "currentColor",
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "ac-terminal-unavailable",
                                            p { "Web 端不提供本机终端。" }
                                            p { class: "ac-terminal-unavailable-hint",
                                                "请使用桌面应用打开交互式 shell。"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            aside {
                class: "{chat_aside_class}",
                style: "{chat_width_style}",
                if show_center() && show_chat() {
                    div {
                        class: "ac-chat-resizer",
                        title: "拖动调整对话区域宽度",
                        onmousedown: move |event| {
                            active_resize.set(Some(ConsoleResizeKind::Chat));
                            last_pointer_x.set(Some(event.data.client_coordinates().x));
                        },
                    }
                }
                // 聊天栏可见时标题栏嵌在栏内；隐藏时改由下方 floating 恢复入口渲染（仅 Web）。
                if show_chat() {
                    WebShellTitlebar {
                        show_sidebar,
                        sidebar_width,
                        show_center,
                        show_terminal,
                        show_chat,
                        chat_history_open,
                        sidebar_shutting_down,
                    }
                }
                if let Some(err) = chat_history_error() {
                    div { class: "ac-chat-header",
                        div { class: "ac-chat-history-error", "{err}" }
                    }
                }
                div { class: "ac-chat-body scrollbar-hide",
                    div {
                        class: "{chat_history_panel_class}",
                        style: "{chat_history_width_style}",
                            div {
                                class: "ac-chat-history-resizer",
                                title: "拖动调整会话历史宽度",
                                onmousedown: move |event| {
                                    chat_history_ctx_menu.set(None);
                                    chat_history_drag_width.set(Some(chat_history_width()));
                                    active_resize.set(Some(ConsoleResizeKind::ChatHistory));
                                    last_pointer_x.set(Some(event.data.client_coordinates().x));
                                },
                            }
                            button {
                                r#type: "button",
                                class: "ac-chat-history-create",
                                title: "开启新对话",
                                aria_label: "开启新对话",
                                disabled: chat_busy(),
                                onclick: move |_| {
                                    if chat_busy() {
                                        return;
                                    }
                                    spawn(async move {
                                        match create_conversation().await {
                                            Ok(conversation) => {
                                                let id = conversation.id.clone();
                                                let mut list = conversations();
                                                list.retain(|item| item.id != id);
                                                list.insert(0, conversation);
                                                conversations.set(list);
                                                active_conversation_id.set(id);
                                                chat_messages.set(welcome_chat_messages());
                                                chat_title_editing.set(false);
                                                chat_title_editing_id.set(None);
                                                chat_title_draft.set(String::new());
                                                chat_history_error.set(None);
                                            }
                                            Err(e) => chat_history_error.set(Some(e.to_string())),
                                        }
                                    });
                                },
                                div { class: "ac-chat-history-create-icon",
                                    Icon {
                                        icon: LdPlus,
                                        width: 16,
                                        height: 16,
                                        fill: "currentColor",
                                        class: "ac-chat-header-icon",
                                    }
                                }
                            }
                            for conversation in conversations.read().iter().cloned() {
                                {
                                    let conv_id = conversation.id.clone();
                                    let is_active = conv_id == active_conversation_id();
                                    let display_title = visible_conversation_title(&conversation.title);
                                    let is_editing = chat_title_editing_id()
                                        .as_deref()
                                        .map(|id| id == conv_id)
                                        .unwrap_or(false);
                                    rsx! {
                                        div {
                                            class: if is_editing {
                                                "ac-chat-history-row is-editing"
                                            } else {
                                                "ac-chat-history-row"
                                            },
                                            if is_editing {
                                                div { class: "ac-chat-history-edit-wrap",
                                                    input {
                                                        class: "ac-chat-history-title-input",
                                                        value: "{chat_title_draft()}",
                                                        maxlength: "48",
                                                        disabled: chat_title_saving(),
                                                        oninput: move |e| chat_title_draft.set(e.value()),
                                                        onkeydown: {
                                                            let conv_id = conv_id.clone();
                                                            move |e: KeyboardEvent| {
                                                                if e.key() == Key::Enter {
                                                                    e.prevent_default();
                                                                    let id = conv_id.clone();
                                                                    let title = chat_title_draft();
                                                                    chat_title_saving.set(true);
                                                                    spawn(async move {
                                                                        match update_conversation_title(id, title).await {
                                                                            Ok(updated) => {
                                                                                let mut list = conversations();
                                                                                if let Some(item) = list.iter_mut().find(|item| item.id == updated.id) {
                                                                                    *item = updated;
                                                                                }
                                                                                conversations.set(list);
                                                                                chat_title_editing.set(false);
                                                                                chat_title_editing_id.set(None);
                                                                                chat_title_draft.set(String::new());
                                                                                chat_history_error.set(None);
                                                                            }
                                                                            Err(e) => chat_history_error.set(Some(e.to_string())),
                                                                        }
                                                                        chat_title_saving.set(false);
                                                                    });
                                                                } else if e.key() == Key::Escape {
                                                                    chat_title_editing.set(false);
                                                                    chat_title_editing_id.set(None);
                                                                    chat_title_draft.set(String::new());
                                                                }
                                                            }
                                                        }
                                                    }
                                                    div { class: "ac-chat-history-edit-actions",
                                                        button {
                                                            r#type: "button",
                                                            class: "ac-chat-history-edit-action",
                                                            title: "保存标题",
                                                            aria_label: "保存标题",
                                                            disabled: chat_title_saving(),
                                                            onclick: {
                                                                let conv_id = conv_id.clone();
                                                                move |_| {
                                                                    let id = conv_id.clone();
                                                                    let title = chat_title_draft();
                                                                    chat_title_saving.set(true);
                                                                    spawn(async move {
                                                                        match update_conversation_title(id, title).await {
                                                                            Ok(updated) => {
                                                                                let mut list = conversations();
                                                                                if let Some(item) = list.iter_mut().find(|item| item.id == updated.id) {
                                                                                    *item = updated;
                                                                                }
                                                                                conversations.set(list);
                                                                                chat_title_editing.set(false);
                                                                                chat_title_editing_id.set(None);
                                                                                chat_title_draft.set(String::new());
                                                                                chat_history_error.set(None);
                                                                            }
                                                                            Err(e) => chat_history_error.set(Some(e.to_string())),
                                                                        }
                                                                        chat_title_saving.set(false);
                                                                    });
                                                                }
                                                            },
                                                            Icon {
                                                                icon: LdCheck,
                                                                width: 13,
                                                                height: 13,
                                                                fill: "currentColor",
                                                                class: "ac-chat-header-icon",
                                                            }
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            class: "ac-chat-history-edit-action",
                                                            title: "取消编辑",
                                                            aria_label: "取消编辑标题",
                                                            disabled: chat_title_saving(),
                                                            onclick: move |_| {
                                                                chat_title_editing.set(false);
                                                                chat_title_editing_id.set(None);
                                                                chat_title_draft.set(String::new());
                                                            },
                                                            Icon {
                                                                icon: LdX,
                                                                width: 13,
                                                                height: 13,
                                                                fill: "currentColor",
                                                                class: "ac-chat-header-icon",
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                div {
                                                    class: if is_active {
                                                        "ac-chat-history-item is-active"
                                                    } else {
                                                        "ac-chat-history-item"
                                                    },
                                                    title: if let Some(title) = display_title.clone() {
                                                        title
                                                    } else if conversation.last_message_preview.trim().is_empty() {
                                                        "暂无消息".to_string()
                                                    } else {
                                                        conversation.last_message_preview.clone()
                                                    },
                                                    onclick: {
                                                        let conv_id = conv_id.clone();
                                                        move |_| {
                                                            chat_history_ctx_menu.set(None);
                                                            if chat_busy() {
                                                                return;
                                                            }
                                                            let id = conv_id.clone();
                                                            spawn(async move {
                                                                match load_conversation_messages(id.clone(), 200).await {
                                                                    Ok(messages) => {
                                                                        active_conversation_id.set(id);
                                                                        chat_messages.set(stored_messages_to_ui(messages));
                                                                        chat_title_editing.set(false);
                                                                        chat_title_editing_id.set(None);
                                                                        chat_title_draft.set(String::new());
                                                                        chat_history_error.set(None);
                                                                        chat_scroll_bottom_request += 1;
                                                                    }
                                                                    Err(e) => chat_history_error.set(Some(e.to_string())),
                                                                }
                                                            });
                                                        }
                                                    },
                                                    oncontextmenu: {
                                                        let conv_id = conv_id.clone();
                                                        move |evt| {
                                                            evt.prevent_default();
                                                            evt.stop_propagation();
                                                            let coords = evt.data.client_coordinates();
                                                            chat_history_ctx_menu.set(Some(ChatHistoryCtxMenu {
                                                                conversation_id: conv_id.clone(),
                                                                x: coords.x,
                                                                y: coords.y,
                                                            }));
                                                        }
                                                    },
                                                    ondblclick: {
                                                        let conv_id = conv_id.clone();
                                                        let display_title = display_title.clone();
                                                        move |evt| {
                                                            evt.stop_propagation();
                                                            if chat_busy() {
                                                                return;
                                                            }
                                                            // 改名需要可读宽度：过窄时加宽到默认，标题仍用连续裁切布局。
                                                            if chat_history_width() < AC_CHAT_HISTORY_MIN_WIDTH_PX {
                                                                chat_history_width.set(AC_CHAT_HISTORY_DEFAULT_WIDTH_PX);
                                                            }
                                                            chat_history_open.set(true);
                                                            chat_title_editing.set(true);
                                                            chat_title_editing_id.set(Some(conv_id.clone()));
                                                            chat_title_draft.set(display_title.clone().unwrap_or_default());
                                                        }
                                                    },
                                                    // 图标 + 标题始终同排；窄栏时标题被 overflow 裁切渐进露出
                                                    div {
                                                        class: "ac-chat-history-item-rail-icon",
                                                        aria_hidden: "true",
                                                        Icon {
                                                            icon: LdMessageCircle,
                                                            width: 14,
                                                            height: 14,
                                                            fill: "currentColor",
                                                            class: "ac-chat-header-icon",
                                                        }
                                                    }
                                                    div { class: "ac-chat-history-item-body",
                                                        div { class: "ac-chat-history-item-preview-row",
                                                            div {
                                                                class: "ac-chat-history-item-preview",
                                                                if conversation.last_message_preview.trim().is_empty() {
                                                                    if let Some(title) = display_title.clone() {
                                                                        "{title}"
                                                                    } else {
                                                                        "暂无消息"
                                                                    }
                                                                } else {
                                                                    if let Some(title) = display_title.clone() {
                                                                        "{title}"
                                                                    } else {
                                                                        "{conversation.last_message_preview}"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                    }
                    div {
                        id: AC_CHAT_THREAD_DOM_ID,
                        class: "ac-chat-thread scrollbar-hide",
                        // 鼠标滚轮上滑：立刻解除“贴底跟随”，避免流式 token 把视图抢回底部。
                        onwheel: move |evt| {
                            if evt.data().delta().strip_units().y < 0.0 {
                                chat_should_stick_bottom.set(false);
                            }
                        },
                        // 触屏拖动：一律先解除贴底；下次滚到底由 onscroll 恢复。
                        ontouchmove: move |_| {
                            chat_should_stick_bottom.set(false);
                        },
                        // 仅在“AI 仍在回答”且用户主动滚回底部附近时，重新启用流式跟随；
                        // AI 没在回答时一律不主动开启吸底，符合「默认不吸底」语义。
                        onscroll: move |_| {
                            if chat_busy() && chat_thread_is_near_bottom() {
                                chat_should_stick_bottom.set(true);
                            }
                        },
                        for (msg_idx, msg) in chat_messages.read().iter().cloned().enumerate() {
                            match msg {
                                UiChatMessage::User {
                                    content,
                                    attachments,
                                    ..
                                } => rsx! {
                                    div { class: "ac-chat-message ac-chat-message-user",
                                        div { class: "ac-chat-bubble ac-chat-bubble-user",
                                            if !attachments.is_empty() {
                                                div { class: "ac-chat-bubble-attach-row",
                                                    for att in attachments {
                                                        if let Some(url) = att.preview_url.clone() {
                                                            img {
                                                                key: "{att.id}",
                                                                class: "ac-chat-bubble-attach-img",
                                                                src: "{url}",
                                                                alt: "{att.name}",
                                                                title: "{att.name}",
                                                            }
                                                        } else {
                                                            span {
                                                                key: "{att.id}",
                                                                class: "ac-chat-bubble-attach-file",
                                                                title: "{att.name}",
                                                                "📎 {att.name}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if !content.is_empty() {
                                                span { "{content}" }
                                            }
                                        }
                                    }
                                },
                                UiChatMessage::Assistant {
                                    content,
                                    agent_run_id,
                                    thinking,
                                    ..
                                } => {
                                    let answer_ready = !content.trim().is_empty();
                                    let show_thinking = thinking.has_visible_content()
                                        || (chat_busy() && !answer_ready);
                                    let thinking_busy = matches!(
                                        thinking.status,
                                        ThinkingStatus::Running
                                    ) && !answer_ready;
                                    let show_bubble = answer_ready;
                                    rsx! {
                                    div { class: "ac-chat-message ac-chat-message-ai",
                                        ChatThinkingHydrator {
                                            msg_index: msg_idx,
                                            chat_messages,
                                        }
                                        div { class: "ac-chat-ai-stack",
                                            if show_thinking {
                                                ChatThinkingPanel {
                                                    msg_index: msg_idx,
                                                    chat_messages,
                                                    thinking: thinking.clone(),
                                                    busy: thinking_busy,
                                                    on_open_trace_file: move |action: TraceFileOpen| {
                                                        match action {
                                                            TraceFileOpen::File { path } => {
                                                                if !super::files::fs_available() {
                                                                    toast.error(
                                                                        "Web 端无法打开本机文件，请使用桌面版。",
                                                                    );
                                                                    return;
                                                                }
                                                                let resolved =
                                                                    reveal_and_select_workspace_file(
                                                                        &path,
                                                                        show_sidebar,
                                                                        sidebar_shutting_down,
                                                                        show_center,
                                                                        center_shutting_down,
                                                                        active_tab,
                                                                        fs_section_open,
                                                                        fs_root_path,
                                                                        fs_expanded,
                                                                        fs_children_cache,
                                                                        fs_selected_path,
                                                                        fs_save_notice,
                                                                    );
                                                                if resolved.is_empty() {
                                                                    return;
                                                                }
                                                                center_tabs.with_mut(|tabs| {
                                                                    active_center_id.with_mut(|active| {
                                                                        center_open_or_focus(
                                                                            tabs,
                                                                            active,
                                                                            CenterTab::file(resolved),
                                                                        );
                                                                    });
                                                                });
                                                            }
                                                            TraceFileOpen::Diff {
                                                                path,
                                                                tool_id,
                                                                old_text,
                                                                new_text,
                                                            } => {
                                                                if !super::files::fs_available() {
                                                                    toast.error(
                                                                        "Web 端无法打开本机文件，请使用桌面版。",
                                                                    );
                                                                    return;
                                                                }
                                                                let resolved =
                                                                    reveal_and_select_workspace_file(
                                                                        &path,
                                                                        show_sidebar,
                                                                        sidebar_shutting_down,
                                                                        show_center,
                                                                        center_shutting_down,
                                                                        active_tab,
                                                                        fs_section_open,
                                                                        fs_root_path,
                                                                        fs_expanded,
                                                                        fs_children_cache,
                                                                        fs_selected_path,
                                                                        fs_save_notice,
                                                                    );
                                                                let path = if resolved.is_empty() {
                                                                    path
                                                                } else {
                                                                    resolved
                                                                };
                                                                if path.is_empty() {
                                                                    return;
                                                                }
                                                                // 无 old/new 时退化为普通打开（带行号）。
                                                                if old_text.is_empty() && new_text.is_empty() {
                                                                    center_tabs.with_mut(|tabs| {
                                                                        active_center_id.with_mut(|active| {
                                                                            center_open_or_focus(
                                                                                tabs,
                                                                                active,
                                                                                CenterTab::file(path),
                                                                            );
                                                                        });
                                                                    });
                                                                    return;
                                                                }
                                                                center_tabs.with_mut(|tabs| {
                                                                    active_center_id.with_mut(|active| {
                                                                        center_open_or_focus(
                                                                            tabs,
                                                                            active,
                                                                            CenterTab::file_diff(
                                                                                tool_id,
                                                                                path,
                                                                                old_text,
                                                                                new_text,
                                                                            ),
                                                                        );
                                                                    });
                                                                });
                                                            }
                                                        }
                                                    },
                                                }
                                            }
                                            if show_bubble {
                                                div { class: "ac-chat-bubble ac-chat-bubble-ai is-visible",
                                                    ChatMarkdownBody {
                                                        content: content.clone(),
                                                    }
                                                }
                                            }
                                            if show_bubble {
                                            div { class: "ac-chat-ai-actions",
                                                button {
                                                    r#type: "button",
                                                    class: "ac-chat-ai-action-btn",
                                                    title: "复制",
                                                    aria_label: "复制助手回复",
                                                    onclick: {
                                                        let content = content.clone();
                                                        move |_| copy_text_to_clipboard(content.clone())
                                                    },
                                                    Icon {
                                                        icon: LdCopy,
                                                        width: 14,
                                                        height: 14,
                                                        fill: "currentColor",
                                                        class: "ac-chat-ai-action-icon",
                                                    }
                                                }
                                                if developer_mode() {
                                                    if let Some(run_id) = agent_run_id {
                                                        button {
                                                            r#type: "button",
                                                            class: "ac-chat-ai-action-btn",
                                                            title: "查看详情",
                                                            aria_label: "查看助手回复详情",
                                                            onclick: move |_| {
                                                                agent_detail_loading.set(true);
                                                                agent_detail_error.set(None);
                                                                selected_agent_run_detail.set(None);
                                                                spawn(async move {
                                                                    match load_agent_run_detail(run_id).await {
                                                                        Ok(detail) => {
                                                                            selected_agent_run_detail.set(Some(detail));
                                                                            agent_detail_error.set(None);
                                                                        }
                                                                        Err(e) => {
                                                                            agent_detail_error.set(Some(e.to_string()));
                                                                        }
                                                                    }
                                                                    agent_detail_loading.set(false);
                                                                });
                                                            },
                                                            Icon {
                                                                icon: LdEye,
                                                                width: 14,
                                                                height: 14,
                                                                fill: "currentColor",
                                                                class: "ac-chat-ai-action-icon",
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            }
                                        }
                                    }
                                    }
                                },
                            }
                        }
                    }
                }
                div { class: "ac-chat-input-wrap",
                    div { class: "ac-chat-input-shell",
                        div { class: "ac-chat-composer-stack",
                            if !chat_attachments().is_empty() {
                                div { class: "ac-chat-attach-preview-row",
                                    for att in chat_attachments() {
                                        {
                                            let remove_id = att.id;
                                            let is_image = att.preview_url.is_some();
                                            rsx! {
                                                div {
                                                    key: "{att.id}",
                                                    class: if is_image {
                                                        "ac-chat-attach-thumb"
                                                    } else {
                                                        "ac-chat-attach-thumb ac-chat-attach-thumb--file"
                                                    },
                                                    title: "{att.name}",
                                                    if let Some(url) = att.preview_url.clone() {
                                                        img {
                                                            class: "ac-chat-attach-thumb-img",
                                                            src: "{url}",
                                                            alt: "{att.name}",
                                                        }
                                                    } else {
                                                        span { class: "ac-chat-attach-thumb-name", "{att.name}" }
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "ac-chat-attach-thumb-remove",
                                                        title: "移除附件",
                                                        aria_label: "移除附件",
                                                        onclick: move |_| {
                                                            chat_attachments.with_mut(|list| {
                                                                list.retain(|a| a.id != remove_id);
                                                            });
                                                        },
                                                        Icon {
                                                            icon: LdX,
                                                            width: 12,
                                                            height: 12,
                                                            fill: "currentColor",
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            ChatComposerField {
                                chat_draft,
                                input_epoch: chat_input_epoch(),
                                disabled: chat_busy(),
                                on_enter_send: enter_send_chat,
                                on_paste_images: on_composer_paste,
                                rows: 4,
                            }
                            div { class: "ac-chat-composer-toolbar",
                                button {
                                    r#type: "button",
                                    class: "ac-chat-toolbar-new-conv ac-chat-toolbar-pill",
                                    title: "新建对话",
                                    aria_label: "新建对话",
                                    disabled: chat_busy(),
                                    onclick: move |_| {
                                        if chat_busy() {
                                            return;
                                        }
                                        chat_model_sheet_open.set(false);
                                        spawn(async move {
                                            match create_conversation().await {
                                                Ok(conversation) => {
                                                    let id = conversation.id.clone();
                                                    let mut list = conversations();
                                                    list.retain(|item| item.id != id);
                                                    list.insert(0, conversation);
                                                    conversations.set(list);
                                                    active_conversation_id.set(id);
                                                    chat_messages.set(welcome_chat_messages());
                                                    chat_title_editing.set(false);
                                                    chat_title_editing_id.set(None);
                                                    chat_title_draft.set(String::new());
                                                    chat_history_error.set(None);
                                                }
                                                Err(e) => chat_history_error.set(Some(e.to_string())),
                                            }
                                        });
                                    },
                                    Icon {
                                        icon: LdPlus,
                                        width: 15,
                                        height: 15,
                                        fill: "currentColor",
                                        class: "ac-chat-toolbar-new-conv-icon",
                                    }
                                }
                                div { class: "ac-chat-model-wrap",
                                    button {
                                        r#type: "button",
                                        class: if chat_model_sheet_open() {
                                            "ac-chat-model-trigger ac-chat-toolbar-pill is-open"
                                        } else {
                                            "ac-chat-model-trigger ac-chat-toolbar-pill"
                                        },
                                        aria_expanded: chat_model_sheet_open(),
                                        aria_haspopup: "menu",
                                        onclick: move |_| {
                                            chat_model_sheet_open.toggle();
                                        },
                                        span { class: "ac-chat-mode-trigger-label",
                                            // 触发器主标题：优先用列表里命中行的 id（大写化展示），
                                            // 没命中则保留当前 chat_model 原值（已自动回落，理论上
                                            // 不会到这里）。display_name 不放进 trigger，避免按钮太宽。
                                            {
                                                let cur = chat_model();
                                                let models = chat_models.read();
                                                let label = models
                                                    .iter()
                                                    .find(|m| m.id == cur)
                                                    .map(|m| m.id.to_uppercase())
                                                    .unwrap_or_else(|| cur.to_uppercase());
                                                label
                                            }
                                        }
                                        if chat_model_sheet_open() {
                                            Icon {
                                                icon: LdChevronDown,
                                                width: 14,
                                                height: 14,
                                                fill: "currentColor",
                                                class: "ac-chat-mode-trigger-chevron",
                                            }
                                        } else {
                                            Icon {
                                                icon: LdChevronUp,
                                                width: 14,
                                                height: 14,
                                                fill: "currentColor",
                                                class: "ac-chat-mode-trigger-chevron",
                                            }
                                        }
                                    }
                                    if chat_model_sheet_open() {
                                        // 全屏透明 backdrop：点击除模型列表以外的任意区域即关闭弹窗。
                                        // z-index 低于 .ac-chat-mode-sheet（5001），高于普通页面元素，
                                        // 与顶栏设置菜单 (.ac-web-titlebar-menu-backdrop) 完全同模式。
                                        div {
                                            class: "ac-chat-mode-sheet-backdrop",
                                            onclick: move |_| chat_model_sheet_open.set(false),
                                        }
                                        div { class: "ac-chat-mode-sheet", role: "menu",
                                            // 模型列表。每行展示 id（大写化）+ 可选的 display_name 副标题。
                                            // .read().clone() 转 owned，避免在 rsx for / closure 里持
                                            // 有 Signal 的 Ref 借用。
                                            if let Some(hint) = chat_models_load_hint() {
                                                span {
                                                    class: "ac-chat-mode-sheet__sub",
                                                    style: "padding: 0.35rem 0.72rem 0.15rem;",
                                                    "{hint}"
                                                }
                                            }
                                            for m in chat_models.read().clone() {
                                                {
                                                    let id = m.id.clone();
                                                    let id_for_click = id.clone();
                                                    let id_for_class = id.clone();
                                                    let id_for_check = id.clone();
                                                    let title = id.to_uppercase();
                                                    let sub = m.display_name.clone().unwrap_or_default();
                                                    rsx! {
                                                        button {
                                                            key: "{id}",
                                                            r#type: "button",
                                                            role: "menuitem",
                                                            class: if chat_model() == id_for_class {
                                                                "ac-chat-mode-sheet__opt is-active"
                                                            } else {
                                                                "ac-chat-mode-sheet__opt"
                                                            },
                                                            onclick: move |_| {
                                                                persist_chat_model(&id_for_click);
                                                                chat_model.set(id_for_click.clone());
                                                                chat_model_sheet_open.set(false);
                                                            },
                                                            span { class: "ac-chat-mode-sheet__title", "{title}" }
                                                            if !sub.is_empty() {
                                                                span { class: "ac-chat-mode-sheet__sub", "{sub}" }
                                                            }
                                                            if chat_model() == id_for_check {
                                                                Icon {
                                                                    icon: LdCheck,
                                                                    width: 18,
                                                                    height: 18,
                                                                    fill: "currentColor",
                                                                    class: "ac-chat-mode-sheet__check",
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                label { class: "ac-chat-upload-btn ac-chat-toolbar-pill",
                                    input {
                                        r#type: "file",
                                        class: "ac-chat-file-input",
                                        accept: "image/*,*/*",
                                        multiple: true,
                                        onchange: move |e: FormEvent| {
                                            let files: Vec<_> = e.data().files().into_iter().collect();
                                            if files.is_empty() {
                                                return;
                                            }
                                            spawn(async move {
                                                for file in files {
                                                    let id = chat_attachment_seq();
                                                    chat_attachment_seq.set(id + 1);
                                                    if let Some(att) =
                                                        attachment_from_file_data(id, file).await
                                                    {
                                                        chat_attachments.with_mut(|list| list.push(att));
                                                    }
                                                }
                                            });
                                        },
                                    }
                                    Icon {
                                        icon: LdPaperclip,
                                        width: 15,
                                        height: 15,
                                        fill: "currentColor",
                                        class: "ac-chat-upload-icon",
                                    }
                                }
                            }
                        }
                        button {
                            r#type: "button",
                            class: if chat_busy() {
                                "ac-chat-send-btn ac-chat-send-btn--pause"
                            } else if !chat_draft().trim().is_empty()
                                || !chat_attachments().is_empty()
                            {
                                "ac-chat-send-btn ac-chat-send-btn--ready"
                            } else {
                                "ac-chat-send-btn"
                            },
                            disabled: cfg!(not(target_arch = "wasm32")) && chat_busy(),
                            onclick: send_or_pause_chat,
                            if chat_busy() {
                                Icon {
                                    icon: BsStopFill,
                                    width: 16,
                                    height: 16,
                                    fill: "currentColor",
                                    class: "ac-chat-send-icon",
                                }
                            } else {
                                Icon {
                                    icon: LdArrowUp,
                                    width: 16,
                                    height: 16,
                                    fill: "currentColor",
                                    class: "ac-chat-send-icon",
                                }
                            }
                        }
                    }
                }
            }

            if let Some(msg) = agent_detail_copy_notice() {
                div { class: "ac-settings-toast", "{msg}" }
            }

            if agent_detail_loading() || agent_detail_error().is_some() || selected_agent_run_detail().is_some() {
                div { class: "ac-agent-detail-backdrop",
                    div { class: "ac-agent-detail-modal",
                        div { class: "ac-agent-detail-head",
                            div {
                                h2 { class: "ac-agent-detail-title", "Agent 运行详情" }
                                p { class: "ac-agent-detail-subtitle", "步骤、耗时、模型输出与工具完整返回内容" }
                            }
                            button {
                                r#type: "button",
                                class: "ac-agent-detail-close",
                                title: "关闭",
                                aria_label: "关闭详情",
                                onclick: move |_| {
                                    selected_agent_run_detail.set(None);
                                    agent_detail_error.set(None);
                                    agent_detail_loading.set(false);
                                    agent_detail_copy_notice.set(None);
                                },
                                Icon {
                                    icon: LdX,
                                    width: 17,
                                    height: 17,
                                    fill: "currentColor",
                                    class: "ac-chat-ai-action-icon",
                                }
                            }
                        }
                        if agent_detail_loading() {
                            div { class: "ac-agent-detail-loading-skeleton",
                                div { class: "ac-agent-detail-skeleton-meta",
                                    for _ in 0..3 {
                                        div { class: "ac-agent-detail-skeleton-chip ac-skeleton-base" }
                                    }
                                }
                                div { class: "ac-agent-detail-skeleton-body",
                                    for i in 0..8 {
                                        div {
                                            class: if i % 3 == 2 {
                                                "ac-agent-detail-skeleton-line ac-agent-detail-skeleton-line--short ac-skeleton-base"
                                            } else {
                                                "ac-agent-detail-skeleton-line ac-skeleton-base"
                                            }
                                        }
                                    }
                                }
                            }
                        } else if let Some(err) = agent_detail_error() {
                            div { class: "ac-agent-detail-error", "{err}" }
                        } else if let Some(detail) = selected_agent_run_detail() {
                            div { class: "ac-agent-detail-meta",
                                span { "模型: {detail.model}" }
                                span { "状态: {detail.status}" }
                                span { "总耗时: {format_duration_ms(detail.total_duration_ms)}" }
                                button {
                                    r#type: "button",
                                    class: "ac-agent-detail-copy",
                                    title: "复制全文",
                                    aria_label: "复制全文",
                                    onclick: {
                                        let detail = detail.clone();
                                        move |_| {
                                            copy_text_to_clipboard(build_agent_run_full_text(&detail));
                                            agent_detail_copy_notice
                                                .set(Some("已复制完整运行详情".to_string()));
                                            spawn(async move {
                                                ui_sleep_ms(1200).await;
                                                agent_detail_copy_notice.set(None);
                                            });
                                        }
                                    },
                                    Icon {
                                        icon: LdCopy,
                                        width: 12,
                                        height: 12,
                                        fill: "currentColor",
                                        class: "ac-chat-ai-action-icon",
                                    }
                                    "复制全文"
                                }
                            }
                            div { class: "ac-agent-detail-body scrollbar-hide",
                                for step in detail.steps.iter().cloned() {
                                    section { class: "ac-agent-detail-step",
                                        div { class: "ac-agent-detail-step-head",
                                            strong { "步骤 #{step.index}" }
                                            span { "{step.phase} · {format_duration_ms(step.duration_ms)}" }
                                        }
                                        if !step.model_output.trim().is_empty() {
                                            div { class: "ac-agent-detail-block-title", "模型输出" }
                                            pre { class: "ac-agent-detail-pre", "{step.model_output}" }
                                        }
                                        for tool in step.tools.iter().cloned() {
                                            div { class: "ac-agent-detail-tool",
                                                div { class: "ac-agent-detail-tool-head",
                                                    strong { "{tool.name}" }
                                                    span { "{format_duration_ms(tool.duration_ms)}" }
                                                }
                                                div { class: "ac-agent-detail-block-title", "参数" }
                                                pre { class: "ac-agent-detail-pre", "{tool.args}" }
                                                div { class: "ac-agent-detail-block-title", "完整返回" }
                                                pre { class: "ac-agent-detail-pre", "{tool.result}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }



            if create_plugin_open() {
                div {
                    class: "ac-api-modal-backdrop ac-delete-role-modal-backdrop",
                    onclick: move |_| {
                        create_plugin_open.set(false);
                        create_plugin_error.set(None);
                    },
                    div {
                        class: "ac-api-modal",
                        onclick: move |evt| evt.stop_propagation(),
                        h2 { class: "ac-api-modal-title", "创建插件" }
                        p { class: "ac-api-modal-desc",
                            "将在工作区 extension/ 下新建插件目录，并写入 README 与 manifest 脚手架。"
                        }
                        div { class: "ac-api-modal-field",
                            input {
                                class: "ac-api-modal-input",
                                r#type: "text",
                                placeholder: "插件名称，例如 my-plugin",
                                value: "{create_plugin_name()}",
                                oninput: move |e| {
                                    create_plugin_name.set(e.value());
                                    create_plugin_error.set(None);
                                },
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter {
                                        let name = create_plugin_name();
                                        match super::files::fs_scaffold_extension_plugin(&name) {
                                            Ok(path) => {
                                                create_plugin_open.set(false);
                                                create_plugin_error.set(None);
                                                apply_created_extension_plugin(
                                                    path,
                                                    fs_root_path,
                                                    fs_expanded,
                                                    fs_children_cache,
                                                    fs_selected_path,
                                                    toast,
                                                    active_tab,
                                                    fs_section_open,
                                                );
                                            }
                                            Err(err) => create_plugin_error.set(Some(err)),
                                        }
                                    }
                                },
                            }
                            p { class: "ac-api-modal-help", "留空则使用默认名 my-plugin。" }
                        }
                        if let Some(err) = create_plugin_error() {
                            p { class: "ac-api-modal-desc", style: "color: var(--danger-color, #b91c1c);", "{err}" }
                        }
                        div { class: "ac-api-modal-actions",
                            button {
                                r#type: "button",
                                class: "ac-role-btn ac-role-btn-secondary",
                                onclick: move |_| {
                                    create_plugin_open.set(false);
                                    create_plugin_error.set(None);
                                },
                                "取消"
                            }
                            button {
                                r#type: "button",
                                class: "ac-role-btn ac-role-btn-primary",
                                onclick: move |_| {
                                    let name = create_plugin_name();
                                    match super::files::fs_scaffold_extension_plugin(&name) {
                                        Ok(path) => {
                                            create_plugin_open.set(false);
                                            create_plugin_error.set(None);
                                            apply_created_extension_plugin(
                                                path,
                                                fs_root_path,
                                                fs_expanded,
                                                fs_children_cache,
                                                fs_selected_path,
                                                toast,
                                                active_tab,
                                                fs_section_open,
                                            );
                                        }
                                        Err(err) => create_plugin_error.set(Some(err)),
                                    }
                                },
                                "创建"
                            }
                        }
                    }
                }
            }

            if create_app_open() {
                div {
                    class: "ac-api-modal-backdrop ac-delete-role-modal-backdrop",
                    onclick: move |_| {
                        create_app_open.set(false);
                        create_app_error.set(None);
                    },
                    div {
                        class: "ac-api-modal",
                        onclick: move |evt| evt.stop_propagation(),
                        h2 { class: "ac-api-modal-title", "创建应用" }
                        p { class: "ac-api-modal-desc",
                            "将在工作区 application/ 下新建应用目录，创建后自动打开为当前项目。"
                        }
                        div { class: "ac-api-modal-field",
                            input {
                                class: "ac-api-modal-input",
                                r#type: "text",
                                placeholder: "应用名称，例如 my-app",
                                value: "{create_app_name()}",
                                oninput: move |e| {
                                    create_app_name.set(e.value());
                                    create_app_error.set(None);
                                },
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter {
                                        let name = create_app_name();
                                        match super::files::fs_scaffold_local_application(&name) {
                                            Ok(path) => {
                                                create_app_open.set(false);
                                                create_app_error.set(None);
                                                apply_created_local_application(
                                                    path,
                                                    fs_root_path,
                                                    fs_expanded,
                                                    fs_children_cache,
                                                    fs_selected_path,
                                                    fs_create_mode,
                                                    fs_create_name,
                                                    fs_create_parent,
                                                    fs_notice,
                                                    recent_project_dirs,
                                                    active_tab,
                                                    center_tabs,
                                                    active_center_id,
                                                    home_logo_menu_open,
                                                    fs_section_open,
                                                    toast,
                                                    plugin_refresh_tick,
                                                );
                                            }
                                            Err(err) => create_app_error.set(Some(err)),
                                        }
                                    }
                                },
                            }
                        }
                        if let Some(err) = create_app_error() {
                            p { class: "ac-api-modal-desc", style: "color: var(--danger-color, #b91c1c);", "{err}" }
                        }
                        div { class: "ac-api-modal-actions",
                            button {
                                r#type: "button",
                                class: "ac-role-btn ac-role-btn-secondary",
                                onclick: move |_| {
                                    create_app_open.set(false);
                                    create_app_error.set(None);
                                },
                                "取消"
                            }
                            button {
                                r#type: "button",
                                class: "ac-role-btn ac-role-btn-primary",
                                onclick: move |_| {
                                    let name = create_app_name();
                                    match super::files::fs_scaffold_local_application(&name) {
                                        Ok(path) => {
                                            create_app_open.set(false);
                                            create_app_error.set(None);
                                            apply_created_local_application(
                                                path,
                                                fs_root_path,
                                                fs_expanded,
                                                fs_children_cache,
                                                fs_selected_path,
                                                fs_create_mode,
                                                fs_create_name,
                                                fs_create_parent,
                                                fs_notice,
                                                recent_project_dirs,
                                                active_tab,
                                                center_tabs,
                                                active_center_id,
                                                home_logo_menu_open,
                                                fs_section_open,
                                                toast,
                                                plugin_refresh_tick,
                                            );
                                        }
                                        Err(err) => create_app_error.set(Some(err)),
                                    }
                                },
                                "创建"
                            }
                        }
                    }
                }
            }

            if let Some(ctx) = chat_history_ctx_menu() {
                {
                    let target_id = ctx.conversation_id.clone();
                    // 菜单锚在指针左侧，避免贴右缘被裁切（right 边对齐 clientX）。
                    let menu_style = format!(
                        "right: calc(100vw - {:.0}px); top: {:.0}px; left: auto;",
                        ctx.x, ctx.y
                    );
                    rsx! {
                        div {
                            class: "ac-chat-history-ctx-backdrop",
                            onclick: move |_| chat_history_ctx_menu.set(None),
                            oncontextmenu: move |evt| {
                                evt.prevent_default();
                                chat_history_ctx_menu.set(None);
                            },
                        }
                        div {
                            class: "ac-chat-history-ctx-menu",
                            role: "menu",
                            style: "{menu_style}",
                            onclick: move |evt| evt.stop_propagation(),
                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item",
                                role: "menuitem",
                                disabled: chat_busy(),
                                onclick: {
                                    let target_id = target_id.clone();
                                    move |_| {
                                        chat_history_ctx_menu.set(None);
                                        if chat_busy() {
                                            return;
                                        }
                                        let draft = conversations()
                                            .iter()
                                            .find(|item| item.id == target_id)
                                            .and_then(|item| visible_conversation_title(&item.title))
                                            .unwrap_or_default();
                                        if chat_history_width() < AC_CHAT_HISTORY_MIN_WIDTH_PX {
                                            chat_history_width.set(AC_CHAT_HISTORY_DEFAULT_WIDTH_PX);
                                        }
                                        chat_history_open.set(true);
                                        chat_title_editing.set(true);
                                        chat_title_editing_id.set(Some(target_id.clone()));
                                        chat_title_draft.set(draft);
                                    }
                                },
                                Icon {
                                    icon: LdPencil,
                                    width: 14,
                                    height: 14,
                                    fill: "currentColor",
                                    class: "ac-chat-header-icon",
                                }
                                "编辑名称"
                            }
                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item is-danger",
                                role: "menuitem",
                                disabled: chat_busy(),
                                onclick: move |_| {
                                    chat_history_ctx_menu.set(None);
                                    if chat_busy() {
                                        return;
                                    }
                                    spawn_delete_conversation_and_refresh(
                                        target_id.clone(),
                                        conversations,
                                        active_conversation_id,
                                        chat_messages,
                                        chat_scroll_bottom_request,
                                        chat_title_editing,
                                        chat_title_editing_id,
                                        chat_title_draft,
                                        chat_history_error,
                                    );
                                },
                                Icon {
                                    icon: LdTrash2,
                                    width: 14,
                                    height: 14,
                                    fill: "currentColor",
                                    class: "ac-chat-header-icon",
                                }
                                "删除对话"
                            }
                        }
                    }
                }
            }

            if let Some((pending_rid, pending_name)) = delete_role_confirm() {
                div {
                    class: "ac-api-modal-backdrop ac-delete-role-modal-backdrop",
                    onclick: move |_| delete_role_confirm.set(None),
                    div {
                        class: "ac-api-modal",
                        onclick: move |evt| evt.stop_propagation(),
                        h2 { class: "ac-api-modal-title", "删除角色" }
                        p { class: "ac-api-modal-desc",
                            "确定删除角色「"
                            "{pending_name}"
                            "」？此操作不可恢复。"
                        }
                        div { class: "ac-api-modal-actions",
                            button {
                                r#type: "button",
                                class: "ac-role-btn ac-role-btn-secondary",
                                onclick: move |_| delete_role_confirm.set(None),
                                "取消"
                            }
                            button {
                                r#type: "button",
                                class: "ac-role-btn ac-role-btn-primary",
                                onclick: move |_| {
                                    delete_role_confirm.set(None);
                                    role_loading.set(true);
                                    spawn(async move {
                                        match delete_role(pending_rid).await {
                                            Ok(()) => {
                                                match load_roles().await {
                                                    Ok(roles) => {
                                                        role_list.set(roles.clone());
                                                        role_error.set(None);
                                                        if let Some(eid) = editing_role_id() {
                                                            if !roles.iter().any(|r| r.id == eid) {
                                                                if let Some(r) = roles
                                                                    .iter()
                                                                    .find(|r| r.is_active)
                                                                    .or(roles.first())
                                                                {
                                                                    editing_role_id.set(Some(r.id));
                                                                    editing_role_name.set(r.name.clone());
                                                                    persona_prompt.set(r.system_prompt.clone());
                                                                    persona_textarea_epoch.with_mut(|x| *x += 1);
                                                                } else {
                                                                    editing_role_id.set(None);
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(e) => role_error.set(Some(e.to_string())),
                                                }
                                            }
                                            Err(e) => role_error.set(Some(e.to_string())),
                                        }
                                        role_loading.set(false);
                                    });
                                },
                                "删除"
                            }
                        }
                    }
                }
            }

            OpenAiSetupModal {
                modal_visible: show_llm_modal,
                api_key: llm_modal_api_key,
                base: llm_modal_base,
                llm_server_openai,
            }
        }
    }
}
