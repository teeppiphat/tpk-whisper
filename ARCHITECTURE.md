# tpk-whisper — Architecture

A minimal, MacWhisper-style dictation tool for macOS. Press a global hotkey,
speak into the mic, press again — the audio is recorded to a temp file, sent to
[Typhoon ASR](https://docs.opentyphoon.ai/en/asr/) (`typhoon-asr-realtime`,
OpenAI-compatible API), and the returned text is **auto-pasted at the cursor**.

The whole app is a tray-only Tauri v2 binary. No persistent window, no Electron,
no heavy framework. A tiny settings window opens only when you need to set the
API key or change the hotkey.

Recording is **push-to-talk**: hold the hotkey to record, release to transcribe.

## Flow

```
                ┌──────────────────────────────────────────────────────────┐
                │                  tpk-whisper (tray app)                     │
                │                                                            │
 ⌨ hold hotkey ▶│  global-shortcut  ─press/release▶  Recorder (cpal)        │
  (Ctrl+Opt+D)  │                                    │                       │
                │                                    │ mic samples           │
                │                                    ▼                       │
                │                          buffer ──stop──▶ WAV (hound)      │
                │                                    │  /tmp/tpk-xxx.wav      │
                │                                    ▼                       │
                │   RateLimiter (≤100/min) ──▶ Transcriber (reqwest)         │
                │                                    │  multipart POST        │
                │                                    ▼                       │
                │                    https://api.opentyphoon.ai/v1/          │
                │                         audio/transcriptions               │
                │                                    │  { "text": "..." }     │
                │                                    ▼                       │
                │       arboard (set clipboard) ──▶ enigo (Cmd+V)            │
                │                                    │                       │
                └────────────────────────────────────┼───────────────────────┘
                                                      ▼
                                          text appears at cursor
                                          in whatever app is focused
```

## Why this stack is "lightest"

- **Tauri v2** ships a single native binary using the macOS system WebView
  (WKWebView). No bundled Chromium. The settings UI is plain HTML/JS — no React,
  no npm build step.
- **cpal** — thin Rust binding over CoreAudio for mic capture.
- **hound** — tiny WAV encoder.
- **reqwest** — HTTP with multipart; the Typhoon endpoint is OpenAI-compatible
  so we just POST the file to `/v1/audio/transcriptions` with `model=typhoon-asr-realtime`.
- **arboard** + **enigo** — set the clipboard, then synthesize ⌘V to paste at
  the cursor (this is how MacWhisper-style "type anywhere" works).

## Modules (`src-tauri/src/`)

| File             | Responsibility |
|------------------|----------------|
| `lib.rs`         | App setup, tray, global hotkey registration, state, Tauri commands |
| `audio.rs`       | `Recorder`: start/stop mic capture on a dedicated thread → mono 16-bit WAV |
| `transcribe.rs`  | `transcribe` (POST WAV to Typhoon) + `transcribe_local` (subprocess to bundled Python) |
| `paste.rs`       | Set clipboard + simulate ⌘V |
| `ratelimit.rs`   | Sliding-window guard, ≤100 requests/minute |
| `config.rs`      | Load/save `~/Library/Application Support/ai.bedrock.tpkwhisper/config.json` (API key, hotkey) |

## Recording model

**Push-to-talk.** The global-shortcut handler receives both `Pressed` and
`Released` events: `Pressed` starts capture (key auto-repeat is ignored via an
"already recording" guard), `Released` stops it. On release the WAV is finalized
and the transcribe→paste pipeline runs on a background thread so the UI/tray
never blocks. The hotkey itself is set in Settings by pressing the key combo.

Audio is captured at the device's native sample rate, **downmixed to mono** and
converted to **16-bit PCM** to keep files small (Typhoon accepts `.wav`).

## macOS permissions (granted once, by the user)

- **Microphone** — to record. Declared via `NSMicrophoneUsageDescription`.
- **Accessibility** — required for the global hotkey to fire while other apps
  are focused, and for `enigo` to synthesize the ⌘V keystroke.
- **Input Monitoring** — may be requested for global key capture.

These are OS-level grants; the app guides the user to System Settings on first run.

## Backends: API vs Local

A `backend` config field selects how audio is transcribed:

- **`local`** (default) — run `typhoon-asr-realtime` on this machine. The app embeds a tiny
  `local_transcribe.py` (via `include_str!`), writes it next to the config, and
  runs `<launcher> local_transcribe.py <wav> --model … --device …` as a subprocess,
  parsing the `{ "text": … }` JSON line it prints. The launcher field is split on
  whitespace, so it accepts a bare interpreter (`python3`, `.venv/bin/python`) or a
  full command like `uv run --with typhoon-asr python`. No API key, no network
  (after the one-time HuggingFace model download), no rate limit. The default
  launcher is `uv run --python 3.10 --with typhoon-asr python`, and the child
  process is given an augmented PATH (`~/.local/bin`, `~/.cargo/bin`,
  `/opt/homebrew/bin`, …) so it works even when the app is launched from Finder.
- **`api`** — POST the WAV to Typhoon's OpenAI-compatible endpoint. Requires an
  API key and obeys the 100/min rate limit.

Both paths converge on the same clipboard+paste step, so the rest of the app is
backend-agnostic.

## Rate limiting

`typhoon-asr-realtime` allows 100 requests/minute. The `RateLimiter` keeps a
sliding window of request timestamps and blocks (or rejects with a tray message)
if a new request would exceed 100 in the last 60 seconds. For a single-user
dictation tool this is effectively never hit, but it's enforced defensively.
