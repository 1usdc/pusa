//! 聊天区 Agent 思考 / 工具步骤（Cursor 式：步骤时间线在上，回答在下）。
use crate::shell::style::{
    CHAT_SUBTASK_BADGE,
    CHAT_SUBTASK_CARD,
    CHAT_SUBTASK_HEAD,
    CHAT_SUBTASK_OPEN,
    CHAT_SUBTASK_STATUS,
    CHAT_SUBTASK_SUMMARY,
    CHAT_SUBTASK_TEXT,
    CHAT_SUBTASK_TITLE,
    CHAT_TRACE,
    CHAT_TRACE_ACTIVITY,
    CHAT_TRACE_ACTIVITY_LABEL,
    CHAT_TRACE_BODY,
    CHAT_TRACE_CHEVRON,
    CHAT_TRACE_LIST,
    CHAT_TRACE_MINI_CHEVRON,
    CHAT_TRACE_PRIMARY,
    CHAT_TRACE_PRIMARY_LINK,
    CHAT_TRACE_PULSE,
    CHAT_TRACE_ROW,
    CHAT_TRACE_SECONDARY,
    CHAT_TRACE_SUMMARY,
    CHAT_TRACE_SUMMARY_LABEL,
    CHAT_TRACE_THOUGHT_BODY,
    CHAT_TRACE_THOUGHT_BTN,
    CHAT_TRACE_THOUGHT_HEAD,
    CHAT_TRACE_VERB,
    CHAT_TRACE_WAITING,
};

use super::facade::load_agent_run_detail;
use super::ui::{
    StepPhase, ThinkingStatus, ToolStatus, UiAgentThinking, UiChatMessage, UiThinkingStep,
    UiThinkingTool,
};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdChevronDown, LdChevronRight};
use dioxus_free_icons::Icon;
use serde_json::Value;

/// 轨迹行点击后打开文件 / 本次编辑前后对比 / 跳转子任务会话。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceFileOpen {
    /// 仅打开文件（读取等）。
    File { path: String },
    /// 展示该次 edit / write 的前后文本（来自工具参数，不依赖当前磁盘内容）。
    Diff {
        path: String,
        tool_id: String,
        old_text: String,
        new_text: String,
    },
    /// 打开 Multitask 子任务对应的独立会话。
    Conversation { id: String },
}

#[derive(Clone, PartialEq)]
struct TimelineRow {
    key: String,
    kind: TimelineKind,
}

#[derive(Clone, PartialEq)]
enum TimelineKind {
    Thought {
        /// 行标题：空流时「思考中 · 步骤 N」，有内容时「思考中」/「思考」。
        label: String,
        full: String,
        running: bool,
    },
    Tool {
        verb: String,
        primary: String,
        secondary: String,
        running: bool,
        open: Option<TraceFileOpen>,
    },
    Subagent {
        title: String,
        prompt: String,
        summary: String,
        conversation_id: Option<String>,
        worktree: String,
        running: bool,
    },
}

fn json_str_field(args: &str, keys: &[&str]) -> Option<String> {
    let v: Value = serde_json::from_str(args).ok()?;
    for key in keys {
        if let Some(s) = v.get(*key).and_then(|x| x.as_str()) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

/// 允许空字符串（如 `new_string: ""` 表示删除片段）。
fn json_str_field_allow_empty(args: &str, keys: &[&str]) -> Option<String> {
    let v: Value = serde_json::from_str(args).ok()?;
    for key in keys {
        if let Some(s) = v.get(*key).and_then(|x| x.as_str()) {
            return Some(s.to_string());
        }
    }
    None
}

fn path_file_name(path: &str) -> String {
    let trimmed = path.trim().trim_matches('"').replace('\\', "/");
    trimmed
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn truncate_chars(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max).collect();
    out.push('…');
    out
}

fn strip_subagent_meta(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.split_once("\n---\n") {
        return rest.1.trim().to_string();
    }
    trimmed
        .lines()
        .filter(|line| {
            let t = line.trim();
            !t.starts_with("【")
                && !t.starts_with("conversation_id:")
                && !t.starts_with("worktree:")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn summary_meta_field(raw: Option<&str>, prefix: &str) -> Option<String> {
    let raw = raw?;
    for line in raw.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix(prefix) {
            let v = rest.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn shorten_worktree_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    if let Some(idx) = normalized.find(".pusa/worktrees/") {
        return normalized[idx..].to_string();
    }
    path.to_string()
}

fn format_tool_row(tool: &UiThinkingTool) -> TimelineKind {
    let running = matches!(tool.status, ToolStatus::Running);
    let name = tool.name.trim();
    match name {
        "Read" | "read_file" => {
            let path = json_str_field(&tool.args, &["file_path", "path"]).unwrap_or_default();
            let open = if path.is_empty() {
                None
            } else {
                Some(TraceFileOpen::File { path: path.clone() })
            };
            TimelineKind::Tool {
                verb: "读取".into(),
                primary: if path.is_empty() {
                    "文件".into()
                } else {
                    path_file_name(&path)
                },
                secondary: String::new(),
                running,
                open,
            }
        }
        "Write" | "write_file" => {
            let path = json_str_field(&tool.args, &["file_path", "path"]).unwrap_or_default();
            let content = json_str_field_allow_empty(&tool.args, &["content"]).unwrap_or_default();
            let open = if path.is_empty() {
                None
            } else if content.is_empty() {
                Some(TraceFileOpen::File { path: path.clone() })
            } else {
                Some(TraceFileOpen::Diff {
                    path: path.clone(),
                    tool_id: tool.id.clone(),
                    old_text: String::new(),
                    new_text: content,
                })
            };
            TimelineKind::Tool {
                verb: "写入".into(),
                primary: if path.is_empty() {
                    "文件".into()
                } else {
                    path_file_name(&path)
                },
                secondary: String::new(),
                running,
                open,
            }
        }
        "Edit" | "edit" => {
            let path = json_str_field(&tool.args, &["file_path", "path"]).unwrap_or_default();
            let old_text =
                json_str_field_allow_empty(&tool.args, &["old_string", "oldText", "old_text"])
                    .unwrap_or_default();
            let new_text =
                json_str_field_allow_empty(&tool.args, &["new_string", "newText", "new_text"])
                    .unwrap_or_default();
            let open = if path.is_empty() {
                None
            } else if old_text.is_empty() && new_text.is_empty() {
                Some(TraceFileOpen::File { path: path.clone() })
            } else {
                Some(TraceFileOpen::Diff {
                    path: path.clone(),
                    tool_id: tool.id.clone(),
                    old_text,
                    new_text,
                })
            };
            TimelineKind::Tool {
                verb: "编辑".into(),
                primary: if path.is_empty() {
                    "文件".into()
                } else {
                    path_file_name(&path)
                },
                secondary: String::new(),
                running,
                open,
            }
        }
        "Grep" | "grep" => {
            let pattern = json_str_field(&tool.args, &["pattern"]).unwrap_or_default();
            let path = json_str_field(&tool.args, &["file_path", "path"]).unwrap_or_default();
            TimelineKind::Tool {
                verb: "搜索".into(),
                primary: truncate_chars(&pattern, 48),
                secondary: if path.is_empty() {
                    String::new()
                } else {
                    path_file_name(&path)
                },
                running,
                open: None,
            }
        }
        "Glob" | "find_files" => {
            let glob = json_str_field(&tool.args, &["pattern", "glob"]).unwrap_or_default();
            TimelineKind::Tool {
                verb: "找文件".into(),
                primary: truncate_chars(&glob, 48),
                secondary: String::new(),
                running,
                open: None,
            }
        }
        "semantic_search" => {
            let query = json_str_field(&tool.args, &["query"]).unwrap_or_default();
            TimelineKind::Tool {
                verb: "语义搜索".into(),
                primary: truncate_chars(&query, 48),
                secondary: String::new(),
                running,
                open: None,
            }
        }
        "exec_bash" => {
            let cmd = json_str_field(&tool.args, &["command"]).unwrap_or_default();
            TimelineKind::Tool {
                verb: "执行".into(),
                primary: truncate_chars(cmd.trim(), 72),
                secondary: String::new(),
                running,
                open: None,
            }
        }
        "search_skills" => {
            let query = json_str_field(&tool.args, &["query"]).unwrap_or_default();
            TimelineKind::Tool {
                verb: "检索技能".into(),
                primary: truncate_chars(&query, 48),
                secondary: json_str_field(&tool.args, &["scope"]).unwrap_or_default(),
                running,
                open: None,
            }
        }
        "search_market_skills" => {
            let query = json_str_field(&tool.args, &["query"]).unwrap_or_default();
            TimelineKind::Tool {
                verb: "搜索市场".into(),
                primary: if query.is_empty() {
                    "浏览".into()
                } else {
                    truncate_chars(&query, 48)
                },
                secondary: String::new(),
                running,
                open: None,
            }
        }
        "install_skill" => {
            let id = json_str_field(&tool.args, &["id", "name"]).unwrap_or_default();
            TimelineKind::Tool {
                verb: "安装技能".into(),
                primary: if id.is_empty() { "技能".into() } else { id },
                secondary: json_str_field(&tool.args, &["source"]).unwrap_or_default(),
                running,
                open: None,
            }
        }
        "spawn_subagent" => {
            let title = json_str_field(&tool.args, &["title"]).unwrap_or_else(|| "子任务".into());
            let prompt = json_str_field(&tool.args, &["prompt"]).unwrap_or_default();
            let conversation_id = json_str_field(&tool.args, &["conversation_id"])
                .or_else(|| summary_meta_field(tool.summary.as_deref(), "conversation_id:"));
            let worktree = json_str_field(&tool.args, &["worktree"])
                .or_else(|| summary_meta_field(tool.summary.as_deref(), "worktree:"))
                .map(|p| shorten_worktree_path(&p))
                .unwrap_or_default();
            let summary = tool
                .summary
                .as_ref()
                .map(|s| strip_subagent_meta(s))
                .filter(|s| !s.is_empty())
                .unwrap_or_default();
            TimelineKind::Subagent {
                title,
                prompt: truncate_chars(&prompt, 120),
                summary: truncate_chars(&summary, 220),
                conversation_id,
                worktree,
                running,
            }
        }
        _ => {
            let detail = tool
                .summary
                .as_ref()
                .map(|s| truncate_chars(s.trim(), 80))
                .filter(|s| !s.is_empty())
                .or_else(|| {
                    let args = tool.args.trim();
                    if args.is_empty() || args == "{}" {
                        None
                    } else {
                        Some(truncate_chars(args, 64))
                    }
                })
                .unwrap_or_default();
            TimelineKind::Tool {
                verb: if name.is_empty() {
                    "调用".into()
                } else {
                    name.to_string()
                },
                primary: detail,
                secondary: String::new(),
                running,
                open: None,
            }
        }
    }
}

/// 流式思考尾部预览：取末尾若干非空行，便于用户不用展开也能看到进度。
fn thought_streaming_snippet(text: &str) -> String {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return String::new();
    }
    let start = lines.len().saturating_sub(3);
    let joined = lines[start..].join("\n");
    truncate_chars(&joined, 180)
}

fn build_timeline(steps: &[UiThinkingStep]) -> Vec<TimelineRow> {
    let mut rows = Vec::new();
    for step in steps {
        let text = step.model_text.trim();
        // Thinking 阶段整段视为进行中（含已有 model_text 的流式窗口）。
        let thought_running = matches!(step.phase, StepPhase::Thinking);
        if !text.is_empty() || thought_running {
            let label = if text.is_empty() {
                format!("思考中 · 步骤 {}", step.index)
            } else if thought_running {
                "思考中".into()
            } else {
                "思考".into()
            };
            rows.push(TimelineRow {
                key: format!("thought-{}", step.index),
                kind: TimelineKind::Thought {
                    label,
                    full: text.to_string(),
                    running: thought_running,
                },
            });
        }
        for tool in &step.tools {
            rows.push(TimelineRow {
                key: format!("tool-{}", tool.id),
                kind: format_tool_row(tool),
            });
        }
    }
    rows
}

fn format_worked_for(ms: Option<i64>, running: bool) -> String {
    if running {
        return "正在处理".into();
    }
    let Some(ms) = ms else {
        return "已完成".into();
    };
    if ms < 1000 {
        format!("工作了 {ms} 毫秒")
    } else if ms < 60_000 {
        let secs = (ms as f64 / 1000.0).round() as i64;
        format!("工作了 {secs} 秒")
    } else {
        let mins = ms / 60_000;
        let secs = (ms % 60_000) / 1000;
        if secs == 0 {
            format!("工作了 {mins} 分钟")
        } else {
            format!("工作了 {mins} 分 {secs} 秒")
        }
    }
}

fn activity_summary(steps: &[UiThinkingStep]) -> Option<String> {
    let mut reads = 0usize;
    let mut writes = 0usize;
    let mut edits = 0usize;
    let mut searches = 0usize;
    let mut execs = 0usize;
    let mut subagents = 0usize;
    let mut other = 0usize;
    for step in steps {
        for tool in &step.tools {
            match tool.name.trim() {
                "Read" | "read_file" => reads += 1,
                "Write" | "write_file" => writes += 1,
                "Edit" | "edit" => edits += 1,
                "search_skills"
                | "search_market_skills"
                | "Grep"
                | "grep"
                | "Glob"
                | "find_files"
                | "semantic_search" => searches += 1,
                "exec_bash" => execs += 1,
                "spawn_subagent" => subagents += 1,
                _ => other += 1,
            }
        }
    }
    let mut parts = Vec::new();
    if subagents > 0 {
        parts.push(format!("派发 {subagents} 个子任务"));
    }
    if reads > 0 {
        parts.push(format!("读取 {reads} 个文件"));
    }
    if writes > 0 {
        parts.push(format!("写入 {writes} 次"));
    }
    if edits > 0 {
        parts.push(format!("编辑 {edits} 次"));
    }
    if searches > 0 {
        parts.push(format!("搜索 {searches} 次"));
    }
    if execs > 0 {
        parts.push(format!("执行 {execs} 次"));
    }
    if other > 0 && parts.is_empty() {
        parts.push(format!("调用 {other} 个工具"));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("，"))
    }
}

/// Cursor 式步骤时间线：摘要可折叠，下方为 Thought / 工具行。
#[component]
pub fn ChatThinkingPanel(
    msg_index: usize,
    mut chat_messages: Signal<Vec<UiChatMessage>>,
    thinking: UiAgentThinking,
    /// 尚无最终回答、仍在跑 Agent 时为 true（用于等待态）。
    busy: bool,
    /// 点击「编辑/读取/写入」文件名时打开中间栏。
    on_open_trace_file: EventHandler<TraceFileOpen>,
) -> Element {
    let rows = build_timeline(&thinking.steps);
    let running = matches!(thinking.status, ThinkingStatus::Running)
        || (busy && !matches!(thinking.status, ThinkingStatus::Error));
    let has_rows = !rows.is_empty();
    if !has_rows && !running {
        return rsx! {};
    }

    let summary = format_worked_for(thinking.resolved_duration_ms(), running);
    let activity = activity_summary(&thinking.steps);
    let expanded = thinking.expanded || running;
    let mut open_thoughts = use_signal(std::collections::HashSet::<String>::new);

    rsx! {
        div { class: CHAT_TRACE,
            button {
                r#type: "button",
                class: CHAT_TRACE_SUMMARY,
                aria_expanded: expanded,
                onclick: move |_| {
                    chat_messages.with_mut(|msgs| {
                        if let Some(UiChatMessage::Assistant { thinking, .. }) =
                            msgs.get_mut(msg_index)
                        {
                            // 运行中强制展开；结束后允许折叠/展开。
                            if !matches!(thinking.status, ThinkingStatus::Running) {
                                thinking.expanded = !thinking.expanded;
                            } else {
                                thinking.expanded = true;
                            }
                        }
                    });
                },
                span { class: CHAT_TRACE_SUMMARY_LABEL, "{summary}" }
                span { class: CHAT_TRACE_CHEVRON, aria_hidden: "true",
                    if expanded {
                        Icon { icon: LdChevronDown, width: 14, height: 14, fill: "currentColor" }
                    } else {
                        Icon { icon: LdChevronRight, width: 14, height: 14, fill: "currentColor" }
                    }
                }
            }

            if expanded {
                div { class: CHAT_TRACE_BODY,
                    if let Some(act) = activity.clone() {
                        div { class: CHAT_TRACE_ACTIVITY,
                            span { class: CHAT_TRACE_ACTIVITY_LABEL, "{act}" }
                        }
                    }
                    if has_rows {
                        div { class: CHAT_TRACE_LIST, role: "list",
                            for row in rows.iter() {
                                {
                                    let key = row.key.clone();
                                    match &row.kind {
                                        TimelineKind::Thought {
                                            label,
                                            full,
                                            running,
                                        } => {
                                            let can_expand = !full.trim().is_empty() && !*running;
                                            let is_open = open_thoughts().contains(&key);
                                            let label = label.clone();
                                            let full = full.clone();
                                            let running = *running;
                                            // 流式中强制展示完整正文（限高滚动）；完成后默认收起，保留尾部摘要。
                                            let show_live = running && !full.is_empty();
                                            let live_body = full.clone();
                                            let collapsed_snip =
                                                if !running && !is_open && !full.is_empty() {
                                                    thought_streaming_snippet(&full)
                                                } else {
                                                    String::new()
                                                };
                                            rsx! {
                                                div {
                                                    key: "{key}",
                                                    class: CHAT_TRACE_ROW,
                                                    "data-thought": true,
                                                    "data-running": running,
                                                    role: "listitem",
                                                    if can_expand {
                                                        button {
                                                            r#type: "button",
                                                            class: CHAT_TRACE_THOUGHT_BTN,
                                                            aria_expanded: is_open,
                                                            onclick: {
                                                                let key = key.clone();
                                                                move |_| {
                                                                    open_thoughts.with_mut(|set| {
                                                                        if set.contains(&key) {
                                                                            set.remove(&key);
                                                                        } else {
                                                                            set.insert(key.clone());
                                                                        }
                                                                    });
                                                                }
                                                            },
                                                            span { class: CHAT_TRACE_VERB, "data-muted": true, "{label}" }
                                                            span { class: CHAT_TRACE_MINI_CHEVRON, aria_hidden: "true",
                                                                if is_open {
                                                                    Icon { icon: LdChevronDown, width: 12, height: 12, fill: "currentColor" }
                                                                } else {
                                                                    Icon { icon: LdChevronRight, width: 12, height: 12, fill: "currentColor" }
                                                                }
                                                            }
                                                        }
                                                        if is_open {
                                                            pre { class: CHAT_TRACE_THOUGHT_BODY, "{full}" }
                                                        } else if !collapsed_snip.is_empty() {
                                                            pre {
                                                                class: CHAT_TRACE_THOUGHT_BODY,
                                                                "data-snip": true,
                                                                "{collapsed_snip}"
                                                            }
                                                        }
                                                    } else {
                                                        div { class: CHAT_TRACE_THOUGHT_HEAD,
                                                            span { class: CHAT_TRACE_VERB, "data-muted": true, "{label}" }
                                                            if running {
                                                                span { class: CHAT_TRACE_PULSE, aria_hidden: "true" }
                                                            }
                                                        }
                                                        if show_live {
                                                            pre {
                                                                class: CHAT_TRACE_THOUGHT_BODY,
                                                                "data-live": true,
                                                                "{live_body}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        TimelineKind::Tool {
                                            verb,
                                            primary,
                                            secondary,
                                            running,
                                            open,
                                        } => {
                                            let verb = verb.clone();
                                            let primary = primary.clone();
                                            let secondary = secondary.clone();
                                            let running = *running;
                                            let open = open.clone();
                                            let tip = match &open {
                                                Some(TraceFileOpen::Diff { .. }) => {
                                                    "在文件树中定位并查看行内 diff"
                                                }
                                                Some(TraceFileOpen::Conversation { .. }) => {
                                                    "打开独立会话"
                                                }
                                                Some(TraceFileOpen::File { .. }) | None => {
                                                    "在文件树中定位并打开"
                                                }
                                            };
                                            rsx! {
                                                div {
                                                    key: "{key}",
                                                    class: CHAT_TRACE_ROW,
                                                    "data-running": running,
                                                    role: "listitem",
                                                    span { class: CHAT_TRACE_VERB, "{verb}" }
                                                    if !primary.is_empty() {
                                                        if let Some(action) = open.clone() {
                                                            button {
                                                                r#type: "button",
                                                                class: CHAT_TRACE_PRIMARY_LINK,
                                                                title: "{tip}",
                                                                onclick: move |_| {
                                                                    on_open_trace_file.call(action.clone());
                                                                },
                                                                "{primary}"
                                                            }
                                                        } else {
                                                            span { class: CHAT_TRACE_PRIMARY, "{primary}" }
                                                        }
                                                    }
                                                    if !secondary.is_empty() {
                                                        span { class: CHAT_TRACE_SECONDARY, "{secondary}" }
                                                    }
                                                    if running {
                                                        span { class: CHAT_TRACE_PULSE, aria_hidden: "true" }
                                                    }
                                                }
                                            }
                                        }
                                        TimelineKind::Subagent {
                                            title,
                                            prompt,
                                            summary,
                                            conversation_id,
                                            worktree,
                                            running,
                                        } => {
                                            let title = title.clone();
                                            let prompt = prompt.clone();
                                            let summary = summary.clone();
                                            let conversation_id = conversation_id.clone();
                                            let worktree = worktree.clone();
                                            let running = *running;
                                            rsx! {
                                                div {
                                                    key: "{key}",
                                                    class: CHAT_SUBTASK_CARD,
                                                    "data-running": running,
                                                    role: "listitem",
                                                    div { class: CHAT_SUBTASK_HEAD,
                                                        span { class: CHAT_SUBTASK_BADGE, "子任务" }
                                                        span { class: CHAT_SUBTASK_TITLE, "{title}" }
                                                        span { class: CHAT_SUBTASK_STATUS,
                                                            if running { "运行中" } else { "已完成" }
                                                        }
                                                        if running {
                                                            span { class: CHAT_TRACE_PULSE, aria_hidden: "true" }
                                                        }
                                                    }
                                                    if !prompt.is_empty() {
                                                        div { class: CHAT_SUBTASK_TEXT, "{prompt}" }
                                                    }
                                                    if !worktree.is_empty() {
                                                        div { class: CHAT_SUBTASK_TEXT, "worktree · {worktree}" }
                                                    }
                                                    if let Some(cid) = conversation_id.clone() {
                                                        button {
                                                            r#type: "button",
                                                            class: CHAT_SUBTASK_OPEN,
                                                            title: "打开独立会话 {cid}",
                                                            onclick: move |_| {
                                                                on_open_trace_file.call(TraceFileOpen::Conversation {
                                                                    id: cid.clone(),
                                                                });
                                                            },
                                                            "打开会话"
                                                        }
                                                    }
                                                    if !summary.is_empty() {
                                                        div { class: CHAT_SUBTASK_SUMMARY, "{summary}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if running {
                        div { class: CHAT_TRACE_WAITING,
                            span { class: CHAT_TRACE_VERB, "data-muted": true, "正在分析" }
                            span { class: CHAT_TRACE_PULSE, aria_hidden: "true" }
                        }
                    }
                }
            }
        }
    }
}

/// 历史助手消息：按 `agent_run_id` 懒加载思考步骤。
#[component]
pub fn ChatThinkingHydrator(
    msg_index: usize,
    mut chat_messages: Signal<Vec<UiChatMessage>>,
) -> Element {
    use_effect(move || {
        let Some(UiChatMessage::Assistant {
            agent_run_id: Some(run_id),
            thinking,
            ..
        }) = chat_messages().get(msg_index).cloned()
        else {
            return;
        };
        if thinking.hydrate_requested || !thinking.steps.is_empty() {
            return;
        }
        chat_messages.with_mut(|msgs| {
            if let Some(UiChatMessage::Assistant { thinking, .. }) = msgs.get_mut(msg_index) {
                thinking.hydrate_requested = true;
            }
        });
        spawn(async move {
            if let Ok(detail) = load_agent_run_detail(run_id).await {
                chat_messages.with_mut(|msgs| {
                    if let Some(UiChatMessage::Assistant {
                        content, thinking, ..
                    }) = msgs.get_mut(msg_index)
                    {
                        let answer = content.clone();
                        *thinking = UiAgentThinking::from_agent_run_detail(&detail);
                        thinking.clear_thoughts_duplicating_answer(&answer);
                    }
                });
            }
        });
    });
    rsx! {}
}
