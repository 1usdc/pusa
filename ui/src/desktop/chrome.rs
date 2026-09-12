//! 本机桌面无边框窗口顶部的自定义标题栏（macOS / Windows；Dioxus desktop / WebView）。
//!
//! macOS：系统红绿灯由 tao `WindowBuilderExtMacOS` 保留（透明标题栏 + fullsize content）；
//! 本组件只渲染业务按钮，左侧用 spacer 避开原生红绿灯命中区。
//! Windows：无系统装饰，仍自绘红绿灯。
#![cfg(all(feature = "native", not(target_arch = "wasm32")))]

use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::{
    BsGear, BsLayoutSidebar, BsLayoutSidebarInset, BsLayoutSidebarInsetReverse,
    BsLayoutSidebarReverse,
};
use dioxus_free_icons::icons::ld_icons::LdGlobe;
use dioxus_free_icons::Icon;

use crate::desktop::window;
use crate::icons::{
    VscLayoutPanel, VscLayoutPanelOff, VscLayoutSidebarLeftDock, VscLayoutSidebarRightDock,
};
use crate::shell::browser::OpenBrowserTick;
use crate::shell::theme::{self, TitlebarThemeToggle};

#[component]
fn NativeResizeHandle(class: String) -> Element {
    rsx! {
        div { class: "{class}" }
    }
}

/// 透明根背景（不依赖 `:has()`，避免渲染器不支持时圆角外仍为白底）。
#[component]
pub fn NativeHostShellStyle() -> Element {
    rsx! {}
}

/// macOS：系统红绿灯已由 tao 绘制，这里只占位避免业务按钮叠上去。
#[cfg(target_os = "macos")]
#[component]
fn NativeTrafficControls() -> Element {
    rsx! {
        div {
            class: "ac-native-traffic-spacer",
            aria_hidden: "true",
        }
    }
}

/// Windows：无系统装饰，自绘红绿灯。
#[cfg(not(target_os = "macos"))]
#[component]
fn NativeTrafficControls() -> Element {
    rsx! {
        div { class: "ac-native-traffic",
            button {
                r#type: "button",
                class: "ac-native-dot ac-native-dot-close",
                title: "关闭",
                aria_label: "关闭",
                onclick: move |_| window::close_window(),
            }
            button {
                r#type: "button",
                class: "ac-native-dot ac-native-dot-min",
                title: "最小化",
                aria_label: "最小化",
                onclick: move |_| window::minimize_window(),
            }
            button {
                r#type: "button",
                class: "ac-native-dot ac-native-dot-zoom",
                title: "缩放",
                aria_label: "缩放",
                onclick: move |_| window::toggle_maximize(),
            }
        }
    }
}

/// Linux 保留系统装饰，不渲染自定义栏（避免无 `Window` 句柄时无法拖拽）。
#[cfg(not(target_os = "linux"))]
#[component]
pub fn NativeTitleBar(
    mut show_sidebar: Signal<bool>,
    mut show_center: Signal<bool>,
    mut show_terminal: Signal<bool>,
    mut show_chat: Signal<bool>,
    mut chat_history_open: Signal<bool>,
    mut show_settings_modal: Signal<bool>,
    mut show_about_modal: Signal<bool>,
    mut show_titlebar_settings_menu: Signal<bool>,
) -> Element {
    let sidebar_title = if show_sidebar() {
        "隐藏左边栏"
    } else {
        "显示左边栏"
    };
    let center_title = if show_center() {
        "聊天栏往左覆盖中间栏"
    } else {
        "聊天栏往右收缩拉出中间栏"
    };
    let terminal_title = if show_terminal() {
        "隐藏终端"
    } else {
        "显示终端"
    };
    let chat_title = if show_chat() {
        "隐藏右边聊天栏"
    } else {
        "显示右边聊天栏"
    };
    let ui_theme = use_signal(theme::load);
    let open_browser = use_context::<OpenBrowserTick>();

    rsx! {
        NativeResizeHandle {
            class: "ac-native-resize-hit ac-native-resize-n".to_string(),
        }
        NativeResizeHandle {
            class: "ac-native-resize-hit ac-native-resize-e".to_string(),
        }
        NativeResizeHandle {
            class: "ac-native-resize-hit ac-native-resize-s".to_string(),
        }
        NativeResizeHandle {
            class: "ac-native-resize-hit ac-native-resize-w".to_string(),
        }
        NativeResizeHandle {
            class: "ac-native-resize-hit ac-native-resize-ne".to_string(),
        }
        NativeResizeHandle {
            class: "ac-native-resize-hit ac-native-resize-nw".to_string(),
        }
        NativeResizeHandle {
            class: "ac-native-resize-hit ac-native-resize-se".to_string(),
        }
        NativeResizeHandle {
            class: "ac-native-resize-hit ac-native-resize-sw".to_string(),
        }
        div {
            class: if cfg!(target_os = "macos") {
                "ac-native-titlebar ac-native-titlebar--macos"
            } else {
                "ac-native-titlebar"
            },
            onmounted: move |_| {
                window::apply_host_window_rounding();
                window::schedule_host_window_rounding_retry();
            },
            NativeTrafficControls {}
            // 左侧：左边栏显隐 → 中间栏覆盖/展开
            div { class: "ac-native-titlebar-actions ac-web-titlebar-actions",
                button {
                    r#type: "button",
                    class: "ac-web-titlebar-btn",
                    title: "{sidebar_title}",
                    aria_label: "{sidebar_title}",
                    aria_expanded: show_sidebar(),
                    onclick: move |_| show_sidebar.toggle(),
                    if show_sidebar() {
                        Icon { icon: BsLayoutSidebar, width: 12, height: 12, fill: "currentColor", class: "ac-web-titlebar-icon" }
                    } else {
                        Icon { icon: BsLayoutSidebarInset, width: 12, height: 12, fill: "currentColor", class: "ac-web-titlebar-icon" }
                    }
                }
                button {
                    r#type: "button",
                    class: "ac-web-titlebar-btn",
                    title: "{center_title}",
                    aria_label: "{center_title}",
                    aria_expanded: show_center(),
                    disabled: !show_chat(),
                    onclick: move |_| {
                        if show_chat() {
                            show_center.toggle();
                        }
                    },
                    if show_center() {
                        Icon { icon: VscLayoutSidebarLeftDock, width: 12, height: 12, fill: "currentColor", class: "ac-web-titlebar-icon" }
                    } else {
                        Icon { icon: VscLayoutSidebarRightDock, width: 12, height: 12, fill: "currentColor", class: "ac-web-titlebar-icon" }
                    }
                }
                button {
                    r#type: "button",
                    class: "ac-web-titlebar-btn",
                    title: "打开 Pusa 浏览器",
                    aria_label: "打开 Pusa 浏览器",
                    onclick: move |_| open_browser.request(),
                    Icon { icon: LdGlobe, width: 12, height: 12, fill: "currentColor", class: "ac-web-titlebar-icon" }
                }
            }
            div {
                class: "ac-native-titlebar-drag",
                onmousedown: move |_| window::drag_window(),
            }
            // 右侧：终端 → 右边聊天栏显隐 → 设置（会话历史靠拖宽手柄）
            div { class: "ac-web-titlebar-actions ac-web-titlebar-actions-end",
                button {
                    r#type: "button",
                    class: "ac-web-titlebar-btn",
                    title: "{terminal_title}",
                    aria_label: "{terminal_title}",
                    aria_expanded: show_terminal(),
                    onclick: move |_| show_terminal.toggle(),
                    if show_terminal() {
                        Icon { icon: VscLayoutPanelOff, width: 12, height: 12, fill: "currentColor", class: "ac-web-titlebar-icon" }
                    } else {
                        Icon { icon: VscLayoutPanel, width: 12, height: 12, fill: "currentColor", class: "ac-web-titlebar-icon" }
                    }
                }
                button {
                    r#type: "button",
                    class: "ac-web-titlebar-btn",
                    title: "{chat_title}",
                    aria_label: "{chat_title}",
                    aria_expanded: show_chat(),
                    onclick: move |_| {
                        let next = !show_chat();
                        show_chat.set(next);
                        if !next {
                            chat_history_open.set(false);
                        }
                    },
                    if show_chat() {
                        Icon { icon: BsLayoutSidebarReverse, width: 12, height: 12, fill: "currentColor", class: "ac-web-titlebar-icon" }
                    } else {
                        Icon { icon: BsLayoutSidebarInsetReverse, width: 12, height: 12, fill: "currentColor", class: "ac-web-titlebar-icon" }
                    }
                }
                div { class: "ac-web-titlebar-settings-wrap",
                    button {
                        r#type: "button",
                        class: if show_titlebar_settings_menu() {
                            "ac-web-titlebar-btn is-menu-open"
                        } else {
                            "ac-web-titlebar-btn"
                        },
                        title: "设置",
                        aria_expanded: show_titlebar_settings_menu(),
                        aria_haspopup: "menu",
                        onclick: move |_| show_titlebar_settings_menu.toggle(),
                        Icon { icon: BsGear, width: 12, height: 12, fill: "currentColor", class: "ac-web-titlebar-icon" }
                    }
                    if show_titlebar_settings_menu() {
                        div {
                            class: "ac-web-titlebar-menu-backdrop",
                            onclick: move |_| show_titlebar_settings_menu.set(false),
                        }
                        div { class: "ac-web-titlebar-menu", role: "menu",
                            button {
                                r#type: "button",
                                role: "menuitem",
                                class: "ac-web-titlebar-menu__item",
                                onclick: move |_| {
                                    show_titlebar_settings_menu.set(false);
                                    show_settings_modal.set(true);
                                },
                                "AI大模型"
                            }
                            TitlebarThemeToggle {
                                    ui_theme,
                                    show_titlebar_settings_menu,
                                }
                            button {
                                r#type: "button",
                                role: "menuitem",
                                class: "ac-web-titlebar-menu__item",
                                onclick: move |_| {
                                    show_titlebar_settings_menu.set(false);
                                    show_about_modal.set(true);
                                },
                                "关于本机"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[component]
pub fn NativeTitleBar(
    _show_sidebar: Signal<bool>,
    _show_center: Signal<bool>,
    _show_terminal: Signal<bool>,
    _show_chat: Signal<bool>,
    _chat_history_open: Signal<bool>,
    _show_settings_modal: Signal<bool>,
    _show_about_modal: Signal<bool>,
    _show_titlebar_settings_menu: Signal<bool>,
) -> Element {
    rsx! {}
}
