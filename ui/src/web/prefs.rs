//! Web：UI 偏好（聊天模型、中间栏标签等）持久化到 `localStorage`。
//!
//! 与 [`crate::web::auth`] 的差异：这里存的是"产品偏好"（与账号无关、刷新后想保留
//! 的客户端状态），不影响登录态。退出登录时不清空——下次同一浏览器再登录希望
//! 模型选择仍然记得。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

/// 聊天栏当前选中的模型 ID（如 `gpt-5.4`、`claude-haiku-4-5`）。
const CHAT_MODEL_KEY: &str = "anotherclaw_chat_model";
const CHAT_AGENT_MODE_KEY: &str = "anotherclaw_chat_agent_mode";
/// 中间栏已打开标签页缓存（JSON：`{ tabs, active_id }`）。
const CENTER_TABS_KEY: &str = "anotherclaw_center_tabs";
/// 侧栏文件树展开/选中状态（JSON：`{ root, expanded, selected_path }`）。
const FILE_TREE_KEY: &str = "anotherclaw_file_tree";

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

/// 读取上次保存的聊天栏 Agent 模式（`agent` / `ask` / `plan` / `multitask`）。
pub fn chat_agent_mode_get() -> Option<String> {
    let raw = storage()?.get_item(CHAT_AGENT_MODE_KEY).ok().flatten()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 写入聊天栏 Agent 模式；空字符串等价于清除。
pub fn chat_agent_mode_set(mode: &str) {
    let Some(st) = storage() else {
        return;
    };
    let trimmed = mode.trim();
    if trimmed.is_empty() {
        let _ = st.remove_item(CHAT_AGENT_MODE_KEY);
    } else {
        let _ = st.set_item(CHAT_AGENT_MODE_KEY, trimmed);
    }
}

/// 用户自定义聊天模型（JSON：对象数组，兼容旧字符串数组）。
const CUSTOM_CHAT_MODELS_KEY: &str = "anotherclaw_custom_chat_models";

/// 读取自定义模型；无缓存或解析失败返回空 Vec。
pub fn custom_chat_models_get() -> Vec<crate::custom_chat_models::CustomChatModelPref> {
    let Some(st) = storage() else {
        return Vec::new();
    };
    let Some(raw) = st.get_item(CUSTOM_CHAT_MODELS_KEY).ok().flatten() else {
        return Vec::new();
    };
    crate::custom_chat_models::parse_custom_chat_models_json(&raw)
}

/// 写入自定义模型列表。
pub fn custom_chat_models_set(items: &[crate::custom_chat_models::CustomChatModelPref]) {
    let Some(st) = storage() else {
        return;
    };
    match crate::custom_chat_models::serialize_custom_chat_models_json(items) {
        None => {
            let _ = st.remove_item(CUSTOM_CHAT_MODELS_KEY);
        }
        Some(json) => {
            let _ = st.set_item(CUSTOM_CHAT_MODELS_KEY, &json);
        }
    }
}

const LLM_CREDENTIALS_ENABLED_KEY: &str = "anotherclaw_llm_credentials_enabled";

/// 已开启的 LLM 密钥 id；从未保存返回 `None`。
pub fn llm_credentials_enabled_get() -> Option<Vec<String>> {
    let raw = storage()?.get_item(LLM_CREDENTIALS_ENABLED_KEY).ok().flatten()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str::<Vec<String>>(trimmed).ok().map(|ids| {
        ids.into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

/// 写入已开启的 LLM 密钥 id 列表。
pub fn llm_credentials_enabled_set(ids: &[String]) {
    let Some(st) = storage() else {
        return;
    };
    let cleaned: Vec<String> = ids
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if let Ok(json) = serde_json::to_string(&cleaned) {
        let _ = st.set_item(LLM_CREDENTIALS_ENABLED_KEY, &json);
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

/// 读取侧栏文件树状态 JSON；无缓存或空白返回 `None`。
pub fn file_tree_get() -> Option<String> {
    let raw = storage()?.get_item(FILE_TREE_KEY).ok().flatten()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 写入侧栏文件树状态；空字符串清除。
pub fn file_tree_set(raw_json: &str) {
    let Some(st) = storage() else {
        return;
    };
    let trimmed = raw_json.trim();
    if trimmed.is_empty() {
        let _ = st.remove_item(FILE_TREE_KEY);
    } else {
        let _ = st.set_item(FILE_TREE_KEY, trimmed);
    }
}

/// UI 主题：`light` / `dark`。
const UI_THEME_KEY: &str = "anotherclaw_ui_theme";

/// 读取 UI 主题；未设置返回 `None`。
pub fn ui_theme_get() -> Option<String> {
    let raw = storage()?.get_item(UI_THEME_KEY).ok().flatten()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 写入 UI 主题。
pub fn ui_theme_set(theme: &str) {
    let Some(st) = storage() else {
        return;
    };
    let trimmed = theme.trim();
    if trimmed.is_empty() {
        let _ = st.remove_item(UI_THEME_KEY);
    } else {
        let _ = st.set_item(UI_THEME_KEY, trimmed);
    }
}
