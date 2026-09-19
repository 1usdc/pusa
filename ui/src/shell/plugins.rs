//! 侧栏扩展市场：已安装扩展 / Open VSX（VS Code 开源扩展），安装到工作区 `extensions/`。

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdCopy, LdDownload, LdExternalLink, LdFolderOpen, LdPlay, LdRefreshCw, LdSearch, LdSparkles,
    LdTrash2,
};
use dioxus_free_icons::Icon;
use keyboard_types::Key;

/// 一级视图：我的扩展 | 扩展市场。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PluginView {
    Installed,
    Market,
}

/// 需通过 `load_epoch` 拉取的列表来源。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PluginListSource {
    Installed,
    Market,
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
    /// Open VSX `.vsix` 下载 URL。
    pub git_url: String,
    pub installed: bool,
    pub install_key: String,
    pub highlight_priority: bool,
    pub version: Option<String>,
    /// Open VSX 扩展图标 URL；缺省时用名称首字母占位。
    pub icon_url: Option<String>,
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
    "gpt-5.4".to_string()
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
        highlight_priority: item.highlight_priority,
        version: item.version,
        icon_url: item.icon_url,
    }
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
async fn fetch_list_items(source: PluginListSource) -> Result<Vec<PluginCard>, String> {
    use crate::desktop::plugins::{browse_openvsx_highlight_catalog, list_installed_plugins};

    match source {
        PluginListSource::Market => {
            let items = browse_openvsx_highlight_catalog().await?;
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
        PluginListSource::Market => run_openvsx_search(String::new()).await,
        PluginListSource::Installed => Ok(Vec::new()),
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

#[cfg(target_arch = "wasm32")]
fn is_highlight_text(name: &str, description: &str) -> bool {
    let blob = format!("{} {}", name, description).to_ascii_lowercase();
    ["grammar", "syntax", "highlight", "textmate", "language", "tmlanguage"]
        .iter()
        .any(|k| blob.contains(k))
}

async fn run_openvsx_search(query: String) -> Result<Vec<PluginCard>, String> {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        let items = crate::desktop::plugins::search_openvsx_extensions(&query).await?;
        Ok(items.into_iter().map(to_card).collect())
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[derive(serde::Deserialize)]
        struct SearchResp {
            #[serde(default)]
            extensions: Vec<SearchItem>,
        }
        #[derive(serde::Deserialize)]
        struct SearchItem {
            #[serde(default)]
            namespace: String,
            #[serde(default)]
            name: String,
            #[serde(default, rename = "displayName")]
            display_name: Option<String>,
            #[serde(default)]
            description: Option<String>,
            #[serde(default)]
            version: Option<String>,
            #[serde(default, rename = "downloadCount")]
            download_count: Option<u64>,
            #[serde(default)]
            url: Option<String>,
            #[serde(default)]
            files: Option<Files>,
            #[serde(default)]
            icon: Option<String>,
        }
        #[derive(serde::Deserialize)]
        struct Files {
            #[serde(default)]
            download: Option<String>,
            #[serde(default)]
            icon: Option<String>,
        }

        async fn one_search(q: &str, category: Option<&str>, size: u32) -> Result<Vec<SearchItem>, String> {
            let mut url = format!(
                "https://open-vsx.org/api/-/search?size={size}&sortBy=downloadCount&sortOrder=desc"
            );
            if !q.trim().is_empty() {
                url.push_str(&format!("&query={}", urlencoding_encode(q)));
            }
            if let Some(cat) = category {
                url.push_str(&format!("&category={}", urlencoding_encode(cat)));
            }
            let resp = gloo_net::http::Request::get(&url)
                .header("Accept", "application/json")
                .header("User-Agent", "PusaPluginMarket/0.2")
                .send()
                .await
                .map_err(|e| format!("Open VSX 搜索失败：{e}"))?;
            if !(200..300).contains(&resp.status()) {
                return Err(format!("Open VSX 搜索失败（HTTP {}）", resp.status()));
            }
            let parsed: SearchResp = resp
                .json()
                .await
                .map_err(|e| format!("Open VSX 结果解析失败：{e}"))?;
            Ok(parsed.extensions)
        }

        fn to_web_card(item: SearchItem) -> Option<PluginCard> {
            if item.namespace.is_empty() || item.name.is_empty() {
                return None;
            }
            let id = format!("{}.{}", item.namespace, item.name);
            let name = item
                .display_name
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| id.clone());
            let description = item
                .description
                .filter(|d| !d.trim().is_empty())
                .unwrap_or_else(|| "（无描述）".into());
            let highlight = is_highlight_text(&name, &description);
            let homepage = item.url.unwrap_or_else(|| {
                format!(
                    "https://open-vsx.org/extension/{}/{}",
                    item.namespace, item.name
                )
            });
            let files = item.files;
            let download = files
                .as_ref()
                .and_then(|f| f.download.clone())
                .unwrap_or_default();
            let icon_url = files
                .as_ref()
                .and_then(|f| f.icon.clone())
                .filter(|s| !s.trim().is_empty())
                .or_else(|| item.icon.filter(|s| !s.trim().is_empty()))
                .map(|raw| {
                    let raw = raw.trim();
                    if raw.starts_with("https://") || raw.starts_with("http://") {
                        raw.to_string()
                    } else if let Some(ver) = item.version.as_deref().filter(|v| !v.is_empty()) {
                        format!(
                            "https://open-vsx.org/api/{}/{}/{ver}/file/{}",
                            item.namespace,
                            item.name,
                            raw.trim_start_matches("./")
                        )
                    } else {
                        raw.to_string()
                    }
                })
                .filter(|s| s.starts_with("https://") || s.starts_with("http://"));
            Some(PluginCard {
                install_key: format!("openvsx:{id}"),
                id,
                name,
                description,
                source_label: "Open VSX".into(),
                language: if highlight {
                    Some("语法高亮".into())
                } else {
                    None
                },
                stars: item.download_count,
                mentions: None,
                homepage,
                git_url: download,
                installed: false,
                highlight_priority: highlight,
                version: item.version,
                icon_url,
            })
        }

        let q = query.trim().to_string();
        let mut by_id = std::collections::HashMap::<String, PluginCard>::new();
        if q.is_empty() {
            for item in one_search("", Some("Programming Languages"), 24).await? {
                if let Some(mut c) = to_web_card(item) {
                    c.highlight_priority = true;
                    by_id.insert(c.id.clone(), c);
                }
            }
            for qq in ["syntax highlight", "grammar"] {
                if let Ok(items) = one_search(qq, None, 10).await {
                    for item in items {
                        if let Some(c) = to_web_card(item) {
                            by_id.entry(c.id.clone()).or_insert(c);
                        }
                    }
                }
            }
        } else {
            if let Ok(items) = one_search(&q, Some("Programming Languages"), 20).await {
                for item in items {
                    if let Some(mut c) = to_web_card(item) {
                        c.highlight_priority = true;
                        by_id.insert(c.id.clone(), c);
                    }
                }
            }
            for item in one_search(&q, None, 30).await? {
                if let Some(c) = to_web_card(item) {
                    by_id.entry(c.id.clone()).or_insert(c);
                }
            }
        }
        let mut out: Vec<PluginCard> = by_id.into_values().collect();
        out.sort_by(|a, b| {
            b.highlight_priority
                .cmp(&a.highlight_priority)
                .then_with(|| b.stars.unwrap_or(0).cmp(&a.stars.unwrap_or(0)))
        });
        Ok(out)
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        let _ = query;
        Err("当前构建不支持 Open VSX 搜索。".into())
    }
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn card_to_item(card: &PluginCard) -> crate::desktop::plugins::PluginItem {
    use crate::desktop::plugins::{PluginItem, PluginSource};
    PluginItem {
        id: card.id.clone(),
        name: card.name.clone(),
        description: card.description.clone(),
        source: PluginSource::OpenVsx,
        language: card.language.clone(),
        stars: card.stars,
        mentions: card.mentions,
        homepage: card.homepage.clone(),
        git_url: card.git_url.clone(),
        installed: card.installed,
        highlight_priority: card.highlight_priority,
        version: card.version.clone(),
        icon_url: card.icon_url.clone(),
    }
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
async fn run_install(card: PluginCard) -> Result<String, String> {
    let item = card_to_item(&card);
    let path = crate::desktop::plugins::install_plugin(&item).await?;
    Ok(format!("已安装到 {}", path.display()))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
async fn run_install(_card: PluginCard) -> Result<String, String> {
    Err("Web 端无法安装到本机 extensions 目录，请使用桌面版。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn open_installed_card(card: &PluginCard) -> Result<(), String> {
    crate::desktop::plugins::open_installed_plugin(&card.id)
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
fn open_installed_card(_card: &PluginCard) -> Result<(), String> {
    Err("Web 端无法打开本机目录，请使用桌面版。".into())
}

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
    Err("Web 端无法卸载本机扩展，请使用桌面版。".into())
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

fn language_meta_label(card: &PluginCard) -> Option<String> {
    let lang = card.language.as_deref()?.trim();
    if lang.is_empty() || lang == "语法高亮" || lang == "语法高亮优先" {
        None
    } else {
        Some(lang.to_string())
    }
}

fn format_meta(card: &PluginCard) -> String {
    let mut parts = vec![card.source_label.clone()];
    if let Some(lang) = language_meta_label(card) {
        parts.push(lang);
    }
    if let Some(ver) = &card.version {
        parts.push(format!("v{ver}"));
    }
    if let Some(stars) = card.stars {
        parts.push(format!("↓{stars}"));
    }
    parts.join(" · ")
}

/// 详情页元信息：不含来源徽章。
fn format_detail_meta(card: &PluginCard) -> String {
    let mut parts = Vec::new();
    if let Some(lang) = language_meta_label(card) {
        parts.push(lang);
    }
    if let Some(ver) = &card.version {
        parts.push(format!("v{ver}"));
    }
    if let Some(stars) = card.stars {
        parts.push(format!("下载 {stars}"));
    }
    parts.join(" · ")
}

fn detail_install_path(card: &PluginCard) -> String {
    format!("extensions/{}/", card.id)
}

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

fn plugin_initial(name: &str) -> String {
    name.chars()
        .find(|c| !c.is_whitespace())
        .map(|c| c.to_uppercase().collect::<String>())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "·".into())
}

#[component]
fn PluginGlyph(icon_url: ReadSignal<Option<String>>, name: String, large: bool) -> Element {
    let mut broken = use_signal(|| false);
    use_effect(move || {
        let _ = icon_url();
        broken.set(false);
    });
    let url = icon_url()
        .as_deref()
        .map(str::trim)
        .filter(|s| {
            !s.is_empty()
                && (s.starts_with("https://")
                    || s.starts_with("http://")
                    || s.starts_with("file://"))
        })
        .map(|s| s.to_string());
    let show_img = url.is_some() && !broken();
    let initial = plugin_initial(&name);
    let box_class = if large {
        if show_img {
            "ac-skills-detail-icon ac-plugin-icon-has-img"
        } else {
            "ac-skills-detail-icon ac-skill-icon-blue"
        }
    } else if show_img {
        "ac-skills-list-icon ac-plugin-icon-has-img"
    } else {
        "ac-skills-list-icon ac-skill-icon-blue"
    };
    rsx! {
        div { class: "{box_class}",
            if let Some(src) = url.as_ref() {
                if !broken() {
                    img {
                        class: "ac-plugin-icon-img",
                        src: "{src}",
                        alt: "",
                        onerror: move |_| broken.set(true),
                    }
                } else {
                    span { class: "ac-plugin-icon-letter", "{initial}" }
                }
            } else {
                span { class: "ac-plugin-icon-letter", "{initial}" }
            }
        }
    }
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
            #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
            {
                if crate::desktop::plugins::is_vscode_extension_install(&id) {
                    return;
                }
            }
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
                                PluginGlyph {
                                    icon_url: card.icon_url.clone(),
                                    name: name.clone(),
                                    large: true,
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
                                                        #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
                                                        {
                                                            if !crate::desktop::plugins::is_vscode_extension_install(
                                                                &install_id,
                                                            ) {
                                                                spawn_ensure_smart_ui_after_install(
                                                                    install_id,
                                                                    toast,
                                                                );
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        toast.error(format!("安装失败：{e}"));
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
                                            "安装中"
                                        } else {
                                            "安装"
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
                    h3 { "扩展" }
                    p { "该扩展已不在当前列表中，可重新搜索后打开。" }
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
    let mut plugin_view = use_signal(|| PluginView::Installed);
    let mut search = use_signal(String::new);
    let mut items = use_signal(Vec::<PluginCard>::new);
    let mut search_items = use_signal(Vec::<PluginCard>::new);
    let mut search_query_done = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let mut search_loading = use_signal(|| false);
    let mut loaded_source = use_signal(|| None::<PluginListSource>);
    let mut load_epoch = use_signal(|| 0_u64);
    let mut search_trigger = use_signal(|| 0_u64);
    let mut error = use_signal(|| None::<String>);
    let mut search_error = use_signal(|| None::<String>);

    let active_source = use_memo(move || {
        if plugin_view() == PluginView::Installed {
            Some(PluginListSource::Installed)
        } else {
            Some(PluginListSource::Market)
        }
    });

    use_effect(move || {
        let tick = refresh_tick();
        if tick == 0 {
            return;
        }
        loaded_source.set(None);
        load_epoch.with_mut(|n| *n += 1);
    });

    use_effect(move || {
        let cat = catalog();
        search_items.with_mut(|list| {
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
        // 市场页若正在展示关键词搜索结果，不覆盖为默认榜单。
        if source == PluginListSource::Market
            && search_query_done()
                .as_ref()
                .map(|q| !q.trim().is_empty())
                .unwrap_or(false)
        {
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
        let trigger = search_trigger();
        if trigger == 0 {
            return;
        }
        let Some(query) = search_query_done() else {
            return;
        };
        if query.trim().is_empty() {
            return;
        }
        search_loading.set(true);
        search_error.set(None);
        spawn(async move {
            match run_openvsx_search(query).await {
                Ok(list) => {
                    sync_catalog(catalog, &list);
                    search_items.set(list);
                    search_error.set(None);
                }
                Err(e) => {
                    search_items.set(Vec::new());
                    search_error.set(Some(e));
                }
            }
            search_loading.set(false);
        });
    });

    let on_market = use_memo(move || plugin_view() == PluginView::Market);
    let on_installed = use_memo(move || plugin_view() == PluginView::Installed);

    let showing_search = use_memo(move || {
        on_market()
            && search_query_done()
                .as_ref()
                .map(|q| !q.trim().is_empty())
                .unwrap_or(false)
    });

    let filtered = use_memo(move || {
        if showing_search() {
            return search_items();
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
                    || card.id.to_lowercase().contains(&q)
                    || card
                        .language
                        .as_ref()
                        .map(|l| l.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>()
    });

    let list_loading = if showing_search() {
        search_loading()
    } else {
        loading()
    };
    let list_error = if showing_search() {
        search_error()
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
                    placeholder: if on_installed() {
                        "筛选已安装扩展"
                    } else {
                        "搜索 Open VSX（Enter）"
                    },
                    oninput: move |e| {
                        let v = e.value();
                        search.set(v.clone());
                        if on_market() && v.trim().is_empty() {
                            search_query_done.set(None);
                            search_items.set(Vec::new());
                            search_error.set(None);
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
                        if q.is_empty() {
                            search_query_done.set(None);
                            search_items.set(Vec::new());
                            search_error.set(None);
                            loaded_source.set(None);
                            load_epoch.with_mut(|n| *n += 1);
                            return;
                        }
                        search_query_done.set(Some(q));
                        search_trigger.with_mut(|n| *n += 1);
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
                    "我的扩展"
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
                        loaded_source.set(None);
                        load_epoch.with_mut(|n| *n += 1);
                    },
                    "扩展市场"
                }
            }
            if !plugin_available() {
                div { class: "ac-sidebar-skills-banner is-warn",
                    "桌面端可从 Open VSX 安装 .vsix 到工作区 extensions/；Web 可浏览与搜索，安装请用桌面版。"
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
                        if on_installed() {
                            if search().trim().is_empty() {
                                "暂无已安装扩展，去「扩展市场」浏览。"
                            } else {
                                "没有匹配的扩展"
                            }
                        } else if showing_search() {
                            "没有匹配的 Open VSX 扩展"
                        } else {
                            "暂无扩展，可输入关键词后按 Enter 搜索"
                        }
                    }
                } else if !filtered().is_empty() {
                    div { class: "ac-skills-section-header",
                        span {
                            if on_installed() {
                                "我的扩展"
                            } else if showing_search() {
                                "Open VSX 搜索"
                            } else {
                                "Open VSX"
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
                                        PluginGlyph {
                                            icon_url: card.icon_url.clone(),
                                            name: card.name.clone(),
                                            large: false,
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
