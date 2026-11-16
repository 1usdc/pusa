//! 主控制台骨架（布局与交互）。
pub(crate) mod browser;
mod console;
pub(crate) mod dev_mode;
pub(crate) mod files;
mod llm_enabled;
mod llm_settings;
mod model_provider_detail;
pub mod settings_page;
mod plugins;
mod search;
mod status_bar;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod textmate;
pub(crate) mod syntax;
pub mod theme;
pub mod toast;

pub(crate) use llm_enabled::{
    activate_id_after_toggle, enabled_after_delete, enabled_after_toggle, enabled_after_upsert,
    overlay_enabled_credential_ids, LlmModelsRefresh,
};
pub(crate) use llm_settings::{format_llm_settings_error, ModelSettingsPanel};
pub(crate) use model_provider_detail::AddProviderSheet;
pub use settings_page::{
    SettingsAboutSectionWrapper, SettingsInfoRow, SettingsInfoRowView, SettingsNavShell,
    SettingsSection, ThemePickerCard, SETTINGS_SECTIONS,
};

pub use console::Console;
pub use status_bar::StatusBar;
