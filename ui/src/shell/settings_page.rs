//! 设置整页外壳：标准「左侧菜单 + 右侧内容」布局，Web / 桌面端共用。
//!
//! - 左侧导航（[`SettingsSection`]）：通用 / 模型设置 / 关于；
//! - 右侧内容：由调用方按 `active` 分区渲染（`children`）；
//! - 页头保留返回按钮与主题快切按钮；
//! - 另含「关于本机」折叠卡片与键值行等公共子组件。

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdChevronLeft, LdInfo, LdBox, LdPalette, LdSlidersHorizontal,
};
use dioxus_free_icons::Icon;

/// 设置页左侧菜单的分区。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsSection {
    /// 通用：外观 / 主题等基础偏好。
    General,
    /// 模型设置：自定义模型供应商（Base URL / API Key / 模型列表）。
    ModelSettings,
    /// 关于本机：版本 / 系统 / 架构。
    About,
}

impl SettingsSection {
    /// 菜单项短标签。
    pub fn label(self) -> &'static str {
        match self {
            Self::General => "通用",
            Self::ModelSettings => "模型设置",
            Self::About => "关于",
        }
    }

    /// 右侧内容区的标题。
    pub fn title(self) -> &'static str {
        match self {
            Self::General => "通用设置",
            Self::ModelSettings => "模型设置",
            Self::About => "关于本机",
        }
    }
}

/// 全部分区（按展示顺序）。
pub const SETTINGS_SECTIONS: [SettingsSection; 3] = [
    SettingsSection::General,
    SettingsSection::ModelSettings,
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

/// 展开状态受控的关于卡片（父级可在展开时懒加载数据）。
#[component]
pub fn SettingsAboutSectionWrapper(
    mut open: Signal<bool>,
    rows: Vec<SettingsInfoRow>,
) -> Element {
    // 三连击彩蛋仅在桌面端启用；Web 端无 DeveloperMode 上下文。
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    let triple_title = use_signal(|| (0u32, 0.0f64));
    // 三连击彩蛋仅在桌面端启用；Web 端无 DeveloperMode 上下文。
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    let developer_mode = use_context::<crate::shell::dev_mode::DeveloperMode>().0;
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    let triple_click = move || {
        crate::shell::dev_mode::register_triple_click(triple_title, move || {
            crate::shell::dev_mode::toggle(developer_mode);
        });
    };
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    let triple_click = || {};
    rsx! {
        section { class: "ac-settings-about",
            button {
                r#type: "button",
                class: "ac-settings-about-toggle",
                aria_expanded: open(),
                title: "连续点击三次可开关开发者模式",
                onclick: move |_| {
                    open.toggle();
                    triple_click();
                },
                span { class: "ac-settings-about-icon",
                    Icon {
                        icon: LdInfo,
                        width: 14,
                        height: 14,
                        fill: "currentColor",
                    }
                }
                span { class: "ac-settings-about-text", "关于本机" }
                span { class: "ac-settings-about-arrow", aria_hidden: "true",
                    if open() { "▾" } else { "▸" }
                }
            }
            if open() {
                dl { class: "ac-settings-about-dl",
                    for row in rows {
                        SettingsInfoRowView { row }
                    }
                }
            }
        }
    }
}

/// 通用设置的「外观」卡片：亮色 / 暗色主题分段选择（选择即保存生效）。
#[component]
pub fn ThemePickerCard(mut ui_theme: Signal<crate::shell::theme::UiTheme>) -> Element {
    let current = ui_theme();
    rsx! {
        div { class: "ac-settings-card ac-settings-card--wide ac-settings-card--page",
            div { class: "ac-settings-head",
                h1 { class: "ac-settings-title", "外观" }
            }
            label { class: "ac-settings-label", "主题" }
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
            p { class: "ac-settings-lead", "切换应用的亮色 / 暗色主题，选择后立即生效并自动保存。" }
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
    rsx! {
        div { class: "ac-settings-page",
            header { class: "ac-settings-page-head",
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
                h1 { class: "ac-settings-page-title", "设置" }
                div { class: "ac-settings-page-head-spacer" }
                button {
                    r#type: "button",
                    class: "ac-settings-page-theme-btn",
                    title: "切换亮色 / 暗色主题",
                    aria_label: "切换亮色 / 暗色主题",
                    onclick: move |_| {
                        crate::shell::theme::toggle();
                    },
                    Icon {
                        icon: LdPalette,
                        width: 16,
                        height: 16,
                        fill: "currentColor",
                    }
                }
            }
            div { class: "ac-settings-page-body",
                nav { class: "ac-settings-nav", "aria-label": "设置分区",
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
                        h2 { class: "ac-settings-content-title", "{section_title}" }
                    }
                    div { class: "ac-settings-content-body",
                        {children}
                    }
                }
            }
        }
    }
}
