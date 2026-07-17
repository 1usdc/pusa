//! build script：仅用于让 cargo 在编译期注入的环境变量发生变化时自动重编 ui crate。
//!
//! `option_env!("REOWN_PROJECT_ID")` 是 rustc 编译期宏，cargo 默认不会跟踪这个环境变量。
//! 不加这段提示，开发者改完 `docker/.env` 重启 `just web` 后 cargo 仍会复用旧产物，登录页
//! 拿到的依然是空 projectId，导致 reown AppKit 不初始化、`<appkit-button>` 不显示。
fn main() {
    println!("cargo:rerun-if-env-changed=REOWN_PROJECT_ID");
    println!("cargo:rerun-if-env-changed=ANOTHERME_BASE_URL");
    println!("cargo:rerun-if-env-changed=ANOTHERME_RELAY_BASE_URL");
}
