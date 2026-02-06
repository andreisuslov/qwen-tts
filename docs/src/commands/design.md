# design

Design a voice from a free-form text description and generate speech with it.

Instead of choosing from a fixed set of voice names, you describe the voice you want in natural language. The model interprets the description and synthesizes speech that matches it.

## Usage

```
qwen-tts design <DESCRIPTION> [OPTIONS]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `DESCRIPTION` | **Required.** A text description of the desired voice (e.g., `"A deep calm British narrator"`, `"An energetic young woman"`). |

## Options

| Option | Description |
|--------|-------------|
| `--text <STRING>` | The text to speak with the designed voice. |
| `--file <PATH>` | Read the text to speak from a file. |
| `--speed <FLOAT>` | Speech speed multiplier (default: config value, typically `1.0`). |
| `-o, --output <PATH>` | Output file path. If omitted, a timestamped `.wav` file is written to the configured `output_dir`. |

> **Note:** You must provide either `--text` or `--file`. If neither is given, the command will return an error.

## Examples

Design a voice and speak a sentence:

```bash
qwen-tts design "A warm, friendly male voice with a slight Southern accent" --text "Howdy, partner."
```

Read text from a file:

```bash
qwen-tts design "A crisp, professional female newsreader" --file headlines.txt -o news.wav
```

## How It Works

The description string is passed directly as the instruction prompt to the TTS model. Qwen3-TTS uses this instruction to condition its output, producing speech that reflects the described characteristics. This does not use any reference audio -- the voice is synthesized entirely from the text description.
