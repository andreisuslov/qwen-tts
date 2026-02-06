# Quick Start

This page walks you through generating your first speech output in three commands.

## 1. Initialize configuration

If you used one of the install scripts, this step is already done. Otherwise:

```bash
qwen-tts config init
```

This auto-detects your platform and backend, then writes a config file to `~/.config/qwen-tts/config.toml`.

## 2. Download a model

```bash
qwen-tts models download --variant pro
```

The `pro` variant downloads the full-precision model. Use `--variant lite` for a smaller quantized model (MLX only: 4-bit; recommended if disk space or memory is limited).

Model files are saved to `~/.qwen-tts/models/pro/` (or `lite/`).

## 3. Generate speech

```bash
qwen-tts speak "Hello, world!"
```

That's it. The audio is saved to `~/.qwen-tts/outputs/` and plays automatically.

## What's next?

Try a few more things:

```bash
# Use a specific emotion
qwen-tts speak "I can't believe we did it!" --emotion "Excited"

# Read text from a file
qwen-tts speak --file article.txt --output narration.wav

# Design a voice from a description
qwen-tts design "A warm, friendly female narrator" --text "Welcome to the show."

# Clone a voice from a reference clip
qwen-tts clone --ref speaker.wav --ref-text "Hello, my name is Alex." --text "Now I can say anything."
```

For complete details on every command and flag, see the [Commands](./commands/speak.md) section.
