//! 应用版本：统一取自 `desktop/Cargo.toml` 的 `[package].version`（编译期嵌入）。

const DESKTOP_CARGO_TOML: &str = include_str!("../../desktop/Cargo.toml");

/// 解析 Cargo.toml 中 `[package]` 段的 `version = "..."`。
pub fn parse_cargo_package_version(toml: &str) -> Option<String> {
    let mut in_package = false;
    for line in toml.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some(rest) = line.strip_prefix("version") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let v = rest.trim().trim_matches('"').trim();
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    None
}

/// 状态栏 / 关于 / 更新检查共用的应用版本号。
pub fn app_version() -> String {
    parse_cargo_package_version(DESKTOP_CARGO_TOML).unwrap_or_else(|| "0.0.0".to_string())
}
