use anyhow::{Context, Result};
use screenshots::Screen;

pub struct Screenshot;

impl Screenshot {
    // Screenshot and return as bytes
    pub fn capture_as_bytes() -> Result<(Vec<u8>, String)> {
        Self::capture_as_bytes_windows()
    }

    #[cfg(target_os = "windows")]
    fn capture_as_bytes_windows() -> Result<(Vec<u8>, String)> {
        let displays = Screen::all().context("Failed to enumerate displays")?;

        if displays.is_empty() {
            return Err(anyhow::anyhow!("No displays found"));
        }

        let display = &displays[0];
        let image = display.capture().context("Failed to capture screen")?;

        let buffer = image.buffer().clone();

        let random_id: u64 = rand::random();
        let filename = format!("screenshot_{:x}.png", random_id);

        Ok((buffer, filename))
    }
}
