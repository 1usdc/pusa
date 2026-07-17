//! 桌面端应用壳层：设置/关于模态 + 标题栏，无登录门禁。
#![cfg(all(feature = "native", not(target_arch = "wasm32")))]

use dioxus::document;
use dioxus::prelude::*;
use protocol::{AboutInfoDto, LlmConfigUpsertBody};

use crate::desktop::agent;
use crate::shell::toast::{use_init_toast_ctx, use_toast, ToastViewport};
use crate::Console;

#[component]
fn DesktopSettingsPage(mut show_settings_modal: Signal<bool>) -> Element {
    let toast = use_toast();
    let mut api_key = use_signal(String::new);
    let mut base_url = use_signal(|| "https://api.openai.com/v1".to_string());
    let mut key_saved = use_signal(|| false);
    let mut load_hint = use_signal(|| None::<String>);

    use_effect(move || {
        spawn(async move {
            match agent::runtime_ctx().llm_config_get().await {
                Ok(cfg) => {
                    base_url.set(cfg.openai_v1_base);
                    key_saved.set(cfg.api_key_configured);
                    if !cfg.api_key.trim().is_empty() {
                        api_key.set(cfg.api_key);
                    }
                    load_hint.set(None);
                }
                Err(err) => load_hint.set(Some(err.to_string())),
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
                    aria_label: "关闭设置",
                    onclick: move |_| show_settings_modal.set(false),
                    "×"
                }
            }
            p { class: "ac-settings-lead",
                "用于对话补全的 OpenAI 兼容接口。密钥与 Base URL 会保存到本机桌面数据库。"
            }
            if let Some(hint) = load_hint() {
                p { class: "ac-settings-saved", "{hint}" }
            }
            label { class: "ac-settings-label", "API Key"
                input {
                    r#type: "password",
                    class: "ac-settings-input",
                    placeholder: "sk-…",
                    value: "{api_key()}",
                    oninput: move |e| api_key.set(e.value()),
                }
                if key_saved() && api_key().trim().is_empty() {
                    p { class: "ac-settings-hint ac-settings-hint--ok", "本机已保存密钥，正在加载…" }
                }
            }
            label { class: "ac-settings-label", "API Base URL（须以 /v1 结尾）"
                input {
                    r#type: "text",
                    class: "ac-settings-input",
                    placeholder: "https://api.openai.com/v1",
                    value: "{base_url()}",
                    oninput: move |e| base_url.set(e.value()),
                }
            }
            div { class: "ac-settings-actions",
                button {
                    r#type: "button",
                    class: "ac-settings-submit",
                    onclick: move |_| {
                        let api_key_value = api_key().trim().to_string();
                        let base_value = if base_url().trim().is_empty() {
                            "https://api.openai.com/v1".to_string()
                        } else {
                            base_url().trim().to_string()
                        };
                        if api_key_value.is_empty() && !key_saved() {
                            load_hint.set(Some("请填写 API Key（首次保存必填）".into()));
                            return;
                        }
                        spawn(async move {
                            match agent::runtime_ctx()
                                .llm_config_upsert(LlmConfigUpsertBody {
                                    api_key: api_key_value,
                                    openai_v1_base: base_value,
                                    prefer_custom_key: None,
                                    clear_api_key: false,
                                    clear_openai_v1_base: false,
                                })
                                .await
                            {
                                Ok(cfg) => {
                                    key_saved.set(cfg.api_key_configured);
                                    if !cfg.api_key.trim().is_empty() {
                                        api_key.set(cfg.api_key);
                                    }
                                    load_hint.set(None);
                                    show_settings_modal.set(false);
                                    toast.success("设置已保存");
                                }
                                Err(err) => load_hint.set(Some(err.to_string())),
                            }
                        });
                    },
                    "保存"
                }
            }
        }
    }
}

#[component]
fn DesktopAboutModal(mut show_about_modal: Signal<bool>) -> Element {
    let dto = AboutInfoDto {
        version: env!("CARGO_PKG_VERSION").to_string(),
        runtime_os: std::env::consts::OS.to_string(),
        runtime_arch: std::env::consts::ARCH.to_string(),
    };
    rsx! {
        div { class: "ac-settings-modal-root ac-about-modal-root",
            div {
                class: "ac-settings-modal-backdrop",
                onclick: move |_| show_about_modal.set(false),
            }
            div { class: "ac-settings-modal-dialog",
                div { class: "ac-settings-card ac-about-card",
                    div { class: "ac-settings-head",
                        h1 { class: "ac-settings-title", "关于本机" }
                        button {
                            r#type: "button",
                            class: "ac-settings-close",
                            title: "关闭",
                            aria_label: "关闭",
                            onclick: move |_| show_about_modal.set(false),
                            "×"
                        }
                    }
                    dl { class: "ac-about-dl",
                        div { class: "ac-about-row",
                            dt { "版本号" }
                            dd { "{dto.version}" }
                        }
                        div { class: "ac-about-row",
                            dt { "系统" }
                            dd { "{dto.runtime_os}" }
                        }
                        div { class: "ac-about-row",
                            dt { "架构" }
                            dd { "{dto.runtime_arch}" }
                        }
                    }
                }
            }
        }
    }
}

fn desktop_host_shell_css() -> &'static str {
    r#"
html {
    background: transparent !important;
}
body {
    margin: 0;
    min-height: 100%;
    height: 100%;
    overflow: hidden;
    overscroll-behavior: none;
    display: flex;
    flex-direction: column;
    background: transparent !important;
}
body > main#main {
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
    min-height: 0;
    height: 100%;
}
"#
}

#[component]
pub fn DesktopAppShell() -> Element {
    let show_sidebar = use_signal(|| true);
    let show_center = use_signal(|| true);
    let show_terminal = use_signal(|| false);
    let show_chat = use_signal(|| true);
    let chat_history_open = use_signal(|| false);
    let active_file_path = use_signal(|| None::<String>);
    let mut show_settings_modal = use_signal(|| false);
    let show_about_modal = use_signal(|| false);
    let show_titlebar_settings_menu = use_signal(|| false);
    let _toast_ctx = use_init_toast_ctx();

    use_effect(move || {
        let host_css = serde_json::to_string(desktop_host_shell_css()).unwrap_or_else(|_| "\"\"".to_string());
        let font_css = serde_json::to_string(&crate::source_han_sans_hwsc_font_face_css())
            .unwrap_or_else(|_| "\"\"".to_string());
        let js = format!(
            r#"(() => {{
  const upsert = (id, css) => {{
    let el = document.getElementById(id);
    if (!el) {{
      el = document.createElement('style');
      el.id = id;
      document.head.appendChild(el);
    }}
    if (el.textContent !== css) el.textContent = css;
  }};
  upsert('ac-native-host-shell-style', {host_css});
  upsert('ac-native-font-face-style', {font_css});
}})();"#,
            host_css = host_css,
            font_css = font_css,
        );
        let _ = document::eval(&js);
    });

    rsx! {
        document::Title { "Pusa AI Console" }
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
            href: crate::SOURCE_HAN_SANS_HWSC_REGULAR,
            r#as: "font",
            crossorigin: "anonymous",
        }
        document::Link {
            rel: "preload",
            href: crate::SOURCE_HAN_SANS_HWSC_BOLD,
            r#as: "font",
            crossorigin: "anonymous",
        }
        document::Link { rel: "icon", r#type: "image/x-icon", href: crate::FAVICON }
        document::Link { rel: "apple-touch-icon", sizes: "180x180", href: crate::APPLE_TOUCH_ICON }
        document::Link { rel: "stylesheet", href: crate::COLORS_CSS }
        document::Link { rel: "stylesheet", href: crate::MAIN_CSS }
        document::Link { rel: "stylesheet", href: crate::TAILWIND_CSS }
        div {
            class: "app-shell app-shell-native-frame",
            // shell 已可挂载：淡出并移除冷启动 HTML 骨架
            onmounted: move |_| {
                let _ = document::eval(
                    r#"(() => {
  const el = document.getElementById('ac-desktop-boot-skel');
  if (!el) return;
  el.classList.add('is-done');
  const remove = () => el.remove();
  el.addEventListener('transitionend', remove, { once: true });
  setTimeout(remove, 320);
})();"#,
                );
            },
            crate::desktop::chrome::NativeTitleBar {
                show_sidebar,
                show_center,
                show_terminal,
                show_chat,
                chat_history_open,
                show_settings_modal,
                show_about_modal,
                show_titlebar_settings_menu,
            }
            Console {
                show_sidebar,
                show_center,
                show_terminal,
                show_chat,
                chat_history_open,
                active_file_path,
            }
            crate::StatusBar { show_terminal, active_file_path }
            if show_settings_modal() {
                div { class: "ac-settings-modal-root",
                    div {
                        class: "ac-settings-modal-backdrop",
                        onclick: move |_| show_settings_modal.set(false),
                    }
                    div { class: "ac-settings-modal-dialog",
                        DesktopSettingsPage { show_settings_modal }
                    }
                }
            }
            if show_about_modal() {
                DesktopAboutModal { show_about_modal }
            }
            ToastViewport {}
        }
    }
}
