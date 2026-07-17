//! 全局轻提示（Toast）系统。
//!
//! 设计：
//! - 根组件（Web / Desktop shell）调一次 [`use_init_toast_ctx`] 创建队列并通过 Dioxus
//!   `use_context_provider` 注入；任何子组件用 [`use_toast`] 取出 [`ToastCtx`] 即可派发
//!   `success / error / warning / info` 四种语义的胶囊提示。
//! - 多条 toast 在 [`ToastViewport`] 里纵向堆叠，按 `id` 顺序渲染；每条在
//!   [`AUTO_DISMISS_MS`] 毫秒后自动从队列里移除，多条互不影响。
//! - [`ToastCtx`] 内部由两个 `Signal` 组成（队列 + 自增 id），整体 `Copy`，可以无副作用
//!   地被 `onclick` / `spawn` 等闭包 `move` 捕获后直接调用。
//!
//! 用法：
//!
//! ```ignore
//! let toast = crate::shell::toast::use_toast();
//! toast.success("设置已保存");
//! toast.error("无法连接后端");
//! toast.warning("当前网络不稳定");
//! toast.info("已切换到 ETH 主网");
//! ```

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdCircleCheck, LdCircleX, LdInfo, LdTriangleAlert,
};
use dioxus_free_icons::Icon;

/// 每条 toast 的显示时长。设计上比标准 1500ms 略长，便于多条堆叠时用户也能看完。
const AUTO_DISMISS_MS: u32 = 3000;

/// Toast 语义类别。映射到 CSS 修饰符 `.ac-toast--{success|error|warning|info}`。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToastKind {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastKind {
    fn css_modifier(self) -> &'static str {
        match self {
            ToastKind::Success => "success",
            ToastKind::Error => "error",
            ToastKind::Warning => "warning",
            ToastKind::Info => "info",
        }
    }
}

/// 队列中的一条 toast。`id` 单调自增，作为 `for` 渲染时的 `key`，也用于 dismiss 回调
/// 精确匹配要移除的项（避免「先派发先消失」错位）。
#[derive(Clone, PartialEq, Eq, Debug)]
struct ToastItem {
    id: u64,
    kind: ToastKind,
    message: String,
}

/// Toast 全局上下文。
///
/// 通过 [`use_init_toast_ctx`] 在根组件创建一次，[`use_toast`] 在任意子组件取出。
/// 字段都是 `Signal`（自身 `Copy`），整个结构也实现 `Copy`，可以直接在闭包里使用。
#[derive(Clone, Copy)]
pub struct ToastCtx {
    items: Signal<Vec<ToastItem>>,
    next_id: Signal<u64>,
}

impl ToastCtx {
    /// 把一条 toast 推入队列，并安排 [`AUTO_DISMISS_MS`] 毫秒后自动移除。
    ///
    /// 派发是同步的（`Signal::write` 立刻触发 reactive 更新），所以视口会在下一帧渲染新条目。
    pub fn show(self, kind: ToastKind, message: impl Into<String>) {
        let mut items = self.items;
        let mut next_id = self.next_id;
        let id = next_id();
        next_id.set(id + 1);
        items.write().push(ToastItem {
            id,
            kind,
            message: message.into(),
        });

        let mut items_drop = self.items;
        spawn(async move {
            toast_sleep_ms(AUTO_DISMISS_MS).await;
            items_drop.write().retain(|t| t.id != id);
        });
    }

    /// 绿色「成功」提示。
    pub fn success(self, message: impl Into<String>) {
        self.show(ToastKind::Success, message)
    }

    /// 红色「错误」提示。
    pub fn error(self, message: impl Into<String>) {
        self.show(ToastKind::Error, message)
    }

    /// 黄色「警告」提示。
    pub fn warning(self, message: impl Into<String>) {
        self.show(ToastKind::Warning, message)
    }

    /// 蓝色「信息」提示。
    pub fn info(self, message: impl Into<String>) {
        self.show(ToastKind::Info, message)
    }
}

#[cfg(target_arch = "wasm32")]
async fn toast_sleep_ms(ms: u32) {
    gloo_timers::future::TimeoutFuture::new(ms).await;
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
async fn toast_sleep_ms(ms: u32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
async fn toast_sleep_ms(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

/// 由根组件调用：创建 toast 队列并注入到 Dioxus context。
///
/// 必须在子组件第一次 [`use_toast`] 之前调用，因此最自然的位置是
/// Web / Desktop AppShell 顶部、其它 `use_signal` 之后。
pub fn use_init_toast_ctx() -> ToastCtx {
    let items = use_signal(Vec::<ToastItem>::new);
    let next_id = use_signal(|| 0u64);
    let ctx = ToastCtx { items, next_id };
    use_context_provider(|| ctx);
    ctx
}

/// 任何子组件中读取已注入的 [`ToastCtx`]。若上层未先调 [`use_init_toast_ctx`]
/// 则会 panic（Dioxus `use_context` 找不到 provider 时的默认行为）。
pub fn use_toast() -> ToastCtx {
    use_context::<ToastCtx>()
}

/// Toast 视口：固定在视口顶部居中，纵向堆叠当前所有 toast。
///
/// 由根组件渲染一次即可；其本身不持有状态，只是 reactive 地遍历 `ctx.items`。
#[component]
pub fn ToastViewport() -> Element {
    let ctx = use_toast();
    // `ctx.items` 是 `Signal<Vec<_>>`，字段名与 method 同名时 rustc 会把 `ctx.items()`
    // 解析为方法调用，因此显式 `.read().clone()` 拿 owned 快照后再迭代。
    let snapshot = ctx.items.read().clone();
    rsx! {
        div { class: "ac-toast-viewport",
            for item in snapshot {
                div {
                    key: "{item.id}",
                    class: "ac-toast ac-toast--{item.kind.css_modifier()}",
                    span { class: "ac-toast__icon", aria_hidden: "true",
                        {render_kind_icon(item.kind)}
                    }
                    span { class: "ac-toast__msg", "{item.message}" }
                }
            }
        }
    }
}

/// 根据 toast 语义渲染左侧 16x16 的 Lucide 线条图标。
///
/// Lucide 在 `dioxus_free_icons` 中 `fill_and_stroke` 返回 `("none", user_color, "2")`，
/// 因此传 `fill: "currentColor"` 实际等价于 `stroke="currentColor"`，颜色会跟随
/// `.ac-toast--*` 修饰符里设置的 `color` 自动变换。
fn render_kind_icon(kind: ToastKind) -> Element {
    match kind {
        ToastKind::Success => rsx! {
            Icon { icon: LdCircleCheck, width: 16, height: 16, fill: "currentColor" }
        },
        ToastKind::Error => rsx! {
            Icon { icon: LdCircleX, width: 16, height: 16, fill: "currentColor" }
        },
        ToastKind::Warning => rsx! {
            Icon { icon: LdTriangleAlert, width: 16, height: 16, fill: "currentColor" }
        },
        ToastKind::Info => rsx! {
            Icon { icon: LdInfo, width: 16, height: 16, fill: "currentColor" }
        },
    }
}
