//! Dioxus UI：组件、静态资源（见 `assets/`）与 shell。
#![allow(clippy::missing_docs_in_private_items)]

use dioxus::prelude::*;

pub mod shell;
pub use shell::{Console, StatusBar};

mod version;
mod icons;
mod chat;
mod custom_chat_models;
#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub mod components;
mod persona;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod desktop;

/// 桌面端 `pusapreview://` 自定义协议：给 HTML 文件夹内置预览加载相对资源。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn serve_html_preview(uri: &str) -> Result<(&'static str, Vec<u8>), u16> {
    desktop::html_preview::serve_request(uri)
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub const HTML_PREVIEW_PROTOCOL: &str = desktop::html_preview::PROTOCOL_NAME;

// favicon：浏览器原生 .ico（多尺寸打包，桌面浏览器最兼容）
#[cfg(any(all(target_arch = "wasm32", feature = "web"), all(not(target_arch = "wasm32"), feature = "native")))]
const FAVICON: Asset = asset!("/assets/icons/favicon.ico");
// apple-touch-icon：iOS Safari「添加到主屏」用，挂最大的 180x180，iOS 会按需缩放
#[cfg(any(all(target_arch = "wasm32", feature = "web"), all(not(target_arch = "wasm32"), feature = "native")))]
const APPLE_TOUCH_ICON: Asset = asset!("/assets/icons/apple-touch-icon-180x180.png");
#[cfg(any(all(target_arch = "wasm32", feature = "web"), all(not(target_arch = "wasm32"), feature = "native")))]
const COLORS_CSS: Asset = asset!("/assets/css/colors.css");
#[cfg(any(all(target_arch = "wasm32", feature = "web"), all(not(target_arch = "wasm32"), feature = "native")))]
const MAIN_CSS: Asset = asset!("/assets/css/main.css");
#[cfg(all(target_arch = "wasm32", feature = "web"))]
const DX_COMPONENTS_THEME_CSS: Asset = asset!("/assets/css/dx-components-theme.css");
#[cfg(any(all(target_arch = "wasm32", feature = "web"), all(not(target_arch = "wasm32"), feature = "native")))]
const TAILWIND_CSS: Asset = asset!("/assets/css/tailwind.css");

/// 中文正文：Source Han Sans HW SC Regular。
#[cfg(any(all(target_arch = "wasm32", feature = "web"), all(not(target_arch = "wasm32"), feature = "native")))]
const SOURCE_HAN_SANS_HWSC_REGULAR: Asset = asset!("/assets/fonts/SourceHanSansHWSC-Regular.otf");
/// 中文加粗：Source Han Sans HW SC Bold。
#[cfg(any(all(target_arch = "wasm32", feature = "web"), all(not(target_arch = "wasm32"), feature = "native")))]
const SOURCE_HAN_SANS_HWSC_BOLD: Asset = asset!("/assets/fonts/SourceHanSansHWSC-Bold.otf");

/// 将 `asset!` 解析后的 URL 写入 `@font-face`，与 `rel=preload` 同源，避免 `main.css` 写死 `/assets/fonts/…` 在 dx 下 404/HTML。
#[cfg(any(all(target_arch = "wasm32", feature = "web"), all(not(target_arch = "wasm32"), feature = "native")))]
fn source_han_sans_hwsc_font_face_css() -> String {
    format!(
        r#"@font-face{{font-family:"Source Han Sans HW SC";src:url("{regular}")format("opentype");font-weight:400;font-style:normal;font-display:swap;}}@font-face{{font-family:"Source Han Sans HW SC";src:url("{bold}")format("opentype");font-weight:700;font-style:normal;font-display:swap;}}"#,
        regular = SOURCE_HAN_SANS_HWSC_REGULAR,
        bold = SOURCE_HAN_SANS_HWSC_BOLD,
    )
}


#[cfg(all(
    feature = "native",
    not(target_arch = "wasm32"),
    not(target_os = "linux")
))]
#[allow(dead_code)]
fn native_host_shell_style() -> Element {
    rsx! {
        crate::desktop::chrome::NativeHostShellStyle {}
    }
}

#[cfg(not(all(
    feature = "native",
    not(target_arch = "wasm32"),
    not(target_os = "linux")
)))]
#[allow(dead_code)]
fn native_host_shell_style() -> Element {
    rsx! {}
}

/// Web（WASM）：资产与样式；标题栏在 [`shell::Console`] 右侧聊天栏顶部。
///
/// 同时在此处一次性挂载：
/// - [`crate::shell::toast::use_init_toast_ctx`]：注入全局 toast 上下文，让子组件通过
///   `use_toast()` 即可派发四种语义的轻提示；
/// - [`crate::shell::toast::ToastViewport`]：渲染所有 toast 的视口（顶部居中堆叠）。
#[cfg(all(target_arch = "wasm32", feature = "web"))]
#[component]
fn WebAppShell() -> Element {
    use crate::shell::toast::{use_init_toast_ctx, ToastViewport};
    use crate::web::about::WebAboutModal;
    use crate::web::settings::WebSettingsPage;

    let show_sidebar = use_signal(|| true);
    let show_center = use_signal(|| true);
    let show_terminal = use_signal(|| false);
    let show_chat = use_signal(|| true);
    let chat_history_open = use_signal(|| false);
    let active_file_path = use_signal(|| None::<String>);
    let active_role_name = use_signal(String::new);
    let mut show_settings_modal = use_signal(|| false);
    let show_about_modal = use_signal(|| false);
    let show_titlebar_settings_menu = use_signal(|| false);
    let developer_mode = use_signal(|| crate::web::dev_mode::get());
    // 必须在子组件首次 `use_toast()` 之前调用：把全局 toast ctx 注入到 Dioxus context。
    let _toast_ctx = use_init_toast_ctx();
    let llm_models_refresh = use_signal(|| 0u32);
    use_context_provider(|| crate::shell::LlmModelsRefresh(llm_models_refresh));

    use_effect(move || {
        crate::web::dev_mode::sync_body_to(developer_mode());
    });

    use_effect(move || {
        crate::shell::theme::restore_on_launch();
    });

    use_context_provider(|| crate::web::ShellChromeCtx {
        show_settings_modal,
        show_about_modal,
        show_titlebar_settings_menu,
        developer_mode,
    });

    rsx! {
        document::Title { "Pusa" }
        {native_host_shell_style()}
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap",
        }
        document::Link {
            rel: "stylesheet",
            href: "https://cdnjs.cloudflare.com/ajax/libs/simple-line-icons/2.5.5/css/simple-line-icons.min.css",
        }
        document::Link {
            rel: "preload",
            href: SOURCE_HAN_SANS_HWSC_REGULAR,
            r#as: "font",
            crossorigin: "anonymous",
        }
        document::Link {
            rel: "preload",
            href: SOURCE_HAN_SANS_HWSC_BOLD,
            r#as: "font",
            crossorigin: "anonymous",
        }
        document::Style { "{source_han_sans_hwsc_font_face_css()}" }
        document::Link { rel: "icon", r#type: "image/x-icon", href: FAVICON }
        document::Link { rel: "apple-touch-icon", sizes: "180x180", href: APPLE_TOUCH_ICON }
        document::Link { rel: "stylesheet", href: COLORS_CSS }
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_THEME_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        div {
            class: "app-shell",
            Console {
                show_sidebar,
                show_center,
                show_terminal,
                show_chat,
                chat_history_open,
                active_file_path,
                active_role_name,
            }
            StatusBar { show_terminal, active_file_path, active_role_name }
            if show_settings_modal() {
                div { class: "ac-settings-modal-root",
                    div {
                        class: "ac-settings-modal-backdrop",
                        onclick: move |_| show_settings_modal.set(false),
                    }
                    div { class: "ac-settings-modal-dialog",
                        WebSettingsPage { show_settings_modal }
                    }
                }
            }
            if show_about_modal() {
                WebAboutModal { show_about_modal }
            }
            ToastViewport {}
        }
    }
}

#[component]
pub fn App() -> Element {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        return rsx! {
            WebAppShell {}
        };
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    {
        return rsx! {
            crate::desktop::auth::DesktopAppShell {}
        };
    }

    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(not(target_arch = "wasm32"), feature = "native")
    )))]
    {
        rsx! {}
    }
}
