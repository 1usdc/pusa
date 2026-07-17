//! Web 设置页：OpenAI API Key 与 Base URL（保存到服务端数据库）。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdX;
use dioxus_free_icons::Icon;

use crate::web::llm_config;
use crate::web::password_field::{AcPasswordInput, PasswordFieldStyle};
use crate::shell::toast::use_toast;

/// Web 设置弹窗：模型凭证与关闭。
#[component]
pub fn WebSettingsPage(mut show_settings_modal: Signal<bool>) -> Element {
    let toast = use_toast();
    let mut api_key = use_signal(String::new);
    let mut base_url = use_signal(|| llm_config::DEFAULT_OPENAI_V1_BASE.to_string());
    let mut key_saved_on_server = use_signal(|| false);
    let mut config_load_failed = use_signal(|| false);
    let mut saved_hint = use_signal(|| None::<String>);

    use_effect(move || {
        spawn(async move {
            match llm_config::fetch_llm_config().await {
                Some(cfg) => {
                    base_url.set(cfg.openai_v1_base);
                    key_saved_on_server.set(cfg.api_key_configured);
                    if !cfg.api_key.trim().is_empty() {
                        api_key.set(cfg.api_key);
                    }
                    config_load_failed.set(false);
                }
                None => config_load_failed.set(true),
            }
        });
    });

    rsx! {
        div { class: "ac-settings-card",
            div { class: "ac-settings-head",
                h1 { class: "ac-settings-title", "设置" }
                button {
                    r#type: "button",
                    class: "ac-settings-close",
                    title: "关闭",
                    aria_label: "关闭",
                    onclick: move |_| show_settings_modal.set(false),
                    Icon {
                        icon: LdX,
                        width: 18,
                        height: 18,
                        fill: "currentColor",
                        class: "ac-settings-close-icon",
                    }
                }
            }
            p { class: "ac-settings-lead",
                "用于对话补全的 OpenAI 兼容接口。密钥与 Base URL 会保存到服务端数据库；刷新后将从服务端自动回填。"
            }
            if config_load_failed() {
                p { class: "ac-settings-saved",
                    "无法从服务端加载配置（请确认网络正常）。下方 Base URL 可能不是已保存的值。"
                }
            }
            label { class: "ac-settings-label", "API Key"
                AcPasswordInput {
                    placeholder: "sk-…",
                    value: api_key,
                    variant: PasswordFieldStyle::Settings,
                }
                if key_saved_on_server() && api_key().trim().is_empty() {
                    p { class: "ac-settings-hint ac-settings-hint--ok",
                        "服务端已保存密钥，正在加载…"
                    }
                }
            }
            label { class: "ac-settings-label",
                "API Base URL（须以 /v1 结尾，可与 OpenAI 官方或代理一致）"
                input {
                    r#type: "text",
                    class: "ac-settings-input",
                    placeholder: "{llm_config::DEFAULT_OPENAI_V1_BASE}",
                    value: "{base_url()}",
                    oninput: move |e| base_url.set(e.value()),
                }
            }
            if let Some(ref h) = saved_hint() {
                p { class: "ac-settings-saved", "{h}" }
            }
            div { class: "ac-settings-actions",
                button {
                    r#type: "button",
                    class: "ac-settings-submit",
                    onclick: move |_| {
                        let k = api_key().trim().to_string();
                        let b = base_url().trim().to_string();
                        let clear_k = k.is_empty();
                        let clear_b = b.is_empty();
                        spawn(async move {
                            if let Some(cfg) = llm_config::save_llm_config_clearing(
                                &k, &b, clear_k, clear_b,
                            ).await {
                                if clear_k {
                                    api_key.set(String::new());
                                } else {
                                    api_key.set(cfg.api_key);
                                }
                                if clear_b {
                                    base_url.set(String::new());
                                } else {
                                    base_url.set(cfg.openai_v1_base);
                                }
                                key_saved_on_server.set(cfg.api_key_configured);
                                saved_hint.set(None);
                                show_settings_modal.set(false);
                                toast.success("设置已保存");
                            } else {
                                saved_hint.set(Some("保存失败，请稍后重试".into()));
                            }
                        });
                    },
                    "保存"
                }
            }
        }
    }
}
