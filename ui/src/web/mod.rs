//! 浏览器（WASM）下开发者开关与 LLM / API 设置。
#![cfg(target_arch = "wasm32")]

/// Web 外壳向 [`crate::shell::Console`] 内标题栏注入的设置相关信号（由 Web 根 `WebAppShell` 通过 `use_context_provider` 提供）。
#[cfg(feature = "web")]
#[derive(Clone, Copy)]
pub struct ShellChromeCtx {
    pub show_settings_modal: dioxus::prelude::Signal<bool>,
    pub show_about_modal: dioxus::prelude::Signal<bool>,
    pub show_titlebar_settings_menu: dioxus::prelude::Signal<bool>,
    pub developer_mode: dioxus::prelude::Signal<bool>,
}

pub mod auth;

#[cfg(feature = "web")]
pub mod dev_mode;
#[cfg(feature = "web")]
pub mod llm_config;
#[cfg(feature = "web")]
pub mod password_field;
#[cfg(feature = "web")]
pub mod about;
#[cfg(feature = "web")]
pub mod settings;
#[cfg(feature = "web")]
pub mod origin;
#[cfg(feature = "web")]
pub mod prefs;
