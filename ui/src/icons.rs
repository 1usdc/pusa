//! 项目内嵌 SVG 图标（`IconShape`），避免引入 npm / 额外 icon crate。

use dioxus::prelude::*;
use dioxus_free_icons::IconShape;

// —— 侧栏模式切换（角色 / 技能 / 文件 / 插件）——
// 选中态由 Fill/Line/Bold/Filled 区分，颜色走 `.ac-sidebar-nav-item.is-active { color: var(--brand) }`。

/// Remix Icon `home-smile-2-fill` — 侧栏角色：选中。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RiHomeSmile2Fill;

impl IconShape for RiHomeSmile2Fill {
    fn view_box(&self) -> &str {
        "0 0 24 24"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M21 20C21 20.5523 20.5523 21 20 21H4C3.44772 21 3 20.5523 3 20V9.31391C3 9.00773 3.14027 8.71843 3.38065 8.52879L11.3807 2.21793C11.7438 1.93142 12.2562 1.93142 12.6193 2.21793L20.6193 8.52879C20.8597 8.71843 21 9.00773 21 9.31391V20ZM7 12C7 14.7614 9.23858 17 12 17C14.7614 17 17 14.7614 17 12H15C15 13.6569 13.6569 15 12 15C10.3431 15 9 13.6569 9 12H7Z",
            }
        }
    }
}

/// Remix Icon `home-smile-2-line` — 侧栏角色：未选中。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RiHomeSmile2Line;

impl IconShape for RiHomeSmile2Line {
    fn view_box(&self) -> &str {
        "0 0 24 24"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M19 19V9.79875L12 4.27675L5 9.79875V19H19ZM21 20C21 20.5523 20.5523 21 20 21H4C3.44772 21 3 20.5523 3 20V9.31391C3 9.00773 3.14027 8.71843 3.38065 8.52879L11.3807 2.21793C11.7438 1.93142 12.2562 1.93142 12.6193 2.21793L20.6193 8.52879C20.8597 8.71843 21 9.00773 21 9.31391V20ZM7 12H9C9 13.6569 10.3431 15 12 15C13.6569 15 15 13.6569 15 12H17C17 14.7614 14.7614 17 12 17C9.23858 17 7 14.7614 7 12Z",
            }
        }
    }
}

/// Phosphor `magic-wand-fill` — 侧栏技能：选中。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PiMagicWandFill;

impl IconShape for PiMagicWandFill {
    fn view_box(&self) -> &str {
        "0 0 256 256"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M248,152a8,8,0,0,1-8,8H224v16a8,8,0,0,1-16,0V160H192a8,8,0,0,1,0-16h16V128a8,8,0,0,1,16,0v16h16A8,8,0,0,1,248,152ZM56,72H72V88a8,8,0,0,0,16,0V72h16a8,8,0,0,0,0-16H88V40a8,8,0,0,0-16,0V56H56a8,8,0,0,0,0,16ZM184,192h-8v-8a8,8,0,0,0-16,0v8h-8a8,8,0,0,0,0,16h8v8a8,8,0,0,0,16,0v-8h8a8,8,0,0,0,0-16ZM219.31,80,80,219.31a16,16,0,0,1-22.62,0L36.68,198.63a16,16,0,0,1,0-22.63L176,36.69a16,16,0,0,1,22.63,0l20.68,20.68A16,16,0,0,1,219.31,80ZM208,68.69,187.31,48l-32,32L176,100.69Z",
            }
        }
    }
}

/// Phosphor `magic-wand-bold` — 侧栏技能：未选中。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PiMagicWandBold;

impl IconShape for PiMagicWandBold {
    fn view_box(&self) -> &str {
        "0 0 256 256"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M252,152a12,12,0,0,1-12,12H228v12a12,12,0,0,1-24,0V164H192a12,12,0,0,1,0-24h12V128a12,12,0,0,1,24,0v12h12A12,12,0,0,1,252,152ZM56,76H68V88a12,12,0,0,0,24,0V76h12a12,12,0,1,0,0-24H92V40a12,12,0,0,0-24,0V52H56a12,12,0,0,0,0,24ZM184,188h-4v-4a12,12,0,0,0-24,0v4h-4a12,12,0,0,0,0,24h4v4a12,12,0,0,0,24,0v-4h4a12,12,0,0,0,0-24ZM222.14,82.83,82.82,222.14a20,20,0,0,1-28.28,0L33.85,201.46a20,20,0,0,1,0-28.29L173.17,33.86a20,20,0,0,1,28.28,0l20.69,20.68A20,20,0,0,1,222.14,82.83ZM159,112,144,97,53.65,187.31l15,15Zm43.31-43.31-15-15L161,80l15,15Z",
            }
        }
    }
}

/// Tabler Icons `file` (outline) — 侧栏文件：未选中。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TbFile;

impl IconShape for TbFile {
    fn view_box(&self) -> &str {
        "0 0 24 24"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        ("none", user_color, "2")
    }
    fn stroke_linecap(&self) -> &str {
        "round"
    }
    fn stroke_linejoin(&self) -> &str {
        "round"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path { d: "M14 3v4a1 1 0 0 0 1 1h4" }
            path { d: "M17 21h-10a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2h7l5 5v11a2 2 0 0 1 -2 2z" }
        }
    }
}

/// Tabler Icons `file` (filled) — 侧栏文件：选中。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TbFileFilled;

impl IconShape for TbFileFilled {
    fn view_box(&self) -> &str {
        "0 0 24 24"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M12 2l.117 .007a1 1 0 0 1 .876 .876l.007 .117v4l.005 .15a2 2 0 0 0 1.838 1.844l.157 .006h4l.117 .007a1 1 0 0 1 .876 .876l.007 .117v9a3 3 0 0 1 -2.824 2.995l-.176 .005h-10a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-14a3 3 0 0 1 2.824 -2.995l.176 -.005h5z",
            }
            path { d: "M19 7h-4l-.001 -4.001z" }
        }
    }
}

/// Remix 风格四宫格（对齐 VS Code Extensions）— 侧栏插件市场：选中。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RiApps2Fill;

impl IconShape for RiApps2Fill {
    fn view_box(&self) -> &str {
        "0 0 24 24"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M4 2h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zm10 0h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zM4 14h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-4a2 2 0 0 1 2-2zm10 0h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2v-4a2 2 0 0 1 2-2z",
            }
        }
    }
}

/// Remix 风格四宫格（描边）— 侧栏插件市场：未选中。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RiApps2Line;

impl IconShape for RiApps2Line {
    fn view_box(&self) -> &str {
        "0 0 24 24"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M4 2h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zm0 2v4h4V4H4zm10-2h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zm0 2v4h4V4h-4zM4 14h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-4a2 2 0 0 1 2-2zm0 2v4h4v-4H4zm10-2h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2v-4a2 2 0 0 1 2-2zm0 2v4h4v-4h-4z",
            }
        }
    }
}

/// Octicons `file-directory` — 状态栏工作区目录名前缀。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GoFileDirectory;

impl IconShape for GoFileDirectory {
    fn view_box(&self) -> &str {
        "0 0 16 16"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M1.75 2.5a.25.25 0 00-.25.25v10.5c0 .138.112.25.25.25h12.5a.25.25 0 00.25-.25v-8.5a.25.25 0 00-.25-.25H7.5c-.55 0-1.07-.26-1.4-.7l-.9-1.2a.25.25 0 00-.2-.1H1.75zM0 2.75C0 1.784.784 1 1.75 1H5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 00.2.1h6.75c.966 0 1.75.784 1.75 1.75v8.5A1.75 1.75 0 0114.25 15H1.75A1.75 1.75 0 010 13.25V2.75z",
                fill_rule: "evenodd",
            }
        }
    }
}

/// VS Code Codicons `layout-sidebar-left-dock` — 中间栏打开时：聊天栏往左覆盖。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VscLayoutSidebarLeftDock;

impl IconShape for VscLayoutSidebarLeftDock {
    fn view_box(&self) -> &str {
        "0 0 16 16"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M9.35254 6.14844C9.15754 5.95344 8.84148 5.95344 8.64648 6.14844L7.14648 7.64844L7.14453 7.64648C6.94968 7.84133 6.94999 8.15848 7.14453 8.35352L8.64453 9.85352C8.74253 9.95052 8.87102 10 8.99902 10C9.12701 9.99999 9.25454 9.9515 9.35254 9.85352C9.54754 9.65852 9.54754 9.34148 9.35254 9.14648L8.70703 8.50098H12.5C12.776 8.50098 13 8.27697 13 8.00098C13 7.72498 12.776 7.50098 12.5 7.50098H8.70703L9.35254 6.85449C9.54753 6.65949 9.54753 6.34344 9.35254 6.14844Z",
            }
            path {
                fill_rule: "evenodd",
                clip_rule: "evenodd",
                d: "M3.5 1C2.119 1 0.999997 2.119 0.999997 3.5L1 12.5C1 13.881 2.119 15 3.5 15H12.5C13.881 15 15 13.881 15 12.5L15 3.5C15 2.119 13.881 1 12.5 1L3.5 1ZM12.5 2C13.328 2 14 2.672 14 3.5L14 12.5C14 13.328 13.328 14 12.5 14H6L6 2H12.5Z",
            }
        }
    }
}

/// VS Code Codicons `layout-sidebar-right-dock` — 中间栏收起时：聊天栏往右收缩拉出中间栏。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VscLayoutSidebarRightDock;

impl IconShape for VscLayoutSidebarRightDock {
    fn view_box(&self) -> &str {
        "0 0 16 16"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M6.64746 6.14844C6.84246 5.95344 7.15852 5.95344 7.35352 6.14844L8.85352 7.64844L8.85547 7.64648C9.05032 7.84133 9.05001 8.15848 8.85547 8.35352L7.35547 9.85352C7.25747 9.95052 7.12898 10 7.00098 10C6.87299 9.99999 6.74546 9.9515 6.64746 9.85352C6.45246 9.65852 6.45246 9.34148 6.64746 9.14648L7.29297 8.50098H3.5C3.22401 8.50098 3.00001 8.27697 3 8.00098C3 7.72498 3.224 7.50098 3.5 7.50098H7.29297L6.64746 6.85449C6.45247 6.65949 6.45247 6.34344 6.64746 6.14844Z",
            }
            path {
                fill_rule: "evenodd",
                clip_rule: "evenodd",
                d: "M12.5 1C13.881 1 15 2.119 15 3.5L15 12.5C15 13.881 13.881 15 12.5 15H3.5C2.119 15 1 13.881 1 12.5L1 3.5C1 2.119 2.119 1 3.5 1L12.5 1ZM3.5 2C2.672 2 2 2.672 2 3.5L2 12.5C2 13.328 2.672 14 3.5 14H10L10 2H3.5Z",
            }
        }
    }
}

/// VS Code Codicons `layout-panel` — 底部面板关闭时显示（点击打开）。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VscLayoutPanel;

impl IconShape for VscLayoutPanel {
    fn view_box(&self) -> &str {
        "0 0 16 16"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M15 12.5C15 13.881 13.881 15 12.5 15H3.5C2.119 15 1 13.881 1 12.5V3.5C1 2.119 2.119 1 3.5 1H12.5C13.881 1 15 2.119 15 3.5V12.5ZM2 10H14V3.5C14 2.672 13.328 2 12.5 2H3.5C2.672 2 2 2.672 2 3.5V10Z",
            }
        }
    }
}

/// VS Code Codicons `layout-panel-off` — 底部面板打开时显示（点击关闭）。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VscLayoutPanelOff;

impl IconShape for VscLayoutPanelOff {
    fn view_box(&self) -> &str {
        "0 0 16 16"
    }
    fn xmlns(&self) -> &str {
        "http://www.w3.org/2000/svg"
    }
    fn fill_and_stroke<'a>(&self, user_color: &'a str) -> (&'a str, &'a str, &'a str) {
        (user_color, "none", "0")
    }
    fn stroke_linecap(&self) -> &str {
        "butt"
    }
    fn stroke_linejoin(&self) -> &str {
        "miter"
    }
    fn child_elements(&self) -> Element {
        rsx! {
            path {
                d: "M12.5 1H3.5C2.122 1 1 2.121 1 3.5V12.5C1 13.879 2.122 15 3.5 15H12.5C13.878 15 15 13.879 15 12.5V3.5C15 2.121 13.878 1 12.5 1ZM14 12.5C14 13.327 13.327 14 12.5 14H3.5C2.673 14 2 13.327 2 12.5V11H14V12.5ZM14 10H2V3.5C2 2.673 2.673 2 3.5 2H12.5C13.327 2 14 2.673 14 3.5V10Z",
            }
        }
    }
}
