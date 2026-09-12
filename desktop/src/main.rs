//! 桌面（系统 WebView / wry）入口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use dioxus::desktop::tao::dpi::LogicalSize;
use dioxus::desktop::wry::http::Response;
use dioxus::desktop::tao::window::Icon;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use ui::App;

/// 本机窗口默认客户区尺寸（逻辑像素）。
const DEFAULT_WINDOW_WIDTH: u32 = 1280;
const DEFAULT_WINDOW_HEIGHT: u32 = 720;

/// 冷启动骨架 index（内联 CSS，窗口出现即可绘出，早于 Dioxus 首帧）。
const BOOT_INDEX_HTML: &str = include_str!("../assets/boot-index.html");

/// WebView 底色：对齐 `--primary-color-1` / `#f8fafc`，避免 HTML 就绪前白闪。
const WEBVIEW_BG: (u8, u8, u8, u8) = (248, 250, 252, 255);

/// 窗口 / Dock 使用处理后的 master：`app-icon.png`
/// （squircle 圆角已烘焙；内容约 82% 画布 / 每侧 ~9% 透明边距，贴近 Dock 邻图标视觉体量）。
fn pusa_icon_png() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/app-icon.png")
}

/// release / 打包后没有控制台：把 panic 写到 `%LOCALAPPDATA%\AnotherClaw\crash.log`。
fn install_release_panic_log() {
    if cfg!(debug_assertions) {
        return;
    }
    std::panic::set_hook(Box::new(|info| {
        let mut dir = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        dir.push("AnotherClaw");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("crash.log"), format!("{info}\n"));
    }));
}

/// 嵌入带透明边距的 `app-icon.png`（非满铺 `Pusa.png`）；开发态 Dock 由此加载。
#[cfg(target_os = "macos")]
const PUSA_DOCK_ICON_PNG: &[u8] = include_bytes!("../assets/app-icon.png");

// Dioxus 0.7 的 `LaunchBuilder::new` 在部分配置下仍会误报弃用。
#[allow(deprecated)]
fn main() {
    // 必须最先执行：Velopack 在安装 / 卸载 / 更新后首启等钩子场景下可能直接退出或重启进程。
    // 非 Velopack 安装（`dx serve`、旧 DMG）时这里是空操作。
    velopack::VelopackApp::build().run();

    install_release_panic_log();
    let icon_path = pusa_icon_png();

    dioxus::LaunchBuilder::new()
        .with_cfg(desktop! {
            // macOS：切勿在 launch 前调用 sharedApplication——会抢先创建默认
            // NSApplication，破坏 tao 的自定义 APP_CLASS，且 Dock 图标会被后续
            // activation 覆盖。必须在窗口建成后（NSApp 已就绪）再 set。
            Config::new()
                .with_window(build_window(&icon_path))
                .with_background_color(WEBVIEW_BG)
                .with_custom_index(BOOT_INDEX_HTML.to_string())
                .with_custom_protocol(ui::HTML_PREVIEW_PROTOCOL, |_id, request| {
                    html_preview_protocol_response(&request.uri().to_string())
                })
                .with_on_window(|_window, _dom| {
                    #[cfg(target_os = "macos")]
                    set_macos_dock_icon();
                })
        })
        .launch(App);
}

fn html_preview_protocol_response(uri: &str) -> Response<Cow<'static, [u8]>> {
    match ui::serve_html_preview(uri) {
        Ok((mime, body)) => Response::builder()
            .status(200)
            .header("Content-Type", mime)
            .header("Cache-Control", "no-cache")
            .header("Access-Control-Allow-Origin", "*")
            .body(Cow::from(body))
            .unwrap_or_else(|_| Response::new(Cow::from(Vec::<u8>::new()))),
        Err(status) => Response::builder()
            .status(status)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Cow::from(b"Not Found".to_vec()))
            .unwrap_or_else(|_| Response::new(Cow::from(Vec::<u8>::new()))),
    }
}

fn load_window_icon(path: &Path) -> Option<Icon> {
    let image = image::open(path).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).ok()
}

/// 开发态（裸二进制）设置 Dock 图标。打包后的 `.app` 仍走 `Info.plist` / `.icns`。
///
/// 必须在 tao/dioxus 创建窗口之后调用：此时自定义 `NSApplication` 子类与
/// `Regular` activation policy 已就绪，`setApplicationIconImage` 才会生效。
#[cfg(target_os = "macos")]
fn set_macos_dock_icon() {
    use objc2::rc::Retained;
    use objc2::{AnyThread, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{
        NSApplication, NSApplicationActivationPolicy, NSImage, NSImageScaling, NSImageView,
    };
    use objc2_foundation::{NSData, NSPoint, NSRect};

    let Some(mtm) = MainThreadMarker::new() else {
        eprintln!("warning: Dock icon skipped — not on main thread");
        return;
    };

    let data = NSData::with_bytes(PUSA_DOCK_ICON_PNG);
    let Some(image): Option<Retained<NSImage>> =
        NSImage::initWithData(NSImage::alloc(), &data)
    else {
        eprintln!("warning: failed to decode embedded Dock icon (app-icon.png)");
        return;
    };
    // 彩色图标，禁止当 template（否则会丢 alpha / 被染色成方块剪影）。
    image.setTemplate(false);

    let app = NSApplication::sharedApplication(mtm);
    // 裸 `cargo run` 进程默认可能不是 Regular；确保 Dock 显示图标位。
    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    // SAFETY: `image` 为有效 NSImage；窗口已存在，AppKit 已完成启动。
    unsafe {
        app.setApplicationIconImage(Some(&image));
    }

    // 裸二进制下 Dock 有时不按 PNG alpha 裁切；再挂一层带 alpha 的 NSImageView。
    let tile = app.dockTile();
    let tile_size = tile.size();
    let view =
        NSImageView::initWithFrame(NSImageView::alloc(mtm), NSRect::new(NSPoint::ZERO, tile_size));
    view.setImage(Some(&image));
    view.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
    tile.setContentView(Some(&view));
    tile.display();
}

/// 按平台配置 `tao::WindowBuilder`（与 dioxus-desktop 使用的窗口后端一致）。
fn build_window(icon_path: &Path) -> WindowBuilder {
    // tao::Icon 走 image crate 解码 RGBA，必须 PNG / JPG 等 raster 格式（.ico 不行）。
    let mut wb = WindowBuilder::new()
        .with_title("Pusa".to_string())
        .with_inner_size(LogicalSize::new(
            DEFAULT_WINDOW_WIDTH as f64,
            DEFAULT_WINDOW_HEIGHT as f64,
        ))
        .with_resizable(true)
        .with_window_icon(load_window_icon(icon_path));

    #[cfg(target_os = "linux")]
    {
        wb = wb.with_decorations(true);
    }

    // macOS：直接走 tao 的 WindowBuilderExtMacOS（非 Dioxus 业务封装）。
    // 透明标题栏 + fullsize content，保留系统红绿灯。
    // traffic_light_inset.y 把标题栏容器高度设为「灯珠高 + y」；y=14 → 约 28pt，与自定义栏同高。
    #[cfg(all(not(target_os = "linux"), target_os = "macos"))]
    {
        use dioxus::desktop::tao::dpi::LogicalPosition;
        use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;
        wb = wb
            .with_transparent(true)
            .with_decorations(true)
            .with_titlebar_transparent(true)
            .with_title_hidden(true)
            .with_titlebar_buttons_hidden(false)
            .with_fullsize_content_view(true)
            .with_has_shadow(true)
            .with_traffic_light_inset(LogicalPosition::new(14.0, 14.0));
    }

    #[cfg(all(not(target_os = "linux"), target_os = "windows"))]
    {
        use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
        wb = wb
            .with_transparent(true)
            .with_decorations(false)
            .with_undecorated_shadow(true);
    }

    #[cfg(all(
        not(target_os = "linux"),
        not(target_os = "macos"),
        not(target_os = "windows")
    ))]
    {
        wb = wb.with_transparent(true).with_decorations(false);
    }

    wb
}
