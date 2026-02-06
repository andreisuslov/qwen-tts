# config

View and modify qwen-tts configuration.

Configuration is stored in `~/.config/qwen-tts/config.toml`.

## Subcommands

### config init

Initialize configuration with auto-detected platform settings.

```bash
qwen-tts config init
```

This command:

1. Detects your operating system and hardware (Apple Silicon, NVIDIA GPU, or CPU-only).
2. Selects the appropriate backend (`mlx`, `cuda`, or `cpu`).
3. Creates the directory structure at `~/.qwen-tts/` (models, voices, outputs).
4. Writes default values to `~/.config/qwen-tts/config.toml`.

Run this once after installation, or again to reset to defaults.

### config show

Display the current configuration.

```bash
qwen-tts config show
```

Prints the full contents of `config.toml` in TOML format.

### config set

Set a single configuration value.

```
qwen-tts config set <KEY> <VALUE>
```

| Argument | Description |
|----------|-------------|
| `KEY` | The configuration key to set. |
| `VALUE` | The new value. |

**Examples:**

```bash
qwen-tts config set default_voice "Ethan"
qwen-tts config set default_speed 1.2
qwen-tts config set auto_play false
qwen-tts config set backend cuda
qwen-tts config set model_variant lite
```

## Configuration Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `python_path` | string | `~/.qwen-tts/venv/bin/python` | Path to the Python interpreter in the virtual environment. |
| `models_dir` | string | `~/.qwen-tts/models` | Directory where downloaded models are stored. |
| `voices_dir` | string | `~/.qwen-tts/voices` | Directory where saved voice references are stored. |
| `output_dir` | string | `~/.qwen-tts/outputs` | Default directory for generated audio files. |
| `backend` | string | auto-detected | Inference backend: `mlx`, `cuda`, or `cpu`. |
| `default_voice` | string | `Vivian` | Default voice name for the `speak` command. |
| `default_speed` | float | `1.0` | Default speech speed multiplier. |
| `auto_play` | bool | `true` | Automatically play audio after generation. |
| `model_variant` | string | `pro` | Active model variant: `pro` or `lite`. |

For a detailed description of each key, see [Configuration](../configuration.md).
