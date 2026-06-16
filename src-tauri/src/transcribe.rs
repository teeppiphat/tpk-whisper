use serde::Deserialize;
use std::path::Path;
use std::process::Command;

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: String,
}

/// POST a WAV file to the Typhoon ASR OpenAI-compatible endpoint and return text.
/// Mirrors the docs' `transcribe_audio_file`:
///   client.audio.transcriptions.create(file=..., model="typhoon-asr-realtime")
pub fn transcribe(
    path: &Path,
    api_key: &str,
    base_url: &str,
    model: &str,
) -> anyhow::Result<String> {
    let bytes = std::fs::read(path)?;
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("audio.wav")
        .to_string();

    let part = reqwest::blocking::multipart::Part::bytes(bytes)
        .file_name(file_name)
        .mime_str("audio/wav")?;
    let form = reqwest::blocking::multipart::Form::new()
        .text("model", model.to_string())
        .part("file", part);

    let url = format!("{}/audio/transcriptions", base_url.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let resp = client
        .post(url)
        .bearer_auth(api_key)
        .multipart(form)
        .send()?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(anyhow::anyhow!("Typhoon ASR error {status}: {body}"));
    }

    let parsed: TranscriptionResponse = resp.json()?;
    Ok(parsed.text)
}

/// Local backend: run the `typhoon-asr` model on this machine via a bundled
/// Python script (`local_transcribe.py`). No network, no API key.
/// The script prints one JSON line: {"text": "..."} or {"error": "..."}.
///
/// `python_cmd` may be a bare interpreter path (`python3`,
/// `/path/.venv/bin/python`) OR a full launcher with arguments
/// (`uv run --with typhoon-asr python`). It is split on whitespace: the first
/// token is the program, the rest are prepended before the script.
pub fn transcribe_local(
    audio: &Path,
    python_cmd: &str,
    script: &Path,
    model: &str,
    device: &str,
) -> anyhow::Result<String> {
    let mut parts = python_cmd.split_whitespace();
    let program = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("empty Python command"))?;

    let mut command = Command::new(program);
    for arg in parts {
        command.arg(arg);
    }
    // GUI apps launched from Finder get a minimal PATH and won't find uv/python.
    // Augment it with the usual install locations so the default `uv …` works.
    command.env("PATH", augmented_path());
    let output = command
        .arg(script)
        .arg(audio)
        .arg("--model")
        .arg(model)
        .arg("--device")
        .arg(device)
        .output()
        .map_err(|e| anyhow::anyhow!("could not launch '{program}': {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Find the last JSON object line the script printed.
    let json_line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .ok_or_else(|| {
            anyhow::anyhow!("no output from local ASR. stderr: {}", stderr.trim())
        })?;

    let value: serde_json::Value = serde_json::from_str(json_line)?;
    if let Some(err) = value.get("error").and_then(|v| v.as_str()) {
        return Err(anyhow::anyhow!("local ASR: {err}"));
    }
    Ok(value
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string())
}

/// Build a PATH that includes common locations for uv / homebrew / cargo,
/// so the local backend works even when the app is launched from Finder.
fn augmented_path() -> String {
    let mut dirs: Vec<String> = std::env::var("PATH")
        .ok()
        .map(|p| p.split(':').map(String::from).collect())
        .unwrap_or_default();

    if let Some(home) = dirs::home_dir() {
        for sub in [".local/bin", ".cargo/bin"] {
            dirs.push(home.join(sub).to_string_lossy().into_owned());
        }
    }
    for d in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"] {
        dirs.push(d.to_string());
    }

    let mut seen = std::collections::HashSet::new();
    dirs.retain(|d| !d.is_empty() && seen.insert(d.clone()));
    dirs.join(":")
}
