//! 模型设置面板：自定义模型供应商（Base URL / API 格式 / API Key / 模型列表）。
//!
//! 左侧供应商列表（含启用状态圆点）+ 右侧详情（见
//! [`crate::shell::model_provider_detail`]）。「添加供应商」支持预设一键预填
//! （见 [`crate::model_presets`]）或完全自定义。供应商数据仍走
//! `LlmCredentialUpsertBody`（服务端 / 核心库存储）；模型列表为本地偏好
//! （`CustomChatModelPref`，按 `credential_id` 归组）。

use std::collections::HashMap;

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdBox, LdPlus, LdRefreshCw};
use dioxus_free_icons::Icon;
use protocol::{LlmApiProvider, LlmCredentialDto, LlmCredentialUpsertBody};

use crate::custom_chat_models::CustomChatModelPref;

/// 核心库 `llm_credential_upsert` 拒绝空 `api_key`（错误码 `api_key_required`）。
/// 选填密钥时写入此占位值；界面展示为「未填写」。
pub(crate) const EMPTY_API_KEY_SENTINEL: &str = "__pusa_no_key__";

pub(crate) fn is_unspecified_api_key(key: &str) -> bool {
    let t = key.trim();
    t.is_empty() || t == EMPTY_API_KEY_SENTINEL
}

pub(crate) fn api_key_for_upsert(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        EMPTY_API_KEY_SENTINEL.to_string()
    } else {
        t.to_string()
    }
}

pub(crate) fn mask_api_key(key: &str) -> String {
    if is_unspecified_api_key(key) {
        return "未填写".into();
    }
    let t = key.trim();
    let chars: Vec<char> = t.chars().collect();
    if chars.len() <= 8 {
        return "••••".into();
    }
    let head: String = chars.iter().take(4).collect();
    let tail: String = chars.iter().rev().take(4).rev().collect();
    format!("{head}…{tail}")
}

pub(crate) fn provider_label(p: LlmApiProvider) -> &'static str {
    match p {
        LlmApiProvider::Openai => "OpenAI",
        LlmApiProvider::Anthropic => "Anthropic",
    }
}

pub(crate) fn display_name(provider: LlmApiProvider, label: &str) -> String {
    let label = label.trim();
    if label.is_empty() {
        provider_label(provider).to_string()
    } else {
        label.to_string()
    }
}

pub(crate) fn credential_display_name(cred: &LlmCredentialDto) -> String {
    display_name(cred.provider, &cred.label)
}

/// 从 Base URL 取出 host（支持 `http://host:port/path`、`[IPv6]`、无 scheme）。
pub(crate) fn host_from_openai_base(base: &str) -> Option<String> {
    let s = base.trim();
    if s.is_empty() {
        return None;
    }
    let rest = s
        .split_once("://")
        .map(|(_, r)| r)
        .unwrap_or(s)
        .trim_start_matches('/');
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = rest.get(..authority_end)?;
    if authority.is_empty() {
        return None;
    }
    let hostport = if let Some(at) = authority.rfind('@') {
        authority.get(at + 1..)?
    } else {
        authority
    };
    if let Some(inner) = hostport.strip_prefix('[') {
        let end = inner.find(']')?;
        return Some(inner[..end].to_string());
    }
    if hostport.matches(':').count() > 1 {
        return Some(hostport.to_string());
    }
    Some(
        hostport
            .split_once(':')
            .map(|(h, _)| h)
            .unwrap_or(hostport)
            .to_string(),
    )
}

fn parse_ipv4(host: &str) -> Option<[u8; 4]> {
    let mut out = [0u8; 4];
    let mut parts = host.split('.');
    for slot in &mut out {
        *slot = parts.next()?.parse().ok()?;
    }
    if parts.next().is_some() {
        return None;
    }
    Some(out)
}

fn is_loopback_or_private_ipv4([a, b, _, _]: [u8; 4]) -> bool {
    a == 127 || a == 10 || (a == 192 && b == 168) || (a == 172 && (16..=31).contains(&b))
}

fn is_loopback_or_private_host(host: &str) -> bool {
    let h = host.trim().trim_matches(|c| c == '[' || c == ']');
    if h.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if h == "::1" || h.eq_ignore_ascii_case("0:0:0:0:0:0:0:1") {
        return true;
    }
    parse_ipv4(h).is_some_and(is_loopback_or_private_ipv4)
}

pub(crate) fn is_local_or_private_openai_base(base: &str) -> bool {
    host_from_openai_base(base).is_some_and(|h| is_loopback_or_private_host(&h))
}

/// 把核心库 / FFI 的原始错误码转成设置页可读文案，避免当成标题展示。
pub(crate) fn format_llm_settings_error(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return "操作失败，请稍后重试。".into();
    }
    let lower = t.to_ascii_lowercase();
    if lower.contains("api_key_required") {
        return "保存失败。API Key 现为选填；若卡片未出现请再试一次。".into();
    }
    if lower.contains("openai_api_key_missing") {
        return "当前对话还没有可用的模型端点。".into();
    }
    let code = t
        .rsplit(|c: char| c == ':' || c == '/' || c == ' ' || c == '\n')
        .find(|s| !s.is_empty())
        .unwrap_or(t)
        .trim()
        .trim_matches('"');
    if !code.is_empty()
        && code.chars().all(|c| c.is_ascii_lowercase() || c == '_')
        && code.contains('_')
    {
        return "操作失败，请稍后重试。".into();
    }
    t.to_string()
}

// —— 模型列表本地偏好（与聊天栏模型选择共用一份存储） ——

/// 读取全部自定义模型偏好（Web `localStorage` / 桌面应用数据目录）。
pub(crate) fn load_model_prefs() -> Vec<CustomChatModelPref> {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        return crate::web::prefs::custom_chat_models_get();
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        return crate::desktop::files::custom_chat_models_get();
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        Vec::new()
    }
}

/// 写入全部自定义模型偏好。
pub(crate) fn persist_model_prefs(items: &[CustomChatModelPref]) {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        crate::web::prefs::custom_chat_models_set(items);
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        crate::desktop::files::custom_chat_models_set(items);
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        let _ = items;
    }
}

/// 某供应商名下的模型（保持写入顺序）。
pub(crate) fn prefs_for_credential(
    prefs: &[CustomChatModelPref],
    cred_id: &str,
) -> Vec<CustomChatModelPref> {
    prefs
        .iter()
        .filter(|p| p.credential_id == cred_id)
        .cloned()
        .collect()
}

/// 追加模型；重复 id（全局）返回 `false` 不写入。
pub(crate) fn add_model_pref(cred_id: &str, model_id: &str) -> bool {
    let id = model_id.trim();
    if id.is_empty() {
        return false;
    }
    let mut prefs = load_model_prefs();
    if prefs.iter().any(|p| p.id == id) {
        return false;
    }
    prefs.push(CustomChatModelPref {
        id: id.to_string(),
        name: String::new(),
        credential_id: cred_id.to_string(),
        enabled: true,
    });
    persist_model_prefs(&prefs);
    true
}

/// 重命名；`next_id` 为空或撞名返回 `false`。
pub(crate) fn rename_model_pref(old_id: &str, next_id: &str) -> bool {
    let next = next_id.trim();
    if next.is_empty() || next == old_id {
        return false;
    }
    let mut prefs = load_model_prefs();
    if prefs.iter().any(|p| p.id == next) {
        return false;
    }
    let mut changed = false;
    for p in prefs.iter_mut() {
        if p.id == old_id {
            p.id = next.to_string();
            changed = true;
        }
    }
    if changed {
        persist_model_prefs(&prefs);
    }
    changed
}

pub(crate) fn remove_model_pref(model_id: &str) {
    let mut prefs = load_model_prefs();
    prefs.retain(|p| p.id != model_id);
    persist_model_prefs(&prefs);
}

pub(crate) fn set_model_pref_enabled(model_id: &str, enabled: bool) {
    let mut prefs = load_model_prefs();
    let mut changed = false;
    for p in prefs.iter_mut() {
        if p.id == model_id && p.enabled != enabled {
            p.enabled = enabled;
            changed = true;
        }
    }
    if changed {
        persist_model_prefs(&prefs);
    }
}

/// 模型设置面板：顶栏（刷新 + 添加供应商）+ 左侧供应商列表 + 右侧详情。
#[component]
pub fn ModelSettingsPanel(
    credentials: Vec<LlmCredentialDto>,
    enabled_ids: Vec<String>,
    hint: Option<String>,
    on_add: EventHandler<LlmCredentialUpsertBody>,
    on_update: EventHandler<LlmCredentialUpsertBody>,
    on_toggle: EventHandler<(String, bool)>,
    on_delete: EventHandler<String>,
    on_refresh: EventHandler<()>,
) -> Element {
    let mut selected_id = use_signal(String::new);
    let mut show_add = use_signal(|| false);
    // 预设供应商落库后待绑定的默认模型：按 label 匹配新增凭证 id。
    let mut pending_defaults = use_signal(|| None::<(String, Vec<String>)>);
    // 模型偏好变化时 bump，驱动右侧模型列表重算。
    let mut prefs_epoch = use_signal(|| 0u64);

    // 渲染时计算：选中项丢失则回落第一项。
    if !credentials.iter().any(|c| c.id == selected_id()) {
        let fallback = credentials.first().map(|c| c.id.clone()).unwrap_or_default();
        if selected_id() != fallback {
            selected_id.set(fallback);
        }
    }

    // 预设默认模型绑定：把 pending 模型写入匹配 label 的凭证（渲染时一次）。
    let pending_now = pending_defaults.peek().cloned();
    if let Some((label, models)) = pending_now {
        let do_clear = models.is_empty()
            || credentials.iter().any(|c| c.label == label);
        if do_clear {
            if !models.is_empty() {
                if let Some(cred) = credentials.iter().find(|c| c.label == label) {
                    let mut bound = 0usize;
                    for m in &models {
                        if add_model_pref(&cred.id, m) {
                            bound += 1;
                        }
                    }
                    if bound > 0 {
                        prefs_epoch += 1;
                    }
                }
            }
            pending_defaults.take();
        }
    }

    let has_credentials = !credentials.is_empty();
    rsx! {
        div { class: "ac-model-settings",
            p { class: "ac-settings-lead",
                "管理自定义模型供应商，配置后可在聊天时选择使用。"
            }
            if let Some(h) = hint.filter(|s| !s.trim().is_empty()) {
                p { class: "ac-settings-saved", "{h}" }
            }
            div { class: "ac-model-settings-toolbar",
                button {
                    r#type: "button",
                    class: "ac-model-toolbar-btn",
                    title: "刷新配置与模型列表",
                    aria_label: "刷新",
                    onclick: move |_| on_refresh.call(()),
                    Icon { icon: LdRefreshCw, width: 14, height: 14, fill: "currentColor" }
                }
                button {
                    r#type: "button",
                    class: "ac-model-add-btn",
                    onclick: move |_| show_add.set(true),
                    Icon { icon: LdPlus, width: 14, height: 14, fill: "currentColor" }
                    span { "添加供应商" }
                }
            }
            div { class: "ac-model-layout",
                aside { class: "ac-model-provider-list",
                    if has_credentials {
                        div { class: "ac-model-provider-group-label", "自定义供应商" }
                        for cred in credentials.iter().cloned() {
                            {
                                let id = cred.id.clone();
                                let name = credential_display_name(&cred);
                                let is_on = enabled_ids.iter().any(|x| x == &id);
                                let is_selected = selected_id() == id;
                                let dot_class = if is_on {
                                    "ac-model-dot is-on"
                                } else {
                                    "ac-model-dot is-off"
                                };
                                rsx! {
                                    button {
                                        key: "{id}",
                                        r#type: "button",
                                        class: if is_selected {
                                            "ac-model-provider-item is-active"
                                        } else {
                                            "ac-model-provider-item"
                                        },
                                        aria_current: is_selected,
                                        onclick: move |_| selected_id.set(id.clone()),
                                        span { class: "ac-model-provider-icon",
                                            Icon { icon: LdBox, width: 14, height: 14, fill: "currentColor" }
                                        }
                                        span { class: "ac-model-provider-name", "{name}" }
                                        span { class: "{dot_class}",
                                            title: if is_on { "已启用" } else { "已停用" },
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "ac-model-provider-empty",
                            "还没有供应商。点击右上角「添加供应商」开始配置。"
                        }
                    }
                }
                section { class: "ac-model-provider-detail",
                    {
                        let selected_now = selected_id();
                        let selected_cred = credentials
                            .iter()
                            .find(|c| c.id == selected_now)
                            .cloned();
                        let is_on = enabled_ids.iter().any(|x| x == &selected_now);
                        rsx! {
                            crate::shell::model_provider_detail::ProviderDetailPane {
                                key: "{selected_now}",
                                cred: selected_cred,
                                is_on,
                                prefs_epoch: prefs_epoch,
                                show_add: show_add,
                                on_update,
                                on_toggle,
                                on_delete,
                            }
                        }
                    }
                }
            }
            if show_add() {
                crate::shell::model_provider_detail::AddProviderSheet {
                    on_add,
                    on_close: move |_| show_add.set(false),
                    on_submit_preset: move |(label, models): (String, Vec<String>)| {
                        pending_defaults.set(Some((label, models)));
                    },
                }
            }
        }
    }
}

/// 连通性检测缓存：`(credential_id, model_id) → ok` 由调用方持有；
/// 此处仅提供按凭证探测的共享逻辑。
pub(crate) type NetStatusMap = HashMap<String, bool>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_loopback_bases_allow_empty_key() {
        for base in [
            "http://127.0.0.1:1234/v1",
            "http://localhost:1234/v1",
            "HTTP://LOCALHOST/v1",
            "http://[::1]:1234/v1",
            "http://192.168.1.10:8080/v1",
            "http://10.0.0.5/v1",
            "http://172.16.0.1/v1",
            "http://172.31.255.1:8000/v1",
        ] {
            assert!(
                is_local_or_private_openai_base(base),
                "expected local/private: {base}"
            );
        }
    }

    #[test]
    fn public_bases_are_not_classified_local() {
        for base in [
            "https://api.openai.com/v1",
            "https://api.anthropic.com/v1",
            "https://openrouter.ai/api/v1",
            "http://8.8.8.8/v1",
            "http://172.15.0.1/v1",
            "http://172.32.0.1/v1",
            "",
        ] {
            assert!(
                !is_local_or_private_openai_base(base),
                "expected public/empty: {base}"
            );
        }
    }

    #[test]
    fn empty_and_sentinel_keys_mask_as_unset() {
        assert_eq!(mask_api_key(""), "未填写");
        assert_eq!(mask_api_key(EMPTY_API_KEY_SENTINEL), "未填写");
        assert_eq!(api_key_for_upsert("  "), EMPTY_API_KEY_SENTINEL);
    }

    #[test]
    fn raw_error_codes_are_not_shown_verbatim() {
        let msg = format_llm_settings_error("api_key_required");
        assert!(!msg.contains("api_key_required"));
        assert!(!msg.is_empty());
        let wrapped = format_llm_settings_error("llm_credential_upsert: api_key_required");
        assert!(!wrapped.contains("api_key_required"));
    }
}
