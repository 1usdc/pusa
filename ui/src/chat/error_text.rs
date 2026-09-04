//! 把后端 / 网关 / 上游的英文错误文案映射成中文友好提示，避免直接渲染
//! "HTTP 502" / "Bad Gateway" / 长堆栈到对话气泡。
//!
//! 适用场景：
//! 1. `transport_wasm::post_chat_stream` 在握手失败时（HTTP 5xx 直接 bail）。
//! 2. `apply_chat_sse_event` 处理 `SseEvent::Error` 时——后端 agent 调上游 LLM 拿到
//!    5xx 会以 `OpenAI HTTP 502: ...` 形式回到前端。
//! 3. `run_chat_turn` 整体失败时，错误转字符串后再走一道映射。

/// 根据原始错误文案，给用户一个能看懂的中文提示。
///
/// 识别策略偏宽松——后端文案大多是英文且包含 status code 与短语，命中关键字就替换。
/// 拿不准的（既不是常见 5xx 也不是 401/429/timeout）原样返回，避免误伤工具错误等真实信号。
pub fn friendly_chat_error_message(raw: &str) -> String {
    let lower = raw.to_lowercase();

    // 优先识别用户必须感知的真实错误：401 / 429。
    if lower.contains("401") || lower.contains("unauthorized") {
        return "登录已失效，请重新登录后再试".to_string();
    }
    if lower.contains("429") || lower.contains("too many requests") {
        return "请求过于频繁，请稍后再试".to_string();
    }

    // 网关 / 平台抖动类：502 / 503 / 504、Bad Gateway、Gateway Timeout、上游 /me 失败。
    let is_gateway_5xx = lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("bad gateway")
        || lower.contains("gateway time")
        || lower.contains("gateway timeout")
        || lower.contains("service unavailable")
        || lower.contains("upstream /me")
        || lower.contains("upstream me");
    if is_gateway_5xx {
        return "云平台暂时抖动，请稍后重试".to_string();
    }

    // 网络层超时 / 连接错误：reqwest / fetch 抛出的常见短语。
    let is_network = lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("connection reset")
        || lower.contains("connection refused")
        || lower.contains("network error")
        || lower.contains("failed to fetch");
    if is_network {
        return "网络暂时不可用，请稍后重试".to_string();
    }

    // 上游把 nginx/Express 错误页整页塞进 `OpenAI HTTP 500: <!DOCTYPE html>…`。
    // 500 不是 JSON `{"error":…}`，正文也不该进聊天气泡。
    if looks_like_html_error_page(&lower) {
        let head = status_line_without_html(raw);
        if head.is_empty() {
            return "上游返回了网页错误（非 JSON），请检查该密钥的 Base URL。".to_string();
        }
        return format!("{head}：对方返回了网页而不是 JSON，已隐藏 HTML。请检查 Base URL 是否为 /v1 兼容接口。");
    }

    raw.to_string()
}

fn looks_like_html_error_page(lower: &str) -> bool {
    lower.contains("<!doctype")
        || lower.contains("<html")
        || lower.contains("<pre>")
        || lower.contains("<body")
}

/// 只保留状态行（`OpenAI HTTP 500 Internal Server Error`），丢掉后面的网页。
fn status_line_without_html(raw: &str) -> String {
    let first = raw.lines().next().unwrap_or(raw);
    let cut = first
        .find('<')
        .map(|i| &first[..i])
        .unwrap_or(first)
        .trim()
        .trim_end_matches(':')
        .trim();
    cut.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_502_bad_gateway_to_chinese_friendly() {
        assert_eq!(
            friendly_chat_error_message("chat stream HTTP 502"),
            "云平台暂时抖动，请稍后重试"
        );
        assert_eq!(
            friendly_chat_error_message("OpenAI HTTP 502: <html>Bad Gateway</html>"),
            "云平台暂时抖动，请稍后重试"
        );
        assert_eq!(
            friendly_chat_error_message("upstream /me returned non-2xx 502"),
            "云平台暂时抖动，请稍后重试"
        );
    }

    #[test]
    fn maps_504_gateway_timeout() {
        assert_eq!(
            friendly_chat_error_message("Gateway Timeout"),
            "云平台暂时抖动，请稍后重试"
        );
        assert_eq!(
            friendly_chat_error_message("HTTP 504"),
            "云平台暂时抖动，请稍后重试"
        );
    }

    #[test]
    fn maps_401_to_login_expired() {
        assert_eq!(
            friendly_chat_error_message("HTTP 401 Unauthorized"),
            "登录已失效，请重新登录后再试"
        );
    }

    #[test]
    fn maps_429_to_rate_limited() {
        assert_eq!(
            friendly_chat_error_message("HTTP 429 Too Many Requests"),
            "请求过于频繁，请稍后再试"
        );
    }

    #[test]
    fn maps_network_timeout() {
        assert_eq!(
            friendly_chat_error_message("operation timed out"),
            "网络暂时不可用，请稍后重试"
        );
        assert_eq!(
            friendly_chat_error_message("Failed to fetch"),
            "网络暂时不可用，请稍后重试"
        );
    }

    #[test]
    fn passes_through_unknown_errors() {
        let raw = "tool 'web_search' returned: invalid arguments";
        assert_eq!(friendly_chat_error_message(raw), raw);
    }

    #[test]
    fn passes_through_business_logic_errors() {
        // 4xx 业务错误（非 401/429）保持原文，避免误把"模型未配置"等可操作错误吃掉。
        let raw = "HTTP 400 model required";
        assert_eq!(friendly_chat_error_message(raw), raw);
    }

    #[test]
    fn hides_html_body_on_openai_http_500() {
        let raw = "OpenAI HTTP 500 Internal Server Error: <!DOCTYPE html>\n<html lang=\"en\">\n<pre>Internal Server Error</pre>";
        let out = friendly_chat_error_message(raw);
        assert!(out.starts_with("OpenAI HTTP 500 Internal Server Error"));
        assert!(!out.to_lowercase().contains("<!doctype"));
        assert!(!out.contains("<html"));
        assert!(out.contains("网页"));
    }
}
