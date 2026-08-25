//! Desktop：惰性初始化 [`shared::RuntimeContext`]（SQLite 在用户数据目录）。

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use once_cell::sync::Lazy;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use protocol::{ChatTurnRequest, SseEvent};
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use shared::RuntimeContext;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
use std::sync::{Arc, RwLock};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
struct RuntimeState {
    ctx: RuntimeContext,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn runtime_root_dir() -> std::path::PathBuf {
    let root = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().expect("cwd"))
        .join("AnotherClaw");
    std::fs::create_dir_all(&root).ok();
    root
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn open_runtime_for_wallet(wallet_ns: &str) -> anyhow::Result<RuntimeContext> {
    let root = runtime_root_dir();
    let ns_dir = root.join("wallets").join(sanitize_wallet_ns(wallet_ns));
    std::fs::create_dir_all(&ns_dir).ok();
    let db = ns_dir.join("app.db");
    let key = std::env::var("OPENAI_API_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let ctx = RuntimeContext::open(db, key)?;
    // 桌面进程内同样启动策略调度器，让 desktop 用户与 server 模式行为一致。
    // 同一切换钱包重新 open，旧 RuntimeContext 上的 scheduler task 会因为对应 broadcast/conn
    // 仍被持有而继续跑；这里只保证「新 ctx 至少有一个 scheduler」。
    shared::strategy_scheduler::spawn_strategy_scheduler(ctx.clone());
    Ok(ctx)
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
static RUNTIME: Lazy<RwLock<RuntimeState>> = Lazy::new(|| {
    let wallet_ns = std::env::var("ANOTHERCLAW_WALLET_ADDRESS")
        .ok()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".to_string());
    let ctx = open_runtime_for_wallet(&wallet_ns).expect("open sqlite");
    RwLock::new(RuntimeState { ctx })
});

/// 文件系统安全的钱包目录名：只保留十六进制 / 字母 / 短横，避免越权写入。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn sanitize_wallet_ns(raw: &str) -> String {
    let s: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if s.is_empty() {
        "default".to_string()
    } else {
        s
    }
}

/// 全局桌面运行时快照。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn runtime_ctx() -> RuntimeContext {
    RUNTIME.read().expect("runtime read").ctx.clone()
}

/// 桌面端中止当前流式对话（对应 Web 的 `AbortController`）。
///
/// 底层 FFI 仍可能跑完本轮 Agent；这里只停止向 UI 推事件，并立刻结束本轮等待。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[derive(Clone)]
pub struct ChatAbort {
    flag: Arc<AtomicBool>,
    notify: Arc<tokio::sync::Notify>,
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
impl ChatAbort {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            flag: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(tokio::sync::Notify::new()),
        })
    }

    pub fn abort(&self) {
        self.flag.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn is_aborted(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    async fn wait(&self) {
        self.notify.notified().await;
    }
}

/// 进程内流式对话（不经 HTTP）。
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub async fn desktop_chat_stream(
    req: ChatTurnRequest,
    abort: Arc<ChatAbort>,
    on_event: &mut impl FnMut(SseEvent),
) -> anyhow::Result<()> {
    let ctx = runtime_ctx();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SseEvent>();
    let worker = tokio::spawn(async move { ctx.run_chat_turn(req, tx).await });

    loop {
        if abort.is_aborted() {
            worker.abort();
            return Ok(());
        }
        tokio::select! {
            biased;
            _ = abort.wait() => {}
            ev = rx.recv() => {
                match ev {
                    Some(ev) => on_event(ev),
                    None => break,
                }
            }
        }
    }

    if abort.is_aborted() {
        worker.abort();
        return Ok(());
    }

    worker
        .await
        .map_err(|e| anyhow::anyhow!("chat worker join: {e}"))??;
    Ok(())
}
