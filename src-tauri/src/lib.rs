mod audio;
mod config;
mod paste;
mod ratelimit;
mod transcribe;

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use audio::Recorder;
use config::Config;
use ratelimit::RateLimiter;

struct AppState {
    recorder: Mutex<Recorder>,
    limiter: Mutex<RateLimiter>,
    config: Mutex<Config>,
}

/// Emit a status string to the settings window (and stderr).
fn status(app: &AppHandle, msg: &str) {
    eprintln!("[tpk-whisper] {msg}");
    let _ = app.emit("status", msg);
}

/// Push-to-talk: start capturing when the hotkey is pressed.
/// Guards against key-repeat firing repeated Pressed events.
fn start_recording(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut recorder = state.recorder.lock().unwrap();
    if recorder.is_recording() {
        return; // already recording (key auto-repeat) — ignore
    }
    match recorder.start() {
        Ok(()) => status(app, "Recording… (release hotkey to transcribe)"),
        Err(e) => status(app, &format!("Could not start recording: {e}")),
    }
}

/// Push-to-talk: stop on key release, then transcribe + paste.
fn stop_and_transcribe(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut recorder = state.recorder.lock().unwrap();
    if !recorder.is_recording() {
        return;
    }
    match recorder.stop() {
        Ok(Some(path)) => {
            status(app, "Transcribing…");
            let app = app.clone();
            std::thread::spawn(move || run_pipeline(app, path));
        }
        Ok(None) => {}
        Err(e) => status(app, &format!("Recording error: {e}")),
    }
}

/// The bundled local-inference Python script, written to disk on demand.
const LOCAL_SCRIPT: &str = include_str!("../python/local_transcribe.py");

/// Ensure local_transcribe.py exists next to the config and return its path.
fn ensure_local_script() -> PathBuf {
    let mut p = config::data_dir();
    p.push("local_transcribe.py");
    // Always rewrite so script updates ship with new app versions.
    let _ = std::fs::write(&p, LOCAL_SCRIPT);
    p
}

/// Runs on a background thread: transcribe (API or local), paste, clean up.
fn run_pipeline(app: AppHandle, path: PathBuf) {
    let state = app.state::<AppState>();
    let cfg = state.config.lock().unwrap().clone();

    let result = if cfg.backend == "local" {
        let script = ensure_local_script();
        transcribe::transcribe_local(&path, &cfg.python_path, &script, &cfg.local_model, &cfg.device)
    } else {
        // API backend: needs a key + obeys the 100/min rate limit.
        if cfg.api_key.trim().is_empty() {
            status(&app, "No API key set. Open Settings to add your Typhoon API key.");
            if let Some(w) = app.get_webview_window("settings") {
                let _ = w.show();
                let _ = w.set_focus();
            }
            let _ = std::fs::remove_file(&path);
            return;
        }
        if !state.limiter.lock().unwrap().try_acquire() {
            status(&app, "Rate limit reached (100/min). Try again shortly.");
            let _ = std::fs::remove_file(&path);
            return;
        }
        transcribe::transcribe(&path, &cfg.api_key, &cfg.base_url, &cfg.model)
    };

    match result {
        Ok(text) if !text.trim().is_empty() => match paste::paste_text(&text) {
            Ok(()) => status(&app, "Pasted ✓"),
            Err(e) => status(&app, &format!("Transcribed but paste failed: {e}")),
        },
        Ok(_) => status(&app, "No speech detected."),
        Err(e) => status(&app, &format!("Transcription failed: {e}")),
    }

    let _ = std::fs::remove_file(&path);
}

#[tauri::command]
fn get_config(state: State<AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

#[derive(serde::Deserialize)]
struct ConfigInput {
    api_key: String,
    hotkey: String,
    backend: String,
    python_path: String,
    local_model: String,
    device: String,
}

#[tauri::command]
fn save_config(
    app: AppHandle,
    state: State<AppState>,
    input: ConfigInput,
) -> Result<(), String> {
    let new_hotkey = input.hotkey.trim().to_string();

    // Re-register the global shortcut.
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    gs.register(new_hotkey.as_str())
        .map_err(|e| format!("Invalid hotkey '{new_hotkey}': {e}"))?;

    let mut cfg = state.config.lock().unwrap();
    cfg.api_key = input.api_key.trim().to_string();
    cfg.hotkey = new_hotkey;
    cfg.backend = if input.backend == "local" { "local" } else { "api" }.to_string();
    cfg.python_path = {
        let p = input.python_path.trim();
        if p.is_empty() { config::DEFAULT_PYTHON_CMD.to_string() } else { p.to_string() }
    };
    cfg.local_model = {
        let m = input.local_model.trim();
        if m.is_empty() { "scb10x/typhoon-asr-realtime".to_string() } else { m.to_string() }
    };
    cfg.device = match input.device.as_str() {
        "cpu" | "cuda" => input.device.clone(),
        _ => "auto".to_string(),
    };
    cfg.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cfg = Config::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| match event.state() {
                    ShortcutState::Pressed => start_recording(app),
                    ShortcutState::Released => stop_and_transcribe(app),
                })
                .build(),
        )
        .manage(AppState {
            recorder: Mutex::new(Recorder::new()),
            limiter: Mutex::new(RateLimiter::new(100, Duration::from_secs(60))),
            config: Mutex::new(cfg.clone()),
        })
        .invoke_handler(tauri::generate_handler![get_config, save_config])
        .setup(move |app| {
            // Sweep any temp WAVs left behind by a previous crash.
            audio::cleanup_leftovers();

            // Tray menu: Settings… / Quit
            let settings_i = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &quit_i])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("tpk-whisper")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        if let Some(w) = app.get_webview_window("settings") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            // Register the configured hotkey (best-effort; needs Accessibility).
            if let Err(e) = app.global_shortcut().register(cfg.hotkey.as_str()) {
                eprintln!("[tpk-whisper] failed to register hotkey '{}': {e}", cfg.hotkey);
            }

            // Keep the settings window hidden on launch (tray-only app).
            if let Some(w) = app.get_webview_window("settings") {
                let _ = w.hide();
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tpk-whisper")
        .run(|_app, event| {
            // Don't quit when the settings window is closed; stay in the tray.
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
