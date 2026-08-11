use std::path::PathBuf;
use std::sync::Mutex;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub const DEFAULT_SHORTCUT: &str = "CmdOrControl+Space";
pub const ALLOWED_SHORTCUTS: &[&str] = &[
    DEFAULT_SHORTCUT,
    "CmdOrControl+Shift+Space",
    "Alt+Space",
    "CmdOrControl+Alt+Space",
];

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ShortcutConfig {
    global_shortcut: String,
}

pub struct ShortcutSettings {
    current: Mutex<String>,
}

impl ShortcutSettings {
    pub fn load() -> Self {
        let current = load_config()
            .filter(|value| is_allowed(value))
            .unwrap_or_else(|| DEFAULT_SHORTCUT.into());
        Self {
            current: Mutex::new(current),
        }
    }

    pub fn current(&self) -> String {
        self.current
            .lock()
            .map_or_else(|_| DEFAULT_SHORTCUT.into(), |value| value.clone())
    }

    fn replace(&self, value: String) -> Result<(), String> {
        let mut current = self
            .current
            .lock()
            .map_err(|_| "shortcut settings are unavailable")?;
        *current = value;
        Ok(())
    }
}

#[tauri::command]
pub fn get_global_shortcut(settings: tauri::State<'_, ShortcutSettings>) -> String {
    settings.current()
}

#[tauri::command]
pub fn set_global_shortcut(
    shortcut: String,
    app: AppHandle,
    settings: tauri::State<'_, ShortcutSettings>,
) -> Result<(), String> {
    if !is_allowed(&shortcut) {
        return Err("unsupported shortcut".into());
    }
    let previous = settings.current();
    if shortcut == previous {
        return Ok(());
    }

    app.global_shortcut()
        .unregister(previous.as_str())
        .map_err(|error| format!("failed to release current shortcut: {error}"))?;
    if let Err(error) = app.global_shortcut().register(shortcut.as_str()) {
        let _ = app.global_shortcut().register(previous.as_str());
        return Err(format!("shortcut is unavailable: {error}"));
    }
    if let Err(error) = save_config(&shortcut) {
        let _ = app.global_shortcut().unregister(shortcut.as_str());
        let _ = app.global_shortcut().register(previous.as_str());
        return Err(error);
    }
    settings.replace(shortcut)
}

pub fn register_initial(app: &AppHandle, settings: &ShortcutSettings) -> Result<String, String> {
    let requested = settings.current();
    if app.global_shortcut().register(requested.as_str()).is_ok() {
        return Ok(requested);
    }
    if requested == DEFAULT_SHORTCUT {
        return Err(format!("global shortcut {DEFAULT_SHORTCUT} is unavailable"));
    }
    app.global_shortcut()
        .register(DEFAULT_SHORTCUT)
        .map_err(|error| format!("configured and default shortcuts are unavailable: {error}"))?;
    settings.replace(DEFAULT_SHORTCUT.into())?;
    save_config(DEFAULT_SHORTCUT)?;
    Ok(DEFAULT_SHORTCUT.into())
}

fn is_allowed(value: &str) -> bool {
    ALLOWED_SHORTCUTS.contains(&value)
}

fn config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "Arclume", "Arclume")
        .map(|project| project.config_dir().join("shortcut.json"))
}

fn load_config() -> Option<String> {
    let content = std::fs::read_to_string(config_path()?).ok()?;
    serde_json::from_str::<ShortcutConfig>(&content)
        .ok()
        .map(|config| config.global_shortcut)
}

fn save_config(value: &str) -> Result<(), String> {
    let path = config_path().ok_or("OS configuration directory is unavailable")?;
    let parent = path.parent().ok_or("invalid shortcut configuration path")?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create configuration directory: {error}"))?;
    let payload = serde_json::to_vec_pretty(&ShortcutConfig {
        global_shortcut: value.into(),
    })
    .map_err(|error| format!("failed to encode shortcut settings: {error}"))?;
    std::fs::write(path, payload)
        .map_err(|error| format!("failed to save shortcut settings: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_documented_shortcuts() {
        for shortcut in ALLOWED_SHORTCUTS {
            assert!(is_allowed(shortcut));
        }
        assert!(!is_allowed("Ctrl+Alt+Delete"));
        assert!(!is_allowed("A"));
    }

    #[test]
    fn config_contract_rejects_unknown_fields() {
        assert!(
            serde_json::from_str::<ShortcutConfig>(
                r#"{"globalShortcut":"Alt+Space","unexpected":true}"#
            )
            .is_err()
        );
    }
}
