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
        div {
            class: "fixed top-[var(--ac-chrome-top,0px)] right-0 bottom-[var(--ac-chrome-bottom,1.625rem)] left-0 z-[1260] flex items-center justify-center px-4 box-border",
            div {
                class: "absolute inset-0 bg-[var(--shadow-color)] backdrop-blur-[10px] cursor-pointer",
                onclick: move |_| show_about_modal.set(false),
            }
            div {
                class: "relative z-[1] w-full max-w-[32rem] h-auto max-h-[calc(100%-40px)] overflow-hidden flex flex-col min-h-0 box-border",
                div { class: crate::shell::style::MODAL_CARD,
                    div { class: crate::shell::style::MODAL_HEAD,
                        h1 {
                            class: "cursor-default select-none m-0 text-[1.25rem] font-extrabold text-secondary-1",
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
                            class: crate::shell::style::MODAL_CLOSE,
                            title: "关闭",
                            aria_label: "关闭",
                            onclick: move |_| show_about_modal.set(false),
                            Icon {
                                icon: LdX,
                                width: 18,
                                height: 18,
                                fill: "currentColor",
                                class: "shrink-0",
                            }
                        }
                    }
                    if loading() {
                        p {
                            class: "grid grid-cols-[5.5rem_1fr] gap-x-4 gap-y-[0.65rem] items-start m-0 mb-[0.85rem] text-[0.9rem] leading-[1.45] flex-[0_1_auto] min-h-0 overflow-x-hidden overflow-y-auto",
                            "加载中…"
                        }
                    } else if let Some(err) = load_error() {
                        p {
                            class: "grid grid-cols-[5.5rem_1fr] gap-x-4 gap-y-[0.65rem] items-start m-0 mb-[0.85rem] text-[0.9rem] leading-[1.45] flex-[0_1_auto] min-h-0 overflow-x-hidden overflow-y-auto",
                            "无法获取版本信息：{err}"
                        }
                    } else if let Some(dto) = info() {
                        dl { class: "m-0 p-0 flex-[0_1_auto] min-h-0 overflow-x-hidden overflow-y-auto",
                            div {
                                class: "grid grid-cols-[5.5rem_1fr] gap-x-4 gap-y-[0.65rem] items-start m-0 mb-[0.85rem] text-[0.9rem] leading-[1.45]",
                                dt { class: "m-0 font-bold", "版本号" }
                                dd { class: "m-0 font-semibold [word-break:break-word]", "{dto.version}" }
                            }
                            if developer_mode() {
                                div {
                                    class: "grid grid-cols-[5.5rem_1fr] gap-x-4 gap-y-[0.65rem] items-start m-0 mb-[0.85rem] text-[0.9rem] leading-[1.45] w-full box-border p-0 bg-transparent text-left cursor-pointer rounded-[0.35rem] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--ac-color-93c5fd)]",
                                    role: "button",
                                    tabindex: "0",
                                    onclick: move |_| {
                                        dev_mode::register_triple_click(triple_dev, move || {
                                            dev_mode::apply(false, developer_mode);
                                        });
                                    },
                                    dt { class: "m-0 font-bold", "开发者模式" }
                                    dd { class: "m-0 font-semibold [word-break:break-word]", "true" }
                                }
                            }
                            div {
                                class: "grid grid-cols-[5.5rem_1fr] gap-x-4 gap-y-[0.65rem] items-start m-0 mb-[0.85rem] text-[0.9rem] leading-[1.45]",
                                dt { class: "m-0 font-bold", "系统" }
                                dd { class: "m-0 font-semibold [word-break:break-word]", "{dto.runtime_os}" }
                            }
                            div {
                                class: "grid grid-cols-[5.5rem_1fr] gap-x-4 gap-y-[0.65rem] items-start m-0 mb-[0.85rem] text-[0.9rem] leading-[1.45]",
                                dt { class: "m-0 font-bold", "架构" }
                                dd { class: "m-0 font-semibold [word-break:break-word]", "{dto.runtime_arch}" }
                            }
                            div {
                                class: "grid grid-cols-[5.5rem_1fr] gap-x-4 gap-y-[0.65rem] items-start m-0 mb-[0.85rem] text-[0.9rem] leading-[1.45]",
                                dt { class: "m-0 font-bold", "浏览器" }
                                dd { class: "m-0 font-semibold [word-break:break-word] font-mono text-[0.76rem] max-h-[4.5rem] overflow-auto", "{browser_user_agent()}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
