//! 设置整页外壳：标准「左侧菜单 + 右侧内容」布局，Web / 桌面端共用。
//!
//! - 左侧导航（[`SettingsSection`]）：通用设置 / 模型设置 / 版本信息，底部固定返回按钮；
//! - 右侧内容：由调用方按 `active` 分区渲染（`children`）；
//! - 页头仅保留居中的「设置」标题；
//! - 另含键值行等公共子组件。

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdBox, LdChevronLeft, LdInfo, LdSlidersHorizontal};
use dioxus_free_icons::Icon;

/// 设置页左侧菜单的分区。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsSection {
    /// 通用：外观 / 主题等基础偏好。
    General,
    /// 模型设置：自定义模型供应商（Base URL / API Key / 模型列表）。
    ModelSettings,
    /// 版本信息：版本 / 系统 / 架构。
    About,
}

impl SettingsSection {
    /// 菜单项短标签。
    pub fn label(self) -> &'static str {
        match self {
            Self::General => "通用设置",
            Self::ModelSettings => "模型设置",
            Self::About => "版本信息",
        }
    }

    /// 右侧内容区的标题。
    pub fn title(self) -> &'static str {
        match self {
            Self::General => "通用设置",
            Self::ModelSettings => "模型设置",
            Self::About => "版本信息",
        }
    }
}

/// 全部分区（按展示顺序）。
pub const SETTINGS_SECTIONS: [SettingsSection; 3] = [
    SettingsSection::ModelSettings,
    SettingsSection::General,
    SettingsSection::About,
];

/// 左侧菜单分区图标。
#[component]
fn SettingsNavIcon(section: SettingsSection) -> Element {
    rsx! {
        match section {
            SettingsSection::General => rsx! {
                Icon { icon: LdSlidersHorizontal, width: 15, height: 15, fill: "currentColor" }
            },
            SettingsSection::ModelSettings => rsx! {
                Icon { icon: LdBox, width: 15, height: 15, fill: "currentColor" }
            },
            SettingsSection::About => rsx! {
                Icon { icon: LdInfo, width: 15, height: 15, fill: "currentColor" }
            },
        }
    }
}

/// 单行键值信息（页脚 / 详情行）。
#[derive(Clone, PartialEq)]
pub struct SettingsInfoRow {
    pub label: String,
    pub value: String,
}

/// 单行键值信息的只读展示。
#[component]
pub fn SettingsInfoRowView(row: SettingsInfoRow) -> Element {
    rsx! {
        div { class: "ac-settings-info-row",
            dt { "{row.label}" }
            dd { "{row.value}" }
        }
    }
}

/// 通用设置的「外观」行：亮色 / 暗色主题分段选择（选择即保存生效）。
#[component]
pub fn ThemeSettingRow(mut ui_theme: Signal<crate::shell::theme::UiTheme>) -> Element {
    let current = ui_theme();
    rsx! {
        div { class: "ac-settings-row",
            div { class: "ac-settings-row-main",
                div { class: "ac-settings-row-title", "外观" }
                p { class: "ac-settings-row-desc", "切换应用的亮色 / 暗色主题，选择后立即生效并自动保存。" }
            }
            div { class: "ac-settings-provider", role: "tablist", aria_label: "主题",
                button {
                    r#type: "button",
                    role: "tab",
                    class: if current == crate::shell::theme::UiTheme::Light {
                        "ac-settings-provider-btn is-active"
                    } else {
                        "ac-settings-provider-btn"
                    },
                    aria_selected: current == crate::shell::theme::UiTheme::Light,
                    onclick: move |_| {
                        let next = crate::shell::theme::UiTheme::Light;
                        crate::shell::theme::set(next);
                        ui_theme.set(next);
                    },
                    "亮色"
                }
                button {
                    r#type: "button",
                    role: "tab",
                    class: if current == crate::shell::theme::UiTheme::Dark {
                        "ac-settings-provider-btn is-active"
                    } else {
                        "ac-settings-provider-btn"
                    },
                    aria_selected: current == crate::shell::theme::UiTheme::Dark,
                    onclick: move |_| {
                        let next = crate::shell::theme::UiTheme::Dark;
                        crate::shell::theme::set(next);
                        ui_theme.set(next);
                    },
                    "暗色"
                }
            }
        }
    }
}

/// 通用设置的「字体大小」行：小 / 中 / 大分段选择（选择即保存生效）。
#[component]
pub fn FontSizeSettingRow(mut font_size: Signal<crate::shell::theme::FontSize>) -> Element {
    let current = font_size();
    rsx! {
        div { class: "ac-settings-row",
            div { class: "ac-settings-row-main",
                div { class: "ac-settings-row-title", "字体大小" }
                p { class: "ac-settings-row-desc", "调整应用界面的文字大小，选择后立即生效并自动保存。" }
            }
            div { class: "ac-settings-provider", role: "tablist", aria_label: "字体大小",
                for size in [
                    crate::shell::theme::FontSize::Small,
                    crate::shell::theme::FontSize::Medium,
                    crate::shell::theme::FontSize::Large,
                ] {
                    button {
                        key: "{size.as_attr()}",
                        r#type: "button",
                        role: "tab",
                        class: if current == size {
                            "ac-settings-provider-btn is-active"
                        } else {
                            "ac-settings-provider-btn"
                        },
                        aria_selected: current == size,
                        onclick: move |_| {
                            crate::shell::theme::set_font_size(size);
                            font_size.set(size);
                        },
                        "{size.label()}"
                    }
                }
            }
        }
    }
}

/// 设置整页外壳：标准「左侧菜单 + 右侧内容」布局。
///
/// - `active`：当前分区（受控信号，父级按它渲染右侧内容）；
/// - `on_close`：返回上一界面（隐藏设置页）；
/// - `children`：右侧内容（按分区条件渲染）。
#[component]
pub fn SettingsNavShell(
    mut active: Signal<SettingsSection>,
    on_close: EventHandler<()>,
    children: Element,
) -> Element {
    let section_title = active().title();
    #[cfg(any(
        all(feature = "native", not(target_arch = "wasm32")),
        all(target_arch = "wasm32", feature = "web")
    ))]
    let version_title_clicks = use_signal(|| (0u32, 0.0f64));
    #[cfg(any(
        all(feature = "native", not(target_arch = "wasm32")),
        all(target_arch = "wasm32", feature = "web")
    ))]
    let developer_mode = use_context::<crate::shell::dev_mode::DeveloperMode>().0;
    rsx! {
        div { class: "ac-settings-page",
            // Web 无系统标题栏，标题保留在页内顶部；桌面端由原生标题栏居中显示。
            if cfg!(all(target_arch = "wasm32", feature = "web")) {
                header { class: "ac-settings-page-head",
                    h1 { class: "ac-settings-page-title", "设置" }
                }
            }
            div { class: "ac-settings-page-body",
                nav { class: "ac-settings-nav", "aria-label": "设置分区",
                    // 返回按钮单独置于导航最上方。
                    div { class: "ac-settings-nav-top",
                        button {
                            r#type: "button",
                            class: "ac-settings-page-back",
                            title: "返回",
                            aria_label: "返回",
                            onclick: move |_| on_close.call(()),
                            Icon {
                                icon: LdChevronLeft,
                                width: 18,
                                height: 18,
                                fill: "currentColor",
                            }
                            span { "返回" }
                        }
                    }
                    for section in SETTINGS_SECTIONS {
                        {
                            let is_active = active() == section;
                            rsx! {
                                button {
                                    key: "{section.label()}",
                                    r#type: "button",
                                    class: if is_active {
                                        "ac-settings-nav-item is-active"
                                    } else {
                                        "ac-settings-nav-item"
                                    },
                                    aria_current: is_active,
                                    onclick: move |_| active.set(section),
                                    SettingsNavIcon { section }
                                    span { class: "ac-settings-nav-label", "{section.label()}" }
                                }
                            }
                        }
                    }
                }
                section { class: "ac-settings-content",
                    header { class: "ac-settings-content-head",
                        h2 {
                            class: "ac-settings-content-title",
                            title: if active() == SettingsSection::About {
                                "连续点击三次可开关开发者模式"
                            } else {
                                ""
                            },
                            onclick: move |_| {
                                if active() != SettingsSection::About {
                                    return;
                                }
                                #[cfg(any(
                                    all(feature = "native", not(target_arch = "wasm32")),
                                    all(target_arch = "wasm32", feature = "web")
                                ))]
                                crate::shell::dev_mode::register_triple_click(
                                    version_title_clicks,
                                    move || crate::shell::dev_mode::toggle(developer_mode),
                                );
                            },
                            "{section_title}"
                        }
                    }
                    div { class: "ac-settings-content-body",
                        {children}
                    }
                }
            }
        }
    }
}
