//! Web：开发者模式开关（委托 [`crate::shell::dev_mode`]）。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

pub use crate::shell::dev_mode::{load as get, set, sync_body_to};
