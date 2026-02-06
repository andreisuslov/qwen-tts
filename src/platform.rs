use std::fmt;
use std::process::Command;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Mlx,
    Cuda,
    Cpu,
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Backend::Mlx => write!(f, "mlx"),
            Backend::Cuda => write!(f, "cuda"),
            Backend::Cpu => write!(f, "cpu"),
        }
    }
}

impl std::str::FromStr for Backend {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "mlx" => Ok(Backend::Mlx),
            "cuda" => Ok(Backend::Cuda),
            "cpu" => Ok(Backend::Cpu),
            _ => anyhow::bail!("unknown backend: {s} (expected mlx, cuda, or cpu)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOs,
    Windows,
    Linux,
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Os::MacOs => write!(f, "macOS"),
            Os::Windows => write!(f, "Windows"),
            Os::Linux => write!(f, "Linux"),
        }
    }
}

pub fn detect_os() -> Os {
    if cfg!(target_os = "macos") {
        Os::MacOs
    } else if cfg!(target_os = "windows") {
        Os::Windows
    } else {
        Os::Linux
    }
}

pub fn is_apple_silicon() -> bool {
    if cfg!(target_os = "macos") {
        cfg!(target_arch = "aarch64")
    } else {
        false
    }
}

pub fn has_nvidia_gpu() -> bool {
    Command::new("nvidia-smi")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn detect_backend() -> Backend {
    let os = detect_os();
    match os {
        Os::MacOs if is_apple_silicon() => Backend::Mlx,
        _ if has_nvidia_gpu() => Backend::Cuda,
        _ => Backend::Cpu,
    }
}

pub fn platform_summary() -> String {
    let os = detect_os();
    let backend = detect_backend();
    let arch = if is_apple_silicon() {
        "Apple Silicon"
    } else {
        std::env::consts::ARCH
    };
    format!("{os} ({arch}) — backend: {backend}")
}
