# Examples

A collection of practical examples for common qwen-tts workflows.

## Basic Speech

Generate speech from a string:

```bash
qwen-tts speak "Hello, world!"
```

Save to a specific file:

```bash
qwen-tts speak "Hello, world!" -o hello.wav
```

## Reading Files

Narrate a text file:

```bash
qwen-tts speak --file article.txt
```

Narrate a file with a specific voice and save the result:

```bash
qwen-tts speak --file chapter1.txt --voice "Ethan" -o chapter1.wav
```

## Voice and Emotion

Speak with emotion:

```bash
qwen-tts speak "We won the championship!" --emotion "Excited"
qwen-tts speak "I'm sorry for your loss." --emotion "Sad and gentle"
```

Change the default voice:

```bash
qwen-tts config set default_voice "Ethan"
qwen-tts speak "This now uses Ethan by default."
```

## Voice Design

Create a voice from a description:

```bash
qwen-tts design "A deep, authoritative male narrator with a British accent" \
  --text "In a world where technology reigns supreme..."
```

Design a voice and narrate a file:

```bash
qwen-tts design "A cheerful young woman with an upbeat tone" \
  --file welcome_message.txt -o welcome.wav
```

## Voice Cloning

Clone a voice from a one-off sample:

```bash
qwen-tts clone \
  --ref ~/recordings/speaker.wav \
  --ref-text "This is a sample of my natural speaking voice." \
  --text "Now the model can generate new speech in this voice."
```

Save a voice for reuse, then use it:

```bash
# Enroll the voice
qwen-tts voices add narrator \
  --ref ~/recordings/narrator_sample.wav \
  --transcript "Welcome to the audiobook. My name is James."

# Use the saved voice
qwen-tts clone --voice narrator --text "Chapter one. It was a dark and stormy night."
qwen-tts clone --voice narrator --file chapter2.txt -o chapter2.wav
```

List and manage saved voices:

```bash
qwen-tts voices list
qwen-tts voices remove narrator
```

## Speed Control

Slow down for clarity:

```bash
qwen-tts speak "Please listen carefully to the following instructions." --speed 0.75
```

Speed up for previewing:

```bash
qwen-tts speak --file draft.txt --speed 1.5
```

Set a permanent default speed:

```bash
qwen-tts config set default_speed 0.9
```

## Batch Processing

Generate speech for multiple files using a shell loop:

```bash
for f in chapters/*.txt; do
  name=$(basename "$f" .txt)
  qwen-tts speak --file "$f" -o "output/${name}.wav"
done
```

Clone a voice across multiple files:

```bash
for f in scripts/*.txt; do
  name=$(basename "$f" .txt)
  qwen-tts clone --voice narrator --file "$f" -o "output/${name}.wav"
done
```

## Disabling Auto-Play

If you are generating many files and do not want each one to play:

```bash
qwen-tts config set auto_play false
```

Re-enable later:

```bash
qwen-tts config set auto_play true
```

## Model Management

Download models:

```bash
# Full-precision model (recommended)
qwen-tts models download --variant pro

# Quantized model (smaller, faster on Apple Silicon)
qwen-tts models download --variant lite
```

Switch between variants:

```bash
qwen-tts config set model_variant lite
```

List installed models:

```bash
qwen-tts models list
```
