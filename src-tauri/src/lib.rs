use tauri::AppHandle; // Removed Manager if unused
use std::sync::{Arc, Mutex, atomic::AtomicBool};

mod seven_zip;
mod scanner;
mod converter;

// use scanner::ScanResult; // Removed unused import

struct AppState {
    scan_cancellation: Arc<AtomicBool>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn scan_directory(app: AppHandle, path: String, state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let cancel = Arc::new(AtomicBool::new(false));
    
    // Update state
    if let Ok(mut s) = state.lock() {
        s.scan_cancellation = cancel.clone();
    }
    
    // Run scan in blocking thread
    let app_handle = app.clone();
    std::thread::spawn(move || {
        scanner::scan_directory(app_handle, path, cancel);
    });
    
    Ok(())
}

#[tauri::command]
fn cancel_scan(state: tauri::State<'_, Arc<Mutex<AppState>>>) {
    if let Ok(s) = state.lock() {
        s.scan_cancellation.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

#[tauri::command]
async fn convert_book(app: AppHandle, path: String) -> Result<String, String> {
    // This could also be long running, maybe wrap in spawn_blocking if it blocks the main loop too much.
    // Tauri async commands run on a separate thread pool anyway, but if we do heavy IO, better offloading or using async IO.
    // Our converter uses blocking 7zz call + blocking zip.
    // Tauri `async fn` is running on a thread, so it's fine for IO, but blocking it for too long might NOT be ideal if the pool is small?
    // Actually tauri command handlers are spawned.
    
    let res = tauri::async_runtime::spawn_blocking(move || {
        converter::convert_file(&app, &path, None)
    }).await.map_err(|e| e.to_string())??;
    
    Ok(res)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(Mutex::new(AppState {
        scan_cancellation: Arc::new(AtomicBool::new(false)),
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![greet, scan_directory, cancel_scan, convert_book])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
