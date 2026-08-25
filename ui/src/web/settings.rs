//! Web 设置页：多组 OpenAI / Anthropic API Key 与 Base URL（保存到服务端数据库）。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdX;
use dioxus_free_icons::Icon;
use protocol::{LlmConfigDto, LlmCredentialDto, LlmCredentialUpsertBody};

use crate::web::llm_config;
use crate::shell::LlmCredentialsPanel;
use crate::shell::toast::use_toast;

fn apply_cfg(
    cfg: LlmConfigDto,
    mut credentials: Signal<Vec<LlmCredentialDto>>,
    mut active_id: Signal<String>,
) {
    credentials.set(cfg.credentials);
    active_id.set(cfg.active_credential_id);
}

/// Web 设置弹窗：模型凭证与关闭。
#[component]
pub fn WebSettingsPage(mut show_settings_modal: Signal<bool>) -> Element {
    let toast = use_toast();
    let credentials = use_signal(Vec::<LlmCredentialDto>::new);
    let active_id = use_signal(String::new);
    let mut load_hint = use_signal(|| None::<String>);

    use_effect(move || {
        spawn(async move {
            match llm_config::fetch_llm_config().await {
                Some(cfg) => {
                    apply_cfg(cfg, credentials, active_id);
                    load_hint.set(None);
                }
                None => load_hint.set(Some("无法从服务端加载配置（请确认网络正常）".into())),
            }
        });
    });

    rsx! {
        div { class: "ac-settings-card ac-settings-card--wide",
            div { class: "ac-settings-head",
                h1 { class: "ac-settings-title", "API Key" }
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
            LlmCredentialsPanel {
                credentials: credentials(),
                active_id: active_id(),
                hint: load_hint(),
                on_add: move |body: LlmCredentialUpsertBody| {
                    spawn(async move {
                        match llm_config::upsert_llm_credential(&body).await {
                            Some(cfg) => {
                                apply_cfg(cfg, credentials, active_id);
                                load_hint.set(None);
                                toast.success("已添加密钥");
                            }
                            None => load_hint.set(Some("添加失败，请稍后重试".into())),
                        }
                    });
                },
                on_activate: move |id: String| {
                    spawn(async move {
                        match llm_config::activate_llm_credential(&id).await {
                            Some(cfg) => {
                                apply_cfg(cfg, credentials, active_id);
                                load_hint.set(None);
                                toast.success("已切换当前密钥");
                            }
                            None => load_hint.set(Some("切换失败，请稍后重试".into())),
                        }
                    });
                },
                on_delete: move |id: String| {
                    spawn(async move {
                        match llm_config::delete_llm_credential(&id).await {
                            Some(cfg) => {
                                apply_cfg(cfg, credentials, active_id);
                                load_hint.set(None);
                                toast.success("已删除密钥");
                            }
                            None => load_hint.set(Some("删除失败，请稍后重试".into())),
                        }
                    });
                },
            }
        }
    }
}
