//! 无边框桌面窗口的系统级拖拽与窗口按钮。
//!
//! 在 **Dioxus desktop（WebView）** 下，前台窗口仍为 `tao` 主窗口，上述 API 通常继续可用。
//! `CAMetalLayer` 子树圆角逻辑主要为旧版 **native/wgpu** 呈现路径编写；在 WebView 模式下遍历多为空操作，可保留。
#![cfg(all(feature = "native", not(target_arch = "wasm32")))]

/// 与 `main.css` 中 `--ac-native-window-radius` 一致（AppKit 逻辑点，约等于 CSS px @1x）。
///
/// 只有 macOS 路径（`mod imp` 下的 `apply_host_window_rounding`）会读这个常量；
/// Windows / Linux 下整个 `imp` 模块被 `#[cfg(target_os = "macos")]` gate 掉，
/// 同步给常量加同样的 gate，避免 dead_code warning。
#[cfg(target_os = "macos")]
pub const NATIVE_HOST_CORNER_RADIUS_PT: f64 = 12.0;

/// 将闭包投递到主队列（Dioxus / wgpu 的 `onmounted` 可能不在主线程，直接调 AppKit 会静默失败）。
///
/// 使用 `dispatch2`：其通过 `_dispatch_main_q` 取主队列；手写 `dispatch_get_main_queue` 在部分链接配置下会缺符号。
#[cfg(target_os = "macos")]
#[inline]
fn dispatch_ui_to_main(job: impl FnOnce() + Send + 'static) {
    dispatch2::DispatchQueue::main().exec_async(job);
}

#[cfg(target_os = "macos")]
mod imp {
    use std::sync::atomic::{AtomicBool, Ordering};

    use objc2::runtime::{AnyClass, NSObjectProtocol};
    use objc2::MainThreadMarker;
    use objc2_app_kit::{
        NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSApplication, NSColor,
        NSWindow, NSWindowButton,
    };
    use objc2_quartz_core::{kCACornerCurveContinuous, CALayer};

    /// 与 `desktop/src/main.rs` 的 `with_traffic_light_inset` 一致。
    const TRAFFIC_LIGHT_INSET_X: f64 = 14.0;
    const TRAFFIC_LIGHT_INSET_Y: f64 = 14.0;

    /// 在自定义标题栏拖拽区域 `mousedown` 时调用（主线程）。
    /// 关闭 / 最小化 / 缩放改由系统红绿灯处理，不再自绘按钮。
    pub fn drag_window() {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let app = NSApplication::sharedApplication(mtm);
        let Some(win) = app.keyWindow().or_else(|| app.mainWindow()) else {
            return;
        };
        let Some(ev) = app.currentEvent() else {
            return;
        };
        win.performWindowDragWithEvent(&ev);
    }

    /// 遍历 `CALayer` 子树，对其中 `CAMetalLayer`（wgpu 呈现目标）同步圆角与 `masksToBounds`。
    ///
    /// 仅裁切根 layer 时，在 `with_inner_size` 触发首帧后 `Resized`、Metal 子层晚于 UI mount 等情况下，
    /// Metal 仍可能铺满直角客户区，导致「外层窗口不圆、内层 CSS 圆角正常」。
    fn apply_rounding_to_layer_tree(root: &CALayer, metal_class: &AnyClass, radius: f64) {
        // SAFETY: `layer` 来自 AppKit 视图树；`sublayers` 仅在主线程由 `apply_rounding` 访问。
        unsafe fn visit(layer: &CALayer, metal_class: &AnyClass, radius: f64) {
            if layer.isKindOfClass(metal_class) {
                layer.setCornerRadius(radius);
                layer.setMasksToBounds(true);
                layer.setOpaque(false);
                layer.setBackgroundColor(None);
                unsafe {
                    layer.setCornerCurve(kCACornerCurveContinuous);
                }
            }
            let Some(subs) = (unsafe { layer.sublayers() }) else {
                return;
            };
            let n = subs.count() as usize;
            for i in 0..n {
                let sub = subs.objectAtIndex(i as _);
                visit(&sub, metal_class, radius);
            }
        }
        unsafe {
            visit(root, metal_class, radius);
        }
    }

    /// 复刻 tao `inset_traffic_lights`，并垂直居中灯珠，避免容器高度被压矮后非激活态重排裁切。
    fn inset_traffic_lights(win: &NSWindow) {
        let Some(close) = win.standardWindowButton(NSWindowButton::CloseButton) else {
            return;
        };
        let Some(miniaturize) = win.standardWindowButton(NSWindowButton::MiniaturizeButton) else {
            return;
        };
        let Some(zoom) = win.standardWindowButton(NSWindowButton::ZoomButton) else {
            return;
        };
        // SAFETY: 标准窗口按钮始终挂在 titlebar 视图层级上。
        let Some(titlebar_view) = (unsafe { close.superview() }) else {
            return;
        };
        let Some(title_bar_container) = (unsafe { titlebar_view.superview() }) else {
            return;
        };

        let close_rect = close.frame();
        let title_bar_frame_height = close_rect.size.height + TRAFFIC_LIGHT_INSET_Y;
        let mut title_bar_rect = title_bar_container.frame();
        title_bar_rect.size.height = title_bar_frame_height;
        title_bar_rect.origin.y = win.frame().size.height - title_bar_frame_height;
        title_bar_container.setFrame(title_bar_rect);

        let space_between = miniaturize.frame().origin.x - close_rect.origin.x;
        let buttons = [close, miniaturize, zoom];
        for (i, button) in buttons.iter().enumerate() {
            // 切勿 setHidden(true) / setAlphaValue(0)：失焦时仍要看见三颗灰圆。
            button.setHidden(false);
            button.setAlphaValue(1.0);
            let mut rect = button.frame();
            rect.origin.x = TRAFFIC_LIGHT_INSET_X + (i as f64 * space_between);
            rect.origin.y = ((title_bar_frame_height - rect.size.height) * 0.5).max(0.0);
            button.setFrameOrigin(rect.origin);
        }
        refresh_traffic_light_button_appearance(win, &buttons);
        titlebar_view.setNeedsDisplay(true);
        title_bar_container.setNeedsDisplay(true);
    }

    /// 非激活态约 `#d1d5db` 的灰圆填充（相对 `#f8fafc` 标题底有足够对比，无描边依赖）。
    fn inactive_traffic_light_fill() -> objc2::rc::Retained<NSColor> {
        NSColor::colorWithSRGBRed_green_blue_alpha(0.820, 0.835, 0.859, 1.0)
    }

    /// 刷新灯珠外观：失焦时保证可见灰盘；聚焦时清掉自定义底色，交给系统彩灯。
    ///
    /// 不要用 `setShowsBorderOnlyWhileMouseInside(true)`：对 `_NSThemeWidget` 会把非激活
    /// 圆形 chrome（fill/bezel）一并压掉，标题栏只剩空白。也不要 `setBordered(false)`。
    /// 仅清 CALayer 描边；失焦时用 layer 圆角底色兜底灰盘（fill-only，无 circumference stroke）。
    fn refresh_traffic_light_button_appearance(
        win: &NSWindow,
        buttons: &[objc2::rc::Retained<objc2_app_kit::NSButton>],
    ) {
        let inactive = !win.isKeyWindow();
        let fill = inactive_traffic_light_fill();
        for button in buttons {
            // 显式恢复默认：先前去描边路径曾把它设为 true，会抹掉失焦灰圆。
            button.setShowsBorderOnlyWhileMouseInside(false);
            if inactive {
                button.setWantsLayer(true);
                if let Some(layer) = button.layer() {
                    let side = button.bounds().size.height.max(button.bounds().size.width);
                    layer.setCornerRadius((side * 0.5).max(1.0));
                    layer.setMasksToBounds(true);
                    layer.setBackgroundColor(Some(&fill.CGColor()));
                    clear_layer_stroke_tree(&layer);
                }
            } else if let Some(layer) = button.layer() {
                // 聚焦：去掉自定义灰底，保留 layer（若已有），只清描边，让系统画红/黄/绿。
                layer.setBackgroundColor(None);
                clear_layer_stroke_tree(&layer);
            }
            // NSControl 有无参 `setNeedsDisplay()` 会遮蔽 NSView 的 bool 版本。
            button.setNeedsDisplayInRect(button.bounds());
        }
    }

    fn clear_layer_stroke_tree(layer: &CALayer) {
        layer.setBorderWidth(0.0);
        layer.setBorderColor(None);
        layer.setShadowOpacity(0.0);
        let Some(subs) = (unsafe { layer.sublayers() }) else {
            return;
        };
        let n = subs.count() as usize;
        for i in 0..n {
            clear_layer_stroke_tree(&subs.objectAtIndex(i as _));
        }
    }

    /// 失焦/聚焦后延迟再刷一次：AppKit 会在状态切换时重绘 theme widget，可能重新加上描边
    /// 或清掉我们刚设的灰底；短延迟后按当前 key 状态对齐。
    fn schedule_traffic_light_appearance_refresh() {
        for delay_ms in [16u64, 80, 200] {
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                super::dispatch_ui_to_main(|| {
                    let Some(mtm) = MainThreadMarker::new() else {
                        return;
                    };
                    let app = NSApplication::sharedApplication(mtm);
                    let Some(win) = app.mainWindow().or_else(|| app.keyWindow()) else {
                        return;
                    };
                    inset_traffic_lights(&win);
                });
            });
        }
    }

    /// 轮询 key 状态（避免再拉 NSNotification/block2 依赖）；仅在聚焦翻转时刷新灯珠。
    fn ensure_traffic_light_focus_watch() {
        static STARTED: AtomicBool = AtomicBool::new(false);
        static LAST_KEY: AtomicBool = AtomicBool::new(false);
        static HAS_SAMPLE: AtomicBool = AtomicBool::new(false);
        if STARTED.swap(true, Ordering::SeqCst) {
            return;
        }
        std::thread::spawn(|| {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(120));
                super::dispatch_ui_to_main(|| {
                    let Some(mtm) = MainThreadMarker::new() else {
                        return;
                    };
                    let app = NSApplication::sharedApplication(mtm);
                    let Some(win) = app.mainWindow().or_else(|| app.keyWindow()) else {
                        return;
                    };
                    let is_key = win.isKeyWindow();
                    let changed = !HAS_SAMPLE.load(Ordering::SeqCst)
                        || LAST_KEY.load(Ordering::SeqCst) != is_key;
                    if !changed {
                        return;
                    }
                    HAS_SAMPLE.store(true, Ordering::SeqCst);
                    LAST_KEY.store(is_key, Ordering::SeqCst);
                    inset_traffic_lights(&win);
                    schedule_traffic_light_appearance_refresh();
                });
            }
        });
    }

    /// 强制浅色 Aqua appearance，窗口底必须透明，再刷新标准按钮。
    ///
    /// 根因（透明标题栏常见问题）：
    /// 1. 若 `NSWindow.backgroundColor` 用不透明浅色铺满直角窗，而 contentView / CSS 已圆角裁切，
    ///    四角楔形会露出矩形底 + 系统阴影按直角路径描边 →「圆角 UI 外还有一圈直角框」。
    /// 2. 窗口底改 `clear` 后，浅底改画在 **已圆角** 的 contentView layer 上（见 `apply_rounding`），
    ///    阴影跟圆角不透明区域走。
    /// 3. **VibrantLight + `setShowsBorderOnlyWhileMouseInside(true)`** 会把失焦灰圆冲淡/压掉
    ///    （vibrancy 在 clear 窗底上几乎无衬底；bezel-only-on-hover 抹掉非激活圆形 chrome）。
    ///    改回 **Aqua**，失焦用 layer 灰盘兜底，聚焦交给系统彩灯；只清 CALayer 描边。
    /// 4. 切勿再给 `NSWindow` 设不透明直角底。
    fn ensure_traffic_light_chrome(win: &NSWindow) {
        // SAFETY: `NSAppearanceNameAqua` 为 AppKit 导出的常量字符串。
        if let Some(aqua) = unsafe { NSAppearance::appearanceNamed(NSAppearanceNameAqua) } {
            win.setAppearance(Some(&aqua));
        }
        win.setOpaque(false);
        win.setBackgroundColor(Some(&NSColor::clearColor()));
        inset_traffic_lights(win);
        ensure_traffic_light_focus_watch();
    }

    /// 用 `NSView` 根 layer 圆角裁切整窗内容（wgpu 的 `CAMetalLayer` 挂在根 layer 子层上，需 `masksToBounds`）。
    ///
    /// 必须在主线程调用；请通过 [`super::apply_host_window_rounding`] 的 `dispatch_async` 路径进入。
    pub(super) fn apply_rounding() {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let app = NSApplication::sharedApplication(mtm);
        let Some(win) = app.mainWindow().or_else(|| app.keyWindow()) else {
            return;
        };
        let Some(cv) = win.contentView() else {
            return;
        };
        // 红绿灯 chrome（Aqua + 透明窗底 + inset + 失焦灰盘）须在圆角 mask 之前就绪。
        ensure_traffic_light_chrome(&win);
        cv.setWantsLayer(true);
        cv.layoutSubtreeIfNeeded();
        let Some(layer) = cv.layer() else {
            return;
        };
        let radius = super::NATIVE_HOST_CORNER_RADIUS_PT;
        // `#f8fafc` == `--primary-color-1`：浅底画在圆角 layer 上（勿设到 NSWindow）。
        let chrome = NSColor::colorWithSRGBRed_green_blue_alpha(0.973, 0.980, 0.988, 1.0);
        layer.setOpaque(false);
        layer.setBackgroundColor(Some(&chrome.CGColor()));
        layer.setCornerRadius(radius);
        layer.setMasksToBounds(true);
        // 去掉 contentView 描边，避免圆角外缘出现一圈深色边线（勿递归清 shadow，以免波及子层）。
        layer.setBorderWidth(0.0);
        layer.setBorderColor(None);
        // `extern "C"` 静态符号在 Rust 2024 中需显式 `unsafe`。
        unsafe {
            layer.setCornerCurve(kCACornerCurveContinuous);
        }
        if let Some(metal_cls) = AnyClass::get(c"CAMetalLayer") {
            apply_rounding_to_layer_tree(&layer, metal_cls, radius);
        }
        // WebView：父层 `masksToBounds` 已裁切；对一级子层再套同半径，避免 WK 默认直角底闪一下。
        if let Some(subs) = unsafe { layer.sublayers() } {
            let n = subs.count() as usize;
            for i in 0..n {
                let sub = subs.objectAtIndex(i as _);
                sub.setCornerRadius(radius);
                sub.setMasksToBounds(true);
                sub.setBorderWidth(0.0);
                sub.setBorderColor(None);
                unsafe {
                    sub.setCornerCurve(kCACornerCurveContinuous);
                }
            }
        }
        win.invalidateShadow();
    }
}

#[cfg(target_os = "windows")]
mod imp {
    // windows-sys 0.52 把 `HWND` 定义为 `isize`（句柄而非裸指针），所以没有 `.is_null()`，
    // 空句柄判断改为与 `0` 比较；同样 `DWMWA_*` 常量是 `i32`，而 `DwmSetWindowAttribute`
    // 的 attribute 参数是 `u32`，必须显式 `as u32` 否则 E0308。
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };
    // ReleaseCapture 在 windows-sys 0.52 中位于 KeyboardAndMouse 子模块，需要对应 feature。
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, IsZoomed, PostMessageW, SendMessageW, ShowWindow, HTCAPTION,
        SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, WM_CLOSE, WM_NCLBUTTONDOWN,
    };

    fn hwnd() -> HWND {
        unsafe { GetForegroundWindow() }
    }

    pub fn drag_window() {
        unsafe {
            let hwnd = hwnd();
            if hwnd == 0 {
                return;
            }
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as usize, 0);
        }
    }

    pub fn minimize_window() {
        unsafe {
            let hwnd = hwnd();
            if hwnd == 0 {
                return;
            }
            ShowWindow(hwnd, SW_MINIMIZE);
        }
    }

    pub fn toggle_maximize() {
        unsafe {
            let hwnd = hwnd();
            if hwnd == 0 {
                return;
            }
            if IsZoomed(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            } else {
                ShowWindow(hwnd, SW_MAXIMIZE);
            }
        }
    }

    pub fn close_window() {
        unsafe {
            let hwnd = hwnd();
            if hwnd == 0 {
                return;
            }
            let _ = PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
    }

    pub(super) fn apply_rounding() {
        let hwnd = hwnd();
        if hwnd == 0 {
            return;
        }
        let pref = DWMWCP_ROUND;
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                std::ptr::addr_of!(pref).cast(),
                std::mem::size_of_val(&pref) as u32,
            );
        }
    }
}

#[cfg(target_os = "linux")]
mod imp {
    pub fn drag_window() {}

    pub fn minimize_window() {}

    pub fn toggle_maximize() {}

    pub fn close_window() {
        std::process::exit(0);
    }

    pub(super) fn apply_rounding() {}
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
mod imp {
    pub fn drag_window() {}

    pub fn minimize_window() {}

    pub fn toggle_maximize() {}

    pub fn close_window() {
        std::process::exit(0);
    }

    pub(super) fn apply_rounding() {}
}

#[cfg(target_os = "macos")]
pub use imp::drag_window;

#[cfg(not(target_os = "macos"))]
pub use imp::{close_window, drag_window, minimize_window, toggle_maximize};

/// 对当前前台窗口应用系统级圆角（macOS：`contentView` 根 layer + 主队列；Windows：DWM）。
pub fn apply_host_window_rounding() {
    #[cfg(target_os = "macos")]
    dispatch_ui_to_main(|| imp::apply_rounding());
    #[cfg(not(target_os = "macos"))]
    imp::apply_rounding();
}

/// 首帧前后句柄 / layer 可能尚未就绪：在后台等待后再次应用圆角。
pub fn schedule_host_window_rounding_retry() {
    #[cfg(target_os = "macos")]
    {
        // `with_inner_size` 常在首帧后再触发 `Resized`，Metal 子层就绪更晚，多档延迟覆盖该窗口期。
        for delay_ms in [80u64, 280, 550, 1200] {
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                dispatch_ui_to_main(|| imp::apply_rounding());
            });
        }
    }
    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(120));
            imp::apply_rounding();
        });
    }
}
