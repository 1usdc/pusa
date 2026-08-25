//! 聊天输入区待发送附件（回形针选择、剪贴板粘贴、文件树加入共用）。

use std::path::Path;

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use dioxus::html::FileData;

/// 输入区中的一条待发送附件。
#[derive(Clone, Debug)]
pub struct ChatPendingAttachment {
    pub id: u64,
    pub name: String,
    pub mime: String,
    /// 图片预览用 data URL；非图片为 `None`。
    pub preview_url: Option<String>,
    /// 发送给视觉模型的 data URL（仅图片）。
    pub image_data_url: Option<String>,
    /// 文件树 / 工作区引用的绝对或相对路径；回形针二进制附件为 `None`。
    pub source_path: Option<String>,
    /// 路径引用是否为目录（chip 配色 / 点击展开侧栏）；二进制附件为 `false`。
    pub is_dir: bool,
}

impl PartialEq for ChatPendingAttachment {
    fn eq(&self, other: &Self) -> bool {
        // 不比较 preview / image data URL 正文：base64 可达数 MB，组件 props diff 会卡顿。
        self.id == other.id
            && self.name == other.name
            && self.mime == other.mime
            && self.source_path == other.source_path
            && self.is_dir == other.is_dir
            && self.preview_url.is_some() == other.preview_url.is_some()
            && self.image_data_url.is_some() == other.image_data_url.is_some()
    }
}

impl ChatPendingAttachment {
    /// 发送给模型时的标签：优先完整路径，否则文件名。
    pub fn send_label(&self) -> &str {
        self.source_path
            .as_deref()
            .filter(|p| !p.trim().is_empty())
            .unwrap_or(self.name.as_str())
    }
}

const ATTACHMENT_MARK: &str = "[附件:";

/// 拆出发送给模型的 `[附件: …]` 后缀：可见正文 + 路径/文件名列表。
pub fn split_attachment_suffix(content: &str) -> (String, Vec<String>) {
    let trimmed = content.trim_end();
    let Some(start) = trimmed.rfind(ATTACHMENT_MARK) else {
        return (content.to_string(), Vec::new());
    };
    let after = trimmed[start + ATTACHMENT_MARK.len()..].trim();
    let Some(inner) = after.strip_suffix(']') else {
        return (content.to_string(), Vec::new());
    };
    let prefix = &trimmed[..start];
    if !prefix.is_empty() && !prefix.ends_with('\n') {
        return (content.to_string(), Vec::new());
    }
    let labels: Vec<String> = inner
        .split(", ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if labels.is_empty() {
        return (content.to_string(), Vec::new());
    }
    (prefix.trim_end().to_string(), labels)
}

/// 气泡 / 编辑器展示用：去掉末尾 `[附件: …]`。
pub fn strip_attachment_suffix(content: &str) -> String {
    split_attachment_suffix(content).0
}

/// 用户气泡片段：与输入框 chip/文字交错顺序一致。
#[derive(Clone, Debug, PartialEq)]
pub enum ChatUserSeg {
    Text(String),
    Attachment(ChatPendingAttachment),
}

fn attachment_matches_label(att: &ChatPendingAttachment, label: &str) -> bool {
    att.send_label() == label
        || att.name == label
        || att.source_path.as_deref() == Some(label)
}

/// 从正文里的 `[附件: …]` 标记还原与输入顺序一致的展示片段。
pub fn parse_user_message_segments(
    content: &str,
    attachments: &[ChatPendingAttachment],
) -> Vec<ChatUserSeg> {
    let mut segs = Vec::new();
    let mut unused: Vec<ChatPendingAttachment> = attachments.to_vec();
    let mut rest = content;

    while let Some(start) = rest.find(ATTACHMENT_MARK) {
        let prefix = &rest[..start];
        if !prefix.is_empty() {
            segs.push(ChatUserSeg::Text(prefix.to_string()));
        }
        let after_mark = &rest[start + ATTACHMENT_MARK.len()..];
        let Some(end) = after_mark.find(']') else {
            segs.push(ChatUserSeg::Text(rest[start..].to_string()));
            rest = "";
            break;
        };
        let inner = after_mark[..end].trim();
        let labels: Vec<&str> = if inner.contains(", ") {
            inner
                .split(", ")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect()
        } else if inner.is_empty() {
            Vec::new()
        } else {
            vec![inner]
        };
        for label in labels {
            if let Some(idx) = unused.iter().position(|a| attachment_matches_label(a, label)) {
                segs.push(ChatUserSeg::Attachment(unused.remove(idx)));
            } else {
                segs.push(ChatUserSeg::Attachment(attachment_from_path(0, label)));
            }
        }
        rest = &after_mark[end + 1..];
    }
    if !rest.is_empty() {
        segs.push(ChatUserSeg::Text(rest.to_string()));
    }
    for att in unused {
        segs.push(ChatUserSeg::Attachment(att));
    }
    segs.retain(|seg| match seg {
        ChatUserSeg::Text(t) => !t.is_empty(),
        ChatUserSeg::Attachment(_) => true,
    });
    segs
}

/// 从历史正文里的附件标记还原路径 chip。
pub fn attachments_from_suffix_labels(labels: &[String], start_id: u64) -> Vec<ChatPendingAttachment> {
    labels
        .iter()
        .enumerate()
        .map(|(i, label)| attachment_from_path(start_id + i as u64, label.clone()))
        .collect()
}

fn path_is_directory(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        Path::new(trimmed).is_dir()
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        false
    }
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
        source_path: None,
        is_dir: false,
    }
}

/// 文件树「加入 Chat」：路径引用 chip，不读入文件字节、不写入草稿文本。
pub fn attachment_from_path(id: u64, path: impl Into<String>) -> ChatPendingAttachment {
    let path = path.into();
    let is_dir = path_is_directory(&path);
    let name = Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path.as_str())
        .to_string();
    let mime = if is_dir {
        "inode/directory".into()
    } else {
        guess_mime(&name, None)
    };
    ChatPendingAttachment {
        id,
        name,
        mime,
        preview_url: None,
        image_data_url: None,
        source_path: Some(path),
        is_dir,
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
    if (!t || !t.closest || !t.closest('[data-ac-composer], .ac-chat-composer-ce')) return;

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

#[cfg(test)]
mod tests {
    use super::{attachment_from_path, parse_user_message_segments, split_attachment_suffix, ChatUserSeg};

    #[test]
    fn strips_trailing_attachment_block() {
        let (visible, labels) = split_attachment_suffix(
            "现在要怎么启动\n\n[附件: /Volumes/SSD/codes/English/app]",
        );
        assert_eq!(visible, "现在要怎么启动");
        assert_eq!(labels, vec!["/Volumes/SSD/codes/English/app".to_string()]);
    }

    #[test]
    fn strips_attachment_only_message() {
        let (visible, labels) = split_attachment_suffix("[附件: /tmp/a, /tmp/b]");
        assert!(visible.is_empty());
        assert_eq!(labels, vec!["/tmp/a".to_string(), "/tmp/b".to_string()]);
    }

    #[test]
    fn keeps_inline_attachment_looking_text() {
        let src = "请看 [附件: 说明] 这一段";
        let (visible, labels) = split_attachment_suffix(src);
        assert_eq!(visible, src);
        assert!(labels.is_empty());
    }

    #[test]
    fn parses_chips_in_composer_order() {
        let atts = vec![
            attachment_from_path(1, "/x/desktop"),
            attachment_from_path(2, "/x/app"),
        ];
        let segs = parse_user_message_segments(
            "[附件: /x/desktop][附件: /x/app]需要参考的内容",
            &atts,
        );
        assert_eq!(segs.len(), 3);
        match &segs[0] {
            ChatUserSeg::Attachment(a) => assert_eq!(a.name, "desktop"),
            _ => panic!("expected first chip"),
        }
        match &segs[1] {
            ChatUserSeg::Attachment(a) => assert_eq!(a.name, "app"),
            _ => panic!("expected second chip"),
        }
        match &segs[2] {
            ChatUserSeg::Text(t) => assert_eq!(t, "需要参考的内容"),
            _ => panic!("expected trailing text"),
        }
    }

    #[test]
    fn parses_text_between_chips() {
        let segs = parse_user_message_segments(
            "[附件: /x/app]参考[附件: /x/desktop]一下",
            &[],
        );
        match &segs[..] {
            [ChatUserSeg::Attachment(a), ChatUserSeg::Text(t1), ChatUserSeg::Attachment(b), ChatUserSeg::Text(t2)] => {
                assert_eq!(a.name, "app");
                assert_eq!(t1, "参考");
                assert_eq!(b.name, "desktop");
                assert_eq!(t2, "一下");
            }
            other => panic!("unexpected segs: {other:?}"),
        }
    }
}
