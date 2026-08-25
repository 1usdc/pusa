//! 设置弹窗：多组 OpenAI / Anthropic API Key + Base URL。

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdTrash2;
use dioxus_free_icons::Icon;
use protocol::{LlmApiProvider, LlmCredentialDto, LlmCredentialUpsertBody};

fn mask_api_key(key: &str) -> String {
    let t = key.trim();
    if t.is_empty() {
        return "未填写".into();
    }
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

fn reset_add_form(
    mut adding: Signal<bool>,
    mut add_provider: Signal<LlmApiProvider>,
    mut add_key: Signal<String>,
    mut add_base: Signal<String>,
) {
    adding.set(false);
    add_provider.set(LlmApiProvider::Openai);
    add_key.set(String::new());
    add_base.set(LlmApiProvider::Openai.default_v1_base().to_string());
}

/// 多凭证列表 + 折叠的添加表单。
#[component]
pub fn LlmCredentialsPanel(
    credentials: Vec<LlmCredentialDto>,
    active_id: String,
    hint: Option<String>,
    on_add: EventHandler<LlmCredentialUpsertBody>,
    on_activate: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    let mut adding = use_signal(|| false);
    let mut add_provider = use_signal(|| LlmApiProvider::Openai);
    let mut add_key = use_signal(String::new);
    let mut add_base = use_signal(|| LlmApiProvider::Openai.default_v1_base().to_string());

    let provider = add_provider();
    let can_add = !add_key().trim().is_empty();

    rsx! {
        p { class: "ac-settings-lead",
            "可添加多组密钥。对话使用当前启用的一组。"
        }
        if let Some(h) = hint {
            p { class: "ac-settings-saved", "{h}" }
        }

        if !credentials.is_empty() {
            p { class: "ac-settings-label", "已保存的密钥" }
            ul { class: "ac-llm-cred-list",
                for cred in credentials.clone() {
                    {
                        let id = cred.id.clone();
                        let id_activate = id.clone();
                        let id_delete = id.clone();
                        let is_active = cred.id == active_id;
                        let name = provider_label(cred.provider);
                        let masked = mask_api_key(&cred.api_key);
                        let base = cred.openai_v1_base.clone();
                        rsx! {
                            li {
                                key: "{id}",
                                class: if is_active {
                                    "ac-llm-cred-card is-active"
                                } else {
                                    "ac-llm-cred-card"
                                },
                                div { class: "ac-llm-cred-main",
                                    div { class: "ac-llm-cred-top",
                                        span { class: "ac-llm-cred-provider", "{name}" }
                                        if is_active {
                                            span { class: "ac-llm-cred-badge", "使用中" }
                                        } else {
                                            button {
                                                r#type: "button",
                                                class: "ac-llm-cred-enable",
                                                onclick: move |_| on_activate.call(id_activate.clone()),
                                                "启用"
                                            }
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
                                }
                            }
                        }
                    }
                }
            }
        }

        if !adding() {
            button {
                r#type: "button",
                class: "ac-settings-add-btn",
                onclick: move |_| {
                    add_provider.set(LlmApiProvider::Openai);
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
            label { class: "ac-settings-label", "API Key"
                input {
                    r#type: "password",
                    class: "ac-settings-input",
                    placeholder: "{provider.key_placeholder()}",
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
                        reset_add_form(adding, add_provider, add_key, add_base);
                    },
                    "取消"
                }
                button {
                    r#type: "button",
                    class: "ac-settings-submit",
                    disabled: !can_add,
                    onclick: move |_| {
                        let provider = add_provider();
                        let api_key = add_key().trim().to_string();
                        if api_key.is_empty() {
                            return;
                        }
                        let openai_v1_base = if add_base().trim().is_empty() {
                            provider.default_v1_base().to_string()
                        } else {
                            add_base().trim().to_string()
                        };
                        on_add.call(LlmCredentialUpsertBody {
                            id: None,
                            provider,
                            api_key,
                            openai_v1_base,
                            label: String::new(),
                        });
                        reset_add_form(adding, add_provider, add_key, add_base);
                    },
                    "添加"
                }
            }
        }
    }
}
