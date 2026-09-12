//! Velopack 自动更新：检查 → 下载（含增量 delta）→ 重启应用。
//!
//! 仅当应用是通过 Velopack 产物（macOS `.pkg` / 便携 zip、Windows `Setup.exe`）安装时可用；
//! `dx serve` / 裸 `cargo run` / 旧 DMG 安装的实例，[`manager`] 返回 `None`，
//! 调用方（状态栏）回退到 GitHub Release API「打开发布页」。
//!
//! 所有函数均为阻塞调用，需在 `spawn_blocking` 或独立线程里执行。

use std::sync::mpsc;

use velopack::sources::GithubSource;
pub use velopack::{UpdateInfo, UpdateManager, VelopackAsset};
use velopack::UpdateCheck;

/// 发布仓库；`vpk upload github --repoUrl` 与之一致，`releases.{channel}.json` 挂在 Release 资产里。
pub const GITHUB_REPO_URL: &str = "https://github.com/1usdc/pusa";

/// 构造更新管理器；未通过 Velopack 安装时返回 `None`。
pub fn manager() -> Option<UpdateManager> {
    let source = GithubSource::new(GITHUB_REPO_URL, None, false);
    UpdateManager::new(source, None, None).ok()
}

/// 一次远端检查的结果。
pub enum Check {
    UpToDate,
    Available(Box<UpdateInfo>),
    Failed(String),
}

pub fn check(um: &UpdateManager) -> Check {
    match um.check_for_updates() {
        Ok(UpdateCheck::UpdateAvailable(info)) => Check::Available(info),
        Ok(UpdateCheck::NoUpdateAvailable) | Ok(UpdateCheck::RemoteIsEmpty) => Check::UpToDate,
        Err(e) => Check::Failed(e.to_string()),
    }
}

/// 下载更新包（优先 delta，失败自动回退全量）；`on_progress` 收到 0..=100。
pub fn download(
    um: &UpdateManager,
    info: &UpdateInfo,
    mut on_progress: impl FnMut(i16) + Send + 'static,
) -> Result<(), String> {
    let (tx, rx) = mpsc::channel::<i16>();
    let forwarder = std::thread::spawn(move || {
        for pct in rx {
            on_progress(pct);
        }
    });
    // `download_updates` 返回时 `tx` 被丢弃，forwarder 的 `for` 循环随之结束。
    let result = um.download_updates(info, Some(tx)).map_err(|e| e.to_string());
    let _ = forwarder.join();
    result
}

/// 退出当前进程、应用更新并重启。成功时不会返回。
pub fn apply_and_restart(um: &UpdateManager, asset: &VelopackAsset) -> Result<(), String> {
    um.apply_updates_and_restart(asset).map_err(|e| e.to_string())
}

/// 已下载、尚未应用的更新包（上次下载完成后用户没点重启）。
pub fn pending_restart(um: &UpdateManager) -> Option<VelopackAsset> {
    um.get_update_pending_restart()
}
