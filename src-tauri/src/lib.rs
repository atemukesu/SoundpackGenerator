// Copyright Atemukesu
// SPDX-License-Identifier: GPL-3.0

mod generator;
mod settings;

use generator::Region;
use std::path::Path;
use std::process::Command;
use settings::{AppSettings, load_settings, save_settings};

#[tauri::command]
fn check_command_available(cmd: String) -> bool {
    let status = Command::new(&cmd).arg("--version").output();
    match status {
        Ok(output) => output.status.success() || !output.stdout.is_empty() || !output.stderr.is_empty(),
        Err(_) => false,
    }
}

#[tauri::command]
fn check_file_executable(path: String) -> bool {
    let p = Path::new(&path);
    if !p.exists() || !p.is_file() {
        return false;
    }
    #[cfg(windows)]
    {
        let ext = p.extension().and_then(|e| e.to_str());
        ext == Some("exe") || ext == Some("bat") || ext == Some("cmd")
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(p)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
}

#[tauri::command]
fn get_settings() -> Result<AppSettings, String> {
    load_settings().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_settings(s: AppSettings) -> Result<(), String> {
    save_settings(&s).map_err(|e| e.to_string())
}

#[tauri::command]
async fn preview_note(
    #[allow(non_snake_case)] fluidsynthPath: String,
    #[allow(non_snake_case)] sf2Path: String,
    bank: u32,
    preset: u32,
    note: u32,
) -> Result<String, String> {
    generator::preview_note_internal(fluidsynthPath, sf2Path, bank, preset, note)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn preview_existing_sample(
    #[allow(non_snake_case)] appPath: String,
    #[allow(non_snake_case)] instrumentId: String,
    pitch: u32,
) -> Result<String, String> {
    generator::preview_existing_sample_internal(appPath, instrumentId, pitch)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn generate_instrument(
    app: tauri::AppHandle,
    #[allow(non_snake_case)] fluidsynthPath: String,
    #[allow(non_snake_case)] sf2Path: String,
    bank: u32,
    preset: u32,
    #[allow(non_snake_case)] instrumentId: String,
    #[allow(non_snake_case)] noteStart: u32,
    #[allow(non_snake_case)] noteEnd: u32,
    #[allow(non_snake_case)] noteStep: u32,
    #[allow(non_snake_case)] outPath: String,
    gain: f64,
    region: Option<Region>,
    #[allow(non_snake_case)] maxCores: u32,
) -> Result<(), String> {
    generator::generate_instrument_internal(
        app,
        fluidsynthPath,
        sf2Path,
        bank,
        preset,
        instrumentId,
        noteStart,
        noteEnd,
        noteStep,
        outPath,
        gain,
        region,
        maxCores,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn convert_custom_audio(
    #[allow(non_snake_case)] inputPath: String,
    #[allow(non_snake_case)] outPath: String,
    #[allow(non_snake_case)] instrumentId: String,
    pitch: u32,
    gain: f64,
    region: Option<Region>,
) -> Result<(), String> {
    generator::convert_custom_audio_internal(
        inputPath,
        outPath,
        instrumentId,
        pitch,
        gain,
        region,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn reprocess_sample(
    #[allow(non_snake_case)] appPath: String,
    #[allow(non_snake_case)] instrumentId: String,
    pitch: u32,
    gain: f64,
    region: Option<Region>,
) -> Result<(), String> {
    generator::reprocess_sample_internal(
        appPath,
        instrumentId,
        pitch,
        gain,
        region,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_sample(
    #[allow(non_snake_case)] appPath: String,
    #[allow(non_snake_case)] instrumentId: String,
    pitch: u32,
) -> Result<(), String> {
    generator::delete_sample_internal(
        appPath,
        instrumentId,
        pitch,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn validate_soundpack(path: String) -> Result<(), String> {
    generator::validate_soundpack(Path::new(&path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_pack_info(path: String) -> Result<serde_json::Value, String> {
    generator::read_pack_info(Path::new(&path))
        .map(|(name, instruments)| {
            serde_json::json!({
                "displayName": name,
                "availableInstruments": instruments
            })
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn refresh_pack(path: String) -> Result<serde_json::Value, String> {
    generator::refresh_pack_internal(path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_to_zip(
    #[allow(non_snake_case)] appPath: String,
    #[allow(non_snake_case)] outputPath: String,
) -> Result<(), String> {
    generator::export_to_zip_internal(&appPath, &outputPath)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_soundpack(
    #[allow(non_snake_case)] parentPath: String,
    #[allow(non_snake_case)] name: String,
    #[allow(non_snake_case)] description: String,
    #[allow(non_snake_case)] packFormat: u32,
) -> Result<String, String> {
    generator::create_soundpack_internal(
        parentPath,
        name,
        description,
        packFormat,
    )
    .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            check_command_available,
            check_file_executable,
            get_settings,
            set_settings,
            preview_note,
            preview_existing_sample,
            generate_instrument,
            convert_custom_audio,
            reprocess_sample,
            delete_sample,
            create_soundpack,
            validate_soundpack,
            read_pack_info,
            refresh_pack,
            export_to_zip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
