//! 设置弹窗：多组 OpenAI / Anthropic API Key + Base URL。

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdPencil, LdTrash2};
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
#[cfg(test)]
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
        && code.chars().all(|c| c.is_ascii_lowercase() || c == '_')
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
    on_add: EventHandler<LlmCredentialUpsertBody>,
    on_update: EventHandler<LlmCredentialUpsertBody>,
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
        div { class: crate::shell::style::MODAL_BODY_FLEX_COL,
            p { class: "m-0 mb-5 text-[0.8rem] leading-[1.5] text-secondary-3 shrink-0",
                "可添加多组AI大模型端点，打开的端点会同时启用，模型列表可切换选择这些模型。"
            }

            if !credentials.is_empty() {
                p { class: crate::shell::style::FIELD_LABEL, "已保存的密钥" }
                ul { class: crate::shell::style::CRED_LIST,
                    for cred in credentials.clone() {
                        {
                            let id = cred.id.clone();
                            let is_on = enabled_ids.iter().any(|x| x == &id);
                            rsx! {
                                LlmCredentialCard {
                                    key: "{id}",
                                    cred,
                                    is_on,
                                    on_update,
                                    on_toggle,
                                    on_delete,
                                }
                            }
                        }
                    }
                }
            }

            div { class: crate::shell::style::CRED_FOOT,
                if !adding() {
                    button {
                        r#type: "button",
                        class: crate::shell::style::BTN_ADD_PILL,
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
                    p { class: crate::shell::style::FIELD_LABEL, "添加密钥" }
                    div { class: crate::shell::style::PROVIDER_TABS, role: "tablist", aria_label: "API 类型",
                        button {
                            r#type: "button",
                            role: "tab",
                            class: crate::shell::style::PROVIDER_TAB,
                            "data-active": provider == LlmApiProvider::Openai,
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
                            class: crate::shell::style::PROVIDER_TAB,
                            "data-active": provider == LlmApiProvider::Anthropic,
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
                    label { class: crate::shell::style::FIELD_LABEL, "名称"
                        input {
                            r#type: "text",
                            class: crate::shell::style::FIELD_INPUT,
                            placeholder: "{provider_label(provider)}",
                            value: "{add_label()}",
                            oninput: move |e| add_label.set(e.value()),
                        }
                    }
                    label { class: crate::shell::style::FIELD_LABEL, "API Key（可选）"
                        input {
                            r#type: "password",
                            class: crate::shell::style::FIELD_INPUT,
                            placeholder: if local_or_private_base {
                                "可留空（本地 / 内网）"
                            } else {
                                "可留空"
                            },
                            value: "{add_key()}",
                            oninput: move |e| add_key.set(e.value()),
                        }
                    }
                    label { class: crate::shell::style::FIELD_LABEL, "Base URL（以 /v1 结尾）"
                        input {
                            r#type: "text",
                            class: crate::shell::style::FIELD_INPUT,
                            placeholder: "{provider.default_v1_base()}",
                            value: "{add_base()}",
                            oninput: move |e| add_base.set(e.value()),
                        }
                    }
                    div { class: crate::shell::style::FIELD_ACTIONS_ROW,
                        button {
                            r#type: "button",
                            class: crate::shell::style::BTN_CANCEL,
                            onclick: move |_| {
                                reset_add_form(adding, add_provider, add_label, add_key, add_base);
                            },
                            "取消"
                        }
                        button {
                            r#type: "button",
                            class: crate::shell::style::BTN_SUBMIT,
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
    on_update: EventHandler<LlmCredentialUpsertBody>,
    on_toggle: EventHandler<(String, bool)>,
    on_delete: EventHandler<String>,
) -> Element {
    let id_toggle = cred.id.clone();
    let id_delete = cred.id.clone();
    let saved_name = display_credential_name(&cred);
    let masked = mask_api_key(&cred.api_key);
    let base = cred.openai_v1_base.clone();
    let switch_title = if is_on {
        "关闭此密钥"
    } else {
        "开启此密钥"
    };
    let mut editing = use_signal(|| false);
    let mut edit_provider = use_signal(|| cred.provider);
    let mut edit_label = use_signal(|| cred.label.clone());
    let mut edit_key = use_signal(String::new);
    let mut edit_base = use_signal(|| cred.openai_v1_base.clone());
    let seed_provider = cred.provider;
    let seed_label = cred.label.clone();
    let seed_base = cred.openai_v1_base.clone();
    let kept_api_key = cred.api_key.clone();
    let stored_label_save = cred.label.clone();

    rsx! {
        li {
            class: crate::shell::style::CRED_CARD,
            "data-on": is_on,
            div { class: crate::shell::style::CRED_ROW,
            div { class: crate::shell::style::CRED_MAIN,
                div { class: crate::shell::style::CRED_TOP,
                    span { class: crate::shell::style::CRED_NAME, "{saved_name}" }
                }
                div { class: crate::shell::style::CRED_KEY, "{masked}" }
                div { class: crate::shell::style::CRED_BASE, "{base}" }
            }
            div { class: crate::shell::style::CRED_ACTIONS,
                if editing() {
                    button {
                        r#type: "button",
                        class: crate::shell::style::CRED_ICON_BTN_DANGER,
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
                } else {
                    button {
                        r#type: "button",
                        class: crate::shell::style::CRED_ICON_BTN,
                        title: "编辑密钥",
                        aria_label: "编辑密钥",
                        onclick: move |_| {
                            edit_provider.set(seed_provider);
                            edit_label.set(seed_label.clone());
                            edit_key.set(String::new());
                            edit_base.set(seed_base.clone());
                            editing.set(true);
                        },
                        Icon {
                            icon: LdPencil,
                            width: 15,
                            height: 15,
                            fill: "currentColor",
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "group m-0 p-0 border-0 bg-transparent inline-flex items-center cursor-pointer",
                    "data-on": is_on,
                    role: "switch",
                    aria_checked: is_on,
                    title: "{switch_title}",
                    aria_label: "{switch_title}",
                    onclick: move |_| on_toggle.call((id_toggle.clone(), !is_on)),
                    span { class: "relative w-[2.15rem] h-[1.2rem] rounded-full bg-primary-2 shrink-0 transition-[background] duration-200 group-data-[on=true]:bg-[var(--button-primary)]",
                        span { class: "absolute top-[2px] left-[2px] w-[calc(1.2rem_-_4px)] h-[calc(1.2rem_-_4px)] rounded-full bg-primary shadow-[0_1px_4px_var(--shadow-color)] transition-transform duration-200 group-data-[on=true]:translate-x-[0.92rem]" }
                    }
                }
            }
            }
            if editing() {
                {
                    let provider_now = edit_provider();
                    let local_or_private_base = is_local_or_private_openai_base(&edit_base());
                    let id_save = cred.id.clone();
                    rsx! {
                        div { class: crate::shell::style::CRED_EDIT,
                            div { class: crate::shell::style::PROVIDER_TABS_TIGHT, role: "tablist", aria_label: "API 类型",
                                button {
                                    r#type: "button",
                                    role: "tab",
                                    class: crate::shell::style::PROVIDER_TAB,
                                    "data-active": provider_now == LlmApiProvider::Openai,
                                    aria_selected: provider_now == LlmApiProvider::Openai,
                                    onclick: move |_| {
                                        if edit_provider() == LlmApiProvider::Openai {
                                            return;
                                        }
                                        edit_provider.set(LlmApiProvider::Openai);
                                        if edit_base().trim().is_empty()
                                            || edit_base().trim() == LlmApiProvider::Anthropic.default_v1_base()
                                        {
                                            edit_base.set(LlmApiProvider::Openai.default_v1_base().to_string());
                                        }
                                    },
                                    "OpenAI"
                                }
                                button {
                                    r#type: "button",
                                    role: "tab",
                                    class: crate::shell::style::PROVIDER_TAB,
                                    "data-active": provider_now == LlmApiProvider::Anthropic,
                                    aria_selected: provider_now == LlmApiProvider::Anthropic,
                                    onclick: move |_| {
                                        if edit_provider() == LlmApiProvider::Anthropic {
                                            return;
                                        }
                                        edit_provider.set(LlmApiProvider::Anthropic);
                                        if edit_base().trim().is_empty()
                                            || edit_base().trim() == LlmApiProvider::Openai.default_v1_base()
                                        {
                                            edit_base.set(LlmApiProvider::Anthropic.default_v1_base().to_string());
                                        }
                                    },
                                    "Anthropic"
                                }
                            }
                            label { class: crate::shell::style::FIELD_LABEL_TIGHT, "名称"
                                input {
                                    r#type: "text",
                                    class: crate::shell::style::FIELD_INPUT,
                                    placeholder: "{provider_label(provider_now)}",
                                    value: "{edit_label()}",
                                    oninput: move |e| edit_label.set(e.value()),
                                }
                            }
                            label { class: crate::shell::style::FIELD_LABEL_TIGHT, "API Key（可选）"
                                input {
                                    r#type: "password",
                                    class: crate::shell::style::FIELD_INPUT,
                                    placeholder: if local_or_private_base {
                                        "留空则不修改（本地 / 内网）"
                                    } else {
                                        "留空则不修改"
                                    },
                                    value: "{edit_key()}",
                                    oninput: move |e| edit_key.set(e.value()),
                                }
                            }
                            label { class: crate::shell::style::FIELD_LABEL_TIGHT, "Base URL（以 /v1 结尾）"
                                input {
                                    r#type: "text",
                                    class: crate::shell::style::FIELD_INPUT,
                                    placeholder: "{provider_now.default_v1_base()}",
                                    value: "{edit_base()}",
                                    oninput: move |e| edit_base.set(e.value()),
                                }
                            }
                            div { class: crate::shell::style::FIELD_ACTIONS_ROW_TIGHT,
                                button {
                                    r#type: "button",
                                    class: crate::shell::style::BTN_CANCEL,
                                    onclick: move |_| editing.set(false),
                                    "取消"
                                }
                                button {
                                    r#type: "button",
                                    class: crate::shell::style::BTN_SUBMIT,
                                    onclick: move |_| {
                                        let provider = edit_provider();
                                        let typed_key = edit_key().trim().to_string();
                                        let api_key = if typed_key.is_empty() {
                                            kept_api_key.clone()
                                        } else {
                                            typed_key
                                        };
                                        let openai_v1_base = if edit_base().trim().is_empty() {
                                            provider.default_v1_base().to_string()
                                        } else {
                                            edit_base().trim().to_string()
                                        };
                                        let label = {
                                            let t = edit_label().trim().to_string();
                                            if t.is_empty() {
                                                stored_label_save.clone()
                                            } else {
                                                t
                                            }
                                        };
                                        on_update.call(LlmCredentialUpsertBody {
                                            id: Some(id_save.clone()),
                                            provider,
                                            api_key,
                                            openai_v1_base,
                                            label,
                                        });
                                        editing.set(false);
                                    },
                                    "保存"
                                }
                            }
                        }
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
