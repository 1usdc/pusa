//! 桌面端集成终端：本机 PTY + 登录 shell，输出推送到 UI signal。
//! 行内提示符编辑 + 多会话（右侧 session rail）。

use std::io::{Read, Write};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use dioxus::events::FormEvent;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdChevronsUp, LdPlus, LdSquareTerminal, LdX};
use dioxus_free_icons::Icon;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

const MAX_OUTPUT_CHARS: usize = 200_000;

/// 向 PTY 写入的句柄（可在 UI 线程克隆）。
#[derive(Clone)]
pub struct TerminalWriter {
    tx: Sender<Vec<u8>>,
}

impl TerminalWriter {
    pub fn write_bytes(&self, bytes: &[u8]) {
        let _ = self.tx.send(bytes.to_vec());
    }

    pub fn write_str(&self, s: &str) {
        self.write_bytes(s.as_bytes());
    }
}

/// 单个终端会话的 UI 状态。
#[derive(Clone, Copy)]
struct TerminalSession {
    id: u64,
    output: Signal<String>,
    draft: Signal<String>,
    err_msg: Signal<Option<String>>,
    pty: Signal<Option<TerminalWriter>>,
}

impl TerminalSession {
    fn spawn(id: u64) -> Self {
        let output = Signal::new(String::new());
        let draft = Signal::new(String::new());
        let mut err_msg = Signal::new(None::<String>);
        let mut pty = Signal::new(None::<TerminalWriter>);
        match spawn_interactive_shell(output) {
            Ok(w) => {
                pty.set(Some(w));
            }
            Err(e) => {
                err_msg.set(Some(e));
            }
        }
        Self {
            id,
            output,
            draft,
            err_msg,
            pty,
        }
    }

    fn submit_line(&self) {
        let line = (self.draft)();
        let mut draft = self.draft;
        draft.set(String::new());
        if let Some(writer) = (self.pty)() {
            writer.write_str(&format!("{line}\n"));
        }
    }

    /// 尽力结束会话：向 shell 发 exit；PTY 子进程此前被 forget，无法强杀。
    fn request_exit(&self) {
        if let Some(writer) = (self.pty)() {
            writer.write_str("exit\n");
        }
        let mut pty = self.pty;
        pty.set(None);
    }
}

fn push_new_session(
    mut sessions: Signal<Vec<TerminalSession>>,
    mut active_id: Signal<Option<u64>>,
    mut next_id: Signal<u64>,
) {
    let id = next_id();
    next_id.set(id + 1);
    let session = TerminalSession::spawn(id);
    sessions.write().push(session);
    active_id.set(Some(id));
}

/// 去掉常见 CSI / OSC，并识别清屏序列。
///
/// 返回值：`(should_clear_display, plain_text)`。
/// `should_clear_display` 为 true 时，应将既有显示缓冲清空后再追加 `plain_text`
///（仅保留本 chunk 中**最后一次**清屏之后的正文）。
///
/// 识别：`CSI 2J` / `CSI 3J`（Erase in Display）、`ESC c`（RIS）。
/// `clear` 常见输出 `\x1b[H\x1b[2J`：CUP 被剥掉，`2J` 触发清空。
fn sanitize_pty_chunk(input: &str) -> (bool, String) {
    let mut out = String::with_capacity(input.len());
    let mut should_clear = false;
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.peek().copied() {
            Some('[') => {
                chars.next();
                let mut params = String::new();
                while let Some(x) = chars.next() {
                    if ('@'..='~').contains(&x) {
                        // ED — Erase in Display：2=整屏，3=整屏+scrollback
                        if x == 'J' {
                            let p = params.trim();
                            if p == "2" || p == "3" {
                                should_clear = true;
                                out.clear();
                            }
                        }
                        break;
                    }
                    params.push(x);
                }
            }
            Some(']') => {
                chars.next();
                while let Some(x) = chars.next() {
                    if x == '\u{7}' || x == '\\' {
                        break;
                    }
                }
            }
            Some('(') | Some(')') => {
                chars.next();
                let _ = chars.next();
            }
            Some('c') => {
                // RIS — Reset to Initial State：等价整缓冲清屏
                chars.next();
                should_clear = true;
                out.clear();
            }
            _ => {
                let _ = chars.next();
            }
        }
    }
    (should_clear, out)
}

/// 将一块 PTY 输出合并进显示缓冲（清屏 / 去 ANSI / 行内控制折叠）。
fn apply_pty_chunk_to_buffer(buf: &mut String, chunk: &str) {
    let (should_clear, cleaned) = sanitize_pty_chunk(chunk);
    if !should_clear && cleaned.is_empty() {
        return;
    }
    if should_clear {
        buf.clear();
    }
    if cleaned.is_empty() {
        return;
    }
    buf.push_str(&cleaned);
    // 必须在整段缓冲上折叠：`\r` 常跨 chunk（先空格后回车再写提示符）。
    *buf = fold_line_controls(buf);
    *buf = strip_leading_eol_marks(buf);
    if buf.len() > MAX_OUTPUT_CHARS {
        let keep_from = buf.len() - MAX_OUTPUT_CHARS;
        let trimmed = buf[keep_from..].to_string();
        *buf = strip_leading_eol_marks(&fold_line_controls(&trimmed));
    }
}

/// 模拟终端对 `\r` / Backspace / BEL 的行内改写，避免 HTML `pre` 把清行空格当成前导空白。
///
/// 典型坏例：shell 以「空格填充 + `\\r` + 提示符」重绘当前行；若不折叠 `\\r`，
/// `white-space: pre` 会保留左侧空格，提示符看起来漂在右上。
fn fold_line_controls(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut line: Vec<char> = Vec::new();
    let mut col = 0usize;
    for c in input.chars() {
        match c {
            '\n' => {
                out.extend(line.iter().copied());
                out.push('\n');
                line.clear();
                col = 0;
            }
            '\r' => {
                col = 0;
            }
            '\u{8}' => {
                if col > 0 {
                    col -= 1;
                    if col < line.len() {
                        line.truncate(col);
                    }
                }
            }
            '\u{7}' => {}
            c => {
                if col < line.len() {
                    line[col] = c;
                } else {
                    line.push(c);
                }
                col += 1;
            }
        }
    }
    out.extend(line.iter().copied());
    out
}

/// 去掉 zsh `PROMPT_SP` / `PROMPT_EOL_MARK` 在行首留下的孤立 `%`。
fn strip_leading_eol_marks(s: &str) -> String {
    let mut start = 0usize;
    let mut rest = s;
    while let Some(rel_end) = rest.find('\n') {
        let line = &rest[..rel_end];
        let content = line.trim_end_matches('\r').trim();
        if content == "%" {
            start += rel_end + 1;
            rest = &s[start..];
            continue;
        }
        break;
    }
    if rest.trim_end_matches(['\r', '\n']).trim() == "%" {
        return String::new();
    }
    s[start..].to_string()
}

fn is_shell_prompt_line(line: &str) -> bool {
    let t = line.trim_end_matches('\r').trim_end();
    matches!(t, "%" | "#" | "$")
        || t.ends_with(" %")
        || t.ends_with(" #")
        || t.ends_with(" $")
}

/// 将缓冲拆成「历史输出」与「当前提示符」，输入光标紧贴提示符 `%` 之后。
pub fn display_parts(buf: &str) -> (String, String) {
    // 防御性折叠：即便缓冲尚未经 append_output 处理，也不把 `\r` 空格原样塞进 pre。
    let cleaned = strip_leading_eol_marks(&fold_line_controls(buf));
    if cleaned.is_empty() {
        return (String::new(), String::new());
    }

    let ends_with_nl = cleaned.ends_with('\n') || cleaned.ends_with("\r\n");
    // 提示符通常是未换行的最后一行；若整段以换行结束，则命令仍在跑——不单独画假提示符。
    if ends_with_nl {
        return (cleaned, String::new());
    }

    let (prefix, last) = match cleaned.rfind('\n') {
        Some(i) => (&cleaned[..i + 1], &cleaned[i + 1..]),
        None => ("", cleaned.as_str()),
    };

    if is_shell_prompt_line(last) {
        // `\r` 覆写后常残留行尾空白，trim 掉以免撑开 prompt flex item。
        let prompt = last.trim_end_matches('\r').trim_end().to_string();
        (prefix.to_string(), prompt)
    } else {
        // 非提示符的未完成行（例如程序正在写同一行）：整段当历史，暂不显示编辑行。
        (cleaned.to_string(), String::new())
    }
}

fn append_output(mut output: Signal<String>, chunk: &str) {
    output.with_mut(|buf| apply_pty_chunk_to_buffer(buf, chunk));
}

fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(windows) {
            "powershell.exe".to_string()
        } else {
            "/bin/zsh".to_string()
        }
    })
}

/// 启动交互式登录 shell；返回写入句柄。输出经 `output` signal 推送。
pub fn spawn_interactive_shell(output: Signal<String>) -> Result<TerminalWriter, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 100,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("打开 PTY 失败：{e}"))?;

    let shell = default_shell();
    let mut cmd = CommandBuilder::new(&shell);
    if !cfg!(windows) {
        cmd.arg("-l");
        // 避免 zsh 在无完整行末尾画孤立的 `%`（PROMPT_EOL_MARK）。
        cmd.env("PROMPT_EOL_MARK", "");
        // GUI 启动时常丢 Homebrew；与 Agent exec_bash 共用增强 PATH。
        cmd.env("PATH", shared::tools::file::enriched_path());
    }
    if let Ok(cwd) = std::env::current_dir() {
        cmd.cwd(cwd);
    }

    let _child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("启动 shell（{shell}）失败：{e}"))?;

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("克隆 PTY reader 失败：{e}"))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("获取 PTY writer 失败：{e}"))?;

    // 持有 master，避免 drop 关闭 PTY。
    let _master_keep: Arc<Mutex<Box<dyn MasterPty + Send>>> =
        Arc::new(Mutex::new(pair.master));

    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let writer = Arc::new(Mutex::new(writer));
    let writer_for_thread = Arc::clone(&writer);

    thread::Builder::new()
        .name("ac-terminal-writer".into())
        .spawn(move || {
            while let Ok(bytes) = rx.recv() {
                let mut w = match writer_for_thread.lock() {
                    Ok(g) => g,
                    Err(_) => break,
                };
                if w.write_all(&bytes).is_err() {
                    break;
                }
                let _ = w.flush();
            }
        })
        .map_err(|e| format!("启动终端写线程失败：{e}"))?;

    let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    thread::Builder::new()
        .name("ac-terminal-reader".into())
        .spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if out_tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        })
        .map_err(|e| format!("启动终端读线程失败：{e}"))?;

    spawn(async move {
        while let Some(bytes) = out_rx.recv().await {
            let text = String::from_utf8_lossy(&bytes);
            append_output(output, text.as_ref());
        }
        append_output(output, "\n[shell 已退出]\n");
    });

    // 避免未使用警告：master 需活到进程结束。
    std::mem::forget(_master_keep);
    std::mem::forget(_child);

    Ok(TerminalWriter { tx })
}

/// 桌面终端面板：行内 prompt 编辑 + 多会话 rail。
#[component]
pub fn DesktopTerminal(
    mut show_terminal: Signal<bool>,
    on_resize_start: EventHandler<MouseEvent>,
    on_toggle_maximize: EventHandler<()>,
) -> Element {
    let mut sessions = use_signal(Vec::<TerminalSession>::new);
    let mut active_id = use_signal(|| None::<u64>);
    let next_id = use_signal(|| 1u64);

    // 首次打开面板时自动建一个会话。
    use_effect(move || {
        if !show_terminal() {
            return;
        }
        if sessions.read().is_empty() {
            push_new_session(sessions, active_id, next_id);
        }
    });

    // 当前会话输出变化时滚到末尾。
    use_effect(move || {
        let Some(id) = active_id() else {
            return;
        };
        let output = sessions
            .read()
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.output);
        if let Some(output) = output {
            let _ = output().len();
            let _ = document::eval(
                r#"(() => {
                    const el = document.getElementById('ac-terminal-body');
                    if (el) el.scrollTop = el.scrollHeight;
                })()"#,
            );
        }
    });

    let active = active_id().and_then(|id| {
        sessions
            .read()
            .iter()
            .copied()
            .find(|s| s.id == id)
    });

    rsx! {
        div { class: "ac-terminal-panel",
            div {
                class: "ac-terminal-resizer",
                title: "拖动调整终端高度",
                onmousedown: move |event| on_resize_start.call(event),
            }
            div { class: "ac-terminal-toolbar",
                span { class: "ac-terminal-toolbar-title", "终端" }
                div { class: "ac-terminal-toolbar-actions",
                    button {
                        r#type: "button",
                        class: "ac-terminal-toolbar-btn ac-terminal-toolbar-add",
                        title: "添加终端",
                        aria_label: "添加终端",
                        onclick: move |_| {
                            push_new_session(sessions, active_id, next_id);
                        },
                        Icon {
                            icon: LdPlus,
                            width: 14,
                            height: 14,
                            fill: "currentColor",
                        }
                    }
                    button {
                        r#type: "button",
                        class: "ac-terminal-toolbar-btn",
                        title: "最大化终端",
                        aria_label: "最大化终端",
                        onclick: move |_| on_toggle_maximize.call(()),
                        Icon {
                            icon: LdChevronsUp,
                            width: 14,
                            height: 14,
                            fill: "currentColor",
                        }
                    }
                    button {
                        r#type: "button",
                        class: "ac-terminal-toolbar-btn",
                        title: "关闭终端",
                        aria_label: "关闭终端",
                        onclick: move |_| show_terminal.set(false),
                        Icon {
                            icon: LdX,
                            width: 14,
                            height: 14,
                            fill: "currentColor",
                        }
                    }
                }
            }
            div { class: "ac-terminal-main",
                {
                    if let Some(session) = active {
                        let (term_history, term_prompt) = display_parts(&(session.output)());
                        let draft_value = (session.draft)();
                        let err = (session.err_msg)();
                        rsx! {
                            div {
                                id: "ac-terminal-body",
                                class: "ac-terminal-body scrollbar-hide",
                                onclick: move |_| {
                                    let _ = document::eval(
                                        r#"(() => {
                                            const el = document.querySelector('#ac-terminal-body .ac-terminal-input');
                                            if (el) el.focus();
                                        })()"#,
                                    );
                                },
                                if let Some(err) = err {
                                    div { class: "ac-terminal-error", "{err}" }
                                } else {
                                    if !term_history.is_empty() {
                                        pre { class: "ac-terminal-output", "{term_history}" }
                                    }
                                    if !term_prompt.is_empty() {
                                        form {
                                            class: "ac-terminal-input-row",
                                            onsubmit: move |event: FormEvent| {
                                                event.prevent_default();
                                                session.submit_line();
                                            },
                                            span { class: "ac-terminal-prompt", "{term_prompt}" }
                                            input {
                                                class: "ac-terminal-input",
                                                r#type: "text",
                                                value: "{draft_value}",
                                                autocomplete: "off",
                                                spellcheck: "false",
                                                autofocus: true,
                                                oninput: move |e| {
                                                    let mut draft = session.draft;
                                                    draft.set(e.value());
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "ac-terminal-body",
                                div { class: "ac-terminal-unavailable",
                                    p { "点击 + 开始。" }
                                }
                            }
                        }
                    }
                }
                div { class: "ac-terminal-session-rail",
                    {
                        let list = sessions();
                        rsx! {
                            for session in list.iter().copied() {
                                {
                                    let id = session.id;
                                    let is_active = active_id() == Some(id);
                                    let btn_class = if is_active {
                                        "ac-terminal-session-btn is-active"
                                    } else {
                                        "ac-terminal-session-btn"
                                    };
                                    rsx! {
                                        button {
                                            key: "{id}",
                                            r#type: "button",
                                            class: "{btn_class}",
                                            title: "终端 {id}",
                                            aria_label: "切换到终端 {id}",
                                            aria_current: if is_active { "true" } else { "false" },
                                            onclick: move |_| active_id.set(Some(id)),
                                            Icon {
                                                icon: LdSquareTerminal,
                                                width: 14,
                                                height: 14,
                                                fill: "currentColor",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "ac-terminal-session-btn ac-terminal-session-add",
                        title: "添加终端",
                        aria_label: "添加终端",
                        onclick: move |_| {
                            push_new_session(sessions, active_id, next_id);
                        },
                        Icon {
                            icon: LdPlus,
                            width: 14,
                            height: 14,
                            fill: "currentColor",
                        }
                    }
                    {
                        let can_close = sessions.read().len() > 1;
                        if can_close {
                            if let Some(id) = active_id() {
                                rsx! {
                                    button {
                                        r#type: "button",
                                        class: "ac-terminal-session-btn ac-terminal-session-close",
                                        title: "关闭当前终端",
                                        aria_label: "关闭当前终端",
                                        onclick: move |_| {
                                            let mut removed = None;
                                            sessions.with_mut(|list| {
                                                if let Some(pos) = list.iter().position(|s| s.id == id) {
                                                    removed = Some(list.remove(pos));
                                                }
                                            });
                                            if let Some(session) = removed {
                                                session.request_exit();
                                            }
                                            let next = sessions.read().last().map(|s| s.id);
                                            active_id.set(next);
                                        },
                                        Icon {
                                            icon: LdX,
                                            width: 12,
                                            height: 12,
                                            fill: "currentColor",
                                        }
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_pty_chunk_to_buffer, display_parts, fold_line_controls, sanitize_pty_chunk,
        strip_leading_eol_marks,
    };

    #[test]
    fn strips_leading_lonely_percent() {
        let raw = "%\nmac@Macbook pusa %";
        assert_eq!(strip_leading_eol_marks(raw), "mac@Macbook pusa %");
        let (history, prompt) = display_parts(raw);
        assert_eq!(history, "");
        assert_eq!(prompt, "mac@Macbook pusa %");
    }

    #[test]
    fn keeps_history_and_moves_prompt() {
        let raw = "hello\nmac@host dir %";
        let (history, prompt) = display_parts(raw);
        assert_eq!(history, "hello\n");
        assert_eq!(prompt, "mac@host dir %");
    }

    #[test]
    fn no_fake_prompt_when_output_ends_with_newline() {
        let (history, prompt) = display_parts("running...\n");
        assert_eq!(history, "running...\n");
        assert_eq!(prompt, "");
    }

    #[test]
    fn folds_cr_clear_line_spaces_before_prompt() {
        let raw = format!("{}{}", " ".repeat(80), "\rmac@Macbook pusa %");
        let folded = fold_line_controls(&raw);
        assert!(!folded.starts_with(' '), "prompt must start at column 0 after CR fold");
        // display_parts 自身也会折叠，覆盖「缓冲里仍带裸 \\r」的路径。
        let (history, prompt) = display_parts(&raw);
        assert_eq!(history, "");
        assert_eq!(prompt, "mac@Macbook pusa %");
    }

    #[test]
    fn folds_cr_overwrite_mid_line() {
        assert_eq!(fold_line_controls("hello world\rHI"), "HIllo world");
        // `\r\n`：回车只复位列，换行提交当前行内容。
        assert_eq!(fold_line_controls("abc\r\ndef"), "abc\ndef");
    }

    #[test]
    fn sanitize_detects_clear_home_and_ed2() {
        // `clear` 典型：CUP + ED2
        let (clear, text) = sanitize_pty_chunk("old\x1b[H\x1b[2Jmac@host %");
        assert!(clear);
        assert_eq!(text, "mac@host %");
    }

    #[test]
    fn sanitize_detects_ed3_and_ris() {
        let (c2, t2) = sanitize_pty_chunk("keep\x1b[3Jafter");
        assert!(c2);
        assert_eq!(t2, "after");

        let (c_ris, t_ris) = sanitize_pty_chunk("keep\x1bcafter");
        assert!(c_ris);
        assert_eq!(t_ris, "after");
    }

    #[test]
    fn sanitize_does_not_clear_on_colors_or_ed0() {
        let (clear, text) = sanitize_pty_chunk("hi\x1b[31mred\x1b[0m\x1b[J");
        assert!(!clear);
        assert_eq!(text, "hired");
    }

    #[test]
    fn apply_clear_empties_buffer_then_keeps_prompt() {
        let mut buf = String::from("cd: no such file\nmac@Macbook pusa % clear\n");
        // 模拟 clear 后 PTY 回写：清屏序列 + 新提示符
        apply_pty_chunk_to_buffer(&mut buf, "\x1b[H\x1b[2Jmac@Macbook pusa %");
        assert_eq!(buf, "mac@Macbook pusa %");
        let (history, prompt) = display_parts(&buf);
        assert_eq!(history, "");
        assert_eq!(prompt, "mac@Macbook pusa %");
    }

    #[test]
    fn apply_clear_only_sequence_empties_buffer() {
        let mut buf = String::from("history line\n");
        apply_pty_chunk_to_buffer(&mut buf, "\x1b[2J");
        assert!(buf.is_empty());
    }
}
