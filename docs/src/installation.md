# Installation

## Prerequisites

Before installing qwen-tts, make sure you have the following:

- **Python 3.10+** -- Required for the TTS inference backend.
- **Rust / Cargo** -- Required to compile the CLI. Install from [rustup.rs](https://rustup.rs/).
- **git** -- Required for model downloads and install scripts.

## Option 1: One-liner (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/andreisuslov/qwen-tts/main/scripts/install.sh | bash
```

This script will:

1. Install the `qwen-tts` binary via `cargo install`.
2. Create a Python virtual environment at `~/.qwen-tts/venv`.
3. Install the required Python dependencies into the venv.
4. Run `qwen-tts config init` to generate a default configuration.

## Option 2: One-liner (Windows)

Open PowerShell and run:

```powershell
irm https://raw.githubusercontent.com/andreisuslov/qwen-tts/main/scripts/install.ps1 | iex
```

This performs the same steps as the macOS/Linux script, adapted for Windows paths and tooling.

## Option 3: Manual Installation

If you prefer to install manually or need more control over the process:

### 1. Install the CLI

```bash
cargo install --git https://github.com/andreisuslov/qwen-tts
```

### 2. Create the Python virtual environment

```bash
python3 -m venv ~/.qwen-tts/venv
```

### 3. Install Python dependencies

On **macOS Apple Silicon** (MLX backend):

```bash
~/.qwen-tts/venv/bin/pip install mlx-audio huggingface-hub
```

On **Linux / Windows with NVIDIA GPU** (CUDA backend):

```bash
~/.qwen-tts/venv/bin/pip install torch transformers huggingface-hub
```

On **CPU-only** systems:

```bash
~/.qwen-tts/venv/bin/pip install torch transformers huggingface-hub --extra-index-url https://download.pytorch.org/whl/cpu
```

### 4. Initialize configuration

```bash
qwen-tts config init
```

This creates `~/.config/qwen-tts/config.toml` with auto-detected platform settings, and sets up the directory structure at `~/.qwen-tts/`.

## Verifying the Installation

```bash
qwen-tts --version
qwen-tts config show
```

The `config show` command will print your current configuration, including the detected backend. If everything is correct, proceed to the [Quick Start](./quickstart.md).
