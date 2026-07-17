//! Web：UI 偏好（聊天模型、中间栏标签等）持久化到 `localStorage`。
//!
//! 与 [`crate::web::auth`] 的差异：这里存的是"产品偏好"（与账号无关、刷新后想保留
//! 的客户端状态），不影响登录态。退出登录时不清空——下次同一浏览器再登录希望
//! 模型选择仍然记得。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

/// 聊天栏当前选中的模型 ID（如 `gpt-5.4`、`claude-haiku-4-5`）。
const CHAT_MODEL_KEY: &str = "anotherclaw_chat_model";
/// 中间栏已打开标签页缓存（JSON：`{ tabs, active_id }`）。
const CENTER_TABS_KEY: &str = "anotherclaw_center_tabs";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

/// 读取上次保存的聊天模型 ID；未设置或为空白返回 `None`。
pub fn chat_model_get() -> Option<String> {
    let raw = storage()?.get_item(CHAT_MODEL_KEY).ok().flatten()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 写入聊天模型 ID；空字符串等价于清除。
pub fn chat_model_set(model_id: &str) {
    let Some(st) = storage() else {
        return;
    };
    let trimmed = model_id.trim();
    if trimmed.is_empty() {
        let _ = st.remove_item(CHAT_MODEL_KEY);
    } else {
        let _ = st.set_item(CHAT_MODEL_KEY, trimmed);
    }
}

/// 读取中间栏标签页操作缓存 JSON；无缓存或空白返回 `None`。
pub fn center_tabs_get() -> Option<String> {
    let raw = storage()?.get_item(CENTER_TABS_KEY).ok().flatten()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 写入中间栏标签页缓存；空字符串清除。
pub fn center_tabs_set(raw_json: &str) {
    let Some(st) = storage() else {
        return;
    };
    let trimmed = raw_json.trim();
    if trimmed.is_empty() {
        let _ = st.remove_item(CENTER_TABS_KEY);
    } else {
        let _ = st.set_item(CENTER_TABS_KEY, trimmed);
    }
}
