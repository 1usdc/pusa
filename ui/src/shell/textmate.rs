//! 已安装 VS Code / Open VSX 扩展的 TextMate 高亮。
//!
//! 启动时扫描 `extensions/` 的语法贡献；打开文件时按扩展名、文件名或语言 id
//! 选用 grammar，并把 scope 映射到现有 `ac-syn-*` class。
//!
//! 支持 JSON / JSONC / plist 里的 `match`、`begin`/`end`、`include`（`#repo` / `$self`）、
//! `captures` 与 `repository`。正则引擎是 `fancy-regex`（前瞻、后顾、反向引用）：
//! 按当前匹配位置处理 `\G`、行首 `^` 与词边界。Oniguruma 独占量词会降级；
//! 无法编译的规则跳过。
//! 不支持：`while`、`injectTo`、跨 grammar 的 `include: "source.js"`、capture 内嵌 patterns。
//! 无可用 grammar、文件过大或正则无法编译时返回 `None`，由 `syntax.rs` 回退内置分词。

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Once, OnceLock};
use std::time::{Duration, Instant, UNIX_EPOCH};

use serde_json::Value;

use crate::desktop::files::extension_root;
use crate::desktop::plugins::list_installed_grammar_contributes;

const MAX_GRAMMAR_BYTES: u64 = 1_500_000;
const MAX_PATTERN_CHARS: usize = 24_000;
const MAX_STACK: usize = 48;
const INCLUDE_DEPTH: u32 = 24;
const INDEX_TTL: Duration = Duration::from_secs(2);
/// 前文长度：让 `\b` 和较短的 lookbehind 能看到光标左侧。
const CTX_CHARS: usize = 32;

struct RePair {
    bol: Option<fancy_regex::Regex>,
    mid: Option<fancy_regex::Regex>,
}

struct Cache {
    by_ext: HashMap<String, PathBuf>,
    by_filename: HashMap<String, PathBuf>,
    by_lang: HashMap<String, PathBuf>,
    loaded: HashMap<PathBuf, Arc<Loaded>>,
    root: PathBuf,
    stamp: u64,
    checked: Instant,
    ready: bool,
}

enum Loaded {
    Failed,
    Grammar(Arc<Grammar>),
}

struct Grammar {
    scope_name: String,
    rules: Vec<Rule>,
    repo: HashMap<String, Vec<usize>>,
    top: Vec<usize>,
    any_regex_ok: AtomicBool,
}

enum Rule {
    Include(IncludeKind),
    Match {
        pattern: String,
        name: Option<String>,
        captures: Vec<(usize, String)>,
        re: OnceLock<RePair>,
    },
    BeginEnd {
        begin: String,
        end: Option<String>,
        name: Option<String>,
        content_name: Option<String>,
        begin_captures: Vec<(usize, String)>,
        end_captures: Vec<(usize, String)>,
        patterns: Vec<usize>,
        apply_end_last: bool,
        /// 没有 `end`（含仅有 `while`）时，行末退出，避免状态吞掉后续全文。
        pop_at_eol: bool,
        begin_re: OnceLock<RePair>,
        end_re: OnceLock<RePair>,
    },
}

enum IncludeKind {
    SelfRef,
    Base,
    Repo(String),
    /// `source.js` 这类外部 grammar：当前不展开。
    External,
}

#[derive(Clone, Copy)]
enum PatternRef {
    Top,
    Rule(usize),
}

struct Frame {
    scope: Option<String>,
    content_scope: Option<String>,
    content_active: bool,
    end_rule: Option<usize>,
    patterns: PatternRef,
    apply_end_last: bool,
    pop_at_eol: bool,
}

struct Caps {
    groups: Vec<Option<(usize, usize)>>,
}

impl Caps {
    fn start(&self) -> usize {
        self.groups.first().copied().flatten().map(|p| p.0).unwrap_or(0)
    }
    fn end(&self) -> usize {
        self.groups.first().copied().flatten().map(|p| p.1).unwrap_or(0)
    }
}

enum Hit {
    Token { rule: usize, caps: Caps },
    Begin { rule: usize, caps: Caps },
}

struct Emitter {
    out: String,
    buf: String,
    class: Option<&'static str>,
    open: bool,
}

/// 后台建立语法索引。真正编译正则发生在首次高亮该语言时。
pub fn preload_installed_grammars() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = std::thread::Builder::new()
            .name("pusa-grammars".into())
            .spawn(|| {
                let mut guard = lock();
                guard.ensure_index();
            });
    });
}

/// 用已安装 grammar 高亮。返回 `None` 表示应回退内置分词。
pub fn try_highlight(source: &str, path: Option<&str>, lang: &str) -> Option<String> {
    let grammar = {
        let mut guard = lock();
        guard.ensure_index();
        let path = guard.select(path, lang)?;
        guard.load(&path)?
    };
    if !grammar.has_color_rules() {
        return None;
    }
    let html = paint(&grammar, source);
    if grammar.any_regex_ok.load(Ordering::Relaxed) {
        Some(html)
    } else {
        None
    }
}

fn lock() -> std::sync::MutexGuard<'static, Cache> {
    cache_mutex().lock().unwrap_or_else(|e| e.into_inner())
}

fn cache_mutex() -> &'static Mutex<Cache> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(Cache::empty()))
}

impl Cache {
    fn empty() -> Self {
        Self {
            by_ext: HashMap::new(),
            by_filename: HashMap::new(),
            by_lang: HashMap::new(),
            loaded: HashMap::new(),
            root: PathBuf::new(),
            stamp: 0,
            checked: Instant::now() - INDEX_TTL,
            ready: false,
        }
    }

    fn ensure_index(&mut self) {
        let root = extension_root();
        let now = Instant::now();
        if self.ready && self.root == root && now.duration_since(self.checked) < INDEX_TTL {
            return;
        }
        self.checked = now;
        let stamp = dir_stamp(&root);
        if self.ready && self.root == root && self.stamp == stamp {
            return;
        }
        self.root = root;
        self.stamp = stamp;
        self.by_ext.clear();
        self.by_filename.clear();
        self.by_lang.clear();
        self.loaded.clear();
        self.ready = true;

        for pack in list_installed_grammar_contributes() {
            let mut lang_path: HashMap<String, PathBuf> = HashMap::new();
            for grammar in &pack.contributes.grammars {
                let Some(rel) = grammar.path.as_deref() else {
                    continue;
                };
                let Some(path) = safe_grammar_path(&pack.dir, rel) else {
                    continue;
                };
                if !usable_primary_grammar(&path) {
                    continue;
                }
                if let Some(lang) = grammar.language.as_deref() {
                    let key = lang.trim().to_ascii_lowercase();
                    if key.is_empty() {
                        continue;
                    }
                    lang_path.entry(key.clone()).or_insert_with(|| path.clone());
                    self.by_lang.entry(key).or_insert(path);
                }
            }
            for lang in &pack.contributes.languages {
                let Some(id) = lang.id.as_deref() else {
                    continue;
                };
                let Some(path) = lang_path.get(&id.trim().to_ascii_lowercase()) else {
                    continue;
                };
                for ext in &lang.extensions {
                    let ext = normalize_ext(ext);
                    if !ext.is_empty() {
                        self.by_ext.entry(ext).or_insert_with(|| path.clone());
                    }
                }
                for name in &lang.filenames {
                    let name = name.trim().to_ascii_lowercase();
                    if !name.is_empty() {
                        self.by_filename.entry(name).or_insert_with(|| path.clone());
                    }
                }
            }
        }
    }

    fn select(&self, path: Option<&str>, lang: &str) -> Option<PathBuf> {
        if let Some(path) = path {
            if let Some(name) = file_name_lower(path) {
                if let Some(p) = self.by_filename.get(&name) {
                    return Some(p.clone());
                }
            }
            if let Some(ext) = file_ext_lower(path) {
                if let Some(p) = self.by_ext.get(&ext) {
                    return Some(p.clone());
                }
            }
        }
        let lang = lang.trim().to_ascii_lowercase();
        if lang.is_empty() || lang == "plain" {
            return None;
        }
        self.by_lang.get(&lang).cloned()
    }

    fn load(&mut self, path: &Path) -> Option<Arc<Grammar>> {
        if let Some(hit) = self.loaded.get(path) {
            return match hit.as_ref() {
                Loaded::Failed => None,
                Loaded::Grammar(g) => Some(Arc::clone(g)),
            };
        }
        let loaded = match load_grammar_file(path) {
            Some(g) => Loaded::Grammar(Arc::new(g)),
            None => Loaded::Failed,
        };
        self.loaded.insert(path.to_path_buf(), Arc::new(loaded));
        match self.loaded.get(path)?.as_ref() {
            Loaded::Failed => None,
            Loaded::Grammar(g) => Some(Arc::clone(g)),
        }
    }
}

fn dir_stamp(root: &Path) -> u64 {
    let mut n = 0u64;
    let mut max_m = 0u64;
    let Ok(rd) = fs::read_dir(root) else {
        return 0;
    };
    for entry in rd.flatten() {
        n = n.wrapping_add(1);
        let modified = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .or_else(|| {
                entry
                    .path()
                    .join("package.json")
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
            });
        if let Some(modified) = modified {
            if let Ok(d) = modified.duration_since(UNIX_EPOCH) {
                max_m = max_m.max(d.as_secs());
            }
        }
    }
    n ^ max_m.wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn normalize_ext(ext: &str) -> String {
    ext.trim().trim_start_matches('.').to_ascii_lowercase()
}

fn file_ext_lower(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| !e.is_empty())
}

fn file_name_lower(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_ascii_lowercase())
        .filter(|n| !n.is_empty())
}

fn safe_grammar_path(dir: &Path, rel: &str) -> Option<PathBuf> {
    let rel = rel.trim().trim_start_matches("./");
    if rel.is_empty() || rel.starts_with('/') || rel.starts_with('\\') {
        return None;
    }
    if rel.split(['/', '\\']).any(|p| p == "..") {
        return None;
    }
    let path = dir.join(rel);
    path.is_file().then_some(path)
}

fn usable_primary_grammar(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if meta.len() == 0 || meta.len() > MAX_GRAMMAR_BYTES {
        return false;
    }
    !is_injection_grammar(path)
}

fn is_injection_grammar(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 3072];
    let n = file.read(&mut buf).unwrap_or(0);
    let head = String::from_utf8_lossy(&buf[..n]).to_ascii_lowercase();
    head.contains("\"injectto\"") || head.contains("<key>injectto</key>")
}

fn load_grammar_file(path: &Path) -> Option<Grammar> {
    let value = load_grammar_value(path)?;
    compile_value(&value)
}

fn load_grammar_value(path: &Path) -> Option<Value> {
    let meta = fs::metadata(path).ok()?;
    if meta.len() == 0 || meta.len() > MAX_GRAMMAR_BYTES {
        return None;
    }
    if let Ok(text) = fs::read_to_string(path) {
        let text = text.trim_start_matches('\u{feff}').trim();
        if text.starts_with('{') || text.starts_with('[') || text.starts_with('/') {
            if let Some(v) = parse_json_lenient(text) {
                return Some(v);
            }
        }
    }
    let value = plist::Value::from_file(path).ok()?;
    Some(plist_to_json(value))
}

fn parse_json_lenient(text: &str) -> Option<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        return Some(v);
    }
    let cleaned = sanitize_jsonc(text);
    serde_json::from_str(&cleaned).ok()
}

fn sanitize_jsonc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_str = false;
    let mut escape = false;
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                out.push(c);
            }
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                for n in chars.by_ref() {
                    if n == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                while let Some(n) = chars.next() {
                    if n == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                }
            }
            ',' => {
                let mut look = chars.clone();
                while let Some(n) = look.peek().copied() {
                    if n.is_whitespace() {
                        look.next();
                        continue;
                    }
                    break;
                }
                if matches!(look.peek(), Some('}' | ']')) {
                    // 丢掉尾逗号
                } else {
                    out.push(c);
                }
            }
            _ => out.push(c),
        }
    }
    out
}

fn plist_to_json(value: plist::Value) -> Value {
    match value {
        plist::Value::Array(items) => {
            Value::Array(items.into_iter().map(plist_to_json).collect())
        }
        plist::Value::Dictionary(dict) => {
            let mut map = serde_json::Map::new();
            for (k, v) in dict {
                map.insert(k, plist_to_json(v));
            }
            Value::Object(map)
        }
        plist::Value::Boolean(b) => Value::Bool(b),
        plist::Value::String(s) => Value::String(s),
        plist::Value::Integer(i) => {
            let n = i.as_signed().unwrap_or(0);
            serde_json::json!(n)
        }
        plist::Value::Real(n) => serde_json::json!(n),
        plist::Value::Data(_) | plist::Value::Date(_) | plist::Value::Uid(_) => Value::Null,
        _ => Value::Null,
    }
}

fn compile_value(value: &Value) -> Option<Grammar> {
    let obj = value.as_object()?;
    if obj.get("injectTo").is_some() {
        return None;
    }
    let scope_name = obj
        .get("scopeName")
        .and_then(|v| v.as_str())
        .unwrap_or("source.unknown")
        .to_string();
    let mut rules = Vec::new();
    let mut repo = HashMap::new();
    if let Some(rep) = obj.get("repository").and_then(|v| v.as_object()) {
        for (key, val) in rep {
            let ids = push_rule(val, &mut rules);
            if !ids.is_empty() {
                repo.insert(key.clone(), ids);
            }
        }
    }
    let mut top = Vec::new();
    if let Some(arr) = obj.get("patterns").and_then(|v| v.as_array()) {
        for item in arr {
            top.extend(push_rule(item, &mut rules));
        }
    }
    let has_color = rules
        .iter()
        .any(|r| matches!(r, Rule::Match { .. } | Rule::BeginEnd { .. }));
    if !has_color {
        return None;
    }
    Some(Grammar {
        scope_name,
        rules,
        repo,
        top,
        any_regex_ok: AtomicBool::new(false),
    })
}

fn push_rule(value: &Value, rules: &mut Vec<Rule>) -> Vec<usize> {
    if value.get("disabled").and_then(|v| v.as_bool()) == Some(true) {
        return Vec::new();
    }
    let begin = value.get("begin").and_then(|v| v.as_str());
    let match_pat = value.get("match").and_then(|v| v.as_str());
    if begin.is_none() && match_pat.is_none() {
        if let Some(inc) = value.get("include").and_then(|v| v.as_str()) {
            let id = rules.len();
            rules.push(Rule::Include(parse_include(inc)));
            return vec![id];
        }
        if let Some(arr) = value.get("patterns").and_then(|v| v.as_array()) {
            let mut ids = Vec::new();
            for item in arr {
                ids.extend(push_rule(item, rules));
            }
            return ids;
        }
        return Vec::new();
    }
    if let Some(begin) = begin {
        let mut children = Vec::new();
        if let Some(arr) = value.get("patterns").and_then(|v| v.as_array()) {
            for item in arr {
                children.extend(push_rule(item, rules));
            }
        }
        let end = value
            .get("end")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let id = rules.len();
        rules.push(Rule::BeginEnd {
            begin: begin.to_string(),
            end,
            name: opt_name(value, "name"),
            content_name: opt_name(value, "contentName"),
            begin_captures: captures_of(value, "beginCaptures", "captures"),
            end_captures: captures_of(value, "endCaptures", "captures"),
            patterns: children,
            apply_end_last: value
                .get("applyEndPatternLast")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            pop_at_eol: value.get("end").and_then(|v| v.as_str()).is_none(),
            begin_re: OnceLock::new(),
            end_re: OnceLock::new(),
        });
        return vec![id];
    }
    if let Some(match_pat) = match_pat {
        let id = rules.len();
        rules.push(Rule::Match {
            pattern: match_pat.to_string(),
            name: opt_name(value, "name"),
            captures: value
                .get("captures")
                .map(parse_captures)
                .unwrap_or_default(),
            re: OnceLock::new(),
        });
        return vec![id];
    }
    Vec::new()
}

fn opt_name(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn parse_include(raw: &str) -> IncludeKind {
    match raw.trim() {
        "$self" => IncludeKind::SelfRef,
        "$base" => IncludeKind::Base,
        rest if rest.starts_with('#') && rest.len() > 1 => IncludeKind::Repo(rest[1..].to_string()),
        _ => IncludeKind::External,
    }
}

fn captures_of(value: &Value, primary: &str, fallback: &str) -> Vec<(usize, String)> {
    if let Some(v) = value.get(primary) {
        let parsed = parse_captures(v);
        if !parsed.is_empty() {
            return parsed;
        }
    }
    value.get(fallback).map(parse_captures).unwrap_or_default()
}

fn parse_captures(value: &Value) -> Vec<(usize, String)> {
    let Some(obj) = value.as_object() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (key, val) in obj {
        let Ok(idx) = key.parse::<usize>() else {
            continue;
        };
        let Some(name) = val.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        out.push((idx, name.to_string()));
    }
    out.sort_by_key(|item| item.0);
    out
}

impl Grammar {
    fn has_color_rules(&self) -> bool {
        self.rules
            .iter()
            .any(|r| matches!(r, Rule::Match { .. } | Rule::BeginEnd { .. }))
    }
}

fn compile_pair(pattern: &str) -> RePair {
    let body = pattern.trim();
    if body.is_empty() || body.len() > MAX_PATTERN_CHARS || body.contains('\0') {
        return RePair {
            bol: None,
            mid: None,
        };
    }
    let bol_src = format!("\\A(?m:(?:{}))", transform_pattern(body, true));
    let mid_src = format!(
        "\\A(?s:.{{{CTX_CHARS}}})(?m:(?:{}))",
        transform_pattern(body, false)
    );
    let bol = fancy_regex::Regex::new(&bol_src).ok();
    let mid = fancy_regex::Regex::new(&mid_src).ok();
    if bol.is_some() || mid.is_some() {
        // flag is set by caller
    }
    RePair { bol, mid }
}

/// `at_bol` 时保留 `^` / `\A`；否则把它们换成永不成立的断言，并把 `\G` 当成“当前位置”。
fn transform_pattern(pattern: &str, at_bol: bool) -> String {
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::with_capacity(pattern.len() + 8);
    let mut i = 0;
    let mut in_class = false;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            let n = chars[i + 1];
            if !in_class && n == 'G' {
                out.push_str("(?:)");
                i += 2;
                continue;
            }
            if !at_bol && !in_class && n == 'A' {
                out.push_str("(?!)");
                i += 2;
                continue;
            }
            if n == 'h' {
                if in_class {
                    out.push_str(" \\t");
                } else {
                    out.push_str("[ \\t]");
                }
                i += 2;
                continue;
            }
            if n == 'H' && !in_class {
                out.push_str("[^ \\t]");
                i += 2;
                continue;
            }
            out.push('\\');
            out.push(n);
            i += 2;
            continue;
        }
        if !in_class && c == '(' && i + 2 < chars.len() && chars[i + 1] == '?' && chars[i + 2] == '>'
        {
            out.push_str("(?:");
            i += 3;
            continue;
        }
        if c == '[' && !in_class {
            in_class = true;
            out.push(c);
            i += 1;
            if i < chars.len() && chars[i] == '^' {
                out.push('^');
                i += 1;
            }
            if i < chars.len() && chars[i] == ']' {
                out.push(']');
                i += 1;
            }
            continue;
        }
        if c == ']' && in_class {
            in_class = false;
            out.push(c);
            i += 1;
            continue;
        }
        if !in_class && (c == '*' || c == '+' || c == '?' || c == '}') && i + 1 < chars.len() && chars[i + 1] == '+'
        {
            out.push(c);
            i += 2;
            continue;
        }
        if c == '^' && !in_class && !at_bol {
            out.push_str("(?!)");
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

fn compiled<'a>(slot: &'a OnceLock<RePair>, pattern: &str, flag: &AtomicBool) -> Option<&'a RePair> {
    let owned = pattern.to_string();
    let pair = slot.get_or_init(|| {
        let pair = compile_pair(&owned);
        if pair.bol.is_some() || pair.mid.is_some() {
            flag.store(true, Ordering::Relaxed);
        }
        pair
    });
    if pair.bol.is_none() && pair.mid.is_none() {
        None
    } else {
        Some(pair)
    }
}

fn exec(re: &RePair, hay: &str, pos: usize) -> Option<Caps> {
    if pos == 0 {
        let regex = re.bol.as_ref()?;
        let caps = regex.captures(hay).ok()??;
        return shift_caps(&caps, 0, 0);
    }
    let regex = re.mid.as_ref()?;
    let (buf, prefix_bytes) = mid_hay(hay, pos);
    let caps = regex.captures(&buf).ok()??;
    shift_caps(&caps, pos, prefix_bytes)
}

fn mid_hay(hay: &str, pos: usize) -> (String, usize) {
    let prefix = prev_chars(hay, pos, CTX_CHARS);
    let have = prefix.chars().count();
    let pad_n = CTX_CHARS.saturating_sub(have);
    let mut buf = String::with_capacity(pad_n * 3 + prefix.len() + hay.len() - pos);
    for _ in 0..pad_n {
        buf.push('\u{E000}');
    }
    buf.push_str(prefix);
    let prefix_bytes = buf.len();
    buf.push_str(&hay[pos..]);
    (buf, prefix_bytes)
}

fn prev_chars(s: &str, pos: usize, n: usize) -> &str {
    if pos == 0 || n == 0 || !s.is_char_boundary(pos) {
        return "";
    }
    let mut start = pos;
    let mut count = 0usize;
    for (i, _) in s[..pos].char_indices().rev() {
        start = i;
        count += 1;
        if count == n {
            break;
        }
    }
    &s[start..pos]
}

fn shift_caps(caps: &fancy_regex::Captures<'_>, pos: usize, prefix_bytes: usize) -> Option<Caps> {
    let whole = caps.get(0)?;
    if whole.start() != 0 {
        return None;
    }
    let mut groups = Vec::with_capacity(caps.len());
    for i in 0..caps.len() {
        groups.push(caps.get(i).map(|m| {
            let start = pos + m.start().saturating_sub(prefix_bytes);
            let end = pos + m.end().saturating_sub(prefix_bytes);
            (start, end)
        }));
    }
    if groups.first().and_then(|g| *g).is_none() {
        return None;
    }
    Some(Caps { groups })
}

fn pattern_ids(grammar: &Grammar, pref: PatternRef) -> &[usize] {
    match pref {
        PatternRef::Top => grammar.top.as_slice(),
        PatternRef::Rule(idx) => match grammar.rules.get(idx) {
            Some(Rule::BeginEnd { patterns, .. }) => patterns.as_slice(),
            _ => &[],
        },
    }
}

fn first_hit(grammar: &Grammar, ids: &[usize], hay: &str, pos: usize, depth: u32) -> Option<Hit> {
    if depth > INCLUDE_DEPTH {
        return None;
    }
    for id in ids {
        let Some(rule) = grammar.rules.get(*id) else {
            continue;
        };
        match rule {
            Rule::Include(IncludeKind::External) => {}
            Rule::Include(IncludeKind::SelfRef | IncludeKind::Base) => {
                if let Some(hit) = first_hit(grammar, &grammar.top, hay, pos, depth + 1) {
                    return Some(hit);
                }
            }
            Rule::Include(IncludeKind::Repo(name)) => {
                if let Some(nested) = grammar.repo.get(name) {
                    if let Some(hit) = first_hit(grammar, nested, hay, pos, depth + 1) {
                        return Some(hit);
                    }
                }
            }
            Rule::Match { re, pattern, .. } => {
                let Some(regex) = compiled(re, pattern, &grammar.any_regex_ok) else {
                    continue;
                };
                if let Some(caps) = exec(regex, hay, pos) {
                    if caps.end() > pos {
                        return Some(Hit::Token { rule: *id, caps });
                    }
                }
            }
            Rule::BeginEnd {
                begin_re, begin, ..
            } => {
                let Some(regex) = compiled(begin_re, begin, &grammar.any_regex_ok) else {
                    continue;
                };
                if let Some(caps) = exec(regex, hay, pos) {
                    if caps.end() > pos {
                        return Some(Hit::Begin { rule: *id, caps });
                    }
                }
            }
        }
    }
    None
}

fn paint(grammar: &Grammar, source: &str) -> String {
    let mut emitter = Emitter::with_capacity(source.len().saturating_mul(2));
    let mut stack = vec![Frame {
        scope: Some(grammar.scope_name.clone()),
        content_scope: None,
        content_active: true,
        end_rule: None,
        patterns: PatternRef::Top,
        apply_end_last: false,
        pop_at_eol: false,
    }];
    let mut rest = source;
    let mut hay = String::new();
    while !rest.is_empty() {
        let before = rest.len();
        let (content, sep, next) = split_line(rest);
        hay.clear();
        hay.push_str(content);
        hay.push('\n');
        paint_line(grammar, &hay, content.len(), &mut stack, &mut emitter);
        while stack.len() > 1 && stack.last().is_some_and(|f| f.pop_at_eol) {
            stack.pop();
        }
        if !sep.is_empty() {
            emitter.push(class_from_stack(&stack, None), sep);
        }
        if next.len() >= before {
            if !next.is_empty() {
                emitter.push(class_from_stack(&stack, None), next);
            }
            break;
        }
        rest = next;
    }
    emitter.finish()
}

fn split_line(source: &str) -> (&str, &str, &str) {
    match source.find('\n') {
        Some(i) => {
            if i > 0 && source.as_bytes()[i - 1] == b'\r' {
                (&source[..i - 1], &source[i - 1..i + 1], &source[i + 1..])
            } else {
                (&source[..i], &source[i..i + 1], &source[i + 1..])
            }
        }
        None => {
            if let Some(content) = source.strip_suffix('\r') {
                let sep_at = content.len();
                (content, &source[sep_at..], "")
            } else {
                (source, "", "")
            }
        }
    }
}

fn paint_line(
    grammar: &Grammar,
    hay: &str,
    content_len: usize,
    stack: &mut Vec<Frame>,
    emitter: &mut Emitter,
) {
    let mut pos = 0usize;
    let mut steps = 0usize;
    let max_steps = content_len.saturating_mul(4).saturating_add(32);
    let mut prev = (usize::MAX, usize::MAX);
    while pos <= content_len && steps < max_steps {
        steps += 1;
        let sig = (pos, stack.len());
        if sig == prev {
            if pos >= content_len {
                break;
            }
            let next = advance_char(hay, pos, content_len);
            emitter.push(class_from_stack(stack, None), &hay[pos..next]);
            pos = next;
            prev = (pos, stack.len());
            continue;
        }
        prev = sig;

        let apply_end_last = stack.last().is_some_and(|f| f.apply_end_last);
        if !apply_end_last {
            if let Some(new_pos) = consume_end(grammar, hay, content_len, pos, stack, emitter) {
                pos = new_pos;
                continue;
            }
        }

        if pos < content_len {
            let pref = stack.last().map(|f| f.patterns).unwrap_or(PatternRef::Top);
            let hit = {
                let ids = pattern_ids(grammar, pref);
                first_hit(grammar, ids, hay, pos, 0)
            };
            if let Some(hit) = hit {
                match hit {
                    Hit::Token { rule, caps } => {
                        let end = caps.end();
                        if let Some(Rule::Match { name, captures, .. }) = grammar.rules.get(rule) {
                            emit_match(
                                emitter,
                                hay,
                                content_len,
                                &caps,
                                stack,
                                name.as_deref(),
                                captures,
                            );
                        }
                        pos = end;
                        if pos > content_len {
                            break;
                        }
                        continue;
                    }
                    Hit::Begin { rule, caps } => {
                        let end = caps.end();
                        if stack.len() >= MAX_STACK {
                            if let Some(Rule::BeginEnd {
                                name,
                                begin_captures,
                                ..
                            }) = grammar.rules.get(rule)
                            {
                                emit_match(
                                    emitter,
                                    hay,
                                    content_len,
                                    &caps,
                                    stack,
                                    name.as_deref(),
                                    begin_captures,
                                );
                            }
                        } else if let Some(frame) = begin_frame(grammar, rule) {
                            let begin_captures = match grammar.rules.get(rule) {
                                Some(Rule::BeginEnd { begin_captures, .. }) => {
                                    begin_captures.as_slice()
                                }
                                _ => &[],
                            };
                            stack.push(frame);
                            emit_match(
                                emitter,
                                hay,
                                content_len,
                                &caps,
                                stack,
                                None,
                                begin_captures,
                            );
                            if let Some(top) = stack.last_mut() {
                                top.content_active = true;
                            }
                        }
                        pos = end;
                        if pos > content_len {
                            break;
                        }
                        continue;
                    }
                }
            }
        }

        if apply_end_last {
            if let Some(new_pos) = consume_end(grammar, hay, content_len, pos, stack, emitter) {
                pos = new_pos;
                continue;
            }
        }

        if pos >= content_len {
            break;
        }
        let next = advance_char(hay, pos, content_len);
        if next == pos {
            break;
        }
        emitter.push(class_from_stack(stack, None), &hay[pos..next]);
        pos = next;
    }
    if pos < content_len && hay.is_char_boundary(pos) {
        emitter.push(class_from_stack(stack, None), &hay[pos..content_len]);
    }
}

fn begin_frame(grammar: &Grammar, rule: usize) -> Option<Frame> {
    let Rule::BeginEnd {
        name,
        content_name,
        apply_end_last,
        pop_at_eol,
        ..
    } = grammar.rules.get(rule)?
    else {
        return None;
    };
    Some(Frame {
        scope: name.clone(),
        content_scope: content_name.clone(),
        content_active: false,
        end_rule: Some(rule),
        patterns: PatternRef::Rule(rule),
        apply_end_last: *apply_end_last,
        pop_at_eol: *pop_at_eol,
    })
}

/// 匹配当前帧的 `end`。成功时弹出帧并返回新的位置。
fn consume_end(
    grammar: &Grammar,
    hay: &str,
    content_len: usize,
    pos: usize,
    stack: &mut Vec<Frame>,
    emitter: &mut Emitter,
) -> Option<usize> {
    if stack.len() <= 1 {
        return None;
    }
    let rule_idx = stack.last()?.end_rule?;
    let pat = match grammar.rules.get(rule_idx) {
        Some(Rule::BeginEnd { end: Some(end), .. }) => end.clone(),
        _ => return None,
    };
    let matched = {
        let Rule::BeginEnd { end_re, .. } = grammar.rules.get(rule_idx)? else {
            return None;
        };
        match compiled(end_re, &pat, &grammar.any_regex_ok) {
            Some(regex) => exec(regex, hay, pos),
            None => {
                stack.pop();
                return Some(pos);
            }
        }
    };
    let caps = matched?;
    if caps.start() != pos {
        return None;
    }
    if let Some(frame) = stack.last_mut() {
        frame.content_active = false;
    }
    let end_captures = match grammar.rules.get(rule_idx) {
        Some(Rule::BeginEnd { end_captures, .. }) => end_captures.clone(),
        _ => Vec::new(),
    };
    emit_match(
        emitter,
        hay,
        content_len,
        &caps,
        stack,
        None,
        &end_captures,
    );
    let new_pos = caps.end();
    stack.pop();
    Some(new_pos)
}

fn emit_match(
    emitter: &mut Emitter,
    hay: &str,
    content_len: usize,
    caps: &Caps,
    stack: &[Frame],
    rule_name: Option<&str>,
    captures: &[(usize, String)],
) {
    let start = caps.start().min(content_len);
    let end = caps.end().min(content_len);
    if start >= end || !hay.is_char_boundary(start) || !hay.is_char_boundary(end) {
        return;
    }
    let mut points = vec![start, end];
    for (idx, _) in captures {
        if *idx == 0 {
            continue;
        }
        if let Some(Some((s, e))) = caps.groups.get(*idx).copied() {
            let s = s.min(content_len);
            let e = e.min(content_len);
            if s > start && s < end && hay.is_char_boundary(s) {
                points.push(s);
            }
            if e > start && e < end && hay.is_char_boundary(e) {
                points.push(e);
            }
        }
    }
    points.sort_unstable();
    points.dedup();
    for pair in points.windows(2) {
        let a = pair[0];
        let b = pair[1];
        if a >= b {
            continue;
        }
        let inner = innermost_capture(caps, captures, a);
        let class = class_layered(stack, rule_name, inner);
        emitter.push(class, &hay[a..b]);
    }
}

fn innermost_capture<'a>(
    caps: &Caps,
    captures: &'a [(usize, String)],
    byte: usize,
) -> Option<&'a str> {
    let mut best: Option<(usize, &str)> = None;
    for (idx, name) in captures {
        if *idx == 0 {
            continue;
        }
        let Some(Some((s, e))) = caps.groups.get(*idx).copied() else {
            continue;
        };
        if byte >= s && byte < e {
            let len = e - s;
            if best.map(|(n, _)| len < n).unwrap_or(true) {
                best = Some((len, name.as_str()));
            }
        }
    }
    best.map(|(_, name)| name)
}

fn advance_char(s: &str, pos: usize, limit: usize) -> usize {
    if pos >= limit {
        return limit;
    }
    let Some(ch) = s[pos..].chars().next() else {
        return limit;
    };
    (pos + ch.len_utf8()).min(limit)
}

fn class_layered(
    stack: &[Frame],
    outer: Option<&str>,
    inner: Option<&str>,
) -> Option<&'static str> {
    if let Some(scope) = inner {
        if let Some(class) = map_scope_string(scope) {
            return Some(class);
        }
    }
    if let Some(scope) = outer {
        if let Some(class) = map_scope_string(scope) {
            return Some(class);
        }
    }
    class_from_stack(stack, None)
}

fn class_from_stack(stack: &[Frame], extra: Option<&str>) -> Option<&'static str> {
    if let Some(scope) = extra {
        if let Some(class) = map_scope_string(scope) {
            return Some(class);
        }
    }
    for frame in stack.iter().rev() {
        if frame.content_active {
            if let Some(scope) = &frame.content_scope {
                if let Some(class) = map_scope_string(scope) {
                    return Some(class);
                }
            }
        }
        if let Some(scope) = &frame.scope {
            if let Some(class) = map_scope_string(scope) {
                return Some(class);
            }
        }
    }
    None
}

fn map_scope_string(scope: &str) -> Option<&'static str> {
    for atom in scope.split_whitespace().rev() {
        if let Some(class) = map_atom(atom) {
            return Some(class);
        }
    }
    None
}

fn map_atom(scope: &str) -> Option<&'static str> {
    if scope.contains("comment") || scope.contains("markup.quote") {
        return Some("comment");
    }
    if scope.contains("constant.numeric") {
        return Some("number");
    }
    if scope.contains("string")
        || scope.contains("markup.raw")
        || scope.contains("markup.inline.raw")
        || scope.contains("constant.character")
    {
        return Some("string");
    }
    if scope.contains("keyword.operator") || scope.contains("punctuation") {
        return Some("punct");
    }
    if scope.contains("keyword")
        || scope.contains("constant.language")
        || scope.contains("variable.language")
        || scope.contains("storage.modifier")
        || scope.contains("storage.type")
        || scope.contains("markup.heading")
    {
        return Some("keyword");
    }
    if scope.contains("entity.name.function")
        || scope.contains("support.function")
        || scope.contains("variable.function")
        || scope.contains("entity.name.macro")
    {
        return Some("func");
    }
    if scope.contains("entity.name.tag") {
        return Some("tag");
    }
    if scope.contains("support.type")
        || scope.contains("support.class")
        || scope.contains("entity.name.type")
        || scope.contains("entity.name.class")
        || scope.contains("entity.name.struct")
        || scope.contains("entity.name.enum")
        || scope.contains("entity.name.interface")
        || scope.contains("entity.name.namespace")
        || scope.contains("entity.other.inherited-class")
        || scope.contains("entity.other.attribute-name")
    {
        return Some("type");
    }
    if scope == "storage" || scope.starts_with("storage.") {
        return Some("keyword");
    }
    if scope == "constant" || scope.starts_with("constant.") {
        return Some("number");
    }
    None
}

impl Emitter {
    fn with_capacity(n: usize) -> Self {
        Self {
            out: String::with_capacity(n),
            buf: String::new(),
            class: None,
            open: false,
        }
    }

    fn push(&mut self, class: Option<&'static str>, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.open && self.class == class {
            self.buf.push_str(text);
            return;
        }
        self.flush();
        self.buf.push_str(text);
        self.class = class;
        self.open = true;
    }

    fn flush(&mut self) {
        if !self.open || self.buf.is_empty() {
            self.buf.clear();
            self.open = false;
            return;
        }
        if let Some(class) = self.class {
            self.out.push_str("<span class=\"ac-syn-");
            self.out.push_str(class);
            self.out.push_str("\">");
            push_escaped(&mut self.out, &self.buf);
            self.out.push_str("</span>");
        } else {
            push_escaped(&mut self.out, &self.buf);
        }
        self.buf.clear();
        self.open = false;
    }

    fn finish(mut self) -> String {
        self.flush();
        self.out
    }
}

fn push_escaped(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEMO: &str = r##"{
      "scopeName": "source.demo",
      "patterns": [
        { "include": "#comments" },
        { "include": "#strings" },
        { "include": "#keywords" },
        { "name": "constant.numeric", "match": "\\b\\d+\\b" }
      ],
      "repository": {
        "comments": {
          "patterns": [
            { "name": "comment.line.double-slash", "match": "//.*" },
            { "name": "comment.block", "begin": "/\\*", "end": "\\*/" }
          ]
        },
        "strings": {
          "name": "string.quoted.double",
          "begin": "\"",
          "end": "\"",
          "patterns": [
            { "name": "constant.character.escape", "match": "\\\\." }
          ]
        },
        "keywords": {
          "patterns": [
            { "name": "keyword.control", "match": "\\b(fn|let|if|else)\\b" }
          ]
        }
      }
    }"##;

    fn highlight_json(json: &str, source: &str) -> String {
        let value: Value = serde_json::from_str(json).unwrap();
        let grammar = compile_value(&value).expect("grammar");
        let html = paint(&grammar, source);
        assert!(
            grammar.any_regex_ok.load(Ordering::Relaxed),
            "regex did not compile"
        );
        html
    }

    fn visible_text(html: &str) -> String {
        let mut out = String::new();
        let mut chars = html.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '<' {
                for n in chars.by_ref() {
                    if n == '>' {
                        break;
                    }
                }
                continue;
            }
            if c == '&' {
                let mut ent = String::new();
                for n in chars.by_ref() {
                    if n == ';' {
                        break;
                    }
                    ent.push(n);
                }
                out.push(match ent.as_str() {
                    "amp" => '&',
                    "lt" => '<',
                    "gt" => '>',
                    "quot" => '"',
                    _ => '?',
                });
                continue;
            }
            out.push(c);
        }
        out
    }

    fn class_at(html: &str, needle: &str) -> Option<String> {
        let i = html.find(needle)?;
        let before = &html[..i];
        let open = before.rfind("<span class=\"ac-syn-")?;
        let close = before.rfind("</span>").unwrap_or(0);
        if close > open {
            return None;
        }
        let rest = &before[open + "<span class=\"ac-syn-".len()..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    }

    #[test]
    fn grammar_colors_keywords_strings_comments_and_numbers() {
        let source = "fn main() {\n    let n = 42;\n    let myfn = 3;\n    let s = \"a < b & c\";\n    /* block\n    still */\n    // tail\n}\n";
        let html = highlight_json(DEMO, source);
        assert_eq!(visible_text(&html), source, "{html}");
        assert_eq!(class_at(&html, "fn").as_deref(), Some("keyword"));
        assert_eq!(class_at(&html, "let").as_deref(), Some("keyword"));
        assert_eq!(class_at(&html, "42").as_deref(), Some("number"));
        assert!(
            html.contains("myfn"),
            "word boundary split an identifier: {html}"
        );
        assert_eq!(class_at(&html, "a &lt;").as_deref(), Some("string"));
        assert_eq!(class_at(&html, "still").as_deref(), Some("comment"));
        assert_eq!(class_at(&html, "tail").as_deref(), Some("comment"));
        assert!(html.contains("&lt;"), "{html}");
        assert!(html.contains("&amp;"), "{html}");
    }

    #[test]
    fn scope_mapping_covers_common_tokens() {
        assert_eq!(map_atom("keyword.control.rust"), Some("keyword"));
        assert_eq!(map_atom("string.quoted.double"), Some("string"));
        assert_eq!(map_atom("comment.line.double-slash"), Some("comment"));
        assert_eq!(map_atom("constant.numeric.integer"), Some("number"));
        assert_eq!(map_atom("entity.name.function"), Some("func"));
        assert_eq!(map_atom("entity.name.type"), Some("type"));
        assert_eq!(map_atom("entity.name.tag"), Some("tag"));
        assert_eq!(map_atom("punctuation.separator"), Some("punct"));
        assert_eq!(map_atom("variable.other"), None);
    }
}
