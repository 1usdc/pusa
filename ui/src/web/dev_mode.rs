//! Web：开发者模式开关（`localStorage` + `<body class="ac-developer-mode">` 供样式与后续调试 UI 使用）。

#![cfg(all(target_arch = "wasm32", feature = "web"))]

const STORAGE_KEY: &str = "anotherclaw_developer_mode";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

/// 是否启用开发者模式。
pub fn get() -> bool {
    storage()
        .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
        .as_deref()
        == Some("1")
}

/// 仅根据布尔值更新 `<body class="ac-developer-mode">`，不写存储（用于与 UI 状态对齐）。
pub fn sync_body_to(enabled: bool) {
    sync_body_class(enabled);
}

fn sync_body_class(enabled: bool) {
    let Some(w) = web_sys::window() else {
        return;
    };
    let Some(doc) = w.document() else {
        return;
    };
    let Some(body) = doc.body() else {
        return;
    };
    let class_list = body.class_list();
    if enabled {
        let _ = class_list.add_1("ac-developer-mode");
    } else {
        let _ = class_list.remove_1("ac-developer-mode");
    }
}

/// 写入存储并同步 `<body>` 上的标记类名。
pub fn set(enabled: bool) {
    if let Some(st) = storage() {
        if enabled {
            let _ = st.set_item(STORAGE_KEY, "1");
        } else {
            let _ = st.remove_item(STORAGE_KEY);
        }
    }
    sync_body_class(enabled);
}
