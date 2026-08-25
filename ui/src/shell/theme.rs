//! 应用亮色 / 暗色主题：持久化 + 写入 `html[data-theme]`。

use dioxus::prelude::*;

/// UI 主题。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTheme {
    Light,
    Dark,
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
            class: "ac-web-titlebar-menu__item",
            onclick: move |_| {
                set(next);
                ui_theme.set(next);
                show_titlebar_settings_menu.set(false);
            },
            "{next.label()}"
        }
    }
}
