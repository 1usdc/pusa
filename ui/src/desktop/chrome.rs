//! 本机桌面无边框窗口顶部的自定义标题栏（macOS / Windows；Dioxus desktop / WebView）。
//!
//! macOS：系统红绿灯由 tao `WindowBuilderExtMacOS` 保留（透明标题栏 + fullsize content）；
//! 本组件只渲染业务按钮，左侧用 spacer 避开原生红绿灯命中区。
//! Windows：右侧 Win 风格最小化 / 最大化 / 关闭（无红绿黄圆点）。
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
use crate::shell::style::{
    NATIVE_CAPTION,
    NATIVE_CAPTION_BTN,
    NATIVE_CAPTION_CLOSE,
    NATIVE_CAPTION_ICON,
    NATIVE_CAPTION_ICON_CLOSE,
    NATIVE_CAPTION_ICON_MAX,
    NATIVE_CAPTION_ICON_MIN,
    NATIVE_CAPTION_ICON_RESTORE,
    NATIVE_RESIZE_E,
    NATIVE_RESIZE_HIT,
    NATIVE_RESIZE_N,
    NATIVE_RESIZE_NE,
    NATIVE_RESIZE_NW,
    NATIVE_RESIZE_S,
    NATIVE_RESIZE_SE,
    NATIVE_RESIZE_SW,
    NATIVE_RESIZE_W,
    NATIVE_TITLEBAR,
    NATIVE_TITLEBAR_ACTIONS,
    NATIVE_TITLEBAR_BTN,
    NATIVE_TITLEBAR_DRAG,
    NATIVE_TITLEBAR_MACOS,
    NATIVE_TITLEBAR_WINDOWS,
    NATIVE_TRAFFIC_SPACER,
    WEB_TITLEBAR_ACTIONS_END,
    WEB_TITLEBAR_BTN,
    WEB_TITLEBAR_ICON,
};

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
            class: NATIVE_TRAFFIC_SPACER,
            aria_hidden: "true",
        }
    }
}

#[cfg(not(target_os = "macos"))]
#[component]
fn NativeTrafficControls() -> Element {
    rsx! {}
}

/// Windows：右侧原生风格窗口按钮（最小化 / 最大化或还原 / 关闭）。
#[cfg(target_os = "windows")]
#[component]
fn NativeCaptionControls() -> Element {
    let mut maximized = use_signal(window::is_maximized);

    rsx! {
        div { class: NATIVE_CAPTION, role: "group", aria_label: "窗口控制",
            button {
                r#type: "button",
                class: NATIVE_CAPTION_BTN,
                title: "最小化",
                aria_label: "最小化",
                onclick: move |_| window::minimize_window(),
                span { class: "{NATIVE_CAPTION_ICON} {NATIVE_CAPTION_ICON_MIN}", aria_hidden: "true" }
            }
            button {
                r#type: "button",
                class: NATIVE_CAPTION_BTN,
                title: if maximized() { "向下还原" } else { "最大化" },
                aria_label: if maximized() { "向下还原" } else { "最大化" },
                onclick: move |_| {
                    window::toggle_maximize();
                    maximized.set(window::is_maximized());
                },
                if maximized() {
                    span { class: "{NATIVE_CAPTION_ICON} {NATIVE_CAPTION_ICON_RESTORE}", aria_hidden: "true" }
                } else {
                    span { class: "{NATIVE_CAPTION_ICON} {NATIVE_CAPTION_ICON_MAX}", aria_hidden: "true" }
                }
            }
            button {
                r#type: "button",
                class: "{NATIVE_CAPTION_BTN} {NATIVE_CAPTION_CLOSE}",
                title: "关闭",
                aria_label: "关闭",
                onclick: move |_| window::close_window(),
                span { class: "{NATIVE_CAPTION_ICON} {NATIVE_CAPTION_ICON_CLOSE}", aria_hidden: "true" }
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
#[component]
fn NativeCaptionControls() -> Element {
    rsx! {}
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
    let open_browser = use_context::<OpenBrowserTick>();

    let titlebar_class = if cfg!(target_os = "macos") {
        format!("{NATIVE_TITLEBAR} {NATIVE_TITLEBAR_MACOS}")
    } else if cfg!(target_os = "windows") {
        format!("{NATIVE_TITLEBAR} {NATIVE_TITLEBAR_WINDOWS}")
    } else {
        NATIVE_TITLEBAR.to_string()
    };

    rsx! {
        NativeResizeHandle {
            class: "{NATIVE_RESIZE_HIT} {NATIVE_RESIZE_N}".to_string(),
        }
        NativeResizeHandle {
            class: "{NATIVE_RESIZE_HIT} {NATIVE_RESIZE_E}".to_string(),
        }
        NativeResizeHandle {
            class: "{NATIVE_RESIZE_HIT} {NATIVE_RESIZE_S}".to_string(),
        }
        NativeResizeHandle {
            class: "{NATIVE_RESIZE_HIT} {NATIVE_RESIZE_W}".to_string(),
        }
        NativeResizeHandle {
            class: "{NATIVE_RESIZE_HIT} {NATIVE_RESIZE_NE}".to_string(),
        }
        NativeResizeHandle {
            class: "{NATIVE_RESIZE_HIT} {NATIVE_RESIZE_NW}".to_string(),
        }
        NativeResizeHandle {
            class: "{NATIVE_RESIZE_HIT} {NATIVE_RESIZE_SE}".to_string(),
        }
        NativeResizeHandle {
            class: "{NATIVE_RESIZE_HIT} {NATIVE_RESIZE_SW}".to_string(),
        }
        div {
            class: "{titlebar_class}",
            onmounted: move |_| {
                window::apply_host_window_rounding();
                window::schedule_host_window_rounding_retry();
            },
            NativeTrafficControls {}
            // 左侧：左边栏显隐 → 中间栏覆盖/展开
            div { class: NATIVE_TITLEBAR_ACTIONS,
                button {
                    r#type: "button",
                    class: WEB_TITLEBAR_BTN,
                    title: "{sidebar_title}",
                    aria_label: "{sidebar_title}",
                    aria_expanded: show_sidebar(),
                    onclick: move |_| show_sidebar.toggle(),
                    if show_sidebar() {
                        Icon { icon: BsLayoutSidebar, width: 12, height: 12, fill: "currentColor", class: WEB_TITLEBAR_ICON }
                    } else {
                        Icon { icon: BsLayoutSidebarInset, width: 12, height: 12, fill: "currentColor", class: WEB_TITLEBAR_ICON }
                    }
                }
                button {
                    r#type: "button",
                    class: WEB_TITLEBAR_BTN,
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
                        Icon { icon: VscLayoutSidebarLeftDock, width: 12, height: 12, fill: "currentColor", class: WEB_TITLEBAR_ICON }
                    } else {
                        Icon { icon: VscLayoutSidebarRightDock, width: 12, height: 12, fill: "currentColor", class: WEB_TITLEBAR_ICON }
                    }
                }
                button {
                    r#type: "button",
                    class: WEB_TITLEBAR_BTN,
                    title: "打开 Pusa 浏览器",
                    aria_label: "打开 Pusa 浏览器",
                    onclick: move |_| open_browser.request(),
                    Icon { icon: LdGlobe, width: 12, height: 12, fill: "currentColor", class: WEB_TITLEBAR_ICON }
                }
            }
            div {
                class: NATIVE_TITLEBAR_DRAG,
                onmousedown: move |_| window::drag_window(),
            }
            // 右侧：终端 → 右边聊天栏显隐 → 设置（会话历史靠拖宽手柄）
            div { class: WEB_TITLEBAR_ACTIONS_END,
                button {
                    r#type: "button",
                    class: WEB_TITLEBAR_BTN,
                    title: "{terminal_title}",
                    aria_label: "{terminal_title}",
                    aria_expanded: show_terminal(),
                    onclick: move |_| show_terminal.toggle(),
                    if show_terminal() {
                        Icon { icon: VscLayoutPanelOff, width: 12, height: 12, fill: "currentColor", class: WEB_TITLEBAR_ICON }
                    } else {
                        Icon { icon: VscLayoutPanel, width: 12, height: 12, fill: "currentColor", class: WEB_TITLEBAR_ICON }
                    }
                }
                button {
                    r#type: "button",
                    class: WEB_TITLEBAR_BTN,
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
                        Icon { icon: BsLayoutSidebarReverse, width: 12, height: 12, fill: "currentColor", class: WEB_TITLEBAR_ICON }
                    } else {
                        Icon { icon: BsLayoutSidebarInsetReverse, width: 12, height: 12, fill: "currentColor", class: WEB_TITLEBAR_ICON }
                    }
                }
                button {
                    r#type: "button",
                    class: WEB_TITLEBAR_BTN,
                    title: "设置",
                    aria_label: "设置",
                    onclick: move |_| show_settings_modal.set(true),
                    Icon { icon: BsGear, width: 12, height: 12, fill: "currentColor", class: WEB_TITLEBAR_ICON }
                }
            }
            // 居中标题位：预留固定槽位，打开设置时显示「设置」，避免布局跳动。
            span {
                class: "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 max-w-[40%] overflow-hidden text-ellipsis whitespace-nowrap text-[13px] font-semibold text-secondary-2 pointer-events-none",
                aria_hidden: "true",
                if show_settings_modal() {
                    "设置"
                }
            }
            NativeCaptionControls {}
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
) -> Element {
    rsx! {}
}
