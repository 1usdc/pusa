//! 窗口底部状态栏：版本号 + 更新状态；左侧工作区/分支（点击可切换终端）；中间显示当前文件路径。

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdGitBranch;
use dioxus_free_icons::Icon;

use crate::icons::{GoFileDirectory, RiBearSmileLine};

/// 仓库根目录 `.version`（编译期嵌入，显示与更新比对共用）。
const APP_VERSION_FILE: &str = include_str!("../../../.version");

/// 状态栏轮询工作区根 / git 的间隔。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
const WORKSPACE_POLL_MS: u64 = 1500;

fn app_version() -> &'static str {
    APP_VERSION_FILE.trim()
}

const GH_REPO: &str = "Another-Me-Labs/Another-Claw-Rs";
const GH_RELEASES_LATEST_PAGE: &str =
    "https://github.com/Another-Me-Labs/Another-Claw-Rs/releases/latest";

#[derive(Clone, PartialEq)]
#[allow(dead_code)] // UpToDate / Available 在 native / wasm 路径构造
enum UpdateState {
    Checking,
    UpToDate,
    Available { latest: String, url: String },
    Unknown,
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

    use super::{app_version, UpdateState};

    const GH_RELEASES_API: &str =
        "https://api.github.com/repos/Another-Me-Labs/Another-Claw-Rs/releases?per_page=20";

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
        let resp = match client.get(GH_RELEASES_API).send().await {
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

/// 底部状态栏（窗口 footer）。
#[component]
pub fn StatusBar(
    mut show_terminal: Signal<bool>,
    active_file_path: Signal<Option<String>>,
    active_role_name: Signal<String>,
) -> Element {
    // native / wasm 路径会 `set`；host 上仅 default feature 检查时可能无写入。
    #[allow(unused_mut)]
    let mut version = use_signal(|| app_version().to_string());
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
            let local = app_version().to_string();
            version.set(local.clone());
            #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
            {
                update.set(desktop_update::fetch(&local).await);
            }
            #[cfg(all(target_arch = "wasm32", feature = "web"))]
            {
                // Web 与桌面共用编译期 `.version`；无独立 web Release 频道。
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
    let (update_label, update_title, update_url, update_clickable) = match update() {
        UpdateState::Checking => (
            "检查中…".to_string(),
            "正在检查更新".to_string(),
            None,
            false,
        ),
        UpdateState::UpToDate => (
            "已是最新".to_string(),
            "当前已是最新版本".to_string(),
            None,
            false,
        ),
        UpdateState::Available { latest, url } => (
            "(+1)".to_string(),
            format!("有更新：v{latest}（点击打开发布页）"),
            Some(url),
            true,
        ),
        UpdateState::Unknown => (String::new(), format!("Pusa · {GH_REPO}"), None, false),
    };

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
                        class: "ac-status-bar-version is-update",
                        title: "{update_title}",
                        onclick: move |_| {
                            let url = update_url
                                .clone()
                                .unwrap_or_else(|| GH_RELEASES_LATEST_PAGE.to_string());
                            open_external_url(&url);
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
