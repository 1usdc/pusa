//! 桌面端工作区文件系统：列表、读写、新建、打开项目。
//!
//! 根目录与 Agent 工具一致，见 [`shared::tools::file::project_root`]：
//! 1. 环境变量 `ANOTHERCLAW_TOOLS_ROOT`（若为有效目录）
//! 2. 否则最近打开过的项目目录（持久化）
//! 3. 否则当前工作目录；若其下有 `skills/`，优先视为项目根

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

/// 启动控制台子进程时不弹出 Windows 控制台窗口（`CREATE_NO_WINDOW`）。
pub fn hidden_command(program: impl AsRef<OsStr>) -> Command {
    let mut cmd = Command::new(program);
    hide_windows_console(&mut cmd);
    cmd
}

fn hide_windows_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = cmd;
    }
}

const MAX_TEXT_BYTES: u64 = 2 * 1024 * 1024;
/// 磁盘上保留的最近目录上限（展示仍取前 [`RECENT_DISPLAY_CAP`] 条）。
const RECENT_STORE_CAP: usize = 20;
/// 首页 logo 右键菜单展示条数。
pub const RECENT_DISPLAY_CAP: usize = 5;

/// 侧栏 / 标签页用的目录项。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct RecentProjectsStore {
    /// 最近打开的**目录**路径，最近在前。
    #[serde(default)]
    dirs: Vec<String>,
}

pub(crate) fn app_data_dir() -> PathBuf {
    let root = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("AnotherClaw");
    let _ = fs::create_dir_all(&root);
    root
}

fn recent_projects_path() -> PathBuf {
    app_data_dir().join("recent_projects.json")
}

fn center_tabs_path() -> PathBuf {
    app_data_dir().join("center_tabs.json")
}

fn file_tree_state_path() -> PathBuf {
    app_data_dir().join("file_tree.json")
}

fn custom_chat_models_path() -> PathBuf {
    app_data_dir().join("custom_chat_models.json")
}

fn llm_credentials_enabled_path() -> PathBuf {
    app_data_dir().join("llm_credentials_enabled.json")
}

fn chat_model_path() -> PathBuf {
    app_data_dir().join("chat_model.txt")
}

fn chat_agent_mode_path() -> PathBuf {
    app_data_dir().join("chat_agent_mode.txt")
}

fn ui_theme_path() -> PathBuf {
    app_data_dir().join("ui_theme.txt")
}

/// 读取中间栏标签页操作缓存 JSON；无文件或读失败返回 `None`。
pub fn center_tabs_cache_get() -> Option<String> {
    let raw = fs::read_to_string(center_tabs_path()).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 写入中间栏标签页缓存；空字符串删除文件。
pub fn center_tabs_cache_set(raw_json: &str) {
    let path = center_tabs_path();
    let trimmed = raw_json.trim();
    if trimmed.is_empty() {
        let _ = fs::remove_file(path);
    } else {
        let _ = fs::write(path, trimmed);
    }
}

/// 读取侧栏文件树展开/选中状态 JSON；无文件或读失败返回 `None`。
pub fn file_tree_state_get() -> Option<String> {
    let raw = fs::read_to_string(file_tree_state_path()).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 写入侧栏文件树状态；空字符串删除文件。
pub fn file_tree_state_set(raw_json: &str) {
    let path = file_tree_state_path();
    let trimmed = raw_json.trim();
    if trimmed.is_empty() {
        let _ = fs::remove_file(path);
    } else {
        let _ = fs::write(path, trimmed);
    }
}

/// 读取用户自定义聊天模型（展示名与可选密钥绑定）。
pub fn custom_chat_models_get() -> Vec<crate::custom_chat_models::CustomChatModelPref> {
    let Ok(raw) = fs::read_to_string(custom_chat_models_path()) else {
        return Vec::new();
    };
    crate::custom_chat_models::parse_custom_chat_models_json(&raw)
}

/// 写入用户自定义聊天模型；空列表删除文件。
pub fn custom_chat_models_set(items: &[crate::custom_chat_models::CustomChatModelPref]) {
    let path = custom_chat_models_path();
    match crate::custom_chat_models::serialize_custom_chat_models_json(items) {
        None => {
            let _ = fs::remove_file(path);
        }
        Some(json) => {
            let _ = fs::write(path, json);
        }
    }
}

/// 读取本地「已开启」的 LLM 密钥 id 列表；从未保存返回 `None`。
pub fn llm_credentials_enabled_get() -> Option<Vec<String>> {
    let raw = fs::read_to_string(llm_credentials_enabled_path()).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str::<Vec<String>>(trimmed).ok().map(|ids| {
        ids.into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

/// 写入已开启的 LLM 密钥 id 列表。
pub fn llm_credentials_enabled_set(ids: &[String]) {
    let path = llm_credentials_enabled_path();
    let cleaned: Vec<String> = ids
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if let Ok(json) = serde_json::to_string_pretty(&cleaned) {
        let _ = fs::write(path, json);
    }
}

/// 读取聊天栏上次选中的模型 ID。
pub fn chat_model_get() -> Option<String> {
    let raw = fs::read_to_string(chat_model_path()).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 写入聊天栏当前选中的模型 ID；空字符串删除文件。
pub fn chat_model_set(model_id: &str) {
    let path = chat_model_path();
    let trimmed = model_id.trim();
    if trimmed.is_empty() {
        let _ = fs::remove_file(path);
    } else {
        let _ = fs::write(path, trimmed);
    }
}

/// 读取聊天栏上次选中的 Agent 模式（`agent` / `ask` / `plan` / `multitask`）。
pub fn chat_agent_mode_get() -> Option<String> {
    let raw = fs::read_to_string(chat_agent_mode_path()).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 写入聊天栏 Agent 模式；空字符串删除文件。
pub fn chat_agent_mode_set(mode: &str) {
    let path = chat_agent_mode_path();
    let trimmed = mode.trim();
    if trimmed.is_empty() {
        let _ = fs::remove_file(path);
    } else {
        let _ = fs::write(path, trimmed);
    }
}

/// 读取 UI 主题（`light` / `dark`）。
pub fn ui_theme_get() -> Option<String> {
    let raw = fs::read_to_string(ui_theme_path()).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 写入 UI 主题。
pub fn ui_theme_set(theme: &str) {
    let path = ui_theme_path();
    let trimmed = theme.trim();
    if trimmed.is_empty() {
        let _ = fs::remove_file(path);
    } else {
        let _ = fs::write(path, trimmed);
    }
}

fn load_recent_store() -> RecentProjectsStore {
    let path = recent_projects_path();
    let Ok(raw) = fs::read_to_string(&path) else {
        return RecentProjectsStore::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_recent_store(store: &RecentProjectsStore) {
    let path = recent_projects_path();
    if let Ok(raw) = serde_json::to_string_pretty(store) {
        let _ = fs::write(path, raw);
    }
}

/// 仅保留仍存在的目录；去重并截断。
fn normalize_recent_dirs(dirs: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for raw in dirs {
        let path = PathBuf::from(raw.trim());
        if !path.is_dir() {
            continue;
        }
        let abs = path.canonicalize().unwrap_or(path);
        let s = abs.to_string_lossy().into_owned();
        if out.iter().any(|existing| existing == &s) {
            continue;
        }
        out.push(s);
        if out.len() >= RECENT_STORE_CAP {
            break;
        }
    }
    out
}

/// 读取最近打开的项目目录（最多 [`RECENT_DISPLAY_CAP`] 条，仅目录）。
pub fn recent_project_dirs() -> Vec<String> {
    normalize_recent_dirs(load_recent_store().dirs)
        .into_iter()
        .take(RECENT_DISPLAY_CAP)
        .collect()
}

/// 将目录记入最近打开列表（去重、最近优先；非目录忽略）。
pub fn record_recent_project_dir(path: &Path) {
    if !path.is_dir() {
        return;
    }
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let key = abs.to_string_lossy().into_owned();
    let mut store = load_recent_store();
    store.dirs.retain(|p| p != &key);
    store.dirs.insert(0, key);
    store.dirs = normalize_recent_dirs(store.dirs);
    save_recent_store(&store);
}

/// 若未设置有效的 `ANOTHERCLAW_TOOLS_ROOT`，用最近打开目录恢复。
fn restore_tools_root_if_needed() {
    if let Ok(root) = std::env::var("ANOTHERCLAW_TOOLS_ROOT") {
        if Path::new(&root).is_dir() {
            return;
        }
    }
    if let Some(dir) = recent_project_dirs().into_iter().next() {
        // 桌面单进程；与 Agent 工具共用同一环境变量。
        std::env::set_var("ANOTHERCLAW_TOOLS_ROOT", dir);
    }
}

/// 系统目录选择器：选择项目根目录。
pub fn pick_project_directory() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("打开项目")
        .pick_folder()
}

/// 将工作区根切换到指定目录，并写入最近列表。
pub fn set_workspace_root(path: &Path) -> Result<PathBuf, String> {
    if !path.is_dir() {
        return Err("请选择有效的项目目录。".into());
    }
    let abs = path
        .canonicalize()
        .map_err(|e| format!("无法解析目录：{e}"))?;
    std::env::set_var("ANOTHERCLAW_TOOLS_ROOT", &abs);
    record_recent_project_dir(&abs);
    Ok(abs)
}

/// 弹出目录选择器并打开为当前项目；取消选择返回 `Ok(None)`。
pub fn open_project_directory_dialog() -> Result<Option<PathBuf>, String> {
    let Some(picked) = pick_project_directory() else {
        return Ok(None);
    };
    set_workspace_root(&picked).map(Some)
}

/// 工作区根路径（绝对路径优先）。
pub fn workspace_root() -> PathBuf {
    restore_tools_root_if_needed();
    let root = shared::tools::file::project_root();
    root.canonicalize().unwrap_or(root)
}

pub fn root_display_name(root: &Path) -> String {
    root.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| root.to_string_lossy().into_owned())
}

/// 当前工作区 git HEAD：分支名 + 是否有未提交改动。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitHeadStatus {
    pub branch: String,
    pub dirty: bool,
}

/// 在 `root` 下查询当前分支；非 git 仓库或命令失败返回 `None`。
pub fn git_head_status(root: &Path) -> Option<GitHeadStatus> {
    if !root.is_dir() {
        return None;
    }
    // 安装目录等工作区没有 .git：不要每 1.5s 拉起 git.exe（GUI 下会闪控制台窗口）。
    if !root.join(".git").exists() {
        return None;
    }
    let root_s = root.to_string_lossy();
    let branch_out = hidden_command("git")
        .args(["-C", root_s.as_ref(), "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if !branch_out.status.success() {
        return None;
    }
    let branch = String::from_utf8_lossy(&branch_out.stdout)
        .trim()
        .to_string();
    if branch.is_empty() {
        return None;
    }
    let dirty = hidden_command("git")
        .args(["-C", root_s.as_ref(), "status", "--porcelain"])
        .output()
        .map(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false);
    Some(GitHeadStatus { branch, dirty })
}

pub fn list_dir(dir: &Path) -> Result<Vec<FsEntry>, String> {
    let rd = fs::read_dir(dir).map_err(|e| format!("无法读取目录：{e}"))?;
    let mut entries = Vec::new();
    for item in rd {
        let item = item.map_err(|e| format!("读取目录项失败：{e}"))?;
        let path = item.path();
        let name = item.file_name().to_string_lossy().into_owned();
        if name == "." || name == ".." {
            continue;
        }
        let is_dir = item.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let abs = path.canonicalize().unwrap_or(path);
        entries.push(FsEntry {
            name,
            path: abs.to_string_lossy().into_owned(),
            is_dir,
        });
    }
    entries.sort_by(|a, b| {
        let a_muted = is_muted_name(&a.name);
        let b_muted = is_muted_name(&b.name);
        match (a_muted, b_muted) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            },
        }
    });
    Ok(entries)
}

fn is_muted_name(name: &str) -> bool {
    matches!(name, "target" | "node_modules" | ".git")
}

/// 常见文本扩展名；未知扩展名会再做 UTF-8 嗅探。
fn has_text_extension(path: &Path) -> bool {
    const TEXT_EXT: &[&str] = &[
        "rs", "toml", "md", "txt", "json", "jsonl", "yaml", "yml", "toml", "css", "scss", "html",
        "htm", "js", "jsx", "ts", "tsx", "mjs", "cjs", "py", "sh", "bash", "zsh", "fish", "sql",
        "graphql", "svg", "xml", "csv", "tsv", "env", "gitignore", "dockerignore", "editorconfig",
        "lock", "just", "dockerfile", "makefile", "cmake", "ini", "cfg", "conf", "log", "rake",
        "rb", "go", "java", "kt", "swift", "c", "h", "cpp", "hpp", "cc", "m", "mm", "plist",
        "properties", "gradle", "proto", "vue", "svelte", "astro", "nix", "tf", "hcl",
    ];
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        // 无扩展名：允许常见点文件 / Makefile 等
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        return matches!(
            name.as_str(),
            "makefile"
                | "dockerfile"
                | "justfile"
                | "license"
                | "licence"
                | "readme"
                | "cargo.lock"
                | "gemfile"
                | ".gitignore"
                | ".env"
                | ".editorconfig"
                | ".dockerignore"
        ) || name.starts_with('.');
    };
    TEXT_EXT.iter().any(|e| e.eq_ignore_ascii_case(ext))
}

pub fn is_probably_text(path: &Path) -> bool {
    if has_text_extension(path) {
        return true;
    }
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() || meta.len() == 0 {
        return meta.is_file();
    }
    if meta.len() > MAX_TEXT_BYTES {
        return false;
    }
    let mut buf = vec![0u8; meta.len().min(512) as usize];
    let Ok(mut f) = fs::File::open(path) else {
        return false;
    };
    use std::io::Read;
    let Ok(n) = f.read(&mut buf) else {
        return false;
    };
    buf.truncate(n);
    if buf.contains(&0) {
        return false;
    }
    std::str::from_utf8(&buf).is_ok()
}

pub fn read_text_file(path: &Path) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| format!("无法读取文件信息：{e}"))?;
    if !meta.is_file() {
        return Err("不是普通文件。".into());
    }
    if meta.len() > MAX_TEXT_BYTES {
        return Err(format!(
            "文件过大（>{} MB），请用外部编辑器打开。",
            MAX_TEXT_BYTES / (1024 * 1024)
        ));
    }
    if !is_probably_text(path) {
        return Err("该文件可能是二进制，无法在此预览。".into());
    }
    fs::read_to_string(path).map_err(|e| format!("读取失败：{e}"))
}

pub fn write_text_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败：{e}"))?;
    }
    fs::write(path, content).map_err(|e| format!("写入失败：{e}"))
}

fn sanitize_child_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("名称不能为空。".into());
    }
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        return Err("名称不能包含路径分隔符或为 . / ..。".into());
    }
    Ok(name)
}

pub fn create_file(parent: &Path, name: &str) -> Result<PathBuf, String> {
    let name = sanitize_child_name(name)?;
    let path = parent.join(name);
    if path.exists() {
        return Err("同名文件或文件夹已存在。".into());
    }
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| format!("创建父目录失败：{e}"))?;
    }
    fs::write(&path, "").map_err(|e| format!("创建文件失败：{e}"))?;
    Ok(path.canonicalize().unwrap_or(path))
}

pub fn create_folder(parent: &Path, name: &str) -> Result<PathBuf, String> {
    let name = sanitize_child_name(name)?;
    let path = parent.join(name);
    if path.exists() {
        return Err("同名文件或文件夹已存在。".into());
    }
    fs::create_dir_all(&path).map_err(|e| format!("创建文件夹失败：{e}"))?;
    Ok(path.canonicalize().unwrap_or(path))
}

/// 重命名文件或文件夹（仅改末段名，不跨目录）。
pub fn rename_entry(path: &Path, new_name: &str) -> Result<PathBuf, String> {
    let new_name = sanitize_child_name(new_name)?;
    let parent = path
        .parent()
        .ok_or_else(|| "无法解析父目录。".to_string())?;
    let dest = parent.join(new_name);
    if dest.exists() {
        return Err("同名文件或文件夹已存在。".into());
    }
    fs::rename(path, &dest).map_err(|e| format!("重命名失败：{e}"))?;
    Ok(dest.canonicalize().unwrap_or(dest))
}

/// 删除文件或目录（目录递归删除）。
pub fn delete_entry(path: &Path) -> Result<(), String> {
    let meta = fs::metadata(path).map_err(|e| format!("无法读取：{e}"))?;
    if meta.is_dir() {
        fs::remove_dir_all(path).map_err(|e| format!("删除文件夹失败：{e}"))
    } else {
        fs::remove_file(path).map_err(|e| format!("删除文件失败：{e}"))
    }
}

/// 将文件/目录复制到目标目录（保留原名）。
pub fn copy_entry_into(src: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    if !dest_dir.is_dir() {
        return Err("目标不是目录。".into());
    }
    let name = src
        .file_name()
        .ok_or_else(|| "无效路径。".to_string())?;
    let dest = dest_dir.join(name);
    if dest.exists() {
        return Err(format!("目标已存在：{}", dest.display()));
    }
    if src.is_dir() {
        copy_dir_recursive(src, &dest)?;
    } else {
        if let Some(p) = dest.parent() {
            fs::create_dir_all(p).map_err(|e| format!("创建目录失败：{e}"))?;
        }
        fs::copy(src, &dest).map_err(|e| format!("复制失败：{e}"))?;
    }
    Ok(dest.canonicalize().unwrap_or(dest))
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("创建目录失败：{e}"))?;
    for item in fs::read_dir(src).map_err(|e| format!("读取目录失败：{e}"))? {
        let item = item.map_err(|e| format!("读取目录项失败：{e}"))?;
        let from = item.path();
        let to = dest.join(item.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| format!("复制失败：{e}"))?;
        }
    }
    Ok(())
}

/// 将条目移动到目标目录。
pub fn move_entry_into(src: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    if !dest_dir.is_dir() {
        return Err("目标不是目录。".into());
    }
    let name = src
        .file_name()
        .ok_or_else(|| "无效路径。".to_string())?;
    let dest = dest_dir.join(name);
    if dest.exists() {
        return Err(format!("目标已存在：{}", dest.display()));
    }
    fs::rename(src, &dest).map_err(|e| format!("移动失败：{e}"))?;
    Ok(dest.canonicalize().unwrap_or(dest))
}

/// 在系统文件管理器中显示并选中该路径。
pub fn reveal_in_os(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("无法打开 Finder：{e}"))?;
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
            .map_err(|e| format!("无法打开资源管理器：{e}"))?;
        return Ok(());
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let parent = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| path.to_path_buf())
        };
        std::process::Command::new("xdg-open")
            .arg(&parent)
            .spawn()
            .map_err(|e| format!("无法打开文件管理器：{e}"))?;
        Ok(())
    }
}

/// 写入系统剪贴板文本。
pub fn clipboard_set_text(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        let mut child = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("pbcopy 失败：{e}"))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| format!("写入剪贴板失败：{e}"))?;
        }
        let status = child.wait().map_err(|e| format!("pbcopy 等待失败：{e}"))?;
        if !status.success() {
            return Err("pbcopy 未成功。".into());
        }
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        use std::io::Write;
        let mut child = hidden_command("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("clip 失败：{e}"))?;
        if let Some(mut stdin) = child.stdin.take() {
            let utf16: Vec<u8> = text
                .encode_utf16()
                .flat_map(|u| u.to_le_bytes())
                .collect();
            stdin
                .write_all(&utf16)
                .map_err(|e| format!("写入剪贴板失败：{e}"))?;
        }
        let status = child.wait().map_err(|e| format!("clip 等待失败：{e}"))?;
        if !status.success() {
            return Err("clip 未成功。".into());
        }
        return Ok(());
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        use std::io::Write;
        let mut child = std::process::Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .or_else(|_| {
                std::process::Command::new("xsel")
                    .args(["--clipboard", "--input"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
            })
            .map_err(|e| format!("需要 xclip 或 xsel：{e}"))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| format!("写入剪贴板失败：{e}"))?;
        }
        let status = child.wait().map_err(|e| format!("剪贴板命令失败：{e}"))?;
        if !status.success() {
            return Err("写入剪贴板未成功。".into());
        }
        Ok(())
    }
}

/// 工作区「全部替换」确认；用户取消返回 `false`。
pub fn confirm_replace_all(file_count: usize, match_count: usize, skipped_dirty: usize) -> bool {
    let mut desc = format!(
        "确定在 {file_count} 个文件中替换 {match_count} 处匹配？此操作将直接写入磁盘，不可撤销。"
    );
    if skipped_dirty > 0 {
        desc.push_str(&format!(
            "\n另有 {skipped_dirty} 个文件因有未保存修改而被跳过。"
        ));
    }
    let result = rfd::MessageDialog::new()
        .set_title("全部替换")
        .set_description(desc)
        .set_buttons(rfd::MessageButtons::OkCancel)
        .set_level(rfd::MessageLevel::Warning)
        .show();
    matches!(result, rfd::MessageDialogResult::Ok)
}

/// 删除确认对话框；用户取消返回 `false`。
pub fn confirm_delete(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_else(|| path.to_str().unwrap_or("该项"));
    let desc = if path.is_dir() {
        format!("确定删除文件夹「{name}」及其全部内容？此操作不可撤销。")
    } else {
        format!("确定删除「{name}」？此操作不可撤销。")
    };
    let result = rfd::MessageDialog::new()
        .set_title("删除")
        .set_description(desc)
        .set_buttons(rfd::MessageButtons::OkCancel)
        .set_level(rfd::MessageLevel::Warning)
        .show();
    matches!(result, rfd::MessageDialogResult::Ok)
}

/// 工作区 `extensions/` 绝对路径（若仅有旧名 `extension/` 则沿用）。
pub fn extension_root() -> PathBuf {
    let ws = workspace_root();
    let modern = ws.join("extensions");
    if modern.is_dir() {
        return modern;
    }
    let legacy = ws.join("extension");
    if legacy.is_dir() {
        return legacy;
    }
    modern
}

/// 在工作区 `extensions/{name}/` 下脚手架内部插件（目录 + README + manifest stub）。
pub fn scaffold_extension_plugin(display_name: &str) -> Result<PathBuf, String> {
    let label = display_name.trim();
    let label = if label.is_empty() { "my-plugin" } else { label };
    let dir_name = sanitize_child_name(label)?;
    let root = extension_root();
    fs::create_dir_all(&root).map_err(|e| format!("无法创建 extensions 目录：{e}"))?;
    let target = root.join(dir_name);
    if target.exists() {
        return Err(format!("已存在：{}", target.display()));
    }
    fs::create_dir_all(&target).map_err(|e| format!("创建插件目录失败：{e}"))?;

    let readme = format!(
        "# {label}\n\n内部插件脚手架（`extensions/{dir_name}`）。\n\n包格式与加载流程见仓库 `extensions/README.md`。\n"
    );
    fs::write(target.join("README.md"), readme).map_err(|e| format!("写入 README 失败：{e}"))?;

    let safe_label = label.replace('\\', "\\\\").replace('"', "\\\"");
    let safe_id = dir_name.replace('\\', "\\\\").replace('"', "\\\"");
    let manifest = format!(
        "{{\n  \"id\": \"{safe_id}\",\n  \"name\": \"{safe_label}\",\n  \"version\": \"0.1.0\",\n  \"description\": \"\"\n}}\n"
    );
    fs::write(target.join("manifest.json"), manifest)
        .map_err(|e| format!("写入 manifest 失败：{e}"))?;

    Ok(target.canonicalize().unwrap_or(target))
}
