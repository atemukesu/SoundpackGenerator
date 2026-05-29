use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub fluidsynth_path: String,
    #[serde(default)]
    pub ffmpeg_path: String,
}

fn settings_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("无法获取配置目录")?;
    Ok(dir.join("SoundpackGenerator").join("settings.json"))
}

pub fn load_settings() -> Result<AppSettings> {
    let path = settings_path()?;
    if path.exists() {
        let data = fs::read_to_string(&path)?;
        let settings: AppSettings = serde_json::from_str(&data)?;
        Ok(settings)
    } else {
        Ok(AppSettings::default())
    }
}

pub fn validate_path(path: &str) -> Result<()> {
    if path.trim().is_empty() {
        return Ok(());
    }
    let p = PathBuf::from(path);
    if !p.exists() {
        anyhow::bail!("路径不存在: {}", path);
    }
    if !p.is_file() {
        anyhow::bail!("路径不是文件: {}", path);
    }
    Ok(())
}

pub fn save_settings(settings: &AppSettings) -> Result<()> {
    validate_path(&settings.fluidsynth_path)?;
    validate_path(&settings.ffmpeg_path)?;

    let path = settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(settings)?;
    fs::write(&path, data)?;
    Ok(())
}
