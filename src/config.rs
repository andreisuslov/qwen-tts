use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::models;
use crate::platform::{self, Backend};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub python_path: String,
    pub models_dir: String,
    pub voices_dir: String,
    pub output_dir: String,
    pub backend: Backend,
    pub default_voice: String,
    pub default_speed: f32,
    pub auto_play: bool,
    pub model_variant: String,
}

impl Default for Config {
    fn default() -> Self {
        let base = base_dir();
        let python_bin = if cfg!(target_os = "windows") {
            base.join("venv").join("Scripts").join("python.exe")
        } else {
            base.join("venv").join("bin").join("python")
        };

        Self {
            python_path: python_bin.to_string_lossy().to_string(),
            models_dir: base.join("models").to_string_lossy().to_string(),
            voices_dir: base.join("voices").to_string_lossy().to_string(),
            output_dir: base.join("outputs").to_string_lossy().to_string(),
            backend: platform::detect_backend(),
            default_voice: "Vivian".to_string(),
            default_speed: 1.0,
            auto_play: true,
            model_variant: "pro".to_string(),
        }
    }
}

/// ~/.qwen-tts
pub fn base_dir() -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".qwen-tts")
}

/// ~/.config/qwen-tts/config.toml
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .expect("could not determine config directory")
        .join("qwen-tts")
        .join("config.toml")
}

pub fn load() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        // Auto-initialize on first use
        let cfg = Config::default();
        ensure_dirs(&cfg)?;
        save(&cfg)?;
        eprintln!("First run — config created at {}", path.display());
        eprintln!("Platform: {}", platform::platform_summary());
        eprintln!();
        models::auto_download_if_needed(&cfg);
        return Ok(cfg);
    }
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let cfg: Config =
        toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(cfg)
}

pub fn load_or_default() -> Config {
    load().unwrap_or_default()
}

fn ensure_dirs(cfg: &Config) -> Result<()> {
    for dir in [&cfg.models_dir, &cfg.voices_dir, &cfg.output_dir] {
        fs::create_dir_all(dir).with_context(|| format!("failed to create directory {dir}"))?;
    }
    Ok(())
}

pub fn save(cfg: &Config) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let text = toml::to_string_pretty(cfg).context("failed to serialize config")?;
    fs::write(&path, text).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn init() -> Result<()> {
    let cfg = Config::default();
    ensure_dirs(&cfg)?;
    save(&cfg)?;
    println!("Config initialized at {}", config_path().display());
    println!("Platform: {}", platform::platform_summary());
    println!("Backend:  {}", cfg.backend);
    Ok(())
}

pub fn show() -> Result<()> {
    let cfg = load()?;
    let text = toml::to_string_pretty(&cfg).context("failed to serialize config")?;
    println!("{}", text);
    Ok(())
}

pub fn set(key: &str, value: &str) -> Result<()> {
    let mut cfg = load()?;

    match key {
        "python_path" => cfg.python_path = value.to_string(),
        "models_dir" => cfg.models_dir = value.to_string(),
        "voices_dir" => cfg.voices_dir = value.to_string(),
        "output_dir" => cfg.output_dir = value.to_string(),
        "backend" => cfg.backend = value.parse()?,
        "default_voice" => cfg.default_voice = value.to_string(),
        "default_speed" => {
            cfg.default_speed = value
                .parse()
                .with_context(|| format!("invalid speed: {value}"))?;
        }
        "auto_play" => {
            cfg.auto_play = value
                .parse()
                .with_context(|| format!("invalid bool: {value}"))?;
        }
        "model_variant" => {
            if value != "pro" && value != "lite" {
                anyhow::bail!("model_variant must be 'pro' or 'lite'");
            }
            cfg.model_variant = value.to_string();
        }
        _ => anyhow::bail!("unknown config key: {key}"),
    }

    save(&cfg)?;
    println!("Set {key} = {value}");
    Ok(())
}

/// Expand ~ to home directory in a path string.
pub fn expand_path(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        dirs::home_dir()
            .expect("could not determine home directory")
            .join(rest)
    } else if p == "~" {
        dirs::home_dir().expect("could not determine home directory")
    } else {
        PathBuf::from(p)
    }
}
