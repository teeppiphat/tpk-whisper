use enigo::{Direction, Enigo, Key, Keyboard, Settings};

/// macOS virtual keycode for the physical "V" key (kVK_ANSI_V).
/// We use a raw keycode rather than `Key::Unicode('v')` because enigo types
/// Unicode chars via CGEventKeyboardSetUnicodeString, which ignores modifier
/// flags — so Cmd+Unicode('v') inserts a literal "v" instead of pasting.
#[cfg(target_os = "macos")]
const KEY_V: Key = Key::Other(9);
#[cfg(not(target_os = "macos"))]
const KEY_V: Key = Key::Unicode('v');

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
        .map_err(|e| anyhow::anyhow!("meta press: {e}"))?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    enigo
        .key(KEY_V, Direction::Click)
        .map_err(|e| anyhow::anyhow!("v click: {e}"))?;
    std::thread::sleep(std::time::Duration::from_millis(20));
    enigo
        .key(Key::Meta, Direction::Release)
        .map_err(|e| anyhow::anyhow!("meta release: {e}"))?;
    Ok(())
}
