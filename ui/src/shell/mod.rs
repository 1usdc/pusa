//! 主控制台骨架（布局与交互）。
mod console;
pub(crate) mod files;
mod llm_enabled;
mod llm_settings;
mod plugins;
mod search;
mod status_bar;
pub(crate) mod syntax;
pub mod theme;
pub mod toast;

pub(crate) use llm_enabled::{
    activate_id_after_toggle, enabled_after_delete, enabled_after_toggle, enabled_after_upsert,
    overlay_enabled_credential_ids, LlmModelsRefresh,
};
pub(crate) use llm_settings::{format_llm_settings_error, LlmCredentialsPanel};

pub use console::Console;
pub use status_bar::StatusBar;
