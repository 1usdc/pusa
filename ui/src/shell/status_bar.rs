//! 窗口底部状态栏：版本号 + 更新状态；左侧项目名（最近项目弹窗）与分支名（分支切换弹窗）；中间显示当前文件路径。

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdGitBranch, LdTrash2};
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
    Available {
        latest: String,
        url: String,
    },
    /// Velopack：后台静默下载增量包中，`pct` 为 0..=100。
    Downloading {
        latest: String,
        pct: i16,
    },
    /// Velopack：更新包已就位，点击即退出→应用→重启。
    RestartReady {
        latest: String,
    },
    /// Velopack 下载/应用失败：退化为跳发布页。
    Failed {
        latest: String,
        msg: String,
    },
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

    /// 与 `scripts/velopack-common.sh` 的 `vpk_channel`、Windows 打包脚本保持一致。
    fn release_channel() -> &'static str {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            "macos-arm64"
        }
        #[cfg(all(target_os = "macos", not(target_arch = "aarch64")))]
        {
            "macos-x64"
        }
        #[cfg(target_os = "windows")]
        {
            "win-x64"
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            "linux-x64"
        }
    }

    /// `vpk upload github` 上传的更新清单，挂在 Release 资产里。
    /// 走 `releases/latest/download/...` 直链，不经过 `api.github.com`，没有 60 次/小时限流。
    fn manifest_url() -> String {
        format!(
            "https://github.com/{GH_REPO}/releases/latest/download/releases.{}.json",
            release_channel()
        )
    }

    #[derive(Clone, Deserialize)]
    struct ReleasesManifest {
        #[serde(rename = "Assets", default)]
        assets: Vec<ManifestAsset>,
    }

    #[derive(Clone, Deserialize)]
    struct ManifestAsset {
        #[serde(rename = "Version", default)]
        version: String,
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

    fn latest_version(manifest: &ReleasesManifest) -> Option<&str> {
        manifest
            .assets
            .iter()
            .map(|a| a.version.trim())
            .filter(|v| !version_parts(v).is_empty())
            .max_by(|a, b| cmp_version(a, b))
    }

    pub async fn fetch(local: &str) -> UpdateState {
        let client = match reqwest::Client::builder()
            .user_agent(format!("Pusa/{}", app_version()))
            .build()
        {
            Ok(c) => c,
            Err(_) => return UpdateState::Unknown,
        };
        let resp = match client.get(manifest_url()).send().await {
            Ok(r) => r,
            Err(_) => return UpdateState::Unknown,
        };
        if !resp.status().is_success() {
            return UpdateState::Unknown;
        }
        let manifest: ReleasesManifest = match resp.json().await {
            Ok(v) => v,
            Err(_) => return UpdateState::Unknown,
        };
        match latest_version(&manifest) {
            Some(latest) if cmp_version(latest, local) == Ordering::Greater => {
                UpdateState::Available {
                    url: format!("https://github.com/{GH_REPO}/releases/tag/desktop-v{latest}"),
                    latest: latest.to_string(),
                }
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

    use super::{desktop_update, UpdateState};
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
            // Velopack 的 GithubSource 走 `api.github.com`，可能被 60 次/小时限流；
            // 失败时改用 Release 资产里的清单再查一次。
            Ok(Some((_, Check::Failed(msg)))) => {
                eprintln!("[pusa] 更新检查失败：{msg}，改用 Release 清单");
                return desktop_update::fetch(&crate::version::app_version()).await;
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
    show_terminal: Signal<bool>,
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
    // 项目 / 分支弹窗：点击状态栏左侧两个按钮时向上弹出。
    let mut show_projects = use_signal(|| false);
    let mut show_branches = use_signal(|| false);
    let mut recent_dirs = use_signal(Vec::<String>::new);
    let mut branch_list = use_signal(Vec::<String>::new);
    let toast = super::toast::use_toast();
    let mut open_project_path = use_context::<super::OpenProjectPath>().0;

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

    // 终端开关保留在标题栏 / 聊天栏按钮；状态栏不再承担该入口。
    let _ = show_terminal;
    let ver = version();
    let dir_name = workspace_dir();
    let git = workspace_git();
    let root_path = workspace_root_path();
    let branch_root = root_path.clone();
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

    let branch_label = git.as_ref().map(|(branch, dirty)| {
        if *dirty {
            format!("{branch}*")
        } else {
            branch.clone()
        }
    });

    let file_path = active_file_path();
    let role_name = active_role_name();

    rsx! {
        footer {
            class: "relative z-20 grid shrink-0 grid-cols-[minmax(0,1fr)_minmax(0,2fr)_minmax(0,1fr)] items-center gap-2 min-h-[1.625rem] px-[0.65rem] box-border border-t border-primary-2 bg-[var(--ac-chrome-bar-bg)] text-[0.6875rem] leading-none text-text-secondary select-none",
            role: "contentinfo",
            div { class: "flex items-center gap-[0.45rem] min-w-0 justify-self-start",
                if !role_name.is_empty() {
                    span {
                        class: "inline-flex items-center gap-[0.28rem] max-w-[9.5rem] min-w-0 shrink tracking-[-0.01em] text-secondary-1",
                        title: "当前角色：{role_name}",
                        span {
                            class: "inline-flex items-center justify-center shrink-0 opacity-90",
                            aria_hidden: "true",
                            Icon {
                                icon: RiBearSmileLine,
                                width: 12,
                                height: 12,
                                fill: "currentColor",
                            }
                        }
                        span { class: "inline-block min-w-0 overflow-hidden text-ellipsis whitespace-nowrap font-normal", "{role_name}" }
                    }
                }
                if !dir_name.is_empty() {
                    // 弹窗打开时，点击空白处关闭。
                    if show_projects() || show_branches() {
                        div {
                            class: "fixed inset-0 z-30",
                            onclick: move |_| {
                                show_projects.set(false);
                                show_branches.set(false);
                            },
                        }
                    }
                    // 项目名：向上弹出最近打开的项目。
                    div { class: "relative",
                        button {
                            r#type: "button",
                            class: "inline-flex items-center gap-[0.35rem] min-w-0 max-w-[14rem] h-5 px-1 m-0 border border-transparent rounded-[0.35rem] bg-transparent text-text-secondary font-[inherit] cursor-pointer hover:border-primary-2 hover:bg-primary-3 data-[active=true]:border-primary-2 data-[active=true]:bg-primary-3",
                            "data-active": show_projects(),
                            title: "{root_path} · 切换最近打开的项目",
                            aria_expanded: show_projects(),
                            onclick: move |_| {
                                let next = !show_projects();
                                if next {
                                    recent_dirs.set(super::files::fs_recent_project_dirs());
                                }
                                show_projects.set(next);
                                show_branches.set(false);
                            },
                            span {
                                class: "shrink-0 inline-flex items-center justify-center leading-[0] text-secondary-1 opacity-90",
                                aria_hidden: "true",
                                Icon {
                                    icon: GoFileDirectory,
                                    width: 12,
                                    height: 12,
                                    fill: "currentColor",
                                }
                            }
                            span { class: "overflow-hidden text-ellipsis whitespace-nowrap max-w-[12rem] text-secondary-1 font-medium", "{dir_name}" }
                        }
                        if show_projects() {
                            div { class: "absolute left-0 bottom-[calc(100%+6px)] z-40 min-w-[230px] max-w-[340px] rounded-[8px] border border-primary-2 bg-primary py-[4px] shadow-[0_10px_28px_var(--shadow-color)]",
                                if recent_dirs().is_empty() {
                                    p { class: "m-0 px-[10px] py-[6px] text-secondary-3", "暂无最近打开的项目" }
                                }
                                {recent_dirs().into_iter().map(|dir| {
                                    let name = super::files::fs_root_display_name(&dir);
                                    let is_current = dir == root_path;
                                    let dir_click = dir.clone();
                                    let dir_remove = dir.clone();
                                    rsx! {
                                        div {
                                            key: "{dir}",
                                            class: "flex items-center gap-[2px] rounded-[5px] hover:bg-[var(--ac-item-hover-bg)] data-[current=true]:bg-[var(--ac-item-selected-bg)]",
                                            "data-current": is_current,
                                            button {
                                                r#type: "button",
                                                class: "flex min-w-0 flex-1 flex-col items-start gap-[2px] px-[10px] py-[6px] border-0 rounded-[5px] bg-transparent text-left font-[inherit] cursor-pointer",
                                                title: "{dir}",
                                                onclick: move |_| {
                                                    open_project_path.set(dir_click.clone());
                                                    show_projects.set(false);
                                                },
                                                span { class: "text-secondary-1 font-medium", "{name}" }
                                                span { class: "max-w-full overflow-hidden text-ellipsis whitespace-nowrap text-secondary-3 text-[0.625rem]", "{dir}" }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "mr-[6px] shrink-0 inline-flex h-[20px] w-[20px] items-center justify-center border-0 rounded-[4px] bg-transparent text-secondary-3 cursor-pointer hover:text-[#dc2626] hover:bg-[var(--primary-color-2)]",
                                                title: "从最近列表移除",
                                                aria_label: "从最近列表移除",
                                                onclick: move |_| {
                                                    super::files::fs_forget_recent_project_dir(&dir_remove);
                                                    recent_dirs.set(super::files::fs_recent_project_dirs());
                                                },
                                                Icon {
                                                    icon: LdTrash2,
                                                    width: 12,
                                                    height: 12,
                                                }
                                            }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                    // 分支名：向上弹出本地分支并切换。
                    if let Some(label) = branch_label.as_ref() {
                        div { class: "relative",
                            button {
                                r#type: "button",
                                class: "inline-flex items-center gap-[0.2rem] min-w-0 h-5 px-1 m-0 border border-transparent rounded-[0.35rem] bg-transparent text-text-secondary font-[inherit] cursor-pointer hover:border-primary-2 hover:bg-primary-3 data-[active=true]:border-primary-2 data-[active=true]:bg-primary-3",
                                "data-active": show_branches(),
                                title: "切换分支",
                                aria_expanded: show_branches(),
                                onclick: move |_| {
                                    let next = !show_branches();
                                    if next {
                                        branch_list.set(super::files::fs_git_branches(&branch_root));
                                    }
                                    show_branches.set(next);
                                    show_projects.set(false);
                                },
                                span {
                                    class: "shrink-0 inline-flex items-center justify-center leading-[0] opacity-85",
                                    aria_hidden: "true",
                                    Icon {
                                        icon: LdGitBranch,
                                        width: 12,
                                        height: 12,
                                        fill: "currentColor",
                                    }
                                }
                                span { class: "overflow-hidden text-ellipsis whitespace-nowrap max-w-[10rem]", "{label}" }
                            }
                            if show_branches() {
                                div { class: "absolute left-0 bottom-[calc(100%+6px)] z-40 min-w-[180px] max-w-[300px] rounded-[8px] border border-primary-2 bg-primary py-[4px] shadow-[0_10px_28px_var(--shadow-color)]",
                                    if branch_list().is_empty() {
                                        p { class: "m-0 px-[10px] py-[6px] text-secondary-3", "未找到本地分支" }
                                    }
                                    {branch_list().into_iter().map(|b| {
                                        let is_current = git.as_ref().is_some_and(|(cur, _)| *cur == b);
                                        let b_click = b.clone();
                                        let root_click = root_path.clone();
                                        rsx! {
                                            button {
                                                r#type: "button",
                                                key: "{b}",
                                                class: "block w-full px-[10px] py-[6px] border-0 rounded-[5px] bg-transparent text-left font-[inherit] cursor-pointer text-text-secondary hover:bg-[var(--ac-item-hover-bg)] data-[current=true]:bg-[var(--ac-item-selected-bg)] data-[current=true]:text-secondary-1",
                                                "data-current": is_current,
                                                onclick: move |_| {
                                                    match super::files::fs_git_checkout(&root_click, &b_click) {
                                                        Ok(()) => {
                                                            workspace_git.set(
                                                                super::files::fs_git_head_status(&root_click),
                                                            );
                                                            toast.success(format!("已切换到分支：{b_click}"));
                                                        }
                                                        Err(err) => toast.error(err),
                                                    }
                                                    show_branches.set(false);
                                                },
                                                "{b}"
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }
            }
            div { class: "flex items-center justify-center justify-self-center min-w-0 max-w-full overflow-hidden",
                if let Some(path) = file_path.as_ref() {
                    span {
                        class: "block max-w-full overflow-hidden text-ellipsis whitespace-nowrap text-center text-secondary-3 font-medium",
                        title: "{path}",
                        "{path}"
                    }
                }
            }
            div { class: "flex items-center justify-end justify-self-end min-w-0",
                if update_clickable {
                    button {
                        r#type: "button",
                        class: "group inline-flex items-baseline gap-[0.1rem] max-w-full px-[0.2rem] py-[0.1rem] rounded-[0.25rem] bg-transparent text-text-secondary whitespace-nowrap cursor-pointer hover:text-secondary-1 hover:bg-primary-3",
                        title: "{update_title}",
                        onclick: move |_| {
                            match update_action.clone() {
                                UpdateAction::OpenUrl(url) => open_external_url(&url),
                                UpdateAction::Restart => restart_to_apply_update(update),
                                UpdateAction::None => {}
                            }
                        },
                        span { "v{ver} " }
                        // 可重启：更新标签做成蓝底胶囊提示点击；普通可用态：蓝色加粗文字
                        span {
                            class: if update_is_restart {
                                "px-[0.35rem] rounded-full bg-ac-blue text-white group-hover:brightness-110"
                            } else {
                                "text-ac-blue font-semibold"
                            },
                            "{update_label}"
                        }
                    }
                } else {
                    span {
                        class: "inline-flex items-baseline gap-[0.1rem] max-w-full px-[0.2rem] py-[0.1rem] rounded-[0.25rem] bg-transparent text-text-secondary whitespace-nowrap",
                        title: "{update_title}",
                        span { "v{ver}" }
                        if !update_label.is_empty() {
                            span { class: "text-text-tertiary", " · {update_label}" }
                        }
                    }
                }
            }
        }
    }
}
