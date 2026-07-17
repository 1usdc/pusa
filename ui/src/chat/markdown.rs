//! 聊天气泡 Markdown 渲染：普通段落为消毒 HTML；围栏代码块可交互（复制）。

use std::path::Path;

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdCopy;
use dioxus_free_icons::Icon;
use pulldown_cmark::html;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

#[derive(Clone, Debug, PartialEq, Eq)]
enum ChatMdPart {
    Html(String),
    Code {
        lang: String,
        path_hint: Option<String>,
        code: String,
    },
}

fn markdown_options() -> Options {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts
}

fn flush_html(buf: &mut String, out: &mut Vec<ChatMdPart>) {
    if buf.is_empty() {
        return;
    }
    let cleaned = ammonia::clean(buf);
    buf.clear();
    if !cleaned.is_empty() {
        out.push(ChatMdPart::Html(cleaned));
    }
}

fn is_known_lang(token: &str) -> bool {
    matches!(
        token.to_ascii_lowercase().as_str(),
        "rust"
            | "rs"
            | "python"
            | "py"
            | "javascript"
            | "js"
            | "typescript"
            | "ts"
            | "tsx"
            | "jsx"
            | "go"
            | "java"
            | "c"
            | "cpp"
            | "c++"
            | "csharp"
            | "cs"
            | "ruby"
            | "rb"
            | "php"
            | "swift"
            | "kotlin"
            | "scala"
            | "html"
            | "css"
            | "scss"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "md"
            | "markdown"
            | "sh"
            | "bash"
            | "shell"
            | "zsh"
            | "sql"
            | "xml"
            | "vue"
            | "svelte"
            | "zig"
            | "lua"
            | "r"
            | "dart"
            | "text"
            | "plaintext"
            | "txt"
            | "diff"
            | "dockerfile"
            | "makefile"
            | "ini"
            | "env"
            | "console"
            | "output"
    )
}

fn looks_like_path(s: &str) -> bool {
    let s = s.trim().trim_matches('"').trim_matches('\'');
    if s.is_empty() || s.contains(' ') {
        return false;
    }
    if s.contains("..") {
        return false;
    }
    if s.contains('/') || s.contains('\\') {
        return true;
    }
    Path::new(s).extension().is_some()
}

fn parse_fence_info(info: &str) -> (String, Option<String>) {
    let info = info.trim();
    if info.is_empty() {
        return (String::new(), None);
    }

    // `rust:src/main.rs` / `ts:app/page.tsx`
    if let Some((lang, rest)) = info.split_once(':') {
        let lang = lang.trim();
        let rest = rest.trim().trim_matches('"').trim_matches('\'');
        if !lang.is_empty() && looks_like_path(rest) {
            return (lang.to_string(), Some(rest.replace('\\', "/")));
        }
    }

    let mut tokens = info
        .split_whitespace()
        .map(|t| t.trim_matches('"').trim_matches('\'').to_string())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        return (String::new(), None);
    }

    if tokens.len() == 1 {
        let t = tokens.remove(0);
        if looks_like_path(&t) {
            return (String::new(), Some(t.replace('\\', "/")));
        }
        return (t, None);
    }

    let first = tokens.remove(0);
    let maybe_path = tokens.join(" ");
    if looks_like_path(&maybe_path) {
        let lang = if is_known_lang(&first) {
            first
        } else {
            String::new()
        };
        return (lang, Some(maybe_path.replace('\\', "/")));
    }
    if is_known_lang(&first) {
        return (first, None);
    }
    (first, None)
}

fn path_hint_from_code_body(code: &str) -> Option<String> {
    let first = code.lines().next()?.trim();
    if first.is_empty() {
        return None;
    }

    let lower = first.to_ascii_lowercase();
    for prefix in [
        "// file:",
        "# file:",
        "<!-- file:",
        "// path:",
        "# path:",
        "file:",
        "//",
        "#",
    ] {
        if lower.starts_with(prefix) {
            let rest = first[prefix.len()..].trim();
            let rest = rest.trim_end_matches("-->").trim();
            let rest = rest.trim_matches('"').trim_matches('\'');
            if looks_like_path(rest) {
                return Some(rest.replace('\\', "/"));
            }
        }
    }
    None
}

/// 将 Markdown 拆成 HTML 段与可交互代码块。
fn parse_chat_md_parts(src: &str) -> Vec<ChatMdPart> {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut html_buf = String::new();
    let mut in_code = false;
    let mut code_info = String::new();
    let mut code_buf = String::new();
    let mut pending: Vec<Event<'_>> = Vec::new();

    let push_pending = |pending: &mut Vec<Event<'_>>, html_buf: &mut String| {
        if pending.is_empty() {
            return;
        }
        html::push_html(html_buf, pending.drain(..));
    };

    for event in Parser::new_ext(src, markdown_options()) {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                push_pending(&mut pending, &mut html_buf);
                flush_html(&mut html_buf, &mut out);
                in_code = true;
                code_buf.clear();
                code_info = match kind {
                    CodeBlockKind::Fenced(info) => info.into_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_code {
                    let (lang, fence_path) = parse_fence_info(&code_info);
                    let body_path = path_hint_from_code_body(&code_buf);
                    let path_hint = fence_path.or(body_path);
                    out.push(ChatMdPart::Code {
                        lang,
                        path_hint,
                        code: code_buf.clone(),
                    });
                    code_buf.clear();
                    code_info.clear();
                    in_code = false;
                }
            }
            Event::Text(t) if in_code => {
                code_buf.push_str(&t);
            }
            other if in_code => {
                // 代码块内偶发 SoftBreak 等：忽略，内容应已在 Text 中。
                let _ = other;
            }
            other => pending.push(other),
        }
    }

    push_pending(&mut pending, &mut html_buf);
    flush_html(&mut html_buf, &mut out);

    if out.is_empty() {
        let cleaned = ammonia::clean(trimmed);
        if !cleaned.is_empty() {
            out.push(ChatMdPart::Html(cleaned));
        }
    }
    out
}

/// Markdown → 经消毒的 HTML（扁平渲染，供测试与兼容）。
#[cfg_attr(not(test), allow(dead_code))]
pub fn render_markdown_html(src: &str) -> String {
    let parts = parse_chat_md_parts(src);
    let mut html = String::new();
    for part in parts {
        match part {
            ChatMdPart::Html(chunk) => html.push_str(&chunk),
            ChatMdPart::Code { lang, code, .. } => {
                html.push_str("<pre><code");
                if !lang.is_empty() {
                    html.push_str(" class=\"language-");
                    html.push_str(&ammonia::clean(&lang));
                    html.push('"');
                }
                html.push('>');
                html.push_str(&html_escape(&code));
                html.push_str("</code></pre>");
            }
        }
    }
    html
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

fn copy_code_to_clipboard(text: String) {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            let _ = clipboard.write_text(&text);
        }
    }
    #[cfg(not(all(target_arch = "wasm32", feature = "web")))]
    {
        let _ = text;
    }
}

/// 助手/用户气泡正文：Markdown + 可复制代码块。
#[component]
pub fn ChatMarkdownBody(content: String) -> Element {
    let parts = parse_chat_md_parts(&content);

    rsx! {
        div {
            class: "ac-chat-md",
            for (idx, part) in parts.into_iter().enumerate() {
                match part {
                    ChatMdPart::Html(html) => rsx! {
                        div {
                            key: "{idx}-html",
                            class: "ac-chat-md-chunk",
                            dangerous_inner_html: html,
                        }
                    },
                    ChatMdPart::Code { lang, path_hint, code } => rsx! {
                        ChatCodeBlock {
                            key: "{idx}-code",
                            lang,
                            path_hint,
                            code,
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn ChatCodeBlock(
    lang: String,
    path_hint: Option<String>,
    code: String,
) -> Element {
    let path_label = path_hint
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();
    let header_right = if path_label.is_empty() {
        if lang.is_empty() {
            String::new()
        } else {
            lang.clone()
        }
    } else {
        path_label.clone()
    };

    rsx! {
        div { class: "ac-chat-code",
            div { class: "ac-chat-code-toolbar",
                span { class: "ac-chat-code-meta",
                    if header_right.is_empty() {
                        "代码"
                    } else {
                        "{header_right}"
                    }
                }
                div { class: "ac-chat-code-actions",
                    button {
                        r#type: "button",
                        class: "ac-chat-code-btn",
                        title: "复制代码",
                        aria_label: "复制代码",
                        onclick: {
                            let code = code.clone();
                            move |_| copy_code_to_clipboard(code.clone())
                        },
                        Icon { icon: LdCopy, width: 14, height: 14, fill: "currentColor" }
                        span { "复制" }
                    }
                }
            }
            pre { class: "ac-chat-code-pre",
                code { "{code}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_headings_and_bold() {
        let html = render_markdown_html("## 执行结论\n\n**仅监控**");
        assert!(html.contains("<h2"));
        assert!(html.contains("执行结论"));
        assert!(html.contains("<strong>"));
        assert!(html.contains("仅监控"));
        assert!(!html.contains("##"));
        assert!(!html.contains("**"));
    }

    #[test]
    fn renders_horizontal_rule() {
        let html = render_markdown_html("a\n\n---\n\nb");
        assert!(html.contains("<hr"));
    }

    #[test]
    fn extracts_fenced_code_with_path() {
        let parts = parse_chat_md_parts("见下：\n\n```rust src/lib.rs\nfn main() {}\n```\n");
        assert!(parts.iter().any(|p| matches!(
            p,
            ChatMdPart::Code {
                path_hint: Some(path),
                ..
            } if path == "src/lib.rs"
        )));
    }
}
