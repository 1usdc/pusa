//! 验证 `RuntimeContext::infer_strategy_frequency_seconds_with_ai` / `_detail`。
//!
//! 在仓库根目录执行：`cargo run -p strategy-frequency-infer`
//!
//! ## 数据库
//!
//! - 默认若存在 `./data/server.db`，则**打开该文件**（与未设置 `DATABASE_PATH` 时的服务端一致），即可读到你在设置里写入的 `app_settings.openai_api_key`。
//! - 也可显式指定：`DATABASE_PATH=/path/to/server.db cargo run -p strategy-frequency-infer`
//! - 若上述文件不存在，则退回到临时空库（此时除非设置 `OPENAI_API_KEY` 环境变量，否则会显示「API Key 为空」）。
//!
//! ## OpenAI
//!
//! - 密钥优先来自数据库 `app_settings`，其次环境变量 `OPENAI_API_KEY`，其次 `RuntimeContext::open` 的构造参数（本脚本传 `None`）。

#![forbid(unsafe_code)]

use shared::{RuntimeContext, StrategyFrequencyInferDetail};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn print_detail(label: &str, d: &StrategyFrequencyInferDetail) {
    println!("── {label} ──");
    println!("  最终秒数: {}", d.seconds);
    println!(
        "  openai_v1_base_used（app_settings.openai_v1_base，缺省官方）: {}",
        if d.openai_v1_base_used.is_empty() {
            "(未写入)"
        } else {
            d.openai_v1_base_used.as_str()
        }
    );
    println!("  是否已发 LLM HTTP: {}", d.llm_http_requested);
    println!("  HTTP 状态: {:?}", d.http_status);
    println!(
        "  模型 content 是否拿到: {}",
        d.raw_assistant_text.is_some()
    );
    if let Some(ref t) = d.raw_assistant_text {
        let short: String = t.chars().take(200).collect();
        let more = if t.chars().count() > 200 { "…" } else { "" };
        println!("  原始 content（前 200 字）: {short}{more}");
    } else {
        println!("  原始 content: (无，未解析出字符串)");
    }
    println!("  从文本抽到的整数（裁剪前）: {:?}", d.extracted_integer);
    println!("  trace: {}", d.trace);
    println!();
}

fn resolve_db_path() -> (PathBuf, bool) {
    if let Ok(p) = std::env::var("DATABASE_PATH") {
        let pb = PathBuf::from(p.trim());
        println!("数据库: DATABASE_PATH -> {}", pb.display());
        return (pb, false);
    }
    let default = PathBuf::from("data/server.db");
    if default.exists() {
        println!(
            "数据库: ./data/server.db（与默认服务端一致；settings 里保存的 Key 会从这里读取）"
        );
        return (default, false);
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!("strategy-infer-{unique}.db"));
    println!(
        "数据库: 临时文件 {}（未找到 ./data/server.db，故不含 UI 保存的密钥）",
        temp.display()
    );
    (temp, true)
}

#[tokio::main]
async fn main() {
    let (db_path, delete_after) = resolve_db_path();

    let openai = std::env::var("OPENAI_API_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty());

    let ctx = RuntimeContext::open(&db_path, openai.clone()).expect("open db");
    let model = "gpt-4o-mini";

    let prompt = "策略每小时运行一次";
    let prompt2 = "建议 60 秒运行，但最多每五秒运行一次";
    let prompt3 = "";

    let d1 = ctx
        .infer_strategy_frequency_seconds_with_ai_detail(prompt, model)
        .await;
    print_detail(&format!("prompt = {prompt:?}"), &d1);

    let d2 = ctx
        .infer_strategy_frequency_seconds_with_ai_detail(prompt2, model)
        .await;
    print_detail(&format!("prompt2 = {prompt2:?}"), &d2);

    let d3 = ctx
        .infer_strategy_frequency_seconds_with_ai_detail(prompt3, model)
        .await;
    print_detail("prompt3 = \"\"（空）", &d3);

    drop(ctx);
    if delete_after {
        let _ = std::fs::remove_file(Path::new(&db_path));
    }

    if openai.is_none() && delete_after {
        println!("提示：当前使用临时库且无 OPENAI_API_KEY，trace 会显示「API Key 为空」。请在仓库根执行（存在 data/server.db）或设置 DATABASE_PATH。");
    }
}
