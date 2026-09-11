//! 桌面端应用壳层：设置/关于模态 + 标题栏，无登录门禁。
#![cfg(all(feature = "native", not(target_arch = "wasm32")))]

use dioxus::document;
use dioxus::prelude::*;
use protocol::{AboutInfoDto, LlmConfigDto, LlmCredentialUpsertBody};

use crate::desktop::agent;
use crate::shell::toast::{use_init_toast_ctx, use_toast, ToastViewport};
use crate::shell::{
    activate_id_after_toggle, enabled_after_delete, enabled_after_toggle, enabled_after_upsert,
    format_llm_settings_error, overlay_enabled_credential_ids, LlmCredentialsPanel,
    LlmModelsRefresh,
};
use crate::Console;

fn apply_cfg(
    cfg: LlmConfigDto,
    mut credentials: Signal<Vec<protocol::LlmCredentialDto>>,
    mut active_id: Signal<String>,
    mut enabled_ids: Signal<Vec<String>>,
) {
    let known: Vec<String> = cfg.credentials.iter().map(|c| c.id.clone()).collect();
    let enabled = overlay_enabled_credential_ids(&known, &cfg.active_credential_id);
    credentials.set(cfg.credentials);
    active_id.set(cfg.active_credential_id);
    enabled_ids.set(enabled);
}

#[component]
fn DesktopSettingsPage(mut show_settings_modal: Signal<bool>) -> Element {
    let toast = use_toast();
    let credentials = use_signal(Vec::<protocol::LlmCredentialDto>::new);
    let active_id = use_signal(String::new);
    let mut enabled_ids = use_signal(Vec::<String>::new);
    let mut load_hint = use_signal(|| None::<String>);
    let refresh = use_context::<LlmModelsRefresh>().0;

    use_effect(move || {
        spawn(async move {
            match agent::runtime_ctx().llm_config_get().await {
                Ok(cfg) => {
                    apply_cfg(cfg, credentials, active_id, enabled_ids);
                    load_hint.set(None);
                }
                Err(err) => load_hint.set(Some(format_llm_settings_error(&err.to_string()))),
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
                    aria_label: "关闭设置",
                    onclick: move |_| show_settings_modal.set(false),
                    "×"
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
                        match agent::runtime_ctx().llm_credential_upsert(body).await {
                            Ok(cfg) => {
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
                            Err(err) => {
                                load_hint.set(Some(format_llm_settings_error(&err.to_string())))
                            }
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
                        match agent::runtime_ctx().llm_credential_upsert(body).await {
                            Ok(cfg) => {
                                apply_cfg(cfg, credentials, active_id, enabled_ids);
                                load_hint.set(None);
                                toast.success("已更新名称");
                            }
                            Err(err) => {
                                load_hint.set(Some(format_llm_settings_error(&err.to_string())))
                            }
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
                            match agent::runtime_ctx().llm_credential_activate(&activate).await {
                                Ok(cfg) => {
                                    apply_cfg(cfg, credentials, active_id, enabled_ids);
                                    enabled_ids.set(next);
                                    load_hint.set(None);
                                }
                                Err(err) => {
                                    load_hint.set(Some(format_llm_settings_error(&err.to_string())))
                                }
                            }
                        }
                    });
                },
                on_delete: move |id: String| {
                    spawn(async move {
                        let prev_enabled = enabled_ids();
                        match agent::runtime_ctx().llm_credential_delete(&id).await {
                            Ok(cfg) => {
                                let known: Vec<String> =
                                    cfg.credentials.iter().map(|c| c.id.clone()).collect();
                                apply_cfg(cfg, credentials, active_id, enabled_ids);
                                enabled_ids.set(enabled_after_delete(&prev_enabled, &known, refresh));
                                load_hint.set(None);
                                toast.success("已删除密钥");
                            }
                            Err(err) => {
                                load_hint.set(Some(format_llm_settings_error(&err.to_string())))
                            }
                        }
                    });
                },
            }
        }
    }
}

#[component]
fn DesktopAboutModal(mut show_about_modal: Signal<bool>) -> Element {
    let dto = AboutInfoDto {
        version: crate::version::app_version(),
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
    let active_role_name = use_signal(String::new);
    let mut show_settings_modal = use_signal(|| false);
    let show_about_modal = use_signal(|| false);
    let show_titlebar_settings_menu = use_signal(|| false);
    let _toast_ctx = use_init_toast_ctx();
    let llm_models_refresh = use_signal(|| 0u32);
    use_context_provider(|| LlmModelsRefresh(llm_models_refresh));
    let open_browser_tick = use_signal(|| 0u64);
    use_context_provider(|| crate::shell::browser::OpenBrowserTick(open_browser_tick));

    use_effect(move || {
        crate::shell::theme::restore_on_launch();
    });

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
        document::Title { "Pusa" }
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
  const started = window.__pusaBootAt || Date.now();
  const finish = () => {
    el.classList.add('is-done');
    const remove = () => { if (el.parentNode) el.remove(); };
    el.addEventListener('transitionend', remove, { once: true });
    setTimeout(remove, 400);
  };
  const waitCss = () => new Promise((resolve) => {
    const ready = () => [...document.styleSheets].some((s) => {
      try { return !!(s.href && /main|colors|tailwind/i.test(s.href)); }
      catch (_) { return false; }
    });
    if (ready()) { resolve(); return; }
    const t0 = Date.now();
    const tick = () => {
      if (ready() || Date.now() - t0 > 1600) resolve();
      else setTimeout(tick, 40);
    };
    tick();
  });
  (async () => {
    await waitCss();
    const left = Math.max(0, 420 - (Date.now() - started));
    if (left) await new Promise((r) => setTimeout(r, left));
    requestAnimationFrame(() => requestAnimationFrame(finish));
  })();
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
                active_role_name,
            }
            crate::StatusBar { show_terminal, active_file_path, active_role_name }
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
