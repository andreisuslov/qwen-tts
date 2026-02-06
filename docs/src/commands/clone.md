# clone

Clone a voice from reference audio and use it to speak new text.

Provide a short audio sample of the target voice (and optionally its transcript), and qwen-tts will generate new speech that sounds like the same speaker.

## Usage

```
qwen-tts clone [OPTIONS]
```

## Options

| Option | Description |
|--------|-------------|
| `--ref <PATH>` | Path to a reference audio file (`.wav`). Required unless `--voice` is used. |
| `--ref-text <STRING>` | Transcript of the reference audio. Providing this improves cloning accuracy. |
| `--voice <NAME>` | Use a previously saved voice by name (see [voices](./voices.md)). Mutually exclusive with `--ref`. |
| `--text <STRING>` | The text to speak with the cloned voice. |
| `--file <PATH>` | Read the text to speak from a file. |
| `--speed <FLOAT>` | Speech speed multiplier (default: config value, typically `1.0`). |
| `-o, --output <PATH>` | Output file path. If omitted, a timestamped `.wav` file is written to the configured `output_dir`. |

> **Note:** You must provide either `--ref` or `--voice` to specify the reference voice. You must also provide either `--text` or `--file` for the content to speak.

## Examples

Clone from a reference audio file:

```bash
qwen-tts clone --ref speaker.wav --ref-text "Hello, my name is Alex." --text "Now I can say anything in Alex's voice."
```

Clone using a saved voice:

```bash
qwen-tts clone --voice alex --text "This uses the saved reference for Alex."
```

Clone and save the output:

```bash
qwen-tts clone --ref narrator.wav --file script.txt -o narration.wav
```

## Voice Resolution

When `--voice` is provided, qwen-tts looks up the corresponding `.wav` file in the voices directory (`~/.qwen-tts/voices/<name>.wav`). If a `.txt` transcript file exists alongside it, that transcript is used automatically. You can still override the transcript with `--ref-text`.

When `--ref` is provided, the audio file is used directly without copying it to the voices directory. To save it for future reuse, see the [voices add](./voices.md) command.

For a deeper guide on voice cloning, see [Voice Cloning](../voice-cloning.md).
