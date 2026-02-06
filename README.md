# qwen-tts

A cross-platform CLI for [Qwen3-TTS](https://huggingface.co/Qwen/Qwen3-TTS) text-to-speech with voice design and voice cloning.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-mdBook-orange)](https://andreisuslov.github.io/qwen-tts/)
[![CI](https://github.com/andreisuslov/qwen-tts/actions/workflows/pages.yml/badge.svg)](https://github.com/andreisuslov/qwen-tts/actions/workflows/pages.yml)

## Features

- **Text-to-speech** -- generate natural-sounding speech from text or files
- **Voice design** -- create voices from text descriptions (e.g. "A deep calm British narrator")
- **Voice cloning** -- clone a voice from a short reference audio clip
- **Saved voices** -- enroll, list, and reuse voices by name
- **Cross-platform** -- runs on macOS (MLX), Linux (CUDA/CPU), and Windows (CUDA/CPU)
- **Auto backend detection** -- automatically selects MLX on Apple Silicon, CUDA when an NVIDIA GPU is present, or CPU fallback
- **Emotion and speed control** -- adjust speech style and tempo per generation

## Quick Install

```bash
# From source (requires Rust toolchain)
cargo install --git https://github.com/andreisuslov/qwen-tts
```

## Quick Usage

```bash
# Speak text directly
qwen-tts speak "Hello, world!"

# Speak from a file
qwen-tts speak --file article.txt --output speech.wav

# Design a voice from a description
qwen-tts design "A warm, friendly female narrator" --text "Welcome to the show."

# Clone a voice from reference audio
qwen-tts clone --ref sample.wav --ref-text "This is my voice." --text "Now I sound like that."

# Use a saved voice
qwen-tts voices add narrator --ref narrator_sample.wav --transcript "Sample transcript."
qwen-tts speak "Good evening." --voice narrator

# Manage models
qwen-tts models list
qwen-tts models download --variant pro
```

## Platform Support

| Platform               | Backend | Status      |
|------------------------|---------|-------------|
| macOS (Apple Silicon)  | MLX     | Supported   |
| Linux (NVIDIA GPU)     | CUDA    | Supported   |
| Linux (CPU only)       | CPU     | Supported   |
| Windows (NVIDIA GPU)   | CUDA    | Supported   |
| Windows (CPU only)     | CPU     | Supported   |

Backend is detected automatically at runtime. You can override it in the configuration:

```bash
qwen-tts config set backend cuda
```

## Documentation

Full documentation is available at **[andreisuslov.github.io/qwen-tts](https://andreisuslov.github.io/qwen-tts/)**.

## License

This project is licensed under the [MIT License](LICENSE).
