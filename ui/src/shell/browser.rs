//! Pusa 浏览器：中间栏标签页里的内嵌网页浏览（地址栏 + iframe）。
//!
//! 受站点 `X-Frame-Options` / CSP `frame-ancestors` 限制的页面可能无法嵌入。

use dioxus::prelude::*;

/// 中间栏标签栏右键菜单的动作（空页菜单与标签栏菜单共用）。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TabStripMenuAction {
    OpenBrowser,
}

/// 外壳标题栏等请求打开 Pusa 浏览器：递增计数即触发一次。
///
/// 由桌面 / Web 根 shell `use_context_provider`；[`crate::shell::Console`] 订阅后开新标签。
#[derive(Clone, Copy)]
pub struct OpenBrowserTick(pub Signal<u64>);

impl OpenBrowserTick {
    /// 请求打开一个新的 Pusa 浏览器标签。
    pub fn request(mut self) {
        self.0.with_mut(|n| *n = n.wrapping_add(1));
    }
}

/// 浏览器标签标题：未输入网址时为「Pusa 浏览器」，否则显示站点域名
/// （多个浏览器标签并存时靠它区分）。
pub fn browser_tab_title(url: &str) -> String {
    let rest = url
        .trim()
        .split_once("://")
        .map_or(url.trim(), |(_, rest)| rest);
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    if host.is_empty() {
        "Pusa 浏览器".to_string()
    } else {
        host.to_string()
    }
}

/// 规范化地址：去空白；没写协议时补 `https://`。空输入返回 `None`。
fn normalize_browser_url(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    if t.contains("://") {
        Some(t.to_string())
    } else {
        Some(format!("https://{t}"))
    }
}

/// 中间栏 Pusa 浏览器面板。
///
/// `url` 为当前加载的网址（空串 = 尚未输入，显示空态）；
/// 用户在地址栏回车 / 点「访问」后通过 `on_navigate` 通知上层更新标签状态
/// （从而随 `center_tabs` 一起持久化到操作缓存）。
#[component]
pub fn WebBrowserPane(url: String, on_navigate: EventHandler<String>) -> Element {
    let mut input_value = use_signal(|| url.clone());

    let mut navigate = move || {
        if let Some(next) = normalize_browser_url(&input_value()) {
            input_value.set(next.clone());
            on_navigate.call(next);
        }
    };

    let url_is_empty = url.is_empty();
    rsx! {
        section { class: "ac-web-browser-pane",
            div { class: "ac-web-browser-bar",
                input {
                    r#type: "text",
                    class: "ac-web-browser-input",
                    placeholder: "输入网址后回车，例如 example.com",
                    autofocus: url_is_empty,
                    value: "{input_value()}",
                    oninput: move |e| input_value.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            e.prevent_default();
                            navigate();
                        }
                    },
                }
                button {
                    r#type: "button",
                    class: "ac-web-browser-go",
                    onclick: move |_| navigate(),
                    "访问"
                }
            }
            if url.is_empty() {
                div { class: "ac-web-browser-empty",
                    p { "在上方地址栏输入网址，回车后即可在 Pusa 内浏览网页。" }
                }
            } else {
                iframe {
                    class: "ac-web-browser-frame",
                    src: "{url}",
                    title: "Pusa 浏览器",
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_scheme_less_input() {
        assert_eq!(
            normalize_browser_url("example.com").as_deref(),
            Some("https://example.com")
        );
        assert_eq!(
            normalize_browser_url("  huangguoai.com/videos  ").as_deref(),
            Some("https://huangguoai.com/videos")
        );
    }

    #[test]
    fn keeps_explicit_scheme() {
        assert_eq!(
            normalize_browser_url("http://localhost:8080").as_deref(),
            Some("http://localhost:8080")
        );
    }

    #[test]
    fn tab_title_uses_host_or_default() {
        assert_eq!(browser_tab_title(""), "Pusa 浏览器");
        assert_eq!(browser_tab_title("https://example.com/a/b?x=1"), "example.com");
        assert_eq!(browser_tab_title("http://localhost:8080"), "localhost:8080");
        assert_eq!(browser_tab_title("example.com#top"), "example.com");
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(normalize_browser_url("   "), None);
    }
}
