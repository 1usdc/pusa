//! `same-origin-api` 模式下基础 URL 的统一拼装：把页面挂载的子路径前缀考虑进去。
//!
//! 背景：线上 Another-Me-LLM 网关把每个 ECS 实例对外暴露成
//! `https://api.anotherme.co/proxy/i-{instance_id}/`（外层网关 strip 该前缀后再透传到
//! 实例内部 nginx）。WASM 在 same-origin 模式下用 `{origin}/llm`
//! / `{origin}/v1` 起请求，会**直接拼成域名 root 下的 path**，绕开 `/proxy/i-xxx/`
//! 这层 —— 外层网关命中不到任何规则，统一 404。
//!
//! 修复方案：运行时检测 `location.pathname`，识别形如 `/proxy/i-{instance_id}` 的网关
//! 前缀并拼回去。其它部署（自有域名直挂 root、本机 dev、桌面端）pathname 不匹配此
//! pattern，返回空串。

#![cfg(all(target_arch = "wasm32", feature = "web", feature = "same-origin-api"))]

/// 从 `location.pathname` 提取应用挂载的子路径前缀。
///
/// 当前只识别 Another-Me-LLM 网关的 per-instance proxy 形态：
/// `pathname == "/proxy/i-xxx/..."` → 返回 `"/proxy/i-xxx"`。其它形态返回空串。
///
/// 不引入正则；网关前缀靠两条硬约束识别（一级路径恰为 `"proxy"` + 二级以 `"i-"`
/// 开头）。万一以后网关换 schema，扩这一处即可，调用方无感。
pub fn mount_prefix() -> String {
    let Some(w) = web_sys::window() else {
        return String::new();
    };
    let pathname = match w.location().pathname() {
        Ok(p) => p,
        Err(_) => return String::new(),
    };
    detect_mount_prefix(&pathname)
}

fn detect_mount_prefix(pathname: &str) -> String {
    let mut parts = pathname.trim_start_matches('/').splitn(3, '/');
    if let (Some(a), Some(b)) = (parts.next(), parts.next()) {
        if a == "proxy" && b.starts_with("i-") && b.len() > 2 {
            return format!("/{a}/{b}");
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::detect_mount_prefix;

    #[test]
    fn matches_proxy_instance() {
        assert_eq!(
            detect_mount_prefix("/proxy/i-j6c10lac1dpb5vqj75yz/"),
            "/proxy/i-j6c10lac1dpb5vqj75yz"
        );
        assert_eq!(
            detect_mount_prefix("/proxy/i-j6c10lac1dpb5vqj75yz/some/page"),
            "/proxy/i-j6c10lac1dpb5vqj75yz"
        );
    }

    #[test]
    fn ignores_non_proxy_paths() {
        assert_eq!(detect_mount_prefix("/"), "");
        assert_eq!(detect_mount_prefix(""), "");
        assert_eq!(detect_mount_prefix("/proxy/"), "");
        assert_eq!(detect_mount_prefix("/proxy/not-instance/"), "");
        assert_eq!(detect_mount_prefix("/proxy/i-/"), "");
    }
}
