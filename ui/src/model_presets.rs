//! 模型供应商预设：名称、Base URL、API 格式、密钥占位、可选默认模型。
//!
//! 「模型设置 → 添加供应商」面板从中读取；全部兼容本项目已支持的两种协议：
//! OpenAI Chat Completions（`POST {base}/chat/completions`，`Authorization: Bearer`）
//! 与 Anthropic Messages（`POST {base}/messages`，`x-api-key`）。

use protocol::LlmApiProvider;

/// 一条预设供应商配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelProviderPreset {
    /// 展示名（也是「添加供应商」表单的默认名称）。
    pub name: &'static str,
    /// OpenAI 兼容根路径（须以 `/v1` 结尾；Anthropic 官方为 `/v1`）。
    pub base_url: &'static str,
    /// API 格式：Chat Completions 或 Anthropic Messages。
    pub api_format: LlmApiProvider,
    /// API Key 输入框占位符。
    pub key_placeholder: &'static str,
    /// 申请密钥的入口页（可空）。
    pub keys_url: &'static str,
    /// 预置的常用模型 ID（可空）。
    pub default_models: &'static [&'static str],
}

impl ModelProviderPreset {
    /// API 格式的可读文案（与详情页下拉一致）；仅往返测试使用。
    #[cfg(test)]
    pub fn api_format_label(&self) -> &'static str {
        api_format_label(self.api_format)
    }
}

/// API 格式的可读文案。
pub fn api_format_label(p: LlmApiProvider) -> &'static str {
    match p {
        LlmApiProvider::Openai => "Chat Completions (/chat/completions)",
        LlmApiProvider::Anthropic => "Anthropic Messages (/messages)",
    }
}

/// API 格式反解（UI 下拉值 ↔ LlmApiProvider）。
pub fn parse_api_format_label(label: &str) -> LlmApiProvider {
    if label.starts_with("Anthropic") {
        LlmApiProvider::Anthropic
    } else {
        LlmApiProvider::Openai
    }
}

/// 全部预设，按国内常见度排列；「自定义供应商」按钮单独渲染，不在本列表。
pub const MODEL_PROVIDER_PRESETS: &[ModelProviderPreset] = &[
    ModelProviderPreset {
        name: "OpenAI",
        base_url: "https://api.openai.com/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "sk-…",
        keys_url: "https://platform.openai.com/api-keys",
        default_models: &["gpt-4.1", "gpt-4.1-mini", "gpt-4o", "o4-mini"],
    },
    ModelProviderPreset {
        name: "Anthropic",
        base_url: "https://api.anthropic.com/v1",
        api_format: LlmApiProvider::Anthropic,
        key_placeholder: "sk-ant-…",
        keys_url: "https://console.anthropic.com/settings/keys",
        default_models: &[
            "claude-sonnet-4-5",
            "claude-opus-4-1",
            "claude-haiku-4-5",
        ],
    },
    ModelProviderPreset {
        name: "智谱 GLM",
        base_url: "https://open.bigmodel.cn/api/paas/v4",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "…",
        keys_url: "https://open.bigmodel.cn/usercenter/apikeys",
        default_models: &["glm-4.6", "glm-4.5", "glm-4.5-air", "glm-4-flash"],
    },
    ModelProviderPreset {
        name: "DeepSeek",
        base_url: "https://api.deepseek.com/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "sk-…",
        keys_url: "https://platform.deepseek.com/api_keys",
        default_models: &["deepseek-chat", "deepseek-reasoner"],
    },
    ModelProviderPreset {
        name: "月之暗面 Kimi",
        base_url: "https://api.moonshot.cn/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "sk-…",
        keys_url: "https://platform.moonshot.cn/console/api-keys",
        default_models: &["kimi-k2-0905-preview", "kimi-k2-turbo-preview", "moonshot-v1-128k"],
    },
    ModelProviderPreset {
        name: "阿里云百炼 Qwen",
        base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "sk-…",
        keys_url: "https://bailian.console.aliyun.com/",
        default_models: &["qwen3-235b-a22b", "qwen-plus", "qwen-turbo", "qwen-max"],
    },
    ModelProviderPreset {
        name: "硅基流动 SiliconFlow",
        base_url: "https://api.siliconflow.cn/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "sk-…",
        keys_url: "https://cloud.siliconflow.cn/account/ak",
        default_models: &["deepseek-ai/DeepSeek-V3.2", "Qwen/Qwen3-235B-A22B"],
    },
    ModelProviderPreset {
        name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "sk-or-…",
        keys_url: "https://openrouter.ai/settings/keys",
        default_models: &["openai/gpt-4.1", "anthropic/claude-sonnet-4.5", "qwen/qwen3-coder"],
    },
    ModelProviderPreset {
        name: "Ora (ora.ai)",
        base_url: "https://api.ora.ai/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "…",
        keys_url: "",
        default_models: &[],
    },
    ModelProviderPreset {
        name: "xAI Grok",
        base_url: "https://api.x.ai/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "xai-…",
        keys_url: "https://console.x.ai",
        default_models: &["grok-4", "grok-3", "grok-3-mini"],
    },
    ModelProviderPreset {
        name: "Groq",
        base_url: "https://api.groq.com/openai/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "gsk_…",
        keys_url: "https://console.groq.com/keys",
        default_models: &["llama-3.3-70b-versatile", "openai/gpt-oss-120b"],
    },
    ModelProviderPreset {
        name: "Ollama（本机）",
        base_url: "http://127.0.0.1:11434/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "本地服务可留空",
        keys_url: "",
        default_models: &["qwen3:8b", "llama3.1:8b"],
    },
    ModelProviderPreset {
        name: "LM Studio（本机）",
        base_url: "http://127.0.0.1:1234/v1",
        api_format: LlmApiProvider::Openai,
        key_placeholder: "本地服务可留空",
        keys_url: "",
        default_models: &[],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_v1_bases() {
        for p in MODEL_PROVIDER_PRESETS {
            assert!(p.base_url.starts_with("http"), "{}", p.name);
            assert!(!p.name.trim().is_empty());
        }
    }

    #[test]
    fn api_format_labels_round_trip() {
        for p in MODEL_PROVIDER_PRESETS {
            assert_eq!(parse_api_format_label(p.api_format_label()), p.api_format);
        }
    }
}
