use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::{self, Config};
use crate::output;
use crate::platform::Backend;

fn prompt_yn(question: &str, default_yes: bool) -> bool {
    let hint = if default_yes { "[Y/n]" } else { "[y/N]" };
    eprint!("{question} {hint} ");
    io::stderr().flush().ok();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return default_yes;
    }
    let input = input.trim().to_lowercase();
    if input.is_empty() {
        return default_yes;
    }
    input.starts_with('y')
}

pub fn repo_id(backend: Backend, variant: &str) -> Result<&'static str> {
    match (backend, variant) {
        // Base: standard TTS + voice cloning (0.6B)
        (Backend::Mlx, "base") => Ok("mlx-community/Qwen3-TTS-12Hz-0.6B-Base-bf16"),
        (Backend::Mlx, "base-4bit") => Ok("mlx-community/Qwen3-TTS-12Hz-0.6B-Base-4bit"),
        // CustomVoice: voice cloning focused (0.6B)
        (Backend::Mlx, "custom") => Ok("mlx-community/Qwen3-TTS-12Hz-0.6B-CustomVoice-bf16"),
        (Backend::Mlx, "custom-4bit") => Ok("mlx-community/Qwen3-TTS-12Hz-0.6B-CustomVoice-4bit"),
        // VoiceDesign: create voices from descriptions (1.7B)
        (Backend::Mlx, "design") => Ok("mlx-community/Qwen3-TTS-12Hz-1.7B-VoiceDesign-bf16"),
        (Backend::Mlx, "design-4bit") => Ok("mlx-community/Qwen3-TTS-12Hz-1.7B-VoiceDesign-4bit"),
        // PyTorch (CUDA/CPU)
        (_, "base") => Ok("Qwen/Qwen3-TTS-12Hz-0.6B-Base"),
        (_, "base-4bit") => Ok("Qwen/Qwen3-TTS-12Hz-0.6B-Base"),
        (_, "custom") => Ok("Qwen/Qwen3-TTS-12Hz-0.6B-Base"),
        (_, "custom-4bit") => Ok("Qwen/Qwen3-TTS-12Hz-0.6B-Base"),
        (_, "design") => Ok("Qwen/Qwen3-TTS-12Hz-0.6B-Base"),
        (_, "design-4bit") => Ok("Qwen/Qwen3-TTS-12Hz-0.6B-Base"),
        // Legacy aliases
        (Backend::Mlx, "pro") => Ok("mlx-community/Qwen3-TTS-12Hz-0.6B-Base-bf16"),
        (Backend::Mlx, "lite") => Ok("mlx-community/Qwen3-TTS-12Hz-0.6B-Base-4bit"),
        (_, "pro") => Ok("Qwen/Qwen3-TTS-12Hz-0.6B-Base"),
        (_, "lite") => Ok("Qwen/Qwen3-TTS-12Hz-0.6B-Base"),
        _ => anyhow::bail!(
            "unknown variant: {variant}\nAvailable: base, base-4bit, custom, custom-4bit, design, design-4bit"
        ),
    }
}

fn model_dir(cfg: &Config, variant: &str) -> PathBuf {
    config::expand_path(&cfg.models_dir).join(variant)
}

fn is_model_installed(cfg: &Config, variant: &str) -> bool {
    let dir = model_dir(cfg, variant);
    dir.exists()
        && fs::read_dir(&dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
}

/// Try downloading with Python huggingface_hub, fall back to git clone.
fn download_repo(cfg: &Config, repo: &str, dest: &PathBuf) -> Result<()> {
    let python = config::expand_path(&cfg.python_path);

    // Try Python huggingface_hub first
    if python.exists() {
        output::status("Downloading", &format!("{repo} via huggingface_hub..."));
        let status = Command::new(python.to_string_lossy().as_ref())
            .args([
                "-c",
                &format!(
                    "from huggingface_hub import snapshot_download; snapshot_download('{}', local_dir='{}')",
                    repo,
                    dest.to_string_lossy()
                ),
            ])
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status();

        if let Ok(s) = status {
            if s.success() {
                return Ok(());
            }
        }
        output::warn("huggingface_hub download failed, trying git clone...");
    }

    // Fallback: git clone from HuggingFace
    let url = format!("https://huggingface.co/{repo}");
    output::status("Downloading", &format!("{repo} via git clone..."));

    // Check if git-lfs is available
    let has_lfs = Command::new("git")
        .args(["lfs", "version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_lfs {
        output::warn("git-lfs not found — large model files may not download correctly");
        eprintln!("Install git-lfs: https://git-lfs.github.com");
    }

    if dest.exists() {
        fs::remove_dir_all(dest).ok();
    }

    let status = Command::new("git")
        .args(["clone", "--depth", "1", &url])
        .arg(dest.to_string_lossy().as_ref())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("failed to run git clone")?;

    if !status.success() {
        anyhow::bail!("git clone failed for {repo}");
    }

    Ok(())
}

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
    let repo = repo_id(cfg.backend, variant)?;
    let dest = model_dir(&cfg, variant);

    eprintln!(
        "{} {} ({} backend)...",
        "Downloading".cyan().bold(),
        repo,
        cfg.backend
    );

    fs::create_dir_all(dest.parent().unwrap())?;
    download_repo(&cfg, repo, &dest)?;

    output::success(&format!("Model '{variant}' ready at {}", dest.display()));
    Ok(())
}

pub fn update(variant: Option<&str>) -> Result<()> {
    let cfg = config::load_or_default();
    let variant = variant.unwrap_or(&cfg.model_variant);
    let repo = repo_id(cfg.backend, variant)?;
    let dest = model_dir(&cfg, variant);

    if dest.exists() {
        eprintln!("{} {} to latest version...", "Updating".cyan().bold(), repo);
        // If it's a git repo, try git pull first
        let is_git = dest.join(".git").exists();
        if is_git {
            let status = Command::new("git")
                .args(["-C", &dest.to_string_lossy(), "pull", "--ff-only"])
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status();

            if let Ok(s) = status {
                if s.success() {
                    output::success(&format!("Model '{variant}' updated."));
                    return Ok(());
                }
            }
            output::warn("git pull failed, re-downloading...");
        }
    } else {
        eprintln!(
            "{} {} (not installed yet)...",
            "Downloading".cyan().bold(),
            repo
        );
    }

    // Full re-download
    fs::create_dir_all(dest.parent().unwrap())?;
    if dest.exists() {
        fs::remove_dir_all(&dest).ok();
    }
    download_repo(&cfg, repo, &dest)?;
    output::success(&format!("Model '{variant}' updated to latest."));
    Ok(())
}

/// Called during first-run auto-init. Prompts the user to download the default model.
pub fn auto_download_if_needed(cfg: &Config) {
    let variant = &cfg.model_variant;
    if is_model_installed(cfg, variant) {
        return;
    }

    let repo = match repo_id(cfg.backend, variant) {
        Ok(r) => r,
        Err(_) => return,
    };

    eprintln!(
        "No TTS model installed. The '{}' model ({}) is required to generate speech.",
        variant, repo
    );

    if !prompt_yn("Download it now?", true) {
        eprintln!("Skipped. Run `qwen-tts models download` later to install.");
        eprintln!();
        return;
    }

    eprintln!();
    let dest = model_dir(cfg, variant);
    if let Err(e) = fs::create_dir_all(dest.parent().unwrap()) {
        output::warn(&format!("Could not create models directory: {e}"));
        return;
    }

    match download_repo(cfg, repo, &dest) {
        Ok(()) => {
            output::success(&format!("Model '{variant}' ready."));
            eprintln!();
        }
        Err(e) => {
            output::warn(&format!("Auto-download failed: {e}"));
            eprintln!("Run `qwen-tts models download` to try again.\n");
        }
    }
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
