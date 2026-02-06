# Introduction

**qwen-tts** is a cross-platform command-line interface for [Qwen3-TTS](https://huggingface.co/Qwen/Qwen3-TTS), Alibaba's state-of-the-art text-to-speech model. It provides a simple, ergonomic way to generate natural-sounding speech from your terminal.

## Key Features

- **Text-to-speech** -- Convert any text or file to spoken audio with a single command.
- **Voice design** -- Describe a voice in plain English (e.g., "a deep, calm British narrator") and the model will synthesize speech in that style.
- **Voice cloning** -- Provide a short reference audio clip and qwen-tts will reproduce that voice for new text.
- **Cross-platform** -- Runs on macOS (Apple Silicon and Intel), Linux, and Windows. Automatically selects the best backend for your hardware: MLX on Apple Silicon, CUDA on NVIDIA GPUs, or CPU fallback.
- **Saved voices** -- Enroll reference audio clips as named voices so you can reuse them without specifying file paths every time.
- **Auto-play** -- Generated audio plays immediately by default. Disable this with a single config toggle.

## How It Works

qwen-tts is a Rust CLI that orchestrates a Python-based TTS pipeline under the hood. It manages model downloads from Hugging Face, handles configuration, and delegates the actual inference to either the `mlx_audio` package (on Apple Silicon) or a PyTorch-based generation script (on CUDA and CPU platforms).

## Next Steps

- [Install qwen-tts](./installation.md)
- [Get started in under a minute](./quickstart.md)
- [Browse the full command reference](./commands/speak.md)
