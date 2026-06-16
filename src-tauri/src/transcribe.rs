use serde::Deserialize;
use std::path::Path;

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
