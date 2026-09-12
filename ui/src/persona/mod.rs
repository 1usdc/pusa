//! Persona 读写：Web → REST；Desktop → [`crate::desktop::agent`]。

#[cfg(target_arch = "wasm32")]
use protocol::PersonaBody;

/// 与 SQLite 默认插入一致，供 UI 兜底。
pub const DEFAULT_SYSTEM_PROMPT: &str = "你是一名「主动执行型」加密交易助手，是用户的执行手而不是只动嘴的顾问。";

/// 从服务端或本地 SQLite 拉取 system prompt。
pub async fn load_persona() -> anyhow::Result<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!(
            "{}/v1/persona",
            crate::chat::api_base_url().trim_end_matches('/')
        );
        let mut req = gloo_net::http::Request::get(&url);
        if let Some(tok) = crate::web::auth::token_get() {
            req = req.header("Authorization", &format!("Bearer {}", tok));
        }
        let resp = req.send().await?;
        if !resp.ok() {
            return Ok(DEFAULT_SYSTEM_PROMPT.to_string());
        }
        let body: PersonaBody = resp.json().await?;
        Ok(body.system_prompt)
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    {
        crate::desktop::agent::runtime_ctx()?
            .persona_get()
            .await
            .map_err(Into::into)
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        Ok(DEFAULT_SYSTEM_PROMPT.to_string())
    }
}

/// 持久化 persona。
pub async fn save_persona_remote(text: &str) -> anyhow::Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!(
            "{}/v1/persona",
            crate::chat::api_base_url().trim_end_matches('/')
        );
        let mut builder = gloo_net::http::Request::put(&url);
        if let Some(tok) = crate::web::auth::token_get() {
            builder = builder.header("Authorization", &format!("Bearer {}", tok));
        }
        let resp = builder
            .json(&PersonaBody {
                system_prompt: text.to_string(),
            })?
            .send()
            .await?;
        if !resp.ok() {
            anyhow::bail!("save persona HTTP {}", resp.status());
        }
        Ok(())
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    {
        crate::desktop::agent::runtime_ctx()?
            .persona_set(text)
            .await?;
        Ok(())
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        let _ = text;
        Ok(())
    }
}

/// 使用服务端保存的 OpenAI 配置润色 system prompt。
pub async fn polish_persona_remote(text: &str, model: &str) -> anyhow::Result<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!(
            "{}/v1/persona/polish",
            crate::chat::api_base_url().trim_end_matches('/')
        );
        let mut builder = gloo_net::http::Request::post(&url);
        if let Some(tok) = crate::web::auth::token_get() {
            builder = builder.header("Authorization", &format!("Bearer {}", tok));
        }
        let resp = builder
            .json(&protocol::PersonaPolishRequest {
                system_prompt: text.to_string(),
                model: model.to_string(),
            })?
            .send()
            .await?;
        if !resp.ok() {
            let status = resp.status();
            let raw = resp.text().await.unwrap_or_default();
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(err) = v.get("error").and_then(|x| x.as_str()) {
                    anyhow::bail!("{}", err);
                }
            }
            anyhow::bail!("润色请求失败（HTTP {status}）");
        }
        let body: protocol::PersonaPolishResponse = resp.json().await?;
        Ok(body.system_prompt)
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    {
        crate::desktop::agent::runtime_ctx()?
            .persona_polish(text, model)
            .await
            .map_err(Into::into)
    }

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        let _ = (text, model);
        anyhow::bail!("persona polish is unavailable for this target")
    }
}
