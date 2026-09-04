//! Web 设置页：多组 OpenAI / Anthropic API Key 与 Base URL（保存到服务端数据库）。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdX;
use dioxus_free_icons::Icon;
use protocol::{LlmConfigDto, LlmCredentialDto, LlmCredentialUpsertBody};

use crate::web::llm_config;
use crate::shell::LlmCredentialsPanel;
use crate::shell::toast::use_toast;
use crate::shell::{
    activate_id_after_toggle, enabled_after_delete, enabled_after_toggle, enabled_after_upsert,
    overlay_enabled_credential_ids,
};

fn apply_cfg(
    cfg: LlmConfigDto,
    mut credentials: Signal<Vec<LlmCredentialDto>>,
    mut active_id: Signal<String>,
    mut enabled_ids: Signal<Vec<String>>,
) {
    let known: Vec<String> = cfg.credentials.iter().map(|c| c.id.clone()).collect();
    let enabled = overlay_enabled_credential_ids(&known, &cfg.active_credential_id);
    credentials.set(cfg.credentials);
    active_id.set(cfg.active_credential_id);
    enabled_ids.set(enabled);
}

/// Web 设置弹窗：模型凭证与关闭。
#[component]
pub fn WebSettingsPage(mut show_settings_modal: Signal<bool>) -> Element {
    let toast = use_toast();
    let credentials = use_signal(Vec::<LlmCredentialDto>::new);
    let active_id = use_signal(String::new);
    let mut enabled_ids = use_signal(Vec::<String>::new);
    let mut load_hint = use_signal(|| None::<String>);
    let refresh = use_context::<crate::shell::LlmModelsRefresh>().0;

    use_effect(move || {
        spawn(async move {
            match llm_config::fetch_llm_config().await {
                Some(cfg) => {
                    apply_cfg(cfg, credentials, active_id, enabled_ids);
                    load_hint.set(None);
                }
                None => load_hint.set(Some("无法从服务端加载配置（请确认网络正常）".into())),
            }
        });
    });

    rsx! {
        div { class: "ac-settings-card ac-settings-card--wide",
            div { class: "ac-settings-head",
                h1 { class: "ac-settings-title", "AI大模型" }
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
                enabled_ids: enabled_ids(),
                hint: load_hint(),
                on_add: move |body: LlmCredentialUpsertBody| {
                    spawn(async move {
                        let prev_ids: Vec<String> =
                            credentials().iter().map(|c| c.id.clone()).collect();
                        let prev_enabled = enabled_ids();
                        match llm_config::upsert_llm_credential(&body).await {
                            Some(cfg) => {
                                let new_ids: Vec<String> =
                                    cfg.credentials.iter().map(|c| c.id.clone()).collect();
                                apply_cfg(cfg, credentials, active_id, enabled_ids);
                                enabled_ids.set(enabled_after_upsert(
                                    &prev_enabled,
                                    &prev_ids,
                                    &new_ids,
                                    refresh,
                                ));
                                load_hint.set(None);
                                toast.success("已添加密钥");
                            }
                            None => load_hint.set(Some("添加失败，请稍后重试".into())),
                        }
                    });
                },
                on_rename: move |(id, label): (String, String)| {
                    spawn(async move {
                        let Some(cred) = credentials().into_iter().find(|c| c.id == id) else {
                            return;
                        };
                        let body = LlmCredentialUpsertBody {
                            id: Some(cred.id),
                            provider: cred.provider,
                            api_key: cred.api_key,
                            openai_v1_base: cred.openai_v1_base,
                            label,
                        };
                        match llm_config::upsert_llm_credential(&body).await {
                            Some(cfg) => {
                                apply_cfg(cfg, credentials, active_id, enabled_ids);
                                load_hint.set(None);
                                toast.success("已更新名称");
                            }
                            None => load_hint.set(Some("更新名称失败，请稍后重试".into())),
                        }
                    });
                },
                on_toggle: move |(id, on): (String, bool)| {
                    spawn(async move {
                        let next = enabled_after_toggle(&enabled_ids(), &id, on, refresh);
                        enabled_ids.set(next.clone());
                        if let Some(activate) =
                            activate_id_after_toggle(&next, &active_id(), &id, on)
                        {
                            match llm_config::activate_llm_credential(&activate).await {
                                Some(cfg) => {
                                    apply_cfg(cfg, credentials, active_id, enabled_ids);
                                    enabled_ids.set(next);
                                    load_hint.set(None);
                                }
                                None => load_hint.set(Some("切换失败，请稍后重试".into())),
                            }
                        }
                    });
                },
                on_delete: move |id: String| {
                    spawn(async move {
                        let prev_enabled = enabled_ids();
                        match llm_config::delete_llm_credential(&id).await {
                            Some(cfg) => {
                                let known: Vec<String> =
                                    cfg.credentials.iter().map(|c| c.id.clone()).collect();
                                apply_cfg(cfg, credentials, active_id, enabled_ids);
                                enabled_ids.set(enabled_after_delete(&prev_enabled, &known, refresh));
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
