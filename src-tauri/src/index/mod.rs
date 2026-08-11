mod database;
mod scanner;
mod settings;
mod watcher;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use rusqlite::Connection;

pub use database::IndexedItem;
pub use settings::IndexSettings;

pub struct FileIndex {
    connection: Mutex<Connection>,
    settings: RwLock<IndexSettings>,
    reconciliation: Mutex<()>,
}

impl FileIndex {
    pub fn open() -> Result<Arc<Self>, String> {
        let settings = IndexSettings::load()?;
        let database_path = settings.database_path();
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create data directory: {error}"))?;
        }
        let connection = database::open(&database_path)?;
        Ok(Arc::new(Self {
            connection: Mutex::new(connection),
            settings: RwLock::new(settings),
            reconciliation: Mutex::new(()),
        }))
    }

    pub fn start(self: &Arc<Self>) {
        let index = Arc::clone(self);
        std::thread::Builder::new()
            .name("arclume-indexer".into())
            .spawn(move || {
                watcher::run(index);
            })
            .expect("failed to start file indexer");
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<IndexedItem> {
        let Ok(connection) = self.connection.lock() else {
            return Vec::new();
        };
        database::search(&connection, query, limit).unwrap_or_else(|error| {
            eprintln!("file search failed: {error}");
            Vec::new()
        })
    }

    pub fn upsert_path(&self, path: &Path) {
        if !self
            .settings
            .read()
            .is_ok_and(|settings| settings.should_index(path))
        {
            return;
        }
        let Ok(connection) = self.connection.lock() else {
            return;
        };
        if let Err(error) = database::upsert(&connection, path) {
            eprintln!("failed to index {}: {error}", path.display());
        }
    }

    pub fn reconcile(&self, reason: &str) {
        let Ok(_reconciliation) = self.reconciliation.lock() else {
            return;
        };
        let started_at = std::time::Instant::now();
        let snapshot = scanner::collect(self);
        let scan_elapsed = started_at.elapsed();
        let complete = snapshot.is_complete();
        let observed = snapshot.entries.len();
        let directory_errors = snapshot.directory_errors;
        let mut connection = match database::open(&self.database_path()) {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("index reconciliation skipped: {error}");
                return;
            }
        };
        let apply_started_at = std::time::Instant::now();
        match database::reconcile(&mut connection, &snapshot.entries, complete) {
            Ok(stats) => eprintln!(
                "index reconciliation reason={reason} observed={observed} indexed={} removed={} directory_errors={directory_errors} stale_removal_skipped={} scan_ms={} apply_ms={} elapsed_ms={}",
                stats.indexed,
                stats.removed,
                stats.stale_removal_skipped,
                scan_elapsed.as_millis(),
                apply_started_at.elapsed().as_millis(),
                started_at.elapsed().as_millis()
            ),
            Err(error) => eprintln!("index reconciliation failed reason={reason}: {error}"),
        }
    }

    pub fn remove_path(&self, path: &Path) {
        let Ok(connection) = self.connection.lock() else {
            return;
        };
        if let Err(error) = database::remove(&connection, path) {
            eprintln!("failed to remove {}: {error}", path.display());
        }
    }

    pub fn roots(&self) -> Vec<PathBuf> {
        self.settings
            .read()
            .map_or_else(|_| Vec::new(), |settings| settings.roots.clone())
    }

    pub fn database_path(&self) -> PathBuf {
        self.settings
            .read()
            .map_or_else(|_| PathBuf::new(), |settings| settings.database_path())
    }

    pub fn classify_for_scan(
        &self,
        path: &Path,
    ) -> std::io::Result<Option<settings::PathClassification>> {
        self.settings
            .read()
            .map_err(|_| std::io::Error::other("index settings are unavailable"))?
            .classify(path)
    }

    pub fn open_item(&self, id: i64) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "file index is unavailable")?;
        let path = database::path_for_id(&connection, id)?;
        open_path(&path)
    }

    pub fn add_root(self: &Arc<Self>, root: PathBuf) -> Result<Vec<PathBuf>, String> {
        self.settings
            .write()
            .map_err(|_| "index settings are unavailable")?
            .add_root(root)?;
        self.reconcile_in_background("root-added");
        Ok(self.roots())
    }

    pub fn remove_root(self: &Arc<Self>, root: &Path) -> Result<Vec<PathBuf>, String> {
        self.settings
            .write()
            .map_err(|_| "index settings are unavailable")?
            .remove_root(root)?;
        self.reconcile_in_background("root-removed");
        Ok(self.roots())
    }

    fn reconcile_in_background(self: &Arc<Self>, reason: &'static str) {
        let index = Arc::clone(self);
        let _ = std::thread::Builder::new()
            .name(format!("arclume-{reason}"))
            .spawn(move || index.reconcile(reason));
    }
}

#[cfg(target_os = "windows")]
fn open_path(path: &Path) -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn open_path(path: &Path) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "linux")]
fn open_path(path: &Path) -> Result<(), String> {
    std::process::Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
