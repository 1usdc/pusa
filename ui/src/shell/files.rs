//! 侧栏文件管理：桌面浏览工作区；Web 端提示不可用。

use std::collections::{HashMap, HashSet};
use std::path::Path;

use dioxus::html::point_interaction::ModifiersInteraction;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdChevronDown, LdChevronRight, LdFile, LdFilePlus, LdFolder, LdFolderPlus, LdRefreshCw,
};
use keyboard_types::{Key, Modifiers};

use crate::icons::AiOutlineVerticalAlignBottom;

/// 目录项（跨平台 DTO）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FsEntryDto {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsCreateKind {
    File,
    Folder,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_available() -> bool {
    true
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_available() -> bool {
    false
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_workspace_root() -> String {
    crate::desktop::files::workspace_root()
        .to_string_lossy()
        .into_owned()
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_workspace_root() -> String {
    String::new()
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_root_display_name(root: &str) -> String {
    crate::desktop::files::root_display_name(Path::new(root))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_root_display_name(_root: &str) -> String {
    "工作区".into()
}

/// 工作区 git 分支与 dirty 状态；非 git / 不可用时返回 `None`。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_git_head_status(root: &str) -> Option<(String, bool)> {
    crate::desktop::files::git_head_status(Path::new(root)).map(|s| (s.branch, s.dirty))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_git_head_status(_root: &str) -> Option<(String, bool)> {
    None
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_list_dir(path: &str) -> Result<Vec<FsEntryDto>, String> {
    crate::desktop::files::list_dir(Path::new(path)).map(|entries| {
        entries
            .into_iter()
            .map(|e| FsEntryDto {
                name: e.name,
                path: e.path,
                is_dir: e.is_dir,
            })
            .collect()
    })
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_list_dir(_path: &str) -> Result<Vec<FsEntryDto>, String> {
    Err("Web 端无法访问本机文件系统。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_read_text(path: &str) -> Result<String, String> {
    crate::desktop::files::read_text_file(Path::new(path))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_read_text(_path: &str) -> Result<String, String> {
    Err("Web 端无法读取本机文件。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_write_text(path: &str, content: &str) -> Result<(), String> {
    crate::desktop::files::write_text_file(Path::new(path), content)
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_write_text(_path: &str, _content: &str) -> Result<(), String> {
    Err("Web 端无法写入本机文件。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_create_file(parent: &str, name: &str) -> Result<String, String> {
    crate::desktop::files::create_file(Path::new(parent), name)
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_create_file(_parent: &str, _name: &str) -> Result<String, String> {
    Err("Web 端无法创建文件。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_create_folder(parent: &str, name: &str) -> Result<String, String> {
    crate::desktop::files::create_folder(Path::new(parent), name)
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_create_folder(_parent: &str, _name: &str) -> Result<String, String> {
    Err("Web 端无法创建文件夹。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_rename(path: &str, new_name: &str) -> Result<String, String> {
    crate::desktop::files::rename_entry(Path::new(path), new_name)
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_rename(_path: &str, _new_name: &str) -> Result<String, String> {
    Err("Web 端无法重命名。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_delete(path: &str) -> Result<(), String> {
    crate::desktop::files::delete_entry(Path::new(path))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_delete(_path: &str) -> Result<(), String> {
    Err("Web 端无法删除。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_confirm_replace_all(file_count: usize, match_count: usize, skipped_dirty: usize) -> bool {
    crate::desktop::files::confirm_replace_all(file_count, match_count, skipped_dirty)
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_confirm_replace_all(
    _file_count: usize,
    _match_count: usize,
    _skipped_dirty: usize,
) -> bool {
    false
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_confirm_delete(path: &str) -> bool {
    crate::desktop::files::confirm_delete(Path::new(path))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_confirm_delete(_path: &str) -> bool {
    false
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_reveal_in_os(path: &str) -> Result<(), String> {
    crate::desktop::files::reveal_in_os(Path::new(path))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_reveal_in_os(_path: &str) -> Result<(), String> {
    Err("Web 端无法在系统中显示。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_copy_into(src: &str, dest_dir: &str) -> Result<String, String> {
    crate::desktop::files::copy_entry_into(Path::new(src), Path::new(dest_dir))
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_copy_into(_src: &str, _dest_dir: &str) -> Result<String, String> {
    Err("Web 端无法复制。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_move_into(src: &str, dest_dir: &str) -> Result<String, String> {
    crate::desktop::files::move_entry_into(Path::new(src), Path::new(dest_dir))
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_move_into(_src: &str, _dest_dir: &str) -> Result<String, String> {
    Err("Web 端无法移动。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_clipboard_set_text(text: &str) -> Result<(), String> {
    crate::desktop::files::clipboard_set_text(text)
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_clipboard_set_text(text: &str) -> Result<(), String> {
    let _ = text;
    Err("当前环境无法写入剪贴板。".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_open_in_browser(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    let url = if path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else {
        format!("file://{}", p.display())
    };
    webbrowser::open(&url).map_err(|e| format!("无法在浏览器打开：{e}"))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_open_in_browser(_path: &str) -> Result<(), String> {
    Err("Web 端请直接使用浏览器打开。".into())
}

/// 系统文件管理器显示文案。
pub fn fs_reveal_label() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "在 Finder 中显示"
    }
    #[cfg(target_os = "windows")]
    {
        "在文件资源管理器中显示"
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        "在文件管理器中显示"
    }
}

/// 相对工作区根的路径；不在根下则返回绝对路径。
pub fn fs_relative_to_root(root: &str, path: &str) -> String {
    let Ok(rel) = Path::new(path).strip_prefix(Path::new(root)) else {
        return path.to_string();
    };
    let s = rel.to_string_lossy().into_owned();
    if s.is_empty() { ".".into() } else { s }
}

/// 最近打开的项目目录（最多 5 条，仅目录）。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_recent_project_dirs() -> Vec<String> {
    crate::desktop::files::recent_project_dirs()
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_recent_project_dirs() -> Vec<String> {
    Vec::new()
}

/// 切换工作区根目录；成功返回规范化路径。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_set_workspace_root(path: &str) -> Result<String, String> {
    crate::desktop::files::set_workspace_root(Path::new(path))
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_set_workspace_root(_path: &str) -> Result<String, String> {
    Err("Web 端无法打开本机项目目录。".into())
}

/// 弹出系统目录选择器并打开为当前项目；`Ok(None)` 表示用户取消。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_open_project_directory_dialog() -> Result<Option<String>, String> {
    crate::desktop::files::open_project_directory_dialog()
        .map(|opt| opt.map(|p| p.to_string_lossy().into_owned()))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_open_project_directory_dialog() -> Result<Option<String>, String> {
    Err("Web 端无法打开本机项目目录。".into())
}

/// 在工作区 `extensions/` 下脚手架内部插件；成功返回插件目录绝对路径。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_scaffold_extension_plugin(name: &str) -> Result<String, String> {
    crate::desktop::files::scaffold_extension_plugin(name).map(|p| p.to_string_lossy().into_owned())
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_scaffold_extension_plugin(_name: &str) -> Result<String, String> {
    Err("Web 端无法创建本机插件，请使用桌面版。".into())
}

/// 在工作区 `applications/` 下脚手架本地应用；成功返回应用目录绝对路径。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_scaffold_local_application(name: &str) -> Result<String, String> {
    crate::desktop::plugins::scaffold_local_application(name)
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_scaffold_local_application(_name: &str) -> Result<String, String> {
    Err("Web 端无法创建本机应用，请使用桌面版。".into())
}

pub fn fs_file_title(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

/// 文本行数（编辑器行号）：空文件为 1；末尾换行计入下一空行。
pub fn count_text_lines(text: &str) -> usize {
    if text.is_empty() {
        return 1;
    }
    let n = text.lines().count();
    if text.ends_with('\n') {
        n + 1
    } else {
        n.max(1)
    }
}

/// 将工具参数中的相对路径解析为工作区内路径（绝对路径原样返回）。
pub fn fs_resolve_tool_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let p = Path::new(trimmed);
    if p.is_absolute() {
        return trimmed.to_string();
    }
    let root = fs_workspace_root();
    if root.is_empty() {
        return trimmed.to_string();
    }
    Path::new(&root)
        .join(trimmed)
        .to_string_lossy()
        .into_owned()
}

/// 在侧栏文件树中展开 `path` 的所有祖先目录并预热缓存，使目标行可见。
pub fn fs_reveal_in_tree(
    root: &str,
    path: &str,
    expanded: &mut HashSet<String>,
    cache: &mut HashMap<String, Result<Vec<FsEntryDto>, String>>,
) {
    if root.is_empty() || path.is_empty() {
        return;
    }
    ensure_dir_cached(cache, root);
    let root_p = Path::new(root);
    let path_p = Path::new(path);
    let Ok(rel) = path_p.strip_prefix(root_p) else {
        return;
    };
    let mut acc = root_p.to_path_buf();
    let comps: Vec<_> = rel.components().collect();
    if comps.is_empty() {
        return;
    }
    // 展开到目标的父目录；末段本身（文件或选中目录）不必展开。
    for (i, comp) in comps.iter().enumerate() {
        acc.push(comp);
        if i + 1 >= comps.len() {
            break;
        }
        let dir = acc.to_string_lossy().into_owned();
        expanded.insert(dir.clone());
        ensure_dir_cached(cache, &dir);
    }
}

/// 构建产物 / VCS / 依赖目录：仍显示但弱化。
pub fn fs_name_is_muted(name: &str) -> bool {
    matches!(name, "target" | "node_modules" | ".git")
}

/// 刷新某一目录缓存；若 `dir` 为根则只写根。
pub fn fs_cache_reload(cache: &mut HashMap<String, Result<Vec<FsEntryDto>, String>>, dir: &str) {
    cache.insert(dir.to_string(), fs_list_dir(dir));
}

/// 刷新工作区根与当前已展开目录；仅在列表内容变化时写入 cache。
/// 保留未展开目录的缓存条目，避免不必要的整树清空。
/// 返回是否有任何目录发生了更新。
pub fn fs_cache_reload_open(
    cache: &mut HashMap<String, Result<Vec<FsEntryDto>, String>>,
    root: &str,
    expanded: &HashSet<String>,
) -> bool {
    if root.is_empty() {
        return false;
    }
    let mut changed = false;
    for dir in fs_open_dirs(root, expanded) {
        let fresh = fs_list_dir(&dir);
        let differs = match cache.get(&dir) {
            Some(prev) => prev != &fresh,
            None => true,
        };
        if differs {
            cache.insert(dir, fresh);
            changed = true;
        }
    }
    changed
}

/// 根 + 已展开目录是否相对 cache 已过期（不修改 cache）。
pub fn fs_cache_open_is_stale(
    cache: &HashMap<String, Result<Vec<FsEntryDto>, String>>,
    root: &str,
    expanded: &HashSet<String>,
) -> bool {
    if root.is_empty() {
        return false;
    }
    for dir in fs_open_dirs(root, expanded) {
        let fresh = fs_list_dir(&dir);
        match cache.get(&dir) {
            Some(prev) if prev == &fresh => {}
            _ => return true,
        }
    }
    false
}

fn fs_open_dirs(root: &str, expanded: &HashSet<String>) -> HashSet<String> {
    let mut dirs = HashSet::new();
    dirs.insert(root.to_string());
    for dir in expanded {
        if !dir.is_empty() {
            dirs.insert(dir.clone());
        }
    }
    dirs
}

/// 桌面端文件树自动轮询间隔（与状态栏工作区轮询同量级）。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
const FS_TREE_POLL_MS: u64 = 800;

/// 将展开集合与缓存铺平成可见行（depth 从 0）。
pub fn fs_visible_rows(
    root: &str,
    expanded: &HashSet<String>,
    cache: &HashMap<String, Result<Vec<FsEntryDto>, String>>,
) -> Vec<(FsEntryDto, u32)> {
    let mut out = Vec::new();
    fn walk(
        parent: &str,
        depth: u32,
        expanded: &HashSet<String>,
        cache: &HashMap<String, Result<Vec<FsEntryDto>, String>>,
        out: &mut Vec<(FsEntryDto, u32)>,
    ) {
        let Some(Ok(entries)) = cache.get(parent) else {
            return;
        };
        for entry in entries {
            out.push((entry.clone(), depth));
            if entry.is_dir && expanded.contains(&entry.path) {
                walk(&entry.path, depth + 1, expanded, cache, out);
            }
        }
    }
    walk(root, 0, expanded, cache, &mut out);
    out
}

fn ensure_dir_cached(cache: &mut HashMap<String, Result<Vec<FsEntryDto>, String>>, dir: &str) {
    match cache.get(dir) {
        Some(Ok(_)) => {}
        _ => fs_cache_reload(cache, dir),
    }
}

/// 展开目录时强制重新列举，避免折叠期间外部门改导致缓存过期。
fn reload_dir_on_expand(cache: &mut HashMap<String, Result<Vec<FsEntryDto>, String>>, dir: &str) {
    fs_cache_reload(cache, dir);
}

fn resolve_create_parent(
    selected: Option<String>,
    root: &str,
    cache: &HashMap<String, Result<Vec<FsEntryDto>, String>>,
) -> String {
    let Some(selected) = selected else {
        return root.to_string();
    };
    if entry_is_dir(&selected, root, cache) {
        return selected;
    }
    Path::new(&selected)
        .parent()
        .map(|par| par.to_string_lossy().into_owned())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| root.to_string())
}

fn entry_is_dir(
    path: &str,
    root: &str,
    cache: &HashMap<String, Result<Vec<FsEntryDto>, String>>,
) -> bool {
    if path == root {
        return true;
    }
    let parent = Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.to_string());
    if let Some(Ok(entries)) = cache.get(&parent) {
        if let Some(entry) = entries.iter().find(|e| e.path == path) {
            return entry.is_dir;
        }
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        return Path::new(path).is_dir();
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        let _ = path;
        false
    }
}

/// 文件树右键菜单锚点。
#[derive(Clone, Debug, PartialEq)]
struct FsCtxMenu {
    path: String,
    is_dir: bool,
    x: f64,
    y: f64,
}

/// 文件剪切/复制板（资源管理器内粘贴用）。
#[derive(Clone, Debug, PartialEq)]
struct FsFileClipboard {
    paths: Vec<String>,
    cut: bool,
}

/// 侧栏「文件」面板。
#[component]
pub fn SidebarFileExplorer(
    root_path: String,
    mut section_open: Signal<bool>,
    mut expanded: Signal<HashSet<String>>,
    mut children_cache: Signal<HashMap<String, Result<Vec<FsEntryDto>, String>>>,
    mut selected_path: Signal<Option<String>>,
    mut create_mode: Signal<Option<FsCreateKind>>,
    mut create_name: Signal<String>,
    mut create_parent: Signal<Option<String>>,
    mut notice: Signal<Option<String>>,
    /// 最近写入成功的路径，短暂高亮。
    highlighted_paths: Signal<HashSet<String>>,
    on_open_file: EventHandler<String>,
    /// 打开终端并 cd 到目录（文件则父目录）。
    on_open_in_terminal: EventHandler<String>,
    /// 将路径加入当前 pusa 聊天草稿（支持多选）。
    on_add_to_chat: EventHandler<Vec<String>>,
    /// 新建 pusa 对话并加入路径（支持多选）。
    on_add_to_new_chat: EventHandler<Vec<String>>,
) -> Element {
    let root_name = fs_root_display_name(&root_path);
    let root_new_file = root_path.clone();
    let root_new_folder = root_path.clone();
    let root_refresh = root_path.clone();
    let root_create = root_path.clone();
    let root_tree = root_path.clone();
    let root_for_menu = root_path.clone();

    let mut ctx_menu = use_signal(|| None::<FsCtxMenu>);
    let mut file_clipboard = use_signal(|| None::<FsFileClipboard>);
    let mut renaming_path = use_signal(|| None::<String>);
    let mut rename_draft = use_signal(String::new);
    // Shift 范围多选；`selected_path` 为主选中 / 锚点。
    let mut selected_paths = use_signal(HashSet::<String>::new);

    // 外部改主选中（打开文件、chip 激活等）时，若新路径不在多选集合内则收敛为单选。
    use_effect(move || match selected_path() {
        Some(p) => {
            selected_paths.with_mut(|set| {
                if !set.contains(&p) {
                    set.clear();
                    set.insert(p);
                }
            });
        }
        None => selected_paths.set(HashSet::new()),
    });

    use_effect({
        let root = root_path.clone();
        move || {
            if !fs_available() || root.is_empty() {
                return;
            }
            children_cache.with_mut(|cache| ensure_dir_cached(cache, &root));
        }
    });

    // 桌面端：轮询工作区目录变化，自动刷新已展开子树（保留展开/选中；仅内容变化时写 cache）。
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        let mut poll_tick = use_signal(|| 0_u64);
        use_hook(|| {
            spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(FS_TREE_POLL_MS)).await;
                    poll_tick.with_mut(|n| *n = n.wrapping_add(1));
                }
            });
        });
        use_effect(move || {
            let _tick = poll_tick();
            if !fs_available() {
                return;
            }
            // 每次 tick 读当前工作区根，避免闭包捕获过期路径。
            let root = fs_workspace_root();
            if root.is_empty() {
                return;
            }
            let open = expanded();
            // 先只读比对，避免无变化时 with_mut 触发无意义重渲染。
            let stale = fs_cache_open_is_stale(&children_cache.peek(), &root, &open);
            if !stale {
                return;
            }
            children_cache.with_mut(|cache| {
                let _ = fs_cache_reload_open(cache, &root, &open);
            });
        });
    }

    if !fs_available() {
        return rsx! {
            div { class: "ac-sidebar-files",
                div { class: "ac-sidebar-files-unavailable",
                    p { "Web 端无法浏览本机文件。" }
                    p { class: "ac-sidebar-files-unavailable-hint",
                        "请使用桌面应用打开工作区文件树。"
                    }
                }
            }
        };
    }

    let rows = fs_visible_rows(&root_path, &expanded(), &children_cache());
    let creating = create_mode();
    let selected_paths_now = selected_paths();

    rsx! {
        div {
            class: if ctx_menu().is_some() {
                "ac-sidebar-files ac-fs-ctx-open"
            } else {
                "ac-sidebar-files"
            },
            div { class: "ac-sidebar-files-header",
                button {
                    r#type: "button",
                    class: "ac-sidebar-files-title-btn",
                    title: "展开 / 收起文件树",
                    aria_expanded: section_open(),
                    onclick: move |_| {
                        section_open.with_mut(|v| *v = !*v);
                    },
                    span { class: "ac-sidebar-files-chevron",
                        if section_open() {
                            Icon { icon: LdChevronDown, width: 14, height: 14, fill: "currentColor" }
                        } else {
                            Icon { icon: LdChevronRight, width: 14, height: 14, fill: "currentColor" }
                        }
                    }
                    span { class: "ac-sidebar-files-root-name", "{root_name}" }
                }
                div { class: "ac-sidebar-files-actions",
                    button {
                        r#type: "button",
                        class: "ac-sidebar-files-action",
                        title: "新建文件夹",
                        aria_label: "新建文件夹",
                        onclick: move |_| {
                            let parent = resolve_create_parent(
                                selected_path(),
                                &root_new_folder,
                                &children_cache(),
                            );
                            create_parent.set(Some(parent));
                            create_name.set(String::new());
                            create_mode.set(Some(FsCreateKind::Folder));
                            notice.set(None);
                            section_open.set(true);
                        },
                        Icon { icon: LdFolderPlus, width: 14, height: 14, fill: "currentColor" }
                    }
                    button {
                        r#type: "button",
                        class: "ac-sidebar-files-action",
                        title: "新建文件",
                        aria_label: "新建文件",
                        onclick: move |_| {
                            let parent = resolve_create_parent(
                                selected_path(),
                                &root_new_file,
                                &children_cache(),
                            );
                            create_parent.set(Some(parent));
                            create_name.set(String::new());
                            create_mode.set(Some(FsCreateKind::File));
                            notice.set(None);
                            section_open.set(true);
                        },
                        Icon { icon: LdFilePlus, width: 14, height: 14, fill: "currentColor" }
                    }
                    button {
                        r#type: "button",
                        class: "ac-sidebar-files-action",
                        title: "刷新",
                        aria_label: "刷新",
                        onclick: move |_| {
                            let root = root_refresh.clone();
                            let open = expanded();
                            children_cache.with_mut(|cache| {
                                cache.clear();
                                let _ = fs_cache_reload_open(cache, &root, &open);
                            });
                            notice.set(None);
                        },
                        Icon { icon: LdRefreshCw, width: 14, height: 14, fill: "currentColor" }
                    }
                    button {
                        r#type: "button",
                        class: "ac-sidebar-files-action",
                        title: "全部折叠",
                        aria_label: "全部折叠",
                        onclick: move |_| {
                            expanded.set(HashSet::new());
                        },
                        Icon { icon: AiOutlineVerticalAlignBottom, width: 14, height: 14, fill: "currentColor" }
                    }
                }
            }

            if let Some(msg) = notice() {
                div { class: "ac-sidebar-files-notice", "{msg}" }
            }

            if section_open() {
                if let Some(kind) = creating {
                    div { class: "ac-sidebar-files-create",
                        span { class: "ac-sidebar-files-create-label",
                            match kind {
                                FsCreateKind::File => "新建文件",
                                FsCreateKind::Folder => "新建文件夹",
                            }
                        }
                        if let Some(parent) = create_parent() {
                            span { class: "ac-sidebar-files-create-parent", title: "{parent}",
                                "{fs_file_title(&parent)}"
                            }
                        }
                        input {
                            r#type: "text",
                            class: "ac-sidebar-files-create-input",
                            placeholder: if kind == FsCreateKind::File { "文件名…" } else { "文件夹名…" },
                            value: "{create_name()}",
                            autofocus: true,
                            oninput: move |e| create_name.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Escape {
                                    e.prevent_default();
                                    create_mode.set(None);
                                    create_name.set(String::new());
                                    return;
                                }
                                if e.key() != Key::Enter {
                                    return;
                                }
                                e.prevent_default();
                                let name = create_name();
                                let parent = create_parent()
                                    .unwrap_or_else(|| root_create.clone());
                                let result = match kind {
                                    FsCreateKind::File => fs_create_file(&parent, &name),
                                    FsCreateKind::Folder => fs_create_folder(&parent, &name),
                                };
                                match result {
                                    Ok(created) => {
                                        children_cache.with_mut(|cache| {
                                            fs_cache_reload(cache, &parent);
                                        });
                                        expanded.with_mut(|set| {
                                            set.insert(parent.clone());
                                        });
                                        selected_path.set(Some(created.clone()));
                                        create_mode.set(None);
                                        create_name.set(String::new());
                                        notice.set(None);
                                        if kind == FsCreateKind::File {
                                            on_open_file.call(created);
                                        }
                                    }
                                    Err(err) => notice.set(Some(err)),
                                }
                            },
                        }
                        button {
                            r#type: "button",
                            class: "ac-sidebar-files-create-cancel",
                            onclick: move |_| {
                                create_mode.set(None);
                                create_name.set(String::new());
                            },
                            "取消"
                        }
                    }
                }

                div { class: "ac-sidebar-files-tree", role: "tree",
                    {
                        match children_cache().get(&root_tree) {
                            Some(Err(err)) => rsx! {
                                div { class: "ac-sidebar-files-empty is-error", "{err}" }
                            },
                            Some(Ok(entries)) if entries.is_empty() => rsx! {
                                div { class: "ac-sidebar-files-empty", "此目录为空" }
                            },
                            _ => rsx! {
                                for (entry, depth) in rows {
                                    {
                                        let path = entry.path.clone();
                                        let path_sel = entry.path.clone();
                                        let path_toggle = entry.path.clone();
                                        let path_ctx = entry.path.clone();
                                        let path_rename = entry.path.clone();
                                        let root_fallback = root_for_menu.clone();
                                        let root_shift = root_for_menu.clone();
                                        let is_dir = entry.is_dir;
                                        let is_open = expanded().contains(&entry.path);
                                        let is_selected = selected_paths_now.contains(&entry.path)
                                            || selected_path().as_deref() == Some(entry.path.as_str());
                                        let is_muted = fs_name_is_muted(&entry.name);
                                        let is_written = highlighted_paths().contains(&entry.path);
                                        let is_renaming = renaming_path().as_deref() == Some(entry.path.as_str());
                                        let row_class = match (is_selected, is_muted, is_written) {
                                            (true, true, true) => {
                                                "ac-sidebar-files-row is-selected is-muted is-just-written"
                                            }
                                            (true, true, false) => {
                                                "ac-sidebar-files-row is-selected is-muted"
                                            }
                                            (true, false, true) => {
                                                "ac-sidebar-files-row is-selected is-just-written"
                                            }
                                            (true, false, false) => "ac-sidebar-files-row is-selected",
                                            (false, true, true) => {
                                                "ac-sidebar-files-row is-muted is-just-written"
                                            }
                                            (false, true, false) => "ac-sidebar-files-row is-muted",
                                            (false, false, true) => {
                                                "ac-sidebar-files-row is-just-written"
                                            }
                                            (false, false, false) => "ac-sidebar-files-row",
                                        };
                                        // depth 0 为工作区根的直接子项；相对根标题「PUSA」明显再缩进一级。
                                        let pad = 1.15 + (depth as f64) * 0.8;
                                        let pad_style = format!("padding-left: {pad}rem");
                                        let name = entry.name.clone();
                                        rsx! {
                                            if is_renaming {
                                                div {
                                                    key: "rename-{path}",
                                                    class: "ac-sidebar-files-row is-renaming",
                                                    style: "{pad_style}",
                                                    span { class: "ac-sidebar-files-row-chevron",
                                                        if is_dir {
                                                            Icon { icon: LdChevronRight, width: 12, height: 12, fill: "currentColor" }
                                                        }
                                                    }
                                                    span { class: "ac-sidebar-files-row-icon",
                                                        if is_dir {
                                                            Icon { icon: LdFolder, width: 13, height: 13, fill: "currentColor" }
                                                        } else {
                                                            Icon { icon: LdFile, width: 13, height: 13, fill: "currentColor" }
                                                        }
                                                    }
                                                    input {
                                                        r#type: "text",
                                                        class: "ac-sidebar-files-rename-input",
                                                        value: "{rename_draft()}",
                                                        autofocus: true,
                                                        oninput: move |e| rename_draft.set(e.value()),
                                                        onkeydown: move |e: KeyboardEvent| {
                                                            if e.key() == Key::Escape {
                                                                e.prevent_default();
                                                                renaming_path.set(None);
                                                                rename_draft.set(String::new());
                                                                return;
                                                            }
                                                            if e.key() != Key::Enter {
                                                                return;
                                                            }
                                                            e.prevent_default();
                                                            let old = path_rename.clone();
                                                            let new_name = rename_draft();
                                                            match fs_rename(&old, &new_name) {
                                                                Ok(new_path) => {
                                                                    let parent = Path::new(&old)
                                                                        .parent()
                                                                        .map(|p| p.to_string_lossy().into_owned())
                                                                        .unwrap_or_else(|| root_fallback.clone());
                                                                    children_cache.with_mut(|cache| {
                                                                        fs_cache_reload(cache, &parent);
                                                                    });
                                                                    if is_dir {
                                                                        expanded.with_mut(|set| {
                                                                            set.remove(&old);
                                                                            set.insert(new_path.clone());
                                                                        });
                                                                    }
                                                                    selected_path.set(Some(new_path.clone()));
                                                                    selected_paths.set(HashSet::from([new_path]));
                                                                    renaming_path.set(None);
                                                                    rename_draft.set(String::new());
                                                                    notice.set(None);
                                                                }
                                                                Err(err) => notice.set(Some(err)),
                                                            }
                                                        },
                                                        onblur: move |_| {
                                                            renaming_path.set(None);
                                                            rename_draft.set(String::new());
                                                        },
                                                    }
                                                }
                                            } else {
                                                button {
                                                    key: "{path}",
                                                    r#type: "button",
                                                    class: "{row_class}",
                                                    style: "{pad_style}",
                                                    role: "treeitem",
                                                    title: "{path}",
                                                    onclick: move |evt| {
                                                        let shift = evt.modifiers().contains(Modifiers::SHIFT);
                                                        if shift {
                                                            // Shift：按可见行范围多选，不展开目录、不打开文件。
                                                            let visible = fs_visible_rows(
                                                                &root_shift,
                                                                &expanded(),
                                                                &children_cache(),
                                                            );
                                                            let paths: Vec<String> = visible
                                                                .iter()
                                                                .map(|(e, _)| e.path.clone())
                                                                .collect();
                                                            let click_idx = paths
                                                                .iter()
                                                                .position(|p| p == &path_sel);
                                                            let Some(click_idx) = click_idx else {
                                                                selected_path.set(Some(path_sel.clone()));
                                                                selected_paths
                                                                    .set(HashSet::from([path_sel.clone()]));
                                                                return;
                                                            };
                                                            let anchor_idx = selected_path()
                                                                .as_ref()
                                                                .and_then(|a| {
                                                                    paths.iter().position(|p| p == a)
                                                                })
                                                                .or_else(|| {
                                                                    selected_paths().iter().find_map(|p| {
                                                                        paths.iter().position(|x| x == p)
                                                                    })
                                                                })
                                                                .unwrap_or(click_idx);
                                                            let (lo, hi) = if anchor_idx <= click_idx {
                                                                (anchor_idx, click_idx)
                                                            } else {
                                                                (click_idx, anchor_idx)
                                                            };
                                                            let mut set = HashSet::new();
                                                            for p in &paths[lo..=hi] {
                                                                set.insert(p.clone());
                                                            }
                                                            selected_paths.set(set);
                                                            selected_path.set(Some(path_sel.clone()));
                                                            return;
                                                        }
                                                        selected_path.set(Some(path_sel.clone()));
                                                        selected_paths
                                                            .set(HashSet::from([path_sel.clone()]));
                                                        if is_dir {
                                                            let opening = !expanded().contains(&path_toggle);
                                                            expanded.with_mut(|set| {
                                                                if opening {
                                                                    set.insert(path_toggle.clone());
                                                                } else {
                                                                    set.remove(&path_toggle);
                                                                }
                                                            });
                                                            if opening {
                                                                children_cache.with_mut(|cache| {
                                                                    reload_dir_on_expand(cache, &path_toggle);
                                                                });
                                                            }
                                                        } else {
                                                            on_open_file.call(path_sel.clone());
                                                        }
                                                    },
                                                    oncontextmenu: move |evt| {
                                                        evt.prevent_default();
                                                        evt.stop_propagation();
                                                        let in_multi = selected_paths().contains(&path_ctx);
                                                        if !in_multi {
                                                            selected_path.set(Some(path_ctx.clone()));
                                                            selected_paths
                                                                .set(HashSet::from([path_ctx.clone()]));
                                                        } else {
                                                            selected_path.set(Some(path_ctx.clone()));
                                                        }
                                                        let coords = evt.data.client_coordinates();
                                                        ctx_menu.set(Some(FsCtxMenu {
                                                            path: path_ctx.clone(),
                                                            is_dir,
                                                            x: coords.x,
                                                            y: coords.y,
                                                        }));
                                                    },
                                                    span { class: "ac-sidebar-files-row-chevron",
                                                        if is_dir {
                                                            if is_open {
                                                                Icon { icon: LdChevronDown, width: 12, height: 12, fill: "currentColor" }
                                                            } else {
                                                                Icon { icon: LdChevronRight, width: 12, height: 12, fill: "currentColor" }
                                                            }
                                                        }
                                                    }
                                                    span { class: "ac-sidebar-files-row-icon",
                                                        if is_dir {
                                                            Icon { icon: LdFolder, width: 13, height: 13, fill: "currentColor" }
                                                        } else {
                                                            Icon { icon: LdFile, width: 13, height: 13, fill: "currentColor" }
                                                        }
                                                    }
                                                    span { class: "ac-sidebar-files-row-name", "{name}" }
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

            if let Some(ctx) = ctx_menu() {
                {
                    let path = ctx.path.clone();
                    let is_dir = ctx.is_dir;
                    let root = root_for_menu.clone();
                    let reveal_label = fs_reveal_label();
                    let has_clipboard = file_clipboard().is_some();
                    let multi = selected_paths();
                    let add_targets: Vec<String> = if multi.contains(&path) && multi.len() > 1 {
                        // 按可见树顺序输出多选路径
                        fs_visible_rows(&root, &expanded(), &children_cache())
                            .iter()
                            .map(|(e, _)| e.path.clone())
                            .filter(|p| multi.contains(p))
                            .collect()
                    } else {
                        vec![path.clone()]
                    };
                    let clip_targets = add_targets.clone();
                    let add_label = if add_targets.len() > 1 {
                        format!("将 {} 个文件加入 Chat", add_targets.len())
                    } else {
                        "加入 Chat".into()
                    };
                    let add_new_label = if add_targets.len() > 1 {
                        format!("将 {} 个文件加入新的 Chat", add_targets.len())
                    } else {
                        "加入新的 Chat".into()
                    };
                    let paste_target = if is_dir {
                        path.clone()
                    } else {
                        Path::new(&path)
                            .parent()
                            .map(|p| p.to_string_lossy().into_owned())
                            .unwrap_or_else(|| root.clone())
                    };
                    let menu_style = format!(
                        "left: {:.0}px; top: {:.0}px; right: auto;",
                        ctx.x, ctx.y
                    );
                    rsx! {
                        div {
                            class: "ac-chat-history-ctx-backdrop",
                            onclick: move |_| ctx_menu.set(None),
                            oncontextmenu: move |evt| {
                                evt.prevent_default();
                                ctx_menu.set(None);
                            },
                        }
                        div {
                            class: "ac-chat-history-ctx-menu ac-fs-ctx-menu",
                            role: "menu",
                            style: "{menu_style}",
                            onclick: move |evt| evt.stop_propagation(),

                            if !is_dir {
                                button {
                                    r#type: "button",
                                    class: "ac-chat-history-ctx-menu__item",
                                    role: "menuitem",
                                    onclick: {
                                        let path = path.clone();
                                        move |_| {
                                            ctx_menu.set(None);
                                            on_open_file.call(path.clone());
                                        }
                                    },
                                    "打开预览"
                                }
                                button {
                                    r#type: "button",
                                    class: "ac-chat-history-ctx-menu__item",
                                    role: "menuitem",
                                    onclick: {
                                        let path = path.clone();
                                        move |_| {
                                            ctx_menu.set(None);
                                            match fs_open_in_browser(&path) {
                                                Ok(()) => notice.set(None),
                                                Err(e) => notice.set(Some(e)),
                                            }
                                        }
                                    },
                                    "在浏览器中打开"
                                }
                            }
                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item",
                                role: "menuitem",
                                onclick: {
                                    let path = path.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        match fs_reveal_in_os(&path) {
                                            Ok(()) => notice.set(None),
                                            Err(e) => notice.set(Some(e)),
                                        }
                                    }
                                },
                                "{reveal_label}"
                            }
                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item",
                                role: "menuitem",
                                onclick: {
                                    let path = path.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        let cd = if is_dir {
                                            path.clone()
                                        } else {
                                            Path::new(&path)
                                                .parent()
                                                .map(|p| p.to_string_lossy().into_owned())
                                                .unwrap_or(path.clone())
                                        };
                                        on_open_in_terminal.call(cd);
                                    }
                                },
                                "在终端中打开"
                            }

                            div { class: "ac-fs-ctx-sep", role: "separator" }

                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item",
                                role: "menuitem",
                                onclick: {
                                    let targets = add_targets.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        on_add_to_chat.call(targets.clone());
                                    }
                                },
                                "{add_label}"
                            }
                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item",
                                role: "menuitem",
                                onclick: {
                                    let targets = add_targets.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        on_add_to_new_chat.call(targets.clone());
                                    }
                                },
                                "{add_new_label}"
                            }

                            div { class: "ac-fs-ctx-sep", role: "separator" }

                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item",
                                role: "menuitem",
                                onclick: {
                                    let paths = clip_targets.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        file_clipboard.set(Some(FsFileClipboard {
                                            paths: paths.clone(),
                                            cut: true,
                                        }));
                                        notice.set(Some(if paths.len() > 1 {
                                            format!("已剪切 {} 项。", paths.len())
                                        } else {
                                            "已剪切。".into()
                                        }));
                                    }
                                },
                                "剪切"
                            }
                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item",
                                role: "menuitem",
                                onclick: {
                                    let paths = clip_targets.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        file_clipboard.set(Some(FsFileClipboard {
                                            paths: paths.clone(),
                                            cut: false,
                                        }));
                                        notice.set(Some(if paths.len() > 1 {
                                            format!("已复制 {} 项。", paths.len())
                                        } else {
                                            "已复制。".into()
                                        }));
                                    }
                                },
                                "复制"
                            }
                            if has_clipboard {
                                button {
                                    r#type: "button",
                                    class: "ac-chat-history-ctx-menu__item",
                                    role: "menuitem",
                                    onclick: {
                                        let dest = paste_target.clone();
                                        move |_| {
                                            ctx_menu.set(None);
                                            let Some(clip) = file_clipboard() else {
                                                return;
                                            };
                                            let mut err = None;
                                            for src in &clip.paths {
                                                let result = if clip.cut {
                                                    fs_move_into(src, &dest)
                                                } else {
                                                    fs_copy_into(src, &dest)
                                                };
                                                if let Err(e) = result {
                                                    err = Some(e);
                                                    break;
                                                }
                                            }
                                            children_cache.with_mut(|cache| {
                                                fs_cache_reload(cache, &dest);
                                                if clip.cut {
                                                    for src in &clip.paths {
                                                        if let Some(parent) = Path::new(src).parent() {
                                                            fs_cache_reload(
                                                                cache,
                                                                &parent.to_string_lossy(),
                                                            );
                                                        }
                                                    }
                                                }
                                            });
                                            if clip.cut {
                                                file_clipboard.set(None);
                                            }
                                            notice.set(err.or_else(|| Some("已粘贴。".into())));
                                        }
                                    },
                                    "粘贴"
                                }
                            }

                            div { class: "ac-fs-ctx-sep", role: "separator" }

                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item",
                                role: "menuitem",
                                onclick: {
                                    let path = path.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        match fs_clipboard_set_text(&path) {
                                            Ok(()) => notice.set(Some("已复制路径。".into())),
                                            Err(e) => notice.set(Some(e)),
                                        }
                                    }
                                },
                                "复制路径"
                            }
                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item",
                                role: "menuitem",
                                onclick: {
                                    let path = path.clone();
                                    let root = root.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        let rel = fs_relative_to_root(&root, &path);
                                        match fs_clipboard_set_text(&rel) {
                                            Ok(()) => notice.set(Some("已复制相对路径。".into())),
                                            Err(e) => notice.set(Some(e)),
                                        }
                                    }
                                },
                                "复制相对路径"
                            }

                            div { class: "ac-fs-ctx-sep", role: "separator" }

                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item",
                                role: "menuitem",
                                onclick: {
                                    let path = path.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        let name = fs_file_title(&path);
                                        rename_draft.set(name);
                                        renaming_path.set(Some(path.clone()));
                                    }
                                },
                                "重命名…"
                            }
                            button {
                                r#type: "button",
                                class: "ac-chat-history-ctx-menu__item is-danger",
                                role: "menuitem",
                                onclick: {
                                    let path = path.clone();
                                    let root = root.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        if !fs_confirm_delete(&path) {
                                            return;
                                        }
                                        match fs_delete(&path) {
                                            Ok(()) => {
                                                let parent = Path::new(&path)
                                                    .parent()
                                                    .map(|p| p.to_string_lossy().into_owned())
                                                    .unwrap_or_else(|| root.clone());
                                                children_cache.with_mut(|cache| {
                                                    fs_cache_reload(cache, &parent);
                                                });
                                                expanded.with_mut(|set| {
                                                    set.remove(&path);
                                                });
                                                if selected_path().as_deref() == Some(path.as_str()) {
                                                    selected_path.set(None);
                                                }
                                                selected_paths.with_mut(|set| {
                                                    set.remove(&path);
                                                });
                                                notice.set(None);
                                            }
                                            Err(e) => notice.set(Some(e)),
                                        }
                                    }
                                },
                                "删除"
                            }
                        }
                    }
                }
            }
        }
    }
}
fn file_editor_try_save(
    path: &str,
    drafts: Signal<HashMap<String, String>>,
    mut baselines: Signal<HashMap<String, String>>,
    mut dirty: Signal<HashSet<String>>,
    mut save_notice: Signal<Option<String>>,
) {
    let Some(body) = drafts().get(path).cloned() else {
        return;
    };
    match fs_write_text(path, &body) {
        Ok(()) => {
            baselines.with_mut(|m| {
                m.insert(path.to_string(), body);
            });
            dirty.with_mut(|s| {
                s.remove(path);
            });
            save_notice.set(Some("已保存".into()));
        }
        Err(e) => save_notice.set(Some(e)),
    }
}

fn is_save_shortcut(e: &KeyboardEvent) -> bool {
    let Key::Character(c) = e.key() else {
        return false;
    };
    if !c.eq_ignore_ascii_case("s") {
        return false;
    }
    let mods = e.modifiers();
    // macOS Cmd ≈ SUPER；Windows/Linux 用 CONTROL。也接受 META。
    mods.contains(Modifiers::SUPER)
        || mods.contains(Modifiers::CONTROL)
        || mods.contains(Modifiers::META)
}

/// 中间栏文本文件查看 / 编辑。
///
/// `baselines` 保存上次加载或成功保存时的内容；编辑时与当前草稿比较以决定 dirty。
/// 保存仅通过 Cmd/Ctrl+S，无可见保存按钮。
#[component]
pub fn FileEditorPane(
    path: String,
    mut drafts: Signal<HashMap<String, String>>,
    mut baselines: Signal<HashMap<String, String>>,
    mut dirty: Signal<HashSet<String>>,
    mut load_errors: Signal<HashMap<String, String>>,
    mut save_notice: Signal<Option<String>>,
) -> Element {
    let path_key = path.clone();

    use_effect({
        let path = path.clone();
        move || {
            if drafts().contains_key(&path) || load_errors().contains_key(&path) {
                return;
            }
            match fs_read_text(&path) {
                Ok(content) => {
                    drafts.with_mut(|m| {
                        m.insert(path.clone(), content.clone());
                    });
                    baselines.with_mut(|m| {
                        m.insert(path.clone(), content);
                    });
                    load_errors.with_mut(|m| {
                        m.remove(&path);
                    });
                }
                Err(err) => {
                    load_errors.with_mut(|m| {
                        m.insert(path.clone(), err);
                    });
                }
            }
        }
    });

    let err = load_errors().get(&path_key).cloned();
    let content = drafts().get(&path_key).cloned();
    let is_dirty = dirty().contains(&path_key);
    let notice = save_notice();
    let show_status = is_dirty || notice.is_some();
    let path_edit = path.clone();
    let path_save = path.clone();

    rsx! {
        section {
            class: "ac-file-editor",
            onkeydown: move |e: KeyboardEvent| {
                if !is_save_shortcut(&e) {
                    return;
                }
                e.prevent_default();
                file_editor_try_save(
                    &path_save,
                    drafts,
                    baselines,
                    dirty,
                    save_notice,
                );
            },
            if show_status {
                div { class: "ac-file-editor-toolbar",
                    div { class: "ac-file-editor-actions",
                        if is_dirty {
                            span { class: "ac-file-editor-dirty", "未保存" }
                        }
                        if let Some(msg) = notice {
                            span { class: "ac-file-editor-save-notice", "{msg}" }
                        }
                    }
                }
            }
            if let Some(err) = err {
                div { class: "ac-file-editor-error", "{err}" }
            } else if let Some(body) = content {
                {
                    let line_count = count_text_lines(&body);
                    let code_style = format!("--ac-editor-lines: {line_count}");
                    let lang = super::syntax::language_from_path(&path_key);
                    let mut highlight = super::syntax::highlight_html(&body, lang);
                    if !body.ends_with('\n') && !body.is_empty() {
                        highlight.push('\n');
                    }
                    rsx! {
                        div { class: "ac-file-editor-scroll",
                            div {
                                class: "ac-file-editor-code",
                                style: "{code_style}",
                                div {
                                    class: "ac-file-editor-gutter",
                                    aria_hidden: "true",
                                    for n in 1..=line_count {
                                        div {
                                            key: "{n}",
                                            class: "ac-file-editor-ln",
                                            "{n}"
                                        }
                                    }
                                }
                                div { class: "ac-file-editor-surface",
                                    pre {
                                        class: "ac-file-editor-highlight",
                                        dangerous_inner_html: "{highlight}",
                                    }
                                    textarea {
                                        class: "ac-file-editor-textarea",
                                        value: "{body}",
                                        spellcheck: "false",
                                        oninput: move |e| {
                                            let v = e.value();
                                            let matches_baseline = baselines()
                                                .get(&path_edit)
                                                .is_some_and(|b| b == &v);
                                            drafts.with_mut(|m| {
                                                m.insert(path_edit.clone(), v);
                                            });
                                            dirty.with_mut(|s| {
                                                if matches_baseline {
                                                    s.remove(&path_edit);
                                                } else {
                                                    s.insert(path_edit.clone());
                                                }
                                            });
                                            save_notice.set(None);
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "ac-file-editor-loading", "正在读取…" }
            }
        }
    }
}

/// 行内 diff 行类型。
#[derive(Clone, Copy, PartialEq, Eq)]
enum DiffLineKind {
    Context,
    Delete,
    Insert,
}

/// 单行 diff：旧/新行号 + 标记 + 文本。
#[derive(Clone, PartialEq)]
struct DiffLineView {
    kind: DiffLineKind,
    old_no: Option<u32>,
    new_no: Option<u32>,
    text: String,
}

fn line_number_at_needle(haystack: &str, needle: &str) -> Option<u32> {
    if needle.is_empty() {
        return None;
    }
    let idx = haystack.find(needle)?;
    Some(haystack[..idx].bytes().filter(|&b| b == b'\n').count() as u32 + 1)
}

/// 尝试用磁盘文件定位片段起始行号（优先 old，其次 new）；失败则从 1 起。
fn snippet_start_line(path: &str, old_text: &str, new_text: &str) -> u32 {
    let Ok(content) = fs_read_text(path) else {
        return 1;
    };
    line_number_at_needle(&content, old_text)
        .or_else(|| line_number_at_needle(&content, new_text))
        .unwrap_or(1)
}

fn push_dump_diff(
    rows: &mut Vec<DiffLineView>,
    old_lines: &[&str],
    new_lines: &[&str],
    start_line: u32,
) {
    let mut old_no = start_line;
    let mut new_no = start_line;
    for line in old_lines {
        rows.push(DiffLineView {
            kind: DiffLineKind::Delete,
            old_no: Some(old_no),
            new_no: None,
            text: (*line).to_string(),
        });
        old_no += 1;
    }
    for line in new_lines {
        rows.push(DiffLineView {
            kind: DiffLineKind::Insert,
            old_no: None,
            new_no: Some(new_no),
            text: (*line).to_string(),
        });
        new_no += 1;
    }
}

/// 将 old/new 片段做成 IDE 风格行内 diff（LCS；过大时退化为整段删+增）。
fn snippet_diff_lines(old_text: &str, new_text: &str, start_line: u32) -> Vec<DiffLineView> {
    let old_lines: Vec<&str> = if old_text.is_empty() {
        Vec::new()
    } else {
        old_text.lines().collect()
    };
    let new_lines: Vec<&str> = if new_text.is_empty() {
        Vec::new()
    } else {
        new_text.lines().collect()
    };
    let n = old_lines.len();
    let m = new_lines.len();
    // 片段过大时避免 O(n·m) 占太多 UI 线程。
    const MAX_LCS: usize = 600;
    let mut rows = Vec::new();
    if n == 0 && m == 0 {
        rows.push(DiffLineView {
            kind: DiffLineKind::Context,
            old_no: Some(start_line),
            new_no: Some(start_line),
            text: String::new(),
        });
        return rows;
    }
    if n > MAX_LCS || m > MAX_LCS {
        push_dump_diff(&mut rows, &old_lines, &new_lines, start_line);
        return rows;
    }

    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            dp[i][j] = if old_lines[i - 1] == new_lines[j - 1] {
                dp[i - 1][j - 1] + 1
            } else {
                dp[i - 1][j].max(dp[i][j - 1])
            };
        }
    }

    enum Op {
        Eq { oi: usize },
        Del { oi: usize },
        Ins { nj: usize },
    }
    let mut ops = Vec::new();
    let (mut i, mut j) = (n, m);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            ops.push(Op::Eq { oi: i - 1 });
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            ops.push(Op::Ins { nj: j - 1 });
            j -= 1;
        } else {
            ops.push(Op::Del { oi: i - 1 });
            i -= 1;
        }
    }
    ops.reverse();

    let mut old_no = start_line;
    let mut new_no = start_line;
    for op in ops {
        match op {
            Op::Eq { oi } => {
                rows.push(DiffLineView {
                    kind: DiffLineKind::Context,
                    old_no: Some(old_no),
                    new_no: Some(new_no),
                    text: old_lines[oi].to_string(),
                });
                old_no += 1;
                new_no += 1;
            }
            Op::Del { oi } => {
                rows.push(DiffLineView {
                    kind: DiffLineKind::Delete,
                    old_no: Some(old_no),
                    new_no: None,
                    text: old_lines[oi].to_string(),
                });
                old_no += 1;
            }
            Op::Ins { nj } => {
                rows.push(DiffLineView {
                    kind: DiffLineKind::Insert,
                    old_no: None,
                    new_no: Some(new_no),
                    text: new_lines[nj].to_string(),
                });
                new_no += 1;
            }
        }
    }
    rows
}

/// 本次工具编辑的行内 diff（数据来自工具参数 oldText/newText）。
#[component]
pub fn FileEditDiffPane(
    path: String,
    old_text: String,
    new_text: String,
    /// 桌面端可再打开当前磁盘文件。
    can_open_current: bool,
    on_open_current: EventHandler<()>,
) -> Element {
    let title = fs_file_title(&path);
    let start_line = snippet_start_line(&path, &old_text, &new_text);
    let lines = snippet_diff_lines(&old_text, &new_text, start_line);
    let is_write = old_text.is_empty() && !new_text.is_empty();
    let is_delete = !old_text.is_empty() && new_text.is_empty();
    let subtitle = if is_write {
        "本次写入（行内 diff）"
    } else if is_delete {
        "本次删除片段（行内 diff）"
    } else {
        "本次编辑（行内 diff）"
    };
    let add_n = lines
        .iter()
        .filter(|l| l.kind == DiffLineKind::Insert)
        .count();
    let del_n = lines
        .iter()
        .filter(|l| l.kind == DiffLineKind::Delete)
        .count();
    let lang = super::syntax::language_from_path(&path);

    rsx! {
        section { class: "ac-file-diff",
            div { class: "ac-file-diff-header",
                div { class: "ac-file-diff-heading",
                    h2 { class: "ac-file-diff-title", "{title}" }
                    p { class: "ac-file-diff-subtitle", "{subtitle}" }
                    p { class: "ac-file-diff-path", title: "{path}", "{path}" }
                    p { class: "ac-file-diff-stats",
                        span { class: "ac-file-diff-stat is-add", "+{add_n}" }
                        span { class: "ac-file-diff-stat is-del", "-{del_n}" }
                    }
                }
                if can_open_current {
                    button {
                        r#type: "button",
                        class: "ac-file-diff-open-btn",
                        onclick: move |_| on_open_current.call(()),
                        "打开当前文件"
                    }
                }
            }
            div { class: "ac-file-diff-inline",
                div { class: "ac-file-diff-inline-body", role: "table",
                    for (idx, line) in lines.iter().enumerate() {
                        {
                            let cls = match line.kind {
                                DiffLineKind::Delete => "ac-file-diff-line is-del",
                                DiffLineKind::Insert => "ac-file-diff-line is-add",
                                DiffLineKind::Context => "ac-file-diff-line",
                            };
                            let mark = match line.kind {
                                DiffLineKind::Delete => '-',
                                DiffLineKind::Insert => '+',
                                DiffLineKind::Context => ' ',
                            };
                            let old_ln = line
                                .old_no
                                .map(|n| n.to_string())
                                .unwrap_or_default();
                            let new_ln = line
                                .new_no
                                .map(|n| n.to_string())
                                .unwrap_or_default();
                            let highlighted =
                                super::syntax::highlight_line_html(&line.text, lang);
                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: "{cls}",
                                    role: "row",
                                    span {
                                        class: "ac-file-diff-ln is-old",
                                        aria_hidden: "true",
                                        "{old_ln}"
                                    }
                                    span {
                                        class: "ac-file-diff-ln is-new",
                                        aria_hidden: "true",
                                        "{new_ln}"
                                    }
                                    span { class: "ac-file-diff-mark", "{mark}" }
                                    span {
                                        class: "ac-file-diff-text",
                                        dangerous_inner_html: "{highlighted}",
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

/// 中间栏 Markdown 预览：优先用编辑草稿，否则读盘。
#[component]
pub fn FileMdPreviewPane(
    path: String,
    drafts: Signal<HashMap<String, String>>,
    load_errors: Signal<HashMap<String, String>>,
) -> Element {
    let path_key = path.clone();
    let content = use_memo(move || {
        if let Some(draft) = drafts().get(&path_key).cloned() {
            return Ok(draft);
        }
        if let Some(err) = load_errors().get(&path_key).cloned() {
            return Err(err);
        }
        fs_read_text(&path_key)
    });

    rsx! {
        section { class: "ac-md-preview-pane",
            match content() {
                Ok(text) if text.is_empty() => rsx! {
                    div { class: "ac-file-editor-empty", "暂无内容" }
                },
                Ok(text) => rsx! {
                    div { class: "ac-md-preview-scroll scrollbar-hide",
                        crate::chat::ChatMarkdownBody { content: text }
                    }
                },
                Err(err) => rsx! {
                    div { class: "ac-file-editor-error", "{err}" }
                },
            }
        }
    }
}
