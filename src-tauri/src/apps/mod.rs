mod model;
mod provider;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, RwLock};

pub use model::{AppEntry, LaunchSpec};

pub struct AppCatalog {
    entries: RwLock<Vec<AppEntry>>,
    icon_cache: Mutex<HashMap<String, Option<String>>>,
}

impl AppCatalog {
    pub fn discover() -> Self {
        #[cfg(target_os = "windows")]
        let discovered = windows::discover();
        #[cfg(target_os = "macos")]
        let discovered = macos::discover();
        #[cfg(target_os = "linux")]
        let discovered = linux::discover();
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let discovered = Vec::new();

        let mut seen = HashSet::new();
        let mut entries: Vec<_> = discovered
            .into_iter()
            .filter(|entry| seen.insert(entry.id.clone()))
            .collect();
        entries.sort_by(|left, right| left.title.cmp(&right.title));
        Self {
            entries: RwLock::new(entries),
            icon_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.read().map_or(0, |entries| entries.len())
    }

    pub fn snapshot(&self) -> Vec<AppEntry> {
        self.entries
            .read()
            .map_or_else(|_| Vec::new(), |entries| entries.clone())
    }

    pub fn launch(&self, id: &str) -> Result<(), String> {
        let entries = self
            .entries
            .read()
            .map_err(|_| "application catalog is unavailable")?;
        let entry = entries
            .iter()
            .find(|entry| entry.id == id)
            .ok_or("application was not found")?;
        entry.launch()
    }

    pub fn icon(&self, id: &str) -> Option<String> {
        if let Ok(cache) = self.icon_cache.lock()
            && let Some(icon) = cache.get(id)
        {
            return icon.clone();
        }
        let icon = self
            .entries
            .read()
            .ok()?
            .iter()
            .find(|entry| entry.id == id)
            .and_then(load_icon);
        if let Ok(mut cache) = self.icon_cache.lock() {
            cache.insert(id.into(), icon.clone());
        }
        icon
    }
}

#[cfg(target_os = "windows")]
fn load_icon(entry: &AppEntry) -> Option<String> {
    let LaunchSpec::WindowsShortcut(path) = &entry.launch;
    windows::load_icon(path)
}

#[cfg(not(target_os = "windows"))]
fn load_icon(_entry: &AppEntry) -> Option<String> {
    None
}
