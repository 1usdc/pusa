//! 主控制台骨架（布局与交互）。
mod console;
pub(crate) mod files;
mod plugins;
mod status_bar;
pub(crate) mod syntax;
pub mod toast;

pub use console::Console;
pub use status_bar::StatusBar;
