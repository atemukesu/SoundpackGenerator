// Copyright Atemukesu
// SPDX-License-Identifier: GPL-3.0

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use midly::num::{u15, u24, u28, u4, u7};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use std::io::{Read, Write};
use tauri::{AppHandle, Emitter};
use tokio::process::Command;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use zip::ZipWriter;
use zip::write::FileOptions;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct Region {
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackJson {
    #[serde(default = "default_display_name", rename = "displayName")]
    pub display_name: String,
    #[serde(default, rename = "available_instruments")]
    pub available_instruments: serde_json::Map<String, serde_json::Value>,
}

fn default_display_name() -> String {
    "Generated Soundpack".to_string()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn resolve_fluidsynth(path: &str) -> String {
    if path.trim().is_empty() {
        "fluidsynth".to_string()
    } else {
        path.to_string()
    }
}

/// Generate a temporary MIDI file with a single sustained note.
fn make_note_midi(bank: u32, preset: u32, note: u32, duration_ms: u32) -> Result<PathBuf> {
    let tempo = 500_000; // µs per quarter note (120 BPM)
    let ticks_per_beat = 480u16;
    let ticks = ((duration_ms as f64 * 1000.0) / (tempo as f64) * ticks_per_beat as f64) as u32;

    let ch = if bank == 128 { u4::new(9) } else { u4::new(0) };

    let smf = Smf {
        header: Header {
            format: Format::SingleTrack,
            timing: Timing::Metrical(u15::new(ticks_per_beat)),
        },
        tracks: vec![{
            let mut events = Vec::new();

            events.push(TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(tempo))),
            });

            if bank != 0 {
                events.push(TrackEvent {
                    delta: u28::new(0),
                    kind: TrackEventKind::Midi {
                        channel: ch,
                        message: MidiMessage::Controller {
                            controller: u7::new(0),
                            value: u7::new((bank >> 7) as u8 & 0x7F),
                        },
                    },
                });
                events.push(TrackEvent {
                    delta: u28::new(0),
                    kind: TrackEventKind::Midi {
                        channel: ch,
                        message: MidiMessage::Controller {
                            controller: u7::new(32),
                            value: u7::new((bank & 0x7F) as u8),
                        },
                    },
                });
            }

            events.push(TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::ProgramChange {
                        program: u7::new((preset & 0x7F) as u8),
                    },
                },
            });

            events.push(TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::NoteOn {
                        key: u7::new(note as u8),
                        vel: u7::new(100),
                    },
                },
            });

            events.push(TrackEvent {
                delta: u28::new(ticks),
                kind: TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::NoteOff {
                        key: u7::new(note as u8),
                        vel: u7::new(0),
                    },
                },
            });

            events.push(TrackEvent {
                delta: u28::new(0),
                kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
            });

            events
        }],
    };

    let mut buf = Vec::new();
    smf.write(&mut buf).map_err(anyhow::Error::msg)?;
    let temp_dir = std::env::temp_dir();
    let midi_path = temp_dir.join(format!("note_{}_{}.mid", preset, note));
    let mut file = std::fs::File::create(&midi_path)?;
    file.write_all(&buf)?;
    Ok(midi_path)
}

/// Execute a command and capture stdout/stderr, with detailed error reporting.
async fn run_command(cmd: &mut Command, label: &str) -> Result<()> {
    #[cfg(windows)]
    { cmd.as_std_mut().creation_flags(0x08000000); }

    let program = cmd.as_std().get_program().to_string_lossy().to_string();
    let args: Vec<String> = cmd.as_std().get_args().map(|a| a.to_string_lossy().to_string()).collect();
    let cmd_str = if args.is_empty() { program.clone() } else { format!("{} {}", program, args.join(" ")) };

    let output = cmd.output().await.with_context(|| {
        format!("无法启动 {} 进程", label)
    })?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    if !output.status.success() {
        anyhow::bail!(
            "━━━ {} 执行失败 ━━━\n\n命令: {}\n退出码: {}\n\n标准错误输出:\n{}\n\n标准输出:\n{}",
            label, cmd_str, exit_code, stderr, stdout
        );
    }

    Ok(())
}

/// Render a single MIDI note to a WAV file via fluidsynth.
async fn render_note_wav(
    fs_cmd: &str,
    sf2_path: &str,
    bank: u32,
    preset: u32,
    note: u32,
    duration_ms: u32,
    out_wav: &Path,
) -> Result<()> {
    let midi_file = make_note_midi(bank, preset, note, duration_ms)?;

    let mut cmd = Command::new(fs_cmd);
    cmd.arg("-ni").arg("-F").arg(out_wav).arg(sf2_path).arg(&midi_file);
    run_command(&mut cmd, "FluidSynth").await?;

    let _ = fs::remove_file(&midi_file);

    Ok(())
}

/// Convert a WAV (or any audio) to mono OGG Vorbis via ffmpeg, with optional
/// region trim and gain adjustment.
async fn convert_to_ogg(
    input: &Path,
    output: &Path,
    region: Option<&Region>,
    gain: f64,
) -> Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y").arg("-i").arg(input);

    let mut filters: Vec<String> = Vec::new();

    if let Some(r) = region {
        let duration = r.end - r.start;
        cmd.arg("-ss").arg(r.start.to_string());
        cmd.arg("-t").arg(duration.to_string());
        let fade_start = (duration - 0.05).max(0.0);
        filters.push(format!("afade=t=out:st={fade_start}:d=0.05"));
    }

    if gain.abs() > f64::EPSILON {
        filters.push(format!("volume={gain}dB"));
    }

    if !filters.is_empty() {
        cmd.arg("-af").arg(filters.join(","));
    }

    cmd.arg("-ac").arg("1");
    cmd.arg("-c:a").arg("libvorbis");
    cmd.arg("-q:a").arg("5");
    cmd.arg(output);

    run_command(&mut cmd, "FFmpeg").await?;

    Ok(())
}

/// Read or create a `pack.json` at the given base path.
fn read_or_create_pack(base: &Path) -> PackJson {
    let pack_path = base.join("pack.json");
    if pack_path.exists() {
        if let Ok(data) = fs::read_to_string(&pack_path) {
            if let Ok(p) = serde_json::from_str::<PackJson>(&data) {
                return p;
            }
        }
    }
    PackJson {
        display_name: default_display_name(),
        available_instruments: serde_json::Map::new(),
    }
}

/// Read `pack.json` and return display name and instrument map.
pub fn read_pack_info(path: &Path) -> Result<(String, serde_json::Map<String, serde_json::Value>)> {
    let pack_path = path.join("pack.json");
    let data = fs::read_to_string(&pack_path).context("Failed to read pack.json")?;
    let pack: PackJson = serde_json::from_str(&data).context("Invalid pack.json")?;
    Ok((pack.display_name, pack.available_instruments))
}

/// Write `pack.json` back to disk.
fn write_pack(base: &Path, pack: &PackJson) -> Result<()> {
    let json = serde_json::to_string_pretty(pack)?;
    fs::write(base.join("pack.json"), json)?;
    Ok(())
}

/// Ensure `pack.mcmeta` exists.
fn ensure_mcmeta(base: &Path) -> Result<()> {
    let mcmeta = base.join("pack.mcmeta");
    if !mcmeta.exists() {
        let content = r#"{
  "pack": {
    "pack_format": 15,
    "description": "Generated by Soundpack Generator"
  }
}"#;
        fs::write(&mcmeta, content)?;
    }
    Ok(())
}

/// Update the `available_instruments` map for a given instrument id.
/// Merges new notes into any existing entry (deduped & sorted).
fn upsert_instrument(pack: &mut PackJson, instrument_id: &str, new_notes: &[u32]) {
    let mut notes: Vec<u32> = Vec::new();

    // Merge with existing entry if present
    if let Some(existing) = pack.available_instruments.get(instrument_id) {
        if let Some(arr) = existing.as_array() {
            for v in arr {
                if let Some(n) = v.as_u64() {
                    notes.push(n as u32);
                }
            }
        }
    }

    for &n in new_notes {
        if !notes.contains(&n) {
            notes.push(n);
        }
    }
    notes.sort();

    let value = serde_json::Value::Array(
        notes
            .into_iter()
            .map(|n| serde_json::Value::Number(n.into()))
            .collect(),
    );
    pack.available_instruments
        .insert(instrument_id.to_string(), value);
}

// ---------------------------------------------------------------------------
// Public API – called from lib.rs Tauri commands
// ---------------------------------------------------------------------------

/// Preview a single note: render 4 s of audio via fluidsynth and return a
/// data-URI with the WAV encoded as base64.
pub async fn preview_note_internal(
    fluidsynth_path: String,
    sf2_path: String,
    bank: u32,
    preset: u32,
    note: u32,
) -> Result<String> {
    let fs_cmd = resolve_fluidsynth(&fluidsynth_path);
    let temp_dir = std::env::temp_dir();
    let out_wav = temp_dir.join("soundpack_preview.wav");

    render_note_wav(&fs_cmd, &sf2_path, bank, preset, note, 4000, &out_wav).await?;

    let wav_data = fs::read(&out_wav)?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&wav_data);
    Ok(format!("data:audio/wav;base64,{}", b64))
}

/// Preview an existing sample file from the soundpack.
pub async fn preview_existing_sample_internal(
    app_path: String,
    instrument_id: String,
    pitch: u32,
) -> Result<String> {
    let base = PathBuf::from(&app_path);
    let sounds_dir = base.join("assets/extendednoteblock/sounds/notes");
    let ogg_path = sounds_dir.join(format!("{}.{}.ogg", instrument_id, pitch));

    if !ogg_path.exists() {
        anyhow::bail!("Sample file not found: {}", ogg_path.display());
    }

    let ogg_data = fs::read(&ogg_path)?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&ogg_data);
    Ok(format!("data:audio/ogg;base64,{}", b64))
}

/// Generate OGG files for every note in [noteStart, noteEnd] from an SF2
/// instrument, emitting progress events along the way, and updating
/// `pack.json` + `pack.mcmeta`.
pub async fn generate_instrument_internal(
    app: AppHandle,
    fluidsynth_path: String,
    sf2_path: String,
    bank: u32,
    preset: u32,
    instrument_id: String,
    note_start: u32,
    note_end: u32,
    note_step: u32,
    out_path: String,
    gain: f64,
    region: Option<Region>,
    max_cores: u32,
) -> Result<()> {
    let base = PathBuf::from(&out_path);
    let sounds_dir = base.join("assets/extendednoteblock/sounds/notes");
    fs::create_dir_all(&sounds_dir).context("Failed to create output directories")?;

    let fs_cmd = resolve_fluidsynth(&fluidsynth_path);
    let notes: Vec<u32> = (note_start..=note_end).step_by(note_step.max(1) as usize).collect();

    let max_concurrent = if max_cores > 0 { max_cores as usize } else { num_cpus::get().min(4) };
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));
    let completed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut handles = Vec::new();
    for &note in &notes {
        let fs_cmd = fs_cmd.clone();
        let sf2_path = sf2_path.clone();
        let instrument_id = instrument_id.clone();
        let sounds_dir = sounds_dir.clone();
        let region = region.clone();
        let semaphore = semaphore.clone();
        let completed = completed.clone();
        let app = app.clone();
        let total_notes = notes.len();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let temp_dir = std::env::temp_dir();
            let temp_wav = temp_dir.join(format!("spg_{}_{}.wav", instrument_id, note));
            let final_ogg = sounds_dir.join(format!("{}.{}.ogg", instrument_id, note));

            render_note_wav(&fs_cmd, &sf2_path, bank, preset, note, 20000, &temp_wav).await?;
            convert_to_ogg(&temp_wav, &final_ogg, region.as_ref(), gain).await?;
            let _ = fs::remove_file(&temp_wav);

            let done = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            let progress = (done as f64 / total_notes as f64) * 100.0;
            let _ = app.emit("generation-progress", progress);

            Ok::<_, anyhow::Error>(note)
        });
        handles.push(handle);
    }

    let mut generated_notes = Vec::new();
    for handle in handles {
        let result = handle.await.context("Task panicked")??;
        generated_notes.push(result);
    }

    generated_notes.sort();

    // Update pack.json
    let mut pack = read_or_create_pack(&base);
    upsert_instrument(&mut pack, &instrument_id, &generated_notes);
    write_pack(&base, &pack)?;

    // Ensure pack.mcmeta
    ensure_mcmeta(&base)?;

    Ok(())
}

/// Re-process an existing sample file with new region and gain settings.
pub async fn reprocess_sample_internal(
    app_path: String,
    instrument_id: String,
    pitch: u32,
    new_gain: f64,
    new_region: Option<Region>,
) -> Result<()> {
    let base = PathBuf::from(&app_path);
    let sounds_dir = base.join("assets/extendednoteblock/sounds/notes");
    let ogg_path = sounds_dir.join(format!("{}.{}.ogg", instrument_id, pitch));

    if !ogg_path.exists() {
        anyhow::bail!("Sample file not found: {}", ogg_path.display());
    }

    let temp_dir = std::env::temp_dir();
    let temp_wav = temp_dir.join(format!("spg_reprocess_{}_{}.wav", instrument_id, pitch));

    {
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y").arg("-i").arg(&ogg_path).arg(&temp_wav);
        run_command(&mut cmd, "FFmpeg 解码").await?;
    }

    let new_ogg_path = ogg_path;
    convert_to_ogg(&temp_wav, &new_ogg_path, new_region.as_ref(), new_gain).await?;
    let _ = fs::remove_file(&temp_wav);

    Ok(())
}

/// Delete a sample file and update pack.json.
pub async fn delete_sample_internal(
    app_path: String,
    instrument_id: String,
    pitch: u32,
) -> Result<()> {
    let base = PathBuf::from(&app_path);
    let sounds_dir = base.join("assets/extendednoteblock/sounds/notes");
    let ogg_path = sounds_dir.join(format!("{}.{}.ogg", instrument_id, pitch));

    if ogg_path.exists() {
        fs::remove_file(&ogg_path).context("Failed to delete sample file")?;
    }

    let mut pack = read_or_create_pack(&base);
    if let Some(existing) = pack.available_instruments.get_mut(&instrument_id) {
        if let Some(arr) = existing.as_array_mut() {
            arr.retain(|v| {
                if let Some(n) = v.as_u64() {
                    n != pitch as u64
                } else {
                    true
                }
            });
        }
    }
    write_pack(&base, &pack)?;

    Ok(())
}

/// Validate that a directory is a valid soundpack structure.
pub fn validate_soundpack(path: &Path) -> Result<()> {
    if !path.is_dir() {
        anyhow::bail!("路径不是有效的目录");
    }

    let required_files = [
        "pack.mcmeta",
        "pack.json",
        "assets/extendednoteblock/sounds/notes",
    ];

    for file in &required_files {
        let full_path = path.join(file);
        if !full_path.exists() {
            anyhow::bail!("缺少必需文件: {}", file);
        }
    }

    // Validate pack.json is valid JSON
    let pack_path = path.join("pack.json");
    if let Ok(data) = fs::read_to_string(&pack_path) {
        if serde_json::from_str::<PackJson>(&data).is_err() {
            anyhow::bail!("pack.json 格式无效");
        }
    }

    Ok(())
}

/// Create a new soundpack directory structure with pack.mcmeta and pack.json.
pub fn create_soundpack_internal(
    parent_path: String,
    name: String,
    description: String,
    pack_format: u32,
) -> Result<String> {
    let parent = PathBuf::from(&parent_path);
    let base = parent.join(&name);

    if base.exists() {
        anyhow::bail!("音色包文件夹已存在: {}", base.display());
    }

    // Create directory structure
    let sounds_dir = base.join("assets/extendednoteblock/sounds/notes");
    fs::create_dir_all(&sounds_dir).context("Failed to create soundpack directories")?;

    // Create pack.mcmeta
    let mcmeta_content = serde_json::json!({
        "pack": {
            "pack_format": pack_format,
            "description": description
        }
    });
    let mcmeta_json = serde_json::to_string_pretty(&mcmeta_content)?;
    fs::write(base.join("pack.mcmeta"), mcmeta_json)?;

    // Create pack.json
    let pack = PackJson {
        display_name: name.clone(),
        available_instruments: serde_json::Map::new(),
    };
    write_pack(&base, &pack)?;

    Ok(base.to_string_lossy().to_string())
}

/// Scan the notes directory and rebuild available_instruments in pack.json.
pub fn refresh_pack_internal(app_path: String) -> Result<serde_json::Value> {
    let base = PathBuf::from(&app_path);
    let sounds_dir = base.join("assets/extendednoteblock/sounds/notes");

    if !sounds_dir.exists() {
        anyhow::bail!("音频目录不存在");
    }

    let mut instruments: std::collections::BTreeMap<u32, Vec<u32>> = std::collections::BTreeMap::new();

    for entry in fs::read_dir(&sounds_dir)? {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        if filename_str.ends_with(".ogg") {
            let parts: Vec<&str> = filename_str.split('.').collect();
            if parts.len() == 3 {
                if let (Ok(inst_id), Ok(pitch)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    instruments.entry(inst_id).or_default().push(pitch);
                }
            }
        }
    }

    for notes in instruments.values_mut() {
        notes.sort();
        notes.dedup();
    }

    let mut pack = read_or_create_pack(&base);
    pack.available_instruments = serde_json::Map::new();

    for (inst_id, notes) in &instruments {
        let value = serde_json::Value::Array(
            notes.iter().map(|&n| serde_json::Value::Number(n.into())).collect(),
        );
        pack.available_instruments.insert(inst_id.to_string(), value);
    }

    write_pack(&base, &pack)?;

    Ok(serde_json::json!({
        "displayName": pack.display_name,
        "availableInstruments": pack.available_instruments
    }))
}

/// Convert a single custom audio file to mono OGG and place it in the
/// soundpack output, then update `pack.json`.
pub async fn convert_custom_audio_internal(
    input_path: String,
    out_path: String,
    instrument_id: String,
    pitch: u32,
    gain: f64,
    region: Option<Region>,
) -> Result<()> {
    let base = PathBuf::from(&out_path);
    let sounds_dir = base.join("assets/extendednoteblock/sounds/notes");
    fs::create_dir_all(&sounds_dir).context("Failed to create output directories")?;

    let input = PathBuf::from(&input_path);
    let final_ogg = sounds_dir.join(format!("{}.{}.ogg", instrument_id, pitch));

    convert_to_ogg(&input, &final_ogg, region.as_ref(), gain).await?;

    // Update pack.json
    let mut pack = read_or_create_pack(&base);
    upsert_instrument(&mut pack, &instrument_id, &[pitch]);
    write_pack(&base, &pack)?;

    // Ensure pack.mcmeta
    ensure_mcmeta(&base)?;

    Ok(())
}

/// Export a soundpack directory to a ZIP file.
pub fn export_to_zip_internal(app_path: &str, output_path: &str) -> Result<()> {
    let base = Path::new(app_path);
    if !base.is_dir() {
        anyhow::bail!("音色包目录不存在");
    }

    let zip_file = fs::File::create(output_path)?;
    let mut zip = ZipWriter::new(zip_file);

    add_dir_to_zip(&mut zip, base, base)?;

    zip.finish()?;
    Ok(())
}

fn add_dir_to_zip(
    zip: &mut ZipWriter<fs::File>,
    dir: &Path,
    base: &Path,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap();
        let name = relative.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            zip.add_directory::<&str, ()>(&name, FileOptions::default())?;
            add_dir_to_zip(zip, &path, base)?;
        } else {
            zip.start_file::<&str, ()>(&name, FileOptions::default())?;
            let mut file = fs::File::open(&path)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    Ok(())
}
