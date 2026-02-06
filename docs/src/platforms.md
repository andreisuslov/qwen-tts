# Platform Support

qwen-tts runs on macOS, Linux, and Windows. The CLI automatically detects your hardware and selects the best inference backend during `config init`.

## Support Matrix

| Platform | Backend | Performance | Notes |
|----------|---------|-------------|-------|
| macOS Apple Silicon (M1/M2/M3/M4) | `mlx` | Best | Native MLX acceleration. Recommended platform. Uses `mlx_audio` for inference with optimized MLX model weights. |
| macOS Intel | `cpu` | Slow | No GPU acceleration available. Falls back to PyTorch CPU inference. |
| Linux + NVIDIA GPU | `cuda` | Fast | Requires NVIDIA drivers and CUDA toolkit. Uses PyTorch with CUDA for inference. |
| Linux CPU-only | `cpu` | Slow | PyTorch CPU inference. Functional but not recommended for regular use. |
| Windows + NVIDIA GPU | `cuda` | Fast | Requires NVIDIA drivers and CUDA toolkit. Uses PyTorch with CUDA for inference. |
| Windows CPU-only | `cpu` | Slow | PyTorch CPU inference. Functional but not recommended for regular use. |

## Backend Detection

When you run `qwen-tts config init`, the following logic determines your backend:

1. If the OS is macOS and the architecture is `aarch64` (Apple Silicon) -> `mlx`
2. Otherwise, if `nvidia-smi` is found and returns success -> `cuda`
3. Otherwise -> `cpu`

You can override the auto-detected backend manually:

```bash
qwen-tts config set backend cuda
```

## Python Dependencies by Backend

Each backend requires different Python packages in the virtual environment:

### MLX (Apple Silicon)

```bash
pip install mlx-audio huggingface-hub
```

### CUDA (NVIDIA GPU)

```bash
pip install torch transformers huggingface-hub
```

### CPU

```bash
pip install torch transformers huggingface-hub --extra-index-url https://download.pytorch.org/whl/cpu
```

## Audio Playback

Generated audio is played automatically when `auto_play` is enabled. The playback command depends on the platform:

| Platform | Command |
|----------|---------|
| macOS | `afplay` (built-in) |
| Windows | PowerShell `SoundPlayer` |
| Linux | `aplay`, `paplay`, or `ffplay` (tried in order) |

If no audio player is found, a warning is printed and the generated file is still saved to disk.

## Model Variants by Backend

| Backend | Pro Variant | Lite Variant |
|---------|-------------|--------------|
| MLX | `mlx-community/Qwen3-TTS-bf16` | `mlx-community/Qwen3-TTS-4bit` |
| CUDA | `Qwen/Qwen3-TTS` | `Qwen/Qwen3-TTS` |
| CPU | `Qwen/Qwen3-TTS` | `Qwen/Qwen3-TTS` |

On non-MLX backends, both `pro` and `lite` use the same upstream PyTorch checkpoint from Qwen.
