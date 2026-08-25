//! 侧栏应用市场：我的应用 / 应用市场（官方搜索含 GitHub、AI 搜索），下载到工作区 `applications/`。

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdCopy, LdDownload, LdExternalLink, LdFolderOpen, LdPlay, LdRefreshCw, LdSearch, LdSparkles,
    LdTrash2,
};
use dioxus_free_icons::Icon;
use keyboard_types::Key;

const DEFAULT_AI_SEARCH_MODEL: &str = "gpt-5.4";

/// 一级视图：我的应用 | 应用市场（与技能侧栏「我的技能 | 技能市场」对齐）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PluginView {
    Installed,
    Market,
}

/// 应用市场下级站台：官方搜索 | AI 搜索。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PluginStation {
    Official,
    AiSearch,
}

impl PluginStation {
    fn label(self) -> &'static str {
        match self {
            Self::Official => "官方搜索",
            Self::AiSearch => "AI 搜索",
        }
    }
}

/// 需通过 `load_epoch` 拉取的列表来源（不含 AI，AI 走独立 trigger）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PluginListSource {
    Installed,
    Official,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginCard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source_label: String,
    pub language: Option<String>,
    pub stars: Option<u64>,
    pub mentions: Option<u64>,
    pub homepage: String,
    pub git_url: String,
    pub installed: bool,
    pub install_key: String,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn plugin_available() -> bool {
    true
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
fn plugin_available() -> bool {
    false
}

fn resolve_ai_model() -> String {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        if let Some(saved) = crate::web::prefs::chat_model_get() {
            let t = saved.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        if let Some(saved) = crate::desktop::files::chat_model_get() {
            let t = saved.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    DEFAULT_AI_SEARCH_MODEL.to_string()
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn to_card(item: crate::desktop::plugins::PluginItem) -> PluginCard {
    PluginCard {
        install_key: format!("{}:{}", item.source.as_str(), item.id),
        id: item.id,
        name: item.name,
        description: item.description,
        source_label: item.source.label().into(),
        language: item.language,
        stars: item.stars,
        mentions: item.mentions,
        homepage: item.homepage,
        git_url: item.git_url,
        installed: item.installed,
    }
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
async fn fetch_list_items(source: PluginListSource) -> Result<Vec<PluginCard>, String> {
    use crate::desktop::plugins::{list_installed_plugins, load_official_catalog};

    match source {
        PluginListSource::Official => {
            let items = load_official_catalog().await?;
            Ok(items.into_iter().map(to_card).collect())
        }
        PluginListSource::Installed => tokio::task::spawn_blocking(list_installed_plugins)
            .await
            .map_err(|e| format!("读取已安装列表失败：{e}"))
            .map(|list| list.into_iter().map(to_card).collect()),
    }
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
async fn fetch_list_items(source: PluginListSource) -> Result<Vec<PluginCard>, String> {
    match source {
        PluginListSource::Official => Ok(web_official_preview()),
        PluginListSource::Installed => Ok(Vec::new()),
    }
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
fn web_official_preview() -> Vec<PluginCard> {
    vec![
        PluginCard {
            id: "servers".into(),
            name: "MCP Servers".into(),
            description: "Model Context Protocol 官方参考服务器集合。".into(),
            source_label: "官方".into(),
            language: Some("TypeScript".into()),
            stars: None,
            mentions: None,
            homepage: "https://github.com/modelcontextprotocol/servers".into(),
            git_url: "https://github.com/modelcontextprotocol/servers.git".into(),
            installed: false,
            install_key: "official:servers".into(),
        },
        PluginCard {
            id: "codex".into(),
            name: "OpenAI Codex CLI".into(),
            description: "轻量终端编程 Agent。".into(),
            source_label: "官方".into(),
            language: Some("Rust".into()),
            stars: None,
            mentions: None,
            homepage: "https://github.com/openai/codex".into(),
            git_url: "https://github.com/openai/codex.git".into(),
            installed: false,
            install_key: "official:codex".into(),
        },
    ]
}

#[cfg(target_arch = "wasm32")]
fn map_github_http_error(status: u16, body: &str) -> String {
    match status {
        401 => "GitHub 认证失败，请检查环境变量 GITHUB_TOKEN / GH_TOKEN。".into(),
        403 | 429 => {
            "GitHub API 请求过于频繁或被限流，请稍后再试（可设置环境变量 GITHUB_TOKEN 提高限额）。"
                .into()
        }
        422 => "GitHub 搜索关键词无效，请换一个词再试。".into(),
        _ => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
                    return format!("GitHub 搜索失败：{msg}");
                }
            }
            format!("GitHub 搜索失败（HTTP {status}）")
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn sanitize_web_dir_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let base = trimmed
        .rsplit('/')
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches(".git");
    let mut out = String::with_capacity(base.len());
    for ch in base.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            out.push(ch);
        } else if ch == ' ' {
            out.push('-');
        }
    }
    if out.is_empty() {
        "app".into()
    } else {
        out
    }
}

async fn run_github_search(query: String) -> Result<Vec<PluginCard>, String> {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        let items = crate::desktop::plugins::search_github_repositories(&query).await?;
        Ok(items.into_iter().map(to_card).collect())
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[derive(serde::Deserialize)]
        struct GhSearch {
            #[serde(default)]
            items: Vec<GhRepo>,
        }
        #[derive(serde::Deserialize)]
        struct GhRepo {
            #[serde(default)]
            name: String,
            #[serde(default)]
            full_name: String,
            #[serde(default)]
            description: Option<String>,
            #[serde(default)]
            html_url: String,
            #[serde(default)]
            clone_url: String,
            #[serde(default)]
            language: Option<String>,
            #[serde(default)]
            stargazers_count: u64,
        }

        let q = query.trim();
        if q.is_empty() {
            return Ok(Vec::new());
        }
        let url = format!(
            "https://api.github.com/search/repositories?q={}&per_page=30&sort=stars",
            urlencoding_encode(q)
        );
        let resp = gloo_net::http::Request::get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "PusaPluginMarket/0.1")
            .send()
            .await
            .map_err(|e| format!("GitHub 搜索请求失败：{e}"))?;
        let status = resp.status();
        if !(200..300).contains(&status) {
            let body = resp.text().await.unwrap_or_default();
            return Err(map_github_http_error(status, &body));
        }
        let parsed: GhSearch = resp.json().await.map_err(|e| format!("GitHub 搜索结果解析失败：{e}"))?;
        Ok(parsed
            .items
            .into_iter()
            .filter(|r| !r.clone_url.trim().is_empty() || !r.html_url.trim().is_empty())
            .map(|r| {
                let id = sanitize_web_dir_name(if r.name.is_empty() {
                    &r.full_name
                } else {
                    &r.name
                });
                let homepage = if r.html_url.trim().is_empty() {
                    r.clone_url.trim_end_matches(".git").to_string()
                } else {
                    r.html_url
                };
                let git_url = if r.clone_url.trim().is_empty() {
                    format!("{}.git", homepage.trim_end_matches('/'))
                } else {
                    r.clone_url
                };
                let description = r
                    .description
                    .filter(|d| !d.trim().is_empty())
                    .unwrap_or_else(|| "（无描述）".into());
                let name = if r.full_name.trim().is_empty() {
                    r.name
                } else {
                    r.full_name
                };
                PluginCard {
                    install_key: format!("github:{id}"),
                    id,
                    name,
                    description,
                    source_label: "GitHub".into(),
                    language: r.language,
                    stars: Some(r.stargazers_count),
                    mentions: None,
                    homepage,
                    git_url,
                    installed: false,
                }
            })
            .collect())
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        let _ = query;
        Err("当前构建不支持 GitHub 搜索。".into())
    }
}

#[cfg(target_arch = "wasm32")]
fn urlencoding_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

async fn run_ai_search(query: String, model: String) -> Result<Vec<PluginCard>, String> {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        let items = crate::desktop::plugins::search_apps_with_ai(&query, &model).await?;
        Ok(items.into_iter().map(to_card).collect())
    }

    #[cfg(target_arch = "wasm32")]
    {
        let base = crate::chat::api_base_url();
        let url = format!("{}/v1/plugins/ai-search", base.trim_end_matches('/'));
        let mut builder = gloo_net::http::Request::post(&url);
        if let Some(tok) = crate::web::auth::token_get() {
            builder = builder.header("Authorization", &format!("Bearer {tok}"));
        }
        let resp = builder
            .json(&protocol::PluginAiSearchRequest { query, model })
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.ok() {
            let status = resp.status();
            let raw = resp.text().await.unwrap_or_default();
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(err) = v.get("error").and_then(|x| x.as_str()) {
                    if err.contains("openai_api_key_missing") {
                        return Err("尚未配置 LLM API Key，请先在设置中配置后再使用 AI 搜索。".into());
                    }
                    return Err(err.to_string());
                }
            }
            return Err(format!("AI 搜索失败（HTTP {status}）"));
        }
        let body: protocol::PluginAiSearchResponse =
            resp.json().await.map_err(|e| e.to_string())?;
        Ok(body
            .items
            .into_iter()
            .map(|d| {
                let homepage = d
                    .homepage
                    .unwrap_or_else(|| d.git_url.trim_end_matches(".git").to_string());
                PluginCard {
                    install_key: format!("ai:{}", d.id),
                    id: d.id,
                    name: d.name,
                    description: d.description,
                    source_label: "AI".into(),
                    language: d.language,
                    stars: None,
                    mentions: None,
                    homepage,
                    git_url: d.git_url,
                    installed: false,
                }
            })
            .collect())
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        let _ = (query, model);
        Err("当前构建不支持 AI 搜索。".into())
    }
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn install_card(card: &PluginCard) -> Result<String, String> {
    use crate::desktop::plugins::{install_plugin, PluginItem, PluginSource};
    let source = if card.install_key.starts_with("ai:") {
        PluginSource::Ai
    } else if card.install_key.starts_with("github:") {
        PluginSource::GitHub
    } else {
        PluginSource::Official
    };
    let item = PluginItem {
        id: card.id.clone(),
        name: card.name.clone(),
        description: card.description.clone(),
        source,
        language: card.language.clone(),
        stars: card.stars,
        mentions: card.mentions,
        homepage: card.homepage.clone(),
        git_url: card.git_url.clone(),
        installed: card.installed,
    };
    let path = install_plugin(&item)?;
    Ok(format!("已安装到 {}", path.display()))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
fn install_card(_card: &PluginCard) -> Result<String, String> {
    Err("Web 端无法下载到本机 application 目录，请使用桌面版。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn open_installed_card(card: &PluginCard) -> Result<(), String> {
    crate::desktop::plugins::open_installed_plugin(&card.id)
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
fn open_installed_card(_card: &PluginCard) -> Result<(), String> {
    Err("Web 端无法打开本机目录，请使用桌面版。".into())
}

/// 已安装应用的绝对路径（桌面）；Web 返回相对展示路径。
fn installed_absolute_path(card: &PluginCard) -> Result<String, String> {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::plugins::plugin_dir(&card.id)
            .map(|p| p.to_string_lossy().into_owned())
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        Ok(detail_install_path(card))
    }
}

fn copy_installed_path(card: &PluginCard) -> Result<String, String> {
    let path = installed_absolute_path(card)?;
    super::files::fs_clipboard_set_text(&path)?;
    Ok(path)
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn uninstall_installed_card(card: &PluginCard) -> Result<(), String> {
    crate::desktop::plugins::uninstall_plugin(&card.id)
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
fn uninstall_installed_card(_card: &PluginCard) -> Result<(), String> {
    Err("Web 端无法卸载本机应用，请使用桌面版。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
async fn run_install(card: PluginCard) -> Result<String, String> {
    tokio::task::spawn_blocking(move || install_card(&card))
        .await
        .unwrap_or_else(|e| Err(format!("安装任务失败：{e}")))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
async fn run_install(card: PluginCard) -> Result<String, String> {
    install_card(&card)
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
async fn run_open_installed(card: PluginCard) -> Result<(), String> {
    tokio::task::spawn_blocking(move || open_installed_card(&card))
        .await
        .unwrap_or_else(|e| Err(format!("打开目录失败：{e}")))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
async fn run_open_installed(card: PluginCard) -> Result<(), String> {
    open_installed_card(&card)
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
async fn run_uninstall(card: PluginCard) -> Result<(), String> {
    tokio::task::spawn_blocking(move || uninstall_installed_card(&card))
        .await
        .unwrap_or_else(|e| Err(format!("卸载任务失败：{e}")))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
async fn run_uninstall(card: PluginCard) -> Result<(), String> {
    uninstall_installed_card(&card)
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
async fn load_smart_ui_for(id: String) -> Result<Option<protocol::PluginSmartUiDto>, String> {
    tokio::task::spawn_blocking(move || crate::desktop::plugins::load_smart_ui(&id))
        .await
        .unwrap_or_else(|e| Err(format!("读取智能 UI 失败：{e}")))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
async fn load_smart_ui_for(_id: String) -> Result<Option<protocol::PluginSmartUiDto>, String> {
    Ok(None)
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
async fn generate_smart_ui_for(
    id: String,
    model: String,
) -> Result<protocol::PluginSmartUiDto, String> {
    crate::desktop::plugins::generate_smart_ui(&id, &model).await
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
async fn generate_smart_ui_for(
    _id: String,
    _model: String,
) -> Result<protocol::PluginSmartUiDto, String> {
    Err("智能 UI 仅桌面版可用。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
async fn ensure_smart_ui_for(
    id: String,
    model: String,
) -> Result<protocol::PluginSmartUiDto, String> {
    crate::desktop::plugins::ensure_smart_ui(&id, &model).await
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
async fn ensure_smart_ui_for(
    _id: String,
    _model: String,
) -> Result<protocol::PluginSmartUiDto, String> {
    Err("智能 UI 仅桌面版可用。".into())
}

/// 后台为缺少缓存的已安装应用扫描并写入 `applications/{id}/.pusa-smart-ui.json`。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn spawn_auto_scan_missing_smart_ui(toast: super::toast::ToastCtx) {
    let model = resolve_ai_model();
    spawn(async move {
        let missing = match tokio::task::spawn_blocking(
            crate::desktop::plugins::list_ids_missing_smart_ui,
        )
        .await
        {
            Ok(list) => list,
            Err(e) => {
                toast.error(format!("扫描智能 UI 失败：{e}"));
                return;
            }
        };
        if missing.is_empty() {
            return;
        }
        let n = missing.len();
        toast.info(format!("正在为 {n} 个应用自动扫描智能 UI…"));
        match crate::desktop::plugins::ensure_missing_smart_uis(&model).await {
            Ok((ok, errs)) => {
                if ok > 0 {
                    toast.success(format!("已为 {ok} 个应用缓存智能 UI"));
                }
                if let Some((_, e)) = errs.first() {
                    if errs.len() == 1 {
                        toast.error(format!("智能 UI 生成失败：{e}"));
                    } else {
                        toast.error(format!(
                            "智能 UI：{ok} 成功，{} 失败（例如：{e})",
                            errs.len()
                        ));
                    }
                }
            }
            Err(e) => toast.error(format!("智能 UI 扫描失败：{e}")),
        }
    });
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
fn spawn_auto_scan_missing_smart_ui(_toast: super::toast::ToastCtx) {}

/// 单个应用安装后：确保智能 UI 已扫描并缓存。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn spawn_ensure_smart_ui_after_install(id: String, toast: super::toast::ToastCtx) {
    let model = resolve_ai_model();
    spawn(async move {
        match ensure_smart_ui_for(id.clone(), model).await {
            Ok(_) => toast.success(format!("「{id}」智能 UI 已缓存")),
            Err(e) => toast.error(format!("「{id}」智能 UI 生成失败：{e}")),
        }
    });
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
fn spawn_ensure_smart_ui_after_install(_id: String, _toast: super::toast::ToastCtx) {}

fn format_meta(card: &PluginCard) -> String {
    let mut parts = vec![card.source_label.clone()];
    if let Some(lang) = &card.language {
        parts.push(lang.clone());
    }
    if let Some(mentions) = card.mentions {
        parts.push(format!("提及 {mentions}"));
    }
    if let Some(stars) = card.stars {
        parts.push(format!("★{stars}"));
    }
    parts.join(" · ")
}

/// 详情页元信息：不含来源徽章（GitHub / AI / 官方）。
fn format_detail_meta(card: &PluginCard) -> String {
    let mut parts = Vec::new();
    if let Some(lang) = &card.language {
        parts.push(lang.clone());
    }
    if let Some(mentions) = card.mentions {
        parts.push(format!("提及 {mentions}"));
    }
    if let Some(stars) = card.stars {
        parts.push(format!("★{stars}"));
    }
    parts.join(" · ")
}

/// 工作区相对安装路径，展示在详情标题下方。
fn detail_install_path(card: &PluginCard) -> String {
    format!("applications/{}/", card.id)
}

/// 去掉「已安装到」前缀；若描述仅是安装路径则返回空（路径改由标题区展示）。
fn detail_description(raw: &str, install_path: &str) -> String {
    let t = raw.trim();
    let t = if let Some(rest) = t.strip_prefix("已安装到") {
        rest.trim_start()
    } else {
        t
    };
    if t.is_empty() || t == install_path {
        String::new()
    } else {
        t.to_string()
    }
}

fn upsert_catalog(catalog: &mut Vec<PluginCard>, card: PluginCard) {
    if let Some(existing) = catalog
        .iter_mut()
        .find(|c| c.install_key == card.install_key)
    {
        *existing = card;
    } else {
        catalog.push(card);
    }
}

fn sync_catalog(mut catalog: Signal<Vec<PluginCard>>, cards: &[PluginCard]) {
    catalog.with_mut(|cat| {
        for card in cards {
            upsert_catalog(cat, card.clone());
        }
    });
}

fn mark_catalog_installed(mut catalog: Signal<Vec<PluginCard>>, key: &str, installed: bool) {
    catalog.with_mut(|cat| {
        for card in cat.iter_mut() {
            if card.install_key == key {
                card.installed = installed;
            }
        }
    });
}

fn remove_catalog_card(mut catalog: Signal<Vec<PluginCard>>, key: &str) {
    catalog.with_mut(|cat| {
        cat.retain(|c| c.install_key != key);
    });
}

#[component]
pub fn PluginDetailPane(
    plugin_key: String,
    catalog: Signal<Vec<PluginCard>>,
    mut refresh_tick: Signal<u64>,
    on_uninstalled: EventHandler<String>,
    /// `(plugin_id, command)`：在 pusa 专用终端中执行。
    on_run_in_terminal: EventHandler<(String, String)>,
) -> Element {
    let toast = super::toast::use_toast();
    let mut installing = use_signal(|| None::<String>);
    let mut uninstalling = use_signal(|| None::<String>);
    let mut smart_ui = use_signal(|| None::<protocol::PluginSmartUiDto>);
    let mut smart_ui_visible = use_signal(|| false);
    let mut smart_ui_busy = use_signal(|| false);
    let mut smart_ui_running = use_signal(|| None::<String>);

    // 打开已安装应用详情时：自动加载/生成智能 UI 并展示。
    use_effect({
        let plugin_key = plugin_key.clone();
        move || {
            let key = plugin_key.clone();
            smart_ui.set(None);
            smart_ui_visible.set(false);
            smart_ui_busy.set(false);
            smart_ui_running.set(None);
            let Some(card) = catalog().into_iter().find(|c| c.install_key == key) else {
                return;
            };
            if !card.installed || !plugin_available() {
                return;
            }
            let id = card.id.clone();
            smart_ui_visible.set(true);
            smart_ui_busy.set(true);
            let model = resolve_ai_model();
            spawn(async move {
                match ensure_smart_ui_for(id, model).await {
                    Ok(dto) => {
                        smart_ui.set(Some(dto));
                        smart_ui_visible.set(true);
                    }
                    Err(e) => {
                        smart_ui_visible.set(false);
                        // 无 API Key / 空项目等：不打断详情浏览，仅提示一次。
                        toast.warning(format!("智能 UI：{e}"));
                    }
                }
                smart_ui_busy.set(false);
            });
        }
    });

    let card = catalog()
        .into_iter()
        .find(|c| c.install_key == plugin_key);

    rsx! {
        section { class: "ac-skills-detail-pane",
            if let Some(card) = card {
                {
                    let key = card.install_key.clone();
                    let key_install = key.clone();
                    let key_uninstall = key.clone();
                    let card_install = card.clone();
                    let card_open = card.clone();
                    let card_copy = card.clone();
                    let card_uninstall = card.clone();
                    let card_smart = card.clone();
                    let homepage = card.homepage.clone();
                    let name = card.name.clone();
                    let path = detail_install_path(&card);
                    let desc = detail_description(&card.description, &path);
                    let meta = format_detail_meta(&card);
                    let installed = card.installed;
                    let available = plugin_available();
                    rsx! {
                        div { class: "ac-skills-detail",
                            div { class: "ac-skills-detail-head",
                                div { class: "ac-skills-detail-icon ac-skill-icon-blue",
                                    Icon {
                                        icon: LdDownload,
                                        width: 28,
                                        height: 28,
                                        fill: "currentColor",
                                        class: "ac-skill-card-icon",
                                    }
                                }
                                div { class: "ac-skills-detail-titles",
                                    h2 { "{name}" }
                                    p { class: "ac-skills-detail-slug", "{path}" }
                                }
                            }
                            if !desc.is_empty() {
                                p { class: "ac-skills-detail-desc", "{desc}" }
                            }
                            if !meta.is_empty() {
                                div { class: "ac-skill-market-meta",
                                    span { "{meta}" }
                                }
                            }
                            if installed && smart_ui_visible() {
                                if let Some(ui) = smart_ui() {
                                    div { class: "ac-smart-ui-panel",
                                        div { class: "ac-smart-ui-head",
                                            div { class: "ac-smart-ui-titles",
                                                h3 { "{ui.title}" }
                                                if !ui.summary.is_empty() {
                                                    p { "{ui.summary}" }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "ac-smart-ui-regen",
                                                title: "重新扫描并生成",
                                                disabled: smart_ui_busy() || smart_ui_running().is_some(),
                                                onclick: {
                                                    let id = card_smart.id.clone();
                                                    move |_| {
                                                        if smart_ui_busy() {
                                                            return;
                                                        }
                                                        smart_ui_busy.set(true);
                                                        toast.info("正在扫描项目并生成智能 UI…");
                                                        let id = id.clone();
                                                        let model = resolve_ai_model();
                                                        spawn(async move {
                                                            match generate_smart_ui_for(id, model).await {
                                                                Ok(dto) => {
                                                                    smart_ui.set(Some(dto));
                                                                    smart_ui_visible.set(true);
                                                                    toast.success("智能 UI 已重新生成并缓存");
                                                                }
                                                                Err(e) => {
                                                                    toast.error(format!("生成失败：{e}"));
                                                                }
                                                            }
                                                            smart_ui_busy.set(false);
                                                        });
                                                    }
                                                },
                                                Icon {
                                                    icon: LdRefreshCw,
                                                    width: 14,
                                                    height: 14,
                                                    fill: "currentColor",
                                                    class: "ac-skill-action-icon",
                                                }
                                                if smart_ui_busy() { "生成中" } else { "重新生成" }
                                            }
                                        }
                                        div { class: "ac-smart-ui-actions",
                                            for action in ui.actions.iter() {
                                                {
                                                    let action_id = action.id.clone();
                                                    let action_label = action.label.clone();
                                                    let action_cmd = action.command.clone();
                                                    let action_desc = action.description.clone().unwrap_or_default();
                                                    let _action_detached = action.detached;
                                                    let plugin_id = card_smart.id.clone();
                                                    let title = if action_desc.is_empty() {
                                                        action_cmd.clone()
                                                    } else {
                                                        format!("{action_desc}\n{action_cmd}")
                                                    };
                                                    rsx! {
                                                        button {
                                                            r#type: "button",
                                                            class: "ac-smart-ui-action",
                                                            title: "{title}",
                                                            disabled: smart_ui_busy() || smart_ui_running().is_some(),
                                                            onclick: move |_| {
                                                                let id = plugin_id.clone();
                                                                let cmd = action_cmd.clone();
                                                                let aid = action_id.clone();
                                                                smart_ui_running.set(Some(aid));
                                                                on_run_in_terminal.call((id, cmd));
                                                                smart_ui_running.set(None);
                                                            },
                                                            Icon {
                                                                icon: LdPlay,
                                                                width: 14,
                                                                height: 14,
                                                                fill: "currentColor",
                                                                class: "ac-skill-action-icon",
                                                            }
                                                            if smart_ui_running() == Some(action_id.clone()) {
                                                                "执行中"
                                                            } else {
                                                                "{action_label}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else if smart_ui_busy() {
                                    div { class: "ac-smart-ui-panel ac-smart-ui-loading",
                                        p { "正在扫描项目并生成智能 UI…" }
                                    }
                                }
                            }
                            div { class: "ac-skills-detail-actions",
                                if installed {
                                    if available {
                                        button {
                                            r#type: "button",
                                            class: "ac-installed-equip",
                                            title: "打开安装目录",
                                            onclick: move |_| {
                                                let card = card_open.clone();
                                                spawn(async move {
                                                    match run_open_installed(card).await {
                                                        Ok(()) => {}
                                                        Err(e) => toast.error(e),
                                                    }
                                                });
                                            },
                                            Icon {
                                                icon: LdFolderOpen,
                                                width: 15,
                                                height: 15,
                                                fill: "currentColor",
                                                class: "ac-skill-action-icon",
                                            }
                                            "打开目录"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "ac-installed-equip",
                                            title: "复制安装目录绝对路径",
                                            onclick: move |_| {
                                                let card = card_copy.clone();
                                                match copy_installed_path(&card) {
                                                    Ok(path) => {
                                                        toast.success(format!("已复制路径：{path}"));
                                                    }
                                                    Err(e) => toast.error(e),
                                                }
                                            },
                                            Icon {
                                                icon: LdCopy,
                                                width: 15,
                                                height: 15,
                                                fill: "currentColor",
                                                class: "ac-skill-action-icon",
                                            }
                                            "复制路径"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "ac-installed-action",
                                            title: "卸载",
                                            disabled: uninstalling().is_some() || installing().is_some() || smart_ui_busy(),
                                            onclick: move |_| {
                                                let card = card_uninstall.clone();
                                                let key = key_uninstall.clone();
                                                uninstalling.set(Some(key.clone()));
                                                spawn(async move {
                                                    match run_uninstall(card).await {
                                                        Ok(()) => {
                                                            toast.success("已卸载");
                                                            remove_catalog_card(catalog, &key);
                                                            refresh_tick.with_mut(|n| *n += 1);
                                                            on_uninstalled.call(key);
                                                        }
                                                        Err(e) => {
                                                            toast.error(format!("卸载失败：{e}"));
                                                        }
                                                    }
                                                    uninstalling.set(None);
                                                });
                                            },
                                            Icon {
                                                icon: LdTrash2,
                                                width: 15,
                                                height: 15,
                                                fill: "currentColor",
                                                class: "ac-skill-action-icon",
                                            }
                                            if uninstalling() == Some(key.clone()) {
                                                "卸载中"
                                            } else {
                                                "卸载"
                                            }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "ac-installed-equip",
                                            title: "AI 扫描项目并生成可执行脚本面板",
                                            disabled: smart_ui_busy() || uninstalling().is_some(),
                                            onclick: {
                                                let id = card_smart.id.clone();
                                                move |_| {
                                                    if smart_ui_busy() {
                                                        return;
                                                    }
                                                    // 本会话已加载：直接展示（缓存已在首次生成/加载时落盘）。
                                                    if smart_ui().is_some() {
                                                        smart_ui_visible.set(true);
                                                        return;
                                                    }
                                                    smart_ui_visible.set(true);
                                                    smart_ui_busy.set(true);
                                                    let id = id.clone();
                                                    let model = resolve_ai_model();
                                                    spawn(async move {
                                                        // 先读 `.pusa-smart-ui.json`；没有再扫描生成并立即缓存。
                                                        match load_smart_ui_for(id.clone()).await {
                                                            Ok(Some(dto)) => {
                                                                smart_ui.set(Some(dto));
                                                                smart_ui_busy.set(false);
                                                                toast.success("已加载智能 UI 缓存");
                                                                return;
                                                            }
                                                            Ok(None) => {}
                                                            Err(e) => {
                                                                toast.error(e);
                                                                smart_ui_visible.set(false);
                                                                smart_ui_busy.set(false);
                                                                return;
                                                            }
                                                        }
                                                        toast.info("正在扫描项目并生成智能 UI…");
                                                        match generate_smart_ui_for(id, model).await {
                                                            Ok(dto) => {
                                                                smart_ui.set(Some(dto));
                                                                toast.success("智能 UI 已生成并缓存");
                                                            }
                                                            Err(e) => {
                                                                smart_ui_visible.set(false);
                                                                toast.error(format!("生成失败：{e}"));
                                                            }
                                                        }
                                                        smart_ui_busy.set(false);
                                                    });
                                                }
                                            },
                                            Icon {
                                                icon: LdSparkles,
                                                width: 15,
                                                height: 15,
                                                fill: "currentColor",
                                                class: "ac-skill-action-icon",
                                            }
                                            if smart_ui_busy() {
                                                "生成中"
                                            } else {
                                                "智能UI"
                                            }
                                        }
                                    }
                                } else if available {
                                    button {
                                        r#type: "button",
                                        class: "ac-skill-install-btn",
                                        disabled: installing().is_some(),
                                        onclick: move |_| {
                                            let card = card_install.clone();
                                            let install_id = card.id.clone();
                                            let key = key_install.clone();
                                            installing.set(Some(key.clone()));
                                            spawn(async move {
                                                match run_install(card).await {
                                                    Ok(msg) => {
                                                        toast.success(msg);
                                                        mark_catalog_installed(catalog, &key, true);
                                                        refresh_tick.with_mut(|n| *n += 1);
                                                        spawn_ensure_smart_ui_after_install(
                                                            install_id,
                                                            toast,
                                                        );
                                                    }
                                                    Err(e) => {
                                                        toast.error(format!("下载失败：{e}"));
                                                    }
                                                }
                                                installing.set(None);
                                            });
                                        },
                                        Icon {
                                            icon: LdDownload,
                                            width: 15,
                                            height: 15,
                                            fill: "currentColor",
                                            class: "ac-skill-action-icon",
                                        }
                                        if installing() == Some(key.clone()) {
                                            "下载中"
                                        } else {
                                            "下载"
                                        }
                                    }
                                } else if !homepage.is_empty() {
                                    a {
                                        class: "ac-skill-install-btn",
                                        href: "{homepage}",
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        Icon {
                                            icon: LdExternalLink,
                                            width: 15,
                                            height: 15,
                                            fill: "currentColor",
                                            class: "ac-skill-action-icon",
                                        }
                                        "在浏览器打开"
                                    }
                                } else {
                                    button {
                                        r#type: "button",
                                        class: "ac-skill-installed-btn",
                                        disabled: true,
                                        "请使用桌面版安装"
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "ac-skills-detail-empty",
                    Icon {
                        icon: LdDownload,
                        width: 40,
                        height: 40,
                        fill: "currentColor",
                        class: "ac-installed-empty-icon",
                    }
                    h3 { "应用" }
                    p { "该应用已不在当前列表中，可重新搜索后打开。" }
                }
            }
        }
    }
}

#[component]
pub fn SidebarPluginMarket(
    mut selected_key: Signal<Option<String>>,
    mut catalog: Signal<Vec<PluginCard>>,
    refresh_tick: Signal<u64>,
    on_open: EventHandler<(String, String)>,
) -> Element {
    let toast = super::toast::use_toast();
    // 与技能侧栏一致：进入应用栏默认「我的应用」，而非「应用市场」。
    let mut plugin_view = use_signal(|| PluginView::Installed);
    let mut station = use_signal(|| PluginStation::Official);
    let mut search = use_signal(String::new);
    let mut items = use_signal(Vec::<PluginCard>::new);
    let mut ai_items = use_signal(Vec::<PluginCard>::new);
    let mut github_items = use_signal(Vec::<PluginCard>::new);
    let mut ai_query_done = use_signal(|| None::<String>);
    let mut github_query_done = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let mut ai_loading = use_signal(|| false);
    let mut github_loading = use_signal(|| false);
    let mut loaded_source = use_signal(|| None::<PluginListSource>);
    let mut load_epoch = use_signal(|| 0_u64);
    let mut ai_search_trigger = use_signal(|| 0_u64);
    let mut github_search_trigger = use_signal(|| 0_u64);
    let mut error = use_signal(|| None::<String>);
    let mut ai_error = use_signal(|| None::<String>);
    let mut github_error = use_signal(|| None::<String>);

    let active_source = use_memo(move || {
        if plugin_view() == PluginView::Installed {
            Some(PluginListSource::Installed)
        } else if station() == PluginStation::Official {
            Some(PluginListSource::Official)
        } else {
            None
        }
    });

    // 仅响应 refresh_tick：强制重新拉取当前源。切勿在此订阅 catalog，
    // 否则 fetch → sync_catalog 写 catalog 会再次触发本 effect，造成无限刷新。
    use_effect(move || {
        let tick = refresh_tick();
        if tick == 0 {
            return;
        }
        loaded_source.set(None);
        load_epoch.with_mut(|n| *n += 1);
    });

    // 目录安装状态变更时，同步到 AI / GitHub 缓存列表（不触发列表重拉）。
    use_effect(move || {
        let cat = catalog();
        ai_items.with_mut(|list| {
            for c in list.iter_mut() {
                if let Some(found) = cat.iter().find(|x| x.install_key == c.install_key) {
                    c.installed = found.installed;
                }
            }
        });
        github_items.with_mut(|list| {
            for c in list.iter_mut() {
                if let Some(found) = cat.iter().find(|x| x.install_key == c.install_key) {
                    c.installed = found.installed;
                }
            }
        });
    });

    use_effect(move || {
        let _epoch = load_epoch();
        let Some(source) = active_source() else {
            return;
        };
        if loaded_source() == Some(source) {
            return;
        }
        loading.set(true);
        error.set(None);
        spawn(async move {
            match fetch_list_items(source).await {
                Ok(list) => {
                    sync_catalog(catalog, &list);
                    items.set(list);
                    error.set(None);
                    loaded_source.set(Some(source));
                    if source == PluginListSource::Installed {
                        spawn_auto_scan_missing_smart_ui(toast);
                    }
                }
                Err(e) => {
                    items.set(Vec::new());
                    error.set(Some(e));
                    loaded_source.set(None);
                }
            }
            loading.set(false);
        });
    });

    use_effect(move || {
        let trigger = ai_search_trigger();
        if trigger == 0 {
            return;
        }
        let Some(query) = ai_query_done() else {
            return;
        };
        if query.trim().is_empty() {
            return;
        }
        ai_loading.set(true);
        ai_error.set(None);
        let model = resolve_ai_model();
        spawn(async move {
            match run_ai_search(query, model).await {
                Ok(list) => {
                    sync_catalog(catalog, &list);
                    ai_items.set(list);
                    ai_error.set(None);
                }
                Err(e) => {
                    ai_items.set(Vec::new());
                    ai_error.set(Some(e));
                }
            }
            ai_loading.set(false);
        });
    });

    use_effect(move || {
        let trigger = github_search_trigger();
        if trigger == 0 {
            return;
        }
        let Some(query) = github_query_done() else {
            return;
        };
        if query.trim().is_empty() {
            return;
        }
        github_loading.set(true);
        github_error.set(None);
        spawn(async move {
            match run_github_search(query).await {
                Ok(list) => {
                    sync_catalog(catalog, &list);
                    github_items.set(list);
                    github_error.set(None);
                }
                Err(e) => {
                    github_items.set(Vec::new());
                    github_error.set(Some(e));
                }
            }
            github_loading.set(false);
        });
    });

    let on_market = use_memo(move || plugin_view() == PluginView::Market);
    let on_installed = use_memo(move || plugin_view() == PluginView::Installed);
    let on_ai = use_memo(move || on_market() && station() == PluginStation::AiSearch);
    let on_official = use_memo(move || on_market() && station() == PluginStation::Official);

    let showing_github = use_memo(move || {
        on_official()
            && github_query_done()
                .as_ref()
                .map(|q| !q.trim().is_empty())
                .unwrap_or(false)
    });

    let filtered = use_memo(move || {
        if on_ai() {
            return ai_items();
        }
        if showing_github() {
            return github_items();
        }
        let q = search().trim().to_lowercase();
        items()
            .into_iter()
            .filter(|card| {
                if q.is_empty() {
                    return true;
                }
                card.name.to_lowercase().contains(&q)
                    || card.description.to_lowercase().contains(&q)
                    || card
                        .language
                        .as_ref()
                        .map(|l| l.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>()
    });

    let list_loading = if on_ai() {
        ai_loading()
    } else if showing_github() {
        github_loading()
    } else {
        loading()
    };

    let list_error = if on_ai() {
        ai_error()
    } else if showing_github() {
        github_error()
    } else {
        error()
    };
    let has_list_error = list_error.is_some();

    rsx! {
        div { class: "ac-sidebar-skills ac-sidebar-plugins",
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
                    value: "{search()}",
                    placeholder: if on_ai() {
                        "搜索应用（Enter 用 AI）"
                    } else if on_installed() {
                        "筛选我的应用"
                    } else {
                        "搜索应用（Enter 搜 GitHub）"
                    },
                    oninput: move |e| {
                        let v = e.value();
                        search.set(v.clone());
                        if on_official() && v.trim().is_empty() {
                            github_query_done.set(None);
                            github_items.set(Vec::new());
                            github_error.set(None);
                        }
                    },
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() != Key::Enter {
                            return;
                        }
                        e.prevent_default();
                        let q = search().trim().to_string();
                        selected_key.set(None);
                        if on_installed() {
                            return;
                        }
                        if on_official() {
                            if q.is_empty() {
                                github_query_done.set(None);
                                github_items.set(Vec::new());
                                github_error.set(None);
                                return;
                            }
                            github_query_done.set(Some(q));
                            github_search_trigger.with_mut(|n| *n += 1);
                            return;
                        }
                        if on_ai() {
                            if q.is_empty() {
                                toast.warning("请输入关键词后再使用 AI 搜索。");
                                return;
                            }
                            ai_query_done.set(Some(q));
                            ai_search_trigger.with_mut(|n| *n += 1);
                        }
                    },
                }
            }
            div { class: "ac-sidebar-skills-toggle",
                button {
                    r#type: "button",
                    class: if on_installed() {
                        "ac-sidebar-skills-toggle-btn is-active"
                    } else {
                        "ac-sidebar-skills-toggle-btn"
                    },
                    onclick: move |_| {
                        if plugin_view() == PluginView::Installed {
                            return;
                        }
                        plugin_view.set(PluginView::Installed);
                        selected_key.set(None);
                        loaded_source.set(None);
                        load_epoch.with_mut(|n| *n += 1);
                    },
                    "我的应用"
                }
                button {
                    r#type: "button",
                    class: if on_market() {
                        "ac-sidebar-skills-toggle-btn is-active"
                    } else {
                        "ac-sidebar-skills-toggle-btn"
                    },
                    onclick: move |_| {
                        if plugin_view() == PluginView::Market {
                            return;
                        }
                        plugin_view.set(PluginView::Market);
                        selected_key.set(None);
                        if station() == PluginStation::Official {
                            loaded_source.set(None);
                            load_epoch.with_mut(|n| *n += 1);
                        } else if ai_query_done().is_none() {
                            let q = search().trim().to_string();
                            if !q.is_empty() {
                                ai_query_done.set(Some(q));
                                ai_search_trigger.with_mut(|n| *n += 1);
                            }
                        }
                    },
                    "应用市场"
                }
            }
            if on_market() {
                div { class: "ac-sidebar-skills-stations",
                    for st in [PluginStation::Official, PluginStation::AiSearch] {
                        {
                            let is_active = station() == st;
                            rsx! {
                                button {
                                    r#type: "button",
                                    class: if is_active {
                                        "ac-sidebar-skills-station-btn is-active"
                                    } else {
                                        "ac-sidebar-skills-station-btn"
                                    },
                                    title: "{st.label()}",
                                    onclick: move |_| {
                                        if station() == st {
                                            return;
                                        }
                                        station.set(st);
                                        selected_key.set(None);
                                        if st == PluginStation::Official {
                                            loaded_source.set(None);
                                            load_epoch.with_mut(|n| *n += 1);
                                        } else if ai_query_done().is_none() {
                                            let q = search().trim().to_string();
                                            if !q.is_empty() {
                                                ai_query_done.set(Some(q));
                                                ai_search_trigger.with_mut(|n| *n += 1);
                                            }
                                        }
                                    },
                                    "{st.label()}"
                                }
                            }
                        }
                    }
                }
            }
            if !plugin_available() {
                div { class: "ac-sidebar-skills-banner is-warn",
                    "桌面端可将项目克隆到工作区 applications/；Web 可浏览官方预览、GitHub 与 AI 搜索。"
                }
            }
            if list_loading {
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
            } else {
                if let Some(err) = list_error.as_ref() {
                    div { class: "ac-sidebar-skills-banner is-error", "{err}" }
                }
                if filtered().is_empty() && !has_list_error {
                    div { class: "ac-sidebar-skills-empty",
                        if on_ai() {
                            if ai_query_done().is_none() {
                                "输入关键词后按 Enter，用 AI 搜索相关开源应用"
                            } else {
                                "没有匹配的应用"
                            }
                        } else if on_installed() {
                            if search().trim().is_empty() {
                                "暂无已安装应用，去「应用市场」浏览。"
                            } else {
                                "没有匹配的应用"
                            }
                        } else if showing_github() {
                            "没有匹配的 GitHub 仓库"
                        } else if search().trim().is_empty() {
                            "暂无应用"
                        } else {
                            "没有匹配的应用（按 Enter 搜索 GitHub）"
                        }
                    }
                } else if !filtered().is_empty() {
                    div { class: "ac-skills-section-header",
                        span {
                            if on_ai() {
                                "AI 推荐"
                            } else if on_installed() {
                                "我的应用"
                            } else if showing_github() {
                                "GitHub 搜索"
                            } else {
                                "官方推荐"
                            }
                        }
                    }
                    div { class: "ac-skills-list",
                        for card in filtered() {
                            {
                                let meta = format_meta(&card);
                                let key = card.install_key.clone();
                                let key_select = key.clone();
                                let title = card.name.clone();
                                let card_open = card.clone();
                                let is_selected = selected_key() == Some(key.clone());
                                rsx! {
                                    div {
                                        class: if is_selected {
                                            "ac-skills-list-row is-selected"
                                        } else {
                                            "ac-skills-list-row"
                                        },
                                        onclick: move |_| {
                                            catalog.with_mut(|cat| {
                                                upsert_catalog(cat, card_open.clone());
                                            });
                                            on_open.call((key_select.clone(), title.clone()));
                                        },
                                        div { class: "ac-skills-list-icon ac-skill-icon-blue",
                                            Icon {
                                                icon: LdDownload,
                                                width: 16,
                                                height: 16,
                                                fill: "currentColor",
                                                class: "ac-skill-card-icon",
                                            }
                                        }
                                        div { class: "ac-skills-list-body",
                                            div { class: "ac-skills-list-title", "{card.name}" }
                                            div { class: "ac-skills-list-desc", "{card.description}" }
                                            div { class: "ac-skills-list-meta", "{meta}" }
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
