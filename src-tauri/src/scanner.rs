// use std::path::Path; // Removed unused import
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;
use crate::seven_zip::{analyze_archive, ArchiveInfo};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ScanResult {
    pub path: String,
    pub info: Option<ArchiveInfo>,
    pub error: Option<String>,
    pub status: String, // "Unsupported", "Supported", "Error"
}

pub fn scan_directory(app: AppHandle, dir_path: String, cancel_flag: Arc<AtomicBool>) {
    let walker = WalkDir::new(&dir_path).into_iter();

    let mut count = 0;
    
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        if cancel_flag.load(Ordering::Relaxed) {
             let _ = app.emit("scan-cancelled", ());
             return;
        }

        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    
                    if ["cbr", "cbz", "rar", "zip"].contains(&ext.as_str()) {
                        // Found a candidate
                        count += 1;
                        let _ = app.emit("scan-progress", count); // Simple progress count
                        
                        // Analyze
                        // For cbz/zip, we might assume they are supported unless verified otherwise?
                        // User request: "Identify CBR files that Komga can't read". 
                        // So priority is scanning .cbr. 
                        // But also general scan.
                        
                        // If it's a zip/cbz, it's rarely "RAR5". But we better check content type using 7zz regardless of extension?
                        // Or just trust extension?
                        // User said: "For each .cbr, detect..."
                        
                        let path_str = path.to_string_lossy().to_string();
                        let mut result = ScanResult {
                            path: path_str.clone(),
                            info: None,
                            error: None,
                            status: "Pending".to_string(),
                        };

                        match analyze_archive(&app, path) {
                            Ok(info) => {
                                let mut is_supported = true;
                                if let Some(_) = info.unsupported_reason {
                                    is_supported = false;
                                }
                                // Komga supports standard Zip/Rar4.
                                // If 7zz says it's "Zip", it's likely supported.
                                
                                result.info = Some(info.clone());
                                result.status = if is_supported { "Supported".to_string() } else { "Unsupported".to_string() };
                                
                                // Send result to frontend
                                let _ = app.emit("scan-result", result);
                            }
                            Err(e) => {
                                result.error = Some(e);
                                result.status = "Error".to_string();
                                let _ = app.emit("scan-result", result);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // Log access error?
                eprintln!("Error reading entry: {}", e);
            }
        }
    }
    
    let _ = app.emit("scan-complete", ());
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}
