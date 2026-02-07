# Configuration

qwen-tts stores its configuration in a TOML file at:

```
~/.config/qwen-tts/config.toml
```

On Windows, this is typically:

```
C:\Users\<you>\AppData\Roaming\qwen-tts\config.toml
```

## Full Reference

Below is a complete example with default values:

```toml
python_path = "~/.qwen-tts/venv/bin/python"
models_dir = "~/.qwen-tts/models"
voices_dir = "~/.qwen-tts/voices"
output_dir = "~/.qwen-tts/outputs"
backend = "mlx"
default_voice = "Vivian"
default_speed = 1.0
auto_play = true
model_variant = "pro"
auto_cleanup = true
cleanup_age_hours = 24
```

## Key Descriptions

### python_path

Path to the Python interpreter used for TTS inference. This should point to the Python binary inside the virtual environment created during installation. On Windows, the default is `~/.qwen-tts/venv/Scripts/python.exe`.

### models_dir

Directory where model files are stored after downloading. Each variant (`pro`, `lite`) is stored in its own subdirectory.

### voices_dir

Directory where saved voice references are stored. Each voice consists of a `.wav` audio file and an optional `.txt` transcript file.

### output_dir

Default directory for generated audio output. When you run a generation command without specifying `--output`, the resulting `.wav` file is written here with a timestamp-based filename (e.g., `tts_1706140800.wav`).

### backend

The inference backend. Auto-detected by `config init`, but can be overridden manually. Valid values:

| Value | Description |
|-------|-------------|
| `mlx` | Apple MLX framework. Best performance on Apple Silicon Macs. Uses `mlx_audio` for inference. |
| `cuda` | NVIDIA CUDA. Requires an NVIDIA GPU with CUDA drivers. Uses PyTorch for inference. |
| `cpu` | CPU-only fallback. Works everywhere but is significantly slower. Uses PyTorch for inference. |

### default_voice

The voice name used by the `speak` command when `--voice` is not specified. This is a string identifier passed to the model's instruction prompt (e.g., `"Vivian"`, `"Ethan"`).

### default_speed

The speech speed multiplier used when `--speed` is not specified. A value of `1.0` produces normal speed. Lower values slow down speech; higher values speed it up.

### auto_play

When `true`, generated audio files are played immediately after creation. Playback uses platform-native tools:

- **macOS:** `afplay`
- **Windows:** PowerShell `SoundPlayer`
- **Linux:** `aplay`, `paplay`, or `ffplay` (tried in order)

Set to `false` to disable automatic playback.

### model_variant

The active model variant: `"pro"` for full precision or `"lite"` for the quantized version. This determines which subdirectory under `models_dir` is used for inference. Must be either `pro` or `lite`.

### auto_cleanup

When `true`, old output files in `output_dir` are automatically deleted at the start of each run. Only files older than `cleanup_age_hours` are removed. Set to `false` to keep all generated files indefinitely.

### cleanup_age_hours

The minimum age (in hours) an output file must reach before it is eligible for automatic cleanup. Only takes effect when `auto_cleanup` is `true`. For example, the default value of `24` means files older than 24 hours are deleted on the next run.

## Editing the Config File Directly

You can edit `~/.config/qwen-tts/config.toml` in any text editor. Changes take effect the next time you run a qwen-tts command. Alternatively, use `qwen-tts config set` to modify individual values from the command line.

## Directory Structure

After initialization, the `~/.qwen-tts/` directory contains:

```
~/.qwen-tts/
  venv/          # Python virtual environment
  models/        # Downloaded model files
    pro/         # Full-precision model
    lite/        # Quantized model
  voices/        # Saved voice references
  outputs/       # Generated audio files
```
