# tpk-whisper

A tiny, MacWhisper-style dictation tool for macOS. Press a global hotkey, talk,
press again — your speech is transcribed by [Typhoon ASR](https://docs.opentyphoon.ai/en/asr/)
(`typhoon-asr-realtime`) and pasted straight into whatever app you're focused on.

Tray-only Tauri v2 binary (native WKWebView, no Electron). See
[`ARCHITECTURE.md`](./ARCHITECTURE.md) for the design.

## How it works

1. Hold the global hotkey (default `Ctrl+Alt+D`) to record; release to transcribe (push-to-talk).
2. Mic is captured with `cpal`, downmixed to mono 16-bit, written to a temp `.wav`.
3. The WAV is POSTed to `https://api.opentyphoon.ai/v1/audio/transcriptions`
   (OpenAI-compatible) with `model=typhoon-asr-realtime`.
4. Returned text is put on the clipboard and pasted with a synthetic ⌘V.

Rate limiting is enforced client-side at 100 requests/minute to match the model's limit.

## Prerequisites (on your Mac)

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Xcode command line tools (for the macOS toolchain)
xcode-select --install

# Tauri CLI
cargo install tauri-cli --version "^2"
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

# Dev (hot-ish; opens the tray app)
cargo tauri dev

# Release build → src-tauri/target/release/bundle/macos/tpk-whisper.app
cargo tauri build
```

## First-run setup

1. Launch the app — it lives in the **menu bar** (no Dock icon).
2. Click the tray icon → **Settings…**, paste your Typhoon API key
   (free at [playground.opentyphoon.ai](https://playground.opentyphoon.ai/asr)),
   optionally change the hotkey, click **Save**.
3. Grant macOS permissions when prompted (see below).

## macOS permissions

Grant these in **System Settings → Privacy & Security**:

- **Microphone** — to record your voice (prompted automatically on first record).
- **Accessibility** — required so the global hotkey works while other apps are
  focused, and so the app can paste with ⌘V. Add `tpk-whisper` (or your terminal,
  during `cargo tauri dev`) under *Accessibility*.
- **Input Monitoring** — may be requested for global key capture.

If the hotkey or auto-paste doesn't work, it's almost always a missing
Accessibility grant. Toggle the app off/on in that list and relaunch.

## Hotkey

Set it in **Settings → Record**: click *Record*, then press your combo (e.g.
hold ⌃⌥ and tap D). The app stores it in Tauri syntax (`Control+Alt+KeyD`,
`Super+Shift+Space`; `Super` = ⌘, letters are `KeyX`). At least one modifier is
required. **Hold** the hotkey to record, **release** to transcribe.

## Project layout

```
tpk-whisper/
├── ARCHITECTURE.md
├── README.md
├── src/                  # settings UI (plain HTML/JS, no bundler)
│   └── index.html
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── Info.plist        # mic usage string + LSUIElement (menu-bar app)
    ├── capabilities/default.json
    ├── build.rs
    └── src/
        ├── main.rs
        ├── lib.rs        # tray, hotkey, state, commands, pipeline
        ├── audio.rs      # cpal capture → mono 16-bit WAV
        ├── transcribe.rs # reqwest multipart → Typhoon ASR
        ├── paste.rs      # arboard clipboard + enigo ⌘V
        ├── config.rs     # JSON config (API key, hotkey)
        └── ratelimit.rs  # 100 req/min sliding window
```

## License

Apache-2.0 (matches the Typhoon ASR model license).
