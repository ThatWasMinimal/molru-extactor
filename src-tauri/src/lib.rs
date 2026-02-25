use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;

#[tauri::command]
fn get_default_extracted_location() -> Result<String, String> {
    std::env::current_dir()
        .map(|d| d.join("lr_extracted").to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn extract_batch(paths: Vec<String>, output_root: Option<String>) -> Result<String, String> {
    let output_root = match output_root {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join("lr_extracted"),
    };

    fs::create_dir_all(&output_root).map_err(|e| e.to_string())?;

    for path in paths {
        let input = PathBuf::from(path);
        if input.exists() {
            process_file(&input, &output_root).map_err(|e| e.to_string())?;
        }
    }

    Ok("Extraction complete".into())
}

#[tauri::command]
fn build_info() -> String {
    format!(
        "{} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH")
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![extract_batch, build_info, get_default_extracted_location])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_unique_subfolder(base: &Path, ext: &str) -> PathBuf {
    let target = base.join(ext);
    if !target.exists() {
        return target;
    }

    let mut c = 1;
    loop {
        let candidate = base.join(format!("{} ({})", ext, c));
        if !candidate.exists() {
            return candidate;
        }
        c += 1;
    }
}

fn find_jpeg_end(data: &[u8], start: usize) -> Option<usize> {
    let mut p = start + 2;
    let max_scan = usize::min(data.len(), start + 20 * 1024 * 1024);

    while p + 1 < max_scan {
        if data[p] == 0xFF && data[p + 1] == 0xD9 {
            return Some(p + 2);
        }
        p += 1;
    }
    None
}

fn calc_ogg_length(data: &[u8], start: usize) -> usize {
    let mut p = start;
    while p + 27 < data.len() {
        if &data[p..p + 4] != b"OggS" {
            break;
        }

        let segments = data[p + 26] as usize;
        let mut size = 27 + segments;

        for i in 0..segments {
            size += data[p + 27 + i] as usize;
        }

        let flags = data[p + 5];
        p += size;

        if flags & 0x04 != 0 {
            break;
        }
    }
    p - start
}

fn scan_until_next_header(data: &[u8], start: usize) -> usize {
    let sigs: [&[u8]; 5] = [b"RIFF", b"OggS", b"\xFF\xD8\xFF", b"\x89PNG", b"UnityFS"];
    let mut p = start + 4;
    let limit = usize::min(data.len(), start + 50 * 1024 * 1024);

    while p + 4 < limit {
        for s in &sigs {
            if data[p..].starts_with(s) {
                return p - start;
            }
        }
        p += 1;
    }

    limit - start
}

fn process_file(input: &Path, output_root: &Path) -> Result<()> {
    let mut file = File::open(input)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let file_name = input.file_stem().unwrap().to_string_lossy();
    let base_output = output_root.join(file_name.to_string());
    fs::create_dir_all(&base_output)?;

    let mut pos = 0usize;
    let mut stats: HashMap<String, usize> = HashMap::new();
    let mut session_folders: HashMap<String, PathBuf> = HashMap::new();

    while pos + 16 < data.len() {
        let header = &data[pos..pos + 16];
        let mut found_ext: Option<&str> = None;
        let mut file_len: usize = 0;

        if header.starts_with(b"RIFF") {
            let size =
                u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize + 8;

            match &header[8..12] {
                b"WAVE" => found_ext = Some("wav"),
                b"WEBP" => found_ext = Some("webp"),
                _ => {}
            }
            file_len = size;
        } else if header.starts_with(b"\x89PNG\r\n\x1a\n") {
            if let Some(end) = data[pos..].windows(4).position(|w| w == b"IEND") {
                found_ext = Some("png");
                file_len = end + 8;
            }
        } else if header.starts_with(b"OggS") {
            found_ext = Some("ogg");
            file_len = calc_ogg_length(&data, pos);
        } else if header.starts_with(b"\xFF\xD8\xFF") {
            if let Some(end) = find_jpeg_end(&data, pos) {
                found_ext = Some("jpg");
                file_len = end - pos;
            }
        } else if header.starts_with(b"\x1A\x45\xDF\xA3") {
            found_ext = Some("webm");
            file_len = scan_until_next_header(&data, pos);
        } else if header.starts_with(b"ID3") {
            found_ext = Some("mp3");
            file_len = scan_until_next_header(&data, pos);
        } else if header.starts_with(b"UnityFS") {
            found_ext = Some("assets");
            file_len = scan_until_next_header(&data, pos);
        }

        if let Some(ext) = found_ext {
            if file_len > 128 && pos + file_len <= data.len() {
                let folder = session_folders.entry(ext.to_string()).or_insert_with(|| {
                    let unique = get_unique_subfolder(&base_output, ext);
                    fs::create_dir_all(&unique).unwrap();
                    unique
                });

                let counter = stats.entry(ext.to_string()).or_insert(0);
                *counter += 1;

                let out_path = folder.join(format!("file_{}.{}", counter, ext));
                let mut out = File::create(out_path)?;
                out.write_all(&data[pos..pos + file_len])?;

                pos += file_len;
                continue;
            }
        }

        pos += 1;
    }

    Ok(())
}
