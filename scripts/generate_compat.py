#!/usr/bin/env python3
"""
generate_compat.py -- PyTorch fallback for Qwen3-TTS on Windows/Linux.

This script provides the same CLI interface as `python -m mlx_audio.tts.generate`
so it can be used as a drop-in replacement on systems where mlx-audio is unavailable
(i.e. anywhere that is not Apple Silicon macOS).

It auto-detects CUDA availability and falls back to CPU when no GPU is found.

Usage (standalone):
    python generate_compat.py --model ./models/pro --text "Hello world" --output_path out.wav

Usage (called by qwen-tts Rust CLI):
    The Rust binary invokes this script with the same flags when backend != mlx.

# TODO: Update model loading once Qwen3-TTS transformers integration is finalized.
#       The Qwen3-TTS model class names and generate API may change as upstream
#       support in HuggingFace transformers matures. The current implementation
#       tries multiple import paths and falls back gracefully.
"""

from __future__ import annotations

import argparse
import os
import platform
import subprocess
import sys
import wave
from pathlib import Path

# ---------------------------------------------------------------------------
# Dependency checks
# ---------------------------------------------------------------------------

def _check_dependencies() -> None:
    """Verify required packages are importable and give clear errors if not."""
    missing: list[str] = []
    for pkg in ("torch", "transformers", "numpy"):
        try:
            __import__(pkg)
        except ImportError:
            missing.append(pkg)
    if missing:
        print(
            f"ERROR: Missing required packages: {', '.join(missing)}\n"
            f"Install them with:\n"
            f"  pip install {' '.join(missing)}\n",
            file=sys.stderr,
        )
        sys.exit(1)


_check_dependencies()

import numpy as np
import torch

# ---------------------------------------------------------------------------
# Device detection
# ---------------------------------------------------------------------------

def _select_device() -> torch.device:
    """Pick the best available device: CUDA > CPU."""
    if torch.cuda.is_available():
        dev = torch.device("cuda")
        name = torch.cuda.get_device_name(0)
        print(f"[device] Using CUDA ({name})")
    else:
        dev = torch.device("cpu")
        print("[device] CUDA not available, using CPU (this will be slow)")
    return dev


# ---------------------------------------------------------------------------
# Model loading helpers
# ---------------------------------------------------------------------------

def _load_model_and_processor(model_path: str, device: torch.device):
    """
    Attempt to load Qwen3-TTS using several known import paths.

    The model may be available under different class names depending on the
    transformers version:
      1. Qwen3TTSModel + Qwen3TTSProcessor  (future dedicated classes)
      2. Qwen2_5OmniModel + Qwen2_5OmniProcessor  (current HF integration)
      3. AutoModelForCausalLM + AutoTokenizer  (generic fallback)

    Returns (model, processor/tokenizer) on success, or exits on failure.
    """

    dtype = torch.float16 if device.type == "cuda" else torch.float32

    # --- Strategy 1: Dedicated Qwen3-TTS classes ---
    try:
        from transformers import Qwen3TTSModel, Qwen3TTSProcessor  # type: ignore[attr-defined]
        print(f"[model] Loading with Qwen3TTSModel from {model_path}")
        processor = Qwen3TTSProcessor.from_pretrained(model_path)
        model = Qwen3TTSModel.from_pretrained(
            model_path,
            torch_dtype=dtype,
            device_map=str(device) if device.type == "cuda" else None,
        )
        if device.type != "cuda":
            model = model.to(device)
        return model, processor
    except (ImportError, AttributeError):
        pass
    except Exception as exc:
        print(f"[model] Qwen3TTSModel found but loading failed: {exc}", file=sys.stderr)

    # --- Strategy 2: Qwen2.5-Omni classes (used by early Qwen-TTS releases) ---
    try:
        from transformers import Qwen2_5OmniModel, Qwen2_5OmniProcessor  # type: ignore[attr-defined]
        print(f"[model] Loading with Qwen2_5OmniModel from {model_path}")
        processor = Qwen2_5OmniProcessor.from_pretrained(model_path)
        model = Qwen2_5OmniModel.from_pretrained(
            model_path,
            torch_dtype=dtype,
            device_map=str(device) if device.type == "cuda" else None,
        )
        if device.type != "cuda":
            model = model.to(device)
        return model, processor
    except (ImportError, AttributeError):
        pass
    except Exception as exc:
        print(f"[model] Qwen2_5OmniModel found but loading failed: {exc}", file=sys.stderr)

    # --- Strategy 3: Generic AutoModel ---
    try:
        from transformers import AutoModelForCausalLM, AutoProcessor, AutoTokenizer

        print(f"[model] Loading with AutoModel from {model_path}")
        # Try AutoProcessor first (covers multi-modal models); fall back to tokenizer.
        try:
            processor = AutoProcessor.from_pretrained(model_path, trust_remote_code=True)
        except Exception:
            processor = AutoTokenizer.from_pretrained(model_path, trust_remote_code=True)

        model = AutoModelForCausalLM.from_pretrained(
            model_path,
            torch_dtype=dtype,
            device_map=str(device) if device.type == "cuda" else None,
            trust_remote_code=True,
        )
        if device.type != "cuda":
            model = model.to(device)
        return model, processor
    except Exception as exc:
        print(
            f"ERROR: Could not load model from {model_path}\n"
            f"  {exc}\n"
            f"Make sure the model has been downloaded (qwen-tts models download) and\n"
            f"that your transformers version supports Qwen3-TTS.\n"
            f"  pip install --upgrade transformers\n",
            file=sys.stderr,
        )
        sys.exit(1)


# ---------------------------------------------------------------------------
# Audio helpers
# ---------------------------------------------------------------------------

def _load_ref_audio(path: str, target_sr: int = 24000) -> np.ndarray:
    """Load a reference audio file and return it as a float32 numpy array.

    Tries torchaudio first, then soundfile, then falls back to reading a raw
    WAV with the stdlib wave module.
    """
    # --- torchaudio ---
    try:
        import torchaudio

        waveform, sr = torchaudio.load(path)
        if sr != target_sr:
            waveform = torchaudio.functional.resample(waveform, sr, target_sr)
        # mono
        if waveform.shape[0] > 1:
            waveform = waveform.mean(dim=0, keepdim=True)
        return waveform.squeeze(0).numpy()
    except ImportError:
        pass

    # --- soundfile ---
    try:
        import soundfile as sf

        data, sr = sf.read(path, dtype="float32")
        if data.ndim > 1:
            data = data.mean(axis=1)
        if sr != target_sr:
            # simple linear resample (not ideal but avoids extra deps)
            indices = np.linspace(0, len(data) - 1, int(len(data) * target_sr / sr))
            data = np.interp(indices, np.arange(len(data)), data).astype(np.float32)
        return data
    except ImportError:
        pass

    # --- stdlib wave (PCM WAV only) ---
    with wave.open(path, "rb") as wf:
        sr = wf.getframerate()
        n_channels = wf.getnchannels()
        sampwidth = wf.getsampwidth()
        frames = wf.readframes(wf.getnframes())
    if sampwidth == 2:
        samples = np.frombuffer(frames, dtype=np.int16).astype(np.float32) / 32768.0
    elif sampwidth == 4:
        samples = np.frombuffer(frames, dtype=np.int32).astype(np.float32) / 2147483648.0
    else:
        print(
            f"ERROR: Unsupported WAV sample width: {sampwidth} bytes. "
            f"Install torchaudio or soundfile for broader format support.",
            file=sys.stderr,
        )
        sys.exit(1)
    if n_channels > 1:
        samples = samples.reshape(-1, n_channels).mean(axis=1)
    if sr != target_sr:
        indices = np.linspace(0, len(samples) - 1, int(len(samples) * target_sr / sr))
        samples = np.interp(indices, np.arange(len(samples)), samples).astype(np.float32)
    return samples


def _save_wav(path: str, audio: np.ndarray, sample_rate: int = 24000) -> None:
    """Write a float32 numpy array as a 16-bit PCM WAV file."""
    # Ensure parent directory exists.
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)

    audio = np.clip(audio, -1.0, 1.0)
    pcm = (audio * 32767).astype(np.int16)
    with wave.open(path, "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(sample_rate)
        wf.writeframes(pcm.tobytes())


def _play_audio(path: str) -> None:
    """Play a WAV file using platform-appropriate commands."""
    system = platform.system()
    try:
        if system == "Darwin":
            subprocess.run(["afplay", path], check=True)
        elif system == "Windows":
            subprocess.run(
                [
                    "powershell",
                    "-c",
                    f"(New-Object Media.SoundPlayer '{path}').PlaySync()",
                ],
                check=True,
            )
        else:
            # Linux: try aplay, paplay, ffplay in order
            for cmd in (
                ["aplay", path],
                ["paplay", path],
                ["ffplay", "-nodisp", "-autoexit", path],
            ):
                try:
                    subprocess.run(cmd, check=True)
                    return
                except FileNotFoundError:
                    continue
            print(
                "WARNING: Could not find an audio player (tried aplay, paplay, ffplay).",
                file=sys.stderr,
            )
    except subprocess.CalledProcessError:
        print("WARNING: Audio playback failed.", file=sys.stderr)
    except FileNotFoundError:
        print("WARNING: Audio player not found.", file=sys.stderr)


# ---------------------------------------------------------------------------
# TTS generation
# ---------------------------------------------------------------------------

def _generate_speech(
    model,
    processor,
    *,
    text: str,
    instruct: str | None = None,
    voice: str | None = None,
    speed: float = 1.0,
    ref_audio: np.ndarray | None = None,
    ref_text: str | None = None,
    device: torch.device,
    sample_rate: int = 24000,
) -> np.ndarray:
    """
    Run TTS generation and return the audio waveform as a numpy array.

    This function tries multiple generation APIs in order:
      1. model.tts_generate()       -- Qwen3-TTS dedicated method
      2. model.generate()           -- generic transformers generate
    """

    # --- Build the conversation / prompt --------------------------------
    # Qwen3-TTS models typically expect a chat-style prompt with a system
    # instruction (voice/emotion) and a user turn (the text to speak).
    system_prompt = instruct or ""
    if voice and not instruct:
        system_prompt = f"You are a TTS model. Speak as {voice}."

    # --- Strategy 1: tts_generate (dedicated Qwen3-TTS method) ----------
    if hasattr(model, "tts_generate"):
        print("[generate] Using model.tts_generate()")
        kwargs: dict = {
            "text": text,
            "speed": speed,
        }
        if system_prompt:
            kwargs["instruct"] = system_prompt
        if ref_audio is not None:
            kwargs["ref_audio"] = torch.from_numpy(ref_audio).unsqueeze(0).to(device)
        if ref_text is not None:
            kwargs["ref_text"] = ref_text

        with torch.no_grad():
            result = model.tts_generate(**kwargs)

        # The return type may be a dict, a tensor, or a named tuple.
        if isinstance(result, dict):
            audio = result.get("audio") or result.get("waveform")
        elif isinstance(result, torch.Tensor):
            audio = result
        elif hasattr(result, "audio"):
            audio = result.audio
        else:
            audio = result

        if isinstance(audio, torch.Tensor):
            audio = audio.squeeze().cpu().float().numpy()
        return audio

    # --- Strategy 2: Processor + model.generate() -----------------------
    print("[generate] Using processor + model.generate()")

    # Build conversation messages in the chat-template style.
    messages = []
    if system_prompt:
        messages.append({"role": "system", "content": system_prompt})
    messages.append({"role": "user", "content": text})

    # Try to use the processor's chat template if available.
    if hasattr(processor, "apply_chat_template"):
        input_text = processor.apply_chat_template(
            messages, add_generation_prompt=True, tokenize=False
        )
    else:
        # Plain concatenation fallback.
        input_text = f"{system_prompt}\n{text}" if system_prompt else text

    # Tokenize.
    if hasattr(processor, "__call__"):
        try:
            inputs = processor(
                text=input_text,
                return_tensors="pt",
                padding=True,
            )
        except TypeError:
            # Some processors don't accept padding.
            inputs = processor(text=input_text, return_tensors="pt")
    else:
        inputs = processor.encode(input_text, return_tensors="pt")
        inputs = {"input_ids": inputs}

    # Handle reference audio for voice cloning.
    if ref_audio is not None and hasattr(processor, "feature_extractor"):
        ref_tensor = torch.from_numpy(ref_audio).unsqueeze(0)
        audio_features = processor.feature_extractor(
            ref_tensor, sampling_rate=sample_rate, return_tensors="pt"
        )
        inputs.update(audio_features)

    # Move to device.
    inputs = {
        k: v.to(device) if isinstance(v, torch.Tensor) else v
        for k, v in inputs.items()
    }

    # Generate.
    gen_kwargs: dict = {}
    if speed != 1.0:
        # Some models accept a speed parameter.
        gen_kwargs["speed"] = speed

    with torch.no_grad():
        output_ids = model.generate(**inputs, **gen_kwargs)

    # Decode audio from token IDs.
    # Qwen-TTS models typically output codec tokens that need decoding.
    if hasattr(processor, "decode_audio"):
        audio = processor.decode_audio(output_ids)
        if isinstance(audio, torch.Tensor):
            audio = audio.squeeze().cpu().float().numpy()
        return audio

    if hasattr(model, "decode_audio"):
        audio = model.decode_audio(output_ids)
        if isinstance(audio, torch.Tensor):
            audio = audio.squeeze().cpu().float().numpy()
        return audio

    # Last resort: treat raw output as waveform values (unlikely to work
    # but provides a path for experimentation).
    print(
        "WARNING: Could not find an audio decoder. Output may be incorrect.",
        file=sys.stderr,
    )
    if isinstance(output_ids, torch.Tensor):
        return output_ids.squeeze().cpu().float().numpy()
    return np.array(output_ids, dtype=np.float32)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def _build_parser() -> argparse.ArgumentParser:
    """Build an argparse parser that mirrors the mlx_audio.tts.generate CLI."""
    parser = argparse.ArgumentParser(
        description="Qwen3-TTS speech generation (PyTorch fallback)",
    )
    parser.add_argument(
        "--model",
        type=str,
        required=True,
        help="Path to the Qwen3-TTS model directory or HuggingFace repo ID",
    )
    parser.add_argument(
        "--text",
        type=str,
        required=True,
        help="Text to synthesise",
    )
    parser.add_argument(
        "--voice",
        type=str,
        default=None,
        help="Voice name (e.g. Vivian, Chelsie)",
    )
    parser.add_argument(
        "--instruct",
        type=str,
        default=None,
        help="Instruction/style prompt (e.g. 'Speak calmly and clearly')",
    )
    parser.add_argument(
        "--speed",
        type=float,
        default=1.0,
        help="Speech speed multiplier (default: 1.0)",
    )
    parser.add_argument(
        "--ref_audio",
        type=str,
        default=None,
        help="Path to reference audio file for voice cloning",
    )
    parser.add_argument(
        "--ref_text",
        type=str,
        default=None,
        help="Transcript of the reference audio",
    )
    parser.add_argument(
        "--play",
        action="store_true",
        default=False,
        help="Auto-play the generated audio",
    )
    parser.add_argument(
        "--output_path",
        type=str,
        default="output.wav",
        help="Output WAV file path (default: output.wav)",
    )
    return parser


def main() -> None:
    parser = _build_parser()
    args = parser.parse_args()

    # ------------------------------------------------------------------
    # Device
    # ------------------------------------------------------------------
    device = _select_device()

    # ------------------------------------------------------------------
    # Model
    # ------------------------------------------------------------------
    print(f"[init] Loading model from {args.model} ...")
    model, processor = _load_model_and_processor(args.model, device)
    model.eval()

    # ------------------------------------------------------------------
    # Reference audio (voice cloning)
    # ------------------------------------------------------------------
    ref_audio: np.ndarray | None = None
    if args.ref_audio:
        if not os.path.isfile(args.ref_audio):
            print(f"ERROR: Reference audio file not found: {args.ref_audio}", file=sys.stderr)
            sys.exit(1)
        print(f"[ref] Loading reference audio from {args.ref_audio}")
        ref_audio = _load_ref_audio(args.ref_audio)

    # ------------------------------------------------------------------
    # Generate
    # ------------------------------------------------------------------
    print(f"[tts] Generating speech ({len(args.text)} chars) ...")
    audio = _generate_speech(
        model,
        processor,
        text=args.text,
        instruct=args.instruct,
        voice=args.voice,
        speed=args.speed,
        ref_audio=ref_audio,
        ref_text=args.ref_text,
        device=device,
    )

    # ------------------------------------------------------------------
    # Save
    # ------------------------------------------------------------------
    _save_wav(args.output_path, audio)
    print(f"[done] Saved to {args.output_path}")

    # ------------------------------------------------------------------
    # Play
    # ------------------------------------------------------------------
    if args.play:
        _play_audio(args.output_path)


if __name__ == "__main__":
    main()
