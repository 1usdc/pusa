//! HTML 文件夹预览：走 `pusapreview://` 自定义协议，避免 Dioxus 把 `http://` 丢给系统浏览器。

use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub const PROTOCOL_NAME: &str = "pusapreview";

/// 内置 STL 查看器页面（纯 WebGL，无第三方依赖，离线可用）。
const STL_VIEWER_HTML: &str = include_str!("../../assets/stl-viewer/index.html");
/// 查看器在预览站点内的虚拟文件名：`pusapreview://localhost/<token>/__pusa_stl_viewer__.html?src=…`。
/// 与被预览的 `.stl` 同源同目录，页面里 `fetch(src)` 不涉及跨域。
const STL_VIEWER_FILE: &str = "__pusa_stl_viewer__.html";

/// 内置 Office 查看器（docx/xlsx/pptx + 旧版 doc/xls/ppt；JS 库随仓库 vendoring，离线可用）。
const OFFICE_VIEWER_HTML: &str = include_str!("../../assets/office-viewer/index.html");
const OFFICE_VIEWER_FILE: &str = "__pusa_office_viewer__.html";
const OFFICE_MAMMOTH_JS: &[u8] = include_bytes!("../../assets/office-viewer/mammoth.browser.min.js");
const OFFICE_XLSX_JS: &[u8] = include_bytes!("../../assets/office-viewer/xlsx.full.min.js");
const OFFICE_JSZIP_JS: &[u8] = include_bytes!("../../assets/office-viewer/jszip.min.js");
const OFFICE_MAMMOTH_FILE: &str = "__pusa_office_mammoth.min.js";
const OFFICE_XLSX_FILE: &str = "__pusa_office_xlsx.full.min.js";
const OFFICE_JSZIP_FILE: &str = "__pusa_office_jszip.min.js";

fn roots() -> &'static Mutex<HashMap<String, PathBuf>> {
    static ROOTS: OnceLock<Mutex<HashMap<String, PathBuf>>> = OnceLock::new();
    ROOTS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn is_html_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("html") || e.eq_ignore_ascii_case("htm"))
}

pub fn is_pdf_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
}

pub fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico" | "bmp" | "avif"
            )
        })
}

pub fn is_audio_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "mp3" | "wav" | "ogg" | "oga" | "m4a" | "aac" | "flac" | "opus" | "webm"
            )
        })
}

/// 图片预览：`pusapreview` 直接回源文件，供 `<img src>` 加载。
pub fn image_preview_src(image_path: &Path) -> Result<String, String> {
    if !is_image_path(image_path) {
        return Err("不是可预览的图片文件。".into());
    }
    if !image_path.is_file() {
        return Err("图片文件不存在。".into());
    }
    let (token, file_name) = register_file(image_path)?;
    Ok(format!(
        "{}{}",
        asset_base(&token),
        encode_path_seg(&file_name)
    ))
}

/// 音频预览：`pusapreview` 直接回源文件，供 `<audio src>` / Web Audio 解码。
pub fn audio_preview_src(audio_path: &Path) -> Result<String, String> {
    if !is_audio_path(audio_path) {
        return Err("不是可预览的音频文件。".into());
    }
    if !audio_path.is_file() {
        return Err("音频文件不存在。".into());
    }
    let (token, file_name) = register_file(audio_path)?;
    Ok(format!(
        "{}{}",
        asset_base(&token),
        encode_path_seg(&file_name)
    ))
}

/// 从本地音频文件解析时长（秒）。WKWebView 自定义协议下 `<audio>.duration` 常为 NaN，
/// 因此预览 UI 优先用这里的结果。支持常见容器的轻量探测，失败返回 `None`。
pub fn audio_duration_secs(audio_path: &Path) -> Option<f64> {
    if !is_audio_path(audio_path) || !audio_path.is_file() {
        return None;
    }
    let data = fs::read(audio_path).ok()?;
    if data.len() < 12 {
        return None;
    }
    let ext = audio_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "wav" => wav_duration_secs(&data),
        "flac" => flac_duration_secs(&data),
        "mp3" => mp3_duration_secs(&data),
        "ogg" | "oga" | "opus" => ogg_duration_secs(&data),
        "m4a" | "aac" | "webm" => None,
        _ => None,
    }
}

fn read_u16_le(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(at..at + 2)?.try_into().ok()?))
}

fn read_u32_le(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

fn read_u32_be(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

fn wav_duration_secs(data: &[u8]) -> Option<f64> {
    if data.get(0..4)? != b"RIFF" || data.get(8..12)? != b"WAVE" {
        return None;
    }
    let mut offset = 12usize;
    let mut byte_rate = 0u32;
    let mut data_size = None::<u32>;
    while offset + 8 <= data.len() {
        let id = data.get(offset..offset + 4)?;
        let size = read_u32_le(data, offset + 4)? as usize;
        let body = offset + 8;
        if id == b"fmt " && size >= 16 {
            // audio_format(2) channels(2) sample_rate(4) byte_rate(4) …
            byte_rate = read_u32_le(data, body + 8)?;
        } else if id == b"data" {
            data_size = Some(size as u32);
            break;
        }
        // Chunks are word-aligned.
        offset = body.saturating_add(size).saturating_add(size % 2);
    }
    if byte_rate == 0 {
        return None;
    }
    let size = data_size? as f64;
    let secs = size / f64::from(byte_rate);
    (secs.is_finite() && secs > 0.0).then_some(secs)
}

fn flac_duration_secs(data: &[u8]) -> Option<f64> {
    // "fLaC" + STREAMINFO metadata block.
    if data.get(0..4)? != b"fLaC" {
        return None;
    }
    let header = data.get(4..)?;
    // STREAMINFO type must be 0; body starts at header[4].
    if header[0] & 0x7F != 0 {
        return None;
    }
    let body = header.get(4..)?;
    if body.len() < 18 {
        return None;
    }
    let sample_rate = ((u32::from(body[10]) << 12)
        | (u32::from(body[11]) << 4)
        | (u32::from(body[12]) >> 4)) as f64;
    let total_samples = ((u64::from(body[13] & 0x0F) << 32)
        | (u64::from(body[14]) << 24)
        | (u64::from(body[15]) << 16)
        | (u64::from(body[16]) << 8)
        | u64::from(body[17])) as f64;
    if sample_rate <= 0.0 || total_samples <= 0.0 {
        return None;
    }
    let secs = total_samples / sample_rate;
    (secs.is_finite() && secs > 0.0).then_some(secs)
}

fn mp3_duration_secs(data: &[u8]) -> Option<f64> {
    // Prefer Xing/Info frame (VBR) or approximate via first frame bitrate (CBR).
    let mut i = 0usize;
    // Skip ID3v2.
    if data.get(0..3) == Some(b"ID3".as_ref()) && data.len() >= 10 {
        let size = ((u32::from(data[6] & 0x7F) << 21)
            | (u32::from(data[7] & 0x7F) << 14)
            | (u32::from(data[8] & 0x7F) << 7)
            | u32::from(data[9] & 0x7F)) as usize;
        i = 10 + size;
    }
    while i + 4 < data.len() {
        if data[i] == 0xFF && data[i + 1] & 0xE0 == 0xE0 {
            break;
        }
        i += 1;
        if i > 64 * 1024 {
            return None;
        }
    }
    if i + 4 >= data.len() {
        return None;
    }
    let header = read_u32_be(data, i)?;
    let version_id = (header >> 19) & 0b11;
    let layer = (header >> 17) & 0b11;
    let bitrate_idx = ((header >> 12) & 0b1111) as usize;
    let sample_idx = ((header >> 10) & 0b11) as usize;
    let padding = (header >> 9) & 1;
    if version_id == 1 || layer == 0 || bitrate_idx == 0 || bitrate_idx == 15 || sample_idx == 3 {
        return None;
    }
    let mpeg1 = version_id == 3;
    let sample_rate = match (mpeg1, sample_idx) {
        (true, 0) => 44100.0,
        (true, 1) => 48000.0,
        (true, 2) => 32000.0,
        (false, 0) => 22050.0,
        (false, 1) => 24000.0,
        (false, 2) => 16000.0,
        _ => return None,
    };
    // Layer III bitrates (kbps), MPEG1 / MPEG2.
    let bitrate = if mpeg1 {
        [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0][bitrate_idx]
    } else {
        [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0][bitrate_idx]
    } as f64;
    if bitrate <= 0.0 {
        return None;
    }
    let samples_per_frame = if mpeg1 { 1152.0 } else { 576.0 };
    let frame_len = ((samples_per_frame / 8.0 * bitrate * 1000.0 / sample_rate)
        + padding as f64)
        .floor() as usize;
    // Xing/Info after side info.
    let side = if mpeg1 { 36 } else { 21 };
    let xing_at = i + 4 + side;
    if let Some(tag) = data.get(xing_at..xing_at + 4) {
        if tag == b"Xing" || tag == b"Info" {
            let flags = read_u32_be(data, xing_at + 4)?;
            if flags & 1 != 0 {
                let frames = read_u32_be(data, xing_at + 8)? as f64;
                let secs = frames * samples_per_frame / sample_rate;
                return (secs.is_finite() && secs > 0.0).then_some(secs);
            }
        }
    }
    // CBR fallback: (file_size - offset) / bytes_per_sec.
    if frame_len == 0 {
        return None;
    }
    let audio_bytes = data.len().saturating_sub(i) as f64;
    let secs = audio_bytes * 8.0 / (bitrate * 1000.0);
    (secs.is_finite() && secs > 0.0).then_some(secs)
}

fn ogg_duration_secs(data: &[u8]) -> Option<f64> {
    // Find last OggS page granule position (PCM sample position for Vorbis/Opus).
    let mut last_granule = None::<u64>;
    let mut sample_rate = None::<f64>;
    let mut i = 0usize;
    while i + 27 <= data.len() {
        if &data[i..i + 4] != b"OggS" {
            i += 1;
            continue;
        }
        let granule = u64::from_le_bytes(data.get(i + 6..i + 14)?.try_into().ok()?);
        let nseg = data[i + 26] as usize;
        let seg_table = data.get(i + 27..i + 27 + nseg)?;
        let body_len: usize = seg_table.iter().map(|&b| b as usize).sum();
        let body_at = i + 27 + nseg;
        let body = data.get(body_at..body_at + body_len)?;
        // OpusHead / Vorbis ident for sample rate.
        if sample_rate.is_none() {
            if body.starts_with(b"OpusHead") && body.len() >= 16 {
                sample_rate = Some(f64::from(read_u32_le(body, 12)?));
            } else if body.len() >= 16 && body[0] == 1 && body[1..7] == *b"vorbis" {
                sample_rate = Some(f64::from(read_u32_le(body, 12)?));
            }
        }
        if granule != u64::MAX && granule > 0 {
            last_granule = Some(granule);
        }
        i = body_at + body_len;
    }
    let rate = sample_rate.filter(|r| *r > 0.0)?;
    let secs = (last_granule? as f64) / rate;
    (secs.is_finite() && secs > 0.0).then_some(secs)
}

/// PDF 预览：同一套 `pusapreview` 协议直接回源文件，交给 WebView 内置的 PDF 渲染器。
pub fn pdf_preview_src(pdf_path: &Path) -> Result<String, String> {
    if !is_pdf_path(pdf_path) {
        return Err("不是 PDF 文件。".into());
    }
    let (token, file_name) = register_file(pdf_path)?;
    Ok(format!(
        "{}{}",
        asset_base(&token),
        encode_path_seg(&file_name)
    ))
}

pub fn is_stl_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("stl"))
}

/// STL 预览 iframe `src`：查看器页面 + query 指向同目录下的模型文件。
/// Windows 走 `stl_preview_srcdoc`，此路径仅非 Windows 桌面使用。
#[cfg(not(target_os = "windows"))]
pub fn stl_preview_src(stl_path: &Path) -> Result<String, String> {
    if !is_stl_path(stl_path) {
        return Err("不是 STL 文件。".into());
    }
    let (token, file_name) = register_file(stl_path)?;
    let base = asset_base(&token);
    let model_url = format!("{base}{}", encode_path_seg(&file_name));
    Ok(format!(
        "{base}{STL_VIEWER_FILE}?src={}&name={}",
        encode_query_value(&model_url),
        encode_query_value(&file_name)
    ))
}

/// Windows WebView2 下 iframe `src` 不能用 `http://pusapreview.localhost`（会弹系统浏览器），
/// 改为 `srcdoc` 注入查看器页面，并把模型地址写进 `window.__PUSA_STL__`；
/// 跨源 `fetch` 依赖协议响应里的 `Access-Control-Allow-Origin: *`。
#[cfg(target_os = "windows")]
pub fn stl_preview_srcdoc(stl_path: &Path) -> Result<String, String> {
    if !is_stl_path(stl_path) {
        return Err("不是 STL 文件。".into());
    }
    let (token, file_name) = register_file(stl_path)?;
    let model_url = format!("{}{}", asset_base(&token), encode_path_seg(&file_name));
    let cfg = serde_json::json!({ "src": model_url, "name": file_name });
    // `</` 转义，避免 JSON 字符串提前闭合 `<script>`。
    let cfg_js = cfg.to_string().replace("</", "<\\/");
    let tag = format!("<script>window.__PUSA_STL__={cfg_js};</script>");
    Ok(inject_head(STL_VIEWER_HTML, &tag))
}

pub fn is_docx_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("docx"))
}

pub fn is_xlsx_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("xlsx"))
}

pub fn is_pptx_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("pptx"))
}

pub fn is_doc_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("doc"))
}

pub fn is_xls_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("xls"))
}

pub fn is_ppt_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ppt"))
}

/// 旧版二进制 Office（OLE）。
pub fn is_legacy_office_path(path: &Path) -> bool {
    is_doc_path(path) || is_xls_path(path) || is_ppt_path(path)
}

/// 可预览的 Office：OOXML + 旧版 .doc/.xls/.ppt。
pub fn is_office_path(path: &Path) -> bool {
    is_docx_path(path)
        || is_xlsx_path(path)
        || is_pptx_path(path)
        || is_legacy_office_path(path)
}

/// 准备预览文件：旧版必要时先转 OOXML，再注册 `pusapreview` 站点。
/// 返回 `(实际预览文件, 工具栏显示名, 查看器格式扩展名)`。
fn office_preview_target(office_path: &Path) -> Result<(PathBuf, String, String), String> {
    if !is_office_path(office_path) {
        return Err(
            "不是可预览的 Office 文件（.doc / .docx / .xls / .xlsx / .ppt / .pptx）。".into(),
        );
    }
    let prepared = crate::desktop::office_convert::prepare_office_preview(office_path)?;
    let display_name = office_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .or_else(|| {
            prepared
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .ok_or_else(|| "文件名无效。".to_string())?;
    // 查看器按 `kind` 分支：转换成功后用 OOXML 扩展名，否则沿用原扩展名。
    let kind = prepared
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if kind.is_empty() {
        return Err("无法识别 Office 文件扩展名。".into());
    }
    Ok((prepared, display_name, kind))
}

fn normalize_ui_theme(theme: &str) -> &'static str {
    if theme.trim().eq_ignore_ascii_case("dark") {
        "dark"
    } else {
        "light"
    }
}

/// Office 预览 iframe `src`：查看器页面 + query 指向同目录下的源文件。
/// `theme` 为应用 `data-theme`（`light` / `dark`），写入 query 供首屏渲染。
/// Windows 走 `office_preview_srcdoc`，此路径仅非 Windows 桌面使用。
#[cfg(not(target_os = "windows"))]
pub fn office_preview_src(office_path: &Path, theme: &str) -> Result<String, String> {
    let (target, display_name, kind) = office_preview_target(office_path)?;
    let (token, file_name) = register_file(&target)?;
    let base = asset_base(&token);
    let file_url = format!("{base}{}", encode_path_seg(&file_name));
    let theme = normalize_ui_theme(theme);
    Ok(format!(
        "{base}{OFFICE_VIEWER_FILE}?src={}&name={}&kind={}&theme={}",
        encode_query_value(&file_url),
        encode_query_value(&display_name),
        encode_query_value(&kind),
        encode_query_value(theme)
    ))
}

/// Windows：`srcdoc` + `<base>` 加载同源虚拟 JS，配置写入 `window.__PUSA_OFFICE__`。
#[cfg(target_os = "windows")]
pub fn office_preview_srcdoc(office_path: &Path, theme: &str) -> Result<String, String> {
    let (target, display_name, kind) = office_preview_target(office_path)?;
    let (token, file_name) = register_file(&target)?;
    let base = asset_base(&token);
    let file_url = format!("{base}{}", encode_path_seg(&file_name));
    let theme = normalize_ui_theme(theme);
    let cfg = serde_json::json!({
        "src": file_url,
        "name": display_name,
        "kind": kind,
        "theme": theme
    });
    let cfg_js = cfg.to_string().replace("</", "<\\/");
    let tag = format!(
        r#"<base href="{base}"><script>window.__PUSA_OFFICE__={cfg_js};</script>"#
    );
    Ok(inject_head(OFFICE_VIEWER_HTML, &tag))
}

/// 文件：自身；目录：`index.html` / `index.htm`，否则第一个 `.html`。
pub fn resolve_html_preview(path: &Path, is_dir: bool) -> Option<PathBuf> {
    if is_dir || path.is_dir() {
        let index_html = path.join("index.html");
        if index_html.is_file() {
            return Some(index_html);
        }
        let index_htm = path.join("index.htm");
        if index_htm.is_file() {
            return Some(index_htm);
        }
        let rd = fs::read_dir(path).ok()?;
        let mut htmls: Vec<PathBuf> = rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && is_html_path(p))
            .collect();
        htmls.sort();
        return htmls.into_iter().next();
    }
    if is_html_path(path) && path.is_file() {
        return Some(path.to_path_buf());
    }
    None
}

/// Windows 走 `preview_srcdoc`，此路径仅非 Windows 桌面使用。
#[cfg(not(target_os = "windows"))]
pub fn preview_iframe_src(html_path: &Path) -> Result<String, String> {
    let (token, file_name) = register_file(html_path)?;
    Ok(format!(
        "{PROTOCOL_NAME}://localhost/{token}/{}",
        encode_path_seg(&file_name)
    ))
}

/// Windows WebView2 把自定义协议写成 `http://`，不能当 iframe `src`（会弹系统浏览器）。
#[cfg(target_os = "windows")]
pub fn preview_srcdoc(html_path: &Path) -> Result<String, String> {
    let (token, _) = register_file(html_path)?;
    let raw = fs::read_to_string(html_path).map_err(|e| format!("无法读取 HTML：{e}"))?;
    let tag = format!(r#"<base href="{}">"#, asset_base(&token));
    Ok(inject_head(&raw, &tag))
}

/// 把文件所在目录注册为预览站点根，返回 (token, 文件名)。
fn register_file(file_path: &Path) -> Result<(String, String), String> {
    let file = file_path
        .canonicalize()
        .map_err(|e| format!("无法解析文件路径：{e}"))?;
    if !file.is_file() {
        return Err("预览目标不是文件。".into());
    }
    let dir = file
        .parent()
        .ok_or_else(|| "文件没有父目录。".to_string())?
        .canonicalize()
        .map_err(|e| format!("无法解析站点目录：{e}"))?;
    let file_name = file
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "文件名无效。".to_string())?
        .to_string();
    Ok((register_root(dir), file_name))
}

fn encode_path_seg(name: &str) -> String {
    let mut out = String::new();
    for b in name.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// query 值编码：与 `encode_path_seg` 同一保留集（`URLSearchParams` 可原样解出 `:` `/` `%` 等）。
#[cfg(not(target_os = "windows"))]
fn encode_query_value(value: &str) -> String {
    encode_path_seg(value)
}

pub fn serve_request(uri: &str) -> Result<(&'static str, Vec<u8>), u16> {
    let (token, rel) = parse_preview_uri(uri).ok_or(404u16)?;
    let root = {
        let map = roots().lock().map_err(|_| 500u16)?;
        map.get(&token).cloned().ok_or(404u16)?
    };
    if rel == STL_VIEWER_FILE {
        return Ok((
            "text/html; charset=utf-8",
            STL_VIEWER_HTML.as_bytes().to_vec(),
        ));
    }
    if rel == OFFICE_VIEWER_FILE {
        return Ok((
            "text/html; charset=utf-8",
            OFFICE_VIEWER_HTML.as_bytes().to_vec(),
        ));
    }
    if rel == OFFICE_MAMMOTH_FILE {
        return Ok(("text/javascript; charset=utf-8", OFFICE_MAMMOTH_JS.to_vec()));
    }
    if rel == OFFICE_XLSX_FILE {
        return Ok(("text/javascript; charset=utf-8", OFFICE_XLSX_JS.to_vec()));
    }
    if rel == OFFICE_JSZIP_FILE {
        return Ok(("text/javascript; charset=utf-8", OFFICE_JSZIP_JS.to_vec()));
    }
    let candidate = if rel.is_empty() {
        root.join("index.html")
    } else {
        root.join(rel)
    };
    let canon = candidate.canonicalize().map_err(|_| 404u16)?;
    if !canon.starts_with(&root) || !canon.is_file() {
        return Err(404);
    }
    let body = fs::read(&canon).map_err(|_| 404u16)?;
    Ok((mime_of(&canon), body))
}

fn register_root(dir: PathBuf) -> String {
    let id = {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        dir.hash(&mut h);
        format!("{:x}", h.finish())
    };
    if let Ok(mut map) = roots().lock() {
        map.insert(id.clone(), dir);
    }
    id
}

/// 预览站点根 URL（带尾斜杠）。Windows WebView2 的自定义协议以 `http://<scheme>.localhost/` 暴露。
fn asset_base(token: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("http://{PROTOCOL_NAME}.localhost/{token}/")
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("{PROTOCOL_NAME}://localhost/{token}/")
    }
}

fn parse_preview_uri(uri: &str) -> Option<(String, String)> {
    let prefixes = [
        format!("{PROTOCOL_NAME}://localhost/"),
        format!("{PROTOCOL_NAME}:///"),
        format!("http://{PROTOCOL_NAME}.localhost/"),
        format!("https://{PROTOCOL_NAME}.localhost/"),
    ];
    let mut rest = None;
    for p in &prefixes {
        if let Some(r) = uri.strip_prefix(p.as_str()) {
            rest = Some(r);
            break;
        }
    }
    let rest = rest.or_else(|| {
        uri.split("://").nth(1).and_then(|s| {
            s.strip_prefix("localhost/")
                .or_else(|| s.strip_prefix('/'))
                .or(Some(s))
        })
    })?;
    let rest = rest.split(['?', '#'].as_slice()).next().unwrap_or(rest);
    let rest = rest.trim_start_matches('/');
    let (token, rel) = rest.split_once('/').unwrap_or((rest, ""));
    if token.is_empty() {
        return None;
    }
    Some((token.to_string(), percent_decode(rel)))
}

/// 把 `tag` 插到 `<head>` 开标签之后；没有 `<head>` 时补一个。
#[cfg(target_os = "windows")]
fn inject_head(html: &str, tag: &str) -> String {
    let lower = html.to_ascii_lowercase();
    if let Some(i) = lower.find("<head") {
        if let Some(gt) = html[i..].find('>') {
            let at = i + gt + 1;
            let mut out = String::with_capacity(html.len() + tag.len());
            out.push_str(&html[..at]);
            out.push_str(&tag);
            out.push_str(&html[at..]);
            return out;
        }
    }
    format!("<!DOCTYPE html><head>{tag}</head>{html}")
}

fn mime_of(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "pdf" => "application/pdf",
        "stl" => "model/stl",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "doc" => "application/msword",
        "xls" => "application/vnd.ms-excel",
        "ppt" => "application/vnd.ms-powerpoint",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "avif" => "image/avif",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" | "oga" => "audio/ogg",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "opus" => "audio/opus",
        "webm" => "audio/webm",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod audio_duration_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn probes_pcm_wav_duration() {
        // 1s mono 16-bit 44100Hz PCM
        let sr = 44100u32;
        let n = sr as usize;
        let mut data = Vec::with_capacity(n * 2);
        data.resize(n * 2, 0);
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&1u16.to_le_bytes());
        fmt.extend_from_slice(&1u16.to_le_bytes());
        fmt.extend_from_slice(&sr.to_le_bytes());
        fmt.extend_from_slice(&(sr * 2).to_le_bytes());
        fmt.extend_from_slice(&2u16.to_le_bytes());
        fmt.extend_from_slice(&16u16.to_le_bytes());
        let mut chunks = Vec::new();
        chunks.extend_from_slice(b"fmt ");
        chunks.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
        chunks.extend_from_slice(&fmt);
        chunks.extend_from_slice(b"data");
        chunks.extend_from_slice(&(data.len() as u32).to_le_bytes());
        chunks.extend_from_slice(&data);
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(4 + chunks.len() as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(&chunks);

        let dir = std::env::temp_dir().join("pusa_audio_dur_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("one_sec.wav");
        fs::File::create(&path)
            .unwrap()
            .write_all(&wav)
            .unwrap();
        let secs = audio_duration_secs(&path).expect("wav duration");
        assert!((secs - 1.0).abs() < 0.01, "got {secs}");
    }
}
