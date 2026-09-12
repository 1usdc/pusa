//! 自定义聊天模型本地偏好：模型 ID、展示名、可选已保存密钥 id。
//!
//! 兼容旧格式 `["model-a","model-b"]`（无名称、无密钥绑定）。

use serde::{Deserialize, Serialize};

/// 一条自定义模型记录。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomChatModelPref {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub credential_id: String,
}

pub fn parse_custom_chat_models_json(raw: &str) -> Vec<CustomChatModelPref> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if let Ok(recs) = serde_json::from_str::<Vec<CustomChatModelPref>>(trimmed) {
        return recs
            .into_iter()
            .map(|mut r| {
                r.id = r.id.trim().to_string();
                r.name = r.name.trim().to_string();
                r.credential_id = r.credential_id.trim().to_string();
                r
            })
            .filter(|r| !r.id.is_empty())
            .collect();
    }
    serde_json::from_str::<Vec<String>>(trimmed)
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|id| CustomChatModelPref {
            id,
            name: String::new(),
            credential_id: String::new(),
        })
        .collect()
}

pub fn serialize_custom_chat_models_json(items: &[CustomChatModelPref]) -> Option<String> {
    let cleaned: Vec<CustomChatModelPref> = items
        .iter()
        .map(|r| CustomChatModelPref {
            id: r.id.trim().to_string(),
            name: r.name.trim().to_string(),
            credential_id: r.credential_id.trim().to_string(),
        })
        .filter(|r| !r.id.is_empty())
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    serde_json::to_string_pretty(&cleaned).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_legacy_string_array() {
        let got = parse_custom_chat_models_json(r#"[" foo ", "bar"]"#);
        assert_eq!(
            got,
            vec![
                CustomChatModelPref {
                    id: "foo".into(),
                    name: String::new(),
                    credential_id: String::new()
                },
                CustomChatModelPref {
                    id: "bar".into(),
                    name: String::new(),
                    credential_id: String::new()
                },
            ]
        );
    }

    #[test]
    fn parses_object_array_with_name_and_credential() {
        let got = parse_custom_chat_models_json(
            r#"[{"id":"local-qwen","name":"Qwen","credential_id":"cred-1"}]"#,
        );
        assert_eq!(got[0].id, "local-qwen");
        assert_eq!(got[0].name, "Qwen");
        assert_eq!(got[0].credential_id, "cred-1");
    }

    #[test]
    fn parses_object_array_without_name() {
        let got = parse_custom_chat_models_json(
            r#"[{"id":"local-qwen","credential_id":"cred-1"}]"#,
        );
        assert_eq!(got[0].id, "local-qwen");
        assert_eq!(got[0].name, "");
        assert_eq!(got[0].credential_id, "cred-1");
    }
}
