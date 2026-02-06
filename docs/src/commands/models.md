# models

Manage TTS model downloads and installations.

## Subcommands

### models list

List all installed models.

```bash
qwen-tts models list
```

Shows each installed model variant along with its size on disk. Models are stored in the models directory (`~/.qwen-tts/models/` by default).

### models download

Download a model from Hugging Face.

```
qwen-tts models download [--variant <VARIANT>]
```

| Option | Description |
|--------|-------------|
| `--variant <VARIANT>` | Model variant to download: `pro` or `lite`. Defaults to `pro`. |

**Example:**

```bash
# Download the full-precision model
qwen-tts models download --variant pro

# Download the smaller quantized model
qwen-tts models download --variant lite
```

## Model Variants

| Variant | Backend | Hugging Face Repository | Notes |
|---------|---------|------------------------|-------|
| `pro` | MLX | `mlx-community/Qwen3-TTS-bf16` | Full bf16 precision. Best quality on Apple Silicon. |
| `lite` | MLX | `mlx-community/Qwen3-TTS-4bit` | 4-bit quantized. Lower memory usage, slightly reduced quality. |
| `pro` | CUDA / CPU | `Qwen/Qwen3-TTS` | Official PyTorch checkpoint. |
| `lite` | CUDA / CPU | `Qwen/Qwen3-TTS` | Same checkpoint (quantization handled at runtime). |

The download command uses the `huggingface_hub` Python library to fetch model files. The appropriate repository is selected automatically based on your configured backend.

## Storage

Downloaded models are saved to `~/.qwen-tts/models/<variant>/`. You can change the models directory with:

```bash
qwen-tts config set models_dir /path/to/models
```
