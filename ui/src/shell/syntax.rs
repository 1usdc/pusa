//! 轻量语法高亮：按扩展名分词，输出已转义 HTML。

use std::path::Path;

const MAX_HIGHLIGHT_BYTES: usize = 220_000;

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn push_plain(out: &mut String, text: &str) {
    if !text.is_empty() {
        out.push_str(&escape_html(text));
    }
}

fn push_span(out: &mut String, class: &str, text: &str) {
    if text.is_empty() {
        return;
    }
    out.push_str("<span class=\"ac-syn-");
    out.push_str(class);
    out.push_str("\">");
    out.push_str(&escape_html(text));
    out.push_str("</span>");
}

/// 由路径推断高亮语言；未知则 `"plain"`。
pub fn language_from_path(path: &str) -> &'static str {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match name.as_str() {
        "dockerfile" => return "dockerfile",
        "makefile" | "gnumakefile" => return "shell",
        "cmakelists.txt" => return "cmake",
        "gemfile" | "rakefile" => return "ruby",
        _ => {}
    }
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "rs" => "rust",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" => "typescript",
        "tsx" => "tsx",
        "jsx" => "jsx",
        "css" | "scss" | "less" => "css",
        "html" | "htm" | "xhtml" | "vue" | "svelte" => "html",
        "json" | "jsonc" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" | "mdx" => "markdown",
        "py" | "pyw" => "python",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "swift" => "swift",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "sql" => "sql",
        "sh" | "bash" | "zsh" | "fish" => "shell",
        "xml" | "svg" | "plist" => "xml",
        "cmake" => "cmake",
        _ => "plain",
    }
}

fn keywords_for(lang: &str) -> &'static [&'static str] {
    match lang {
        "rust" => &[
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
            "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
            "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static",
            "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            "union",
        ],
        "javascript" | "typescript" | "jsx" | "tsx" => &[
            "async", "await", "break", "case", "catch", "class", "const", "continue",
            "debugger", "default", "delete", "do", "else", "export", "extends", "false",
            "finally", "for", "from", "function", "if", "import", "in", "instanceof",
            "let", "new", "null", "of", "return", "static", "super", "switch", "this",
            "throw", "true", "try", "typeof", "undefined", "var", "void", "while",
            "with", "yield", "enum", "implements", "interface", "package", "private",
            "protected", "public", "readonly", "type", "namespace", "abstract", "as",
            "keyof", "infer", "satisfies",
        ],
        "python" => &[
            "and", "as", "assert", "async", "await", "break", "class", "continue",
            "def", "del", "elif", "else", "except", "False", "finally", "for", "from",
            "global", "if", "import", "in", "is", "lambda", "None", "nonlocal", "not",
            "or", "pass", "raise", "return", "True", "try", "while", "with", "yield",
        ],
        "go" => &[
            "break", "case", "chan", "const", "continue", "default", "defer", "else",
            "fallthrough", "for", "func", "go", "goto", "if", "import", "interface",
            "map", "package", "range", "return", "select", "struct", "switch", "type",
            "var", "true", "false", "nil",
        ],
        "java" | "kotlin" | "csharp" | "swift" => &[
            "abstract", "as", "async", "await", "break", "case", "catch", "class",
            "const", "continue", "default", "do", "else", "enum", "extends", "false",
            "final", "finally", "for", "fun", "func", "function", "if", "implements",
            "import", "in", "interface", "internal", "is", "let", "new", "null",
            "override", "package", "private", "protected", "public", "return", "static",
            "struct", "super", "switch", "this", "throw", "true", "try", "typealias",
            "var", "void", "while", "yield",
        ],
        "c" | "cpp" => &[
            "auto", "bool", "break", "case", "catch", "char", "class", "const",
            "constexpr", "continue", "default", "delete", "do", "double", "else",
            "enum", "explicit", "extern", "false", "float", "for", "friend", "goto",
            "if", "inline", "int", "long", "mutable", "namespace", "new", "noexcept",
            "nullptr", "operator", "private", "protected", "public", "return", "short",
            "signed", "sizeof", "static", "struct", "switch", "template", "this",
            "throw", "true", "try", "typedef", "typename", "union", "unsigned",
            "using", "virtual", "void", "volatile", "while",
        ],
        "ruby" => &[
            "alias", "and", "begin", "break", "case", "class", "def", "do", "else",
            "elsif", "end", "ensure", "false", "for", "if", "in", "module", "next",
            "nil", "not", "or", "redo", "rescue", "retry", "return", "self", "super",
            "then", "true", "undef", "unless", "until", "when", "while", "yield",
        ],
        "php" => &[
            "abstract", "and", "array", "as", "break", "case", "catch", "class",
            "clone", "const", "continue", "default", "do", "echo", "else", "elseif",
            "empty", "endfor", "endforeach", "endif", "endswitch", "endwhile", "extends",
            "final", "finally", "fn", "for", "foreach", "function", "global", "if",
            "implements", "include", "include_once", "instanceof", "interface", "isset",
            "list", "match", "namespace", "new", "or", "print", "private", "protected",
            "public", "require", "require_once", "return", "static", "switch", "throw",
            "trait", "try", "unset", "use", "var", "while", "xor", "yield", "true",
            "false", "null",
        ],
        "sql" => &[
            "add", "all", "alter", "and", "as", "asc", "between", "by", "case",
            "check", "column", "constraint", "create", "database", "default", "delete",
            "desc", "distinct", "drop", "else", "end", "exists", "foreign", "from",
            "full", "group", "having", "in", "index", "inner", "insert", "into",
            "is", "join", "key", "left", "like", "limit", "not", "null", "on",
            "or", "order", "outer", "primary", "references", "right", "select",
            "set", "table", "then", "union", "unique", "update", "values", "when",
            "where",
        ],
        "shell" | "dockerfile" | "cmake" => &[
            "if", "then", "else", "elif", "fi", "for", "while", "do", "done",
            "case", "esac", "function", "in", "FROM", "RUN", "CMD", "ENTRYPOINT",
            "ENV", "ARG", "COPY", "ADD", "WORKDIR", "USER", "VOLUME", "EXPOSE",
            "LABEL", "AS",
        ],
        "css" => &[
            "important", "and", "or", "not", "only", "from", "to", "var", "url",
            "rgb", "rgba", "hsl", "hsla", "calc", "min", "max", "clamp",
        ],
        "json" | "toml" | "yaml" => &["true", "false", "null"],
        _ => &[],
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '$' || c == '@'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$' || c == '-'
}

fn is_keyword(lang: &str, word: &str) -> bool {
    keywords_for(lang).iter().any(|k| *k == word)
}

fn line_comment_prefix(lang: &str) -> Option<&'static str> {
    match lang {
        "python" | "toml" | "yaml" | "shell" | "dockerfile" | "cmake" | "ruby" => Some("#"),
        "sql" => Some("--"),
        "html" | "xml" | "markdown" | "plain" => None,
        _ => Some("//"),
    }
}

fn supports_block_comment(lang: &str) -> bool {
    !matches!(
        lang,
        "python" | "toml" | "yaml" | "shell" | "dockerfile" | "markdown" | "plain" | "json"
    )
}

/// 将源码高亮为 HTML；未知语言或超大文件退化为转义纯文本。
pub fn highlight_html(source: &str, lang: &str) -> String {
    if source.is_empty() {
        return String::new();
    }
    if source.len() > MAX_HIGHLIGHT_BYTES || lang == "plain" {
        return escape_html(source);
    }
    match lang {
        "markdown" => highlight_markdown(source),
        "html" | "xml" => highlight_markup(source),
        _ => highlight_code(source, lang),
    }
}

fn highlight_code(source: &str, lang: &str) -> String {
    let mut out = String::with_capacity(source.len().saturating_mul(2));
    let b = source.as_bytes();
    let n = b.len();
    let mut i = 0usize;
    let line_prefix = line_comment_prefix(lang);
    let block_ok = supports_block_comment(lang);

    while i < n {
        let c = b[i] as char;

        // 空白
        if c.is_whitespace() {
            let start = i;
            i += 1;
            while i < n && (b[i] as char).is_whitespace() {
                i += 1;
            }
            push_plain(&mut out, &source[start..i]);
            continue;
        }

        // 行注释
        if let Some(prefix) = line_prefix {
            if source[i..].starts_with(prefix) {
                let start = i;
                i += prefix.len();
                while i < n && b[i] != b'\n' {
                    i += 1;
                }
                push_span(&mut out, "comment", &source[start..i]);
                continue;
            }
        }

        // 块注释 /* */
        if block_ok && c == '/' && i + 1 < n && b[i + 1] == b'*' {
            let start = i;
            i += 2;
            while i + 1 < n && !(b[i] == b'*' && b[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < n {
                i += 2;
            } else {
                i = n;
            }
            push_span(&mut out, "comment", &source[start..i]);
            continue;
        }

        // 字符串
        if c == '"' || c == '\'' || c == '`' {
            // Rust 生命周期 / 字符：'a 或 'x'
            if lang == "rust" && c == '\'' {
                if i + 2 < n && b[i + 2] == b'\'' {
                    // 字符字面量 'x'
                    push_span(&mut out, "string", &source[i..i + 3]);
                    i += 3;
                    continue;
                }
                if i + 1 < n && is_ident_start(b[i + 1] as char) {
                    let start = i;
                    i += 1;
                    while i < n && is_ident_continue(b[i] as char) {
                        i += 1;
                    }
                    push_span(&mut out, "type", &source[start..i]);
                    continue;
                }
            }
            let quote = c;
            let start = i;
            i += 1;
            while i < n {
                let ch = b[i] as char;
                if ch == '\\' && i + 1 < n {
                    i += 2;
                    continue;
                }
                if ch == quote {
                    i += 1;
                    break;
                }
                // JS/TS 模板串中的 ${...}：整段仍标为 string（够用）
                i += 1;
            }
            push_span(&mut out, "string", &source[start..i]);
            continue;
        }

        // 数字
        if c.is_ascii_digit() || (c == '.' && i + 1 < n && (b[i + 1] as char).is_ascii_digit()) {
            let start = i;
            i += 1;
            while i < n {
                let ch = b[i] as char;
                if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                    i += 1;
                } else {
                    break;
                }
            }
            push_span(&mut out, "number", &source[start..i]);
            continue;
        }

        // 标识符 / 关键字
        if is_ident_start(c) {
            let start = i;
            i += 1;
            while i < n && is_ident_continue(b[i] as char) {
                i += 1;
            }
            let word = &source[start..i];
            if is_keyword(lang, word) {
                push_span(&mut out, "keyword", word);
            } else if i < n && b[i] == b'(' {
                push_span(&mut out, "func", word);
            } else if word
                .chars()
                .next()
                .is_some_and(|ch| ch.is_uppercase() && lang != "css")
            {
                push_span(&mut out, "type", word);
            } else {
                push_plain(&mut out, word);
            }
            continue;
        }

        // 标点（多字节安全：非 ASCII 当普通字符）
        if c.is_ascii() {
            push_span(&mut out, "punct", &source[i..i + 1]);
            i += 1;
        } else {
            let start = i;
            i += c.len_utf8();
            push_plain(&mut out, &source[start..i]);
        }
    }

    out
}

fn highlight_markup(source: &str) -> String {
    let mut out = String::with_capacity(source.len().saturating_mul(2));
    let b = source.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i < n {
        if b[i] == b'<' {
            let start = i;
            i += 1;
            while i < n && b[i] != b'>' {
                let ch = b[i] as char;
                if ch == '"' || ch == '\'' {
                    let q = b[i];
                    i += 1;
                    while i < n {
                        if b[i] == b'\\' && i + 1 < n {
                            i += 2;
                            continue;
                        }
                        if b[i] == q {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                    continue;
                }
                i += 1;
            }
            if i < n {
                i += 1;
            }
            push_span(&mut out, "tag", &source[start..i]);
        } else {
            let start = i;
            while i < n && b[i] != b'<' {
                i += 1;
            }
            push_plain(&mut out, &source[start..i]);
        }
    }
    out
}

fn highlight_markdown(source: &str) -> String {
    let mut out = String::new();
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start_matches([' ', '\t']);
        if trimmed.starts_with('#') {
            push_span(&mut out, "keyword", line);
        } else if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            push_span(&mut out, "punct", line);
        } else if trimmed.starts_with('>') {
            push_span(&mut out, "comment", line);
        } else {
            let mut rest = line;
            while let Some(start) = rest.find('`') {
                push_plain(&mut out, &rest[..start]);
                rest = &rest[start..];
                if let Some(end) = rest[1..].find('`') {
                    let end = end + 1;
                    push_span(&mut out, "string", &rest[..=end]);
                    rest = &rest[end + 1..];
                } else {
                    push_plain(&mut out, rest);
                    rest = "";
                    break;
                }
            }
            push_plain(&mut out, rest);
        }
    }
    out
}

/// 单行高亮（diff 行）。
pub fn highlight_line_html(line: &str, lang: &str) -> String {
    let trimmed = line.strip_suffix('\n').unwrap_or(line);
    let trimmed = trimmed.strip_suffix('\r').unwrap_or(trimmed);
    highlight_html(trimmed, lang)
}
