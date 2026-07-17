#[test]
fn open_sqlite_via_cdylib() {
    let dir = std::env::temp_dir().join(format!("pusa-ffi-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let db = dir.join("t.db");
    let ctx = shared::RuntimeContext::open(&db, None).expect("open");
    let _ = ctx; // drop
}
