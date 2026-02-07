use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{self, Config};
use crate::editor;
use crate::models;
use crate::output;
use crate::platform::Backend;
use anyhow::{Context, Result};

pub struct SpeakArgs {
    pub text: Option<String>,
    pub file: Option<String>,
    pub voice: Option<String>,
    pub emotion: Option<String>,
    pub speed: Option<f32>,
    pub output: Option<String>,
}

pub struct DesignArgs {
    pub description: String,
    pub text: Option<String>,
    pub file: Option<String>,
    pub speed: Option<f32>,
    pub output: Option<String>,
}

pub struct CloneArgs {
    pub ref_audio: Option<String>,
    pub ref_text: Option<String>,
    pub voice: Option<String>,
    pub text: Option<String>,
    pub file: Option<String>,
    pub speed: Option<f32>,
    pub output: Option<String>,
}

fn resolve_text(text: Option<&str>, file: Option<&str>) -> Result<String> {
    match (text, file) {
        (Some(t), _) => Ok(t.to_string()),
        (None, Some(f)) => {
            let path = config::expand_path(f);
            fs::read_to_string(&path)
                .with_context(|| format!("failed to read text file: {}", path.display()))
        }
        (None, None) => {
            // Open TUI editor for multi-line input
            match editor::open("Enter text (multi-line)")? {
                Some(t) if !t.is_empty() => Ok(t),
                _ => anyhow::bail!("no text provided (editor cancelled)"),
            }
        }
    }
}

fn resolve_output(output: Option<&str>, cfg: &Config) -> PathBuf {
    match output {
        Some(p) => config::expand_path(p),
        None => {
            let dir = config::expand_path(&cfg.output_dir);
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            dir.join(format!("tts_{ts}"))
        }
    }
}

/// Find the actual audio file produced by mlx_audio.
/// With --join_audio it creates a single audio.wav inside the output directory.
fn find_output_file(output_dir: &Path) -> Option<PathBuf> {
    if output_dir.is_dir() {
        // --join_audio produces audio.wav in the directory
        let joined = output_dir.join("audio.wav");
        if joined.exists() {
            return Some(joined);
        }
        // Fallback: first wav file found
        let mut wavs: Vec<_> = fs::read_dir(output_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("wav"))
            .collect();
        wavs.sort();
        wavs.into_iter().next()
    } else if output_dir.exists() {
        Some(output_dir.to_path_buf())
    } else {
        None
    }
}

/// Returns the model path or repo ID. Prefers local, downloads if missing.
fn model_id(cfg: &Config) -> Result<String> {
    let local = config::expand_path(&cfg.models_dir).join(&cfg.model_variant);
    if local.exists() {
        return Ok(local.to_string_lossy().to_string());
    }
    // Model not installed — download it now
    output::status("Model", "not found locally, downloading...");
    models::download(&cfg.model_variant)?;
    Ok(local.to_string_lossy().to_string())
}

pub fn speak(args: SpeakArgs) -> Result<()> {
    let cfg = config::load()?;
    let text = resolve_text(args.text.as_deref(), args.file.as_deref())?;
    let out = resolve_output(args.output.as_deref(), &cfg);
    let voice = args.voice.as_deref().unwrap_or(&cfg.default_voice);
    let speed = args.speed.unwrap_or(cfg.default_speed);

    // Build instruct text for voice personality
    let instruct = match &args.emotion {
        Some(emo) => format!("Speak as {voice} with {emo} emotion."),
        None => format!("Speak as {voice}."),
    };

    output::status("Generating", &format!("speech with {voice} voice..."));

    let status = run_tts_command(
        &cfg,
        &TtsParams {
            text: &text,
            instruct: &instruct,
            speed,
            output_path: &out,
            ref_audio: None,
            ref_text: None,
            voice: Some(voice),
        },
    )?;

    if !status.success() {
        anyhow::bail!("TTS generation failed");
    }

    let actual = find_output_file(&out).unwrap_or(out);
    output::success(&format!("Saved to {}", actual.display()));

    if cfg.auto_play {
        play_audio(&actual)?;
    }

    Ok(())
}

pub fn design(args: DesignArgs) -> Result<()> {
    let cfg = config::load()?;
    let text = resolve_text(args.text.as_deref(), args.file.as_deref())?;
    let out = resolve_output(args.output.as_deref(), &cfg);
    let speed = args.speed.unwrap_or(cfg.default_speed);

    let instruct = args.description;

    output::status("Designing", "voice from description...");

    let status = run_tts_command(
        &cfg,
        &TtsParams {
            text: &text,
            instruct: &instruct,
            speed,
            output_path: &out,
            ref_audio: None,
            ref_text: None,
            voice: None,
        },
    )?;

    if !status.success() {
        anyhow::bail!("TTS generation failed");
    }

    let actual = find_output_file(&out).unwrap_or(out);
    output::success(&format!("Saved to {}", actual.display()));

    if cfg.auto_play {
        play_audio(&actual)?;
    }

    Ok(())
}

pub fn clone(args: CloneArgs) -> Result<()> {
    let cfg = config::load()?;
    let text = resolve_text(args.text.as_deref(), args.file.as_deref())?;
    let out = resolve_output(args.output.as_deref(), &cfg);
    let speed = args.speed.unwrap_or(cfg.default_speed);

    // Resolve reference audio — either from --ref or --voice (saved voice)
    let (ref_audio, ref_text) = if let Some(voice_name) = &args.voice {
        let voices_dir = config::expand_path(&cfg.voices_dir);
        let wav = voices_dir.join(format!("{voice_name}.wav"));
        let txt = voices_dir.join(format!("{voice_name}.txt"));
        if !wav.exists() {
            anyhow::bail!(
                "voice '{voice_name}' not found (no {}.wav in voices dir)",
                voice_name
            );
        }
        let transcript = if txt.exists() {
            Some(fs::read_to_string(&txt).context("failed to read voice transcript")?)
        } else {
            args.ref_text.clone()
        };
        (wav.to_string_lossy().to_string(), transcript)
    } else if let Some(ref_path) = &args.ref_audio {
        (ref_path.clone(), args.ref_text.clone())
    } else {
        anyhow::bail!("provide either --ref <audio_file> or --voice <saved_voice>");
    };

    output::status("Cloning", "voice from reference audio...");

    let status = run_tts_command(
        &cfg,
        &TtsParams {
            text: &text,
            instruct: "Clone the voice from the reference audio.",
            speed,
            output_path: &out,
            ref_audio: Some(&ref_audio),
            ref_text: ref_text.as_deref(),
            voice: None,
        },
    )?;

    if !status.success() {
        anyhow::bail!("TTS generation failed");
    }

    let actual = find_output_file(&out).unwrap_or(out);
    output::success(&format!("Saved to {}", actual.display()));

    if cfg.auto_play {
        play_audio(&actual)?;
    }

    Ok(())
}

struct TtsParams<'a> {
    text: &'a str,
    instruct: &'a str,
    speed: f32,
    output_path: &'a Path,
    ref_audio: Option<&'a str>,
    ref_text: Option<&'a str>,
    voice: Option<&'a str>,
}

fn run_tts_command(cfg: &Config, params: &TtsParams) -> Result<std::process::ExitStatus> {
    let python = config::expand_path(&cfg.python_path);
    let model = model_id(cfg)?;

    // Ensure output directory exists
    if let Some(parent) = params.output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut cmd = Command::new(python.to_string_lossy().as_ref());

    match cfg.backend {
        Backend::Mlx => {
            cmd.args(["-m", "mlx_audio.tts.generate"]);
        }
        Backend::Cuda | Backend::Cpu => {
            let script = config::base_dir().join("generate_compat.py");
            cmd.arg(script.to_string_lossy().as_ref());
        }
    }

    cmd.args(["--model", &model]);
    cmd.args(["--text", params.text]);
    cmd.args(["--instruct", params.instruct]);
    cmd.args(["--speed", &params.speed.to_string()]);
    cmd.args(["--output_path", &params.output_path.to_string_lossy()]);

    // Use --voice to enforce consistent voice across all chunks
    if let Some(voice) = params.voice {
        cmd.args(["--voice", voice]);
    }

    // Join all audio chunks into a single file instead of a directory of fragments
    cmd.arg("--join_audio");

    if let Some(ref_audio) = params.ref_audio {
        cmd.args(["--ref_audio", ref_audio]);
    }
    if let Some(ref_text) = params.ref_text {
        cmd.args(["--ref_text", ref_text]);
    }

    cmd.stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("failed to run TTS command")
}

fn play_audio(path: &Path) -> Result<()> {
    // mlx_audio may create a directory of chunks instead of a single file
    let files = if path.is_dir() {
        let mut wavs: Vec<_> = fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("wav"))
            .collect();
        wavs.sort();
        wavs
    } else {
        vec![path.to_path_buf()]
    };

    for file in &files {
        output::status("Playing", &file.to_string_lossy());
        let status = play_single(file);
        match status {
            Ok(s) if s.success() => {}
            Ok(_) => output::warn("Audio playback finished with non-zero exit code"),
            Err(e) => output::warn(&format!("Could not play audio: {e}")),
        }
    }
    Ok(())
}

fn play_single(path: &Path) -> std::result::Result<std::process::ExitStatus, std::io::Error> {
    if cfg!(target_os = "macos") {
        Command::new("afplay")
            .arg(path.to_string_lossy().as_ref())
            .status()
    } else if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args([
                "-c",
                &format!(
                    "(New-Object Media.SoundPlayer '{}').PlaySync()",
                    path.display()
                ),
            ])
            .status()
    } else {
        Command::new("aplay")
            .arg(path.to_string_lossy().as_ref())
            .status()
            .or_else(|_| {
                Command::new("paplay")
                    .arg(path.to_string_lossy().as_ref())
                    .status()
            })
            .or_else(|_| {
                Command::new("ffplay")
                    .args(["-nodisp", "-autoexit"])
                    .arg(path.to_string_lossy().as_ref())
                    .status()
            })
    }
}
