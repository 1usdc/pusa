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
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
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
