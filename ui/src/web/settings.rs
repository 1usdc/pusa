//! Web 设置整页：标准「左侧菜单 + 右侧内容」布局。
//!
//! 左侧菜单：通用 / 模型设置 / 关于；右侧按选中分区渲染对应卡片。
//! 模型设置（多组供应商：Base URL / API 格式 / API Key / 模型列表）保存到服务端数据库。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

use dioxus::prelude::*;
use protocol::{LlmConfigDto, LlmCredentialDto, LlmCredentialUpsertBody};

use crate::shell::{
    activate_id_after_toggle, enabled_after_delete, enabled_after_toggle, enabled_after_upsert,
    overlay_enabled_credential_ids, LlmModelsRefresh, ModelSettingsPanel, SettingsInfoRow,
    SettingsNavShell, SettingsSection, ThemePickerCard,
};
use crate::web::llm_config;
use crate::shell::toast::use_toast;

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

/// Web 设置整页：左侧菜单（通用 / 模型设置 / 关于）+ 右侧内容。
#[component]
pub fn WebSettingsPage(mut show_settings_modal: Signal<bool>) -> Element {
    let toast = use_toast();
    let credentials = use_signal(Vec::<LlmCredentialDto>::new);
    let active_id = use_signal(String::new);
    let mut enabled_ids = use_signal(Vec::<String>::new);
    let mut load_hint = use_signal(|| None::<String>);
    let refresh = use_context::<crate::shell::LlmModelsRefresh>().0;

    // 左侧菜单当前分区；打开时默认进入模型设置。
    let mut section = use_signal(|| SettingsSection::ModelSettings);

    // 通用分区：主题分段选择（与标题栏快切按钮共享持久化）。
    let ui_theme = use_signal(crate::shell::theme::load);

    // 关于分区：懒加载的版本 / 系统 / 架构信息。
    let mut about_rows = use_signal(Vec::<SettingsInfoRow>::new);
    let mut about_error = use_signal(|| None::<String>);
    let mut about_loaded = use_signal(|| false);

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

    // 首次切到「关于」分区时拉取版本 / 系统 / UA。
    use_effect(move || {
        if section() != SettingsSection::About || about_loaded() {
            return;
        }
        about_loaded.set(true);
        spawn(async move {
            match crate::chat::transport_wasm::load_about_info(&crate::chat::api_base_url()).await {
                Ok(dto) => {
                    about_rows.set(vec![
                        SettingsInfoRow {
                            label: "版本号".into(),
                            value: dto.version,
                        },
                        SettingsInfoRow {
                            label: "系统".into(),
                            value: dto.runtime_os,
                        },
                        SettingsInfoRow {
                            label: "架构".into(),
                            value: dto.runtime_arch,
                        },
                    ]);
                    about_error.set(None);
                }
                Err(e) => about_error.set(Some(e.to_string())),
            }
        });
    });

    rsx! {
        SettingsNavShell {
            active: section,
            on_close: move |_| show_settings_modal.set(false),
            if section() == SettingsSection::General {
                ThemePickerCard { ui_theme }
            }
            if section() == SettingsSection::ModelSettings {
                ModelSettingsPanel {
                    credentials: credentials(),
                    enabled_ids: enabled_ids(),
                    hint: load_hint(),
                    on_refresh: move |_| {
                        spawn(async move {
                            match llm_config::fetch_llm_config().await {
                                Some(cfg) => {
                                    apply_cfg(cfg, credentials, active_id, enabled_ids);
                                    load_hint.set(None);
                                    toast.success("已刷新");
                                }
                                None => load_hint.set(Some("刷新失败，请稍后重试".into())),
                            }
                        });
                    },
                    on_update: move |body: LlmCredentialUpsertBody| {
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
                                    toast.success("已保存供应商");
                                }
                                None => load_hint.set(Some("更新失败，请稍后重试".into())),
                            }
                        });
                    },
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
                                    toast.success("已添加供应商");
                                }
                                None => load_hint.set(Some("添加失败，请稍后重试".into())),
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
                                    enabled_ids.set(enabled_after_delete(
                                        &prev_enabled,
                                        &known,
                                        refresh,
                                    ));
                                    load_hint.set(None);
                                    toast.success("已删除供应商");
                                }
                                None => load_hint.set(Some("删除失败，请稍后重试".into())),
                            }
                        });
                    },
                }
            }
            if section() == SettingsSection::About {
                div { class: "ac-settings-card ac-settings-card--wide ac-settings-card--page",
                    div { class: "ac-settings-head",
                        h1 { class: "ac-settings-title", "版本信息" }
                    }
                    {
                        let mut rows = about_rows();
                        if let Some(err) = about_error() {
                            rows.push(SettingsInfoRow { label: "加载失败".into(), value: err });
                        }
                        if rows.is_empty() {
                            rsx! {
                                p { class: "ac-settings-lead", "正在加载…" }
                            }
                        } else {
                            rsx! {
                                dl { class: "ac-settings-about-dl ac-settings-about-dl--flat",
                                    for row in rows {
                                        crate::shell::SettingsInfoRowView { row }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
