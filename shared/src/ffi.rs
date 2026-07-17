//! Dynamic loader for `libpusa_core` (cdylib).

use std::ffi::{CStr, CString, OsString};
use std::os::raw::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use libloading::Library;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::{json, Value};

type FnOpen = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_void;
type FnFree = unsafe extern "C" fn(*mut c_void);
type FnStrFree = unsafe extern "C" fn(*mut c_char);
type FnCall = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> *mut c_char;
type StreamCb = unsafe extern "C" fn(*const c_char, *mut c_void);
type FnCallStream =
    unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char, StreamCb, *mut c_void) -> *mut c_char;
type FnSpawn = unsafe extern "C" fn(*mut c_void);
type FnPump = unsafe extern "C" fn(*mut c_void, StreamCb, *mut c_void);
type FnPath = unsafe extern "C" fn() -> *mut c_char;

struct Api {
    _lib: &'static Library,
    open: FnOpen,
    free: FnFree,
    str_free: FnStrFree,
    call: FnCall,
    call_stream: FnCallStream,
    spawn: FnSpawn,
    pump: FnPump,
    project_root: FnPath,
    enriched_path: FnPath,
}

fn dylib_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "pusa_core.dll"
    } else if cfg!(target_os = "macos") {
        "libpusa_core.dylib"
    } else {
        "libpusa_core.so"
    }
}

fn current_target_triple() -> String {
    std::env::var("PUSA_CORE_TARGET").unwrap_or_else(|_| {
        if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            "aarch64-apple-darwin".into()
        } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
            "x86_64-apple-darwin".into()
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            "x86_64-unknown-linux-gnu".into()
        } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            "aarch64-unknown-linux-gnu".into()
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            "x86_64-pc-windows-msvc".into()
        } else {
            "unknown".into()
        }
    })
}

fn candidate_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var("PUSA_CORE_LIB") {
        out.push(PathBuf::from(p));
    }
    let triple = current_target_triple();
    let name = dylib_name();
    out.push(PathBuf::from("vendor/pusa-core").join(&triple).join(name));
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let root = PathBuf::from(manifest).join("..");
        out.push(root.join("vendor/pusa-core").join(&triple).join(name));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join(name));
            out.push(dir.join("pusa-core").join(name));
        }
    }
    out
}

fn load_api() -> Result<Api> {
    let mut last = None;
    for path in candidate_paths() {
        if !path.is_file() {
            continue;
        }
        match unsafe { Library::new(&path) } {
            Ok(lib) => {
                let lib: &'static Library = Box::leak(Box::new(lib));
                unsafe {
                    let open: FnOpen = *lib.get(b"pusa_runtime_open\0")?;
                    let free: FnFree = *lib.get(b"pusa_runtime_free\0")?;
                    let str_free: FnStrFree = *lib.get(b"pusa_string_free\0")?;
                    let call: FnCall = *lib.get(b"pusa_runtime_call\0")?;
                    let call_stream: FnCallStream = *lib.get(b"pusa_runtime_call_stream\0")?;
                    let spawn: FnSpawn = *lib.get(b"pusa_spawn_strategy_scheduler\0")?;
                    let pump: FnPump = *lib.get(b"pusa_runtime_start_strategy_event_pump\0")?;
                    let project_root: FnPath = *lib.get(b"pusa_project_root\0")?;
                    let enriched_path: FnPath = *lib.get(b"pusa_enriched_path\0")?;
                    return Ok(Api {
                        _lib: lib,
                        open,
                        free,
                        str_free,
                        call,
                        call_stream,
                        spawn,
                        pump,
                        project_root,
                        enriched_path,
                    });
                }
            }
            Err(e) => last = Some(format!("{}: {e}", path.display())),
        }
    }
    Err(anyhow!(
        "failed to load {}; set PUSA_CORE_LIB or run `cd pusa-core && just pack`. last={:?}",
        dylib_name(),
        last
    ))
}

static API: Lazy<Result<Api, String>> = Lazy::new(|| load_api().map_err(|e| e.to_string()));

fn api() -> Result<&'static Api> {
    API.as_ref().map_err(|e| anyhow!(e.clone()))
}

fn take_c_string(p: *mut c_char) -> Result<String> {
    if p.is_null() {
        return Err(anyhow!("null c string from pusa-core"));
    }
    let s = unsafe { CStr::from_ptr(p) }
        .to_string_lossy()
        .into_owned();
    unsafe {
        (api()?.str_free)(p);
    }
    Ok(s)
}

#[derive(Deserialize)]
struct Wire {
    ok: bool,
    #[serde(default)]
    data: Value,
    #[serde(default)]
    error: Option<String>,
}

pub struct Handle {
    ptr: *mut c_void,
}

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            if let Ok(a) = api() {
                unsafe { (a.free)(self.ptr) }
            }
            self.ptr = std::ptr::null_mut();
        }
    }
}

pub fn open(db_path: &Path, openai_api_key: Option<String>) -> Result<Arc<Handle>> {
    let a = api()?;
    let path = CString::new(db_path.to_string_lossy().as_ref())?;
    let key = openai_api_key
        .as_deref()
        .map(CString::new)
        .transpose()?;
    let key_ptr = key.as_ref().map(|c| c.as_ptr()).unwrap_or(std::ptr::null());
    let ptr = unsafe { (a.open)(path.as_ptr(), key_ptr) };
    if ptr.is_null() {
        return Err(anyhow!("pusa_runtime_open returned null"));
    }
    Ok(Arc::new(Handle { ptr }))
}

pub fn call_json<T: for<'de> Deserialize<'de>>(
    handle: &Handle,
    method: &str,
    args: Value,
) -> Result<T> {
    let a = api()?;
    let m = CString::new(method)?;
    let args_s = CString::new(args.to_string())?;
    let raw = unsafe { (a.call)(handle.ptr, m.as_ptr(), args_s.as_ptr()) };
    let s = take_c_string(raw)?;
    let wire: Wire =
        serde_json::from_str(&s).with_context(|| format!("decode {method}: {s}"))?;
    if !wire.ok {
        return Err(anyhow!(wire.error.unwrap_or(s)));
    }
    if wire.data.is_null() {
        return Err(anyhow!("{method}: ok but empty data (use call_unit for null)"));
    }
    serde_json::from_value(wire.data).with_context(|| format!("data {method}"))
}

pub fn call_json_opt<T: for<'de> Deserialize<'de>>(
    handle: &Handle,
    method: &str,
    args: Value,
) -> Result<Option<T>> {
    let a = api()?;
    let m = CString::new(method)?;
    let args_s = CString::new(args.to_string())?;
    let raw = unsafe { (a.call)(handle.ptr, m.as_ptr(), args_s.as_ptr()) };
    let s = take_c_string(raw)?;
    let wire: Wire =
        serde_json::from_str(&s).with_context(|| format!("decode {method}: {s}"))?;
    if !wire.ok {
        return Err(anyhow!(wire.error.unwrap_or(s)));
    }
    if wire.data.is_null() {
        Ok(None)
    } else {
        Ok(Some(serde_json::from_value(wire.data)?))
    }
}

pub fn call_unit(handle: &Handle, method: &str, args: Value) -> Result<()> {
    let a = api()?;
    let m = CString::new(method)?;
    let args_s = CString::new(args.to_string())?;
    let raw = unsafe { (a.call)(handle.ptr, m.as_ptr(), args_s.as_ptr()) };
    let s = take_c_string(raw)?;
    let wire: Wire = serde_json::from_str(&s)?;
    if !wire.ok {
        return Err(anyhow!(wire.error.unwrap_or(s)));
    }
    Ok(())
}

pub fn call_stream(
    handle: &Handle,
    method: &str,
    args: Value,
    on_event: impl FnMut(&str) + Send + 'static,
) -> Result<()> {
    let a = api()?;
    let m = CString::new(method)?;
    let args_s = CString::new(args.to_string())?;

    struct CbState {
        f: Box<dyn FnMut(&str) + Send>,
    }
    let state = Box::into_raw(Box::new(CbState {
        f: Box::new(on_event),
    }));

    unsafe extern "C" fn trampoline(event_json: *const c_char, user: *mut c_void) {
        if event_json.is_null() || user.is_null() {
            return;
        }
        let s = CStr::from_ptr(event_json).to_string_lossy();
        let state = &mut *(user as *mut CbState);
        (state.f)(&s);
    }

    let raw = unsafe {
        (a.call_stream)(
            handle.ptr,
            m.as_ptr(),
            args_s.as_ptr(),
            trampoline,
            state as *mut c_void,
        )
    };
    let s = take_c_string(raw)?;
    unsafe {
        drop(Box::from_raw(state));
    }
    let wire: Wire = serde_json::from_str(&s)?;
    if !wire.ok {
        return Err(anyhow!(wire.error.unwrap_or(s)));
    }
    Ok(())
}

pub fn spawn_scheduler(handle: &Handle) {
    if let Ok(a) = api() {
        unsafe { (a.spawn)(handle.ptr) }
    }
}

pub fn start_event_pump(handle: &Handle, on_event: impl FnMut(&str) + Send + 'static) {
    let Ok(a) = api() else {
        return;
    };
    struct CbState {
        f: Box<dyn FnMut(&str) + Send>,
    }
    let state = Box::into_raw(Box::new(CbState {
        f: Box::new(on_event),
    }));
    unsafe extern "C" fn trampoline(event_json: *const c_char, user: *mut c_void) {
        if event_json.is_null() || user.is_null() {
            return;
        }
        let s = CStr::from_ptr(event_json).to_string_lossy();
        let state = &mut *(user as *mut CbState);
        (state.f)(&s);
    }
    unsafe {
        (a.pump)(handle.ptr, trampoline, state as *mut c_void);
    }
}

pub fn project_root() -> PathBuf {
    let Ok(a) = api() else {
        return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    };
    let p = unsafe { (a.project_root)() };
    take_c_string(p)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn enriched_path() -> OsString {
    let Ok(a) = api() else {
        return OsString::from(std::env::var_os("PATH").unwrap_or_default());
    };
    let p = unsafe { (a.enriched_path)() };
    take_c_string(p)
        .map(OsString::from)
        .unwrap_or_else(|_| OsString::from(std::env::var_os("PATH").unwrap_or_default()))
}

pub fn args_empty() -> Value {
    json!({})
}
