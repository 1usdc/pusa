//! `same-origin-api` 模式下基础 URL 的统一拼装：把页面挂载的子路径前缀考虑进去。
//!
//! 若站点挂在子路径（例如网关 `…/proxy/i-{instance_id}/`），WASM 在 same-origin
//! 模式下用 `{origin}/v1` 起请求时需要带上该前缀，否则会打到域名 root 并 404。
//!
//! 运行时检测 `location.pathname`，识别形如 `/proxy/i-{instance_id}` 的挂载前缀
//! 并拼回去。其它部署（自有域名直挂 root、本机 dev、桌面端）pathname 不匹配此
//! pattern，返回空串。

#![cfg(all(target_arch = "wasm32", feature = "web", feature = "same-origin-api"))]

/// 从 `location.pathname` 提取应用挂载的子路径前缀。
///
/// 当前识别：`pathname == "/proxy/i-xxx/..."` → 返回 `"/proxy/i-xxx"`。其它形态返回空串。
///
/// 不引入正则；前缀靠两条硬约束识别（一级路径恰为 `"proxy"` + 二级以 `"i-"`
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
