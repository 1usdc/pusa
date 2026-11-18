//! 模型设置右侧详情与「添加供应商」面板。
//!
//! 详情面板：名称 + 启用开关 + 「…」更多菜单、Base URL、API 格式、API Key、
//! 模型列表（测试连通 / 重命名 / 删除 / 单独启用）。
//! 添加面板：预设供应商一键预填（[`crate::model_presets`]）或自定义。

use std::collections::HashMap;

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdBox, LdEllipsis, LdEye, LdEyeOff, LdPencil, LdTrash2, LdWifi, LdX,
};
use dioxus_free_icons::Icon;
use protocol::{LlmApiProvider, LlmCredentialDto, LlmCredentialUpsertBody};

use crate::model_presets::{api_format_label, parse_api_format_label, MODEL_PROVIDER_PRESETS};
use crate::shell::llm_settings::{
    add_model_pref, api_key_for_upsert, credential_display_name, display_name,
    is_local_or_private_openai_base, is_unspecified_api_key, load_model_prefs,
    prefs_for_credential, remove_model_pref, rename_model_pref, set_model_pref_enabled,
};

#[cfg(any(
    all(target_arch = "wasm32", feature = "web"),
    all(feature = "native", not(target_arch = "wasm32"))
))]
use crate::chat::probe_v1_models_endpoint;

/// 探测 `{base}/models` 是否可达；不支持的目标一律返回 `false`。
async fn probe_endpoint(base: String, api_key: String) -> bool {
    #[cfg(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    ))]
    {
        probe_v1_models_endpoint(&base, &api_key).await
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        let _ = (base, api_key);
        false
    }
}

/// 详情面板：单个供应商的完整配置视图。
#[component]
pub fn ProviderDetailPane(
    cred: Option<LlmCredentialDto>,
    is_on: bool,
    mut prefs_epoch: Signal<u64>,
    mut show_add: Signal<bool>,
    on_update: EventHandler<LlmCredentialUpsertBody>,
    on_toggle: EventHandler<(String, bool)>,
    on_delete: EventHandler<String>,
) -> Element {
    let Some(cred) = cred else {
        return rsx! {
            div { class: "ac-model-detail-empty",
                Icon { icon: LdBox, width: 32, height: 32, fill: "currentColor" }
                p { "在左侧选择一个供应商，或点击「添加供应商」。" }
            }
        };
    };

    let mut editing = use_signal(|| false);
    let mut edit_name = use_signal(String::new);
    let mut edit_base = use_signal(String::new);
    let mut edit_key = use_signal(String::new);
    let mut edit_format = use_signal(|| LlmApiProvider::Openai);
    let mut show_key = use_signal(|| false);
    let mut more_open = use_signal(|| false);
    // 模型列表：新增草稿 / 行内编辑 / 连通性检测。
    let mut model_draft = use_signal(String::new);
    let mut model_error = use_signal(|| None::<String>);
    let row_edit_id = use_signal(|| None::<String>);
    let row_edit_draft = use_signal(String::new);
    let net_status = use_signal(HashMap::<String, bool>::new);
    let testing = use_signal(|| false);

    // 供 rsx 各 move 闭包分别持有的快照（避免共享所有权冲突）。
    let saved_name = credential_display_name(&cred);
    let saved_base = cred.openai_v1_base.clone();
    let saved_format = cred.provider;
    let saved_key = cred.api_key.clone();
    let saved_label_raw = cred.label.clone();
    let key_masked = crate::shell::llm_settings::mask_api_key(&saved_key);
    let local_base = is_local_or_private_openai_base(&saved_base);
    // 进入编辑态要预填的种子值：每处 handler 一份。
    let seed_name_1 = saved_name.clone();
    let seed_base_1 = saved_base.clone();
    let seed_name_2 = saved_name.clone();
    let seed_base_2 = saved_base.clone();
    let seed_name_3 = saved_name.clone();
    let seed_base_3 = saved_base.clone();
    let seed_name_4 = saved_name.clone();
    let seed_base_4 = saved_base.clone();
    let cred_id_toggle = cred.id.clone();
    let cred_id_delete = cred.id.clone();
    let cred_id_save = cred.id.clone();
    let cred_id_add_enter = cred.id.clone();
    let base_for_test = saved_base.clone();
    let key_for_test = saved_key.clone();
    let label_for_save = saved_label_raw.clone();
    let key_for_save = saved_key.clone();

    // 待新增 / 待编辑模型的快照（渲染期逐项克隆）。
    let prefs = {
        let _epoch = prefs_epoch();
        prefs_for_credential(&load_model_prefs(), &cred.id)
    };

    rsx! {
        div { class: "ac-model-detail",
            div { class: "ac-model-detail-head",
                span { class: "ac-model-detail-icon",
                    Icon { icon: LdBox, width: 20, height: 20, fill: "currentColor" }
                }
                h3 { class: "ac-model-detail-name", "{saved_name}" }
                div { class: "ac-model-detail-head-actions",
                    button {
                        r#type: "button",
                        class: if is_on {
                            "ac-model-switch is-on"
                        } else {
                            "ac-model-switch"
                        },
                        role: "switch",
                        aria_checked: is_on,
                        title: if is_on { "停用此供应商" } else { "启用此供应商" },
                        onclick: move |_| on_toggle.call((cred_id_toggle.clone(), !is_on)),
                        span { class: "ac-model-switch-track",
                            span { class: "ac-model-switch-knob" }
                        }
                    }
                    div { class: "ac-model-more-wrap",
                        button {
                            r#type: "button",
                            class: "ac-model-more-btn",
                            title: "更多操作",
                            aria_label: "更多操作",
                            onclick: move |_| more_open.toggle(),
                            Icon { icon: LdEllipsis, width: 16, height: 16, fill: "currentColor" }
                        }
                        if more_open() {
                            div { class: "ac-model-more-backdrop",
                                onclick: move |_| more_open.set(false),
                            }
                            div { class: "ac-model-more-menu", role: "menu",
                                button {
                                    r#type: "button",
                                    role: "menuitem",
                                    class: "ac-model-more-item",
                                    onclick: move |_| {
                                        more_open.set(false);
                                        edit_name.set(seed_name_1.clone());
                                        edit_base.set(seed_base_1.clone());
                                        edit_key.set(String::new());
                                        edit_format.set(saved_format);
                                        editing.set(true);
                                    },
                                    Icon { icon: LdPencil, width: 14, height: 14, fill: "currentColor" }
                                    span { "编辑供应商" }
                                }
                                button {
                                    r#type: "button",
                                    role: "menuitem",
                                    class: "ac-model-more-item is-danger",
                                    onclick: move |_| {
                                        more_open.set(false);
                                        on_delete.call(cred_id_delete.clone());
                                    },
                                    Icon { icon: LdTrash2, width: 14, height: 14, fill: "currentColor" }
                                    span { "删除供应商" }
                                }
                            }
                        }
                    }
                }
            }
            if !editing() {
                div { class: "ac-model-field",
                    label { class: "ac-model-field-label", "Base URL" }
                    input {
                        r#type: "text",
                        class: "ac-model-input",
                        value: "{saved_base}",
                        readonly: true,
                        ondoubleclick: move |_| {
                            edit_name.set(seed_name_2.clone());
                            edit_base.set(seed_base_2.clone());
                            edit_key.set(String::new());
                            edit_format.set(saved_format);
                            editing.set(true);
                        },
                    }
                }
                div { class: "ac-model-field",
                    label { class: "ac-model-field-label", "API 格式" }
                    input {
                        r#type: "text",
                        class: "ac-model-input",
                        value: "{api_format_label(saved_format)}",
                        readonly: true,
                        ondoubleclick: move |_| {
                            edit_name.set(seed_name_3.clone());
                            edit_base.set(seed_base_3.clone());
                            edit_key.set(String::new());
                            edit_format.set(saved_format);
                            editing.set(true);
                        },
                    }
                }
                div { class: "ac-model-field",
                    label { class: "ac-model-field-label", "API Key" }
                    div { class: "ac-model-key-row",
                        input {
                            r#type: if show_key() { "text" } else { "password" },
                            class: "ac-model-input",
                            value: if is_unspecified_api_key(&saved_key) {
                                "".to_string()
                            } else if show_key() {
                                saved_key.clone()
                            } else {
                                key_masked.clone()
                            },
                            readonly: true,
                            placeholder: "未填写",
                            ondoubleclick: move |_| {
                                edit_name.set(seed_name_4.clone());
                                edit_base.set(seed_base_4.clone());
                                edit_key.set(String::new());
                                edit_format.set(saved_format);
                                editing.set(true);
                            },
                        }
                        button {
                            r#type: "button",
                            class: "ac-model-key-eye",
                            title: if show_key() { "隐藏" } else { "显示" },
                            aria_label: if show_key() { "隐藏密钥" } else { "显示密钥" },
                            onclick: move |_| show_key.toggle(),
                            if show_key() {
                                Icon { icon: LdEyeOff, width: 15, height: 15, fill: "currentColor" }
                            } else {
                                Icon { icon: LdEye, width: 15, height: 15, fill: "currentColor" }
                            }
                        }
                    }
                    if local_base && is_unspecified_api_key(&saved_key) {
                        p { class: "ac-model-field-hint", "本地 / 内网端点可不填密钥。" }
                    }
                    p { class: "ac-model-field-hint", "双击输入框或从「…」菜单进入编辑。" }
                }
            } else {
                div { class: "ac-model-edit",
                    div { class: "ac-model-field",
                        label { class: "ac-model-field-label", "名称" }
                        input {
                            r#type: "text",
                            class: "ac-model-input",
                            value: "{edit_name()}",
                            oninput: move |e| edit_name.set(e.value()),
                        }
                    }
                    div { class: "ac-model-field",
                        label { class: "ac-model-field-label", "Base URL" }
                        input {
                            r#type: "text",
                            class: "ac-model-input",
                            value: "{edit_base()}",
                            oninput: move |e| edit_base.set(e.value()),
                        }
                    }
                    div { class: "ac-model-field",
                        label { class: "ac-model-field-label", "API 格式" }
                        select {
                            class: "ac-model-input",
                            value: "{api_format_label(edit_format())}",
                            onchange: move |e| edit_format.set(parse_api_format_label(&e.value())),
                            option {
                                value: "{api_format_label(LlmApiProvider::Openai)}",
                                "{api_format_label(LlmApiProvider::Openai)}"
                            }
                            option {
                                value: "{api_format_label(LlmApiProvider::Anthropic)}",
                                "{api_format_label(LlmApiProvider::Anthropic)}"
                            }
                        }
                    }
                    div { class: "ac-model-field",
                        label { class: "ac-model-field-label", "API Key" }
                        div { class: "ac-model-key-row",
                            input {
                                r#type: if show_key() { "text" } else { "password" },
                                class: "ac-model-input",
                                value: "{edit_key()}",
                                placeholder: if local_base {
                                    "留空则不修改（本地 / 内网）"
                                } else {
                                    "留空则不修改"
                                },
                                oninput: move |e| edit_key.set(e.value()),
                            }
                            button {
                                r#type: "button",
                                class: "ac-model-key-eye",
                                title: if show_key() { "隐藏" } else { "显示" },
                                onclick: move |_| show_key.toggle(),
                                if show_key() {
                                    Icon { icon: LdEyeOff, width: 15, height: 15, fill: "currentColor" }
                                } else {
                                    Icon { icon: LdEye, width: 15, height: 15, fill: "currentColor" }
                                }
                            }
                        }
                    }
                    div { class: "ac-model-edit-actions",
                        button {
                            r#type: "button",
                            class: "ac-model-btn-ghost",
                            onclick: move |_| editing.set(false),
                            "取消"
                        }
                        button {
                            r#type: "button",
                            class: "ac-model-btn-primary",
                            onclick: move |_| {
                                let name_t = edit_name().trim().to_string();
                                let label = if name_t.is_empty() {
                                    display_name(edit_format(), &label_for_save)
                                } else {
                                    name_t
                                };
                                let base_t = edit_base().trim().to_string();
                                let base = if base_t.is_empty() {
                                    edit_format().default_v1_base().to_string()
                                } else {
                                    base_t
                                };
                                let typed = edit_key().trim().to_string();
                                let api_key = if typed.is_empty() {
                                    key_for_save.clone()
                                } else {
                                    api_key_for_upsert(&typed)
                                };
                                on_update.call(LlmCredentialUpsertBody {
                                    id: Some(cred_id_save.clone()),
                                    provider: edit_format(),
                                    api_key,
                                    openai_v1_base: base,
                                    label,
                                });
                                editing.set(false);
                            },
                            "保存"
                        }
                    }
                }
            }

            div { class: "ac-model-models",
                div { class: "ac-model-models-head",
                    span { class: "ac-model-field-label", "模型列表" }
                }
                div { class: "ac-model-model-add-row",
                    input {
                        r#type: "text",
                        class: "ac-model-input",
                        placeholder: "输入模型 ID，回车添加，例如 qwen/qwen3.8-27b",
                        value: "{model_draft()}",
                        oninput: move |e| model_draft.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter {
                                let id = model_draft().trim().to_string();
                                if id.is_empty() {
                                    return;
                                }
                                if add_model_pref(&cred_id_add_enter, &id) {
                                    model_draft.set(String::new());
                                    model_error.set(None);
                                    prefs_epoch += 1;
                                } else {
                                    model_error.set(Some("该模型 ID 已存在。".into()));
                                }
                            }
                        },
                    }
                }
                if let Some(err) = model_error() {
                    p { class: "ac-model-field-hint is-error", "{err}" }
                }
                if prefs.is_empty() {
                    p { class: "ac-model-models-empty",
                        "暂无手动添加的模型。远端端点模型会由聊天栏自动发现；也可在上方手动登记。"
                    }
                } else {
                    ul { class: "ac-model-model-list",
                        for pref in prefs.iter().cloned() {
                            ModelRow {
                                key: "{pref.id}",
                                pref,
                                saved_base: base_for_test.clone(),
                                saved_key: key_for_test.clone(),
                                prefs_epoch,
                                row_edit_id,
                                row_edit_draft,
                                net_status,
                                testing,
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 模型列表中的一行（拆出组件避免外层 rsx 闭包借用冲突）。
#[component]
fn ModelRow(
    pref: crate::custom_chat_models::CustomChatModelPref,
    saved_base: String,
    saved_key: String,
    mut prefs_epoch: Signal<u64>,
    mut row_edit_id: Signal<Option<String>>,
    mut row_edit_draft: Signal<String>,
    mut net_status: Signal<HashMap<String, bool>>,
    mut testing: Signal<bool>,
) -> Element {
    let model_id = pref.id.clone();
    let is_editing = row_edit_id().as_deref() == Some(model_id.as_str());
    let enabled = pref.enabled;
    let net = net_status().get(&model_id).copied();
    let mid_toggle = model_id.clone();
    let mid_delete = model_id.clone();
    let mid_edit = model_id.clone();
    let mid_test = model_id.clone();
    let mid_for_rename = model_id.clone();
    let rename_target = model_id.clone();
    let base = saved_base.clone();
    let key = saved_key.clone();

    rsx! {
        li {
            key: "{model_id}",
            class: "ac-model-model-row",
            if is_editing {
                input {
                    r#type: "text",
                    class: "ac-model-input ac-model-model-rename",
                    value: "{row_edit_draft()}",
                    autofocus: true,
                    oninput: move |e| row_edit_draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            let next = row_edit_draft().trim().to_string();
                            if rename_model_pref(&rename_target, &next) {
                                prefs_epoch += 1;
                            }
                            row_edit_id.set(None);
                        } else if e.key() == Key::Escape {
                            row_edit_id.set(None);
                        }
                    },
                }
            } else {
                span { class: "ac-model-model-name", "{model_id}" }
            }
            if let Some(ok) = net {
                span {
                    class: if ok {
                        "ac-model-net-dot is-ok"
                    } else {
                        "ac-model-net-dot is-bad"
                    },
                    title: if ok { "连通正常" } else { "连接失败" },
                }
            }
            div { class: "ac-model-model-actions",
                button {
                    r#type: "button",
                    class: "ac-model-model-icon-btn",
                    title: "测试连通",
                    aria_label: "测试连通",
                    disabled: testing(),
                    onclick: move |_| {
                        if testing() {
                            return;
                        }
                        testing.set(true);
                        let base = base.clone();
                        let key = key.clone();
                        let mid = mid_test.clone();
                        spawn(async move {
                            let ok = if base.trim().is_empty() {
                                false
                            } else {
                                probe_endpoint(base, key).await
                            };
                            net_status.with_mut(|m| m.insert(mid, ok));
                            testing.set(false);
                        });
                    },
                    Icon { icon: LdWifi, width: 14, height: 14, fill: "currentColor" }
                }
                button {
                    r#type: "button",
                    class: "ac-model-model-icon-btn",
                    title: "重命名",
                    aria_label: "重命名模型",
                    onclick: move |_| {
                        row_edit_draft.set(mid_for_rename.clone());
                        row_edit_id.set(Some(mid_edit.clone()));
                    },
                    Icon { icon: LdPencil, width: 14, height: 14, fill: "currentColor" }
                }
                button {
                    r#type: "button",
                    class: "ac-model-model-icon-btn is-danger",
                    title: "删除模型",
                    aria_label: "删除模型",
                    onclick: move |_| {
                        remove_model_pref(&mid_delete);
                        prefs_epoch += 1;
                    },
                    Icon { icon: LdTrash2, width: 14, height: 14, fill: "currentColor" }
                }
                button {
                    r#type: "button",
                    class: if enabled {
                        "ac-model-switch is-on"
                    } else {
                        "ac-model-switch"
                    },
                    role: "switch",
                    aria_checked: enabled,
                    title: if enabled { "停用此模型" } else { "启用此模型" },
                    onclick: move |_| {
                        set_model_pref_enabled(&mid_toggle, !enabled);
                        prefs_epoch += 1;
                    },
                    span { class: "ac-model-switch-track",
                        span { class: "ac-model-switch-knob" }
                    }
                }
            }
        }
    }
}

/// 「添加供应商」面板：预设列表 + 自定义表单。
#[component]
pub fn AddProviderSheet(
    on_add: EventHandler<LlmCredentialUpsertBody>,
    on_close: EventHandler<()>,
    on_submit_preset: EventHandler<(String, Vec<String>)>,
) -> Element {
    let mut mode_preset = use_signal(|| true);
    let mut name = use_signal(String::new);
    let mut base = use_signal(|| LlmApiProvider::Openai.default_v1_base().to_string());
    let mut key = use_signal(String::new);
    let mut format = use_signal(|| LlmApiProvider::Openai);
    let mut picked = use_signal(|| None::<usize>);
    let mut error = use_signal(|| None::<String>);

    rsx! {
        div { class: "ac-model-add-overlay",
            div {
                class: "ac-model-add-backdrop",
                onclick: move |_| on_close.call(()),
            }
            div { class: "ac-model-add-sheet",
            div { class: "ac-model-add-head",
                h3 { class: "ac-model-detail-name", "添加供应商" }
                button {
                    r#type: "button",
                    class: "ac-model-model-icon-btn",
                    title: "关闭",
                    aria_label: "关闭",
                    onclick: move |_| on_close.call(()),
                    Icon { icon: LdX, width: 15, height: 15, fill: "currentColor" }
                }
            }
            div { class: "ac-model-add-tabs",
                button {
                    r#type: "button",
                    class: if mode_preset() {
                        "ac-model-add-tab is-active"
                    } else {
                        "ac-model-add-tab"
                    },
                    onclick: move |_| mode_preset.set(true),
                    "从预设添加"
                }
                button {
                    r#type: "button",
                    class: if !mode_preset() {
                        "ac-model-add-tab is-active"
                    } else {
                        "ac-model-add-tab"
                    },
                    onclick: move |_| {
                        mode_preset.set(false);
                        picked.set(None);
                        name.set(String::new());
                        base.set(LlmApiProvider::Openai.default_v1_base().to_string());
                        format.set(LlmApiProvider::Openai);
                        key.set(String::new());
                        error.set(None);
                    },
                    "自定义"
                }
            }
            if mode_preset() {
                div { class: "ac-model-preset-grid",
                    for (idx, p) in MODEL_PROVIDER_PRESETS.iter().enumerate() {
                        {
                            let is_picked = picked() == Some(idx);
                            let idx_click = idx;
                            let preset = p;
                            rsx! {
                                button {
                                    key: "{preset.name}",
                                    r#type: "button",
                                    class: if is_picked {
                                        "ac-model-preset-card is-active"
                                    } else {
                                        "ac-model-preset-card"
                                    },
                                    onclick: move |_| {
                                        picked.set(Some(idx_click));
                                        name.set(preset.name.to_string());
                                        base.set(preset.base_url.to_string());
                                        format.set(preset.api_format);
                                        key.set(String::new());
                                        error.set(None);
                                    },
                                    span { class: "ac-model-preset-icon",
                                        Icon { icon: LdBox, width: 15, height: 15, fill: "currentColor" }
                                    }
                                    span { class: "ac-model-preset-name", "{preset.name}" }
                                    span { class: "ac-model-preset-base", "{preset.base_url}" }
                                    if is_picked && !preset.default_models.is_empty() {
                                        span { class: "ac-model-preset-hint",
                                            "将预置 {preset.default_models.len()} 个常用模型"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if picked().is_some() {
                    p { class: "ac-model-field-hint",
                        "已按预设填好下方表单，可直接「添加」，或先调整。"
                    }
                }
            }
            div { class: "ac-model-field",
                label { class: "ac-model-field-label", "名称" }
                input {
                    r#type: "text",
                    class: "ac-model-input",
                    placeholder: "例如 My Provider",
                    value: "{name()}",
                    oninput: move |e| name.set(e.value()),
                }
            }
            div { class: "ac-model-field",
                label { class: "ac-model-field-label", "Base URL（以 /v1 结尾）" }
                input {
                    r#type: "text",
                    class: "ac-model-input",
                    placeholder: "https://api.openai.com/v1",
                    value: "{base()}",
                    oninput: move |e| base.set(e.value()),
                }
            }
            div { class: "ac-model-field",
                label { class: "ac-model-field-label", "API 格式" }
                select {
                    class: "ac-model-input",
                    value: "{api_format_label(format())}",
                    onchange: move |e| format.set(parse_api_format_label(&e.value())),
                    option {
                        value: "{api_format_label(LlmApiProvider::Openai)}",
                        "{api_format_label(LlmApiProvider::Openai)}"
                    }
                    option {
                        value: "{api_format_label(LlmApiProvider::Anthropic)}",
                        "{api_format_label(LlmApiProvider::Anthropic)}"
                    }
                }
            }
            div { class: "ac-model-field",
                label { class: "ac-model-field-label", "API Key（可选）" }
                input {
                    r#type: "password",
                    class: "ac-model-input",
                    placeholder: {
                        let idx = picked();
                        idx.and_then(|i| MODEL_PROVIDER_PRESETS.get(i))
                            .map(|p| p.key_placeholder)
                            .unwrap_or("可留空（本地 / 内网端点）")
                    },
                    value: "{key()}",
                    oninput: move |e| key.set(e.value()),
                }
            }
            if let Some(err) = error() {
                p { class: "ac-model-field-hint is-error", "{err}" }
            }
            div { class: "ac-model-edit-actions",
                button {
                    r#type: "button",
                    class: "ac-model-btn-ghost",
                    onclick: move |_| on_close.call(()),
                    "取消"
                }
                button {
                    r#type: "button",
                    class: "ac-model-btn-primary",
                    onclick: move |_| {
                        let name_t = name().trim().to_string();
                        if name_t.is_empty() {
                            error.set(Some("请填写供应商名称。".into()));
                            return;
                        }
                        let base_t = base().trim().to_string();
                        if base_t.is_empty() {
                            error.set(Some("请填写 Base URL。".into()));
                            return;
                        }
                        let fmt = format();
                        let api_key = api_key_for_upsert(&key());
                        on_add.call(LlmCredentialUpsertBody {
                            id: None,
                            provider: fmt,
                            api_key,
                            openai_v1_base: base_t,
                            label: name_t,
                        });
                        on_close.call(());
                    },
                    "添加"
                }
                if mode_preset() && picked().is_some() {
                    button {
                        r#type: "button",
                        class: "ac-model-btn-primary",
                        onclick: move |_| {
                            let Some(idx) = picked() else {
                                return;
                            };
                            let preset = &MODEL_PROVIDER_PRESETS[idx];
                            let name_t = name().trim().to_string();
                            if name_t.is_empty() {
                                error.set(Some("请填写供应商名称。".into()));
                                return;
                            }
                            let base_t = base().trim().to_string();
                            if base_t.is_empty() {
                                error.set(Some("请填写 Base URL。".into()));
                                return;
                            }
                            let models: Vec<String> = preset
                                .default_models
                                .iter()
                                .map(|s| s.to_string())
                                .collect();
                            // 预置流程：先保存凭证，再登记预置模型。
                            on_add.call(LlmCredentialUpsertBody {
                                id: None,
                                provider: preset.api_format,
                                api_key: api_key_for_upsert(&key()),
                                openai_v1_base: base_t,
                                label: name_t.clone(),
                            });
                            on_submit_preset.call((name_t, models));
                            on_close.call(());
                        },
                        "添加并预置模型"
                    }
                }
            }
        }
        }
    }
}
