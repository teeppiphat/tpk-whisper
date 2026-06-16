#!/usr/bin/env python3
"""Minimal wrapper around the `typhoon-asr` package for tpk-whisper's local
backend. Transcribes one audio file and prints a single JSON line: {"text": ...}.

Install once on the user's machine:
    pip install typhoon-asr

Usage:
    python local_transcribe.py <audio.wav> [--model NAME] [--device auto|cpu|cuda]
"""
import argparse
import json
import sys


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("input")
    parser.add_argument("--model", default="scb10x/typhoon-asr-realtime")
    parser.add_argument("--device", default="auto")
    args = parser.parse_args()

    try:
        from typhoon_asr import transcribe
    except Exception as e:  # noqa: BLE001
        print(
            json.dumps(
                {"error": f"typhoon-asr not installed: {e}. Run: pip install typhoon-asr"}
            ),
            flush=True,
        )
        return 2

    try:
        result = transcribe(args.input, model_name=args.model, device=args.device)
    except Exception as e:  # noqa: BLE001
        print(json.dumps({"error": str(e)}), flush=True)
        return 1

    raw = result.get("text", "") if isinstance(result, dict) else result
    text = to_text(raw)
    # Single, clean JSON line on stdout — the Rust side parses the last such line.
    print(json.dumps({"text": text}, ensure_ascii=False), flush=True)
    return 0


def to_text(value) -> str:
    """Coerce typhoon_asr / NeMo output into a plain string.

    Depending on version, transcribe() may return a str, a NeMo `Hypothesis`
    object (with a `.text` attribute), or a list of either.
    """
    if isinstance(value, str):
        return value
    if isinstance(value, (list, tuple)):
        return " ".join(to_text(v) for v in value).strip()
    inner = getattr(value, "text", None)
    if isinstance(inner, str):
        return inner
    return str(value)


if __name__ == "__main__":
    sys.exit(main())
