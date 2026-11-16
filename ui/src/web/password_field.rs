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
    fn input_class(self) -> String {
        match self {
            PasswordFieldStyle::Settings => crate::shell::style::FIELD_INPUT_PASSWORD.to_string(),
            // `ac-api-modal-input` 属 console.rs 的 API 弹窗（P5 迁移）；这里用 `!` 提高
            // 右侧内边距优先级，压过 legacy 选择器里的 padding 简写。
            PasswordFieldStyle::Modal => format!("{} pr-[2.75rem]!", crate::shell::style::API_MODAL_INPUT),
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
        div { class: "relative block w-full",
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
                class: "absolute right-[0.35rem] top-1/2 -translate-y-1/2 flex items-center justify-center w-8 h-8 p-0 border-0 rounded-[0.4rem] bg-transparent text-secondary-3 cursor-pointer hover:text-secondary-3 hover:bg-primary-1 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-[color:var(--border-color-dark)]",
                title: if revealed() { "隐藏密码" } else { "显示密码" },
                onclick: move |_| revealed.toggle(),
                if revealed() {
                    Icon {
                        icon: LdEyeOff,
                        width: 18,
                        height: 18,
                        fill: "currentColor",
                        class: "shrink-0",
                    }
                } else {
                    Icon {
                        icon: LdEye,
                        width: 18,
                        height: 18,
                        fill: "currentColor",
                        class: "shrink-0",
                    }
                }
            }
        }
    }
}
