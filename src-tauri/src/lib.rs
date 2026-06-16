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

/// Runs on a background thread: rate-limit, transcribe, paste, clean up.
fn run_pipeline(app: AppHandle, path: PathBuf) {
    let state = app.state::<AppState>();

    let (api_key, base_url, model) = {
        let cfg = state.config.lock().unwrap();
        (cfg.api_key.clone(), cfg.base_url.clone(), cfg.model.clone())
    };

    if api_key.trim().is_empty() {
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

    match transcribe::transcribe(&path, &api_key, &base_url, &model) {
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

#[tauri::command]
fn save_config(
    app: AppHandle,
    state: State<AppState>,
    api_key: String,
    hotkey: String,
) -> Result<(), String> {
    let new_hotkey = hotkey.trim().to_string();

    // Re-register the global shortcut.
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    gs.register(new_hotkey.as_str())
        .map_err(|e| format!("Invalid hotkey '{new_hotkey}': {e}"))?;

    let mut cfg = state.config.lock().unwrap();
    cfg.api_key = api_key.trim().to_string();
    cfg.hotkey = new_hotkey;
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
