//! 旧版 Office（`.doc` / `.xls` / `.ppt`）预览准备：能转则转成 OOXML，再复用现有 JS 查看器。
//!
//! - `.xls`：不转换，交给 SheetJS 直接读 BIFF。
//! - `.doc`：优先 macOS `textutil`，否则 `soffice` → `.docx`。
//! - `.ppt`：`soffice` → `.pptx`。
//! 转换失败时仍返回原路径，由前端做 OLE 文本兜底；错误信息在转换器缺失时写进日志语义由调用方展示。

use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::html_preview::{is_doc_path, is_ppt_path, is_xls_path};

/// 将预览路径解析为可交给现有查看器的文件（现代 OOXML、`.xls` 原样，或转换产物）。
pub fn prepare_office_preview(path: &Path) -> Result<PathBuf, String> {
    let path = path
        .canonicalize()
        .map_err(|e| format!("无法解析文件路径：{e}"))?;
    if !path.is_file() {
        return Err("预览目标不是文件。".into());
    }

    if is_xls_path(&path) {
        return Ok(path);
    }
    if is_doc_path(&path) {
        return convert_or_passthrough(&path, "docx", ConvertKind::Doc);
    }
    if is_ppt_path(&path) {
        return convert_or_passthrough(&path, "pptx", ConvertKind::Ppt);
    }
    Ok(path)
}

#[derive(Clone, Copy)]
enum ConvertKind {
    Doc,
    Ppt,
}

fn convert_or_passthrough(src: &Path, out_ext: &str, kind: ConvertKind) -> Result<PathBuf, String> {
    match try_convert(src, out_ext, kind) {
        Ok(converted) => Ok(converted),
        Err(err) => {
            // 转换失败：仍返回原文件，前端 OLE 文本兜底；把原因记到 stderr 便于排查。
            eprintln!("[pusa office] 旧版转换失败，回退原文件预览：{err}");
            Ok(src.to_path_buf())
        }
    }
}

fn try_convert(src: &Path, out_ext: &str, kind: ConvertKind) -> Result<PathBuf, String> {
    let out = cached_output_path(src, out_ext)?;
    if out.is_file() {
        return Ok(out);
    }
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("无法创建转换缓存目录：{e}"))?;
    }

    let mut errors: Vec<String> = Vec::new();

    match kind {
        ConvertKind::Doc => {
            #[cfg(target_os = "macos")]
            {
                if let Err(e) = convert_doc_textutil(src, &out) {
                    errors.push(e);
                } else if out.is_file() {
                    return Ok(out);
                }
            }
            match convert_via_soffice(src, out.parent().unwrap_or(Path::new(".")), out_ext) {
                Ok(p) => {
                    if p != out {
                        let _ = fs::rename(&p, &out).or_else(|_| fs::copy(&p, &out).map(|_| ()));
                    }
                    if out.is_file() {
                        return Ok(out);
                    }
                    if p.is_file() {
                        return Ok(p);
                    }
                    errors.push("LibreOffice 转换后未找到输出文件。".into());
                }
                Err(e) => errors.push(e),
            }
        }
        ConvertKind::Ppt => {
            match convert_via_soffice(src, out.parent().unwrap_or(Path::new(".")), out_ext) {
                Ok(p) => {
                    if p != out {
                        let _ = fs::rename(&p, &out).or_else(|_| fs::copy(&p, &out).map(|_| ()));
                    }
                    if out.is_file() {
                        return Ok(out);
                    }
                    if p.is_file() {
                        return Ok(p);
                    }
                    errors.push("LibreOffice 转换后未找到输出文件。".into());
                }
                Err(e) => errors.push(e),
            }
        }
    }

    Err(format_convert_error(kind, &errors))
}

fn format_convert_error(kind: ConvertKind, errors: &[String]) -> String {
    let detail = if errors.is_empty() {
        "未知错误".to_string()
    } else {
        errors.join("；")
    };
    match kind {
        ConvertKind::Doc => format!(
            "无法将 .doc 转为 .docx（{detail}）。macOS 可依赖系统 textutil；其它平台请安装 LibreOffice（命令行 soffice）。"
        ),
        ConvertKind::Ppt => format!(
            "无法将 .ppt 转为 .pptx（{detail}）。请安装 LibreOffice 并确保 `soffice` 可用：macOS 可用 `brew install --cask libreoffice`，Windows/Linux 安装后将程序目录加入 PATH。"
        ),
    }
}

#[cfg(target_os = "macos")]
fn convert_doc_textutil(src: &Path, dest: &Path) -> Result<(), String> {
    let output = Command::new("textutil")
        .args(["-convert", "docx", "-output"])
        .arg(dest)
        .arg(src)
        .output()
        .map_err(|e| format!("无法启动 textutil：{e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "textutil 转换失败：{}",
            stderr.trim().if_empty("无详细输出")
        ));
    }
    if !dest.is_file() {
        return Err("textutil 未写出 docx。".into());
    }
    Ok(())
}

fn convert_via_soffice(src: &Path, outdir: &Path, out_ext: &str) -> Result<PathBuf, String> {
    let soffice = find_soffice().ok_or_else(|| {
        "未找到可用的 LibreOffice（soffice）。安装后即可预览旧版 .doc / .ppt。".to_string()
    })?;

    fs::create_dir_all(outdir).map_err(|e| format!("无法创建输出目录：{e}"))?;

    // 独立 UserInstallation，避免与正在运行的 LibreOffice GUI 抢配置锁。
    let profile = std::env::temp_dir().join("pusa-libreoffice-profile");
    let _ = fs::create_dir_all(&profile);
    let profile_uri = path_to_file_uri(&profile);

    let output = Command::new(&soffice)
        .args([
            "--headless",
            "--norestore",
            "--nolockcheck",
            "--nodefault",
            "--nofirststartwizard",
        ])
        .arg(format!("-env:UserInstallation={profile_uri}"))
        .arg("--convert-to")
        .arg(out_ext)
        .arg("--outdir")
        .arg(outdir)
        .arg(src)
        .output()
        .map_err(|e| format!("无法启动 soffice（{}）：{e}", soffice.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = [stderr.trim(), stdout.trim()]
            .into_iter()
            .find(|s| !s.is_empty())
            .unwrap_or("退出码非零");
        return Err(format!("soffice 转换失败：{msg}"));
    }

    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "源文件名无效。".to_string())?;
    let candidate = outdir.join(format!("{stem}.{out_ext}"));
    if candidate.is_file() {
        return Ok(candidate);
    }

    // LibreOffice 有时会改名；在 outdir 里找最新的同扩展名文件。
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    if let Ok(rd) = fs::read_dir(outdir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case(out_ext))
            {
                let modified = ent
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(UNIX_EPOCH);
                match &newest {
                    Some((t, _)) if modified <= *t => {}
                    _ => newest = Some((modified, p)),
                }
            }
        }
    }
    newest
        .map(|(_, p)| p)
        .ok_or_else(|| "soffice 未生成目标文件。".into())
}

fn cached_output_path(src: &Path, out_ext: &str) -> Result<PathBuf, String> {
    let meta = fs::metadata(src).map_err(|e| format!("无法读取文件元数据：{e}"))?;
    let modified = meta
        .modified()
        .unwrap_or(UNIX_EPOCH)
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    let size = meta.len();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    src.hash(&mut hasher);
    let id = format!("{:x}_{modified}_{size}", hasher.finish());
    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    // 安全文件名：去掉路径分隔符。
    let safe_stem: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    Ok(std::env::temp_dir()
        .join("pusa-office-preview")
        .join(id)
        .join(format!("{safe_stem}.{out_ext}")))
}

fn find_soffice() -> Option<PathBuf> {
    static CACHED: Mutex<Option<Option<PathBuf>>> = Mutex::new(None);
    if let Ok(guard) = CACHED.lock() {
        if let Some(ref cached) = *guard {
            return cached.clone();
        }
    }

    let found = find_soffice_uncached();
    if let Ok(mut guard) = CACHED.lock() {
        *guard = Some(found.clone());
    }
    found
}

fn find_soffice_uncached() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    for name in ["soffice", "libreoffice"] {
        if let Some(p) = which(name) {
            candidates.push(p);
        }
    }

    #[cfg(target_os = "macos")]
    {
        candidates.push(PathBuf::from(
            "/Applications/LibreOffice.app/Contents/MacOS/soffice",
        ));
        // Homebrew Cask 未 link 到 /Applications 时仍可能在 Caskroom。
        for base in [
            "/opt/homebrew/Caskroom/libreoffice",
            "/usr/local/Caskroom/libreoffice",
        ] {
            if let Ok(rd) = fs::read_dir(base) {
                for ent in rd.flatten() {
                    let p = ent.path().join("LibreOffice.app/Contents/MacOS/soffice");
                    candidates.push(p);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let pf = std::env::var_os("ProgramFiles").map(PathBuf::from);
        let pf86 = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from);
        for root in [pf, pf86].into_iter().flatten() {
            candidates.push(root.join("LibreOffice/program/soffice.exe"));
            candidates.push(root.join("LibreOffice/program/soffice.com"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        candidates.push(PathBuf::from("/usr/bin/soffice"));
        candidates.push(PathBuf::from("/usr/bin/libreoffice"));
        candidates.push(PathBuf::from("/usr/lib/libreoffice/program/soffice"));
        candidates.push(PathBuf::from("/snap/bin/libreoffice"));
    }

    candidates.into_iter().find(|p| soffice_usable(p))
}

fn soffice_usable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    // Homebrew 包装脚本若指向缺失的 .app，会立刻失败。
    Command::new(path)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(target_os = "windows")]
        {
            let exe = dir.join(format!("{name}.exe"));
            if exe.is_file() {
                return Some(exe);
            }
            let com = dir.join(format!("{name}.com"));
            if com.is_file() {
                return Some(com);
            }
        }
    }
    None
}

fn path_to_file_uri(path: &Path) -> String {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let s = abs.to_string_lossy();
    #[cfg(target_os = "windows")]
    {
        let normalized = s.replace('\\', "/");
        if normalized.starts_with("//") {
            format!("file:{normalized}")
        } else {
            format!("file:///{normalized}")
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("file://{s}")
    }
}

trait IfEmpty {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str;
}

impl IfEmpty for str {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.is_empty() {
            fallback
        } else {
            self
        }
    }
}
