//! 窗口底部状态栏：版本号 + 更新状态；左侧工作区/分支（点击可切换终端）；中间显示当前文件路径。

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdGitBranch;
use dioxus_free_icons::Icon;

use crate::icons::{GoFileDirectory, RiBearSmileLine};
use crate::version::app_version;

/// 状态栏轮询工作区根 / git 的间隔。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
const WORKSPACE_POLL_MS: u64 = 1500;

const GH_REPO: &str = "1usdc/pusa";
const GH_RELEASES_LATEST_PAGE: &str = "https://github.com/1usdc/pusa/releases/latest";

/// 更新检查的重试 / 周期间隔（Velopack 与 GitHub API 回退共用）。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
const UPDATE_RECHECK_SECS: u64 = 60 * 60;

#[derive(Clone, PartialEq)]
#[allow(dead_code)] // 各变体在 native / wasm 不同路径构造
enum UpdateState {
    Checking,
    UpToDate,
    /// 非 Velopack 安装（`dx serve`、旧 DMG）：只能跳到发布页手动下载。
    Available { latest: String, url: String },
    /// Velopack：后台静默下载增量包中，`pct` 为 0..=100。
    Downloading { latest: String, pct: i16 },
    /// Velopack：更新包已就位，点击即退出→应用→重启。
    RestartReady { latest: String },
    /// Velopack 下载/应用失败：退化为跳发布页。
    Failed { latest: String, msg: String },
    Unknown,
}

/// 点击版本号时要做的事。
#[derive(Clone, PartialEq)]
enum UpdateAction {
    None,
    OpenUrl(String),
    Restart,
}

/// 退出并应用已下载的 Velopack 更新；失败则回退为「打开发布页」。
fn restart_to_apply_update(mut update: Signal<UpdateState>) {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        spawn(async move {
            if let Err(msg) = velopack_update::apply_and_restart().await {
                let latest = match update() {
                    UpdateState::RestartReady { latest } => latest,
                    _ => String::new(),
                };
                update.set(UpdateState::Failed { latest, msg });
            }
        });
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        let _ = &mut update;
    }
}

fn open_external_url(url: &str) {
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        let _ = webbrowser::open(url);
    }
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    {
        if let Some(w) = web_sys::window() {
            let _ = w.open_with_url_and_target(url, "_blank");
        }
    }
    #[cfg(not(any(
        all(feature = "native", not(target_arch = "wasm32")),
        all(target_arch = "wasm32", feature = "web")
    )))]
    {
        let _ = url;
    }
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod desktop_update {
    use std::cmp::Ordering;

    use serde::Deserialize;

    use super::{app_version, UpdateState, GH_REPO};

    fn releases_api_url() -> String {
        format!("https://api.github.com/repos/{GH_REPO}/releases?per_page=20")
    }

    #[derive(Clone, Deserialize)]
    struct GhRelease {
        tag_name: String,
        html_url: String,
        #[serde(default)]
        draft: bool,
        #[serde(default)]
        prerelease: bool,
    }

    fn strip_version_prefix(raw: &str) -> &str {
        let t = raw.trim();
        t.strip_prefix("desktop-v")
            .or_else(|| t.strip_prefix('v'))
            .unwrap_or(t)
    }

    fn version_parts(v: &str) -> Vec<u64> {
        strip_version_prefix(v)
            .split(|c| c == '.' || c == '-' || c == '+')
            .take_while(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            .filter_map(|s| s.parse().ok())
            .collect()
    }

    fn cmp_version(a: &str, b: &str) -> Ordering {
        let pa = version_parts(a);
        let pb = version_parts(b);
        let n = pa.len().max(pb.len());
        for i in 0..n {
            let x = pa.get(i).copied().unwrap_or(0);
            let y = pb.get(i).copied().unwrap_or(0);
            match x.cmp(&y) {
                Ordering::Equal => {}
                other => return other,
            }
        }
        Ordering::Equal
    }

    fn pick_latest_desktop_release(releases: &[GhRelease]) -> Option<(String, String)> {
        releases.iter().find_map(|r| {
            if r.draft || r.prerelease {
                return None;
            }
            let tag = r.tag_name.trim();
            let ver = tag.strip_prefix("desktop-v")?;
            if version_parts(ver).is_empty() {
                return None;
            }
            Some((ver.to_string(), r.html_url.clone()))
        })
    }

    pub async fn fetch(local: &str) -> UpdateState {
        let client = match reqwest::Client::builder()
            .user_agent(format!("Pusa/{}", app_version()))
            .build()
        {
            Ok(c) => c,
            Err(_) => return UpdateState::Unknown,
        };
        let resp = match client.get(releases_api_url()).send().await {
            Ok(r) => r,
            Err(_) => return UpdateState::Unknown,
        };
        if !resp.status().is_success() {
            return UpdateState::Unknown;
        }
        let releases: Vec<GhRelease> = match resp.json().await {
            Ok(v) => v,
            Err(_) => return UpdateState::Unknown,
        };
        match pick_latest_desktop_release(&releases) {
            Some((latest, url)) if cmp_version(&latest, local) == Ordering::Greater => {
                UpdateState::Available { latest, url }
            }
            Some(_) => UpdateState::UpToDate,
            None => UpdateState::Unknown,
        }
    }
}

/// Velopack 路径：检查 → 后台静默下载（增量优先）→ 等用户点击重启。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod velopack_update {
    use dioxus::prelude::*;

    use super::UpdateState;
    use crate::desktop::updater::{self, Check, UpdateInfo, UpdateManager};

    /// 应用是否通过 Velopack 安装（决定走一键更新还是 GitHub API 回退）。
    pub async fn is_installed() -> bool {
        tokio::task::spawn_blocking(|| updater::manager().is_some())
            .await
            .unwrap_or(false)
    }

    /// 完整跑一轮：检查并下载。返回终态（`UpToDate` / `RestartReady` / `Failed` / `Unknown`）。
    pub async fn run_once(mut update: Signal<UpdateState>) -> UpdateState {
        let checked = tokio::task::spawn_blocking(|| {
            let um = updater::manager()?;
            let result = updater::check(&um);
            Some((um, result))
        })
        .await;

        let (um, info) = match checked {
            Ok(Some((_, Check::UpToDate))) => return UpdateState::UpToDate,
            Ok(Some((um, Check::Available(info)))) => (um, info),
            // 网络错误等：不打扰用户，下个周期再试。
            Ok(Some((_, Check::Failed(msg)))) => {
                eprintln!("[pusa] 检查更新失败：{msg}");
                return UpdateState::Unknown;
            }
            Ok(None) | Err(_) => return UpdateState::Unknown,
        };

        let latest = info.TargetFullRelease.Version.clone();
        update.set(UpdateState::Downloading {
            latest: latest.clone(),
            pct: 0,
        });
        match download(um, info, update, &latest).await {
            Ok(()) => UpdateState::RestartReady { latest },
            Err(msg) => UpdateState::Failed { latest, msg },
        }
    }

    async fn download(
        um: UpdateManager,
        info: Box<UpdateInfo>,
        mut update: Signal<UpdateState>,
        latest: &str,
    ) -> Result<(), String> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<i16>();
        let task = tokio::task::spawn_blocking(move || {
            updater::download(&um, &info, move |pct| {
                let _ = tx.send(pct);
            })
        });
        // 进度只能在 Dioxus 运行时线程写 Signal，所以经 channel 转发到这里。
        while let Some(pct) = rx.recv().await {
            update.set(UpdateState::Downloading {
                latest: latest.to_string(),
                pct: pct.clamp(0, 100),
            });
        }
        match task.await {
            Ok(r) => r,
            Err(e) => Err(format!("下载任务异常退出：{e}")),
        }
    }

    /// 应用已下载的更新并重启；成功时进程直接退出，不会返回。
    pub async fn apply_and_restart() -> Result<(), String> {
        tokio::task::spawn_blocking(|| {
            let um = updater::manager().ok_or("应用未通过 Velopack 安装")?;
            let asset = updater::pending_restart(&um).ok_or("没有已下载的更新包")?;
            updater::apply_and_restart(&um, &asset)
        })
        .await
        .map_err(|e| format!("重启任务异常退出：{e}"))?
    }
}

/// 底部状态栏（窗口 footer）。
#[component]
pub fn StatusBar(
    mut show_terminal: Signal<bool>,
    active_file_path: Signal<Option<String>>,
    active_role_name: Signal<String>,
) -> Element {
    // native / wasm 路径会 `set`；host 上仅 default feature 检查时可能无写入。
    #[allow(unused_mut)]
    let mut version = use_signal(app_version);
    #[allow(unused_mut)]
    let mut update = use_signal(|| UpdateState::Checking);
    // 工作区目录 basename；空字符串表示无可显示项目。
    #[allow(unused_mut)]
    let mut workspace_dir = use_signal(String::new);
    // `(branch, dirty)`；`None` 表示非 git 或不适用。
    #[allow(unused_mut)]
    let mut workspace_git = use_signal(|| Option::<(String, bool)>::None);
    #[allow(unused_mut)]
    let mut workspace_root_path = use_signal(String::new);

    use_hook(|| {
        spawn(async move {
            let local = app_version();
            version.set(local.clone());
            #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
            {
                let via_velopack = velopack_update::is_installed().await;
                loop {
                    let state = if via_velopack {
                        velopack_update::run_once(update).await
                    } else {
                        desktop_update::fetch(&local).await
                    };
                    let done = matches!(state, UpdateState::RestartReady { .. });
                    update.set(state);
                    if done {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(UPDATE_RECHECK_SECS)).await;
                    if matches!(update(), UpdateState::UpToDate | UpdateState::Unknown) {
                        update.set(UpdateState::Checking);
                    }
                }
            }
            #[cfg(all(target_arch = "wasm32", feature = "web"))]
            {
                // Web 与桌面共用 desktop/Cargo.toml 版本；无独立 web Release 频道。
                let _ = local;
                update.set(UpdateState::UpToDate);
            }
            #[cfg(not(any(
                all(feature = "native", not(target_arch = "wasm32")),
                all(target_arch = "wasm32", feature = "web")
            )))]
            {
                let _ = local;
                update.set(UpdateState::Unknown);
            }
        });

        #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
        spawn(async move {
            loop {
                let root = super::files::fs_workspace_root();
                if root.is_empty() {
                    workspace_dir.set(String::new());
                    workspace_git.set(None);
                    workspace_root_path.set(String::new());
                } else {
                    let name = super::files::fs_root_display_name(&root);
                    let git = super::files::fs_git_head_status(&root);
                    workspace_dir.set(name);
                    workspace_git.set(git);
                    workspace_root_path.set(root);
                }
                tokio::time::sleep(std::time::Duration::from_millis(WORKSPACE_POLL_MS)).await;
            }
        });
    });

    let terminal_title = if show_terminal() {
        "隐藏终端"
    } else {
        "显示终端"
    };
    let ver = version();
    let dir_name = workspace_dir();
    let git = workspace_git();
    let root_path = workspace_root_path();
    let (update_label, update_title, update_action) = match update() {
        UpdateState::Checking => (
            "检查中…".to_string(),
            "正在检查更新".to_string(),
            UpdateAction::None,
        ),
        // 最新版不显示文字，只保留悬停提示
        UpdateState::UpToDate => (
            String::new(),
            "当前已是最新版本".to_string(),
            UpdateAction::None,
        ),
        UpdateState::Available { latest, url } => (
            "(+1)".to_string(),
            format!("有更新：v{latest}（点击打开发布页）"),
            UpdateAction::OpenUrl(url),
        ),
        UpdateState::Downloading { latest, pct } => (
            format!("下载 v{latest} {pct}%"),
            format!("正在后台下载更新 v{latest}（增量包），完成后点击重启即可"),
            UpdateAction::None,
        ),
        UpdateState::RestartReady { latest } => (
            "重启更新".to_string(),
            format!("v{latest} 已下载，点击重启完成更新"),
            UpdateAction::Restart,
        ),
        UpdateState::Failed { latest, msg } => (
            "(+1)".to_string(),
            format!("v{latest} 自动更新失败：{msg}（点击打开发布页手动下载）"),
            UpdateAction::OpenUrl(GH_RELEASES_LATEST_PAGE.to_string()),
        ),
        UpdateState::Unknown => (
            String::new(),
            format!("Pusa · {GH_REPO}"),
            UpdateAction::None,
        ),
    };
    let update_clickable = !matches!(update_action, UpdateAction::None);
    let update_is_restart = matches!(update_action, UpdateAction::Restart);

    let workspace_title = if dir_name.is_empty() {
        String::new()
    } else if let Some((ref branch, dirty)) = git {
        let star = if dirty { "*" } else { "" };
        format!("{root_path} · {branch}{star}")
    } else {
        root_path.clone()
    };
    let branch_label = git.as_ref().map(|(branch, dirty)| {
        if *dirty {
            format!("{branch}*")
        } else {
            branch.clone()
        }
    });

    let workspace_click_title = if workspace_title.is_empty() {
        terminal_title.to_string()
    } else {
        format!("{workspace_title} · {terminal_title}")
    };
    let file_path = active_file_path();
    let role_name = active_role_name();

    rsx! {
        footer { class: "ac-status-bar", role: "contentinfo",
            div { class: "ac-status-bar-left",
                if !role_name.is_empty() {
                    span {
                        class: "ac-status-bar-role",
                        title: "当前角色：{role_name}",
                        span {
                            class: "ac-status-bar-role-icon",
                            aria_hidden: "true",
                            Icon {
                                icon: RiBearSmileLine,
                                width: 12,
                                height: 12,
                                fill: "currentColor",
                            }
                        }
                        span { class: "ac-status-bar-role-name", "{role_name}" }
                    }
                }
                if !dir_name.is_empty() {
                    button {
                        r#type: "button",
                        class: if show_terminal() {
                            "ac-status-bar-workspace is-active"
                        } else {
                            "ac-status-bar-workspace"
                        },
                        title: "{workspace_click_title}",
                        aria_label: "{terminal_title}",
                        aria_expanded: show_terminal(),
                        onclick: move |_| show_terminal.toggle(),
                        span {
                            class: "ac-status-bar-dir-icon",
                            aria_hidden: "true",
                            Icon {
                                icon: GoFileDirectory,
                                width: 12,
                                height: 12,
                                fill: "currentColor",
                            }
                        }
                        span { class: "ac-status-bar-dir", "{dir_name}" }
                        if let Some(label) = branch_label.as_ref() {
                            span { class: "ac-status-bar-branch",
                                span {
                                    class: "ac-status-bar-branch-icon",
                                    aria_hidden: "true",
                                    Icon {
                                        icon: LdGitBranch,
                                        width: 12,
                                        height: 12,
                                        fill: "currentColor",
                                    }
                                }
                                span { class: "ac-status-bar-branch-name", "{label}" }
                            }
                        }
                    }
                }
            }
            div { class: "ac-status-bar-center",
                if let Some(path) = file_path.as_ref() {
                    span {
                        class: "ac-status-bar-file-path",
                        title: "{path}",
                        "{path}"
                    }
                }
            }
            div { class: "ac-status-bar-right",
                if update_clickable {
                    button {
                        r#type: "button",
                        class: if update_is_restart {
                            "ac-status-bar-version is-update is-restart"
                        } else {
                            "ac-status-bar-version is-update"
                        },
                        title: "{update_title}",
                        onclick: move |_| {
                            match update_action.clone() {
                                UpdateAction::OpenUrl(url) => open_external_url(&url),
                                UpdateAction::Restart => restart_to_apply_update(update),
                                UpdateAction::None => {}
                            }
                        },
                        span { "v{ver} " }
                        span { class: "ac-status-bar-update", "{update_label}" }
                    }
                } else {
                    span {
                        class: "ac-status-bar-version",
                        title: "{update_title}",
                        span { "v{ver}" }
                        if !update_label.is_empty() {
                            span { class: "ac-status-bar-update-muted", " · {update_label}" }
                        }
                    }
                }
            }
        }
    }
}
