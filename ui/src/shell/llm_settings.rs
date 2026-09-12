//! 设置弹窗：多组 OpenAI / Anthropic API Key + Base URL。

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdTrash2;
use dioxus_free_icons::Icon;
use protocol::{LlmApiProvider, LlmCredentialDto, LlmCredentialUpsertBody};

/// 核心库 `llm_credential_upsert` 拒绝空 `api_key`（错误码 `api_key_required`）。
/// 选填密钥时写入此占位值；界面展示为「未填写」。
const EMPTY_API_KEY_SENTINEL: &str = "__pusa_no_key__";

/// 从 Base URL 取出 host（支持 `http://host:port/path`、`[IPv6]`、无 scheme）。
fn host_from_openai_base(base: &str) -> Option<String> {
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

fn is_local_or_private_openai_base(base: &str) -> bool {
    host_from_openai_base(base).is_some_and(|h| is_loopback_or_private_host(&h))
}

fn is_unspecified_api_key(key: &str) -> bool {
    let t = key.trim();
    t.is_empty() || t == EMPTY_API_KEY_SENTINEL
}

fn api_key_for_upsert(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        EMPTY_API_KEY_SENTINEL.to_string()
    } else {
        t.to_string()
    }
}

fn mask_api_key(key: &str) -> String {
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

fn provider_label(p: LlmApiProvider) -> &'static str {
    match p {
        LlmApiProvider::Openai => "OpenAI",
        LlmApiProvider::Anthropic => "Anthropic",
    }
}

fn display_credential_name(cred: &LlmCredentialDto) -> String {
    display_name(cred.provider, &cred.label)
}

fn display_name(provider: LlmApiProvider, label: &str) -> String {
    let label = label.trim();
    if label.is_empty() {
        provider_label(provider).to_string()
    } else {
        label.to_string()
    }
}

/// 有改动才返回要写入 DTO `label` 的新名称。
fn renamed_label(draft: &str, stored_label: &str, provider: LlmApiProvider) -> Option<String> {
    let next = draft.trim();
    if next.is_empty() {
        return None;
    }
    if next == display_name(provider, stored_label) {
        None
    } else {
        Some(next.to_string())
    }
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
        && code
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '_')
        && code.contains('_')
    {
        return "操作失败，请稍后重试。".into();
    }
    t.to_string()
}

fn reset_add_form(
    mut adding: Signal<bool>,
    mut add_provider: Signal<LlmApiProvider>,
    mut add_label: Signal<String>,
    mut add_key: Signal<String>,
    mut add_base: Signal<String>,
) {
    adding.set(false);
    add_provider.set(LlmApiProvider::Openai);
    add_label.set(String::new());
    add_key.set(String::new());
    add_base.set(LlmApiProvider::Openai.default_v1_base().to_string());
}

/// 多凭证列表 + 折叠的添加表单。
#[component]
pub fn LlmCredentialsPanel(
    credentials: Vec<LlmCredentialDto>,
    enabled_ids: Vec<String>,
    hint: Option<String>,
    on_add: EventHandler<LlmCredentialUpsertBody>,
    on_rename: EventHandler<(String, String)>,
    on_toggle: EventHandler<(String, bool)>,
    on_delete: EventHandler<String>,
) -> Element {
    let mut adding = use_signal(|| false);
    let mut add_provider = use_signal(|| LlmApiProvider::Openai);
    let mut add_label = use_signal(String::new);
    let mut add_key = use_signal(String::new);
    let mut add_base = use_signal(|| LlmApiProvider::Openai.default_v1_base().to_string());

    let provider = add_provider();
    let local_or_private_base = is_local_or_private_openai_base(&add_base());

    rsx! {
        div { class: "ac-llm-cred-panel",
            p { class: "ac-settings-lead",
                "可添加多组端点。API Key 可留空。打开开关的端点会同时启用，模型列表合并这些地址。"
            }
            if let Some(h) = hint.filter(|s| !s.trim().is_empty()) {
                p { class: "ac-settings-saved", "{h}" }
            }

            if !credentials.is_empty() {
                p { class: "ac-settings-label", "已保存的密钥" }
                ul { class: "ac-llm-cred-list",
                    for cred in credentials.clone() {
                        {
                            let id = cred.id.clone();
                            let is_on = enabled_ids.iter().any(|x| x == &id);
                            rsx! {
                                LlmCredentialCard {
                                    key: "{id}",
                                    cred,
                                    is_on,
                                    on_rename,
                                    on_toggle,
                                    on_delete,
                                }
                            }
                        }
                    }
                }
            }

            div { class: "ac-llm-cred-foot",
                if !adding() {
                    button {
                        r#type: "button",
                        class: "ac-settings-add-btn",
                        onclick: move |_| {
                            add_provider.set(LlmApiProvider::Openai);
                            add_label.set(String::new());
                            add_key.set(String::new());
                            add_base.set(LlmApiProvider::Openai.default_v1_base().to_string());
                            adding.set(true);
                        },
                        "添加密钥"
                    }
                } else {
                    p { class: "ac-settings-label", "添加密钥" }
                    div { class: "ac-settings-provider", role: "tablist", aria_label: "API 类型",
                        button {
                            r#type: "button",
                            role: "tab",
                            class: if provider == LlmApiProvider::Openai {
                                "ac-settings-provider-btn is-active"
                            } else {
                                "ac-settings-provider-btn"
                            },
                            aria_selected: provider == LlmApiProvider::Openai,
                            onclick: move |_| {
                                if add_provider() == LlmApiProvider::Openai {
                                    return;
                                }
                                add_provider.set(LlmApiProvider::Openai);
                                if add_base().trim().is_empty()
                                    || add_base().trim() == LlmApiProvider::Anthropic.default_v1_base()
                                {
                                    add_base.set(LlmApiProvider::Openai.default_v1_base().to_string());
                                }
                            },
                            "OpenAI"
                        }
                        button {
                            r#type: "button",
                            role: "tab",
                            class: if provider == LlmApiProvider::Anthropic {
                                "ac-settings-provider-btn is-active"
                            } else {
                                "ac-settings-provider-btn"
                            },
                            aria_selected: provider == LlmApiProvider::Anthropic,
                            onclick: move |_| {
                                if add_provider() == LlmApiProvider::Anthropic {
                                    return;
                                }
                                add_provider.set(LlmApiProvider::Anthropic);
                                if add_base().trim().is_empty()
                                    || add_base().trim() == LlmApiProvider::Openai.default_v1_base()
                                {
                                    add_base.set(LlmApiProvider::Anthropic.default_v1_base().to_string());
                                }
                            },
                            "Anthropic"
                        }
                    }
                    label { class: "ac-settings-label", "名称"
                        input {
                            r#type: "text",
                            class: "ac-settings-input",
                            placeholder: "{provider_label(provider)}",
                            value: "{add_label()}",
                            oninput: move |e| add_label.set(e.value()),
                        }
                    }
                    label { class: "ac-settings-label", "API Key（可选）"
                        input {
                            r#type: "password",
                            class: "ac-settings-input",
                            placeholder: if local_or_private_base {
                                "可留空（本地 / 内网）"
                            } else {
                                "可留空"
                            },
                            value: "{add_key()}",
                            oninput: move |e| add_key.set(e.value()),
                        }
                    }
                    label { class: "ac-settings-label", "Base URL（须以 /v1 结尾）"
                        input {
                            r#type: "text",
                            class: "ac-settings-input",
                            placeholder: "{provider.default_v1_base()}",
                            value: "{add_base()}",
                            oninput: move |e| add_base.set(e.value()),
                        }
                    }
                    div { class: "ac-settings-actions ac-settings-actions--row",
                        button {
                            r#type: "button",
                            class: "ac-settings-cancel",
                            onclick: move |_| {
                                reset_add_form(adding, add_provider, add_label, add_key, add_base);
                            },
                            "取消"
                        }
                        button {
                            r#type: "button",
                            class: "ac-settings-submit",
                            onclick: move |_| {
                                let provider = add_provider();
                                let api_key = api_key_for_upsert(&add_key());
                                let openai_v1_base = if add_base().trim().is_empty() {
                                    provider.default_v1_base().to_string()
                                } else {
                                    add_base().trim().to_string()
                                };
                                let label = {
                                    let t = add_label().trim().to_string();
                                    if t.is_empty() {
                                        provider_label(provider).to_string()
                                    } else {
                                        t
                                    }
                                };
                                on_add.call(LlmCredentialUpsertBody {
                                    id: None,
                                    provider,
                                    api_key,
                                    openai_v1_base,
                                    label,
                                });
                                reset_add_form(adding, add_provider, add_label, add_key, add_base);
                            },
                            "添加"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LlmCredentialCard(
    cred: LlmCredentialDto,
    is_on: bool,
    on_rename: EventHandler<(String, String)>,
    on_toggle: EventHandler<(String, bool)>,
    on_delete: EventHandler<String>,
) -> Element {
    let id_toggle = cred.id.clone();
    let id_delete = cred.id.clone();
    let id_rename_enter = cred.id.clone();
    let id_rename_blur = cred.id.clone();
    let provider = cred.provider;
    let stored_label = cred.label.clone();
    let stored_label_enter = stored_label.clone();
    let stored_label_blur = stored_label;
    let saved_name = display_credential_name(&cred);
    let mut name_draft = use_signal(|| saved_name.clone());
    let masked = mask_api_key(&cred.api_key);
    let base = cred.openai_v1_base.clone();
    let switch_title = if is_on { "关闭此密钥" } else { "开启此密钥" };

    rsx! {
        li {
            class: if is_on {
                "ac-llm-cred-card is-on"
            } else {
                "ac-llm-cred-card"
            },
            div { class: "ac-llm-cred-main",
                div { class: "ac-llm-cred-top",
                    input {
                        r#type: "text",
                        class: "ac-llm-cred-provider-input",
                        value: "{name_draft()}",
                        aria_label: "密钥名称",
                        title: "点击修改名称",
                        oninput: move |e| name_draft.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() != Key::Enter {
                                return;
                            }
                            if name_draft().trim().is_empty() {
                                name_draft.set(display_name(provider, &stored_label_enter));
                                return;
                            }
                            if let Some(next) =
                                renamed_label(&name_draft(), &stored_label_enter, provider)
                            {
                                on_rename.call((id_rename_enter.clone(), next));
                            }
                        },
                        onblur: move |_| {
                            if name_draft().trim().is_empty() {
                                name_draft.set(display_name(provider, &stored_label_blur));
                                return;
                            }
                            if let Some(next) =
                                renamed_label(&name_draft(), &stored_label_blur, provider)
                            {
                                on_rename.call((id_rename_blur.clone(), next));
                            }
                        },
                    }
                }
                div { class: "ac-llm-cred-key", "{masked}" }
                div { class: "ac-llm-cred-base", "{base}" }
            }
            div { class: "ac-llm-cred-actions",
                button {
                    r#type: "button",
                    class: "ac-llm-cred-icon-btn ac-llm-cred-icon-btn--danger",
                    title: "删除",
                    aria_label: "删除",
                    onclick: move |_| on_delete.call(id_delete.clone()),
                    Icon {
                        icon: LdTrash2,
                        width: 15,
                        height: 15,
                        fill: "currentColor",
                    }
                }
                button {
                    r#type: "button",
                    class: if is_on {
                        "ac-llm-switch is-on"
                    } else {
                        "ac-llm-switch"
                    },
                    role: "switch",
                    aria_checked: is_on,
                    title: "{switch_title}",
                    aria_label: "{switch_title}",
                    onclick: move |_| on_toggle.call((id_toggle.clone(), !is_on)),
                    span { class: "ac-llm-switch-track",
                        span { class: "ac-llm-switch-knob" }
                    }
                }
            }
        }
    }
}

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
            "http://[0:0:0:0:0:0:0:1]/v1",
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

    #[test]
    fn renamed_label_skips_unchanged_provider_fallback() {
        assert_eq!(renamed_label("OpenAI", "", LlmApiProvider::Openai), None);
        assert_eq!(
            renamed_label("LM Studio", "", LlmApiProvider::Openai),
            Some("LM Studio".into())
        );
    }
}
