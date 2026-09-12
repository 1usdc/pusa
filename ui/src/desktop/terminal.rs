//! 桌面端集成终端：本机 PTY + xterm.js 真终端模拟器（键直通）。
//! 多会话右侧 rail；输出保留 ANSI，由 xterm 渲染。

use std::io::{Read, Write};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdChevronDown, LdChevronUp, LdPlus, LdSquareTerminal, LdX};
use dioxus_free_icons::Icon;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::Deserialize;

const MAX_SCROLLBACK_CHARS: usize = 400_000;
///  bump 后强制重装 xterm 桥（选区 / Add to Pusa / 主题等）。
const XTERM_BRIDGE_VERSION: u32 = 9;

const XTERM_CSS: Asset = asset!("/assets/terminal/xterm.css");
const XTERM_JS: Asset = asset!("/assets/terminal/xterm.js");
const XTERM_FIT_JS: Asset = asset!("/assets/terminal/addon-fit.js");

/// 智能 UI / 外部请求：在指定命名会话中执行命令。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalRunRequest {
    /// 会话显示名；智能 UI 固定为 `"pusa"`。
    pub session_name: String,
    /// 先 `cd` 到此目录（若有）。
    pub cwd: Option<String>,
    /// 写入 PTY 的命令（自动追加换行）。
    pub command: String,
}

#[derive(Debug, Deserialize)]
struct TermBridgeEvent {
    kind: String,
    #[serde(default)]
    data: String,
    #[serde(default)]
    cols: u16,
    #[serde(default)]
    rows: u16,
}

/// 向 PTY 写入的句柄（可在 UI 线程克隆）；持有 master 以便 resize。
#[derive(Clone)]
pub struct TerminalWriter {
    tx: Sender<Vec<u8>>,
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    last_size: Arc<Mutex<(u16, u16)>>,
}

impl TerminalWriter {
    pub fn write_bytes(&self, bytes: &[u8]) {
        let _ = self.tx.send(bytes.to_vec());
    }

    pub fn write_str(&self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    pub fn resize(&self, rows: u16, cols: u16) {
        let rows = rows.max(2);
        let cols = cols.max(2);
        {
            let Ok(mut last) = self.last_size.lock() else {
                return;
            };
            if *last == (rows, cols) {
                return;
            }
            *last = (rows, cols);
        }
        if let Ok(master) = self.master.lock() {
            let _ = master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
    }
}

/// 单个终端会话的 UI 状态。
#[derive(Clone, Copy)]
struct TerminalSession {
    id: u64,
    /// 空字符串表示普通会话；非空为命名会话（如 `pusa`）。
    name: Signal<String>,
    /// 原始 PTY 输出（含 ANSI），切会话时回放给 xterm。
    scrollback: Signal<String>,
    err_msg: Signal<Option<String>>,
    pty: Signal<Option<TerminalWriter>>,
}

impl TerminalSession {
    fn spawn(id: u64, name: String) -> Self {
        Self {
            id,
            name: Signal::new(name),
            scrollback: Signal::new(String::new()),
            err_msg: Signal::new(None::<String>),
            pty: Signal::new(None::<TerminalWriter>),
        }
    }

    fn start_pty(&self, active_id: Signal<Option<u64>>, rows: u16, cols: u16) {
        if self.pty.peek().is_some() || self.err_msg.peek().is_some() {
            return;
        }
        match spawn_interactive_shell(self.id, active_id, self.scrollback, rows, cols) {
            Ok(w) => {
                let mut pty = self.pty;
                pty.set(Some(w));
            }
            Err(e) => {
                let mut err_msg = self.err_msg;
                err_msg.set(Some(e));
            }
        }
    }

    /// 尽力结束会话：向 shell 发 exit。
    fn request_exit(&self) {
        if let Some(writer) = (self.pty)() {
            writer.write_str("exit\n");
        }
        let mut pty = self.pty;
        pty.set(None);
    }
}

fn push_new_session(
    sessions: Signal<Vec<TerminalSession>>,
    active_id: Signal<Option<u64>>,
    next_id: Signal<u64>,
) {
    push_named_session(sessions, active_id, next_id, String::new());
}

fn push_named_session(
    mut sessions: Signal<Vec<TerminalSession>>,
    mut active_id: Signal<Option<u64>>,
    mut next_id: Signal<u64>,
    name: String,
) -> u64 {
    let id = next_id();
    next_id.set(id + 1);
    let session = TerminalSession::spawn(id, name);
    sessions.write().push(session);
    active_id.set(Some(id));
    id
}

/// 复用同名会话，否则新建并激活。
fn ensure_named_session(
    sessions: Signal<Vec<TerminalSession>>,
    mut active_id: Signal<Option<u64>>,
    next_id: Signal<u64>,
    name: &str,
) -> u64 {
    if let Some(existing) = sessions
        .read()
        .iter()
        .find(|s| (s.name)() == name)
        .map(|s| s.id)
    {
        active_id.set(Some(existing));
        return existing;
    }
    push_named_session(sessions, active_id, next_id, name.to_string())
}

fn attach_pending_ptys(
    sessions: Signal<Vec<TerminalSession>>,
    active_id: Signal<Option<u64>>,
    rows: u16,
    cols: u16,
) {
    let list: Vec<TerminalSession> = sessions.read().iter().copied().collect();
    for session in list {
        session.start_pty(active_id, rows, cols);
    }
}

fn shell_quote(path: &str) -> String {
    format!("\"{}\"", path.replace('\\', "\\\\").replace('"', "\\\""))
}

fn default_shell() -> String {
    if cfg!(windows) {
        // Windows 上 Git/Cursor 常设置 SHELL=bash，GUI 进程里拉 bash 会弹出真实控制台窗口。
        std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
    }
}

/// 终端默认工作目录：当前项目根；不可用时回退进程 cwd。
fn shell_initial_cwd() -> Option<std::path::PathBuf> {
    let root = crate::desktop::files::workspace_root();
    if root.is_dir() {
        return Some(root);
    }
    std::env::current_dir().ok()
}

/// 清屏序列之后保留正文；不清空颜色等 CSI。
fn append_pty_scrollback(buf: &mut String, chunk: &str) {
    let clear_at = chunk
        .rmatch_indices("\u{1b}[3J")
        .map(|(i, _)| i)
        .chain(chunk.rmatch_indices("\u{1b}[2J").map(|(i, _)| i))
        .chain(chunk.rmatch_indices("\u{1b}c").map(|(i, _)| i))
        .max();
    if let Some(i) = clear_at {
        buf.clear();
        buf.push_str(&chunk[i..]);
    } else {
        buf.push_str(chunk);
    }
    if buf.len() > MAX_SCROLLBACK_CHARS {
        let drop_to = buf.len() - MAX_SCROLLBACK_CHARS;
        let cut = buf[drop_to..]
            .find('\n')
            .map(|rel| drop_to + rel + 1)
            .unwrap_or(drop_to);
        let _ = buf.drain(..cut);
    }
}

/// 去掉 CSI/OSC，便于比较提示符是否重复。
fn strip_ansi_for_compare(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            if c != '\r' {
                out.push(c);
            }
            continue;
        }
        match chars.peek().copied() {
            Some('[') => {
                chars.next();
                while let Some(x) = chars.next() {
                    if ('@'..='~').contains(&x) {
                        break;
                    }
                }
            }
            Some(']') => {
                chars.next();
                while let Some(x) = chars.next() {
                    if x == '\u{7}' {
                        break;
                    }
                    if x == '\u{1b}' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
            }
            Some(_) => {
                chars.next();
            }
            None => {}
        }
    }
    out
}

fn is_shell_prompt_line(plain: &str) -> bool {
    let t = plain.trim_end();
    if t.is_empty() {
        return false;
    }
    t.ends_with(" %")
        || t.ends_with(" #")
        || t.ends_with(" $")
        || t.ends_with('❯')
        || matches!(t, "%" | "#" | "$")
}

fn last_significant_line(buf: &str) -> &str {
    buf.lines()
        .rev()
        .map(|l| l.trim_end_matches('\r'))
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
}

/// fit/WINCH 后 shell 常重打同一提示符；丢弃「整段只是重复提示符」的 chunk。
fn filter_duplicate_prompt_chunk(existing: &str, chunk: &str) -> String {
    if chunk.is_empty() || existing.is_empty() {
        return chunk.to_string();
    }
    let last_plain = strip_ansi_for_compare(last_significant_line(existing));
    if !is_shell_prompt_line(&last_plain) {
        return chunk.to_string();
    }
    let last = last_plain.trim_end();
    let chunk_plain = strip_ansi_for_compare(chunk);
    let content_lines: Vec<&str> = chunk_plain
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| !l.is_empty())
        .collect();
    if !content_lines.is_empty() && content_lines.iter().all(|l| *l == last) {
        return String::new();
    }
    chunk.to_string()
}

/// 滚动缓冲末尾若连续两行相同提示符，去掉重复的一行。
fn collapse_trailing_dup_prompts(buf: &mut String) {
    loop {
        let lines: Vec<&str> = buf.lines().collect();
        let mut nonempty_idx: Vec<usize> = Vec::new();
        for (i, line) in lines.iter().enumerate().rev() {
            if !line.trim().is_empty() {
                nonempty_idx.push(i);
                if nonempty_idx.len() == 2 {
                    break;
                }
            }
        }
        if nonempty_idx.len() < 2 {
            break;
        }
        let later = nonempty_idx[0];
        let earlier = nonempty_idx[1];
        let a = strip_ansi_for_compare(lines[earlier]);
        let b = strip_ansi_for_compare(lines[later]);
        if !(is_shell_prompt_line(&a) && a.trim_end() == b.trim_end()) {
            break;
        }
        let ended_nl = buf.ends_with('\n');
        let mut new_buf = String::with_capacity(buf.len());
        for (i, line) in lines.iter().enumerate() {
            if i == later {
                continue;
            }
            new_buf.push_str(line);
            new_buf.push('\n');
        }
        if !ended_nl && new_buf.ends_with('\n') {
            new_buf.pop();
        }
        *buf = new_buf;
    }
}

fn xterm_js_call(expr: &str) {
    let _ = document::eval(&format!(
        r#"(() => {{ try {{ {expr} }} catch (_) {{}} }})()"#
    ));
}

fn xterm_write(data: &str) {
    if data.is_empty() {
        return;
    }
    let payload = serde_json::to_string(data).unwrap_or_else(|_| "\"\"".into());
    xterm_js_call(&format!(
        "window.__acXterm && window.__acXterm.write({payload})"
    ));
}

fn xterm_reset(scrollback: &str) {
    let mut cleaned = scrollback.to_string();
    collapse_trailing_dup_prompts(&mut cleaned);
    let payload = serde_json::to_string(&cleaned).unwrap_or_else(|_| "\"\"".into());
    xterm_js_call(&format!(
        "window.__acXterm && window.__acXterm.reset({payload})"
    ));
}

fn xterm_focus() {
    xterm_js_call("window.__acXterm && window.__acXterm.focus()");
}

fn xterm_fit() {
    xterm_js_call("window.__acXterm && window.__acXterm.fit()");
}

fn xterm_ensure() {
    xterm_js_call("window.__acXtermEnsure && window.__acXtermEnsure()");
}

fn xterm_clear_selection() {
    xterm_js_call(
        "window.__acXterm && window.__acXterm.clearSelection && window.__acXterm.clearSelection()",
    );
}

/// 启动交互式登录 shell；原始输出写入 scrollback，若为当前会话则推给 xterm。
pub fn spawn_interactive_shell(
    session_id: u64,
    active_id: Signal<Option<u64>>,
    mut scrollback: Signal<String>,
    rows: u16,
    cols: u16,
) -> Result<TerminalWriter, String> {
    let rows = rows.max(2);
    let cols = cols.max(2);
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("打开 PTY 失败：{e}"))?;

    let shell = default_shell();
    let mut cmd = CommandBuilder::new(&shell);
    if !cfg!(windows) {
        cmd.arg("-l");
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        // 避免 zsh 在无完整行末尾画孤立的 `%`（PROMPT_EOL_MARK）。
        cmd.env("PROMPT_EOL_MARK", "");
        // GUI 启动时常丢 Homebrew；与 Agent exec_bash 共用增强 PATH。
        cmd.env("PATH", shared::tools::file::enriched_path());
    }
    if let Some(cwd) = shell_initial_cwd() {
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

    let master: Arc<Mutex<Box<dyn MasterPty + Send>>> = Arc::new(Mutex::new(pair.master));
    let last_size = Arc::new(Mutex::new((rows, cols)));

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
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let existing = scrollback.peek().clone();
            let filtered = filter_duplicate_prompt_chunk(&existing, &text);
            if filtered.is_empty() {
                continue;
            }
            scrollback.with_mut(|buf| {
                append_pty_scrollback(buf, &filtered);
                collapse_trailing_dup_prompts(buf);
            });
            if active_id() == Some(session_id) {
                xterm_write(&filtered);
            }
        }
        let bye = "\r\n[shell 已退出]\r\n";
        scrollback.with_mut(|buf| buf.push_str(bye));
        if active_id() == Some(session_id) {
            xterm_write(bye);
        }
    });

    // 子进程随 master/writer 生命周期；勿 forget master（resize 需要）。
    std::mem::forget(_child);

    Ok(TerminalWriter {
        tx,
        master,
        last_size,
    })
}

fn install_xterm_bridge_js(css_href: &str, js_src: &str, fit_src: &str) -> String {
    format!(
        r#"(async () => {{
  const CSS_HREF = {css};
  const JS_SRC = {js};
  const FIT_SRC = {fit};

  function loadCss(href) {{
    if (document.querySelector('link[data-ac-xterm-css]')) return;
    const l = document.createElement('link');
    l.rel = 'stylesheet';
    l.href = href;
    l.setAttribute('data-ac-xterm-css', '1');
    document.head.appendChild(l);
  }}

  function loadScript(src, marker) {{
    return new Promise((resolve, reject) => {{
      if (document.querySelector('script[' + marker + ']')) {{
        resolve();
        return;
      }}
      const s = document.createElement('script');
      s.src = src;
      s.async = false;
      s.setAttribute(marker.replace(/[\[\]]/g, '').replace('data-', 'data-'), '1');
      s.setAttribute(marker, '1');
      s.onload = () => resolve();
      s.onerror = () => reject(new Error('load failed: ' + src));
      document.head.appendChild(s);
    }});
  }}

  loadCss(CSS_HREF);
  await loadScript(JS_SRC, 'data-ac-xterm-js');
  await loadScript(FIT_SRC, 'data-ac-xterm-fit');

  const TermCtor = window.Terminal;
  const FitCtor = (window.FitAddon && window.FitAddon.FitAddon) || window.FitAddon;
  if (!TermCtor || !FitCtor) {{
    try {{ dioxus.send({{ kind: 'error', data: 'xterm 未能加载' }}); }} catch (_) {{}}
    return;
  }}

  // 每次新桥安装必须换新实例：旧 term.onData 绑的是已失效的 dioxus 通道，
  // 复用会导致「能显示、不能输入」。
  try {{
    if (window.__acXtermRo) {{ window.__acXtermRo.disconnect(); }}
  }} catch (_) {{}}
  window.__acXtermRo = null;
  try {{
    if (window.__acXterm && window.__acXterm.term) {{
      window.__acXterm.term.dispose();
    }}
  }} catch (_) {{}}
  window.__acXterm = null;

  const BRIDGE_MARK = 9;

  function ensure() {{
    const host = document.getElementById('ac-xterm-host');
    if (!host) return null;
    if (window.__acXterm && window.__acXterm.term) {{
      if (window.__acXterm.term.__acSelBound === BRIDGE_MARK
          && window.__acXterm.__acBridgeGen === BRIDGE_MARK) {{
        if (!host.contains(window.__acXterm.term.element)) {{
          host.innerHTML = '';
          host.appendChild(window.__acXterm.term.element);
        }}
        try {{
          // 同一 eval 内 ping/重挂：把 onData 重新绑到当前 dioxus 通道。
          if (window.__acXterm.rebindChannel) window.__acXterm.rebindChannel();
          if (window.__acXterm.applyTheme) window.__acXterm.applyTheme();
        }} catch (_) {{}}
        return window.__acXterm;
      }}
      try {{ window.__acXterm.term.dispose(); }} catch (_) {{}}
      window.__acXterm = null;
      host.innerHTML = '';
    }}
    host.innerHTML = '';
    function readIsDark() {{
      try {{
        return (document.documentElement.getAttribute('data-theme') || '') === 'dark';
      }} catch (_) {{
        return false;
      }}
    }}
    function themeFor(dark) {{
      if (dark) {{
        return {{
          background: '#0e0e10',
          foreground: '#cccccc',
          cursor: '#cccccc',
          cursorAccent: '#0e0e10',
          selectionBackground: '#264f78',
          selectionForeground: '#ffffff',
        }};
      }}
      // Light: --text-primary / --secondary-color-1 (#2a2f37), not near-black #1f2328.
      return {{
        background: '#ffffff',
        foreground: '#2a2f37',
        cursor: '#2a2f37',
        cursorAccent: '#ffffff',
        selectionBackground: '#b3d4fc',
        selectionForeground: '#2a2f37',
      }};
    }}
    const isDark = readIsDark();
    const term = new TermCtor({{
      cursorBlink: true,
      fontSize: 12,
      fontFamily: 'SF Mono, Menlo, Monaco, ui-monospace, Cascadia Mono, Consolas, monospace',
      theme: themeFor(isDark),
      fontWeight: 'normal',
      // Light: PTY SGR 1 (just/ls) should not render heavier than the prompt.
      fontWeightBold: isDark ? 'bold' : 'normal',
      allowProposedApi: true,
      scrollback: 5000,
    }});
    const fitAddon = new FitCtor();
    term.loadAddon(fitAddon);
    term.open(host);
    try {{ fitAddon.fit(); }} catch (_) {{}}

    let dataDisp = null;
    let resizeDisp = null;
    function rebindChannel() {{
      try {{ if (dataDisp) dataDisp.dispose(); }} catch (_) {{}}
      try {{ if (resizeDisp) resizeDisp.dispose(); }} catch (_) {{}}
      dataDisp = term.onData((data) => {{
        try {{ dioxus.send({{ kind: 'data', data: String(data) }}); }} catch (_) {{}}
      }});
      resizeDisp = term.onResize(({{cols, rows}}) => {{
        try {{ dioxus.send({{ kind: 'resize', cols, rows }}); }} catch (_) {{}}
      }});
    }}
    rebindChannel();

    const api = {{
      term,
      fitAddon,
      __acBridgeGen: BRIDGE_MARK,
      rebindChannel,
      write(data) {{ try {{ term.write(data); }} catch (_) {{}} }},
      reset(data) {{
        try {{
          term.reset();
          if (data) term.write(data);
        }} catch (_) {{}}
      }},
      focus() {{ try {{ term.focus(); }} catch (_) {{}} }},
      fit() {{
        try {{
          if (!host || host.offsetWidth < 16 || host.offsetHeight < 16) return;
          fitAddon.fit();
          const cols = term.cols | 0;
          const rows = term.rows | 0;
          try {{ dioxus.send({{ kind: 'resize', cols, rows }}); }} catch (_) {{}}
        }} catch (_) {{}}
      }},
      applyTheme() {{
        try {{
          const dark = readIsDark();
          term.options.theme = themeFor(dark);
          term.options.fontWeightBold = dark ? 'bold' : 'normal';
          if (host) {{
            host.style.background = dark ? '#0e0e10' : '#ffffff';
          }}
        }} catch (_) {{}}
      }},
    }};
    window.__acXterm = api;
    api.applyTheme();

    function lineStr(buf, y) {{
      const line = buf.getLine(y);
      return line ? line.translateToString(true) : '';
    }}

    function trimLine(y) {{
      return lineStr(term.buffer.active, y).replace(/\s+$/g, '');
    }}

    function isPromptLine(t) {{
      const s = (t || '').trimEnd();
      if (!s) return false;
      return /[%$#❯]\s*$/.test(s) || /\S+@\S+\s+.*[%$#❯]\s*$/.test(s);
    }}

    function isErrorStart(t) {{
      const s = (t || '').trim();
      if (!s) return false;
      return /^(error:|error\[|Error:|ERROR|panic!|thread '.+' panicked|Traceback \(most recent call last\):|FAILED|FAILURES|错误[:：]|失败[:：])/i.test(s)
        || /^cargo:error=/i.test(s);
    }}

    function isErrorRelated(t) {{
      const s = (t || '').trimEnd();
      if (!s) return true; // 块内允许空行
      if (isErrorStart(s)) return true;
      if (isPromptLine(s)) return false;
      return /^(Caused by:|note:|warning:|help:|-->|\.\.\.|---\s*(stdout|stderr)|Process finished|exit status)/i.test(s.trim())
        || /^\s+/.test(s)
        || /^[=-]{{3,}}/.test(s.trim())
        || /^\|/.test(s.trim())
        || /^`/.test(s.trim());
    }}

    function notifySelection() {{
      const text = (term.getSelection && term.getSelection()) || '';
      try {{ dioxus.send({{ kind: 'selection', data: String(text) }}); }} catch (_) {{}}
    }}

    function bufferYFromEvent(e) {{
      try {{
        const buf = term.buffer.active;
        const core = term._core;
        const cellH = core && core._renderService && core._renderService.dimensions
          && core._renderService.dimensions.css
          && core._renderService.dimensions.css.cell
          && core._renderService.dimensions.css.cell.height;
        const target = term.screenElement || term.element || host;
        const rect = target.getBoundingClientRect();
        if (!cellH || cellH <= 0) return null;
        const row = Math.floor((e.clientY - rect.top) / cellH);
        if (row < 0 || row >= term.rows) return null;
        return Math.max(0, Math.min(buf.length - 1, buf.viewportY + row));
      }} catch (_) {{
        return null;
      }}
    }}

    function findNearestContentY(buf, y) {{
      if (y < 0) y = 0;
      if (y >= buf.length) y = buf.length - 1;
      if (trimLine(y)) return y;
      for (let d = 1; d < buf.length; d++) {{
        if (y - d >= 0 && trimLine(y - d)) return y - d;
        if (y + d < buf.length && trimLine(y + d)) return y + d;
      }}
      return y;
    }}

    function findErrorStart(buf, anchor) {{
      // 从锚点向上找最近的 error 起始行（允许穿过空行）
      for (let y = anchor; y >= 0 && anchor - y < 120; y--) {{
        const t = trimLine(y);
        if (isErrorStart(t)) return y;
        if (y < anchor && isPromptLine(t)) break;
      }}
      // 回退：缓冲区中最后一次 error 起始
      for (let y = buf.length - 1; y >= 0; y--) {{
        if (isErrorStart(trimLine(y))) return y;
      }}
      return -1;
    }}

    function expandErrorBlock(start) {{
      const buf = term.buffer.active;
      let end = start;
      let blankRun = 0;
      for (let y = start + 1; y < buf.length; y++) {{
        const t = trimLine(y);
        if (isPromptLine(t)) break;
        if (!t) {{
          blankRun += 1;
          // 单空行仍视为块内；连续两空行结束
          if (blankRun >= 2) break;
          continue;
        }}
        blankRun = 0;
        // 新的另一段 error（且已离开当前块）则停
        if (isErrorStart(t) && y > start) break;
        if (!isErrorRelated(t) && !isErrorStart(t)) break;
        end = y;
      }}
      // 去掉首尾空行，避免顶部多出一行高亮
      while (start < end && !trimLine(start)) start += 1;
      while (end > start && !trimLine(end)) end -= 1;
      return {{ start, end }};
    }}

    function expandErrorSelection(e) {{
      try {{
        const buf = term.buffer.active;
        if (!buf || buf.length <= 0) {{
          notifySelection();
          return;
        }}
        let anchor = null;
        if (e) anchor = bufferYFromEvent(e);
        if (anchor == null) {{
          const pos = term.getSelectionPosition && term.getSelectionPosition();
          if (pos && pos.start && pos.end) {{
            anchor = Math.min(pos.start.y, pos.end.y);
          }} else {{
            anchor = buf.baseY + buf.cursorY;
          }}
        }}
        anchor = findNearestContentY(buf, anchor);
        const errorStart = findErrorStart(buf, anchor);
        if (errorStart < 0) {{
          notifySelection();
          return;
        }}
        const range = expandErrorBlock(errorStart);
        if (term.selectLines) {{
          term.selectLines(range.start, range.end);
        }}
      }} catch (_) {{}}
      notifySelection();
    }}

    function bindSelection() {{
      if (term.__acSelBound === BRIDGE_MARK) return;
      term.__acSelBound = BRIDGE_MARK;
      if (term.onSelectionChange) {{
        term.onSelectionChange(() => notifySelection());
      }}
      const el = term.element || host;
      el.addEventListener('dblclick', (e) => {{
        // 阻止默认「选词」，改为选整段错误块
        e.preventDefault();
        e.stopPropagation();
        setTimeout(() => expandErrorSelection(e), 0);
      }}, true);
    }}

    bindSelection();

    api.clearSelection = () => {{
      try {{
        if (term.clearSelection) term.clearSelection();
        notifySelection();
      }} catch (_) {{}}
    }};
    api.bindSelection = bindSelection;

    const ro = new ResizeObserver(() => {{
      try {{ api.fit(); }} catch (_) {{}}
    }});
    ro.observe(host);
    window.__acXtermRo = ro;

    if (!window.__acXtermThemeObs) {{
      try {{
        const obs = new MutationObserver(() => {{
          try {{
            if (window.__acXterm && window.__acXterm.applyTheme) {{
              window.__acXterm.applyTheme();
            }}
          }} catch (_) {{}}
        }});
        obs.observe(document.documentElement, {{ attributes: true, attributeFilter: ['data-theme'] }});
        window.__acXtermThemeObs = obs;
      }} catch (_) {{}}
    }}

    try {{ dioxus.send({{ kind: 'ready' }}); }} catch (_) {{}}
    try {{ api.fit(); api.focus(); }} catch (_) {{}}
    return api;
  }}

  window.__acXtermEnsure = ensure;

  // 宿主可能晚于脚本就绪（先画「点击 +」再创建会话）；持续重试直到挂上。
  for (let i = 0; i < 120; i++) {{
    if (ensure()) break;
    await new Promise((r) => setTimeout(r, 50));
  }}

  while (true) {{
    let msg;
    try {{ msg = await dioxus.recv(); }} catch (_) {{ break; }}
    if (!msg || typeof msg !== 'object') continue;
    if (msg.kind === 'ping') {{
      ensure();
    }}
  }}
}})()"#,
        css = serde_json::to_string(css_href).unwrap_or_else(|_| "\"\"".into()),
        js = serde_json::to_string(js_src).unwrap_or_else(|_| "\"\"".into()),
        fit = serde_json::to_string(fit_src).unwrap_or_else(|_| "\"\"".into()),
    )
}

/// 桌面终端面板：xterm.js 模拟器 + 多会话 rail。
#[component]
pub fn DesktopTerminal(
    mut show_terminal: Signal<bool>,
    /// 外部请求：打开终端后 `cd` 到该目录（一次性消费）。
    mut pending_cd: Signal<Option<String>>,
    /// 外部请求：在命名会话（如 `pusa`）中执行命令。
    mut pending_run: Signal<Option<TerminalRunRequest>>,
    on_resize_start: EventHandler<MouseEvent>,
    is_maximized: bool,
    on_maximize: EventHandler<()>,
    on_minimize: EventHandler<()>,
    /// 将终端选区加入 Pusa Chat。
    on_add_to_pusa: EventHandler<String>,
) -> Element {
    let mut sessions = use_signal(Vec::<TerminalSession>::new);
    let mut active_id = use_signal(|| None::<u64>);
    let next_id = use_signal(|| 1u64);
    let mut bridge_version = use_signal(|| 0u32);
    let mut selection_text = use_signal(String::new);
    let mut xterm_size = use_signal(|| None::<(u16, u16)>);

    // 安装 xterm 桥：键盘 → PTY，resize → PTY，选区 → Add to Pusa。
    use_effect(move || {
        if bridge_version() >= XTERM_BRIDGE_VERSION {
            return;
        }
        bridge_version.set(XTERM_BRIDGE_VERSION);
        let css = XTERM_CSS.to_string();
        let js = XTERM_JS.to_string();
        let fit = XTERM_FIT_JS.to_string();
        spawn(async move {
            let mut eval = document::eval(&install_xterm_bridge_js(&css, &js, &fit));
            loop {
                let Ok(ev) = eval.recv::<TermBridgeEvent>().await else {
                    break;
                };
                match ev.kind.as_str() {
                    "ready" => {
                        xterm_fit();
                        xterm_focus();
                    }
                    "data" => {
                        let Some(id) = active_id() else {
                            continue;
                        };
                        let writer = sessions
                            .read()
                            .iter()
                            .find(|s| s.id == id)
                            .and_then(|s| (s.pty)());
                        if let Some(writer) = writer {
                            writer.write_str(&ev.data);
                        }
                    }
                    "resize" => {
                        if ev.cols < 2 || ev.rows < 2 {
                            continue;
                        }
                        xterm_size.set(Some((ev.rows, ev.cols)));
                        attach_pending_ptys(sessions, active_id, ev.rows, ev.cols);
                        let Some(id) = active_id() else {
                            continue;
                        };
                        let writer = sessions
                            .read()
                            .iter()
                            .find(|s| s.id == id)
                            .and_then(|s| (s.pty)());
                        if let Some(writer) = writer {
                            writer.resize(ev.rows, ev.cols);
                        }
                    }
                    "selection" => {
                        selection_text.set(ev.data);
                    }
                    "error" => {
                        eprintln!("[ac-terminal] {}", ev.data);
                    }
                    _ => {}
                }
            }
        });
    });

    // 首次打开面板：先 fit 拿到真实尺寸，再启动 shell（避免默认 24×100 后再 WINCH 重打提示符）。
    use_effect(move || {
        if !show_terminal() {
            return;
        }
        if pending_run().is_some() {
            xterm_ensure();
            xterm_fit();
            return;
        }
        if sessions.read().is_empty() {
            push_new_session(sessions, active_id, next_id);
        }
        xterm_ensure();
        xterm_fit();
        xterm_focus();
    });

    // 切换活动会话：先回放 scrollback；尺寸已知则立刻启动 PTY（peek 避免尺寸变化时再次 reset）。
    use_effect(move || {
        let Some(id) = active_id() else {
            return;
        };
        let sb = sessions
            .peek()
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.scrollback.peek().clone())
            .unwrap_or_default();
        xterm_ensure();
        xterm_reset(&sb);
        xterm_focus();
        if let Some((rows, cols)) = *xterm_size.peek() {
            attach_pending_ptys(sessions, active_id, rows, cols);
        }
    });

    // xterm 量到真实尺寸后再 spawn，避免默认尺寸 → WINCH 重打提示符。
    use_effect(move || {
        let Some((rows, cols)) = xterm_size() else {
            return;
        };
        attach_pending_ptys(sessions, active_id, rows, cols);
    });

    // 消费 pending_cd：确保有会话后写入 `cd …`。
    use_effect(move || {
        let Some(dir) = pending_cd() else {
            return;
        };
        if !show_terminal() {
            return;
        }
        if sessions.read().is_empty() {
            push_new_session(sessions, active_id, next_id);
        }
        let Some(id) = active_id() else {
            return;
        };
        let writer = sessions
            .read()
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| (s.pty)());
        if let Some(writer) = writer {
            writer.write_str(&format!("cd {}\n", shell_quote(&dir)));
            pending_cd.set(None);
            xterm_focus();
        }
    });

    // 消费 pending_run：复用/创建命名会话，cd 后执行命令。
    use_effect(move || {
        let Some(req) = pending_run() else {
            return;
        };
        if !show_terminal() {
            return;
        }
        let id = ensure_named_session(sessions, active_id, next_id, &req.session_name);
        let writer = sessions
            .read()
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| (s.pty)());
        let Some(writer) = writer else {
            return;
        };
        let mut line = String::new();
        if let Some(cwd) = req.cwd.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
            line.push_str("cd ");
            line.push_str(&shell_quote(cwd));
            line.push_str(" && ");
        }
        line.push_str(&crate::desktop::plugins::unwrap_shell_c_command(req.command.trim()));
        if line.trim().is_empty() {
            pending_run.set(None);
            return;
        }
        line.push('\n');
        writer.write_str(&line);
        pending_run.set(None);
        xterm_focus();
    });

    let active_err = active_id().and_then(|id| {
        sessions
            .read()
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| (s.err_msg)())
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
                    if !selection_text().trim().is_empty() {
                        button {
                            r#type: "button",
                            class: "ac-terminal-add-pusa-btn",
                            title: "Add to Pusa",
                            aria_label: "Add to Pusa",
                            onclick: move |_| {
                                let text = selection_text();
                                if text.trim().is_empty() {
                                    return;
                                }
                                on_add_to_pusa.call(text);
                                selection_text.set(String::new());
                                xterm_clear_selection();
                            },
                            "Add to Pusa"
                        }
                    }
                    if is_maximized {
                        button {
                            r#type: "button",
                            class: "ac-terminal-toolbar-btn",
                            title: "最小化终端",
                            aria_label: "最小化终端",
                            onclick: move |_| {
                                on_minimize.call(());
                                spawn(async move {
                                    tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                                    xterm_fit();
                                });
                            },
                            Icon {
                                icon: LdChevronDown,
                                width: 14,
                                height: 14,
                                fill: "currentColor",
                            }
                        }
                    } else {
                        button {
                            r#type: "button",
                            class: "ac-terminal-toolbar-btn",
                            title: "最大化终端",
                            aria_label: "最大化终端",
                            onclick: move |_| {
                                on_maximize.call(());
                                spawn(async move {
                                    tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                                    xterm_fit();
                                });
                            },
                            Icon {
                                icon: LdChevronUp,
                                width: 14,
                                height: 14,
                                fill: "currentColor",
                            }
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
                div { class: "ac-terminal-body ac-terminal-body--xterm",
                    if let Some(err) = active_err {
                        div { class: "ac-terminal-error", "{err}" }
                    }
                    div {
                        id: "ac-xterm-host",
                        class: "ac-xterm-host",
                        onmounted: move |_| {
                            xterm_ensure();
                            xterm_fit();
                            xterm_focus();
                        },
                        onclick: move |_| {
                            xterm_focus();
                        },
                    }
                }
                div { class: "ac-terminal-session-rail",
                    {
                        let list = sessions();
                        rsx! {
                            for session in list.iter().copied() {
                                {
                                    let id = session.id;
                                    let label = {
                                        let n = (session.name)();
                                        if n.is_empty() {
                                            format!("终端 {id}")
                                        } else {
                                            n
                                        }
                                    };
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
                                            title: "{label}",
                                            aria_label: "切换到 {label}",
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
                        let list = sessions();
                        let can_close = list.len() > 1;
                        if let Some(id) = active_id() {
                            if can_close {
                                rsx! {
                                    button {
                                        r#type: "button",
                                        class: "ac-terminal-session-btn",
                                        title: "关闭当前终端",
                                        aria_label: "关闭当前终端",
                                        onclick: move |_| {
                                            let mut next_focus = None;
                                            sessions.with_mut(|list| {
                                                if let Some(idx) = list.iter().position(|s| s.id == id) {
                                                    list[idx].request_exit();
                                                    list.remove(idx);
                                                    next_focus = if idx < list.len() {
                                                        Some(list[idx].id)
                                                    } else {
                                                        list.last().map(|s| s.id)
                                                    };
                                                }
                                            });
                                            active_id.set(next_focus);
                                        },
                                        Icon {
                                            icon: LdX,
                                            width: 14,
                                            height: 14,
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
        append_pty_scrollback, collapse_trailing_dup_prompts, filter_duplicate_prompt_chunk,
    };

    #[test]
    fn clear_sequence_resets_scrollback() {
        let mut buf = String::from("old output\n");
        append_pty_scrollback(&mut buf, "keep\u{1b}[2Jfresh");
        assert!(buf.contains("fresh"));
        assert!(!buf.contains("old output"));
    }

    #[test]
    fn normal_append_keeps_ansi() {
        let mut buf = String::new();
        append_pty_scrollback(&mut buf, "\u{1b}[31mred\u{1b}[0m");
        assert_eq!(buf, "\u{1b}[31mred\u{1b}[0m");
    }

    #[test]
    fn filters_winch_duplicate_prompt() {
        let existing = "mac@Macbook English %";
        let chunk = "\u{1b}[0mmac@Macbook English %";
        assert!(filter_duplicate_prompt_chunk(existing, chunk).is_empty());
    }

    #[test]
    fn filters_winch_prompt_after_newline() {
        let existing = "mac@Macbook English %";
        let chunk = "\r\nmac@Macbook English %";
        assert!(filter_duplicate_prompt_chunk(existing, chunk).is_empty());
    }

    #[test]
    fn filters_winch_repeated_prompt_chunk() {
        let existing = "mac@Macbook English %";
        let chunk = "mac@Macbook English %\nmac@Macbook English %";
        assert!(filter_duplicate_prompt_chunk(existing, chunk).is_empty());
    }

    #[test]
    fn keeps_real_command_output() {
        let existing = "mac@Macbook English %";
        let chunk = "hello\nmac@Macbook English %";
        assert!(!filter_duplicate_prompt_chunk(existing, chunk).is_empty());
    }

    #[test]
    fn collapses_two_trailing_prompts() {
        let mut buf = String::from("mac@Macbook English %\nmac@Macbook English %\n");
        collapse_trailing_dup_prompts(&mut buf);
        assert_eq!(buf.matches("mac@Macbook English %").count(), 1);
    }
}
