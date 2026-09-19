//! 扩展市场：Open VSX（VS Code 开源扩展），安装到工作区 `extensions/`。
//! 本地脚手架应用仍写入 `applications/`（首页「创建应用」等）。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use super::files::{extension_root, workspace_root};

const OPEN_VSX_API: &str = "https://open-vsx.org/api";
const USER_AGENT: &str = "PusaPluginMarket/0.2 (OpenVSX)";

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
    /// Open VSX 下载 URL（.vsix）；本地脚手架可为空。
    pub git_url: String,
    pub installed: bool,
    /// 是否偏向语法高亮 / 语言 Grammar（用于排序与徽章）。
    pub highlight_priority: bool,
    pub version: Option<String>,
    /// Open VSX `files.icon` / `icon`，或由 namespace/name/version 拼出的图标 URL。
    pub icon_url: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSource {
    OpenVsx,
    Local,
}

impl PluginSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenVsx => "openvsx",
            Self::Local => "local",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::OpenVsx => "Open VSX",
            Self::Local => "本地",
        }
    }

    fn from_meta(raw: &str) -> Self {
        match raw.trim() {
            "openvsx" | "official" | "github" | "ai" => Self::OpenVsx,
            _ => Self::Local,
        }
    }
}

/// 解析「应用库」目录（容纳各已安装本地应用的父目录）。
///
/// - 工作区本身是 `…/applications/{app}`：返回上一级 `applications/`
/// - 工作区本身叫 `applications`（或旧名 `application`）：直接返回它
/// - 工作区下存在 `applications/`（或旧名 `application/`）：返回该目录
/// - 否则回退为 `工作区/applications`
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

pub fn ensure_extension_root() -> Result<PathBuf, String> {
    let root = extension_root();
    fs::create_dir_all(&root).map_err(|e| format!("无法创建 extensions 目录：{e}"))?;
    Ok(root)
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
        "ext".into()
    } else {
        out
    }
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
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    icon_url: Option<String>,
}

fn read_install_meta(dir: &Path) -> Option<InstallMeta> {
    let path = dir.join(".pusa-plugin.json");
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_install_meta(target: &Path, item: &PluginItem) -> Result<(), String> {
    let meta = InstallMeta {
        id: item.id.clone(),
        name: item.name.clone(),
        source: item.source.as_str().into(),
        git_url: item.git_url.clone(),
        homepage: item.homepage.clone(),
        description: item.description.clone(),
        version: item.version.clone(),
        icon_url: item.icon_url.clone(),
    };
    let path = target.join(".pusa-plugin.json");
    let raw = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| format!("写入安装元数据失败：{e}"))
}

/// 解析 VS Code 扩展 `package.json` 中的 `contributes.grammars` / `languages`。
///
/// 安装后写入 `.pusa-grammars.json`。编辑器在打开文件时通过
/// [`list_installed_grammar_contributes`] 取回扩展目录与贡献，再加载 tmLanguage
///（见 `shell/textmate`）。无匹配时回退 `shell/syntax.rs` 内置分词。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrammarContribution {
    pub language: Option<String>,
    pub scope_name: Option<String>,
    pub path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageContribution {
    pub id: Option<String>,
    pub extensions: Vec<String>,
    pub aliases: Vec<String>,
    /// 无扩展名的文件名（如 `Dockerfile`）。旧缓存缺省为空。
    #[serde(default)]
    pub filenames: Vec<String>,
}

/// `.pusa-grammars.json` 结构版本。低于此值时重新解析 `package.json`。
const GRAMMAR_CACHE_SCHEMA: u32 = 2;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionContributes {
    pub extension_id: String,
    pub grammars: Vec<GrammarContribution>,
    pub languages: Vec<LanguageContribution>,
    /// 缓存结构版本；旧文件缺省为 0，触发重新解析。
    #[serde(default)]
    pub schema: u32,
}

/// 已安装扩展目录及其语法贡献。
#[derive(Clone, Debug)]
pub struct InstalledGrammarContributes {
    pub dir: PathBuf,
    pub contributes: ExtensionContributes,
}

const GRAMMARS_CACHE: &str = ".pusa-grammars.json";

/// 从已解压扩展目录读取 `package.json` 的 contributes 摘要。
pub fn parse_extension_contributes(dir: &Path) -> Result<ExtensionContributes, String> {
    let pkg_path = dir.join("package.json");
    let raw = fs::read_to_string(&pkg_path).map_err(|e| format!("读取 package.json 失败：{e}"))?;
    let v: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("解析 package.json 失败：{e}"))?;

    let publisher = v
        .get("publisher")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
    let extension_id = if !publisher.is_empty() && !name.is_empty() {
        format!("{publisher}.{name}")
    } else {
        dir.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    };

    let contributes = v.get("contributes").cloned().unwrap_or(serde_json::json!({}));
    let mut grammars = Vec::new();
    if let Some(arr) = contributes.get("grammars").and_then(|x| x.as_array()) {
        for g in arr {
            grammars.push(GrammarContribution {
                language: g
                    .get("language")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                scope_name: g
                    .get("scopeName")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                path: g
                    .get("path")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
            });
        }
    }
    let mut languages = Vec::new();
    if let Some(arr) = contributes.get("languages").and_then(|x| x.as_array()) {
        for lang in arr {
            let extensions = lang
                .get("extensions")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let aliases = lang
                .get("aliases")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let filenames = lang
                .get("filenames")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            languages.push(LanguageContribution {
                id: lang
                    .get("id")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string()),
                extensions,
                aliases,
                filenames,
            });
        }
    }

    Ok(ExtensionContributes {
        extension_id,
        grammars,
        languages,
        schema: GRAMMAR_CACHE_SCHEMA,
    })
}

fn write_grammars_cache(dir: &Path, contributes: &ExtensionContributes) -> Result<(), String> {
    let path = dir.join(GRAMMARS_CACHE);
    let raw = serde_json::to_string_pretty(contributes).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| format!("写入语法贡献缓存失败：{e}"))
}

/// 扫描已安装扩展的 Grammar 贡献（目录 + `.pusa-grammars.json` / `package.json`）。
///
/// 编辑器按返回的 `dir` 与 `grammars[].path` 加载 TextMate。
pub fn list_installed_grammar_contributes() -> Vec<InstalledGrammarContributes> {
    let root = extension_root();
    let Ok(entries) = fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let cache = path.join(GRAMMARS_CACHE);
        if cache.is_file() {
            if let Ok(raw) = fs::read_to_string(&cache) {
                if let Ok(c) = serde_json::from_str::<ExtensionContributes>(&raw) {
                    if c.schema >= GRAMMAR_CACHE_SCHEMA {
                        out.push(InstalledGrammarContributes {
                            dir: path,
                            contributes: c,
                        });
                        continue;
                    }
                }
            }
        }
        if let Ok(c) = parse_extension_contributes(&path) {
            let _ = write_grammars_cache(&path, &c);
            out.push(InstalledGrammarContributes {
                dir: path,
                contributes: c,
            });
        }
    }
    out.sort_by(|a, b| a.contributes.extension_id.cmp(&b.contributes.extension_id));
    out
}

fn looks_like_vscode_extension(dir: &Path) -> bool {
    dir.join("package.json").is_file()
}

fn path_to_file_url(path: &Path) -> String {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    #[cfg(windows)]
    {
        let s = abs.to_string_lossy().replace('\\', "/");
        if s.starts_with('/') {
            format!("file://{s}")
        } else {
            format!("file:///{s}")
        }
    }
    #[cfg(not(windows))]
    {
        format!("file://{}", abs.to_string_lossy())
    }
}

/// 已安装扩展图标：优先 `package.json` 的绝对 URL，其次拼 Open VSX file URL，最后本地 `file://`。
fn package_icon_url(dir: &Path) -> Option<String> {
    let raw = fs::read_to_string(dir.join("package.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let icon_rel = v.get("icon").and_then(|x| x.as_str())?.trim();
    if icon_rel.is_empty() {
        return None;
    }
    if icon_rel.starts_with("https://") || icon_rel.starts_with("http://") {
        return Some(icon_rel.to_string());
    }
    let rel = icon_rel.trim_start_matches("./");
    let publisher = v.get("publisher").and_then(|x| x.as_str()).unwrap_or("");
    let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("");
    let version = v.get("version").and_then(|x| x.as_str()).unwrap_or("");
    if !publisher.is_empty() && !name.is_empty() && !version.is_empty() {
        return Some(format!(
            "https://open-vsx.org/api/{publisher}/{name}/{version}/file/{rel}"
        ));
    }
    let local = dir.join(rel);
    if local.is_file() {
        return Some(path_to_file_url(&local));
    }
    None
}

fn resolve_openvsx_icon(
    files_icon: Option<&str>,
    top_icon: Option<&str>,
    namespace: &str,
    name: &str,
    version: Option<&str>,
) -> Option<String> {
    let pick = |raw: Option<&str>| -> Option<String> {
        let u = raw.map(str::trim).filter(|s| !s.is_empty())?;
        if u.starts_with("https://") || u.starts_with("http://") {
            return Some(u.to_string());
        }
        let ver = version.map(str::trim).filter(|s| !s.is_empty())?;
        if namespace.is_empty() || name.is_empty() {
            return None;
        }
        let path = u.trim_start_matches("./");
        Some(format!(
            "https://open-vsx.org/api/{namespace}/{name}/{ver}/file/{path}"
        ))
    };
    pick(files_icon).or_else(|| pick(top_icon))
}

/// 扫描 `extensions/` 下已安装的 VS Code 扩展（及带 `.pusa-plugin.json` 的目录）。
pub fn list_installed_ids() -> Vec<String> {
    let root = extension_root();
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
        if looks_like_vscode_extension(&path) || path.join(".pusa-plugin.json").is_file() {
            ids.push(name.to_string());
        }
    }
    ids.sort();
    ids
}

fn package_display_info(dir: &Path) -> Option<(String, String, bool)> {
    let raw = fs::read_to_string(dir.join("package.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let name = v
        .get("displayName")
        .and_then(|x| x.as_str())
        .or_else(|| v.get("name").and_then(|x| x.as_str()))
        .unwrap_or("")
        .to_string();
    let description = v
        .get("description")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let categories: Vec<String> = v
        .get("categories")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let keywords: Vec<String> = v
        .get("keywords")
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let contributes = v.get("contributes");
    let has_grammar = contributes
        .and_then(|c| c.get("grammars"))
        .and_then(|g| g.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    let highlight = has_grammar
        || is_highlight_extension(&categories, &keywords, &name, &description);
    Some((name, description, highlight))
}

/// 扫描已安装扩展，优先读取 `.pusa-plugin.json` / `package.json`。
pub fn list_installed_plugins() -> Vec<PluginItem> {
    let root = extension_root();
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
        if !looks_like_vscode_extension(&path) && !path.join(".pusa-plugin.json").is_file() {
            continue;
        }
        let id = dirname.to_string();
        let meta = read_install_meta(&path);
        let stored_icon = meta.as_ref().and_then(|m| {
            m.icon_url
                .clone()
                .filter(|s| !s.trim().is_empty())
        });
        let icon_url = stored_icon.or_else(|| package_icon_url(&path));
        let pkg = package_display_info(&path);
        let (name, description, source, git_url, homepage, version, highlight) =
            if let Some(m) = meta {
                let name = if m.name.trim().is_empty() {
                    pkg.as_ref()
                        .map(|(n, _, _)| n.clone())
                        .unwrap_or_else(|| dirname.to_string())
                } else {
                    m.name
                };
                let description = if m.description.trim().is_empty() {
                    pkg.as_ref()
                        .map(|(_, d, _)| d.clone())
                        .unwrap_or_else(|| format!("extensions/{dirname}/"))
                } else {
                    m.description
                };
                let homepage = if m.homepage.trim().is_empty() {
                    format!("https://open-vsx.org/extension/{}", id.replace('.', "/"))
                } else {
                    m.homepage
                };
                let highlight = pkg
                    .as_ref()
                    .map(|(_, _, h)| *h)
                    .unwrap_or(false);
                (
                    name,
                    description,
                    PluginSource::from_meta(&m.source),
                    m.git_url,
                    homepage,
                    m.version,
                    highlight,
                )
            } else if let Some((name, description, highlight)) = pkg {
                (
                    if name.is_empty() {
                        dirname.to_string()
                    } else {
                        name
                    },
                    if description.is_empty() {
                        format!("extensions/{dirname}/")
                    } else {
                        description
                    },
                    PluginSource::OpenVsx,
                    String::new(),
                    format!("https://open-vsx.org/extension/{}", id.replace('.', "/")),
                    None,
                    highlight,
                )
            } else {
                (
                    dirname.to_string(),
                    format!("extensions/{dirname}/"),
                    PluginSource::Local,
                    String::new(),
                    String::new(),
                    None,
                    false,
                )
            };
        items.push(PluginItem {
            id,
            name,
            description,
            source,
            language: if highlight {
                Some("语法高亮".into())
            } else {
                None
            },
            stars: None,
            mentions: None,
            homepage,
            git_url,
            installed: true,
            highlight_priority: highlight,
            version,
            icon_url,
        });
    }
    items.sort_by(|a, b| {
        b.highlight_priority
            .cmp(&a.highlight_priority)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    items
}

/// 在系统文件管理器中打开已安装扩展/应用目录。
pub fn open_installed_plugin(id: &str) -> Result<(), String> {
    let path = plugin_dir(id)?;
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

/// 卸载扩展：优先 `extensions/{id}`，否则 `applications/{id}`。
pub fn uninstall_plugin(id: &str) -> Result<(), String> {
    let id = validate_plugin_id(id)?;
    let candidates = [
        (extension_root(), "extensions"),
        (application_root(), "applications"),
    ];
    for (root, label) in candidates {
        let target = root.join(&id);
        if !target.is_dir() {
            continue;
        }
        let canonical_root = root
            .canonicalize()
            .map_err(|e| format!("无法解析 {label} 目录：{e}"))?;
        let canonical_target = target
            .canonicalize()
            .map_err(|e| format!("无法解析插件目录：{e}"))?;
        if !canonical_target.starts_with(&canonical_root) || canonical_target == canonical_root {
            return Err(format!("拒绝卸载：路径不在 {label}/ 下"));
        }
        fs::remove_dir_all(&canonical_target).map_err(|e| format!("卸载失败：{e}"))?;
        return Ok(());
    }
    Err(format!("未找到已安装目录：{id}"))
}

pub fn is_highlight_extension(
    categories: &[String],
    tags_or_keywords: &[String],
    name: &str,
    description: &str,
) -> bool {
    let cat_hit = categories.iter().any(|c| {
        let l = c.to_ascii_lowercase();
        l.contains("programming languages") || l == "themes" || l.contains("language")
    });
    if cat_hit {
        return true;
    }
    let blob = format!(
        "{} {} {}",
        name.to_ascii_lowercase(),
        description.to_ascii_lowercase(),
        tags_or_keywords
            .iter()
            .map(|t| t.to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join(" ")
    );
    const KEYS: &[&str] = &[
        "grammar",
        "syntax",
        "highlight",
        "textmate",
        "tmlanguage",
        "language support",
        "syntax highlighting",
    ];
    KEYS.iter().any(|k| blob.contains(k))
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败：{e}"))
}

#[derive(Clone, Debug, Deserialize)]
struct OpenVsxSearchResponse {
    #[serde(default)]
    extensions: Vec<OpenVsxSearchItem>,
}

#[derive(Clone, Debug, Deserialize)]
struct OpenVsxSearchItem {
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
    files: Option<OpenVsxFiles>,
    #[serde(default)]
    icon: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct OpenVsxFiles {
    #[serde(default)]
    download: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct OpenVsxDetail {
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
    categories: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    files: Option<OpenVsxFiles>,
    #[serde(default)]
    downloads: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

fn search_item_display_name(item: &OpenVsxSearchItem) -> String {
    item.display_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            if item.namespace.is_empty() {
                item.name.clone()
            } else {
                format!("{}.{}", item.namespace, item.name)
            }
        })
}

fn host_platform_key() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "darwin-arm64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "darwin-x64"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "win32-x64"
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        "win32-arm64"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "linux-arm64"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux-x64"
    }
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "aarch64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "linux", target_arch = "x86_64"),
    )))]
    {
        "universal"
    }
}

fn pick_vsix_url(detail: &OpenVsxDetail) -> Option<String> {
    if let Some(map) = &detail.downloads {
        let key = host_platform_key();
        if let Some(u) = map.get(key) {
            return Some(u.clone());
        }
        // 无本机平台时优先无后缀的 universal（files.download）
    }
    detail
        .files
        .as_ref()
        .and_then(|f| f.download.clone())
        .filter(|u| !u.trim().is_empty())
}

fn openvsx_item_to_plugin(
    namespace: &str,
    name: &str,
    display: String,
    description: String,
    version: Option<String>,
    downloads: Option<u64>,
    download_url: String,
    homepage: String,
    icon_url: Option<String>,
    categories: &[String],
    tags: &[String],
    installed: &[String],
) -> PluginItem {
    let id = sanitize_dir_name(&format!("{namespace}.{name}"));
    let highlight = is_highlight_extension(categories, tags, &display, &description);
    let language = if highlight {
        Some("语法高亮".into())
    } else {
        categories.first().cloned()
    };
    let mut item = PluginItem {
        id: id.clone(),
        name: display,
        description: if description.trim().is_empty() {
            "（无描述）".into()
        } else {
            description
        },
        source: PluginSource::OpenVsx,
        language,
        stars: downloads,
        mentions: None,
        homepage,
        git_url: download_url,
        installed: false,
        highlight_priority: highlight,
        version,
        icon_url,
    };
    item.installed = installed.iter().any(|x| x == &id);
    item
}

fn search_item_to_plugin(item: OpenVsxSearchItem, installed: &[String]) -> Option<PluginItem> {
    if item.namespace.trim().is_empty() || item.name.trim().is_empty() {
        return None;
    }
    let namespace = item.namespace.clone();
    let name = item.name.clone();
    let display = search_item_display_name(&item);
    let description = item
        .description
        .clone()
        .filter(|d| !d.trim().is_empty())
        .unwrap_or_else(|| "（无描述）".into());
    let download_url = item
        .files
        .as_ref()
        .and_then(|f| f.download.clone())
        .unwrap_or_default();
    let homepage = format!("https://open-vsx.org/extension/{namespace}/{name}");
    let downloads = item.download_count;
    let version = item.version.clone();
    let icon_url = resolve_openvsx_icon(
        item.files.as_ref().and_then(|f| f.icon.as_deref()),
        item.icon.as_deref(),
        &namespace,
        &name,
        item.version.as_deref(),
    );
    Some(openvsx_item_to_plugin(
        &namespace,
        &name,
        display,
        description,
        version,
        downloads,
        download_url,
        homepage,
        icon_url,
        &[],
        &[],
        installed,
    ))
}

fn sort_highlight_first(items: &mut [PluginItem]) {
    items.sort_by(|a, b| {
        b.highlight_priority
            .cmp(&a.highlight_priority)
            .then_with(|| b.stars.unwrap_or(0).cmp(&a.stars.unwrap_or(0)))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
}

async fn openvsx_search_raw(
    query: &str,
    category: Option<&str>,
    size: u32,
) -> Result<Vec<OpenVsxSearchItem>, String> {
    let client = http_client()?;
    let mut req = client
        .get(format!("{OPEN_VSX_API}/-/search"))
        .query(&[("size", size.to_string()), ("sortBy", "downloadCount".into()), ("sortOrder", "desc".into())]);
    if !query.trim().is_empty() {
        req = req.query(&[("query", query)]);
    }
    if let Some(cat) = category {
        req = req.query(&[("category", cat)]);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("Open VSX 搜索失败：{e}"))?;
    let status = resp.status().as_u16();
    if !(200..300).contains(&status) {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Open VSX 搜索失败（HTTP {status}）：{}", body.chars().take(120).collect::<String>()));
    }
    let parsed = resp
        .json::<OpenVsxSearchResponse>()
        .await
        .map_err(|e| format!("Open VSX 搜索结果解析失败：{e}"))?;
    Ok(parsed.extensions)
}

/// 默认浏览：优先 Programming Languages 类扩展，并合并语法高亮向查询。
pub async fn browse_openvsx_highlight_catalog() -> Result<Vec<PluginItem>, String> {
    let installed = list_installed_ids();
    let mut by_id = std::collections::HashMap::<String, PluginItem>::new();

    let lang_items = openvsx_search_raw("", Some("Programming Languages"), 24).await?;
    for item in lang_items {
        if let Some(mut p) = search_item_to_plugin(item, &installed) {
            // 分类浏览进来的一律视为高亮优先候选
            p.highlight_priority = true;
            if p.language.is_none() {
                p.language = Some("Programming Languages".into());
            }
            by_id.insert(p.id.clone(), p);
        }
    }

    for q in ["syntax highlight", "grammar", "textmate"] {
        if let Ok(items) = openvsx_search_raw(q, None, 12).await {
            for item in items {
                if let Some(p) = search_item_to_plugin(item, &installed) {
                    by_id.entry(p.id.clone()).or_insert(p);
                }
            }
        }
    }

    let mut out: Vec<PluginItem> = by_id.into_values().collect();
    sort_highlight_first(&mut out);
    out.truncate(40);
    Ok(out)
}

/// Open VSX 关键词搜索；结果按语法高亮相关性优先排序。
pub async fn search_openvsx_extensions(query: &str) -> Result<Vec<PluginItem>, String> {
    let q = query.trim();
    if q.is_empty() {
        return browse_openvsx_highlight_catalog().await;
    }
    let installed = list_installed_ids();
    let mut by_id = std::collections::HashMap::<String, PluginItem>::new();

    // 先按 Programming Languages 分类收窄，再做全库补充。
    if let Ok(items) = openvsx_search_raw(q, Some("Programming Languages"), 20).await {
        for item in items {
            if let Some(mut p) = search_item_to_plugin(item, &installed) {
                p.highlight_priority = true;
                by_id.insert(p.id.clone(), p);
            }
        }
    }
    let items = openvsx_search_raw(q, None, 30).await?;
    for item in items {
        if let Some(p) = search_item_to_plugin(item, &installed) {
            by_id.entry(p.id.clone()).or_insert(p);
        }
    }

    let mut out: Vec<PluginItem> = by_id.into_values().collect();
    sort_highlight_first(&mut out);
    Ok(out)
}

async fn resolve_download_url(item: &PluginItem) -> Result<String, String> {
    if !item.git_url.trim().is_empty() && !item.git_url.contains('@') {
        // 无平台后缀的通用包可直接用
        return Ok(item.git_url.clone());
    }
    let parts: Vec<&str> = item.id.splitn(2, '.').collect();
    if parts.len() != 2 {
        if !item.git_url.trim().is_empty() {
            return Ok(item.git_url.clone());
        }
        return Err("无法解析扩展命名空间".into());
    }
    let (ns, name) = (parts[0], parts[1]);
    let client = http_client()?;
    let url = format!("{OPEN_VSX_API}/{ns}/{name}/latest");
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("查询扩展详情失败：{e}"))?;
    if !resp.status().is_success() {
        if !item.git_url.trim().is_empty() {
            return Ok(item.git_url.clone());
        }
        return Err(format!("查询扩展详情失败（HTTP {}）", resp.status()));
    }
    let detail = resp
        .json::<OpenVsxDetail>()
        .await
        .map_err(|e| format!("解析扩展详情失败：{e}"))?;
    pick_vsix_url(&detail)
        .or_else(|| {
            if item.git_url.trim().is_empty() {
                None
            } else {
                Some(item.git_url.clone())
            }
        })
        .ok_or_else(|| "该扩展没有可用的 .vsix 下载地址".into())
}

fn extract_vsix(vsix_path: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("创建目标目录失败：{e}"))?;
    #[cfg(target_os = "windows")]
    {
        // Expand-Archive 需要 .zip 后缀
        let zip_path = vsix_path.with_extension("zip");
        if zip_path != vsix_path {
            fs::copy(vsix_path, &zip_path).map_err(|e| format!("准备 zip 失败：{e}"))?;
        }
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
                    zip_path.display(),
                    dest.display()
                ),
            ])
            .status()
            .map_err(|e| format!("解压 .vsix 失败：{e}"))?;
        if zip_path != vsix_path {
            let _ = fs::remove_file(&zip_path);
        }
        if !status.success() {
            return Err("解压 .vsix 失败".into());
        }
        return Ok(());
    }
    #[cfg(not(target_os = "windows"))]
    {
        let status = Command::new("unzip")
            .args([
                "-oq",
                vsix_path.to_str().ok_or("路径无效")?,
                "-d",
                dest.to_str().ok_or("路径无效")?,
            ])
            .status()
            .map_err(|e| format!("解压 .vsix 失败（需要 unzip）：{e}"))?;
        if !status.success() {
            return Err("解压 .vsix 失败".into());
        }
        Ok(())
    }
}

/// 下载 Open VSX `.vsix` 并解压到 `extensions/{id}/`，解析 grammars 贡献。
pub async fn install_plugin(item: &PluginItem) -> Result<PathBuf, String> {
    let root = ensure_extension_root()?;
    let target = root.join(&item.id);
    if target.exists() {
        return Err(format!("已存在：{}", target.display()));
    }

    let download_url = resolve_download_url(item).await?;
    let client = http_client()?;
    let resp = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载 .vsix 失败：{e}"))?;
    if !resp.status().is_success() {
        return Err(format!("下载 .vsix 失败（HTTP {}）", resp.status()));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("读取 .vsix 内容失败：{e}"))?;

    let tmp_dir = std::env::temp_dir().join(format!(
        "pusa-vsix-{}-{}",
        std::process::id(),
        item.id.replace('.', "_")
    ));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).map_err(|e| format!("创建临时目录失败：{e}"))?;
    let vsix_path = tmp_dir.join("ext.vsix");
    {
        let mut f = fs::File::create(&vsix_path).map_err(|e| format!("写入临时文件失败：{e}"))?;
        f.write_all(&bytes)
            .map_err(|e| format!("写入临时文件失败：{e}"))?;
    }

    let extract_tmp = tmp_dir.join("unpacked");
    if let Err(e) = extract_vsix(&vsix_path, &extract_tmp) {
        let _ = fs::remove_dir_all(&tmp_dir);
        return Err(e);
    }

    // 部分包在 zip 根目录，部分在 extension/ 子目录
    let content_root = if extract_tmp.join("package.json").is_file() {
        extract_tmp.clone()
    } else if extract_tmp.join("extension").join("package.json").is_file() {
        extract_tmp.join("extension")
    } else {
        extract_tmp.clone()
    };

    if let Err(e) = fs::rename(&content_root, &target) {
        // 跨设备 rename 失败时改为递归复制
        if let Err(copy_err) = copy_dir_recursive(&content_root, &target) {
            let _ = fs::remove_dir_all(&tmp_dir);
            let _ = fs::remove_dir_all(&target);
            return Err(format!("安装失败：{e} / {copy_err}"));
        }
    }
    let _ = fs::remove_dir_all(&tmp_dir);

    write_install_meta(&target, item)?;
    match parse_extension_contributes(&target) {
        Ok(c) => {
            let _ = write_grammars_cache(&target, &c);
        }
        Err(e) => {
            // 非致命：仍算安装成功
            let _ = e;
        }
    }
    Ok(target.canonicalize().unwrap_or(target))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 是否为 Open VSX / VS Code 扩展（智能 UI 扫描应跳过）。
pub fn is_vscode_extension_install(id: &str) -> bool {
    let Ok(id) = validate_plugin_id(id) else {
        return false;
    };
    let dir = extension_root().join(&id);
    if looks_like_vscode_extension(&dir) {
        return true;
    }
    if let Some(m) = read_install_meta(&dir) {
        return PluginSource::from_meta(&m.source) == PluginSource::OpenVsx;
    }
    false
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
        version: None,
        icon_url: None,
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

/// 已安装扩展/应用目录：优先 `extensions/{id}`，其次 `applications/{id}`。
pub fn plugin_dir(id: &str) -> Result<PathBuf, String> {
    let id = validate_plugin_id(id)?;
    for root in [extension_root(), application_root()] {
        let path = root.join(&id);
        if path.is_dir() {
            return Ok(path.canonicalize().unwrap_or(path));
        }
    }
    Err(format!("未找到已安装目录：{id}"))
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

/// 尚未缓存智能 UI 的已安装应用 id 列表（跳过 VS Code / Open VSX 扩展）。
pub fn list_ids_missing_smart_ui() -> Vec<String> {
    // 智能 UI 面向 applications/ 脚手架应用；VS Code 扩展无脚本面板需求。
    let app_root = application_root();
    let Ok(entries) = fs::read_dir(&app_root) else {
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
        if name.starts_with('.') {
            continue;
        }
        if is_vscode_extension_install(name) {
            continue;
        }
        if !has_smart_ui_cache(name) {
            ids.push(name.to_string());
        }
    }
    ids
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
        .map_err(|e| e.to_string())?
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
