# speak

Generate speech from text using the Qwen3-TTS model.

## Usage

```
qwen-tts speak [TEXT] [OPTIONS]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `TEXT` | The text to speak. Optional if `--file` is provided. |

## Options

| Option | Description |
|--------|-------------|
| `--file <PATH>` | Read the input text from a file instead of the command line. |
| `--voice <NAME>` | Voice name for the speaker identity. Uses the `default_voice` config value if not specified (default: `Vivian`). |
| `--emotion <STYLE>` | Emotion or style instruction, such as `"Excited"`, `"Calm"`, or `"Whispered"`. When set, the model is prompted to speak with the given emotion. |
| `--speed <FLOAT>` | Speech speed multiplier. `1.0` is normal speed. Values below 1.0 slow down; above 1.0 speed up. Uses the `default_speed` config value if not specified. |
| `-o, --output <PATH>` | Output file path. If omitted, a timestamped `.wav` file is written to the configured `output_dir` (default: `~/.qwen-tts/outputs/`). |

## Examples

Basic text-to-speech:

```bash
qwen-tts speak "The quick brown fox jumps over the lazy dog."
```

With a specific voice and emotion:

```bash
qwen-tts speak "Breaking news from the capital." --voice "Ethan" --emotion "Serious"
```

Read from a file and save to a specific path:

```bash
qwen-tts speak --file chapter1.txt --output chapter1.wav
```

Slow down the speech:

```bash
qwen-tts speak "Take your time." --speed 0.8
```

## Behavior

1. Text is resolved from the positional argument or `--file` (positional takes priority).
2. A voice instruction is built from the `--voice` and optional `--emotion` flags.
3. The TTS backend generates a `.wav` file.
4. If `auto_play` is enabled in the config, the audio plays immediately after generation.
