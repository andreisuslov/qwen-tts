use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config;

pub fn list() -> Result<()> {
    let cfg = config::load_or_default();
    let models_dir = config::expand_path(&cfg.models_dir);

    if !models_dir.exists() {
        println!("No models directory found at {}", models_dir.display());
        println!("Run `qwen-tts models download` to download models.");
        return Ok(());
    }

    let mut found = false;
    for entry in fs::read_dir(&models_dir).context("failed to read models directory")? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name();
            let size = dir_size(&entry.path()).unwrap_or(0);
            println!(
                "  {} ({})",
                name.to_string_lossy().green(),
                human_size(size)
            );
            found = true;
        }
    }

    if !found {
        println!("No models installed.");
        println!("Run `qwen-tts models download` to download models.");
    }

    Ok(())
}

pub fn download(variant: &str) -> Result<()> {
    let cfg = config::load_or_default();
    let python = config::expand_path(&cfg.python_path);
    let models_dir = config::expand_path(&cfg.models_dir);

    if !python.exists() {
        anyhow::bail!(
            "Python not found at {}. Run the install script first.",
            python.display()
        );
    }

    let repo_id = match (cfg.backend, variant) {
        (crate::platform::Backend::Mlx, "pro") => {
            "mlx-community/Qwen3-TTS-bf16"
        }
        (crate::platform::Backend::Mlx, "lite") => {
            "mlx-community/Qwen3-TTS-4bit"
        }
        (_, "pro") => "Qwen/Qwen3-TTS",
        (_, "lite") => "Qwen/Qwen3-TTS",
        _ => anyhow::bail!("unknown variant: {variant} (expected 'pro' or 'lite')"),
    };

    println!(
        "{} {} ({} backend)...",
        "Downloading".cyan().bold(),
        repo_id,
        cfg.backend
    );

    let status = Command::new(python.to_string_lossy().as_ref())
        .args([
            "-c",
            &format!(
                "from huggingface_hub import snapshot_download; snapshot_download('{}', local_dir='{}')",
                repo_id,
                models_dir.join(variant).to_string_lossy()
            ),
        ])
        .status()
        .context("failed to run Python for model download")?;

    if !status.success() {
        anyhow::bail!("model download failed");
    }

    println!("{} Model '{}' downloaded.", "Done!".green().bold(), variant);
    Ok(())
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_file() {
            total += meta.len();
        } else if meta.is_dir() {
            total += dir_size(&entry.path())?;
        }
    }
    Ok(total)
}

fn human_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
