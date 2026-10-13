mod scan;

use scan::{Node, Progress};
use serde::Serialize;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

#[derive(Default)]
struct App {
    result: Mutex<Option<(PathBuf, Node)>>,
    progress: Mutex<Arc<Progress>>,
}

#[derive(Serialize)]
struct Volume {
    total: u64,
    free: u64,
}

fn volume(path: &str) -> Option<Volume> {
    let c = std::ffi::CString::new(path).ok()?;
    let mut s: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statfs(c.as_ptr(), &mut s) } != 0 {
        return None;
    }
    let bs = s.f_bsize as u64;
    Some(Volume { total: s.f_blocks * bs, free: s.f_bavail * bs })
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/".into())
}

#[tauri::command]
fn places() -> Value {
    let h = home();
    json!({
        "home": h,
        "volume": volume("/System/Volumes/Data").or_else(|| volume("/")),
    })
}

/// Startet den Scan im Hintergrund. Fortschritt kommt als Event `scan-progress`,
/// das Ende als `scan-done`.
#[tauri::command]
fn start_scan(path: String, app: AppHandle, state: State<'_, Arc<App>>) {
    let p = Arc::new(Progress::default());
    let old = std::mem::replace(&mut *state.progress.lock().unwrap(), p.clone());
    old.cancel.store(true, std::sync::atomic::Ordering::Relaxed); // laufenden Scan abbrechen
    let st = state.inner().clone();
    let started = Instant::now();

    let (pp, ap) = (p.clone(), app.clone());
    let ticker = std::thread::spawn(move || {
        while !pp.cancel.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = ap.emit("scan-progress", tick(&pp, started));
            std::thread::sleep(Duration::from_millis(100));
        }
    });

    std::thread::spawn(move || {
        let root = PathBuf::from(&path);
        let node = scan::scan(&root, &p);
        let cancelled = p.cancel.swap(true, std::sync::atomic::Ordering::Relaxed); // stoppt auch den Ticker
        let _ = ticker.join();
        let mut done = tick(&p, started);
        done["cancelled"] = json!(cancelled);
        done["volume"] = json!(volume(&path));
        *st.result.lock().unwrap() = Some((root, node));
        let _ = app.emit("scan-done", done);
    });
}

fn tick(p: &Progress, started: Instant) -> Value {
    use std::sync::atomic::Ordering::Relaxed;
    json!({
        "files": p.files.load(Relaxed),
        "bytes": p.bytes.load(Relaxed),
        "denied": p.denied.load(Relaxed),
        "current": p.current.lock().map(|c| c.clone()).unwrap_or_default(),
        "secs": started.elapsed().as_secs_f64(),
    })
}

#[tauri::command]
fn cancel_scan(state: State<'_, Arc<App>>) {
    state.progress.lock().unwrap().cancel.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Knoten `id` mit `depth` Ebenen Kindern. Pro Ebene höchstens `limit` Kinder; alles darunter
/// und alles kleiner als `min_frac` der Knotengröße wird zu „N weitere“ zusammengefasst.
#[tauri::command]
fn tree(id: Vec<u32>, depth: u32, limit: usize, min_frac: f64, state: State<'_, Arc<App>>) -> Option<Value> {
    let r = state.result.lock().unwrap();
    let (_, root) = r.as_ref()?;
    let n = scan::find(root, &id)?;
    Some(to_json(n, id, depth, limit, (n.size as f64 * min_frac) as u64))
}

fn to_json(n: &Node, id: Vec<u32>, depth: u32, limit: usize, min: u64) -> Value {
    let mut v = json!({ "id": id, "name": n.name, "size": n.size, "files": n.files, "dir": n.dir, "count": n.children.len() });
    if depth > 0 && !n.children.is_empty() {
        let shown: Vec<Value> = n
            .children
            .iter()
            .take(limit)
            .take_while(|c| c.size >= min && c.size > 0)
            .enumerate()
            .map(|(i, c)| {
                let mut cid = id.clone();
                cid.push(i as u32);
                to_json(c, cid, depth - 1, limit, min)
            })
            .collect();
        let rest = &n.children[shown.len()..];
        let mut kids = shown;
        if !rest.is_empty() {
            kids.push(json!({
                "id": null, "rest": true, "name": format!("{} weitere", rest.len()),
                "size": rest.iter().map(|c| c.size).sum::<u64>(),
                "files": rest.iter().map(|c| c.files).sum::<u64>(), "dir": false, "count": 0
            }));
        }
        v["children"] = json!(kids);
    }
    v
}

#[tauri::command]
fn full_path(id: Vec<u32>, state: State<'_, Arc<App>>) -> Option<String> {
    let r = state.result.lock().unwrap();
    let (path, root) = r.as_ref()?;
    let mut p = path.clone();
    let mut n = root;
    for i in id {
        n = n.children.get(i as usize)?;
        p.push(&*n.name);
    }
    Some(p.to_string_lossy().into_owned())
}

#[tauri::command]
fn largest_files(n: usize, state: State<'_, Arc<App>>) -> Vec<Value> {
    let r = state.result.lock().unwrap();
    let Some((path, root)) = r.as_ref() else { return vec![] };
    scan::largest_files(root, n)
        .into_iter()
        .map(|(size, id)| {
            let mut p = PathBuf::new();
            let mut node = root;
            for &i in &id {
                node = &node.children[i as usize];
                p.push(&*node.name);
            }
            json!({ "id": id, "name": node.name, "dir": p.parent().map(|d| path.join(d)), "size": size })
        })
        .collect()
}

/// Nur für Tests: `SB_AUTOSCAN=/pfad` startet den Scan direkt (nicht in Release-Builds).
#[tauri::command]
fn dev_autoscan() -> Option<String> {
    cfg!(debug_assertions).then(|| std::env::var("SB_AUTOSCAN").ok()).flatten()
}

#[tauri::command]
fn frontend_log(msg: String) {
    eprintln!("[frontend] {msg}");
}

#[tauri::command]
fn open_privacy_settings() {
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
        .spawn();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Arc::new(App::default()))
        .invoke_handler(tauri::generate_handler![
            places,
            start_scan,
            cancel_scan,
            tree,
            full_path,
            largest_files,
            open_privacy_settings,
            dev_autoscan,
            frontend_log
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
