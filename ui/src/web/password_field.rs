//! 密码输入右侧「眼睛」显隐切换（Lucide）。登录 / 注册页配合 `dx-components` 的 `dx-input` 样式。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdEye, LdEyeOff};
use dioxus_free_icons::Icon;
use dioxus_primitives::dioxus_attributes::attributes;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PasswordFieldStyle {
    Settings,
    Modal,
}

impl PasswordFieldStyle {
    fn input_class(self) -> &'static str {
        match self {
            PasswordFieldStyle::Settings => "ac-settings-input",
            PasswordFieldStyle::Modal => "ac-api-modal-input",
        }
    }
}

/// 受控密码行：右侧图标切换 `text` / `password`。
///
/// `disabled=true` 时输入框只读、`oninput` 不再回写信号（用于显示「来自平台
/// 的密钥」这种场景：用户能眼睛切换查看，但改不动），同时 input 加上 HTML 原生
/// `disabled` 属性以拿到一致的视觉态。
#[component]
pub fn AcPasswordInput(
    placeholder: &'static str,
    mut value: Signal<String>,
    variant: PasswordFieldStyle,
    #[props(default)] field_id: Option<&'static str>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(default)] disabled: bool,
) -> Element {
    let mut revealed = use_signal(|| false);
    let input_cls = variant.input_class();
    let id_attrs = field_id
        .map(|id| attributes!(input { id: id }))
        .unwrap_or_default();

    rsx! {
        div { class: "ac-password-field",
            input {
                r#type: if revealed() { "text" } else { "password" },
                class: "{input_cls}",
                placeholder: "{placeholder}",
                value: "{value()}",
                disabled,
                readonly: disabled,
                oninput: move |ev| {
                    if !disabled {
                        value.set(ev.value());
                    }
                },
                onkeydown: move |ev| {
                    if let Some(handler) = &onkeydown {
                        handler.call(ev);
                    }
                },
                ..id_attrs,
            }
            button {
                r#type: "button",
                class: "ac-password-toggle",
                title: if revealed() { "隐藏密码" } else { "显示密码" },
                onclick: move |_| revealed.toggle(),
                if revealed() {
                    Icon {
                        icon: LdEyeOff,
                        width: 18,
                        height: 18,
                        fill: "currentColor",
                        class: "ac-password-toggle-icon",
                    }
                } else {
                    Icon {
                        icon: LdEye,
                        width: 18,
                        height: 18,
                        fill: "currentColor",
                        class: "ac-password-toggle-icon",
                    }
                }
            }
        }
    }
}
