//! 应用市场：我的应用、官方搜索（含 GitHub）、AI 搜索，克隆到工作区 `applications/`。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use super::files::workspace_root;

/// 预留：日后可改为远程官方目录 URL；当前为空则用内置榜单。
const OFFICIAL_CATALOG_URL: Option<&str> = None;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: PluginSource,
    pub language: Option<String>,
    pub stars: Option<u64>,
    pub mentions: Option<u64>,
    pub homepage: String,
    pub git_url: String,
    pub installed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSource {
    Official,
    GitHub,
    Ai,
    Local,
}

impl PluginSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::GitHub => "github",
            Self::Ai => "ai",
            Self::Local => "local",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Official => "官方",
            Self::GitHub => "GitHub",
            Self::Ai => "AI",
            Self::Local => "本地",
        }
    }

    fn from_meta(raw: &str) -> Self {
        match raw.trim() {
            "official" => Self::Official,
            "github" => Self::GitHub,
            "ai" => Self::Ai,
            _ => Self::Local,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OfficialCatalogFile {
    #[serde(default)]
    items: Vec<OfficialCatalogEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OfficialCatalogEntry {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    git_url: String,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    language: Option<String>,
}

/// 解析「应用库」目录（容纳各已安装应用的父目录）。
///
/// - 工作区本身是 `…/applications/{app}`：返回上一级 `applications/`（打开单个应用后仍能列出兄弟应用）
/// - 工作区本身叫 `applications`（或旧名 `application`）：直接返回它
/// - 工作区下存在 `applications/`（或旧名 `application/`）：返回该目录
/// - 否则回退为 `工作区/applications`（后续 `ensure` 会创建）
///
/// 注意：不向上穿越到文件系统根去找 `applications/`，避免在 macOS 上误命中系统 `/Applications`。
pub fn application_root() -> PathBuf {
    let ws = workspace_root();
    resolve_application_root(&ws)
}

fn is_applications_dirname(name: &str) -> bool {
    name.eq_ignore_ascii_case("applications") || name.eq_ignore_ascii_case("application")
}

fn resolve_application_root(ws: &Path) -> PathBuf {
    if ws
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(is_applications_dirname)
    {
        return ws.to_path_buf();
    }
    if let Some(parent) = ws.parent() {
        if parent
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(is_applications_dirname)
        {
            return parent.to_path_buf();
        }
    }
    let nested = ws.join("applications");
    if nested.is_dir() {
        return nested;
    }
    let legacy = ws.join("application");
    if legacy.is_dir() {
        return legacy;
    }
    nested
}

pub fn ensure_application_root() -> Result<PathBuf, String> {
    let root = application_root();
    fs::create_dir_all(&root).map_err(|e| format!("无法创建 applications 目录：{e}"))?;
    Ok(root)
}

/// 扫描 `applications/` 下已安装的子目录名。
pub fn list_installed_ids() -> Vec<String> {
    let root = application_root();
    let Ok(entries) = fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with('.') || name.eq_ignore_ascii_case("readme.md") {
            continue;
        }
        ids.push(name.to_string());
    }
    ids.sort();
    ids
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct InstallMeta {
    id: String,
    name: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    git_url: String,
    #[serde(default)]
    homepage: String,
    #[serde(default)]
    description: String,
}

fn read_install_meta(dir: &Path) -> Option<InstallMeta> {
    let path = dir.join(".pusa-plugin.json");
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// 扫描 `applications/` 下已安装应用，优先读取 `.pusa-plugin.json`。
pub fn list_installed_plugins() -> Vec<PluginItem> {
    let root = application_root();
    let Ok(entries) = fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut items = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(dirname) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if dirname.starts_with('.') || dirname.eq_ignore_ascii_case("readme.md") {
            continue;
        }
        // 以目录名为 id，保证打开/卸载路径与「官方榜单已安装」标记一致。
        let id = dirname.to_string();
        let meta = read_install_meta(&path);
        let (name, description, source, git_url, homepage) = if let Some(m) = meta {
            let name = if m.name.trim().is_empty() {
                dirname.to_string()
            } else {
                m.name
            };
            let description = if m.description.trim().is_empty() {
                format!("applications/{dirname}/")
            } else {
                m.description
            };
            let homepage = if m.homepage.trim().is_empty() {
                m.git_url.trim_end_matches(".git").to_string()
            } else {
                m.homepage
            };
            (
                name,
                description,
                PluginSource::from_meta(&m.source),
                m.git_url,
                homepage,
            )
        } else {
            (
                dirname.to_string(),
                format!("applications/{dirname}/"),
                PluginSource::Local,
                String::new(),
                String::new(),
            )
        };
        items.push(PluginItem {
            id,
            name,
            description,
            source,
            language: None,
            stars: None,
            mentions: None,
            homepage,
            git_url,
            installed: true,
        });
    }
    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    items
}

fn validate_plugin_id(id: &str) -> Result<String, String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("插件 id 无效".into());
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err("插件 id 非法".into());
    }
    Ok(id.to_string())
}

/// 在系统文件管理器中打开已安装插件目录。
pub fn open_installed_plugin(id: &str) -> Result<(), String> {
    let id = validate_plugin_id(id)?;
    let path = application_root().join(&id);
    if !path.is_dir() {
        return Err(format!("未找到已安装目录：{}", path.display()));
    }
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .arg(&path)
            .status()
            .map_err(|e| format!("打开目录失败：{e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("打开目录失败".into())
        }
    }
    #[cfg(target_os = "windows")]
    {
        let status = Command::new("explorer")
            .arg(&path)
            .status()
            .map_err(|e| format!("打开目录失败：{e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("打开目录失败".into())
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let status = Command::new("xdg-open")
            .arg(&path)
            .status()
            .map_err(|e| format!("打开目录失败：{e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("打开目录失败".into())
        }
    }
}

/// 卸载 `applications/{id}`（删除目录）。
pub fn uninstall_plugin(id: &str) -> Result<(), String> {
    let id = validate_plugin_id(id)?;
    let root = application_root();
    let target = root.join(&id);
    if !target.is_dir() {
        return Err(format!("未找到已安装目录：{}", target.display()));
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("无法解析 applications 目录：{e}"))?;
    let canonical_target = target
        .canonicalize()
        .map_err(|e| format!("无法解析插件目录：{e}"))?;
    if !canonical_target.starts_with(&canonical_root) || canonical_target == canonical_root {
        return Err("拒绝卸载：路径不在 applications/ 下".into());
    }
    fs::remove_dir_all(&canonical_target).map_err(|e| format!("卸载失败：{e}"))
}

fn sanitize_dir_name(raw: &str) -> String {
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

fn mark_installed(items: &mut [PluginItem], installed: &[String]) {
    for item in items {
        item.installed = installed.iter().any(|id| id == &item.id);
    }
}

/// 内置官方榜单（无远程目录时使用）。
fn builtin_official_catalog() -> Vec<OfficialCatalogEntry> {
    vec![
        OfficialCatalogEntry {
            id: "servers".into(),
            name: "MCP Servers".into(),
            description: "Model Context Protocol 官方参考服务器集合。".into(),
            git_url: "https://github.com/modelcontextprotocol/servers.git".into(),
            homepage: Some("https://github.com/modelcontextprotocol/servers".into()),
            language: Some("TypeScript".into()),
        },
        OfficialCatalogEntry {
            id: "codex".into(),
            name: "OpenAI Codex CLI".into(),
            description: "轻量终端编程 Agent。".into(),
            git_url: "https://github.com/openai/codex.git".into(),
            homepage: Some("https://github.com/openai/codex".into()),
            language: Some("Rust".into()),
        },
        OfficialCatalogEntry {
            id: "goose".into(),
            name: "goose".into(),
            description: "本地优先的开源 AI Agent 框架。".into(),
            git_url: "https://github.com/block/goose.git".into(),
            homepage: Some("https://github.com/block/goose".into()),
            language: Some("Rust".into()),
        },
        OfficialCatalogEntry {
            id: "aider".into(),
            name: "aider".into(),
            description: "终端里的 AI 结对编程工具。".into(),
            git_url: "https://github.com/Aider-AI/aider.git".into(),
            homepage: Some("https://github.com/Aider-AI/aider".into()),
            language: Some("Python".into()),
        },
        OfficialCatalogEntry {
            id: "open-webui".into(),
            name: "Open WebUI".into(),
            description: "自托管 LLM Web 界面。".into(),
            git_url: "https://github.com/open-webui/open-webui.git".into(),
            homepage: Some("https://github.com/open-webui/open-webui".into()),
            language: Some("Python".into()),
        },
        OfficialCatalogEntry {
            id: "anything-llm".into(),
            name: "AnythingLLM".into(),
            description: "全栈私有知识库 / Agent 应用。".into(),
            git_url: "https://github.com/Mintplex-Labs/anything-llm.git".into(),
            homepage: Some("https://github.com/Mintplex-Labs/anything-llm".into()),
            language: Some("JavaScript".into()),
        },
    ]
}

fn catalog_to_items(entries: Vec<OfficialCatalogEntry>, installed: &[String]) -> Vec<PluginItem> {
    let mut items: Vec<PluginItem> = entries
        .into_iter()
        .map(|e| {
            let id = sanitize_dir_name(if e.id.is_empty() { &e.name } else { &e.id });
            let homepage = e
                .homepage
                .unwrap_or_else(|| e.git_url.trim_end_matches(".git").to_string());
            PluginItem {
                id,
                name: e.name,
                description: e.description,
                source: PluginSource::Official,
                language: e.language,
                stars: None,
                mentions: None,
                homepage,
                git_url: e.git_url,
                installed: false,
            }
        })
        .collect();
    mark_installed(&mut items, installed);
    items
}

async fn fetch_remote_official_catalog() -> Result<Vec<OfficialCatalogEntry>, String> {
    let Some(url) = OFFICIAL_CATALOG_URL else {
        return Ok(builtin_official_catalog());
    };
    let client = reqwest::Client::builder()
        .user_agent("PusaPluginMarket/0.1")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("官方目录 HTTP {}", resp.status()));
    }
    let body = resp
        .json::<OfficialCatalogFile>()
        .await
        .map_err(|e| format!("官方目录 JSON 解析失败：{e}"))?;
    if body.items.is_empty() {
        return Err("官方目录为空".into());
    }
    Ok(body.items)
}

pub async fn load_official_catalog() -> Result<Vec<PluginItem>, String> {
    let installed = list_installed_ids();
    let entries = match fetch_remote_official_catalog().await {
        Ok(items) => items,
        Err(err) => {
            // 远程失败时回落到内置榜单，并附带错误信息由调用方决定是否展示。
            let _ = err;
            builtin_official_catalog()
        }
    };
    Ok(catalog_to_items(entries, &installed))
}

#[derive(Clone, Debug, Deserialize)]
struct GitHubSearchResponse {
    #[serde(default)]
    items: Vec<GitHubRepoItem>,
}

#[derive(Clone, Debug, Deserialize)]
struct GitHubRepoItem {
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

fn github_token_from_env() -> Option<String> {
    for key in ["GITHUB_TOKEN", "GH_TOKEN"] {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

fn map_github_http_error(status: u16, body: &str) -> String {
    match status {
        401 => "GitHub 认证失败，请检查环境变量 GITHUB_TOKEN / GH_TOKEN。".into(),
        403 | 429 => {
            "GitHub API 请求过于频繁或被限流，请稍后再试（可设置环境变量 GITHUB_TOKEN 提高限额）。"
                .into()
        }
        422 => "GitHub 搜索关键词无效，请换一个词再试。".into(),
        _ => {
            let detail = body.trim();
            if detail.is_empty() {
                format!("GitHub 搜索失败（HTTP {status}）")
            } else if let Ok(v) = serde_json::from_str::<serde_json::Value>(detail) {
                if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
                    format!("GitHub 搜索失败：{msg}")
                } else {
                    format!("GitHub 搜索失败（HTTP {status}）")
                }
            } else {
                format!("GitHub 搜索失败（HTTP {status}）")
            }
        }
    }
}

/// 通过 GitHub Search API 检索公开仓库（官方榜单页回车搜索）。
pub async fn search_github_repositories(query: &str) -> Result<Vec<PluginItem>, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::Client::builder()
        .user_agent("PusaPluginMarket/0.1")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败：{e}"))?;

    let mut builder = client
        .get("https://api.github.com/search/repositories")
        .query(&[("q", q), ("per_page", "30"), ("sort", "stars")]);
    if let Some(token) = github_token_from_env() {
        builder = builder.bearer_auth(token);
    }

    let resp = builder
        .send()
        .await
        .map_err(|e| format!("GitHub 搜索请求失败：{e}"))?;
    let status = resp.status().as_u16();
    if !(200..300).contains(&status) {
        let body = resp.text().await.unwrap_or_default();
        return Err(map_github_http_error(status, &body));
    }

    let parsed = resp
        .json::<GitHubSearchResponse>()
        .await
        .map_err(|e| format!("GitHub 搜索结果解析失败：{e}"))?;

    let installed = list_installed_ids();
    let mut items: Vec<PluginItem> = parsed
        .items
        .into_iter()
        .filter(|r| !r.clone_url.trim().is_empty() || !r.html_url.trim().is_empty())
        .map(|r| {
            let id = sanitize_dir_name(if r.name.is_empty() {
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
            PluginItem {
                id,
                name,
                description,
                source: PluginSource::GitHub,
                language: r.language,
                stars: Some(r.stargazers_count),
                mentions: None,
                homepage,
                git_url,
                installed: false,
            }
        })
        .collect();
    mark_installed(&mut items, &installed);
    Ok(items)
}

/// 调用 RuntimeContext LLM，按关键词检索相关开源应用。
pub async fn search_apps_with_ai(query: &str, model: &str) -> Result<Vec<PluginItem>, String> {
    let dtos = super::agent::runtime_ctx()
        .plugin_ai_search(query, model)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("openai_api_key_missing") {
                "尚未配置 LLM API Key，请先在设置中配置后再使用 AI 搜索。".into()
            } else {
                msg
            }
        })?;
    let installed = list_installed_ids();
    let mut items: Vec<PluginItem> = dtos
        .into_iter()
        .map(|d| {
            let homepage = d
                .homepage
                .unwrap_or_else(|| d.git_url.trim_end_matches(".git").to_string());
            PluginItem {
                id: sanitize_dir_name(&d.id),
                name: d.name,
                description: d.description,
                source: PluginSource::Ai,
                language: d.language,
                stars: None,
                mentions: None,
                homepage,
                git_url: d.git_url,
                installed: false,
            }
        })
        .collect();
    mark_installed(&mut items, &installed);
    Ok(items)
}

/// `git clone --depth 1` 到 `applications/{id}`。
pub fn install_plugin(item: &PluginItem) -> Result<PathBuf, String> {
    let root = ensure_application_root()?;
    let target = root.join(&item.id);
    if target.exists() {
        return Err(format!("已存在：{}", target.display()));
    }

    // 先确认 git 可用
    let git_ok = Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !git_ok {
        return Err("未找到 git，请先安装 Git 后再试。".into());
    }

    let output = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            &item.git_url,
            target.to_str().ok_or("目标路径无效")?,
        ])
        .output()
        .map_err(|e| format!("启动 git clone 失败：{e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            stdout.trim().to_string()
        };
        // 清理半成品目录
        if target.exists() {
            let _ = fs::remove_dir_all(&target);
        }
        return Err(if detail.is_empty() {
            "git clone 失败".into()
        } else {
            format!("git clone 失败：{detail}")
        });
    }

    write_install_meta(&target, item)?;
    Ok(target)
}

fn write_install_meta(target: &Path, item: &PluginItem) -> Result<(), String> {
    let meta = InstallMeta {
        id: item.id.clone(),
        name: item.name.clone(),
        source: item.source.as_str().into(),
        git_url: item.git_url.clone(),
        homepage: item.homepage.clone(),
        description: item.description.clone(),
    };
    let path = target.join(".pusa-plugin.json");
    let raw = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| format!("写入安装元数据失败：{e}"))
}

/// 本地新建插件目录名：保留 Unicode 字母数字，空白变 `-`，去掉路径不安全字符。
fn sanitize_local_app_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            out.push(ch);
        } else if ch.is_whitespace() {
            if !out.ends_with('-') {
                out.push('-');
            }
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "my-app".into()
    } else {
        out
    }
}

/// 在工作区 `applications/{id}/` 脚手架本地应用（README + `.pusa-plugin.json`）。
pub fn scaffold_local_application(display_name: &str) -> Result<PathBuf, String> {
    let name = display_name.trim();
    let name = if name.is_empty() { "my-app" } else { name };
    let id = sanitize_local_app_name(name);
    let root = ensure_application_root()?;
    let target = root.join(&id);
    if target.exists() {
        return Err(format!("已存在：{}", target.display()));
    }
    fs::create_dir_all(&target).map_err(|e| format!("创建应用目录失败：{e}"))?;

    let readme = format!(
        "# {name}\n\n本地脚手架应用，位于工作区 `applications/{id}/`。\n\n包格式与加载逻辑见仓库 `applications/README.md`。\n"
    );
    fs::write(target.join("README.md"), readme).map_err(|e| format!("写入 README 失败：{e}"))?;

    let meta = InstallMeta {
        id: id.clone(),
        name: name.to_string(),
        source: "local".into(),
        git_url: String::new(),
        homepage: String::new(),
        description: format!("本地脚手架应用，位于工作区 applications/{id}/"),
    };
    let meta_path = target.join(".pusa-plugin.json");
    let raw = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    if let Err(e) = fs::write(&meta_path, raw) {
        let _ = fs::remove_dir_all(&target);
        return Err(format!("写入安装元数据失败：{e}"));
    }
    Ok(target.canonicalize().unwrap_or(target))
}

const SMART_UI_FILENAME: &str = ".pusa-smart-ui.json";
const SMART_UI_SCAN_BUDGET: usize = 18_000;

/// 已安装应用目录（绝对路径优先）。
pub fn plugin_dir(id: &str) -> Result<PathBuf, String> {
    let id = validate_plugin_id(id)?;
    let path = application_root().join(&id);
    if !path.is_dir() {
        return Err(format!("未找到已安装目录：{}", path.display()));
    }
    Ok(path.canonicalize().unwrap_or(path))
}

fn smart_ui_path(dir: &Path) -> PathBuf {
    dir.join(SMART_UI_FILENAME)
}

/// 读取已缓存的智能 UI（若存在）。
pub fn load_smart_ui(id: &str) -> Result<Option<protocol::PluginSmartUiDto>, String> {
    let dir = plugin_dir(id)?;
    let path = smart_ui_path(&dir);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("读取智能 UI 失败：{e}"))?;
    match serde_json::from_str::<protocol::PluginSmartUiDto>(&raw) {
        Ok(dto) if !dto.actions.is_empty() => {
            let fixed = apply_package_manager_fixes(&dir, dto);
            // 静默写回纠正后的命令，避免每次仍显示 yarn。
            let _ = save_smart_ui(&dir, &fixed);
            Ok(Some(fixed))
        }
        // 损坏或空缓存：当作未缓存，触发重新生成。
        Ok(_) | Err(_) => Ok(None),
    }
}

/// 是否已有有效的 `.pusa-smart-ui.json` 缓存。
pub fn has_smart_ui_cache(id: &str) -> bool {
    matches!(load_smart_ui(id), Ok(Some(_)))
}

/// 尚未缓存智能 UI 的已安装应用 id 列表。
pub fn list_ids_missing_smart_ui() -> Vec<String> {
    list_installed_ids()
        .into_iter()
        .filter(|id| !has_smart_ui_cache(id))
        .collect()
}

/// 有缓存则读取，否则扫描项目并用 LLM 生成后写入 `applications/{id}/.pusa-smart-ui.json`。
pub async fn ensure_smart_ui(
    id: &str,
    model: &str,
) -> Result<protocol::PluginSmartUiDto, String> {
    if let Some(dto) = load_smart_ui(id)? {
        return Ok(dto);
    }
    generate_smart_ui(id, model).await
}

/// 为所有缺少缓存的已安装应用依次生成智能 UI。
/// 返回 `(成功数, 失败列表)`；若无 API Key 等致命错误会提前停止。
pub async fn ensure_missing_smart_uis(
    model: &str,
) -> Result<(usize, Vec<(String, String)>), String> {
    let missing = list_ids_missing_smart_ui();
    if missing.is_empty() {
        return Ok((0, Vec::new()));
    }
    let mut ok = 0usize;
    let mut errs = Vec::new();
    for id in missing {
        match ensure_smart_ui(&id, model).await {
            Ok(_) => ok += 1,
            Err(e) => {
                let fatal = e.contains("API Key") || e.contains("openai_api_key_missing");
                errs.push((id, e));
                if fatal {
                    break;
                }
            }
        }
    }
    Ok((ok, errs))
}

/// 将智能 UI 原子写入 `.pusa-smart-ui.json`（先写临时文件再 rename）。
fn save_smart_ui(dir: &Path, dto: &protocol::PluginSmartUiDto) -> Result<(), String> {
    let path = smart_ui_path(dir);
    let raw = serde_json::to_string_pretty(dto).map_err(|e| e.to_string())?;
    let tmp = dir.join(format!(
        "{}.tmp.{}",
        SMART_UI_FILENAME,
        std::process::id()
    ));
    fs::write(&tmp, &raw).map_err(|e| format!("写入智能 UI 缓存失败：{e}"))?;
    // Windows 上 rename 不能覆盖已有目标，先尝试删除旧文件。
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
    fs::rename(&tmp, &path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("保存智能 UI 缓存失败：{e}")
    })?;
    Ok(())
}

fn append_scan_file(out: &mut String, root: &Path, rel: &str, max_chars: usize) {
    if out.len() >= SMART_UI_SCAN_BUDGET {
        return;
    }
    let path = root.join(rel);
    let Ok(meta) = fs::metadata(&path) else {
        return;
    };
    if !meta.is_file() || meta.len() > 400_000 {
        return;
    }
    let Ok(raw) = fs::read_to_string(&path) else {
        return;
    };
    let remain = SMART_UI_SCAN_BUDGET.saturating_sub(out.len());
    let take = remain.min(max_chars);
    if take == 0 {
        return;
    }
    let slice: String = raw.chars().take(take).collect();
    out.push_str(&format!("\n### {rel}\n```\n{slice}\n```\n"));
}

fn append_scripts_dir(out: &mut String, root: &Path) {
    let scripts = root.join("scripts");
    let Ok(entries) = fs::read_dir(&scripts) else {
        return;
    };
    out.push_str("\n### scripts/\n");
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        names.push(name.to_string());
        if names.len() >= 24 {
            break;
        }
    }
    names.sort();
    for name in &names {
        out.push_str(&format!("- {name}\n"));
        if out.len() >= SMART_UI_SCAN_BUDGET {
            break;
        }
        let rel = format!("scripts/{name}");
        append_scan_file(out, root, &rel, 800);
    }
}

/// 根据锁文件 / workspace 清单推断 JS 包管理器（优先级：pnpm > yarn > npm > bun）。
fn detect_js_package_manager(root: &Path) -> Option<&'static str> {
    let has = |name: &str| root.join(name).is_file();
    let has_dir_file = |name: &str| root.join(name).exists();
    if has("pnpm-lock.yaml") || has_dir_file("pnpm-workspace.yaml") {
        return Some("pnpm");
    }
    if has("bun.lockb") || has("bun.lock") {
        return Some("bun");
    }
    // package.json 的 packageManager 字段优先于裸 yarn.lock（避免 yarn1 误装 berry/pnpm 项目）。
    if let Ok(raw) = fs::read_to_string(root.join("package.json")) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(pm) = v.get("packageManager").and_then(|x| x.as_str()) {
                let pm = pm.to_ascii_lowercase();
                if pm.starts_with("pnpm@") {
                    return Some("pnpm");
                }
                if pm.starts_with("yarn@") {
                    return Some("yarn");
                }
                if pm.starts_with("npm@") {
                    return Some("npm");
                }
                if pm.starts_with("bun@") {
                    return Some("bun");
                }
            }
        }
    }
    if has("yarn.lock") {
        return Some("yarn");
    }
    if has("package-lock.json") {
        return Some("npm");
    }
    if root.join("package.json").is_file() {
        return Some("npm");
    }
    None
}

fn append_package_manager_hint(out: &mut String, root: &Path) {
    let Some(pm) = detect_js_package_manager(root) else {
        return;
    };
    out.push_str("\n### 包管理器提示（辅助，若与 README 冲突以 README 为准）\n");
    out.push_str(&format!("锁文件推断：{pm}\n"));
    match pm {
        "pnpm" => {
            out.push_str(
                "- 若 README「本地开发」写的是 pnpm，安装用 pnpm install\n\
                 - 不要仅因存在 yarn.lock 残留就改用 yarn\n",
            );
        }
        "yarn" => {
            out.push_str("- 若 README 未另有说明，本地开发可用 yarn install\n");
        }
        "bun" => {
            out.push_str("- 若 README 未另有说明，本地开发可用 bun install\n");
        }
        _ => {
            out.push_str("- 若 README 未另有说明，本地开发可用 npm install\n");
        }
    }
    out.push_str("- 锁文件/工作区清单：");
    let mut flags = Vec::new();
    for name in [
        "pnpm-lock.yaml",
        "pnpm-workspace.yaml",
        "yarn.lock",
        "package-lock.json",
        "bun.lockb",
        "bun.lock",
    ] {
        if root.join(name).exists() {
            flags.push(name);
        }
    }
    if flags.is_empty() {
        out.push_str("（无）\n");
    } else {
        out.push_str(&format!("{}\n", flags.join(", ")));
    }
}

/// 去掉模型常包的多余 `sh -c '…'` / `bash -c "…"` 外壳（终端已在交互 shell 中执行）。
pub fn unwrap_shell_c_command(command: &str) -> String {
    let t = command.trim();
    for (prefix, quote) in [
        ("sh -c ", '\''),
        ("bash -c ", '\''),
        ("sh -c ", '"'),
        ("bash -c ", '"'),
    ] {
        if let Some(rest) = t.strip_prefix(prefix) {
            let rest = rest.trim();
            if rest.len() >= 2
                && rest.starts_with(quote)
                && rest.ends_with(quote)
            {
                return rest[1..rest.len() - 1].to_string();
            }
        }
    }
    t.to_string()
}

/// 扫描插件目录关键清单，拼成 LLM 上下文。
///
/// **主依据是 README.md**（安装 / 开发 / 启动命令以文档为准）；
/// package.json、锁文件等仅作补充，避免模型只看 workspaces 误判包管理器。
pub fn scan_plugin_project_context(id: &str) -> Result<String, String> {
    let dir = plugin_dir(id)?;
    let mut out = format!("项目目录：applications/{id}/\n");
    out.push_str(
        "生成规则：以 README 中的安装与开发说明为主提取按钮命令；\
         其它清单文件仅在 README 未写清时作补充。\n",
    );

    if let Ok(entries) = fs::read_dir(&dir) {
        let mut names = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if name.starts_with('.') && name != ".env.example" {
                continue;
            }
            names.push(name.to_string());
        }
        names.sort();
        out.push_str("顶层条目：\n");
        for name in names.iter().take(60) {
            out.push_str(&format!("- {name}\n"));
        }
    }

    // —— 主依据：README（尽量给足篇幅）——
    out.push_str("\n## 【主依据】README（优先采用其中的安装/开发/启动命令）\n");
    let mut readme_ok = false;
    for rel in ["README.md", "README", "readme.md", "docs/README.md"] {
        let before = out.len();
        append_scan_file(&mut out, &dir, rel, 10_000);
        if out.len() > before {
            readme_ok = true;
            break;
        }
    }
    if !readme_ok {
        out.push_str("（未找到 README，将回退到辅助清单推断。）\n");
    }

    // —— 辅助：清单 / 锁文件 / 脚本（README 未覆盖时再用）——
    out.push_str("\n## 【辅助】清单与锁文件（仅补充 README 未写明的细节）\n");
    append_package_manager_hint(&mut out, &dir);
    for rel in [
        "package.json",
        "pnpm-workspace.yaml",
        "Cargo.toml",
        "Makefile",
        "makefile",
        "justfile",
        "Justfile",
        "pyproject.toml",
        "requirements.txt",
        "go.mod",
        "composer.json",
        "Gemfile",
        "Dockerfile",
        "docker-compose.yml",
        "docker-compose.yaml",
    ] {
        append_scan_file(&mut out, &dir, rel, 2_000);
    }
    append_scripts_dir(&mut out, &dir);

    for rel in [
        "main.py",
        "app.py",
        "manage.py",
        "src/main.rs",
        "bin/main.rs",
        "index.js",
        "index.ts",
    ] {
        append_scan_file(&mut out, &dir, rel, 400);
    }

    if out.len() < 80 {
        return Err("项目目录几乎为空，无法生成智能 UI。".into());
    }
    Ok(out)
}

/// 扫描项目并用 LLM 生成智能 UI，立即写入 `.pusa-smart-ui.json`。
pub async fn generate_smart_ui(
    id: &str,
    model: &str,
) -> Result<protocol::PluginSmartUiDto, String> {
    let dir = plugin_dir(id)?;
    let context = scan_plugin_project_context(id)?;
    let mut dto = super::agent::runtime_ctx()
        .plugin_smart_ui_generate(&context, model)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("openai_api_key_missing") {
                "尚未配置 LLM API Key，请先在设置中配置后再使用智能 UI。".into()
            } else {
                msg
            }
        })?;
    if dto.generated_at.is_none() {
        dto.generated_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        );
    }
    dto = apply_package_manager_fixes(&dir, dto);
    save_smart_ui(&dir, &dto)?;
    // 回读确认缓存落盘，避免「界面有内容、磁盘无文件」。
    if !smart_ui_path(&dir).is_file() {
        return Err("智能 UI 已生成，但写入缓存文件失败。".into());
    }
    Ok(dto)
}

/// 若能检测包管理器，把误用的「裸装依赖」命令改成正确工具。
/// 不改写带参数的命令（如 `npm install -g pkg`），以免覆盖 README 原意。
fn rewrite_install_command_for_pm(command: &str, pm: &str) -> Option<String> {
    let t = command.trim();
    let lower = t.to_ascii_lowercase();
    let is_bare_install = matches!(
        lower.as_str(),
        "yarn"
            | "yarn install"
            | "npm install"
            | "npm i"
            | "pnpm install"
            | "pnpm i"
            | "bun install"
            | "bun i"
    );
    if !is_bare_install {
        return None;
    }
    Some(match pm {
        "pnpm" => "pnpm install".into(),
        "yarn" => "yarn install".into(),
        "bun" => "bun install".into(),
        _ => "npm install".into(),
    })
}

fn yarn_script_to_pnpm(rest: &str) -> String {
    let rest = rest.trim();
    if rest.is_empty() {
        return "pnpm install".into();
    }
    if rest.starts_with("workspace ") {
        // yarn workspace <pkg> <cmd> → pnpm --filter <pkg> <cmd>
        let mut parts = rest.split_whitespace();
        let _ = parts.next(); // workspace
        let pkg = parts.next().unwrap_or("");
        let cmd: Vec<&str> = parts.collect();
        if pkg.is_empty() {
            return format!("pnpm {rest}");
        }
        if cmd.is_empty() {
            return format!("pnpm --filter {pkg} run");
        }
        return format!("pnpm --filter {pkg} {}", cmd.join(" "));
    }
    // yarn <script> / yarn run <script>
    let rest = rest.strip_prefix("run ").unwrap_or(rest);
    format!("pnpm run {rest}")
}

fn apply_package_manager_fixes(
    dir: &Path,
    mut dto: protocol::PluginSmartUiDto,
) -> protocol::PluginSmartUiDto {
    let pm = detect_js_package_manager(dir);
    for action in &mut dto.actions {
        action.command = unwrap_shell_c_command(&action.command);
        if let Some(pm) = pm {
            if let Some(fixed) = rewrite_install_command_for_pm(&action.command, pm) {
                action.command = fixed;
            } else if pm == "pnpm" {
                let cmd = action.command.trim();
                if let Some(rest) = cmd.strip_prefix("yarn ") {
                    action.command = yarn_script_to_pnpm(rest);
                } else if cmd == "yarn" {
                    action.command = "pnpm install".into();
                }
            }
        }
    }
    if pm == Some("pnpm") {
        dto.summary = dto
            .summary
            .replace("Yarn Workspace", "pnpm workspace")
            .replace("Yarn workspace", "pnpm workspace")
            .replace("yarn workspace", "pnpm workspace");
    }
    dto
}

#[cfg(test)]
mod tests {
    use super::resolve_application_root;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn resolves_when_workspace_is_single_app() {
        let base = std::env::temp_dir().join(format!(
            "pusa-app-root-test-{}",
            std::process::id()
        ));
        let app_dir = base.join("applications").join("MyApp");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&app_dir).unwrap();
        let got = resolve_application_root(&app_dir);
        assert_eq!(got, base.join("applications"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn resolves_nested_application_under_repo() {
        let base = std::env::temp_dir().join(format!(
            "pusa-app-root-repo-{}",
            std::process::id()
        ));
        let apps = base.join("applications");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&apps).unwrap();
        let got = resolve_application_root(&base);
        assert_eq!(got, apps);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn fallback_when_no_application_dir() {
        let base = PathBuf::from("/tmp/pusa-no-apps-xyz-should-not-exist");
        let got = resolve_application_root(&base);
        assert_eq!(got, base.join("applications"));
    }

    #[test]
    fn prefers_legacy_application_dir_if_present() {
        let base = std::env::temp_dir().join(format!(
            "pusa-app-root-legacy-{}",
            std::process::id()
        ));
        let legacy = base.join("application");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&legacy).unwrap();
        let got = resolve_application_root(&base);
        assert_eq!(got, legacy);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn detects_pnpm_over_yarn_lock() {
        let base = std::env::temp_dir().join(format!(
            "pusa-pm-detect-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'\n").unwrap();
        fs::write(base.join("yarn.lock"), "# yarn\n").unwrap();
        assert_eq!(super::detect_js_package_manager(&base), Some("pnpm"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn unwraps_sh_c_and_rewrites_yarn_install_to_pnpm() {
        assert_eq!(
            super::unwrap_shell_c_command("sh -c 'yarn install'"),
            "yarn install"
        );
        assert_eq!(
            super::rewrite_install_command_for_pm("yarn install", "pnpm").as_deref(),
            Some("pnpm install")
        );
        assert_eq!(
            super::yarn_script_to_pnpm("dev"),
            "pnpm run dev"
        );
    }
}
