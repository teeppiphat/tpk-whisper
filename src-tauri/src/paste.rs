use enigo::{Direction, Enigo, Key, Keyboard, Settings};

/// Put `text` on the clipboard, then synthesize Cmd+V to paste at the cursor.
/// Requires the Accessibility permission on macOS.
pub fn paste_text(text: &str) -> anyhow::Result<()> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| anyhow::anyhow!("clipboard init: {e}"))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| anyhow::anyhow!("clipboard set: {e}"))?;

    // Let the clipboard settle before pasting.
    std::thread::sleep(std::time::Duration::from_millis(120));

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| anyhow::anyhow!("enigo: {e}"))?;
    enigo
        .key(Key::Meta, Direction::Press)
        .map_err(|e| anyhow::anyhow!("key press: {e}"))?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| anyhow::anyhow!("key v: {e}"))?;
    enigo
        .key(Key::Meta, Direction::Release)
        .map_err(|e| anyhow::anyhow!("key release: {e}"))?;
    Ok(())
}
