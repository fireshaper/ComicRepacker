use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use walkdir::WalkDir;
use zip::write::FileOptions;

pub fn convert_file(app: &AppHandle, file_path_str: &str, output_dir: Option<String>) -> Result<String, String> {
    let file_path = Path::new(file_path_str);
    if !file_path.exists() {
        return Err("File not found".to_string());
    }

    // 1. Create temp directory
    let temp_dir = app.path().temp_dir().map_err(|e| e.to_string())?.join("comics-scanner-conversion").join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    // 2. Extract using 7zz
    // sidecar("7zz").args(["x", "-y", format!("-o{}", temp_dir.display()).as_str(), file_path_str])
    let sidecar_command = app.shell().sidecar("7zz")
        .map_err(|e| format!("Failed to create 7zz command: {}", e))?
        .args(["x", "-y", &format!("-o{}", temp_dir.display()), file_path_str]);

    let output = tauri::async_runtime::block_on(async {
        sidecar_command.output().await
    }).map_err(|e| format!("Failed to run 7zz: {}", e))?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_dir);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Extraction failed: {}", stderr));
    }

    // 3. Zip to .cbz
    // Determine output path
    let file_stem = file_path.file_stem().unwrap().to_string_lossy();
    let parent_dir = if let Some(d) = output_dir {
        PathBuf::from(d)
    } else {
        file_path.parent().unwrap().to_path_buf()
    };
    
    // Ensure output filename doesn't conflict? 
    // We want file.cbz.
    let output_file_path = parent_dir.join(format!("{}.cbz", file_stem));
    
    // Create zip file
    let file = File::create(&output_file_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated).unix_permissions(0o755);

    let walker = WalkDir::new(&temp_dir).into_iter();
    let mut buffer = Vec::new();

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        // Skip directories themselves (ZipWriter usually handles them implicitly or explicit add_directory)
        // But for standard CBZ, we just add files.
        if path.is_file() {
            let name = path.strip_prefix(&temp_dir).unwrap();
            let name_str = name.to_str().ok_or("Invalid UTF-8 in extracted filename")?;
            
            // For Windows compatibility, ensure forward slashes in zip
            let zip_path = name_str.replace("\\", "/");

            zip.start_file(zip_path, options).map_err(|e| format!("Zip error: {}", e))?;
            let mut f = File::open(path).map_err(|e| format!("Read extracted file error: {}", e))?;
            f.read_to_end(&mut buffer).map_err(|e| format!("Read error: {}", e))?;
            zip.write_all(&buffer).map_err(|e| format!("Write error: {}", e))?;
            buffer.clear();
        }
    }

    zip.finish().map_err(|e| format!("Failed to finish zip: {}", e))?;

    // 4. Cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    Ok(output_file_path.to_string_lossy().to_string())
}
