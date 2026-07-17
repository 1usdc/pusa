//! 聊天输入区待发送附件（回形针选择与剪贴板粘贴共用）。

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use dioxus::html::FileData;

/// 输入区中的一条待发送附件。
#[derive(Clone, PartialEq)]
pub struct ChatPendingAttachment {
    pub id: u64,
    pub name: String,
    pub mime: String,
    /// 图片预览用 data URL；非图片为 `None`。
    pub preview_url: Option<String>,
    /// 发送给视觉模型的 data URL（仅图片）。
    pub image_data_url: Option<String>,
}

fn guess_mime(name: &str, content_type: Option<&str>) -> String {
    if let Some(ct) = content_type.map(str::trim).filter(|s| !s.is_empty()) {
        return ct.to_string();
    }
    let ext = name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png".into(),
        "jpg" | "jpeg" => "image/jpeg".into(),
        "gif" => "image/gif".into(),
        "webp" => "image/webp".into(),
        "bmp" => "image/bmp".into(),
        "svg" => "image/svg+xml".into(),
        _ => "application/octet-stream".into(),
    }
}

fn bytes_to_data_url(mime: &str, bytes: &[u8]) -> String {
    format!("data:{mime};base64,{}", B64.encode(bytes))
}

/// 由原始字节构建附件（粘贴 / 读文件后共用）。
pub fn attachment_from_bytes(id: u64, name: String, mime: String, bytes: Vec<u8>) -> ChatPendingAttachment {
    let mime = guess_mime(&name, Some(mime.as_str()));
    let is_image = mime.starts_with("image/");
    let data_url = if is_image && !bytes.is_empty() {
        Some(bytes_to_data_url(&mime, &bytes))
    } else {
        None
    };
    ChatPendingAttachment {
        id,
        name,
        mime,
        preview_url: data_url.clone(),
        image_data_url: data_url,
    }
}

/// 从 Dioxus [`FileData`]（回形针 / 部分平台粘贴）异步读入附件。
pub async fn attachment_from_file_data(
    id: u64,
    file: FileData,
) -> Option<ChatPendingAttachment> {
    let name = file.name();
    let mime = guess_mime(&name, file.content_type().as_deref());
    let bytes = file.read_bytes().await.ok()?.to_vec();
    Some(attachment_from_bytes(id, name, mime, bytes))
}

/// 粘贴文件名兜底（剪贴板里经常是空名或 `image.png`）。
#[cfg(target_arch = "wasm32")]
pub fn pasted_image_name(mime: &str, index: usize) -> String {
    let ext = mime
        .strip_prefix("image/")
        .unwrap_or("png")
        .split(';')
        .next()
        .unwrap_or("png");
    let ext = if ext == "jpeg" { "jpg" } else { ext };
    if index <= 1 {
        format!("paste.{ext}")
    } else {
        format!("paste-{index}.{ext}")
    }
}

/// Desktop（wry WebView）：dioxus 序列化的 `ClipboardEvent` 不含文件，需在 JS 侧
/// capture `clipboardData`，再经 `dioxus.send` 把 base64 送回 Rust。
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop_paste {
    use super::*;
    use serde::Deserialize;

    /// 挂载于 `document` capture 的粘贴钩子；保持 eval channel 常开。
    pub const INSTALL_HOOK_JS: &str = r#"(async () => {
  const readFileB64 = (file) => new Promise((resolve, reject) => {
    const r = new FileReader();
    r.onload = () => {
      const s = String(r.result || '');
      const i = s.indexOf(',');
      resolve(i >= 0 ? s.slice(i + 1) : s);
    };
    r.onerror = () => reject(r.error);
    r.readAsDataURL(file);
  });

  const pasteName = (mime, index) => {
    let ext = (mime || 'image/png').replace(/^image\//, '').split(';')[0] || 'png';
    if (ext === 'jpeg') ext = 'jpg';
    return index <= 1 ? ('paste.' + ext) : ('paste-' + index + '.' + ext);
  };

  const handler = (e) => {
    const t = e.target;
    if (!t || !t.classList || !t.classList.contains('ac-chat-input-field')) return;

    const dt = e.clipboardData;
    if (!dt) return;

    const files = [];
    if (dt.items) {
      for (let i = 0; i < dt.items.length; i++) {
        const item = dt.items[i];
        if (item.kind === 'file' && item.type && item.type.indexOf('image/') === 0) {
          const f = item.getAsFile();
          if (f) files.push(f);
        }
      }
    }
    if (files.length === 0 && dt.files) {
      for (let i = 0; i < dt.files.length; i++) {
        const f = dt.files[i];
        if (f && f.type && f.type.indexOf('image/') === 0) files.push(f);
      }
    }
    if (files.length === 0) return;

    // 有图片时拦截默认粘贴，避免二进制进 textarea；纯文本仍走浏览器默认。
    e.preventDefault();
    e.stopPropagation();

    const text = (dt.getData('text/plain') || '').trim();

    (async () => {
      const images = [];
      for (let i = 0; i < files.length; i++) {
        const f = files[i];
        const mime = f.type || 'image/png';
        let name = f.name || '';
        if (!name.trim() || name === 'image.png') name = pasteName(mime, i + 1);
        try {
          const b64 = await readFileB64(f);
          if (b64) images.push({ name: name, mime: mime, b64: b64 });
        } catch (_) {}
      }
      if (images.length === 0 && !text) return;
      dioxus.send({ images: images, text: text || null });
    })();
  };

  if (window.__acChatPasteHandler) {
    document.removeEventListener('paste', window.__acChatPasteHandler, true);
  }
  window.__acChatPasteHandler = handler;
  document.addEventListener('paste', handler, true);

  await new Promise(() => {});
})()"#;

    #[derive(Debug, Clone, Deserialize)]
    pub struct PastePayload {
        pub images: Vec<PasteImage>,
        pub text: Option<String>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct PasteImage {
        pub name: String,
        pub mime: String,
        pub b64: String,
    }

    pub fn attachment_from_paste_image(
        id: u64,
        img: PasteImage,
    ) -> Option<ChatPendingAttachment> {
        let bytes = B64.decode(img.b64.trim()).ok()?;
        if bytes.is_empty() {
            return None;
        }
        Some(attachment_from_bytes(id, img.name, img.mime, bytes))
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm_paste {
    use super::*;
    use wasm_bindgen::JsCast;
    use web_sys::{ClipboardEvent, DataTransfer};

    /// 从剪贴板取出图片文件；同时返回纯文本（若有）。
    ///
    /// 优先 `items` + `getAsFile`（截图常用），再回退 `files`。
    pub fn extract_clipboard_images(
        data_transfer: &DataTransfer,
    ) -> (Vec<(String, String, web_sys::File)>, Option<String>) {
        let mut images = Vec::new();
        let text = {
            let t = data_transfer.get_data("text/plain").unwrap_or_default();
            let t = t.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        };

        let items = data_transfer.items();
        let len = items.length();
        for i in 0..len {
            let Some(item) = items.get(i) else {
                continue;
            };
            if item.kind() != "file" {
                continue;
            }
            let mime = item.type_();
            if !mime.starts_with("image/") {
                continue;
            }
            let Ok(Some(file)) = item.get_as_file() else {
                continue;
            };
            let name = {
                let n = file.name();
                if n.trim().is_empty() || n == "image.png" {
                    pasted_image_name(&mime, images.len() + 1)
                } else {
                    n
                }
            };
            images.push((name, mime, file));
        }

        if images.is_empty() {
            if let Some(files) = data_transfer.files() {
                let len = files.length();
                for i in 0..len {
                    let Some(file) = files.item(i) else {
                        continue;
                    };
                    let mime = file.type_();
                    if !mime.starts_with("image/") {
                        continue;
                    }
                    let name = {
                        let n = file.name();
                        if n.trim().is_empty() {
                            pasted_image_name(&mime, images.len() + 1)
                        } else {
                            n
                        }
                    };
                    images.push((name, mime, file));
                }
            }
        }

        (images, text)
    }

    pub fn clipboard_data_from_event(e: &dioxus::prelude::ClipboardEvent) -> Option<DataTransfer> {
        let data = e.data();
        let web_ev = data.downcast::<web_sys::Event>()?;
        let clip: ClipboardEvent = web_ev.clone().dyn_into().ok()?;
        clip.clipboard_data()
    }

    pub async fn read_web_file_bytes(file: &web_sys::File) -> Option<Vec<u8>> {
        let buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
            .await
            .ok()?;
        let arr = js_sys::Uint8Array::new(&buffer);
        Some(arr.to_vec())
    }
}
