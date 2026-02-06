# voices

Manage saved voices for voice cloning. Saved voices let you reuse reference audio clips by name instead of specifying file paths each time.

## Subcommands

### voices list

List all saved voices.

```bash
qwen-tts voices list
```

Displays each saved voice name along with a preview of its transcript (if available). Voice files are stored as `.wav` files in the voices directory (`~/.qwen-tts/voices/` by default).

### voices add

Enroll a new voice from a reference audio file.

```
qwen-tts voices add <NAME> --ref <PATH> [--transcript <TEXT>]
```

| Argument / Option | Description |
|-------------------|-------------|
| `NAME` | **Required.** A name for the voice (used to reference it later). |
| `--ref <PATH>` | **Required.** Path to a reference audio file (`.wav`). The file is copied into the voices directory. |
| `--transcript <TEXT>` | Optional transcript of the reference audio. Stored alongside the audio as `<name>.txt`. Providing a transcript improves cloning quality. |

**Example:**

```bash
qwen-tts voices add alex --ref ~/recordings/alex_sample.wav --transcript "Hi, my name is Alex and this is how I normally speak."
```

After enrollment, you can use `--voice alex` with the `clone` command:

```bash
qwen-tts clone --voice alex --text "Any new text in Alex's voice."
```

### voices remove

Remove a saved voice.

```
qwen-tts voices remove <NAME>
```

| Argument | Description |
|----------|-------------|
| `NAME` | **Required.** The name of the voice to remove. |

**Example:**

```bash
qwen-tts voices remove alex
```

This deletes both the `.wav` file and the associated `.txt` transcript (if present) from the voices directory.
