use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use directories::UserDirs;
use tauri::{Manager, image::Image};
use tauri_plugin_clipboard_manager::ClipboardExt;
use xcap::Monitor;

#[derive(Clone, Copy, Debug)]
pub struct ScreenshotAction;

impl ScreenshotAction {
    pub fn parse(query: &str) -> Option<Self> {
        match query
            .trim()
            .trim_start_matches('>')
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "screenshot" | "print screen" | "capture screen" | "tirar print" | "print da tela"
            | "captura de tela" => Some(Self),
            _ => None,
        }
    }

    pub fn execute(self, app: &tauri::AppHandle) -> Result<(), String> {
        let window = app.get_webview_window("main");
        if let Some(window) = &window {
            window
                .hide()
                .map_err(|error| format!("failed to hide Arclume before capture: {error}"))?;
        }
        thread::sleep(Duration::from_millis(180));

        let result = capture_primary_to_disk().and_then(|(_, rgba, width, height)| {
            app.clipboard()
                .write_image(&Image::new_owned(rgba, width, height))
                .map_err(|error| format!("screenshot was saved but could not be copied: {error}"))
        });
        if result.is_err()
            && let Some(window) = window
        {
            let _ = window.show();
            let _ = window.set_focus();
        }
        result
    }
}

fn capture_primary_to_disk() -> Result<(PathBuf, Vec<u8>, u32, u32), String> {
    let monitor = Monitor::all()
        .map_err(|error| format!("failed to list screens: {error}"))?
        .into_iter()
        .find(|monitor| monitor.is_primary().unwrap_or(false))
        .ok_or("primary screen was not found")?;
    let captured = monitor
        .capture_image()
        .map_err(|error| format!("failed to capture primary screen: {error}"))?;
    let (width, height) = captured.dimensions();
    let rgba = captured.into_raw();
    let path = capture_path()?;

    image::save_buffer_with_format(
        &path,
        &rgba,
        width,
        height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .map_err(|error| format!("failed to save screenshot: {error}"))?;
    Ok((path, rgba, width, height))
}

fn capture_path() -> Result<PathBuf, String> {
    let user_dirs = UserDirs::new().ok_or("user directories are unavailable")?;
    let base = user_dirs
        .picture_dir()
        .map(PathBuf::from)
        .unwrap_or_else(|| user_dirs.home_dir().join("Pictures"));
    let directory = base.join("Arclume");
    fs::create_dir_all(&directory)
        .map_err(|error| format!("failed to create screenshot directory: {error}"))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock is before Unix epoch")?
        .as_millis();
    Ok(directory.join(format!("Arclume-{timestamp}.png")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_explicit_screenshot_commands() {
        assert!(ScreenshotAction::parse("screenshot").is_some());
        assert!(ScreenshotAction::parse("> tirar print").is_some());
        assert!(ScreenshotAction::parse("captura de tela").is_some());
        assert!(ScreenshotAction::parse("screenshot anything").is_none());
    }

    #[test]
    #[ignore = "captures the real desktop and writes to the user's Pictures directory"]
    fn real_primary_screen_capture() {
        let (path, _, width, height) = capture_primary_to_disk().expect("screen capture failed");
        assert!(path.is_file());
        assert!(width > 0 && height > 0);
        println!("{}", path.display());
    }
}
