//! 侧栏文件管理：桌面浏览工作区；Web 端提示不可用。

use std::collections::{HashMap, HashSet};
use std::path::Path;

use dioxus::html::point_interaction::ModifiersInteraction;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdCaseSensitive, LdCaseUpper, LdChevronDown, LdChevronRight, LdChevronUp, LdFile, LdFilePlus,
    LdFolder, LdFolderPlus, LdMonitor, LdPause, LdPlay, LdRefreshCw, LdRegex, LdReplace,
    LdReplaceAll, LdRotateCcw, LdSmartphone, LdWholeWord, LdX,
};
use dioxus_free_icons::Icon;
use keyboard_types::{Key, Modifiers};

use crate::icons::AiOutlineVerticalAlignBottom;

// rsx 字符串插值只支持简单标识符，拼接类名时需先 `use` 进来。
use crate::shell::style::{
    AUDIO_CARD, AUDIO_EL, AUDIO_ERROR, AUDIO_META, AUDIO_PANE, AUDIO_PLAY, AUDIO_STAGE,
    AUDIO_TITLE, AUDIO_TRACK, AUDIO_TRACK_FILL, CTX_BACKDROP, CTX_ITEM, CTX_ITEM_DANGER, CTX_MENU,
    FILES_ACTION, FILES_EMPTY, FILES_EMPTY_ERROR, FILES_PANEL, FILES_ROW_CHEVRON, FILES_ROW_ICON,
    FILES_ROW_NAME, FS_CTX_MENU, FS_CTX_SEP, MD_PREVIEW_PANE, MD_PREVIEW_SCROLL, PREVIEW_CHIP,
    PREVIEW_DEVICE, PREVIEW_DEVICE_BODY, PREVIEW_DEVICE_SLOT, PREVIEW_DRAWER,
    PREVIEW_DRAWER_MASK, PREVIEW_DRAWER_TITLE, PREVIEW_FAB, PREVIEW_FAB_BALL,
    PREVIEW_FAB_BALL_SECONDARY, PREVIEW_FRAME, PREVIEW_FRAME_FILL, PREVIEW_IMAGE, PREVIEW_MODEL,
    PREVIEW_MODEL_NAME, PREVIEW_MODEL_SIZE, PREVIEW_PANE, PREVIEW_PANE_SELECTABLE,
    PREVIEW_PHONE_BAR, PREVIEW_RESIZE_MASK, PREVIEW_STAGE, PREVIEW_STAGE_IMAGE,
    PREVIEW_STAGE_MOBILE, PREVIEW_STAGE_PAPER, DIFF, DIFF_HEADER, DIFF_HEADING, DIFF_TITLE,
    DIFF_SUBTITLE, DIFF_PATH, DIFF_STATS, DIFF_STAT_ADD, DIFF_STAT_DEL, DIFF_OPEN_BTN, DIFF_INLINE,
    DIFF_INLINE_BODY, DIFF_LINE, DIFF_LN, DIFF_MARK, DIFF_TEXT, EDITOR, EDITOR_TOOLBAR,
    EDITOR_ACTIONS, EDITOR_DIRTY, EDITOR_SAVE_NOTICE, EDITOR_ERROR, EDITOR_LOADING, EDITOR_SCROLL,
    EDITOR_CODE, EDITOR_GUTTER, EDITOR_LN, EDITOR_SURFACE, EDITOR_FIND_MARKS, EDITOR_HIGHLIGHT,
    EDITOR_TEXTAREA, FIND, FIND_ROW, FIND_TOGGLE, FIND_SPACER, FIND_INPUT_WRAP, FIND_INPUT,
    FIND_OPT, FIND_ACTIONS, FIND_COUNT, FIND_BTN,
};

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

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_confirm_delete_many(paths: &[String]) -> bool {
    match paths.len() {
        0 => false,
        1 => fs_confirm_delete(&paths[0]),
        n => crate::desktop::files::confirm_delete_many(n),
    }
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_confirm_delete_many(_paths: &[String]) -> bool {
    false
}

/// 多选删除时去掉已被父路径覆盖的子项，并按深度从深到浅排序。
fn fs_order_delete_paths(paths: &[String]) -> Vec<String> {
    let mut out: Vec<String> = paths
        .iter()
        .filter(|p| {
            let path = Path::new(p.as_str());
            !paths.iter().any(|other| {
                if other == *p {
                    return false;
                }
                path.starts_with(Path::new(other.as_str()))
            })
        })
        .cloned()
        .collect();
    out.sort_by_key(|p| std::cmp::Reverse(Path::new(p).components().count()));
    out
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_reveal_in_os(path: &str) -> Result<(), String> {
    crate::desktop::files::reveal_in_os(Path::new(path))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_reveal_in_os(_path: &str) -> Result<(), String> {
    Err("Web 端无法在系统中显示。".into())
}

/// 用系统默认应用打开文件或目录。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_open_in_default_app(path: &str) -> Result<(), String> {
    crate::desktop::files::open_in_default_app(Path::new(path))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_open_in_default_app(_path: &str) -> Result<(), String> {
    Err("Web 端无法用系统默认应用打开。".into())
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

/// 文件树「在Pusa中预览」：HTML 文件或含 index/任意 html 的目录。
pub fn fs_resolve_html_preview(path: &str, is_dir: bool) -> Option<String> {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::html_preview::resolve_html_preview(Path::new(path), is_dir)
            .map(|p| p.to_string_lossy().into_owned())
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        let is_html = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("html") || e.eq_ignore_ascii_case("htm"));
        if !is_dir && is_html {
            Some(path.to_string())
        } else {
            None
        }
    }
}

pub fn fs_is_pdf_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
}

pub fn fs_is_image_path(path: &str) -> bool {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::html_preview::is_image_path(Path::new(path))
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| {
                matches!(
                    e.to_ascii_lowercase().as_str(),
                    "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico" | "bmp" | "avif"
                )
            })
    }
}

pub fn fs_is_audio_path(path: &str) -> bool {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::html_preview::is_audio_path(Path::new(path))
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| {
                matches!(
                    e.to_ascii_lowercase().as_str(),
                    "mp3" | "wav" | "ogg" | "oga" | "m4a" | "aac" | "flac" | "opus" | "webm"
                )
            })
    }
}

/// 图片预览 `<img>` 的 `src`（走 `pusapreview` 协议）。
pub fn fs_image_preview_src(path: &str) -> Result<String, String> {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::html_preview::image_preview_src(Path::new(path))
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        let _ = path;
        Err("Web 端暂不支持图片预览。".into())
    }
}

/// 音频预览 `<audio>` 的 `src`（走 `pusapreview` 协议）。
pub fn fs_audio_preview_src(path: &str) -> Result<String, String> {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::html_preview::audio_preview_src(Path::new(path))
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        let _ = path;
        Err("Web 端暂不支持音频预览。".into())
    }
}

/// 本地解析音频时长（秒）；Web 端返回 `None`。
pub fn fs_audio_duration_secs(path: &str) -> Option<f64> {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::html_preview::audio_duration_secs(Path::new(path))
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        let _ = path;
        None
    }
}

/// PDF 预览 iframe 的 `src`（走 `pusapreview` 协议，由 WebView 自带的 PDF 渲染器显示）。
pub fn fs_pdf_preview_src(path: &str) -> Result<String, String> {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::html_preview::pdf_preview_src(Path::new(path))
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        let _ = path;
        Err("Web 端暂不支持 PDF 预览。".into())
    }
}

pub fn fs_is_stl_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("stl"))
}

/// STL 预览 iframe：内置 WebGL 查看器页面（走 `pusapreview` 协议，同源 `fetch` 模型文件）。
/// Windows 与 HTML 预览一样改走 `srcdoc`。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_stl_preview_frame(path: &str) -> Result<HtmlPreviewFrame, String> {
    let p = Path::new(path);
    #[cfg(target_os = "windows")]
    {
        crate::desktop::html_preview::stl_preview_srcdoc(p).map(HtmlPreviewFrame::SrcDoc)
    }
    #[cfg(not(target_os = "windows"))]
    {
        crate::desktop::html_preview::stl_preview_src(p).map(HtmlPreviewFrame::Url)
    }
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_stl_preview_frame(_path: &str) -> Result<HtmlPreviewFrame, String> {
    Err("Web 端暂不支持 STL 预览。".into())
}

pub fn fs_is_docx_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("docx"))
}

pub fn fs_is_xlsx_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("xlsx"))
}

pub fn fs_is_pptx_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("pptx"))
}

pub fn fs_is_doc_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("doc"))
}

pub fn fs_is_xls_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("xls"))
}

pub fn fs_is_ppt_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ppt"))
}

/// 可预览 Office：OOXML + 旧版 .doc / .xls / .ppt。
pub fn fs_is_office_path(path: &str) -> bool {
    fs_is_docx_path(path)
        || fs_is_xlsx_path(path)
        || fs_is_pptx_path(path)
        || fs_is_doc_path(path)
        || fs_is_xls_path(path)
        || fs_is_ppt_path(path)
}

/// 旧版 .doc / .ppt 可能触发 LibreOffice / textutil 转换（阻塞）。
pub fn fs_office_preview_may_convert(path: &str) -> bool {
    fs_is_doc_path(path) || fs_is_ppt_path(path)
}

/// Office 预览 iframe：内置 JS 查看器（走 `pusapreview` 协议，同源 `fetch` 源文件）。
/// Windows 与 HTML / STL 预览一样改走 `srcdoc`。旧版格式可能先做系统转换。
/// `theme`：应用当前 `data-theme`（`light` / `dark`），写入查看器首屏配置。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_office_preview_frame(path: &str, theme: &str) -> Result<HtmlPreviewFrame, String> {
    let p = Path::new(path);
    #[cfg(target_os = "windows")]
    {
        crate::desktop::html_preview::office_preview_srcdoc(p, theme).map(HtmlPreviewFrame::SrcDoc)
    }
    #[cfg(not(target_os = "windows"))]
    {
        crate::desktop::html_preview::office_preview_src(p, theme).map(HtmlPreviewFrame::Url)
    }
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_office_preview_frame(_path: &str, _theme: &str) -> Result<HtmlPreviewFrame, String> {
    Err("Web 端暂不支持 Office 预览。".into())
}

fn office_preview_post_theme_js(theme: &str) -> String {
    let theme = if theme.eq_ignore_ascii_case("dark") {
        "dark"
    } else {
        "light"
    };
    format!(
        r#"(() => {{
  const theme = '{theme}';
  document.querySelectorAll('iframe[data-office-preview-frame]').forEach((frame) => {{
    try {{ frame.contentWindow.postMessage({{ type: 'pusa-theme', theme }}, '*'); }} catch (_) {{}}
  }});
}})();"#
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HtmlPreviewViewport {
    Desktop,
    Mobile,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HtmlPreviewResizeEdge {
    N,
    S,
    E,
    W,
    Ne,
    Nw,
    Se,
    Sw,
}

#[derive(Clone, Copy)]
struct HtmlPreviewResizeDrag {
    edge: HtmlPreviewResizeEdge,
    start_x: f64,
    start_y: f64,
    start_w: i32,
    start_h: i32,
    /// 机身当前的缩放比例；鼠标位移是屏幕像素，要换算回设备像素。
    scale: f64,
}

/// 手机机身外圈的边框宽度（px），与 `style::PREVIEW_DEVICE` 的 `border` 宽度一致。
const HTML_PREVIEW_PHONE_BEZEL: i32 = 1;

/// 让「机身 + 边框」整体落在舞台内容区内的缩放比例，最多 1（不放大）。
/// 舞台尺寸未知（尚未测量）时返回 1。
fn html_preview_fit_scale(stage_w: f64, stage_h: f64, phone_w: i32, phone_h: i32) -> f64 {
    if stage_w <= 0.0 || stage_h <= 0.0 {
        return 1.0;
    }
    let outer_w = (phone_w + 2 * HTML_PREVIEW_PHONE_BEZEL) as f64;
    let outer_h = (phone_h + 2 * HTML_PREVIEW_PHONE_BEZEL) as f64;
    let s = (stage_w / outer_w).min(stage_h / outer_h).min(1.0);
    ((s * 1000.0).floor() / 1000.0).max(0.2)
}

struct HtmlPreviewPhoneModel {
    label: &'static str,
    width: i32,
    height: i32,
}

const HTML_PREVIEW_PHONE_MODELS: &[HtmlPreviewPhoneModel] = &[
    HtmlPreviewPhoneModel {
        label: "苹果 iPhone 16 Pro",
        width: 393,
        height: 852,
    },
    HtmlPreviewPhoneModel {
        label: "苹果 iPhone SE",
        width: 375,
        height: 667,
    },
    HtmlPreviewPhoneModel {
        label: "谷歌 Pixel 8",
        width: 412,
        height: 915,
    },
    HtmlPreviewPhoneModel {
        label: "常见 Android 360",
        width: 360,
        height: 800,
    },
    HtmlPreviewPhoneModel {
        label: "常见 Android 390",
        width: 390,
        height: 844,
    },
];

const HTML_PREVIEW_PHONE_MIN_W: i32 = 240;
const HTML_PREVIEW_PHONE_MIN_H: i32 = 320;
const HTML_PREVIEW_PHONE_MAX_W: i32 = 720;
const HTML_PREVIEW_PHONE_MAX_H: i32 = 1400;

const HTML_PREVIEW_RESIZE_HANDLES: [(&str, HtmlPreviewResizeEdge); 8] = [
    ("n", HtmlPreviewResizeEdge::N),
    ("s", HtmlPreviewResizeEdge::S),
    ("e", HtmlPreviewResizeEdge::E),
    ("w", HtmlPreviewResizeEdge::W),
    ("ne", HtmlPreviewResizeEdge::Ne),
    ("nw", HtmlPreviewResizeEdge::Nw),
    ("se", HtmlPreviewResizeEdge::Se),
    ("sw", HtmlPreviewResizeEdge::Sw),
];

/// 把手位置 → 工具类片段（原先拼 `ac-html-preview-handle-{name}`）。
fn html_preview_handle_class(name: &str) -> &'static str {
    use crate::shell::style;
    match name {
        "n" => style::PREVIEW_HANDLE_N,
        "s" => style::PREVIEW_HANDLE_S,
        "e" => style::PREVIEW_HANDLE_E,
        "w" => style::PREVIEW_HANDLE_W,
        "ne" => style::PREVIEW_HANDLE_NE,
        "nw" => style::PREVIEW_HANDLE_NW,
        "se" => style::PREVIEW_HANDLE_SE,
        "sw" => style::PREVIEW_HANDLE_SW,
        _ => style::PREVIEW_HANDLE_N,
    }
}

fn html_preview_apply_resize(drag: HtmlPreviewResizeDrag, x: f64, y: f64) -> (i32, i32) {
    let scale = if drag.scale > 0.0 { drag.scale } else { 1.0 };
    let dx = ((x - drag.start_x) / scale).round() as i32;
    let dy = ((y - drag.start_y) / scale).round() as i32;
    let mut w = drag.start_w;
    let mut h = drag.start_h;
    match drag.edge {
        HtmlPreviewResizeEdge::E | HtmlPreviewResizeEdge::Ne | HtmlPreviewResizeEdge::Se => {
            w = drag.start_w + dx;
        }
        HtmlPreviewResizeEdge::W | HtmlPreviewResizeEdge::Nw | HtmlPreviewResizeEdge::Sw => {
            w = drag.start_w - dx;
        }
        _ => {}
    }
    match drag.edge {
        HtmlPreviewResizeEdge::S | HtmlPreviewResizeEdge::Se | HtmlPreviewResizeEdge::Sw => {
            h = drag.start_h + dy;
        }
        HtmlPreviewResizeEdge::N | HtmlPreviewResizeEdge::Ne | HtmlPreviewResizeEdge::Nw => {
            h = drag.start_h - dy;
        }
        _ => {}
    }
    (
        w.clamp(HTML_PREVIEW_PHONE_MIN_W, HTML_PREVIEW_PHONE_MAX_W),
        h.clamp(HTML_PREVIEW_PHONE_MIN_H, HTML_PREVIEW_PHONE_MAX_H),
    )
}

#[derive(Clone, PartialEq)]
pub enum HtmlPreviewFrame {
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    #[cfg_attr(not(all(feature = "native", not(target_arch = "wasm32"))), allow(dead_code))]
    Url(String),
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    SrcDoc(String),
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_html_preview_frame(path: &str) -> Result<HtmlPreviewFrame, String> {
    let p = Path::new(path);
    #[cfg(target_os = "windows")]
    {
        crate::desktop::html_preview::preview_srcdoc(p).map(HtmlPreviewFrame::SrcDoc)
    }
    #[cfg(not(target_os = "windows"))]
    {
        crate::desktop::html_preview::preview_iframe_src(p).map(HtmlPreviewFrame::Url)
    }
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_html_preview_frame(_path: &str) -> Result<HtmlPreviewFrame, String> {
    Err("HTML 内置预览仅桌面端可用。".into())
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
    if s.is_empty() {
        ".".into()
    } else {
        s
    }
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

/// 从最近打开列表移除目录。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_forget_recent_project_dir(path: &str) {
    crate::desktop::files::forget_recent_project_dir(path);
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_forget_recent_project_dir(_path: &str) {}

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

/// 工作区本地 git 分支列表。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_git_branches(root: &str) -> Vec<String> {
    crate::desktop::files::git_branches(Path::new(root))
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_git_branches(_root: &str) -> Vec<String> {
    Vec::new()
}

/// 切换工作区到已存在的本地分支。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn fs_git_checkout(root: &str, branch: &str) -> Result<(), String> {
    crate::desktop::files::git_checkout(Path::new(root), branch)
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub fn fs_git_checkout(_root: &str, _branch: &str) -> Result<(), String> {
    Err("Web 端无法切换本机分支。".into())
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
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
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
    /// 最近写入成功的路径，短暂高亮。
    highlighted_paths: Signal<HashSet<String>>,
    on_open_file: EventHandler<String>,
    /// HTML 文件/站点在中间栏内置浏览器打开。
    on_open_html_preview: EventHandler<String>,
    on_open_pdf_preview: EventHandler<String>,
    /// 图片在中间栏内置查看器打开。
    on_open_image_preview: EventHandler<String>,
    /// 音频在中间栏内置播放器打开（播放按钮 + 声纹）。
    on_open_audio_preview: EventHandler<String>,
    /// STL 模型在中间栏内置 3D 查看器打开。
    on_open_stl_preview: EventHandler<String>,
    /// Office（含旧版 doc/xls/ppt）在中间栏内置 JS 查看器打开。
    on_open_office_preview: EventHandler<String>,
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

    let toast = super::toast::use_toast();

    let mut ctx_menu = use_signal(|| None::<FsCtxMenu>);
    let mut file_clipboard = use_signal(|| None::<FsFileClipboard>);
    let mut renaming_path = use_signal(|| None::<String>);
    let mut rename_draft = use_signal(String::new);
    // 右键「重命名」后强制聚焦输入框（桌面 WebView 上 autofocus 不可靠）。
    let rename_focus_tick = use_signal(|| 0u64);
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

    use_effect(move || {
        let _ = rename_focus_tick();
        if renaming_path().is_some() {
            focus_sidebar_rename_input();
        }
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
            div { class: crate::shell::style::FILES_PANEL,
                div { class: crate::shell::style::FILES_UNAVAILABLE,
                    p { "Web 端无法浏览本机文件。" }
                    p { class: crate::shell::style::FILES_UNAVAILABLE_HINT,
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
            class: "{FILES_PANEL}",
            div { class: crate::shell::style::FILES_HEADER,
                button {
                    r#type: "button",
                    class: crate::shell::style::FILES_TITLE_BTN,
                    title: "展开 / 收起文件树",
                    aria_expanded: section_open(),
                    onclick: move |_| {
                        section_open.with_mut(|v| *v = !*v);
                    },
                    span { class: crate::shell::style::FILES_TITLE_CHEVRON,
                        if section_open() {
                            Icon { icon: LdChevronDown, width: 14, height: 14, fill: "currentColor" }
                        } else {
                            Icon { icon: LdChevronRight, width: 14, height: 14, fill: "currentColor" }
                        }
                    }
                    span { class: crate::shell::style::FILES_ROOT_NAME, "{root_name}" }
                }
                div { class: crate::shell::style::FILES_ACTIONS,
                    button {
                        r#type: "button",
                        class: FILES_ACTION,
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

                            section_open.set(true);
                        },
                        Icon { icon: LdFolderPlus, width: 14, height: 14, fill: "currentColor" }
                    }
                    button {
                        r#type: "button",
                        class: FILES_ACTION,
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

                            section_open.set(true);
                        },
                        Icon { icon: LdFilePlus, width: 14, height: 14, fill: "currentColor" }
                    }
                    button {
                        r#type: "button",
                        class: FILES_ACTION,
                        title: "刷新",
                        aria_label: "刷新",
                        onclick: move |_| {
                            let root = root_refresh.clone();
                            let open = expanded();
                            children_cache.with_mut(|cache| {
                                cache.clear();
                                let _ = fs_cache_reload_open(cache, &root, &open);
                            });

                        },
                        Icon { icon: LdRefreshCw, width: 14, height: 14, fill: "currentColor" }
                    }
                    button {
                        r#type: "button",
                        class: FILES_ACTION,
                        title: "全部折叠",
                        aria_label: "全部折叠",
                        onclick: move |_| {
                            expanded.set(HashSet::new());
                        },
                        Icon { icon: AiOutlineVerticalAlignBottom, width: 14, height: 14, fill: "currentColor" }
                    }
                }
            }

            if section_open() {
                if let Some(kind) = creating {
                    div { class: crate::shell::style::FILES_CREATE,
                        span { class: crate::shell::style::FILES_CREATE_LABEL,
                            match kind {
                                FsCreateKind::File => "新建文件",
                                FsCreateKind::Folder => "新建文件夹",
                            }
                        }
                        if let Some(parent) = create_parent() {
                            span { class: crate::shell::style::FILES_CREATE_PARENT, title: "{parent}",
                                "{fs_file_title(&parent)}"
                            }
                        }
                        input {
                            r#type: "text",
                            class: crate::shell::style::FILES_CREATE_INPUT,
                            placeholder: if kind == FsCreateKind::File { "文件名…" } else { "文件夹名…" },
                            value: "{create_name()}",
                            autofocus: true,
                            oninput: move |e| create_name.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Escape {
                                    e.prevent_default();
                                    e.stop_propagation();
                                    create_mode.set(None);
                                    create_name.set(String::new());
                                    return;
                                }
                                if e.key() != Key::Enter {
                                    return;
                                }
                                e.prevent_default();
                                e.stop_propagation();
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

                                        if kind == FsCreateKind::File {
                                            on_open_file.call(created);
                                        }
                                    }
                                    Err(err) => toast.error(err),
                                }
                            },
                        }
                        button {
                            r#type: "button",
                            class: crate::shell::style::FILES_CREATE_CANCEL,
                            onclick: move |_| {
                                create_mode.set(None);
                                create_name.set(String::new());
                            },
                            "取消"
                        }
                    }
                }

                div {
                    class: crate::shell::style::FILES_TREE,
                    role: "tree",
                    tabindex: "0",
                    // 焦点在树上时：回车重命名当前主选中项。
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() != Key::Enter {
                            return;
                        }
                        if renaming_path().is_some() || create_mode().is_some() {
                            return;
                        }
                        let Some(path) = selected_path() else {
                            return;
                        };
                        e.prevent_default();
                        begin_sidebar_rename(
                            &path,
                            renaming_path,
                            rename_draft,
                            rename_focus_tick,
                        );
                    },
                    {
                        match children_cache().get(&root_tree) {
                            Some(Err(err)) => rsx! {
                                div { class: FILES_EMPTY_ERROR, "{err}" }
                            },
                            Some(Ok(entries)) if entries.is_empty() => rsx! {
                                div { class: FILES_EMPTY, "此目录为空" }
                            },
                            _ => rsx! {
                                for (entry, depth) in rows {
                                    {
                                        let path = entry.path.clone();
                                        let path_sel = entry.path.clone();
                                        let path_toggle = entry.path.clone();
                                        let path_ctx = entry.path.clone();
                                        let path_rename = entry.path.clone();
                                        let path_enter_rename = entry.path.clone();
                                        let root_fallback = root_for_menu.clone();
                                        let root_shift = root_for_menu.clone();
                                        let is_dir = entry.is_dir;
                                        let is_open = expanded().contains(&entry.path);
                                        let is_selected = selected_paths_now.contains(&entry.path)
                                            || selected_path().as_deref() == Some(entry.path.as_str());
                                        let is_muted = fs_name_is_muted(&entry.name);
                                        let is_written = highlighted_paths().contains(&entry.path);
                                        let is_renaming = renaming_path().as_deref() == Some(entry.path.as_str());
                                        // depth 0 为工作区根的直接子项；相对根标题「PUSA」明显再缩进一级。
                                        let pad = 1.15 + (depth as f64) * 0.8;
                                        let pad_style = format!("padding-left: {pad}rem");
                                        let name = entry.name.clone();
                                        rsx! {
                                            if is_renaming {
                                                div {
                                                    key: "rename-{path}",
                                                    class: crate::shell::style::FILES_ROW_RENAMING,
                                                    style: "{pad_style}",
                                                    span { class: FILES_ROW_CHEVRON,
                                                        if is_dir {
                                                            Icon { icon: LdChevronRight, width: 12, height: 12, fill: "currentColor" }
                                                        }
                                                    }
                                                    span { class: FILES_ROW_ICON,
                                                        if is_dir {
                                                            Icon { icon: LdFolder, width: 13, height: 13, fill: "currentColor" }
                                                        } else {
                                                            Icon { icon: LdFile, width: 13, height: 13, fill: "currentColor" }
                                                        }
                                                    }
                                                    input {
                                                        r#type: "text",
                                                        class: crate::shell::style::FILES_RENAME_INPUT,
                                                        "data-files-rename-input": true,
                                                        value: "{rename_draft()}",
                                                        autofocus: true,
                                                        onmounted: move |_| {
                                                            focus_sidebar_rename_input();
                                                        },
                                                        oninput: move |e| rename_draft.set(e.value()),
                                                        onkeydown: move |e: KeyboardEvent| {
                                                            if e.key() == Key::Escape {
                                                                e.prevent_default();
                                                                e.stop_propagation();
                                                                renaming_path.set(None);
                                                                rename_draft.set(String::new());
                                                                return;
                                                            }
                                                            if e.key() != Key::Enter {
                                                                return;
                                                            }
                                                            // 必须 stop：否则冒泡到树的 Enter 会在 renaming_path
                                                            // 已清空后再次 begin_sidebar_rename，输入框还会出现。
                                                            e.prevent_default();
                                                            e.stop_propagation();
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

                                                                }
                                                                Err(err) => toast.error(err),
                                                            }
                                                        },
                                                        onblur: move |_| {
                                                            // 菜单关闭 / 强制 focus 可能抢焦点；等两帧后再判断是否真的离开。
                                                            spawn(async move {
                                                                let _ = document::eval(
                                                                    r#"return await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(() => r(true))));"#,
                                                                )
                                                                .join::<bool>()
                                                                .await;
                                                                if renaming_path().is_none() {
                                                                    return;
                                                                }
                                                                let keep = document::eval(
                                                                    r#"return !!(document.activeElement && document.activeElement.hasAttribute && document.activeElement.hasAttribute('data-files-rename-input'));"#,
                                                                )
                                                                .join::<bool>()
                                                                .await
                                                                .unwrap_or(false);
                                                                if keep {
                                                                    return;
                                                                }
                                                                renaming_path.set(None);
                                                                rename_draft.set(String::new());
                                                            });
                                                        },
                                                    }
                                                }
                                            } else {
                                                button {
                                                    key: "{path}",
                                                    r#type: "button",
                                                    class: crate::shell::style::FILES_ROW,
                                                    "data-selected": is_selected,
                                                    "data-muted": is_muted,
                                                    "data-written": is_written,
                                                    style: "{pad_style}",
                                                    role: "treeitem",
                                                    title: "{path}",
                                                    onkeydown: move |e: KeyboardEvent| {
                                                        if e.key() != Key::Enter {
                                                            return;
                                                        }
                                                        // 选中行回车：进入重命名（勿触发按钮默认「点击打开」）。
                                                        if renaming_path().is_some()
                                                            || create_mode().is_some()
                                                        {
                                                            return;
                                                        }
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                        begin_sidebar_rename(
                                                            &path_enter_rename,
                                                            renaming_path,
                                                            rename_draft,
                                                            rename_focus_tick,
                                                        );
                                                    },
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
                                                    span { class: FILES_ROW_CHEVRON,
                                                        if is_dir {
                                                            if is_open {
                                                                Icon { icon: LdChevronDown, width: 12, height: 12, fill: "currentColor" }
                                                            } else {
                                                                Icon { icon: LdChevronRight, width: 12, height: 12, fill: "currentColor" }
                                                            }
                                                        }
                                                    }
                                                    span { class: FILES_ROW_ICON,
                                                        if is_dir {
                                                            Icon { icon: LdFolder, width: 13, height: 13, fill: "currentColor" }
                                                        } else {
                                                            Icon { icon: LdFile, width: 13, height: 13, fill: "currentColor" }
                                                        }
                                                    }
                                                    span { class: FILES_ROW_NAME, "{name}" }
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
                        format!("将 {} 个文件加入对话", add_targets.len())
                    } else {
                        "加入对话".into()
                    };
                    let add_new_label = if add_targets.len() > 1 {
                        format!("将 {} 个文件加入新的对话", add_targets.len())
                    } else {
                        "加入新的对话".into()
                    };
                    let delete_targets = add_targets.clone();
                    let delete_label = if delete_targets.len() > 1 {
                        format!("删除 {} 项", delete_targets.len())
                    } else {
                        "删除".into()
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
                    let anchor_x = ctx.x;
                    let anchor_y = ctx.y;
                    rsx! {
                        div {
                            class: "{CTX_BACKDROP} z-[1300]!",
                            onclick: move |_| ctx_menu.set(None),
                            oncontextmenu: move |evt| {
                                evt.prevent_default();
                                ctx_menu.set(None);
                            },
                        }
                        div {
                            id: "ac-fs-ctx-menu",
                            class: "{CTX_MENU} {FS_CTX_MENU}",
                            role: "menu",
                            style: "{menu_style}",
                            onclick: move |evt| evt.stop_propagation(),
                            onmounted: move |_| {
                                // 超出视口底部时底边对齐窗口底部；超出右侧时左移，避免菜单被裁切。
                                let _ = document::eval(&format!(
                                    r#"(() => {{
  const place = () => {{
    const el = document.getElementById('ac-fs-ctx-menu');
    if (!el) return false;
    const margin = 8;
    const vh = window.innerHeight || document.documentElement.clientHeight || 800;
    const vw = window.innerWidth || document.documentElement.clientWidth || 1200;
    const rect = el.getBoundingClientRect();
    let top = {anchor_y};
    let left = {anchor_x};
    const h = Math.max(rect.height, 1);
    const w = Math.max(rect.width, 1);
    // 底部放不下：底边贴住窗口底部（不再翻到选项上方）。
    if (top + h > vh - margin) {{
      top = Math.max(margin, vh - h - margin);
    }}
    if (left + w > vw - margin) {{
      left = Math.max(margin, vw - w - margin);
    }}
    if (left < margin) left = margin;
    if (top < margin) top = margin;
    el.style.top = top + 'px';
    el.style.left = left + 'px';
    el.style.right = 'auto';
    el.style.maxHeight = Math.max(120, vh - top - margin) + 'px';
    return true;
  }};
  if (!place()) {{
    requestAnimationFrame(() => {{ if (!place()) requestAnimationFrame(place); }});
  }}
}})()"#
                                ));
                            },

                            {
                                let html_preview = fs_resolve_html_preview(&path, is_dir);
                                let is_pdf = !is_dir && fs_is_pdf_path(&path);
                                let is_image = !is_dir && fs_is_image_path(&path);
                                let is_audio = !is_dir && fs_is_audio_path(&path);
                                let is_stl = !is_dir && fs_is_stl_path(&path);
                                let is_office = !is_dir && fs_is_office_path(&path);
                                rsx! {
                                    if let Some(html) = html_preview.clone() {
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: move |_| {
                                                ctx_menu.set(None);
                                                on_open_html_preview.call(html.clone());
                                            },
                                            "在Pusa中预览"
                                        }
                                    } else if is_image {
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    on_open_image_preview.call(path.clone());
                                                }
                                            },
                                            "在Pusa中预览"
                                        }
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    match fs_open_in_browser(&path) {
                                                        Ok(()) => {},
                                                        Err(e) => toast.error(e),
                                                    }
                                                }
                                            },
                                            "在浏览器中打开"
                                        }
                                    } else if is_audio {
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    on_open_audio_preview.call(path.clone());
                                                }
                                            },
                                            "在Pusa中预览"
                                        }
                                    } else if is_pdf {
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    on_open_pdf_preview.call(path.clone());
                                                }
                                            },
                                            "在Pusa中预览"
                                        }
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    match fs_open_in_browser(&path) {
                                                        Ok(()) => {},
                                                        Err(e) => toast.error(e),
                                                    }
                                                }
                                            },
                                            "在浏览器中打开"
                                        }
                                    } else if is_stl {
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    on_open_stl_preview.call(path.clone());
                                                }
                                            },
                                            "在Pusa中预览"
                                        }
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    match fs_open_in_browser(&path) {
                                                        Ok(()) => {},
                                                        Err(e) => toast.error(e),
                                                    }
                                                }
                                            },
                                            "在浏览器中打开"
                                        }
                                    } else if is_office {
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    on_open_office_preview.call(path.clone());
                                                }
                                            },
                                            "在Pusa中预览"
                                        }
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    match fs_open_in_browser(&path) {
                                                        Ok(()) => {},
                                                        Err(e) => toast.error(e),
                                                    }
                                                }
                                            },
                                            "在浏览器中打开"
                                        }
                                    } else if !is_dir {
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    on_open_file.call(path.clone());
                                                }
                                            },
                                            "在Pusa中预览"
                                        }
                                        button {
                                            r#type: "button",
                                            class: CTX_ITEM,
                                            role: "menuitem",
                                            onclick: {
                                                let path = path.clone();
                                                move |_| {
                                                    ctx_menu.set(None);
                                                    match fs_open_in_browser(&path) {
                                                        Ok(()) => {},
                                                        Err(e) => toast.error(e),
                                                    }
                                                }
                                            },
                                            "在浏览器中打开"
                                        }
                                    }
                                }
                            }
                            button {
                                r#type: "button",
                                class: CTX_ITEM,
                                role: "menuitem",
                                onclick: {
                                    let path = path.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        match fs_reveal_in_os(&path) {
                                            Ok(()) => {},
                                            Err(e) => toast.error(e),
                                        }
                                    }
                                },
                                "{reveal_label}"
                            }
                            button {
                                r#type: "button",
                                class: CTX_ITEM,
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

                            div { class: FS_CTX_SEP, role: "separator" }

                            button {
                                r#type: "button",
                                class: CTX_ITEM,
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
                                class: CTX_ITEM,
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

                            div { class: FS_CTX_SEP, role: "separator" }

                            button {
                                r#type: "button",
                                class: CTX_ITEM,
                                role: "menuitem",
                                onclick: {
                                    let paths = clip_targets.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        file_clipboard.set(Some(FsFileClipboard {
                                            paths: paths.clone(),
                                            cut: true,
                                        }));
                                        toast.success(if paths.len() > 1 {
                                            format!("已剪切 {} 项。", paths.len())
                                        } else {
                                            "已剪切。".into()
                                        });
                                    }
                                },
                                "剪切"
                            }
                            button {
                                r#type: "button",
                                class: CTX_ITEM,
                                role: "menuitem",
                                onclick: {
                                    let paths = clip_targets.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        file_clipboard.set(Some(FsFileClipboard {
                                            paths: paths.clone(),
                                            cut: false,
                                        }));
                                        toast.success(if paths.len() > 1 {
                                            format!("已复制 {} 项。", paths.len())
                                        } else {
                                            "已复制。".into()
                                        });
                                    }
                                },
                                "复制"
                            }
                            if has_clipboard {
                                button {
                                    r#type: "button",
                                    class: CTX_ITEM,
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
                                            if let Some(e) = err {
                                                toast.error(e);
                                            } else {
                                                toast.success("已粘贴。");
                                            }
                                        }
                                    },
                                    "粘贴"
                                }
                            }

                            div { class: FS_CTX_SEP, role: "separator" }

                            button {
                                r#type: "button",
                                class: CTX_ITEM,
                                role: "menuitem",
                                onclick: {
                                    let path = path.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        match fs_clipboard_set_text(&path) {
                                            Ok(()) => toast.success("已复制路径。"),
                                            Err(e) => toast.error(e),
                                        }
                                    }
                                },
                                "复制路径"
                            }
                            button {
                                r#type: "button",
                                class: CTX_ITEM,
                                role: "menuitem",
                                onclick: {
                                    let path = path.clone();
                                    let root = root.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        let rel = fs_relative_to_root(&root, &path);
                                        match fs_clipboard_set_text(&rel) {
                                            Ok(()) => toast.success("已复制相对路径。"),
                                            Err(e) => toast.error(e),
                                        }
                                    }
                                },
                                "复制相对路径"
                            }

                            div { class: FS_CTX_SEP, role: "separator" }

                            button {
                                r#type: "button",
                                class: CTX_ITEM,
                                role: "menuitem",
                                onclick: {
                                    let path = path.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        begin_sidebar_rename(
                                            &path,
                                            renaming_path,
                                            rename_draft,
                                            rename_focus_tick,
                                        );
                                    }
                                },
                                "重命名…"
                            }
                            button {
                                r#type: "button",
                                class: CTX_ITEM_DANGER,
                                role: "menuitem",
                                onclick: {
                                    let targets = delete_targets.clone();
                                    let root = root.clone();
                                    move |_| {
                                        ctx_menu.set(None);
                                        if targets.is_empty() {
                                            return;
                                        }
                                        if !fs_confirm_delete_many(&targets) {
                                            return;
                                        }
                                        let ordered = fs_order_delete_paths(&targets);
                                        let mut parents = HashSet::<String>::new();
                                        let mut err: Option<String> = None;
                                        for path in &ordered {
                                            match fs_delete(path) {
                                                Ok(()) => {
                                                    if let Some(parent) = Path::new(path).parent() {
                                                        parents.insert(
                                                            parent.to_string_lossy().into_owned(),
                                                        );
                                                    } else {
                                                        parents.insert(root.clone());
                                                    }
                                                    expanded.with_mut(|set| {
                                                        set.remove(path);
                                                    });
                                                    selected_paths.with_mut(|set| {
                                                        set.remove(path);
                                                    });
                                                    if selected_path().as_deref()
                                                        == Some(path.as_str())
                                                    {
                                                        selected_path.set(None);
                                                    }
                                                }
                                                Err(e) => {
                                                    err = Some(e);
                                                    break;
                                                }
                                            }
                                        }
                                        children_cache.with_mut(|cache| {
                                            for parent in &parents {
                                                fs_cache_reload(cache, parent);
                                            }
                                            if parents.is_empty() {
                                                fs_cache_reload(cache, &root);
                                            }
                                        });
                                        if selected_paths().is_empty() {
                                            selected_path.set(None);
                                        } else if selected_path()
                                            .as_ref()
                                            .is_none_or(|p| !selected_paths().contains(p))
                                        {
                                            selected_path.set(
                                                selected_paths().iter().next().cloned(),
                                            );
                                        }
                                        if let Some(e) = err {
                                            toast.error(e);
                                        }
                                    }
                                },
                                "{delete_label}"
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

fn begin_sidebar_rename(
    path: &str,
    mut renaming_path: Signal<Option<String>>,
    mut rename_draft: Signal<String>,
    mut rename_focus_tick: Signal<u64>,
) {
    rename_draft.set(fs_file_title(path));
    renaming_path.set(Some(path.to_string()));
    rename_focus_tick.with_mut(|n| *n = n.wrapping_add(1));
}

/// 聚焦侧栏重命名输入框，并选中文件名主干（不含扩展名）。
fn focus_sidebar_rename_input() {
    let _ = document::eval(
        r#"(() => {
  const run = () => {
    const el = document.querySelector('input[data-files-rename-input]');
    if (!el) return false;
    el.focus();
    const v = el.value || '';
    const base = v.includes('/') ? v.slice(v.lastIndexOf('/') + 1) : v;
    const dot = base.lastIndexOf('.');
    if (dot > 0) {
      const start = v.length - base.length;
      el.setSelectionRange(start, start + dot);
    } else {
      el.select();
    }
    return true;
  };
  if (!run()) {
    requestAnimationFrame(() => {
      if (!run()) requestAnimationFrame(run);
    });
  }
})()"#,
    );
}

fn is_primary_mod(mods: Modifiers) -> bool {
    mods.contains(Modifiers::SUPER)
        || mods.contains(Modifiers::CONTROL)
        || mods.contains(Modifiers::META)
}

fn is_save_shortcut(e: &KeyboardEvent) -> bool {
    let Key::Character(c) = e.key() else {
        return false;
    };
    c.eq_ignore_ascii_case("s") && is_primary_mod(e.modifiers())
}

fn is_find_shortcut(e: &KeyboardEvent) -> bool {
    let Key::Character(c) = e.key() else {
        return false;
    };
    c.eq_ignore_ascii_case("f") && is_primary_mod(e.modifiers())
}

fn is_replace_shortcut(e: &KeyboardEvent) -> bool {
    let Key::Character(c) = e.key() else {
        return false;
    };
    c.eq_ignore_ascii_case("h") && is_primary_mod(e.modifiers())
}

fn is_chat_shortcut(e: &KeyboardEvent) -> bool {
    let Key::Character(c) = e.key() else {
        return false;
    };
    c.eq_ignore_ascii_case("l") && is_primary_mod(e.modifiers())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FindMatch {
    /// UTF-8 byte offset
    start: usize,
    end: usize,
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_whole_word_at(text: &str, start: usize, end: usize) -> bool {
    let before_ok = text
        .get(..start)
        .and_then(|s| s.chars().next_back())
        .is_none_or(|c| !is_word_char(c));
    let after_ok = text
        .get(end..)
        .and_then(|s| s.chars().next())
        .is_none_or(|c| !is_word_char(c));
    before_ok && after_ok
}

fn collect_find_matches(
    text: &str,
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
) -> Result<Vec<FindMatch>, String> {
    if query.is_empty() {
        return Ok(Vec::new());
    }
    const MAX: usize = 10_000;
    let mut out = Vec::new();
    if use_regex {
        let re = regex::RegexBuilder::new(query)
            .case_insensitive(!case_sensitive)
            .multi_line(true)
            .build()
            .map_err(|e| format!("无效正则：{e}"))?;
        for m in re.find_iter(text) {
            if whole_word && !is_whole_word_at(text, m.start(), m.end()) {
                continue;
            }
            out.push(FindMatch {
                start: m.start(),
                end: m.end(),
            });
            if out.len() >= MAX {
                break;
            }
        }
        return Ok(out);
    }

    if case_sensitive {
        let mut from = 0;
        while let Some(rel) = text[from..].find(query) {
            let start = from + rel;
            let end = start + query.len();
            if !whole_word || is_whole_word_at(text, start, end) {
                out.push(FindMatch { start, end });
                if out.len() >= MAX {
                    break;
                }
            }
            from = start + query.len().max(1);
            if from > text.len() {
                break;
            }
        }
    } else {
        let lower_q = query.to_lowercase();
        let lower_t = text.to_lowercase();
        // 大小写折叠后字节偏移与原文不完全一致（非 ASCII）；对常见代码 ASCII 足够。
        // 非 ASCII 内容回退按字符扫描。
        if text.is_ascii() && query.is_ascii() {
            let mut from = 0;
            while let Some(rel) = lower_t[from..].find(&lower_q) {
                let start = from + rel;
                let end = start + query.len();
                if !whole_word || is_whole_word_at(text, start, end) {
                    out.push(FindMatch { start, end });
                    if out.len() >= MAX {
                        break;
                    }
                }
                from = start + query.len().max(1);
                if from > text.len() {
                    break;
                }
            }
        } else {
            let chars: Vec<(usize, char)> = text.char_indices().collect();
            let q_chars: Vec<char> = query.chars().collect();
            let q_lower: Vec<char> = q_chars
                .iter()
                .map(|c| c.to_lowercase().next().unwrap_or(*c))
                .collect();
            let n = q_lower.len();
            if n == 0 {
                return Ok(out);
            }
            let mut i = 0;
            while i + n <= chars.len() {
                let matched = (0..n).all(|k| {
                    chars[i + k]
                        .1
                        .to_lowercase()
                        .next()
                        .unwrap_or(chars[i + k].1)
                        == q_lower[k]
                });
                if matched {
                    let start = chars[i].0;
                    let end = if i + n < chars.len() {
                        chars[i + n].0
                    } else {
                        text.len()
                    };
                    if !whole_word || is_whole_word_at(text, start, end) {
                        out.push(FindMatch { start, end });
                        if out.len() >= MAX {
                            break;
                        }
                    }
                    i += n;
                } else {
                    i += 1;
                }
            }
        }
    }
    Ok(out)
}

fn utf16_offset(s: &str, byte_idx: usize) -> usize {
    let idx = byte_idx.min(s.len());
    s.get(..idx)
        .map(|part| part.encode_utf16().count())
        .unwrap_or(0)
}

fn escape_find_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// 在透明文本层上画出全部匹配高亮；当前项加 `is-current`。
fn build_find_marks_html(source: &str, matches: &[FindMatch], current: usize) -> String {
    if matches.is_empty() || source.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(source.len().saturating_mul(2));
    let mut last = 0usize;
    for (i, m) in matches.iter().enumerate() {
        if m.start < last || m.end > source.len() || m.start > m.end {
            continue;
        }
        out.push_str(&escape_find_html(&source[last..m.start]));
        let cls = if i == current {
            crate::shell::style::FIND_HIT_CURRENT
        } else {
            crate::shell::style::FIND_HIT
        };
        out.push_str(if i == current {
            "<mark data-find-hit data-current class=\""
        } else {
            "<mark data-find-hit class=\""
        });
        out.push_str(cls);
        out.push_str("\">");
        out.push_str(&escape_find_html(&source[m.start..m.end]));
        out.push_str("</mark>");
        last = m.end;
    }
    out.push_str(&escape_find_html(&source[last..]));
    if !source.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// 选中当前匹配，并滚动到 `mark[data-find-hit][data-current]`（高亮层渲染后再定位）。
fn select_find_match_in_editor(text: &str, m: FindMatch, focus_editor: bool) {
    let start = utf16_offset(text, m.start);
    let end = utf16_offset(text, m.end);
    let focus_js = if focus_editor { "ta.focus();" } else { "" };
    let _ = document::eval(&format!(
        r#"(() => {{
  const scrollToHit = () => {{
    const scroll = document.querySelector('[data-editor-scroll]');
    const hit = document.querySelector('mark[data-find-hit][data-current]');
    if (scroll && hit) {{
      const sRect = scroll.getBoundingClientRect();
      const hRect = hit.getBoundingClientRect();
      const top = hRect.top - sRect.top + scroll.scrollTop;
      scroll.scrollTop = Math.max(0, top - Math.max(48, scroll.clientHeight / 3));
      if (hRect.left < sRect.left + 8 || hRect.right > sRect.right - 8) {{
        const left = hRect.left - sRect.left + scroll.scrollLeft;
        scroll.scrollLeft = Math.max(0, left - 48);
      }}
      return true;
    }}
    // 高亮层尚未挂上时，按行号估算滚动。
    const ta = document.querySelector('textarea[data-editor-textarea]');
    if (!ta || !scroll) return false;
    const before = ta.value.slice(0, {start});
    const lines = before.split('\\n').length;
    let lineHeight = parseFloat(getComputedStyle(ta).lineHeight);
    if (!Number.isFinite(lineHeight) || lineHeight <= 0) {{
      const fs = parseFloat(getComputedStyle(ta).fontSize) || 13;
      lineHeight = fs * 1.45;
    }}
    const pad = parseFloat(getComputedStyle(ta).paddingTop) || 0;
    scroll.scrollTop = Math.max(0, pad + (lines - 1) * lineHeight - scroll.clientHeight / 3);
    return true;
  }};
  const ta = document.querySelector('textarea[data-editor-textarea]');
  if (ta) {{
    {focus_js}
    try {{ ta.setSelectionRange({start}, {end}); }} catch (_) {{}}
  }}
  if (!scrollToHit()) {{
    requestAnimationFrame(() => {{
      if (!scrollToHit()) requestAnimationFrame(scrollToHit);
    }});
  }}
}})()"#
    ));
}

fn focus_file_find_input() {
    let _ = document::eval(
        r#"(() => {
  const el = document.querySelector('input[data-find-query]');
  if (!el) return;
  el.focus();
  el.select();
})()"#,
    );
}

fn apply_preserve_case(matched: &str, replacement: &str) -> String {
    if replacement.is_empty() || matched.is_empty() {
        return replacement.to_string();
    }
    let matched_alpha: String = matched.chars().filter(|c| c.is_alphabetic()).collect();
    if matched_alpha.is_empty() {
        return replacement.to_string();
    }
    if matched_alpha.chars().all(|c| c.is_uppercase()) {
        return replacement.to_uppercase();
    }
    if matched_alpha.chars().all(|c| c.is_lowercase()) {
        return replacement.to_lowercase();
    }
    let mut chars = replacement.chars();
    let Some(first) = chars.next() else {
        return replacement.to_string();
    };
    if matched
        .chars()
        .find(|c| c.is_alphabetic())
        .is_some_and(|c| c.is_uppercase())
    {
        let mut out = first.to_uppercase().to_string();
        out.extend(chars);
        out
    } else {
        replacement.to_string()
    }
}

fn set_editor_draft(
    path: &str,
    next: String,
    mut drafts: Signal<HashMap<String, String>>,
    baselines: Signal<HashMap<String, String>>,
    mut dirty: Signal<HashSet<String>>,
    mut save_notice: Signal<Option<String>>,
) {
    let matches_baseline = baselines().get(path).is_some_and(|b| b == &next);
    drafts.with_mut(|m| {
        m.insert(path.to_string(), next);
    });
    dirty.with_mut(|s| {
        if matches_baseline {
            s.remove(path);
        } else {
            s.insert(path.to_string());
        }
    });
    save_notice.set(None);
}

fn editor_goto_find_match(delta: isize, matches: &[FindMatch], mut match_index: Signal<usize>) {
    if matches.is_empty() {
        return;
    }
    let len = matches.len() as isize;
    let next = ((match_index() as isize + delta).rem_euclid(len)) as usize;
    match_index.set(next);
    // 选区 / 滚动在高亮层把 is-current 更新后由 use_effect 执行。
}

fn editor_replace_one(
    path: &str,
    drafts: Signal<HashMap<String, String>>,
    baselines: Signal<HashMap<String, String>>,
    dirty: Signal<HashSet<String>>,
    save_notice: Signal<Option<String>>,
    matches: &[FindMatch],
    mut match_index: Signal<usize>,
    query: &str,
    replacement: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
    preserve_case: bool,
) {
    if matches.is_empty() {
        return;
    }
    let idx = match_index().min(matches.len() - 1);
    let m = matches[idx];
    let Some(body) = drafts().get(path).cloned() else {
        return;
    };
    if m.end > body.len() || m.start > m.end {
        return;
    }
    let matched = &body[m.start..m.end];
    let repl = if preserve_case {
        apply_preserve_case(matched, replacement)
    } else {
        replacement.to_string()
    };
    let mut next = String::with_capacity(body.len() - (m.end - m.start) + repl.len());
    next.push_str(&body[..m.start]);
    next.push_str(&repl);
    next.push_str(&body[m.end..]);
    set_editor_draft(path, next.clone(), drafts, baselines, dirty, save_notice);
    let new_matches = collect_find_matches(&next, query, case_sensitive, whole_word, use_regex)
        .unwrap_or_default();
    if new_matches.is_empty() {
        match_index.set(0);
    } else {
        match_index.set(idx.min(new_matches.len() - 1));
    }
}

fn editor_replace_all(
    path: &str,
    drafts: Signal<HashMap<String, String>>,
    baselines: Signal<HashMap<String, String>>,
    dirty: Signal<HashSet<String>>,
    save_notice: Signal<Option<String>>,
    mut match_index: Signal<usize>,
    query: &str,
    replacement: &str,
    case_sensitive: bool,
    whole_word: bool,
    use_regex: bool,
    preserve_case: bool,
) {
    let Some(body) = drafts().get(path).cloned() else {
        return;
    };
    let list = match collect_find_matches(&body, query, case_sensitive, whole_word, use_regex) {
        Ok(v) => v,
        Err(_) => return,
    };
    if list.is_empty() {
        return;
    }
    let mut out = String::with_capacity(body.len());
    let mut last = 0usize;
    for m in &list {
        if m.start < last || m.end > body.len() {
            continue;
        }
        out.push_str(&body[last..m.start]);
        let matched = &body[m.start..m.end];
        let repl = if preserve_case {
            apply_preserve_case(matched, replacement)
        } else {
            replacement.to_string()
        };
        out.push_str(&repl);
        last = m.end;
    }
    out.push_str(&body[last..]);
    set_editor_draft(path, out, drafts, baselines, dirty, save_notice);
    match_index.set(0);
}

/// 中间栏文本文件查看 / 编辑。
///
/// `baselines` 保存上次加载或成功保存时的内容；编辑时与当前草稿比较以决定 dirty。
/// 保存仅通过 Cmd/Ctrl+S，无可见保存按钮。Cmd/Ctrl+F 打开查找，Cmd/Ctrl+H 打开替换。
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
    // key 随 path 重建；Signal 便于在多个 rsx 闭包中 Copy 读取路径。
    let editor_path = use_signal(|| path.clone());
    let mut find_open = use_signal(|| false);
    let mut replace_open = use_signal(|| false);
    let mut find_query = use_signal(String::new);
    let mut replace_text = use_signal(String::new);
    let mut case_sensitive = use_signal(|| false);
    let mut whole_word = use_signal(|| false);
    let mut use_regex = use_signal(|| false);
    let mut preserve_case = use_signal(|| false);
    let mut match_index = use_signal(|| 0usize);
    let mut find_error = use_signal(|| Option::<String>::None);
    let mut find_focus_tick = use_signal(|| 0u64);

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

    use_effect(move || {
        let _ = find_focus_tick();
        if find_open() {
            focus_file_find_input();
        }
    });

    let err = load_errors().get(&path_key).cloned();
    let content = drafts().get(&path_key).cloned();
    let is_dirty = dirty().contains(&path_key);
    let notice = save_notice();
    let show_status = is_dirty || notice.is_some();
    let path_edit = path.clone();
    let path_save = path.clone();

    let find_matches = use_memo(move || {
        if !find_open() {
            return Vec::<FindMatch>::new();
        }
        let body = drafts().get(&editor_path()).cloned().unwrap_or_default();
        match collect_find_matches(
            &body,
            &find_query(),
            case_sensitive(),
            whole_word(),
            use_regex(),
        ) {
            Ok(v) => v,
            Err(_) => Vec::new(),
        }
    });

    let find_err_msg = use_memo(move || {
        if !find_open() || find_query().is_empty() || !use_regex() {
            return None;
        }
        let body = drafts().get(&editor_path()).cloned().unwrap_or_default();
        collect_find_matches(&body, &find_query(), case_sensitive(), whole_word(), true).err()
    });

    use_effect(move || {
        find_error.set(find_err_msg());
        let list = find_matches();
        let len = list.len();
        if !find_open() || len == 0 {
            if match_index() != 0 {
                match_index.set(0);
            }
            return;
        }
        let idx = match_index().min(len - 1);
        if idx != match_index() {
            match_index.set(idx);
            return; // 等下一次 effect：高亮层 is-current 已更新后再滚
        }
        if let Some(body) = drafts().get(&editor_path()) {
            select_find_match_in_editor(body, list[idx], false);
        }
    });

    let match_count = find_matches().len();
    let match_label = if find_query().is_empty() {
        String::new()
    } else if let Some(err) = find_error() {
        err
    } else if match_count == 0 {
        "无结果".into()
    } else {
        format!("第 {} 项，共 {match_count} 项", match_index() + 1)
    };

    rsx! {
        section {
            class: EDITOR,
            "data-file-editor": true,
            onkeydown: move |e: KeyboardEvent| {
                if is_save_shortcut(&e) {
                    e.prevent_default();
                    file_editor_try_save(
                        &path_save,
                        drafts,
                        baselines,
                        dirty,
                        save_notice,
                    );
                    return;
                }
                if is_find_shortcut(&e) {
                    e.prevent_default();
                    find_open.set(true);
                    find_focus_tick.with_mut(|n| *n = n.wrapping_add(1));
                    return;
                }
                if is_replace_shortcut(&e) {
                    e.prevent_default();
                    find_open.set(true);
                    replace_open.set(true);
                    find_focus_tick.with_mut(|n| *n = n.wrapping_add(1));
                    return;
                }
                // Cmd/Ctrl+L 由外层 Console 打开 Chat；此处仅拦截浏览器默认行为。
                if is_chat_shortcut(&e) {
                    e.prevent_default();
                }
                if find_open() && e.key() == Key::Escape {
                    e.prevent_default();
                    find_open.set(false);
                    replace_open.set(false);
                }
            },
            if find_open() {
                div {
                    class: FIND,
                    "data-replace": replace_open(),
                    role: "search",
                    div { class: FIND_ROW,
                        button {
                            r#type: "button",
                            class: FIND_TOGGLE,
                            title: if replace_open() { "收起替换" } else { "展开替换" },
                            aria_label: if replace_open() { "收起替换" } else { "展开替换" },
                            aria_expanded: replace_open(),
                            onclick: move |_| {
                                replace_open.with_mut(|v| *v = !*v);
                            },
                            if replace_open() {
                                Icon { icon: LdChevronDown, width: 14, height: 14, fill: "currentColor" }
                            } else {
                                Icon { icon: LdChevronRight, width: 14, height: 14, fill: "currentColor" }
                            }
                        }
                        div { class: FIND_INPUT_WRAP,
                            input {
                                r#type: "text",
                                class: FIND_INPUT,
                                "data-find-query": true,
                                value: "{find_query}",
                                placeholder: "查找",
                                spellcheck: false,
                                oninput: move |e| {
                                    find_query.set(e.value());
                                    match_index.set(0);
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Enter {
                                        e.prevent_default();
                                        let delta: isize = if e.modifiers().contains(Modifiers::SHIFT) {
                                            -1
                                        } else {
                                            1
                                        };
                                        editor_goto_find_match(delta, &find_matches(), match_index);
                                    }
                                },
                            }
                            button {
                                r#type: "button",
                                class: FIND_OPT,
                                "data-active": case_sensitive(),
                                title: "区分大小写",
                                aria_label: "区分大小写",
                                aria_pressed: case_sensitive(),
                                onclick: move |_| {
                                    case_sensitive.with_mut(|v| *v = !*v);
                                    match_index.set(0);
                                },
                                Icon { icon: LdCaseSensitive, width: 13, height: 13, fill: "currentColor" }
                            }
                            button {
                                r#type: "button",
                                class: FIND_OPT,
                                "data-active": whole_word(),
                                title: "全字匹配",
                                aria_label: "全字匹配",
                                aria_pressed: whole_word(),
                                onclick: move |_| {
                                    whole_word.with_mut(|v| *v = !*v);
                                    match_index.set(0);
                                },
                                Icon { icon: LdWholeWord, width: 13, height: 13, fill: "currentColor" }
                            }
                            button {
                                r#type: "button",
                                class: FIND_OPT,
                                "data-active": use_regex(),
                                title: "使用正则表达式",
                                aria_label: "使用正则表达式",
                                aria_pressed: use_regex(),
                                onclick: move |_| {
                                    use_regex.with_mut(|v| *v = !*v);
                                    match_index.set(0);
                                },
                                Icon { icon: LdRegex, width: 13, height: 13, fill: "currentColor" }
                            }
                        }
                        div { class: FIND_ACTIONS,
                            span {
                                class: FIND_COUNT,
                                "data-empty": find_error().is_some()
                                    || (match_count == 0 && !find_query().is_empty()),
                                "{match_label}"
                            }
                            button {
                                r#type: "button",
                                class: FIND_BTN,
                                title: "上一个",
                                aria_label: "上一个",
                                disabled: match_count == 0,
                                onclick: move |_| {
                                    editor_goto_find_match(-1, &find_matches(), match_index);
                                },
                                Icon { icon: LdChevronUp, width: 14, height: 14, fill: "currentColor" }
                            }
                            button {
                                r#type: "button",
                                class: FIND_BTN,
                                title: "下一个",
                                aria_label: "下一个",
                                disabled: match_count == 0,
                                onclick: move |_| {
                                    editor_goto_find_match(1, &find_matches(), match_index);
                                },
                                Icon { icon: LdChevronDown, width: 14, height: 14, fill: "currentColor" }
                            }
                            button {
                                r#type: "button",
                                class: FIND_BTN,
                                title: "关闭",
                                aria_label: "关闭",
                                onclick: move |_| {
                                    find_open.set(false);
                                    replace_open.set(false);
                                },
                                Icon { icon: LdX, width: 14, height: 14, fill: "currentColor" }
                            }
                        }
                    }
                    if replace_open() {
                        div { class: FIND_ROW,
                            span { class: FIND_SPACER, aria_hidden: "true" }
                            div { class: FIND_INPUT_WRAP,
                                input {
                                    r#type: "text",
                                    class: FIND_INPUT,
                                    value: "{replace_text}",
                                    placeholder: "替换",
                                    spellcheck: false,
                                    oninput: move |e| replace_text.set(e.value()),
                                    onkeydown: move |e: KeyboardEvent| {
                                        if e.key() == Key::Enter {
                                            e.prevent_default();
                                            editor_replace_one(
                                                &editor_path(),
                                                drafts,
                                                baselines,
                                                dirty,
                                                save_notice,
                                                &find_matches(),
                                                match_index,
                                                &find_query(),
                                                &replace_text(),
                                                case_sensitive(),
                                                whole_word(),
                                                use_regex(),
                                                preserve_case(),
                                            );
                                        }
                                    },
                                }
                                button {
                                    r#type: "button",
                                    class: FIND_OPT,
                                "data-active": preserve_case(),
                                    title: "保留大小写",
                                    aria_label: "保留大小写",
                                    aria_pressed: preserve_case(),
                                    onclick: move |_| {
                                        preserve_case.with_mut(|v| *v = !*v);
                                    },
                                    Icon { icon: LdCaseUpper, width: 13, height: 13, fill: "currentColor" }
                                }
                            }
                            div { class: FIND_ACTIONS,
                                button {
                                    r#type: "button",
                                    class: FIND_BTN,
                                    title: "替换",
                                    aria_label: "替换",
                                    disabled: match_count == 0,
                                    onclick: move |_| {
                                        editor_replace_one(
                                            &editor_path(),
                                            drafts,
                                            baselines,
                                            dirty,
                                            save_notice,
                                            &find_matches(),
                                            match_index,
                                            &find_query(),
                                            &replace_text(),
                                            case_sensitive(),
                                            whole_word(),
                                            use_regex(),
                                            preserve_case(),
                                        );
                                    },
                                    Icon { icon: LdReplace, width: 14, height: 14, fill: "currentColor" }
                                }
                                button {
                                    r#type: "button",
                                    class: FIND_BTN,
                                    title: "全部替换",
                                    aria_label: "全部替换",
                                    disabled: match_count == 0,
                                    onclick: move |_| {
                                        editor_replace_all(
                                            &editor_path(),
                                            drafts,
                                            baselines,
                                            dirty,
                                            save_notice,
                                            match_index,
                                            &find_query(),
                                            &replace_text(),
                                            case_sensitive(),
                                            whole_word(),
                                            use_regex(),
                                            preserve_case(),
                                        );
                                    },
                                    Icon { icon: LdReplaceAll, width: 14, height: 14, fill: "currentColor" }
                                }
                            }
                        }
                    }
                }
            }
            if show_status {
                div { class: EDITOR_TOOLBAR,
                    div { class: EDITOR_ACTIONS,
                        if is_dirty {
                            span { class: EDITOR_DIRTY, "未保存" }
                        }
                        if let Some(msg) = notice {
                            span { class: EDITOR_SAVE_NOTICE, "{msg}" }
                        }
                    }
                }
            }
            if let Some(err) = err {
                div { class: EDITOR_ERROR, "{err}" }
            } else if let Some(body) = content {
                {
                    let line_count = count_text_lines(&body);
                    let code_style = format!("--ac-editor-lines: {line_count}");
                    let mut highlight = super::syntax::highlight_file_html(&body, &path_key);
                    if !body.ends_with('\n') && !body.is_empty() {
                        highlight.push('\n');
                    }
                    let find_list = find_matches();
                    let find_marks = if find_open() && !find_list.is_empty() {
                        let idx = match_index().min(find_list.len() - 1);
                        build_find_marks_html(&body, &find_list, idx)
                    } else {
                        String::new()
                    };
                    rsx! {
                        div { class: EDITOR_SCROLL,
                            "data-editor-scroll": true,
                            div {
                                class: EDITOR_CODE,
                                style: "{code_style}",
                                div {
                                    class: EDITOR_GUTTER,
                                    aria_hidden: "true",
                                    for n in 1..=line_count {
                                        div {
                                            key: "{n}",
                                            class: EDITOR_LN,
                                            "{n}"
                                        }
                                    }
                                }
                                div { class: EDITOR_SURFACE,
                                    if !find_marks.is_empty() {
                                        pre {
                                            class: EDITOR_FIND_MARKS,
                                            aria_hidden: "true",
                                            dangerous_inner_html: "{find_marks}",
                                        }
                                    }
                                    pre {
                                        class: EDITOR_HIGHLIGHT,
                                        dangerous_inner_html: "{highlight}",
                                    }
                                    textarea {
                                        class: EDITOR_TEXTAREA,
                                        "data-editor-textarea": true,
                                        value: "{body}",
                                        spellcheck: "false",
                                        oninput: move |e| {
                                            let v = e.value();
                                            set_editor_draft(
                                                &path_edit,
                                                v,
                                                drafts,
                                                baselines,
                                                dirty,
                                                save_notice,
                                            );
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: EDITOR_LOADING, "正在读取…" }
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

/// 本次工具编辑的行内 diff（新数据来自 old_string/new_string，兼容历史 oldText/newText）。
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

    rsx! {
        section { class: DIFF,
            "data-file-diff": true,
            div { class: DIFF_HEADER,
                div { class: DIFF_HEADING,
                    h2 { class: DIFF_TITLE, "{title}" }
                    p { class: DIFF_SUBTITLE, "{subtitle}" }
                    p { class: DIFF_PATH, title: "{path}", "{path}" }
                    p { class: DIFF_STATS,
                        span { class: DIFF_STAT_ADD, "+{add_n}" }
                        span { class: DIFF_STAT_DEL, "-{del_n}" }
                    }
                }
                if can_open_current {
                    button {
                        r#type: "button",
                        class: DIFF_OPEN_BTN,
                        onclick: move |_| on_open_current.call(()),
                        "打开当前文件"
                    }
                }
            }
            div { class: DIFF_INLINE,
                div { class: DIFF_INLINE_BODY, role: "table",
                    for (idx, line) in lines.iter().enumerate() {
                        {
                            let kind = match line.kind {
                                DiffLineKind::Delete => "del",
                                DiffLineKind::Insert => "add",
                                DiffLineKind::Context => "",
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
                                super::syntax::highlight_file_html(&line.text, &path);
                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: DIFF_LINE,
                                    "data-kind": kind,
                                    role: "row",
                                    span {
                                        class: DIFF_LN,
                                        aria_hidden: "true",
                                        "{old_ln}"
                                    }
                                    span {
                                        class: DIFF_LN,
                                        aria_hidden: "true",
                                        "{new_ln}"
                                    }
                                    span { class: DIFF_MARK, "{mark}" }
                                    span {
                                        class: DIFF_TEXT,
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
        section { class: MD_PREVIEW_PANE,
            match content() {
                Ok(text) if text.is_empty() => rsx! {
                    div { class: "px-[1.1rem] py-5 text-center text-[0.85rem] font-[650] text-secondary-3", "暂无内容" }
                },
                Ok(text) => rsx! {
                    div { class: "{MD_PREVIEW_SCROLL} scrollbar-hide",
                        crate::chat::ChatMarkdownBody { content: text }
                    }
                },
                Err(err) => rsx! {
                    div { class: EDITOR_ERROR, "{err}" }
                },
            }
        }
    }
}

/// 中间栏内置预览：`srcdoc` + `pusapreview://` 资源，不打开系统浏览器。
#[component]
pub fn FileHtmlPreviewPane(path: String) -> Element {
    let path_key = path.clone();
    let frame = use_memo(move || fs_html_preview_frame(&path_key));
    let title = fs_file_title(&path);
    let mut viewport = use_signal(|| HtmlPreviewViewport::Desktop);
    let mut drawer_open = use_signal(|| false);
    let mut model_idx = use_signal(|| 0usize);
    let default_model = &HTML_PREVIEW_PHONE_MODELS[0];
    let mut phone_w = use_signal(|| default_model.width);
    let mut phone_h = use_signal(|| default_model.height);
    let mut resized = use_signal(|| false);
    let mut drag = use_signal(|| None::<HtmlPreviewResizeDrag>);
    // 舞台内容区尺寸（不含 padding），由 ResizeObserver 回填；用于把机身缩放到能完整放下。
    let mut stage_size = use_signal(|| (0.0f64, 0.0f64));
    let is_mobile = viewport() == HtmlPreviewViewport::Mobile;
    let (stage_w, stage_h) = stage_size();
    let phone_scale = html_preview_fit_scale(stage_w, stage_h, phone_w(), phone_h());
    let slot_w = (phone_w() + 2 * HTML_PREVIEW_PHONE_BEZEL) as f64 * phone_scale;
    let slot_h = (phone_h() + 2 * HTML_PREVIEW_PHONE_BEZEL) as f64 * phone_scale;
    let stage_class = if is_mobile {
        PREVIEW_STAGE_MOBILE
    } else {
        PREVIEW_STAGE
    };
    let selected_model = HTML_PREVIEW_PHONE_MODELS
        .get(model_idx())
        .unwrap_or(&HTML_PREVIEW_PHONE_MODELS[0]);
    let iframe_title = format!("预览 {title}");

    rsx! {
        section {
            class: PREVIEW_PANE,
            "data-resizing": drag().is_some(),
            onmousemove: move |event| {
                let Some(current) = drag() else {
                    return;
                };
                let coords = event.data.client_coordinates();
                let (next_w, next_h) = html_preview_apply_resize(current, coords.x, coords.y);
                if next_w != phone_w() || next_h != phone_h() {
                    phone_w.set(next_w);
                    phone_h.set(next_h);
                    resized.set(true);
                }
            },
            onmouseup: move |_| drag.set(None),
            onmouseleave: move |_| drag.set(None),
            div {
                class: "{stage_class}",
                onresize: move |event| {
                    if let Ok(size) = event.data().get_content_box_size() {
                        let next = (size.width, size.height);
                        if next != stage_size() {
                            stage_size.set(next);
                        }
                    }
                },
                if is_mobile {
                    // slot 占据缩放后的实际尺寸，机身内部仍按真实设备像素排版，只做视觉缩放，
                    // 这样 iframe 里的媒体查询与真机一致，同时机身永远不会顶到舞台上下边。
                    div {
                        class: PREVIEW_DEVICE_SLOT,
                        style: "width: {slot_w:.1}px; height: {slot_h:.1}px;",
                        div {
                            class: PREVIEW_DEVICE,
                            style: "width: {phone_w()}px; height: {phone_h()}px; transform: scale({phone_scale});",
                            div { class: PREVIEW_DEVICE_BODY,
                                match frame() {
                                    Ok(HtmlPreviewFrame::Url(src)) => rsx! {
                                        iframe {
                                            class: PREVIEW_FRAME_FILL,
                                            src: "{src}",
                                            title: "{iframe_title}",
                                        }
                                    },
                                    Ok(HtmlPreviewFrame::SrcDoc(srcdoc)) => rsx! {
                                        iframe {
                                            class: PREVIEW_FRAME_FILL,
                                            srcdoc: "{srcdoc}",
                                            title: "{iframe_title}",
                                        }
                                    },
                                    Err(err) => rsx! {
                                        div { class: EDITOR_ERROR, "{err}" }
                                    },
                                }
                            }
                            for (name, edge) in HTML_PREVIEW_RESIZE_HANDLES {
                                div {
                                    class: html_preview_handle_class(name),
                                    onmousedown: move |event| {
                                        event.prevent_default();
                                        event.stop_propagation();
                                        let coords = event.data.client_coordinates();
                                        drag.set(Some(HtmlPreviewResizeDrag {
                                            edge,
                                            start_x: coords.x,
                                            start_y: coords.y,
                                            start_w: phone_w(),
                                            start_h: phone_h(),
                                            scale: phone_scale,
                                        }));
                                    },
                                }
                            }
                        }
                    }
                } else {
                    match frame() {
                        Ok(HtmlPreviewFrame::Url(src)) => rsx! {
                            iframe {
                                class: PREVIEW_FRAME,
                                src: "{src}",
                                title: "{iframe_title}",
                            }
                        },
                        Ok(HtmlPreviewFrame::SrcDoc(srcdoc)) => rsx! {
                            iframe {
                                class: PREVIEW_FRAME,
                                srcdoc: "{srcdoc}",
                                title: "{iframe_title}",
                            }
                        },
                        Err(err) => rsx! {
                            div { class: EDITOR_ERROR, "{err}" }
                        },
                    }
                }
            }
            if is_mobile {
                div { class: PREVIEW_PHONE_BAR,
                    button {
                        r#type: "button",
                        class: PREVIEW_CHIP,
                        "data-active": drawer_open(),
                        title: "选择手机型号",
                        aria_label: "选择手机型号",
                        aria_expanded: drawer_open(),
                        onclick: move |_| drawer_open.toggle(),
                        Icon {
                            icon: LdSmartphone,
                            width: 14,
                            height: 14,
                            fill: "currentColor",
                        }
                        span { "{selected_model.label}" }
                    }
                }
                if drawer_open() {
                    button {
                        r#type: "button",
                        class: PREVIEW_DRAWER_MASK,
                        aria_label: "关闭型号列表",
                        onclick: move |_| drawer_open.set(false),
                    }
                }
                aside {
                    class: PREVIEW_DRAWER,
                    "data-open": drawer_open(),
                    "aria-label": "手机型号",
                    p { class: PREVIEW_DRAWER_TITLE, "手机型号" }
                    for (idx, model) in HTML_PREVIEW_PHONE_MODELS.iter().enumerate() {
                        button {
                            r#type: "button",
                            class: PREVIEW_MODEL,
                            "data-active": idx == model_idx(),
                            onclick: move |_| {
                                model_idx.set(idx);
                                phone_w.set(HTML_PREVIEW_PHONE_MODELS[idx].width);
                                phone_h.set(HTML_PREVIEW_PHONE_MODELS[idx].height);
                                resized.set(false);
                                drawer_open.set(false);
                            },
                            span { class: PREVIEW_MODEL_NAME, "{model.label}" }
                            span { class: PREVIEW_MODEL_SIZE, "{model.width} × {model.height}" }
                        }
                    }
                }
            }
            // 悬浮球：单击直接在桌面 / 手机视口之间切换，图标显示当前模式。
            div { class: PREVIEW_FAB,
                button {
                    r#type: "button",
                    class: PREVIEW_FAB_BALL,
                    title: if is_mobile { "切换为桌面预览" } else { "切换为手机预览" },
                    aria_label: if is_mobile { "切换为桌面预览" } else { "切换为手机预览" },
                    aria_pressed: is_mobile,
                    onclick: move |_| {
                        if viewport() == HtmlPreviewViewport::Mobile {
                            viewport.set(HtmlPreviewViewport::Desktop);
                            drawer_open.set(false);
                        } else {
                            viewport.set(HtmlPreviewViewport::Mobile);
                        }
                    },
                    if is_mobile {
                        Icon {
                            icon: LdSmartphone,
                            width: 16,
                            height: 16,
                            fill: "currentColor",
                        }
                    } else {
                        Icon {
                            icon: LdMonitor,
                            width: 16,
                            height: 16,
                            fill: "currentColor",
                        }
                    }
                }
                // 手动拖过机身尺寸后才出现：还原为当前型号的默认尺寸。
                if is_mobile && resized() {
                    button {
                        r#type: "button",
                        class: PREVIEW_FAB_BALL_SECONDARY,
                        title: "还原为当前型号默认尺寸",
                        aria_label: "还原尺寸",
                        onclick: move |_| {
                            let model = HTML_PREVIEW_PHONE_MODELS
                                .get(model_idx())
                                .unwrap_or(&HTML_PREVIEW_PHONE_MODELS[0]);
                            phone_w.set(model.width);
                            phone_h.set(model.height);
                            resized.set(false);
                        },
                        Icon {
                            icon: LdRotateCcw,
                            width: 16,
                            height: 16,
                            fill: "currentColor",
                        }
                    }
                }
            }
            if drag().is_some() {
                div { class: PREVIEW_RESIZE_MASK }
            }
        }
    }
}

/// 图片内置预览：居中自适应，支持缩放容器滚动。
#[component]
pub fn FileImagePreviewPane(path: String) -> Element {
    let path_key = path.clone();
    let src = use_memo(move || fs_image_preview_src(&path_key));
    let title = fs_file_title(&path);
    let alt = format!("预览 {title}");

    rsx! {
        section { class: PREVIEW_PANE,
            "data-preview-pane": true,
            div { class: PREVIEW_STAGE_IMAGE,
                match src() {
                    Ok(src) => rsx! {
                        img {
                            class: PREVIEW_IMAGE,
                            src: "{src}",
                            alt: "{alt}",
                            draggable: "false",
                        }
                    },
                    Err(err) => rsx! {
                        div { class: EDITOR_ERROR, "{err}" }
                    },
                }
            }
        }
    }
}

fn format_audio_clock(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "0:00".into();
    }
    let total = secs.floor() as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{m}:{s:02}")
}

/// 音频内置预览：播放按钮 + 进度条 + 时长。
#[component]
pub fn FileAudioPreviewPane(path: String) -> Element {
    let path_key = path.clone();
    let src = use_memo(move || fs_audio_preview_src(&path_key));
    let title = fs_file_title(&path);
    let audio_dom_id = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        path.hash(&mut h);
        format!("ac-audio-{}", h.finish())
    };
    let track_dom_id = format!("{audio_dom_id}-track");
    let fill_dom_id = format!("{audio_dom_id}-fill");
    let mut playing = use_signal(|| false);
    let mut current_secs = use_signal(|| 0.0_f64);
    // 首屏即用本地文件头解析时长，避免依赖 WebView `<audio>.duration`（自定义协议下常为 NaN）。
    let initial_duration = fs_audio_duration_secs(&path).unwrap_or(0.0);
    let mut duration_secs = use_signal(move || {
        if initial_duration.is_finite() && initial_duration > 0.0 {
            initial_duration
        } else {
            0.0
        }
    });
    let mut load_error = use_signal(|| None::<String>);

    // 绑定播放进度；eval 上下文里必须用注入的 `dioxus.send`，不是 `window.dioxus`。
    // 进度条宽度由 JS 每帧直写 DOM，时钟文案节流回传，避免整卡重渲染卡顿。
    {
        let audio_dom_id = audio_dom_id.clone();
        let fill_dom_id = fill_dom_id.clone();
        let src_ok = src().is_ok();
        let seed_duration = duration_secs();
        use_effect(move || {
            playing.set(false);
            current_secs.set(0.0);
            load_error.set(None);
            if !src_ok {
                return;
            }
            let audio_id_js =
                serde_json::to_string(&audio_dom_id).unwrap_or_else(|_| "\"\"".into());
            let fill_id_js = serde_json::to_string(&fill_dom_id).unwrap_or_else(|_| "\"\"".into());
            let seed_js = if seed_duration.is_finite() && seed_duration > 0.0 {
                format!("{seed_duration:.6}")
            } else {
                "0".into()
            };
            spawn(async move {
                let mut eval = document::eval(&format!(
                    r#"
(async () => {{
  const audioId = {audio_id_js};
  const fillId = {fill_id_js};
  const send = (payload) => {{
    try {{ dioxus.send(payload); }} catch (_) {{}}
  }};
  const cleanupKey = '__acAudioPreviewCleanup_' + audioId;
  try {{
    const prev = window[cleanupKey];
    if (typeof prev === 'function') prev();
  }} catch (_) {{}}

  let raf = 0;
  let aborted = false;
  let knownDuration = {seed_js};
  let lastUiMs = 0;

  const audio = () => document.getElementById(audioId);
  const fill = () => document.getElementById(fillId);
  const finiteDuration = (el) => {{
    const d = el && el.duration;
    return (Number.isFinite(d) && d > 0) ? d : 0;
  }};
  const paint = (current) => {{
    const el = fill();
    if (!el) return;
    const pct = knownDuration > 0
      ? Math.min(100, Math.max(0, (current / knownDuration) * 100))
      : 0;
    el.style.width = pct.toFixed(3) + '%';
  }};
  const syncUi = (forcePlaying, force) => {{
    if (aborted) return;
    const el = audio();
    if (!el) return;
    const fromEl = finiteDuration(el);
    if (fromEl > 0) knownDuration = fromEl;
    const current = el.currentTime || 0;
    paint(current);
    const now = performance.now();
    if (!force && now - lastUiMs < 200) return;
    lastUiMs = now;
    const playing = typeof forcePlaying === 'boolean' ? forcePlaying : !el.paused;
    send({{
      kind: 'time',
      current,
      duration: knownDuration,
      playing,
    }});
  }};

  const tick = () => {{
    if (aborted) return;
    const el = audio();
    if (!el) return;
    const fromEl = finiteDuration(el);
    if (fromEl > 0) knownDuration = fromEl;
    paint(el.currentTime || 0);
    if (!el.paused) {{
      // 时钟约 5Hz 回传；进度条跟 RAF。
      syncUi(true, false);
      raf = requestAnimationFrame(tick);
    }}
  }};

  const onPlay = () => {{
    send({{ kind: 'playing', playing: true }});
    syncUi(true, true);
    cancelAnimationFrame(raf);
    raf = requestAnimationFrame(tick);
  }};
  const onPause = () => {{ syncUi(false, true); }};
  const onEnded = () => {{
    const el = audio();
    if (el) try {{ el.currentTime = 0; }} catch (_) {{}}
    paint(0);
    send({{ kind: 'playing', playing: false }});
    syncUi(false, true);
  }};
  const onMeta = () => syncUi(undefined, true);
  const onError = () => {{ send({{ kind: 'error', message: '无法加载音频' }}); }};

  const bindAudio = () => {{
    const el = audio();
    if (!el) return false;
    for (const [ev, fn] of [
      ['play', onPlay], ['pause', onPause], ['ended', onEnded],
      ['loadedmetadata', onMeta], ['durationchange', onMeta],
      ['loadeddata', onMeta], ['canplay', onMeta],
      ['error', onError],
    ]) {{
      el.removeEventListener(ev, fn);
      el.addEventListener(ev, fn);
    }}
    syncUi(undefined, true);
    return true;
  }};

  window[cleanupKey] = () => {{
    aborted = true;
    try {{ cancelAnimationFrame(raf); }} catch (_) {{}}
    const el = audio();
    if (el) {{
      try {{ el.pause(); }} catch (_) {{}}
      for (const [ev, fn] of [
        ['play', onPlay], ['pause', onPause], ['ended', onEnded],
        ['loadedmetadata', onMeta], ['durationchange', onMeta],
        ['loadeddata', onMeta], ['canplay', onMeta],
        ['error', onError],
      ]) el.removeEventListener(ev, fn);
    }}
  }};

  await new Promise((r) => requestAnimationFrame(() => r()));
  if (aborted) return;
  bindAudio();
  if (knownDuration > 0) send({{ kind: 'duration', duration: knownDuration }});
  await new Promise(() => {{}});
}})();
"#,
                    audio_id_js = audio_id_js,
                    fill_id_js = fill_id_js,
                    seed_js = seed_js,
                ));
                while let Ok(val) = eval.recv::<serde_json::Value>().await {
                    let kind = val.get("kind").and_then(|k| k.as_str()).unwrap_or("");
                    match kind {
                        "duration" => {
                            if let Some(d) = val.get("duration").and_then(|v| v.as_f64()) {
                                if d.is_finite() && d > 0.0 {
                                    duration_secs.set(d);
                                }
                            }
                        }
                        "time" => {
                            current_secs
                                .set(val.get("current").and_then(|v| v.as_f64()).unwrap_or(0.0));
                            if let Some(d) = val.get("duration").and_then(|v| v.as_f64()) {
                                if d.is_finite() && d > 0.0 {
                                    duration_secs.set(d);
                                }
                            }
                            if let Some(p) = val.get("playing").and_then(|v| v.as_bool()) {
                                playing.set(p);
                            }
                        }
                        "playing" => {
                            if let Some(p) = val.get("playing").and_then(|v| v.as_bool()) {
                                playing.set(p);
                            }
                        }
                        "error" => {
                            let msg = val
                                .get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("音频预览失败")
                                .to_string();
                            load_error.set(Some(msg));
                        }
                        _ => {}
                    }
                }
            });
        });
    }

    let toggle_play = {
        let audio_dom_id = audio_dom_id.clone();
        move |_| {
            let id_js = serde_json::to_string(&audio_dom_id).unwrap_or_else(|_| "\"\"".into());
            let _ = document::eval(&format!(
                r#"(() => {{
  const el = document.getElementById({id_js});
  if (!el) return;
  if (el.paused) {{ el.play().catch(() => {{}}); }}
  else {{ el.pause(); }}
}})();"#
            ));
        }
    };

    let seek = {
        let audio_dom_id = audio_dom_id.clone();
        let track_dom_id = track_dom_id.clone();
        let fill_dom_id = fill_dom_id.clone();
        move |evt: MouseEvent| {
            let id_js = serde_json::to_string(&audio_dom_id).unwrap_or_else(|_| "\"\"".into());
            let track_js = serde_json::to_string(&track_dom_id).unwrap_or_else(|_| "\"\"".into());
            let fill_js = serde_json::to_string(&fill_dom_id).unwrap_or_else(|_| "\"\"".into());
            let fallback = duration_secs();
            let coords = evt.data.client_coordinates();
            let _ = document::eval(&format!(
                r#"(() => {{
  const el = document.getElementById({id_js});
  const track = document.getElementById({track_js});
  const fill = document.getElementById({fill_js});
  if (!el || !track) return;
  const dur = (Number.isFinite(el.duration) && el.duration > 0) ? el.duration : {fallback};
  if (!(dur > 0)) return;
  const rect = track.getBoundingClientRect();
  const x = Math.min(Math.max({client_x} - rect.left, 0), rect.width || 1);
  const ratio = x / (rect.width || 1);
  el.currentTime = ratio * dur;
  if (fill) fill.style.width = (ratio * 100).toFixed(3) + '%';
}})();"#,
                id_js = id_js,
                track_js = track_js,
                fill_js = fill_js,
                fallback = if fallback.is_finite() && fallback > 0.0 {
                    fallback
                } else {
                    0.0
                },
                client_x = coords.x,
            ));
        }
    };

    rsx! {
        section { class: AUDIO_PANE,
            "data-preview-pane": true,
            div { class: AUDIO_STAGE,
                match src() {
                    Ok(src) => rsx! {
                        audio {
                            id: "{audio_dom_id}",
                            class: AUDIO_EL,
                            src: "{src}",
                            preload: "metadata",
                        }
                        div { class: AUDIO_CARD,
                            div { class: AUDIO_TITLE, "{title}" }
                            button {
                                r#type: "button",
                                class: AUDIO_PLAY,
                                title: if playing() { "暂停" } else { "播放" },
                                aria_label: if playing() { "暂停" } else { "播放" },
                                onclick: toggle_play,
                                if playing() {
                                    Icon { icon: LdPause, width: 28, height: 28, fill: "currentColor" }
                                } else {
                                    Icon { icon: LdPlay, width: 28, height: 28, fill: "currentColor" }
                                }
                            }
                            div {
                                id: "{track_dom_id}",
                                class: AUDIO_TRACK,
                                role: "slider",
                                aria_label: "音频进度",
                                aria_valuemin: "0",
                                aria_valuemax: "100",
                                onclick: seek,
                                div {
                                    id: "{fill_dom_id}",
                                    class: AUDIO_TRACK_FILL,
                                }
                            }
                            div { class: AUDIO_META,
                                span { "{format_audio_clock(current_secs())}" }
                                span { "{format_audio_clock(duration_secs())}" }
                            }
                            if let Some(err) = load_error() {
                                div { class: AUDIO_ERROR, "{err}" }
                            }
                        }
                    },
                    Err(err) => rsx! {
                        div { class: EDITOR_ERROR, "{err}" }
                    },
                }
            }
        }
    }
}

/// PDF 内置预览：整页 iframe，渲染交给 WebView（WKWebView / WebView2 均内置 PDF 阅读器）。
#[component]
pub fn FilePdfPreviewPane(path: String) -> Element {
    let path_key = path.clone();
    let src = use_memo(move || fs_pdf_preview_src(&path_key));
    let title = fs_file_title(&path);
    let iframe_title = format!("预览 {title}");

    rsx! {
        section { class: PREVIEW_PANE_SELECTABLE,
            "data-preview-pane": true,
            div { class: PREVIEW_STAGE_PAPER,
                match src() {
                    Ok(src) => rsx! {
                        iframe {
                            class: PREVIEW_FRAME,
                            src: "{src}",
                            title: "{iframe_title}",
                        }
                    },
                    Err(err) => rsx! {
                        div { class: EDITOR_ERROR, "{err}" }
                    },
                }
            }
        }
    }
}

/// STL 内置预览：整页 iframe 载入自带的 WebGL 查看器（旋转 / 缩放 / 平移）。
#[component]
pub fn FileStlPreviewPane(path: String) -> Element {
    let path_key = path.clone();
    let frame = use_memo(move || fs_stl_preview_frame(&path_key));
    let title = fs_file_title(&path);
    let iframe_title = format!("预览 {title}");

    rsx! {
        section { class: PREVIEW_PANE,
            "data-preview-pane": true,
            div { class: PREVIEW_STAGE_PAPER,
                match frame() {
                    Ok(HtmlPreviewFrame::Url(src)) => rsx! {
                        iframe {
                            class: PREVIEW_FRAME,
                            src: "{src}",
                            title: "{iframe_title}",
                        }
                    },
                    Ok(HtmlPreviewFrame::SrcDoc(srcdoc)) => rsx! {
                        iframe {
                            class: PREVIEW_FRAME,
                            srcdoc: "{srcdoc}",
                            title: "{iframe_title}",
                        }
                    },
                    Err(err) => rsx! {
                        div { class: EDITOR_ERROR, "{err}" }
                    },
                }
            }
        }
    }
}

/// Office 内置预览：整页 iframe 载入 JS 查看器（docx/xlsx/pptx；旧版 doc/xls/ppt）。
/// `.doc` / `.ppt` 可能阻塞转换，放到后台线程并显示加载态。
/// 主题跟随应用 `html[data-theme]`：首屏写入 query/`__PUSA_OFFICE__`，切换时 `postMessage`。
#[component]
pub fn FileOfficePreviewPane(path: String) -> Element {
    let path_key = path.clone();
    let mut frame = use_signal(|| None::<Result<HtmlPreviewFrame, String>>);
    let mut ui_theme = use_signal(crate::shell::theme::load);
    let title = fs_file_title(&path);
    let iframe_title = format!("预览 {title}");
    let loading_hint = if fs_office_preview_may_convert(&path) {
        "正在转换旧版 Office 文档…".to_string()
    } else {
        "正在准备预览…".to_string()
    };

    // 与终端一致：监听主文档 `data-theme`；并应答 iframe 的 `pusa-theme-request`。
    use_effect(move || {
        let mut eval = document::eval(
            r#"(() => {
  const read = () => {
    const v = (document.documentElement.getAttribute('data-theme') || 'light').toLowerCase();
    return v === 'dark' ? 'dark' : 'light';
  };
  const push = () => { try { dioxus.send(read()); } catch (_) {} };
  const reply = (source) => {
    try {
      if (source) source.postMessage({ type: 'pusa-theme', theme: read() }, '*');
    } catch (_) {}
  };
  const onMsg = (ev) => {
    if (ev && ev.data && ev.data.type === 'pusa-theme-request') reply(ev.source);
  };
  window.addEventListener('message', onMsg);
  push();
  const obs = new MutationObserver(push);
  obs.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
})();"#,
        );
        spawn(async move {
            loop {
                let Ok(raw) = eval.recv::<String>().await else {
                    break;
                };
                let next = crate::shell::theme::UiTheme::from_stored(&raw);
                if ui_theme() != next {
                    ui_theme.set(next);
                }
            }
        });
    });

    // 路径变化时重建 iframe；主题用 peek，避免切主题触发旧版转换重跑。
    use_effect({
        let path = path_key.clone();
        move || {
            frame.set(None);
            let theme = ui_theme.peek().as_attr().to_string();
            #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
            {
                let path = path.clone();
                spawn(async move {
                    let result =
                        tokio::task::spawn_blocking(move || fs_office_preview_frame(&path, &theme))
                            .await
                            .unwrap_or_else(|e| Err(format!("预览任务失败：{e}")));
                    frame.set(Some(result));
                });
            }
            #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
            {
                frame.set(Some(fs_office_preview_frame(&path, &theme)));
            }
        }
    });

    // iframe 就绪或主题变更：向查看器 postMessage（不重载，保留滚动位置）。
    use_effect(move || {
        let theme = ui_theme().as_attr();
        let ready = matches!(frame(), Some(Ok(_)));
        if !ready {
            return;
        }
        let _ = document::eval(&office_preview_post_theme_js(theme));
    });

    rsx! {
        section { class: PREVIEW_PANE_SELECTABLE,
            "data-preview-pane": true,
            div { class: PREVIEW_STAGE_PAPER,
                match frame() {
                    None => rsx! {
                        div { class: EDITOR_LOADING, "{loading_hint}" }
                    },
                    Some(Ok(HtmlPreviewFrame::Url(src))) => rsx! {
                        iframe {
                            class: PREVIEW_FRAME,
                            "data-office-preview-frame": true,
                            src: "{src}",
                            title: "{iframe_title}",
                        }
                    },
                    Some(Ok(HtmlPreviewFrame::SrcDoc(srcdoc))) => rsx! {
                        iframe {
                            class: PREVIEW_FRAME,
                            "data-office-preview-frame": true,
                            srcdoc: "{srcdoc}",
                            title: "{iframe_title}",
                        }
                    },
                    Some(Err(err)) => rsx! {
                        div { class: EDITOR_ERROR, "{err}" }
                    },
                }
            }
        }
    }
}
