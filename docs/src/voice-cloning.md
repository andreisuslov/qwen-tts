# Voice Cloning

Voice cloning lets you reproduce a specific person's voice from a short audio sample. This page explains how it works, how to prepare good reference audio, and how to save voices for repeated use.

## How It Works

Qwen3-TTS supports zero-shot voice cloning. You provide:

1. **Reference audio** -- A short `.wav` clip of the target speaker.
2. **Reference transcript** -- The text spoken in the reference audio (optional but recommended).
3. **Target text** -- The new text you want spoken in the cloned voice.

The model analyzes the speaker characteristics in the reference audio (pitch, timbre, cadence) and applies them when generating the target text. No fine-tuning is required.

## Preparing Reference Audio

For best results, follow these guidelines:

- **Length:** 5 to 15 seconds is ideal. Shorter clips may not capture enough speaker characteristics. Longer clips increase processing time without proportional quality gains.
- **Format:** WAV format is required. Convert other formats with `ffmpeg`:
  ```bash
  ffmpeg -i recording.mp3 -ar 16000 -ac 1 recording.wav
  ```
- **Quality:** Use clean audio with minimal background noise. Avoid clips with music, multiple speakers, or heavy compression artifacts.
- **Content:** The reference audio should contain natural, conversational speech. Avoid whispering, shouting, or singing unless you want those characteristics reproduced.
- **Transcript accuracy:** If you provide a transcript, make sure it matches the audio exactly. Mismatched transcripts degrade cloning quality.

## Basic Cloning

Clone from a one-off reference file:

```bash
qwen-tts clone \
  --ref ~/recordings/speaker.wav \
  --ref-text "This is how I normally speak." \
  --text "The cloned voice will say this sentence."
```

## Saving Voices for Reuse

If you plan to use the same voice repeatedly, save it with `voices add`:

```bash
qwen-tts voices add sarah \
  --ref ~/recordings/sarah_sample.wav \
  --transcript "Hi, I'm Sarah and this is a sample of my voice."
```

This copies the audio and transcript into the voices directory. Now you can reference it by name:

```bash
qwen-tts clone --voice sarah --text "Any new text in Sarah's voice."
```

To see all saved voices:

```bash
qwen-tts voices list
```

To remove a saved voice:

```bash
qwen-tts voices remove sarah
```

## Tips

- **Provide transcripts.** The model uses the transcript to align audio features with linguistic content. Cloning quality improves noticeably when transcripts are provided.
- **Test with short text first.** Before generating a long narration, test the cloned voice with a short sentence to verify quality.
- **Multiple references.** The current implementation supports a single reference clip per invocation. If you have multiple samples of the same speaker, choose the cleanest one.
- **Combining with speed control.** You can adjust the speed of cloned speech with `--speed` without affecting voice quality:
  ```bash
  qwen-tts clone --voice sarah --text "Slower speech." --speed 0.8
  ```
