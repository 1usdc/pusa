//! 主控制台骨架（布局与交互）。
pub(crate) mod browser;
mod console;
pub(crate) mod dev_mode;
pub(crate) mod files;
mod llm_enabled;
mod llm_settings;
mod model_provider_detail;
mod plugins;
mod search;
pub mod settings_page;
mod status_bar;
pub(crate) mod style;
pub(crate) mod syntax;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod textmate;
pub mod theme;
pub mod toast;

pub use theme::FontSize;

pub(crate) use llm_enabled::{
    activate_id_after_toggle, enabled_after_delete, enabled_after_toggle, enabled_after_upsert,
    overlay_enabled_credential_ids, LlmModelsRefresh,
};
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub(crate) use llm_settings::format_llm_settings_error;
pub(crate) use llm_settings::ModelSettingsPanel;
pub use settings_page::{
    FontSizeSettingRow, SettingsInfoRow, SettingsInfoRowView, SettingsNavShell, SettingsSection,
    ThemeSettingRow, SETTINGS_SECTIONS,
};

pub use console::Console;
pub use status_bar::StatusBar;

/// 状态栏「项目名」弹窗选中的待打开项目路径；由 [`Console`] 监听并完成文件树 / 标签页刷新。
#[derive(Clone, Copy)]
pub struct OpenProjectPath(pub dioxus::prelude::Signal<String>);
