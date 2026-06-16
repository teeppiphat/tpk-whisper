# tpk-whisper

A tiny, MacWhisper-style dictation tool for macOS. **Hold** a global hotkey and
talk; **release** it and your speech is transcribed and pasted straight into
whatever app you're focused on.

By default it runs the [Typhoon ASR](https://github.com/scb-10x/typhoon-asr)
model (`typhoon-asr-realtime`) **locally** — offline, no API key. You can switch
to Typhoon's cloud API in Settings if you prefer.

Tray-only Tauri v2 binary (native WKWebView, no Electron). See
[`ARCHITECTURE.md`](./ARCHITECTURE.md) for the design.
Thai README: [`README.th.md`](./README.th.md).

## How it works

1. **Hold** the global hotkey (default `Ctrl+Alt+D`) to record; **release** to
   transcribe (push-to-talk).
2. Mic is captured with `cpal`, downmixed to mono 16-bit, written to a temp `.wav`.
3. The WAV is transcribed by the selected backend:
   - **Local (default):** a bundled `local_transcribe.py` runs `typhoon-asr` on
     your machine via your configured launcher (uv by default).
   - **API:** the WAV is POSTed to `https://api.opentyphoon.ai/v1/audio/transcriptions`
     (OpenAI-compatible) with `model=typhoon-asr-realtime`.
4. The returned text is put on the clipboard and pasted with a synthetic ⌘V.
5. The temp WAV is deleted right after; any leftovers are swept on next launch.

API mode obeys a client-side rate limit of 100 requests/minute (the model's
limit); local mode has no key, no network, and no rate limit.

## Prerequisites (on your Mac)

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Xcode command line tools (for the macOS toolchain)
xcode-select --install

# Tauri CLI
cargo install tauri-cli --version "^2"

# uv — used by the default local backend
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## App icons (one-time)

Tauri needs an icon set referenced in `tauri.conf.json`. Generate it from any
square PNG (1024×1024 recommended):

```bash
cd tpk-whisper
cargo tauri icon path/to/your-logo.png
```

This populates `src-tauri/icons/`. (Until you do this, the build will complain
about missing icons.)

## Run / build

```bash
cd tpk-whisper

# Dev (opens the tray app)
cargo tauri dev

# Release build → src-tauri/target/release/bundle/macos/tpk-whisper.app
cargo tauri build
```

## First-run setup

The app defaults to **Local (offline)** mode, which runs via `uv` automatically —
just have [`uv`](https://docs.astral.sh/uv/) installed; no API key, no manual
`pip install`.

1. Install `uv` if needed (see Prerequisites).
2. Launch the app — it lives in the **menu bar** (no Dock icon).
3. Click the tray icon → **Settings…**, set the hotkey (see below), click **Save**.
4. Grant macOS permissions when prompted (see below).

The first transcription downloads Python 3.10 + `typhoon-asr` (torch/NeMo, several
GB) via uv and loads the model; this is slow once. Subsequent runs reuse the uv
cache and are much faster. To use the cloud API instead, switch the backend to
*Typhoon API* in Settings and paste a key from
[playground.opentyphoon.ai](https://playground.opentyphoon.ai/settings/api-key).

## Hotkey (push-to-talk)

Set it in **Settings → Record**: click *Record*, then press your combo (e.g. hold
⌃⌥ and tap D). The app stores it in Tauri syntax (`Control+Alt+KeyD`,
`Super+Shift+Space`; `Super` = ⌘, letters are `KeyX`). At least one modifier is
required. **Hold** the hotkey to record, **release** to transcribe.

## Backends

Choose under **Settings → Transcription backend**.

### Local (default, offline)

Runs `typhoon-asr-realtime` on your machine — no API key, no network (after the
one-time model download), no rate limit, audio never leaves the device.

The **Python interpreter / launcher** field accepts a bare path **or** a full
launcher (it's split on whitespace). Pick one of:

- **uv, no manual install (default):** `uv run --python 3.10 --with typhoon-asr python`
  — uv builds a cached, pinned env with the package on first run.
- **uv venv:** `uv venv --python 3.10 && uv pip install typhoon-asr`, then point the
  field at `.venv/bin/python`.
- **pip:** `pip install typhoon-asr` (Python 3.10), then use `python3` or a venv's python.

Also configurable: **Device** (`auto`/`cpu`/`cuda`; Macs use `cpu`) and **Model id**
(`scb10x/typhoon-asr-realtime`, or `scb10x/typhoon-isan-asr-realtime` for Isan).

The app ships `local_transcribe.py` (embedded in the binary) and runs it as a
subprocess. The child process gets an augmented `PATH` (`~/.local/bin`,
`~/.cargo/bin`, `/opt/homebrew/bin`, …) so `uv`/`python` are found even when the
app is launched from Finder. Backed by
[scb-10x/typhoon-asr](https://github.com/scb-10x/typhoon-asr) (NeMo + PyTorch).

### Typhoon API (cloud)

Paste an API key (free at
[playground.opentyphoon.ai](https://playground.opentyphoon.ai/settings/api-key)).
The WAV is sent to Typhoon's OpenAI-compatible endpoint. Subject to 100 req/min.

## macOS permissions

Grant these in **System Settings → Privacy & Security**:

- **Microphone** — to record your voice (prompted automatically on first record).
- **Accessibility** — required so the global hotkey works while other apps are
  focused, and so the app can paste with ⌘V. Add `tpk-whisper` (or your terminal,
  during `cargo tauri dev`) under *Accessibility*.
- **Input Monitoring** — may be requested for global key capture.

If the hotkey or auto-paste doesn't work, it's almost always a missing
Accessibility grant. Toggle the app off/on in that list and relaunch.

## Notes & limitations

- **Cold start each time:** local mode currently loads the model on every
  transcription, so there's a few-seconds delay before text appears. (A persistent
  worker process that keeps the model loaded would remove this — see ARCHITECTURE.)
- **Pasting** uses a synthetic ⌘V against the clipboard, so it briefly overwrites
  your clipboard contents and won't work in apps that block synthetic input.
- **Thai-focused model:** `typhoon-asr-realtime` is optimized for Thai.

## Project layout

```
tpk-whisper/
├── ARCHITECTURE.md
├── README.md             # this file
├── README.th.md          # Thai README
├── src/                  # settings UI (plain HTML/JS, no bundler)
│   └── index.html
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── Info.plist        # mic usage string + LSUIElement (menu-bar app)
    ├── capabilities/default.json
    ├── build.rs
    ├── python/
    │   └── local_transcribe.py   # local inference wrapper (embedded via include_str!)
    └── src/
        ├── main.rs
        ├── lib.rs        # tray, hotkey, state, commands, pipeline, temp sweep
        ├── audio.rs      # cpal capture → mono 16-bit WAV
        ├── transcribe.rs # Typhoon API + local subprocess backends
        ├── paste.rs      # arboard clipboard + enigo ⌘V (raw V keycode)
        ├── config.rs     # JSON config (backend, key, hotkey, launcher, …)
        └── ratelimit.rs  # 100 req/min sliding window (API mode)
```

## License

Apache-2.0 (matches the Typhoon ASR model license).
