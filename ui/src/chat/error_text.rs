//! 对话错误文案入口。
//!
//! 当前直接返回上游 / 传输层原文，便于排查 Base URL、网关与模型端点问题。
//! 调用点仍经本函数，便于日后按需再加截断或脱敏。

/// 返回原始错误文案（不做中文友好改写）。
pub fn friendly_chat_error_message(raw: &str) -> String {
    raw.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_through_raw_errors() {
        assert_eq!(
            friendly_chat_error_message("chat stream HTTP 502"),
            "chat stream HTTP 502"
        );
        assert_eq!(
            friendly_chat_error_message("OpenAI HTTP 502: <html>Bad Gateway</html>"),
            "OpenAI HTTP 502: <html>Bad Gateway</html>"
        );
        assert_eq!(
            friendly_chat_error_message("HTTP 401 Unauthorized"),
            "HTTP 401 Unauthorized"
        );
        let raw = "OpenAI HTTP 500 Internal Server Error: <!DOCTYPE html>\n<html>";
        assert_eq!(friendly_chat_error_message(raw), raw);
    }
}
