use std::path::Path;
use tauri::AppHandle; // Removed Manager
use tauri_plugin_shell::ShellExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveInfo {
    pub file_type: String, // "Rar5", "Rar", "Zip", etc.
    pub is_solid: bool,
    pub is_encrypted: bool,
    pub image_count: usize,
    pub unsupported_reason: Option<String>,
}

// get_7zz_command removed

// Function to analyze a file using 7zz l -slt
pub fn analyze_archive(app: &AppHandle, file_path: &Path) -> Result<ArchiveInfo, String> {
    println!("Scanning: {}", file_path.display()); // Log start

    let sidecar_command = app.shell().sidecar("7zz")
        .map_err(|e| format!("Failed to create sidecar command: {}", e))?
        .args(["l", "-slt", "-y", file_path.to_str().unwrap_or("")]); // Removed -sccUTF-8 just in case

    let output = tauri::async_runtime::block_on(async {
        sidecar_command.output().await
    }).map_err(|e| format!("Failed to execute 7zz: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    let parse_result = parse_7zz_output(&stdout);

    // If we successfully parsed valid archive info, return it regardless of exit code
    // (7zz often returns Code 2 for minor header errors which, while "Fatal" to 7zz strictness, often leave the listing intact)
    if let Ok(info) = parse_result {
        println!("Scanned {}: Type={}, Images={}, Solid={}", file_path.display(), info.file_type, info.image_count, info.is_solid);
        return Ok(info);
    }

    let exit_code = output.status.code().unwrap_or(-1);
    
    // If parsing failed AND we have an error code, report the robust error
    if exit_code > 0 {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // We already have stdout
        println!("Error scanning {}: Code {}. Stdout: {}", file_path.display(), exit_code, stdout.chars().take(100).collect::<String>());
        return Err(format!("7zz failed with code {}. Stderr: '{}'. Stdout trace: '{}'", exit_code, stderr.trim(), stdout.trim().chars().take(200).collect::<String>()));
    }

    // If exit code 0 but parsing failed (e.g. empty output)
    parse_result
}

fn parse_7zz_output(output: &str) -> Result<ArchiveInfo, String> {
    let mut file_type = String::new();
    let mut is_solid = false;
    let mut is_encrypted = false;
    let mut image_count = 0;
    
    // Simple state parser
    // First block after "Listing archive: ..." is the archive info.
    // Subsequent blocks are file info.
    
    let blocks: Vec<&str> = output.split("\n\n").collect();
    
    if blocks.is_empty() {
        // Fallback: if we don't have double newlines, try treating the whole thing as one block (rare but possible for single file)
        if !output.trim().is_empty() {
             // proceed with single block logic?
             // Actually, let's just warn and rely on the robust loop below.
        } else {
            return Err("Empty output from 7zz".to_string());
        }
    }

    let mut archive_props_found = false;

    for block in blocks {
        if block.trim().is_empty() { continue; }
        
        let lines: Vec<&str> = block.lines().collect();
        let mut props = std::collections::HashMap::new();
        
        for line in &lines {
            // More robust parsing: split by first '=', then trim. 
            // 7z usually outputs "Key = Value", but "Key=Value" is possible in some environments/versions.
            if let Some((key_raw, value_raw)) = line.split_once('=') {
                let key = key_raw.trim();
                let value = value_raw.trim();
                if !key.is_empty() {
                    props.insert(key, value);
                }
            }
        }

        if props.contains_key("Type") && !archive_props_found {
            if let Some(t) = props.get("Type") {
                // This is the archive block
                file_type = t.to_string();
                
                if let Some(solid) = props.get("Solid") {
                    is_solid = *solid == "+";
                }
                
                // Block-level or Header encryption
                if let Some(enc) = props.get("Encrypted") {
                    if *enc == "+" {
                        is_encrypted = true;
                    }
                }
                
                archive_props_found = true;
                // Don't continue, we want to scan file blocks too for images
                // But we found the archive block.
            }
        }
        
        // If it's a file block
        if let Some(path) = props.get("Path") {
            // Check based on file extension
            let clean_path = path.to_lowercase();
            if clean_path.ends_with(".jpg") 
                || clean_path.ends_with(".jpeg") 
                || clean_path.ends_with(".png") 
                || clean_path.ends_with(".webp") 
                || clean_path.ends_with(".gif") {
                image_count += 1;
            }
        }
    }
    
    // Fallback: If file_type is still empty, try to fuzzy find "Type = Rar5" in the raw string?
    // This handles edge cases where block splitting failed.
    if file_type.is_empty() {
         if output.contains("Type = Rar5") {
             file_type = "Rar5".to_string();
         } else if output.contains("Type = Rar") {
             file_type = "Rar".to_string();
         }
    }
    
    // Debugging: If file_type is empty, print the blocks to see what went wrong
    if file_type.is_empty() {
        println!("DEBUG: Type is empty. Dumping raw output:\n{}", output);
    }

    println!("DEBUG: Parsed Type='{}', Solid={}", file_type, is_solid);

    // Detection logic
    let mut unsupported_reason = None;
    
    // Check case insensitive
    if file_type.eq_ignore_ascii_case("rar5") {
        unsupported_reason = Some("RAR5 format".to_string());
    } else if is_solid {
        unsupported_reason = Some("Solid archive".to_string());
    }
    
    Ok(ArchiveInfo {
        file_type,
        is_solid,
        is_encrypted,
        image_count,
        unsupported_reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rar5_solid() {
        let output = r#"Listing archive: test.rar

--
Path = test.rar
Type = Rar5
Physical Size = 1048576
Solid = +
Blocks = 1
Multivolume = -
Volume = -
Encrypted = -

----------
Path = cover.jpg
Folder = -
Size = 123456
Packed Size = 123456
Modified = 2023-01-01 12:00:00
Attributes = A
CRC = ABCDEF01
Encrypted = -
Method = LZMA2:24

----------
Path = page01.png
Folder = -
"#;
        let info = parse_7zz_output(output).unwrap();
        assert_eq!(info.file_type, "Rar5");
        assert!(info.is_solid);
        assert!(!info.is_encrypted);
        assert_eq!(info.image_count, 2);
        assert_eq!(info.unsupported_reason, Some("RAR5 format".to_string()));
    }
}
