//! 应用亮色 / 暗色主题：持久化 + 写入 `html[data-theme]`。
//! 字体大小：持久化 + 写入 `html[data-font-size]`。

use dioxus::prelude::*;

/// UI 主题。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTheme {
    Light,
    Dark,
}

/// 字体大小。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontSize {
    Small,
    Medium,
    Large,
}

impl Default for FontSize {
    fn default() -> Self {
        Self::Medium
    }
}

impl UiTheme {
    pub fn as_attr(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn from_stored(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "dark" => Self::Dark,
            _ => Self::Light,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "亮色主题",
            Self::Dark => "暗色主题",
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

impl FontSize {
    pub fn as_attr(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }

    pub fn from_stored(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "small" => Self::Small,
            "large" => Self::Large,
            _ => Self::Medium,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Small => "小",
            Self::Medium => "中",
            Self::Large => "大",
        }
    }

    pub fn css_value(self) -> &'static str {
        match self {
            Self::Small => "14px",
            Self::Medium => "16px",
            Self::Large => "18px",
        }
    }
}

fn persist_load() -> Option<String> {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        return crate::web::prefs::ui_theme_get();
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        return crate::desktop::files::ui_theme_get();
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        None
    }
}

fn persist_save(theme: UiTheme) {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        crate::web::prefs::ui_theme_set(theme.as_attr());
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::files::ui_theme_set(theme.as_attr());
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        let _ = theme;
    }
}

fn font_size_persist_load() -> Option<String> {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        return crate::web::prefs::font_size_get();
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        return crate::desktop::files::font_size_get();
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        None
    }
}

fn font_size_persist_save(font_size: FontSize) {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        crate::web::prefs::font_size_set(font_size.as_attr());
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::files::font_size_set(font_size.as_attr());
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        let _ = font_size;
    }
}

/// 读取已保存主题；缺省为亮色。
pub fn load() -> UiTheme {
    persist_load()
        .as_deref()
        .map(UiTheme::from_stored)
        .unwrap_or(UiTheme::Light)
}

/// 写入 `html[data-theme]`（WebView / 浏览器）。
pub fn apply_dom(theme: UiTheme) {
    let attr = theme.as_attr();
    let js = format!(
        r#"(() => {{
  const root = document.documentElement;
  if (!root) return;
  root.setAttribute('data-theme', '{attr}');
  root.style.colorScheme = '{attr}';
}})();"#
    );
    let _ = document::eval(&js);
}

/// 应用并持久化。
pub fn set(theme: UiTheme) {
    persist_save(theme);
    apply_dom(theme);
}

/// 启动时恢复主题到 DOM。
pub fn restore_on_launch() {
    apply_dom(load());
}

/// 读取已保存字体大小；缺省为中。
pub fn load_font_size() -> FontSize {
    font_size_persist_load()
        .as_deref()
        .map(FontSize::from_stored)
        .unwrap_or(FontSize::Medium)
}

/// 写入 `html[data-font-size]`（WebView / 浏览器）。
pub fn apply_font_size_dom(font_size: FontSize) {
    let attr = font_size.as_attr();
    let css_value = font_size.css_value();
    let js = format!(
        r#"(() => {{
  const root = document.documentElement;
  if (!root) return;
  root.setAttribute('data-font-size', '{attr}');
  root.style.fontSize = '{css_value}';
}})();"#
    );
    let _ = document::eval(&js);
}

/// 应用并持久化字体大小。
pub fn set_font_size(font_size: FontSize) {
    font_size_persist_save(font_size);
    apply_font_size_dom(font_size);
}

/// 启动时恢复字体大小到 DOM。
pub fn restore_font_size_on_launch() {
    apply_font_size_dom(load_font_size());
}
