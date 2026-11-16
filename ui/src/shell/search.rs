//! 侧栏工作区内容搜索：桌面端扫文件；Web 端提示不可用。

use std::collections::{HashMap, HashSet};
use std::path::Path;

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdCaseSensitive, LdCaseUpper, LdChevronDown, LdChevronRight, LdFile, LdRefreshCw, LdRegex,
    LdReplaceAll, LdSearch, LdWholeWord, LdX,
};
use dioxus_free_icons::Icon;
use keyboard_types::Key;

use crate::icons::AiOutlineVerticalAlignBottom;

const SEARCH_DEBOUNCE_MS: u64 = 80;
const SEARCH_TIMEOUT_SECS: u64 = 25;
const MAX_MATCHES: usize = 2000;
const MAX_FILES: usize = 200;
const MAX_FILES_SCANNED: usize = 8_000;
const MAX_LINE_CHARS: usize = 240;
/// 结果自动展开的文件数，其余默认折叠。
const AUTO_EXPAND_FILES: usize = 5;
/// 文件数超过该值时启用虚拟滚动，只渲染可见区域。
const VIRTUAL_MIN_FILES: usize = 50;
const VIRTUAL_OVERSCAN: usize = 4;
/// 虚拟滚动用的估算行高（与 CSS 实际高度允许有偏差）。
const EST_FILE_HEAD_PX: f64 = 28.0;
const EST_HIT_ROW_PX: f64 = 22.0;
const EST_VIEWPORT_PX: f64 = 560.0;
/// 流式推送节奏：每累积这么多条匹配、或距上次推送超过 50ms，就发一次快照。
const STREAM_FLUSH_MATCHES: usize = 24;
const STREAM_FLUSH_MS: u64 = 50;
const SKIP_DIR_NAMES: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".venv",
    "venv",
    "__pycache__",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
    ".idea",
    ".vscode",
    "Pods",
    "vendor",
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchHit {
    line_no: u32,
    line: String,
    start: usize,
    end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchFileHits {
    path: String,
    rel: String,
    hits: Vec<SearchHit>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct SearchOutcome {
    files: Vec<SearchFileHits>,
    match_count: usize,
    truncated: bool,
    error: Option<String>,
}

#[derive(Clone, Debug)]
struct SearchRequest {
    root: String,
    query: String,
    case_sensitive: bool,
    whole_word: bool,
    regex: bool,
    include: String,
    exclude: String,
}

/// 文件组展开状态：`open` 为当前展开集合；`user_collapsed` 记录用户手动折叠过的路径，
/// 流式追加结果时不再自动展开它们，避免打断用户操作。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ExpandState {
    open: HashSet<String>,
    user_collapsed: HashSet<String>,
}

/// 流式搜索事件：搜索线程边扫边推，UI 增量渲染。
#[derive(Clone, Debug)]
enum SearchEvent {
    Progress(SearchOutcome),
    Done(SearchOutcome),
    Error(String),
}

/// 侧栏「搜索」面板。
#[component]
pub fn SidebarSearchPanel(
    root_path: String,
    dirty: Signal<HashSet<String>>,
    mut drafts: Signal<HashMap<String, String>>,
    mut baselines: Signal<HashMap<String, String>>,
    on_open_file: EventHandler<String>,
) -> Element {
    let toast = super::toast::use_toast();
    let mut query = use_signal(String::new);
    let mut replace_text = use_signal(String::new);
    let mut replace_open = use_signal(|| false);
    let mut case_sensitive = use_signal(|| false);
    let mut whole_word = use_signal(|| false);
    let mut use_regex = use_signal(|| false);
    let mut preserve_case = use_signal(|| false);
    let mut include_glob = use_signal(String::new);
    let mut exclude_glob = use_signal(String::new);
    let mut exclude_open = use_signal(|| false);
    let mut outcome = use_signal(SearchOutcome::default);
    let mut loading = use_signal(|| false);
    let mut search_epoch = use_signal(|| 0_u64);
    let mut expanded = use_signal(ExpandState::default);
    let mut replacing = use_signal(|| false);
    let mut scroll_top = use_signal(|| 0.0_f64);

    if !super::files::fs_available() {
        return rsx! {
            div { class: "ac-sidebar-search",
                div { class: "ac-sidebar-files-unavailable",
                    p { "Web 端无法搜索本机文件" }
                    p { class: "ac-sidebar-files-unavailable-hint",
                        "请使用桌面应用打开工作区并搜索文件内容。"
                    }
                }
            }
        };
    }

    let current = outcome();
    let has_query = !query().trim().is_empty();
    let file_count = current.files.len();
    let match_count = current.match_count;
    let summary = if let Some(err) = current.error.as_ref() {
        err.clone()
    } else if !has_query {
        "输入关键词即可实时搜索文件内容".to_string()
    } else if loading() && match_count == 0 {
        "正在搜索…".to_string()
    } else if match_count == 0 {
        "未找到结果".to_string()
    } else {
        // 流式搜索中：保留旧结果统计，边搜边涨。
        let mut s = format!("已找到 {match_count} 个结果 · {file_count} 个文件");
        if current.truncated {
            s.push_str("（已截断）");
        }
        if loading() {
            s.push_str("（搜索中…）");
        }
        s
    };

    // 虚拟滚动：文件数超过阈值时只渲染可见窗口，其余用 spacer 撑高。
    let expanded_now = expanded();
    let total_files = current.files.len();
    let virtualize = total_files > VIRTUAL_MIN_FILES;
    let mut offsets: Vec<f64> = Vec::with_capacity(total_files);
    let mut heights: Vec<f64> = Vec::with_capacity(total_files);
    let mut acc_px = 0.0_f64;
    for g in &current.files {
        offsets.push(acc_px);
        let h = if expanded_now.open.contains(&g.path) {
            EST_FILE_HEAD_PX + g.hits.len() as f64 * EST_HIT_ROW_PX
        } else {
            EST_FILE_HEAD_PX
        };
        heights.push(h);
        acc_px += h;
    }
    let total_px = acc_px;
    let st = scroll_top();
    let (view_start, view_end) = if virtualize {
        let bottom = st + EST_VIEWPORT_PX;
        let mut s = 0usize;
        while s + 1 < total_files && offsets[s] + heights[s] < st {
            s += 1;
        }
        s = s.saturating_sub(VIRTUAL_OVERSCAN);
        let mut e = s;
        while e < total_files && offsets[e] < bottom {
            e += 1;
        }
        (s, (e + VIRTUAL_OVERSCAN).min(total_files))
    } else {
        (0, total_files)
    };
    let top_pad_px = if virtualize && view_start > 0 {
        offsets[view_start]
    } else {
        0.0
    };
    let bottom_pad_px = if virtualize && view_end < total_files {
        total_px - offsets[view_end]
    } else {
        0.0
    };

    rsx! {
        div { class: "ac-sidebar-search",
            div { class: "ac-sidebar-search-header",
                span { class: "ac-sidebar-search-title", title: "{root_path}", "搜索" }
                div { class: "ac-sidebar-search-actions",
                    button {
                        r#type: "button",
                        class: "ac-sidebar-files-action",
                        title: "刷新",
                        aria_label: "刷新",
                        disabled: loading() || !has_query,
                        onclick: move |_| {

                            kick_search(
                                query(),
                                case_sensitive(),
                                whole_word(),
                                use_regex(),
                                include_glob(),
                                exclude_glob(),
                                0,
                                search_epoch,
                                loading,
                                outcome,
                                expanded,
                            );
                        },
                        Icon { icon: LdRefreshCw, width: 14, height: 14, fill: "currentColor" }
                    }
                    button {
                        r#type: "button",
                        class: "ac-sidebar-files-action",
                        title: "清除结果",
                        aria_label: "清除结果",
                        onclick: move |_| {
                            search_epoch.with_mut(|n| *n += 1);
                            outcome.set(SearchOutcome::default());
                            expanded.set(ExpandState::default());
                            loading.set(false);
                            scroll_top.set(0.0);
                        },
                        Icon { icon: LdX, width: 14, height: 14, fill: "currentColor" }
                    }
                    button {
                        r#type: "button",
                        class: "ac-sidebar-files-action",
                        title: "全部折叠",
                        aria_label: "全部折叠",
                        onclick: move |_| {
                            // 全部折叠：同时记入 user_collapsed，防止流式追加时被重新展开。
                            let paths: HashSet<String> =
                                outcome().files.iter().map(|f| f.path.clone()).collect();
                            expanded.set(ExpandState {
                                open: HashSet::new(),
                                user_collapsed: paths,
                            });
                        },
                        Icon { icon: AiOutlineVerticalAlignBottom, width: 14, height: 14, fill: "currentColor" }
                    }
                }
            }

            div { class: "ac-sidebar-search-fields",
                div { class: "ac-search-pair",
                    button {
                        r#type: "button",
                        class: "ac-search-pair-toggle",
                        title: if replace_open() { "收起替换" } else { "展开替换" },
                        aria_label: if replace_open() { "收起替换" } else { "展开替换" },
                        aria_expanded: replace_open(),
                        onclick: move |_| {
                            replace_open.with_mut(|v| *v = !*v);
                        },
                        if replace_open() {
                            Icon { icon: LdChevronDown, width: 14, height: 14, fill: "currentColor" }
                        } else {
                            Icon { icon: LdChevronRight, width: 14, height: 14, fill: "currentColor" }
                        }
                    }
                    div { class: "ac-search-pair-inputs",
                        div { class: "ac-search-input-row",
                            input {
                                r#type: "search",
                                class: "ac-search-input",
                                value: "{query}",
                                placeholder: "搜索",
                                spellcheck: false,
                                oninput: move |e| {
                                    query.set(e.value());

                                    kick_search(
                                        e.value(),
                                        case_sensitive(),
                                        whole_word(),
                                        use_regex(),
                                        include_glob(),
                                        exclude_glob(),
                                        SEARCH_DEBOUNCE_MS,
                                        search_epoch,
                                        loading,
                                        outcome,
                                        expanded,
                                    );
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() != Key::Enter {
                                        return;
                                    }
                                    e.prevent_default();

                                    kick_search(
                                        query(),
                                        case_sensitive(),
                                        whole_word(),
                                        use_regex(),
                                        include_glob(),
                                        exclude_glob(),
                                        0,
                                        search_epoch,
                                        loading,
                                        outcome,
                                        expanded,
                                    );
                                },
                            }
                            button {
                                r#type: "button",
                                class: if case_sensitive() {
                                    "ac-search-toggle is-active"
                                } else {
                                    "ac-search-toggle"
                                },
                                title: "区分大小写",
                                aria_label: "区分大小写",
                                aria_pressed: case_sensitive(),
                                onclick: move |_| {
                                    let next = !case_sensitive();
                                    case_sensitive.set(next);
                                    if !query().trim().is_empty() {
                                        kick_search(
                                            query(),
                                            next,
                                            whole_word(),
                                            use_regex(),
                                            include_glob(),
                                            exclude_glob(),
                                            0,
                                            search_epoch,
                                            loading,
                                            outcome,
                                            expanded,
                                        );
                                    }
                                },
                                Icon { icon: LdCaseSensitive, width: 13, height: 13, fill: "currentColor" }
                            }
                            button {
                                r#type: "button",
                                class: if whole_word() {
                                    "ac-search-toggle is-active"
                                } else {
                                    "ac-search-toggle"
                                },
                                title: "全字匹配",
                                aria_label: "全字匹配",
                                aria_pressed: whole_word(),
                                onclick: move |_| {
                                    let next = !whole_word();
                                    whole_word.set(next);
                                    if !query().trim().is_empty() {
                                        kick_search(
                                            query(),
                                            case_sensitive(),
                                            next,
                                            use_regex(),
                                            include_glob(),
                                            exclude_glob(),
                                            0,
                                            search_epoch,
                                            loading,
                                            outcome,
                                            expanded,
                                        );
                                    }
                                },
                                Icon { icon: LdWholeWord, width: 13, height: 13, fill: "currentColor" }
                            }
                            button {
                                r#type: "button",
                                class: if use_regex() {
                                    "ac-search-toggle is-active"
                                } else {
                                    "ac-search-toggle"
                                },
                                title: "正则表达式",
                                aria_label: "正则表达式",
                                aria_pressed: use_regex(),
                                onclick: move |_| {
                                    let next = !use_regex();
                                    use_regex.set(next);
                                    if !query().trim().is_empty() {
                                        kick_search(
                                            query(),
                                            case_sensitive(),
                                            whole_word(),
                                            next,
                                            include_glob(),
                                            exclude_glob(),
                                            0,
                                            search_epoch,
                                            loading,
                                            outcome,
                                            expanded,
                                        );
                                    }
                                },
                                Icon { icon: LdRegex, width: 13, height: 13, fill: "currentColor" }
                            }
                        }
                        if replace_open() {
                            div { class: "ac-search-input-row",
                                input {
                                    r#type: "text",
                                    class: "ac-search-input",
                                    value: "{replace_text}",
                                    placeholder: "替换",
                                    spellcheck: false,
                                    oninput: move |e| replace_text.set(e.value()),
                                }
                                button {
                                    r#type: "button",
                                    class: if preserve_case() {
                                        "ac-search-toggle is-active"
                                    } else {
                                        "ac-search-toggle"
                                    },
                                    title: "保留大小写",
                                    aria_label: "保留大小写",
                                    aria_pressed: preserve_case(),
                                    onclick: move |_| {
                                        preserve_case.with_mut(|v| *v = !*v);
                                    },
                                    Icon { icon: LdCaseUpper, width: 13, height: 13, fill: "currentColor" }
                                }
                                button {
                                    r#type: "button",
                                    class: "ac-search-toggle ac-search-replace-all",
                                    title: "全部替换",
                                    aria_label: "全部替换",
                                    disabled: replacing() || match_count == 0 || query().trim().is_empty(),
                                    onclick: move |_| {
                                        if replacing() {
                                            return;
                                        }
                                        let req = SearchRequest {
                                            root: super::files::fs_workspace_root(),
                                            query: query(),
                                            case_sensitive: case_sensitive(),
                                            whole_word: whole_word(),
                                            regex: use_regex(),
                                            include: include_glob(),
                                            exclude: exclude_glob(),
                                        };
                                        let replacement = replace_text();
                                        let keep_case = preserve_case();
                                        let files = outcome().files;
                                        if files.is_empty() {
                                            toast.warning("没有可替换的结果。");
                                            return;
                                        }
                                        let dirty_now = dirty();
                                        let skipped = files
                                            .iter()
                                            .filter(|f| dirty_now.contains(&f.path))
                                            .count();
                                        let writable: Vec<String> = files
                                            .iter()
                                            .filter(|f| !dirty_now.contains(&f.path))
                                            .map(|f| f.path.clone())
                                            .collect();
                                        if writable.is_empty() {
                                            toast.warning(
                                                "当前匹配的文件都有未保存修改，请先保存或撤销后再替换。",
                                            );
                                            return;
                                        }
                                        if !super::files::fs_confirm_replace_all(
                                            writable.len(),
                                            match_count,
                                            skipped,
                                        ) {
                                            return;
                                        }
                                        replacing.set(true);

                                        spawn(async move {
                                            let result = run_replace_all(
                                                req,
                                                replacement,
                                                keep_case,
                                                writable,
                                            )
                                            .await;
                                            match result {
                                                Ok(done) => {
                                                    for (path, content) in &done.written {
                                                        let path = path.clone();
                                                        let content = content.clone();
                                                        drafts.with_mut(|m| {
                                                            m.insert(path.clone(), content.clone());
                                                        });
                                                        baselines.with_mut(|m| {
                                                            m.insert(path, content);
                                                        });
                                                    }
                                                    let mut msg = format!(
                                                        "已替换 {} 处，写入 {} 个文件",
                                                        done.replaced,
                                                        done.written.len()
                                                    );
                                                    if skipped > 0 {
                                                        msg.push_str(&format!(
                                                            "，跳过 {skipped} 个未保存文件"
                                                        ));
                                                    }
                                                    if !done.errors.is_empty() {
                                                        msg.push_str(&format!(
                                                            "，{} 个失败",
                                                            done.errors.len()
                                                        ));
                                                        toast.warning(msg.clone());
                                                    } else {
                                                        toast.success(msg.clone());
                                                    }
                                                }
                                                Err(e) => {
                                                    toast.error(e.clone());
                                                }
                                            }
                                            replacing.set(false);
                                            kick_search(
                                                query(),
                                                case_sensitive(),
                                                whole_word(),
                                                use_regex(),
                                                include_glob(),
                                                exclude_glob(),
                                                0,
                                                search_epoch,
                                                loading,
                                                outcome,
                                                expanded,
                                            );
                                        });
                                    },
                                    Icon { icon: LdReplaceAll, width: 13, height: 13, fill: "currentColor" }
                                }
                            }
                        }
                    }
                }

                div { class: "ac-search-pair ac-search-pair--filters",
                    button {
                        r#type: "button",
                        class: "ac-search-pair-toggle",
                        title: if exclude_open() { "收起排除文件" } else { "展开排除文件" },
                        aria_label: if exclude_open() { "收起排除文件" } else { "展开排除文件" },
                        aria_expanded: exclude_open(),
                        onclick: move |_| {
                            exclude_open.with_mut(|v| *v = !*v);
                        },
                        if exclude_open() {
                            Icon { icon: LdChevronDown, width: 14, height: 14, fill: "currentColor" }
                        } else {
                            Icon { icon: LdChevronRight, width: 14, height: 14, fill: "currentColor" }
                        }
                    }
                    div { class: "ac-search-pair-inputs",
                        label { class: "ac-search-filter",
                            span { class: "ac-search-filter-label", "包含文件" }
                            input {
                                r#type: "text",
                                class: "ac-search-filter-input",
                                value: "{include_glob}",
                                placeholder: "例如：*.rs, src/**/*.ts",
                                spellcheck: false,
                                oninput: move |e| {
                                    include_glob.set(e.value());
                                    if query().trim().is_empty() {
                                        return;
                                    }
                                    kick_search(
                                        query(),
                                        case_sensitive(),
                                        whole_word(),
                                        use_regex(),
                                        e.value(),
                                        exclude_glob(),
                                        SEARCH_DEBOUNCE_MS,
                                        search_epoch,
                                        loading,
                                        outcome,
                                        expanded,
                                    );
                                },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() != Key::Enter {
                                        return;
                                    }
                                    e.prevent_default();
                                    kick_search(
                                        query(),
                                        case_sensitive(),
                                        whole_word(),
                                        use_regex(),
                                        include_glob(),
                                        exclude_glob(),
                                        0,
                                        search_epoch,
                                        loading,
                                        outcome,
                                        expanded,
                                    );
                                },
                            }
                        }
                        if exclude_open() {
                            label { class: "ac-search-filter",
                                span { class: "ac-search-filter-label", "排除文件" }
                                input {
                                    r#type: "text",
                                    class: "ac-search-filter-input",
                                    value: "{exclude_glob}",
                                    placeholder: "例如：*.lock, dist",
                                    spellcheck: false,
                                    oninput: move |e| {
                                        exclude_glob.set(e.value());
                                        if query().trim().is_empty() {
                                            return;
                                        }
                                        kick_search(
                                            query(),
                                            case_sensitive(),
                                            whole_word(),
                                            use_regex(),
                                            include_glob(),
                                            e.value(),
                                            SEARCH_DEBOUNCE_MS,
                                            search_epoch,
                                            loading,
                                            outcome,
                                            expanded,
                                        );
                                    },
                                    onkeydown: move |e: KeyboardEvent| {
                                        if e.key() != Key::Enter {
                                            return;
                                        }
                                        e.prevent_default();
                                        kick_search(
                                            query(),
                                            case_sensitive(),
                                            whole_word(),
                                            use_regex(),
                                            include_glob(),
                                            exclude_glob(),
                                            0,
                                            search_epoch,
                                            loading,
                                            outcome,
                                            expanded,
                                        );
                                    },
                                }
                            }
                        }
                    }
                }
            }

            div { class: "ac-sidebar-search-summary", "{summary}" }

            div { class: "ac-sidebar-search-results", role: "tree",
                onscroll: move |e| {
                    scroll_top.set(e.data().scroll_top());
                },
                if loading() && current.files.is_empty() {
                    div { class: "ac-sidebar-files-empty",
                        Icon { icon: LdSearch, width: 16, height: 16, fill: "currentColor" }
                        "正在搜索工作区…"
                    }
                } else if !loading() && current.files.is_empty() && has_query {
                    div { class: "ac-sidebar-files-empty",
                        Icon { icon: LdSearch, width: 16, height: 16, fill: "currentColor" }
                        if let Some(err) = current.error.as_ref() {
                            "{err}"
                        } else {
                            "未找到匹配"
                        }
                    }
                }
                if top_pad_px > 0.0 {
                    div { style: "height: {top_pad_px}px; flex: none;", aria_hidden: "true" }
                }
                for group in current.files[view_start..view_end].iter() {
                    {
                        let path = group.path.clone();
                        let path_open = path.clone();
                        let path_toggle = path.clone();
                        let rel = group.rel.clone();
                        let open = expanded_now.open.contains(&path);
                        let hits = group.hits.clone();
                        let hit_n = hits.len();
                        rsx! {
                            div { class: "ac-search-file", key: "{path}",
                                button {
                                    r#type: "button",
                                    class: "ac-search-file-head",
                                    title: "{rel}",
                                    onclick: move |_| {
                                        let p = path_toggle.clone();
                                        expanded.with_mut(|state| {
                                            if state.open.remove(&p) {
                                                state.user_collapsed.insert(p);
                                            } else {
                                                state.open.insert(p.clone());
                                                state.user_collapsed.remove(&p);
                                            }
                                        });
                                    },
                                    ondoubleclick: move |_| {
                                        on_open_file.call(path_open.clone());
                                    },
                                    span { class: "ac-sidebar-files-row-chevron",
                                        if open {
                                            Icon { icon: LdChevronDown, width: 12, height: 12, fill: "currentColor" }
                                        } else {
                                            Icon { icon: LdChevronRight, width: 12, height: 12, fill: "currentColor" }
                                        }
                                    }
                                    span { class: "ac-sidebar-files-row-icon",
                                        Icon { icon: LdFile, width: 13, height: 13, fill: "currentColor" }
                                    }
                                    span { class: "ac-search-file-name", "{rel}" }
                                    span { class: "ac-search-file-count", "{hit_n}" }
                                }
                                if open {
                                    for (idx, hit) in hits.iter().enumerate() {
                                        {
                                            let path = path.clone();
                                            let line = hit.line.clone();
                                            let line_no = hit.line_no;
                                            let start = hit.start;
                                            let end = hit.end;
                                            rsx! {
                                                button {
                                                    r#type: "button",
                                                    class: "ac-search-hit",
                                                    key: "{path}:{line_no}:{idx}",
                                                    title: "{rel}:{line_no}",
                                                    onclick: move |_| {
                                                        on_open_file.call(path.clone());
                                                    },
                                                    span { class: "ac-search-hit-ln", "{line_no}" }
                                                    span { class: "ac-search-hit-text",
                                                        {highlight_line(&line, start, end)}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if bottom_pad_px > 0.0 {
                    div { style: "height: {bottom_pad_px}px; flex: none;", aria_hidden: "true" }
                }
            }
        }
    }
}

/// 流式追加结果时，自动展开前 N 个文件；用户手动折叠过的不动。
fn auto_expand_first(files: &[SearchFileHits], mut expanded: Signal<ExpandState>) {
    let mut state = expanded();
    let mut changed = false;
    for f in files.iter().take(AUTO_EXPAND_FILES) {
        if !state.open.contains(&f.path) && !state.user_collapsed.contains(&f.path) {
            state.open.insert(f.path.clone());
            changed = true;
        }
    }
    if changed {
        expanded.set(state);
    }
}

fn highlight_line(line: &str, start: usize, end: usize) -> Element {
    let valid = start <= end
        && end <= line.len()
        && line.is_char_boundary(start)
        && line.is_char_boundary(end);
    if !valid {
        return rsx! { "{line}" };
    }
    let before = &line[..start];
    let mid = &line[start..end];
    let after = &line[end..];
    rsx! {
        "{before}"
        span { class: "ac-search-hit-mark", "{mid}" }
        "{after}"
    }
}

fn kick_search(
    query: String,
    case_sensitive: bool,
    whole_word: bool,
    regex: bool,
    include: String,
    exclude: String,
    delay_ms: u64,
    mut epoch: Signal<u64>,
    mut loading: Signal<bool>,
    mut outcome: Signal<SearchOutcome>,
    mut expanded: Signal<ExpandState>,
) {
    epoch.with_mut(|n| *n += 1);
    let this_epoch = epoch();
    let root = super::files::fs_workspace_root();
    let trimmed = query.trim().to_string();
    if trimmed.is_empty() {
        loading.set(false);
        outcome.set(SearchOutcome::default());
        expanded.set(ExpandState::default());
        return;
    }
    let req = SearchRequest {
        root,
        query: trimmed,
        case_sensitive,
        whole_word,
        regex,
        include,
        exclude,
    };
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    {
        // 注意：不清空旧 outcome，输入期间保留上一次结果，流式结果到达后增量替换。
        loading.set(true);
        spawn(async move {
            if delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }
            if epoch() != this_epoch {
                return;
            }
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SearchEvent>();
            let req_thread = req.clone();
            if std::thread::Builder::new()
                .name("pusa-sidebar-search".into())
                .spawn(move || run_search_stream(req_thread, tx))
                .is_err()
            {
                outcome.set(SearchOutcome {
                    error: Some("无法启动搜索线程。".into()),
                    ..SearchOutcome::default()
                });
                loading.set(false);
                return;
            }

            let deadline =
                tokio::time::Instant::now() + std::time::Duration::from_secs(SEARCH_TIMEOUT_SECS);
            // 旧结果保留到首批新结果到达；随后按新查询重新计算默认展开项。
            let mut first_result = true;
            loop {
                let ev = match tokio::time::timeout_at(deadline, rx.recv()).await {
                    Ok(Some(ev)) => ev,
                    Ok(None) => break, // 线程结束且未发 Done（异常），由下方兜底
                    Err(_) => {
                        outcome.set(SearchOutcome {
                            error: Some(format!(
                                "搜索超时（>{SEARCH_TIMEOUT_SECS}s）。可缩小范围，或填写「包含文件」如 *.rs。"
                            )),
                            ..SearchOutcome::default()
                        });
                        loading.set(false);
                        return;
                    }
                };
                // 已被更新的查询取代：丢弃本批，不再触碰状态。
                if epoch() != this_epoch {
                    return;
                }
                match ev {
                    SearchEvent::Progress(out) => {
                        if first_result {
                            expanded.set(ExpandState::default());
                            first_result = false;
                        }
                        auto_expand_first(&out.files, expanded);
                        outcome.set(out);
                    }
                    SearchEvent::Done(out) => {
                        if first_result {
                            expanded.set(ExpandState::default());
                        }
                        auto_expand_first(&out.files, expanded);
                        outcome.set(out);
                        loading.set(false);
                        break;
                    }
                    SearchEvent::Error(e) => {
                        outcome.set(SearchOutcome {
                            error: Some(e),
                            ..SearchOutcome::default()
                        });
                        loading.set(false);
                        break;
                    }
                }
            }
            // channel 意外关闭（线程 panic 等）时兜底，避免永远停在「正在搜索…」。
            if epoch() == this_epoch && loading() {
                loading.set(false);
            }
        });
    }
    #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
    {
        let _ = (delay_ms, req, this_epoch, loading, outcome, expanded, epoch);
    }
}

/// 在独立线程跑搜索：优先 ripgrep 流式读取，边扫边通过 channel 推送快照；
/// 无 rg 且非正则时回退到 walk_search（一次性发 Done）。超时由接收侧控制。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn run_search_stream(req: SearchRequest, tx: tokio::sync::mpsc::UnboundedSender<SearchEvent>) {
    if req.root.trim().is_empty() {
        let _ = tx.send(SearchEvent::Error("未打开工作区。".into()));
        return;
    }
    match try_rg_stream(&req, &tx) {
        Ok(Some(())) => {}
        Ok(None) => {
            if req.regex {
                let _ = tx.send(SearchEvent::Error(
                    "正则搜索需要系统已安装 ripgrep（rg）。".into(),
                ));
            } else {
                let out = walk_search(&req);
                let _ = tx.send(SearchEvent::Done(out));
            }
        }
        Err(e) => {
            let _ = tx.send(SearchEvent::Error(e));
        }
    }
}

struct ReplaceDone {
    replaced: usize,
    written: Vec<(String, String)>,
    errors: Vec<String>,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
async fn run_replace_all(
    req: SearchRequest,
    replacement: String,
    preserve_case: bool,
    writable: Vec<String>,
) -> Result<ReplaceDone, String> {
    tokio::task::spawn_blocking(move || {
        replace_in_files(&req, &replacement, preserve_case, &writable)
    })
    .await
    .map_err(|e| format!("替换任务失败：{e}"))?
}

#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
async fn run_replace_all(
    _req: SearchRequest,
    _replacement: String,
    _preserve_case: bool,
    _writable: Vec<String>,
) -> Result<ReplaceDone, String> {
    Err("Web 端无法搜索本机文件".into())
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn replace_in_files(
    req: &SearchRequest,
    replacement: &str,
    preserve_case: bool,
    writable: &[String],
) -> Result<ReplaceDone, String> {
    let mut done = ReplaceDone {
        replaced: 0,
        written: Vec::new(),
        errors: Vec::new(),
    };
    for path in writable {
        match super::files::fs_read_text(path) {
            Ok(old) => match replace_text(&old, req, replacement, preserve_case, path) {
                Ok((new, n)) => {
                    if n == 0 || new == old {
                        continue;
                    }
                    if new.is_empty() && !old.is_empty() {
                        done.errors.push(format!("{path}：替换结果为空，已跳过。"));
                        continue;
                    }
                    match super::files::fs_write_text(path, &new) {
                        Ok(()) => {
                            done.replaced += n;
                            done.written.push((path.clone(), new));
                        }
                        Err(e) => done.errors.push(format!("{path}：{e}")),
                    }
                }
                Err(e) => done.errors.push(format!("{path}：{e}")),
            },
            Err(e) => done.errors.push(format!("{path}：{e}")),
        }
    }
    Ok(done)
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn replace_text(
    text: &str,
    req: &SearchRequest,
    replacement: &str,
    preserve_case: bool,
    path: &str,
) -> Result<(String, usize), String> {
    if req.regex {
        return replace_text_regex(path, req, replacement);
    }
    let spans = find_literal_spans(text, &req.query, req.case_sensitive, req.whole_word);
    if spans.is_empty() {
        return Ok((text.to_string(), 0));
    }
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for (start, end) in &spans {
        if *start < last || *end > text.len() || *start > *end {
            continue;
        }
        if !text.is_char_boundary(*start) || !text.is_char_boundary(*end) {
            continue;
        }
        out.push_str(&text[last..*start]);
        let matched = &text[*start..*end];
        if preserve_case {
            out.push_str(&apply_preserve_case(matched, replacement));
        } else {
            out.push_str(replacement);
        }
        last = *end;
    }
    out.push_str(&text[last..]);
    Ok((out, spans.len()))
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn replace_text_regex(
    path: &str,
    req: &SearchRequest,
    replacement: &str,
) -> Result<(String, usize), String> {
    use std::process::Stdio;

    let mut count_cmd = crate::desktop::files::hidden_command("rg");
    count_cmd
        .arg("--count-matches")
        .arg("--max-filesize")
        .arg("2M")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if !req.case_sensitive {
        count_cmd.arg("-i");
    }
    if req.whole_word {
        count_cmd.arg("-w");
    }
    count_cmd.arg("--").arg(&req.query).arg(path);
    let count_out = match count_cmd.output() {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err("正则全部替换需要系统已安装 ripgrep (rg)。".into());
        }
        Err(e) => return Err(format!("启动 ripgrep 失败：{e}")),
        Ok(out) => out,
    };
    let count = String::from_utf8_lossy(&count_out.stdout)
        .rsplit(':')
        .next()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);
    if count == 0 {
        return Ok((String::new(), 0));
    }

    let mut cmd = crate::desktop::files::hidden_command("rg");
    cmd.arg("--replace")
        .arg(replacement)
        .arg("--passthrough")
        .arg("-N")
        .arg("--max-filesize")
        .arg("2M")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if !req.case_sensitive {
        cmd.arg("-i");
    }
    if req.whole_word {
        cmd.arg("-w");
    }
    cmd.arg("--").arg(&req.query).arg(path);
    let output = cmd.output().map_err(|e| format!("ripgrep 替换失败：{e}"))?;
    if !output.status.success() && output.status.code() != Some(0) {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !err.is_empty() {
            return Err(err);
        }
    }
    let new =
        String::from_utf8(output.stdout).map_err(|_| "替换结果不是有效 UTF-8。".to_string())?;
    Ok((new, count))
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn resolve_rg_bin() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    if let Ok(path) = which_rg_on_path() {
        return Some(path);
    }
    // Dock / Finder 启动时 PATH 往往很短，补常见安装位置。
    const CANDIDATES: &[&str] = &[
        "/opt/homebrew/bin/rg",
        "/usr/local/bin/rg",
        "/usr/bin/rg",
        "/opt/local/bin/rg",
    ];
    for c in CANDIDATES {
        let p = PathBuf::from(c);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn which_rg_on_path() -> Result<std::path::PathBuf, ()> {
    use std::path::PathBuf;
    let mut cmd = crate::desktop::files::hidden_command("rg");
    cmd.arg("--version");
    match cmd.output() {
        Ok(out) if out.status.success() => Ok(PathBuf::from("rg")),
        _ => Err(()),
    }
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn try_rg_stream(
    req: &SearchRequest,
    tx: &tokio::sync::mpsc::UnboundedSender<SearchEvent>,
) -> Result<Option<()>, String> {
    use std::io::{BufRead, BufReader, Read};
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let Some(rg) = resolve_rg_bin() else {
        return Ok(None);
    };

    let mut cmd = crate::desktop::files::hidden_command(&rg);
    cmd.arg("--json")
        .arg("--max-filesize")
        .arg("2M")
        .arg("--max-columns")
        .arg("400")
        .arg("--max-columns-preview")
        .current_dir(&req.root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.arg("--glob").arg("!**/.git/**");
    cmd.arg("--glob").arg("!**/node_modules/**");
    cmd.arg("--glob").arg("!**/target/**");
    cmd.arg("--glob").arg("!**/dist/**");
    cmd.arg("--glob").arg("!**/.venv/**");
    cmd.arg("--glob").arg("!**/venv/**");
    for g in split_globs(&req.include) {
        cmd.arg("--glob").arg(g);
    }
    for g in split_globs(&req.exclude) {
        cmd.arg("--glob").arg(format!("!{g}"));
    }
    if !req.case_sensitive {
        cmd.arg("-i");
    }
    if req.whole_word {
        cmd.arg("-w");
    }
    if !req.regex {
        cmd.arg("-F");
    }
    cmd.arg("--").arg(&req.query).arg(".");

    let mut child = match cmd.spawn() {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("启动 ripgrep 失败：{e}")),
        Ok(c) => c,
    };
    let Some(stdout) = child.stdout.take() else {
        return Err("无法读取 ripgrep 输出。".into());
    };

    // 流式读取：每行立即解析累积，按节奏推送快照到 UI。
    let mut acc = RgAccumulator::new(Path::new(&req.root));
    let mut last_flush = Instant::now();
    let mut since_flush = 0usize;
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if acc.push_line(&line) {
            since_flush += 1;
            if since_flush >= STREAM_FLUSH_MATCHES
                || last_flush.elapsed() >= Duration::from_millis(STREAM_FLUSH_MS)
            {
                if tx.send(SearchEvent::Progress(acc.snapshot())).is_err() {
                    // 接收端已取消（新查询取代），杀掉 rg 尽早退出。
                    let _ = child.kill();
                    let _ = child.wait();
                    return Ok(Some(()));
                }
                last_flush = Instant::now();
                since_flush = 0;
            }
        }
        if acc.truncated {
            let _ = child.kill();
            break;
        }
    }

    let status = child.wait();
    let mut err_text = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        let _ = stderr.read_to_string(&mut err_text);
    }
    match status {
        Ok(s) if !s.success() && s.code() != Some(1) && !acc.truncated => {
            let err = err_text.trim().to_string();
            return Err(if err.is_empty() {
                "ripgrep 搜索失败。".into()
            } else {
                err
            });
        }
        Err(_) if !acc.truncated => {
            return Err("ripgrep 搜索中断。".into());
        }
        _ => {}
    }

    let _ = tx.send(SearchEvent::Done(acc.snapshot()));
    Ok(Some(()))
}

/// rg `--json` 行格式（强类型，替代 serde_json::Value 动态解析）。
/// 非 match 类型（begin/end/summary）的 data 字段也能解析：字段全为 Option/default。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[derive(serde::Deserialize)]
struct RgLine {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    data: Option<RgMatchData>,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[derive(serde::Deserialize)]
struct RgMatchData {
    #[serde(default)]
    path: Option<RgText>,
    #[serde(default)]
    lines: Option<RgText>,
    #[serde(default)]
    line_number: Option<u64>,
    #[serde(default)]
    submatches: Vec<RgSubmatch>,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[derive(serde::Deserialize)]
struct RgText {
    #[serde(default)]
    text: Option<String>,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[derive(serde::Deserialize)]
struct RgSubmatch {
    #[serde(default)]
    start: Option<usize>,
    #[serde(default)]
    end: Option<usize>,
}

/// 流式累积器：逐行吞 rg JSON，随时可对当前进度拍快照。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
struct RgAccumulator {
    root: std::path::PathBuf,
    files: Vec<SearchFileHits>,
    index: HashMap<String, usize>,
    match_count: usize,
    truncated: bool,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
impl RgAccumulator {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            files: Vec::new(),
            index: HashMap::new(),
            match_count: 0,
            truncated: false,
        }
    }

    /// 解析一行，返回是否为有效 match（用于推送节奏计数）。
    fn push_line(&mut self, line: &str) -> bool {
        if self.truncated {
            return false;
        }
        let Ok(msg) = serde_json::from_str::<RgLine>(line) else {
            return false;
        };
        if msg.kind != "match" {
            return false;
        }
        let Some(data) = msg.data else { return false };
        let Some(path) = data.path.and_then(|p| p.text) else {
            return false;
        };
        if path.is_empty() {
            return false;
        }
        let abs = {
            let p = Path::new(&path);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                self.root.join(p)
            }
        };
        let abs_s = abs.to_string_lossy().into_owned();
        let rel = rel_path(&self.root, &abs);
        let line_no = data.line_number.unwrap_or(0) as u32;
        let raw_line = data.lines.and_then(|l| l.text).unwrap_or_default();
        let raw_line = raw_line.trim_end_matches(['\n', '\r']);
        let (start, end) = data
            .submatches
            .first()
            .map(|sub| (sub.start.unwrap_or(0), sub.end.unwrap_or(0)))
            .unwrap_or((0, 0));
        let (line, hs, he) = truncate_hit_line(raw_line, start, end);
        let idx = match self.index.get(&abs_s).copied() {
            Some(i) => i,
            None => {
                if self.files.len() >= MAX_FILES {
                    self.truncated = true;
                    return true;
                }
                let i = self.files.len();
                self.index.insert(abs_s.clone(), i);
                self.files.push(SearchFileHits {
                    path: abs_s.clone(),
                    rel,
                    hits: Vec::new(),
                });
                i
            }
        };
        self.files[idx].hits.push(SearchHit {
            line_no,
            line,
            start: hs,
            end: he,
        });
        self.match_count += 1;
        if self.match_count >= MAX_MATCHES {
            self.truncated = true;
        }
        true
    }

    fn snapshot(&self) -> SearchOutcome {
        SearchOutcome {
            files: self.files.clone(),
            match_count: self.match_count,
            truncated: self.truncated,
            error: None,
        }
    }
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn walk_search(req: &SearchRequest) -> SearchOutcome {
    let root = Path::new(&req.root);
    if !root.is_dir() {
        return SearchOutcome {
            error: Some("工作区目录不存在。".into()),
            ..SearchOutcome::default()
        };
    }
    let include = split_globs(&req.include);
    let exclude = split_globs(&req.exclude);
    let mut files: Vec<SearchFileHits> = Vec::new();
    let mut match_count = 0usize;
    let mut truncated = false;
    let mut scanned = 0usize;
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        if truncated {
            break;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut kids: Vec<_> = entries.flatten().collect();
        kids.sort_by_key(|e| e.file_name());
        for entry in kids {
            if truncated {
                break;
            }
            let path = entry.path();
            let name = entry.file_name();
            let name_s = name.to_string_lossy();
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                if SKIP_DIR_NAMES
                    .iter()
                    .any(|n| name_s.eq_ignore_ascii_case(n))
                {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if !ft.is_file() {
                continue;
            }
            scanned += 1;
            if scanned > MAX_FILES_SCANNED {
                truncated = true;
                break;
            }
            let rel = rel_path(root, &path);
            if !include.is_empty() && !include.iter().any(|g| glob_match(g, &rel)) {
                continue;
            }
            if exclude.iter().any(|g| glob_match(g, &rel)) {
                continue;
            }
            let Some(text) = read_searchable_text(&path) else {
                continue;
            };
            let mut hits = Vec::new();
            for (i, raw) in text.lines().enumerate() {
                let line = raw.trim_end_matches('\r');
                for (start, end) in
                    find_literal_spans(line, &req.query, req.case_sensitive, req.whole_word)
                {
                    let (shown, hs, he) = truncate_hit_line(line, start, end);
                    hits.push(SearchHit {
                        line_no: (i + 1) as u32,
                        line: shown,
                        start: hs,
                        end: he,
                    });
                    match_count += 1;
                    if match_count >= MAX_MATCHES {
                        truncated = true;
                        break;
                    }
                }
                if truncated {
                    break;
                }
            }
            if hits.is_empty() {
                continue;
            }
            if files.len() >= MAX_FILES {
                truncated = true;
                break;
            }
            files.push(SearchFileHits {
                path: path.to_string_lossy().into_owned(),
                rel,
                hits,
            });
        }
    }

    SearchOutcome {
        files,
        match_count,
        truncated,
        error: None,
    }
}

fn split_globs(raw: &str) -> Vec<String> {
    raw.split([',', ';', '\n'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn glob_match(pat: &str, rel: &str) -> bool {
    let pat = pat.replace('\\', "/");
    let rel = rel.replace('\\', "/");
    let pat = pat.trim_start_matches('/');
    if !pat.contains('/') {
        glob_match_full(&format!("**/{pat}"), &rel) || glob_match_full(pat, &rel)
    } else {
        glob_match_full(pat, &rel)
    }
}

fn glob_match_full(pat: &str, text: &str) -> bool {
    glob_at(pat.as_bytes(), 0, text.as_bytes(), 0)
}

fn glob_at(pat: &[u8], pi: usize, text: &[u8], ti: usize) -> bool {
    if pi == pat.len() {
        return ti == text.len();
    }
    match pat[pi] {
        b'*' => {
            if pi + 1 < pat.len() && pat[pi + 1] == b'*' {
                let mut npi = pi + 2;
                if npi < pat.len() && pat[npi] == b'/' {
                    npi += 1;
                }
                if glob_at(pat, npi, text, ti) {
                    return true;
                }
                for k in ti..text.len() {
                    if glob_at(pat, npi, text, k + 1) {
                        return true;
                    }
                }
                false
            } else {
                if glob_at(pat, pi + 1, text, ti) {
                    return true;
                }
                let mut k = ti;
                while k < text.len() && text[k] != b'/' {
                    if glob_at(pat, pi + 1, text, k + 1) {
                        return true;
                    }
                    k += 1;
                }
                false
            }
        }
        b'?' => {
            if ti >= text.len() || text[ti] == b'/' {
                return false;
            }
            glob_at(pat, pi + 1, text, ti + 1)
        }
        ch => {
            if ti >= text.len() {
                return false;
            }
            let same = text[ti] == ch || ch.eq_ignore_ascii_case(&text[ti]);
            same && glob_at(pat, pi + 1, text, ti + 1)
        }
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_whole_word(hay: &str, start: usize, end: usize) -> bool {
    if start > hay.len() || end > hay.len() || start > end {
        return false;
    }
    if !hay.is_char_boundary(start) || !hay.is_char_boundary(end) {
        return false;
    }
    let before_ok = start == 0
        || hay[..start]
            .chars()
            .next_back()
            .map(|c| !is_word_char(c))
            .unwrap_or(true);
    let after_ok = end >= hay.len()
        || hay[end..]
            .chars()
            .next()
            .map(|c| !is_word_char(c))
            .unwrap_or(true);
    before_ok && after_ok
}

fn find_literal_spans(
    hay: &str,
    needle: &str,
    case_sensitive: bool,
    whole_word: bool,
) -> Vec<(usize, usize)> {
    if needle.is_empty() {
        return Vec::new();
    }
    if case_sensitive {
        let mut out = Vec::new();
        let mut from = 0;
        while from <= hay.len() {
            let Some(pos) = hay[from..].find(needle) else {
                break;
            };
            let start = from + pos;
            let end = start + needle.len();
            if !whole_word || is_whole_word(hay, start, end) {
                out.push((start, end));
            }
            from = if end > start { end } else { start + 1 };
            while from < hay.len() && !hay.is_char_boundary(from) {
                from += 1;
            }
            if from >= hay.len() {
                break;
            }
        }
        return out;
    }
    let needle_l = needle.to_lowercase();
    let nlen = needle_l.chars().count();
    if nlen == 0 {
        return Vec::new();
    }
    let chars: Vec<(usize, char)> = hay.char_indices().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + nlen <= chars.len() {
        let slice: String = chars[i..i + nlen].iter().map(|(_, c)| *c).collect();
        if slice.to_lowercase() == needle_l {
            let start = chars[i].0;
            let end = if i + nlen < chars.len() {
                chars[i + nlen].0
            } else {
                hay.len()
            };
            if !whole_word || is_whole_word(hay, start, end) {
                out.push((start, end));
            }
            i += nlen;
        } else {
            i += 1;
        }
    }
    out
}

fn apply_preserve_case(matched: &str, replacement: &str) -> String {
    if replacement.is_empty() || matched.is_empty() {
        return replacement.to_string();
    }
    let letters: Vec<char> = matched.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return replacement.to_string();
    }
    if letters.iter().all(|c| c.is_uppercase()) {
        return replacement.to_uppercase();
    }
    if letters.iter().all(|c| c.is_lowercase()) {
        return replacement.to_lowercase();
    }
    let Some(first) = matched.chars().find(|c| c.is_alphabetic()) else {
        return replacement.to_string();
    };
    let rest_lower = matched
        .chars()
        .skip_while(|c| !c.is_alphabetic())
        .skip(1)
        .filter(|c| c.is_alphabetic())
        .all(|c| c.is_lowercase());
    if first.is_uppercase() && rest_lower {
        let mut chars = replacement.chars();
        let Some(r0) = chars.next() else {
            return replacement.to_string();
        };
        let mut out = String::new();
        out.extend(r0.to_uppercase());
        out.extend(chars);
        return out;
    }
    replacement.to_string()
}

fn truncate_hit_line(line: &str, start: usize, end: usize) -> (String, usize, usize) {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    if chars.len() <= MAX_LINE_CHARS {
        let s = start.min(line.len());
        let e = end.min(line.len()).max(s);
        return (line.to_string(), s, e);
    }
    let match_char = chars.iter().position(|(off, _)| *off >= start).unwrap_or(0);
    let window = MAX_LINE_CHARS;
    let from = match_char.saturating_sub(window / 4);
    let to = (from + window).min(chars.len());
    let from = to.saturating_sub(window);
    let byte_from = chars[from].0;
    let byte_to = if to < chars.len() {
        chars[to].0
    } else {
        line.len()
    };
    let mut shown = String::new();
    if from > 0 {
        shown.push('…');
    }
    shown.push_str(&line[byte_from..byte_to]);
    if to < chars.len() {
        shown.push('…');
    }
    let prefix = if from > 0 { 1 } else { 0 };
    let hs = if start >= byte_from {
        (start - byte_from) + prefix
    } else {
        prefix
    };
    let he = if end >= byte_from {
        ((end.min(byte_to)) - byte_from) + prefix
    } else {
        prefix
    };
    let hs = hs.min(shown.len());
    let he = he.min(shown.len()).max(hs);
    (shown, hs, he)
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn read_searchable_text(path: &Path) -> Option<String> {
    if !crate::desktop::files::is_probably_text(path) {
        return None;
    }
    super::files::fs_read_text(&path.to_string_lossy()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_star_matches_nested_rs() {
        assert!(glob_match("*.rs", "src/shell/search.rs"));
        assert!(glob_match("src/**/*.rs", "src/shell/search.rs"));
        assert!(!glob_match("*.toml", "src/shell/search.rs"));
    }

    #[test]
    fn whole_word_skips_partial() {
        let hits = find_literal_spans("searching search", "search", true, true);
        assert_eq!(hits, vec![(10, 16)]);
    }
}
