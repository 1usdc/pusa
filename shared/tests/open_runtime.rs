#[test]
fn open_sqlite_via_cdylib() {
    let dir = std::env::temp_dir().join(format!("pusa-ffi-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let db = dir.join("t.db");
    let ctx = shared::RuntimeContext::open(&db, None).expect("open");
    shared::strategy_scheduler::spawn_strategy_scheduler(ctx);
    std::thread::sleep(std::time::Duration::from_millis(300));
}
