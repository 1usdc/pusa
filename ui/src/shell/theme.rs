//! 应用亮色 / 暗色主题：持久化 + 写入 `html[data-theme]`。
//! 字体大小：持久化 + 写入 `html[data-font-size]`。

use dioxus::prelude::*;
use crate::shell::style::{
    WEB_TITLEBAR_MENU_ITEM,
};

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

/// 设置下拉里的主题切换：显示当前主题的相反项，一点即切。
#[component]
pub fn TitlebarThemeToggle(
    mut ui_theme: Signal<UiTheme>,
    mut show_titlebar_settings_menu: Signal<bool>,
) -> Element {
    let next = ui_theme().opposite();
    rsx! {
        button {
            r#type: "button",
            role: "menuitem",
            class: WEB_TITLEBAR_MENU_ITEM,
            onclick: move |_| {
                set(next);
                ui_theme.set(next);
                show_titlebar_settings_menu.set(false);
            },
            "{next.label()}"
        }
    }
}

/// 字体大小设置弹窗。
#[component]
pub fn FontSizeModal(mut show_font_size_modal: Signal<bool>) -> Element {
    let mut font_size = use_signal(load_font_size);
    rsx! {
        div {
            class: "fixed top-[var(--ac-chrome-top,0px)] right-0 bottom-[var(--ac-chrome-bottom,1.625rem)] left-0 z-[1250] flex items-center justify-center px-4 box-border",
            div {
                class: "absolute inset-0 bg-[var(--shadow-color)] backdrop-blur-[10px] cursor-pointer",
                onclick: move |_| show_font_size_modal.set(false),
            }
            div {
                class: "relative z-[1] w-full max-w-[32rem] h-auto max-h-[calc(100%-40px)] overflow-hidden flex flex-col min-h-0 box-border",
                div { class: crate::shell::style::MODAL_CARD,
                    div { class: crate::shell::style::MODAL_HEAD,
                        h1 { class: crate::shell::style::MODAL_TITLE, "字体大小" }
                        button {
                            r#type: "button",
                            class: crate::shell::style::MODAL_CLOSE,
                            title: "关闭",
                            aria_label: "关闭",
                            onclick: move |_| show_font_size_modal.set(false),
                            "×"
                        }
                    }
                    div { class: crate::shell::style::MODAL_BODY_FLEX_ROW,
                        button {
                            r#type: "button",
                            class: "flex-1 px-4 py-3 rounded-[0.5rem] text-[1rem] font-medium cursor-pointer bg-transparent border-0 transition-all duration-150 ease-[ease] data-[active=true]:bg-primary-2 data-[active=true]:font-semibold data-[active=true]:text-black dark:data-[active=true]:text-white",
                            "data-active": font_size() == FontSize::Small,
                            onclick: move |_| {
                                set_font_size(FontSize::Small);
                                font_size.set(FontSize::Small);
                            },
                            "小"
                        }
                        button {
                            r#type: "button",
                            class: "flex-1 px-4 py-3 rounded-[0.5rem] text-[1rem] font-medium cursor-pointer bg-transparent border-0 transition-all duration-150 ease-[ease] data-[active=true]:bg-primary-2 data-[active=true]:font-semibold data-[active=true]:text-black dark:data-[active=true]:text-white",
                            "data-active": font_size() == FontSize::Medium,
                            onclick: move |_| {
                                set_font_size(FontSize::Medium);
                                font_size.set(FontSize::Medium);
                            },
                            "中"
                        }
                        button {
                            r#type: "button",
                            class: "flex-1 px-4 py-3 rounded-[0.5rem] text-[1rem] font-medium cursor-pointer bg-transparent border-0 transition-all duration-150 ease-[ease] data-[active=true]:bg-primary-2 data-[active=true]:font-semibold data-[active=true]:text-black dark:data-[active=true]:text-white",
                            "data-active": font_size() == FontSize::Large,
                            onclick: move |_| {
                                set_font_size(FontSize::Large);
                                font_size.set(FontSize::Large);
                            },
                            "大"
                        }
                    }
                }
            }
        }
    }
}
