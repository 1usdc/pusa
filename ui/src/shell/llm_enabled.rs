//! 已保存 LLM 密钥的「独立启用」状态。
//!
//! 核心库仍只有单一 `active_credential_id`（对话实际走这一组）。
//! 开关允许多选：启用集合存在本地；模型列表合并所有已开启端点。

use dioxus::prelude::*;

/// 设置里开关变化后，聊天栏重新拉取合并模型列表。
#[derive(Clone, Copy)]
pub struct LlmModelsRefresh(pub Signal<u32>);

/// 已持久化的启用 id；从未保存过则 `None`（回落到当前 `active_credential_id`）。
pub fn load_enabled_credential_ids() -> Option<Vec<String>> {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        return crate::web::prefs::llm_credentials_enabled_get();
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        return crate::desktop::files::llm_credentials_enabled_get();
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        None
    }
}

pub fn persist_enabled_credential_ids(ids: &[String]) {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        crate::web::prefs::llm_credentials_enabled_set(ids);
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::files::llm_credentials_enabled_set(ids);
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        let _ = ids;
    }
}

/// 与现存密钥对齐：丢掉已删除的 id。
/// 尚无本地记录时，仅打开当前 `active_id`（与旧的单选启用一致）。
pub fn overlay_enabled_credential_ids(known_ids: &[String], active_id: &str) -> Vec<String> {
    let known: std::collections::HashSet<&str> = known_ids.iter().map(String::as_str).collect();
    if let Some(saved) = load_enabled_credential_ids() {
        return saved
            .into_iter()
            .filter(|id| known.contains(id.as_str()))
            .collect();
    }
    let active = active_id.trim();
    if !active.is_empty() && known.contains(active) {
        vec![active.to_string()]
    } else {
        Vec::new()
    }
}

pub fn bump_llm_models_refresh(mut tick: Signal<u32>) {
    tick += 1;
}

fn persist_enabled(ids: &[String], refresh: Signal<u32>) {
    persist_enabled_credential_ids(ids);
    bump_llm_models_refresh(refresh);
}

pub fn enabled_after_toggle(
    current: &[String],
    id: &str,
    on: bool,
    refresh: Signal<u32>,
) -> Vec<String> {
    let mut out: Vec<String> = current
        .iter()
        .filter(|x| x.as_str() != id)
        .cloned()
        .collect();
    if on {
        out.push(id.to_string());
    }
    persist_enabled(&out, refresh);
    out
}

pub fn enabled_after_upsert(
    prev_enabled: &[String],
    prev_ids: &[String],
    new_ids: &[String],
    refresh: Signal<u32>,
) -> Vec<String> {
    let known: std::collections::HashSet<&str> = new_ids.iter().map(String::as_str).collect();
    let mut out: Vec<String> = prev_enabled
        .iter()
        .filter(|id| known.contains(id.as_str()))
        .cloned()
        .collect();
    for id in new_ids {
        if !prev_ids.iter().any(|p| p == id) && !out.iter().any(|x| x == id) {
            out.push(id.clone());
        }
    }
    persist_enabled(&out, refresh);
    out
}

pub fn enabled_after_delete(
    prev_enabled: &[String],
    known_ids: &[String],
    refresh: Signal<u32>,
) -> Vec<String> {
    let known: std::collections::HashSet<&str> = known_ids.iter().map(String::as_str).collect();
    let out: Vec<String> = prev_enabled
        .iter()
        .filter(|id| known.contains(id.as_str()))
        .cloned()
        .collect();
    persist_enabled(&out, refresh);
    out
}

/// 开关变化后，是否需要把核心库的单一 `active_id` 指到某一组（对话仍走这一组）。
pub fn activate_id_after_toggle(
    enabled: &[String],
    active_id: &str,
    toggled_id: &str,
    on: bool,
) -> Option<String> {
    if on {
        if active_id.trim().is_empty() || !enabled.iter().any(|x| x == active_id) {
            Some(toggled_id.to_string())
        } else {
            None
        }
    } else if active_id == toggled_id {
        enabled.first().cloned()
    } else {
        None
    }
}
