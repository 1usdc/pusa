//! 桌面 Dioxus 壳（非 WASM）：进程内 Agent、窗口与标题栏。
#![cfg(all(feature = "native", not(target_arch = "wasm32")))]

pub mod agent;
pub mod auth;
pub mod chrome;
pub mod files;
pub mod html_preview;
pub mod plugins;
pub mod terminal;
pub mod updater;
pub mod window;
