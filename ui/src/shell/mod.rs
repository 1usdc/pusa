//! 主控制台骨架（布局与交互）。
mod console;
pub(crate) mod files;
mod llm_settings;
mod plugins;
mod search;
mod status_bar;
pub(crate) mod syntax;
pub mod theme;
pub mod toast;

pub(crate) use llm_settings::LlmCredentialsPanel;

pub use console::Console;
pub use status_bar::StatusBar;
