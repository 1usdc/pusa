//! 开发者模式：关于弹窗标题三连击开关；聊天栏复制旁显示详情（眼睛）入口。
//! Web 持久化到 localStorage；桌面持久化到用户数据目录。

use dioxus::document;
use dioxus::prelude::*;

/// 连续三次点击判定窗口（毫秒）。
const TRIPLE_CLICK_WINDOW_MS: f64 = 700.0;

#[cfg(all(target_arch = "wasm32", feature = "web"))]
const STORAGE_KEY: &str = "anotherclaw_developer_mode";

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn developer_mode_path() -> std::path::PathBuf {
    crate::desktop::files::app_data_dir().join("developer_mode.txt")
}

/// 注入到组件树的开发者模式开关。
#[derive(Clone, Copy)]
pub struct DeveloperMode(pub Signal<bool>);

/// 读取持久化开关。
pub fn load() -> bool {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        return web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
            .as_deref()
            == Some("1");
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        return std::fs::read_to_string(developer_mode_path())
            .ok()
            .map(|s| s.trim() == "1")
            .unwrap_or(false);
    }
    #[cfg(not(any(
        all(target_arch = "wasm32", feature = "web"),
        all(feature = "native", not(target_arch = "wasm32"))
    )))]
    {
        false
    }
}

/// 写入持久化并同步 `<body class="ac-developer-mode">`（Web / 桌面 WebView）。
pub fn set(enabled: bool) {
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        if let Some(st) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            if enabled {
                let _ = st.set_item(STORAGE_KEY, "1");
            } else {
                let _ = st.remove_item(STORAGE_KEY);
            }
        }
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        let path = developer_mode_path();
        if enabled {
            let _ = std::fs::write(&path, "1\n");
        } else {
            let _ = std::fs::remove_file(&path);
        }
    }
    sync_body_to(enabled);
}

/// 仅同步 body class，与 UI 信号对齐。
pub fn sync_body_to(enabled: bool) {
    let on = if enabled { "1" } else { "0" };
    let _ = document::eval(&format!(
        r#"(() => {{
  const b = document.body;
  if (!b) return;
  if ({on} === 1) b.classList.add('ac-developer-mode');
  else b.classList.remove('ac-developer-mode');
}})()"#,
        on = on
    ));
}

/// 在窗口期内累计点击，满 3 次执行 `action` 并清零。
pub fn register_triple_click(mut tally: Signal<(u32, f64)>, action: impl FnOnce()) {
    let t = now_ms();
    let (mut n, last) = tally();
    n = if t - last <= TRIPLE_CLICK_WINDOW_MS {
        n.saturating_add(1)
    } else {
        1
    };
    tally.set((n, t));
    if n >= 3 {
        tally.set((0, 0.0));
        action();
    }
}

fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        return js_sys::Date::now();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }
}

/// 切换开发者模式（信号 + 持久化）。
pub fn toggle(mut developer_mode: Signal<bool>) {
    let next = !developer_mode();
    developer_mode.set(next);
    set(next);
}

/// 显式开/关。
pub fn apply(enabled: bool, mut developer_mode: Signal<bool>) {
    developer_mode.set(enabled);
    set(enabled);
}
