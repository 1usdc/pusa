//! Web：关于本机弹窗（版本等信息来自 `GET /v1/about`）。
//! 「关于本机」标题连续点击 3 次可开关开发者模式（隐藏入口）。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdX;
use dioxus_free_icons::Icon;
use protocol::AboutInfoDto;

use crate::chat::{api_base_url, transport_wasm};
use crate::shell::dev_mode;
use crate::web::ShellChromeCtx;

fn browser_user_agent() -> String {
    web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "—".to_string())
}

/// 关于本机弹窗（居中）。
#[component]
pub fn WebAboutModal(mut show_about_modal: Signal<bool>) -> Element {
    let ctx = use_context::<ShellChromeCtx>();
    let developer_mode = ctx.developer_mode;
    let mut info = use_signal(|| None::<AboutInfoDto>);
    let mut load_error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);
    let mut triple_title = use_signal(|| (0u32, 0.0f64));
    let mut triple_dev = use_signal(|| (0u32, 0.0f64));

    use_effect(move || {
        if !show_about_modal() {
            triple_title.set((0, 0.0));
            triple_dev.set((0, 0.0));
            return;
        }
        loading.set(true);
        load_error.set(None);
        spawn(async move {
            match transport_wasm::load_about_info(&api_base_url()).await {
                Ok(dto) => {
                    info.set(Some(dto));
                    load_error.set(None);
                }
                Err(e) => {
                    info.set(None);
                    load_error.set(Some(e.to_string()));
                }
            }
            loading.set(false);
        });
    });

    rsx! {
        div { class: "ac-settings-modal-root ac-about-modal-root",
            div {
                class: "ac-settings-modal-backdrop",
                onclick: move |_| show_about_modal.set(false),
            }
            div { class: "ac-settings-modal-dialog",
                div { class: "ac-settings-card ac-about-card",
                    div { class: "ac-settings-head",
                        h1 {
                            class: "ac-settings-title ac-about-title-tappable",
                            role: "button",
                            tabindex: "0",
                            title: "连续点击三次可开关开发者模式",
                            onclick: move |_| {
                                dev_mode::register_triple_click(triple_title, move || {
                                    dev_mode::toggle(developer_mode);
                                });
                            },
                            "关于本机"
                        }
                        button {
                            r#type: "button",
                            class: "ac-settings-close",
                            title: "关闭",
                            aria_label: "关闭",
                            onclick: move |_| show_about_modal.set(false),
                            Icon {
                                icon: LdX,
                                width: 18,
                                height: 18,
                                fill: "currentColor",
                                class: "ac-settings-close-icon",
                            }
                        }
                    }
                    if loading() {
                        p { class: "ac-about-row ac-about-muted", "加载中…" }
                    } else if let Some(err) = load_error() {
                        p { class: "ac-about-row ac-about-error", "无法获取版本信息：{err}" }
                    } else if let Some(dto) = info() {
                        dl { class: "ac-about-dl",
                            div { class: "ac-about-row",
                                dt { "版本号" }
                                dd { "{dto.version}" }
                            }
                            if developer_mode() {
                                div {
                                    class: "ac-about-row ac-about-row-tappable",
                                    role: "button",
                                    tabindex: "0",
                                    onclick: move |_| {
                                        dev_mode::register_triple_click(triple_dev, move || {
                                            dev_mode::apply(false, developer_mode);
                                        });
                                    },
                                    dt { "开发者模式" }
                                    dd { "true" }
                                }
                            }
                            div { class: "ac-about-row",
                                dt { "系统" }
                                dd { "{dto.runtime_os}" }
                            }
                            div { class: "ac-about-row",
                                dt { "架构" }
                                dd { "{dto.runtime_arch}" }
                            }
                            div { class: "ac-about-row",
                                dt { "浏览器" }
                                dd { class: "ac-about-mono ac-about-ua", "{browser_user_agent()}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
