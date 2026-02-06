use std::fs;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config;

pub fn list() -> Result<()> {
    let cfg = config::load_or_default();
    let voices_dir = config::expand_path(&cfg.voices_dir);

    if !voices_dir.exists() {
        println!("No voices directory found.");
        println!("Use `qwen-tts voices add` to enroll a voice.");
        return Ok(());
    }

    let mut found = false;
    for entry in fs::read_dir(&voices_dir).context("failed to read voices directory")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("wav") {
            let name = path.file_stem().unwrap().to_string_lossy();
            let txt_path = path.with_extension("txt");
            let transcript = if txt_path.exists() {
                fs::read_to_string(&txt_path).unwrap_or_default()
            } else {
                "(no transcript)".to_string()
            };
            println!(
                "  {} — {}",
                name.green(),
                transcript.trim().chars().take(60).collect::<String>()
            );
            found = true;
        }
    }

    if !found {
        println!("No saved voices.");
        println!("Use `qwen-tts voices add <name> --ref <audio.wav>` to enroll one.");
    }

    Ok(())
}

pub fn add(name: &str, ref_audio: &str, transcript: Option<&str>) -> Result<()> {
    let cfg = config::load_or_default();
    let voices_dir = config::expand_path(&cfg.voices_dir);
    fs::create_dir_all(&voices_dir)?;

    let src = config::expand_path(ref_audio);
    if !src.exists() {
        anyhow::bail!("reference audio not found: {}", src.display());
    }

    let dest_wav = voices_dir.join(format!("{name}.wav"));
    fs::copy(&src, &dest_wav).with_context(|| {
        format!(
            "failed to copy {} → {}",
            src.display(),
            dest_wav.display()
        )
    })?;

    if let Some(t) = transcript {
        let dest_txt = voices_dir.join(format!("{name}.txt"));
        fs::write(&dest_txt, t)?;
    }

    println!("{} Voice '{}' enrolled.", "Done!".green().bold(), name);
    Ok(())
}

pub fn remove(name: &str) -> Result<()> {
    let cfg = config::load_or_default();
    let voices_dir = config::expand_path(&cfg.voices_dir);

    let wav = voices_dir.join(format!("{name}.wav"));
    let txt = voices_dir.join(format!("{name}.txt"));

    if !wav.exists() {
        anyhow::bail!("voice '{name}' not found");
    }

    fs::remove_file(&wav)?;
    if txt.exists() {
        fs::remove_file(&txt)?;
    }

    println!("{} Voice '{}' removed.", "Done!".green().bold(), name);
    Ok(())
}
