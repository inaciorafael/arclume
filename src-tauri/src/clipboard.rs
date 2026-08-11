use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use image::{ColorType, ImageEncoder, codecs::png::PngEncoder};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, image::Image};
use tauri_plugin_clipboard_manager::ClipboardExt;

const POLL_INTERVAL: Duration = Duration::from_millis(750);
const MAX_RAW_IMAGE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardSettings {
    pub enabled: bool,
    pub max_items: u32,
    pub max_total_bytes: u64,
    pub retention_days: u32,
}

impl Default for ClipboardSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_items: 100,
            max_total_bytes: 50 * 1024 * 1024,
            retention_days: 7,
        }
    }
}

impl ClipboardSettings {
    fn validate(&self) -> Result<(), String> {
        if !(10..=500).contains(&self.max_items) {
            return Err("clipboard maxItems must be between 10 and 500".into());
        }
        if !(5 * 1024 * 1024..=500 * 1024 * 1024).contains(&self.max_total_bytes) {
            return Err("clipboard maxTotalBytes must be between 5 MB and 500 MB".into());
        }
        if !(1..=90).contains(&self.retention_days) {
            return Err("clipboard retentionDays must be between 1 and 90".into());
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    pub id: i64,
    pub kind: String,
    pub preview: String,
    pub byte_size: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub captured_at: i64,
}

pub struct ClipboardHistory {
    connection: Mutex<Connection>,
    settings: RwLock<ClipboardSettings>,
    settings_path: PathBuf,
}

impl ClipboardHistory {
    pub fn open(database_path: &Path) -> Result<Arc<Self>, String> {
        let directory = database_path
            .parent()
            .ok_or("clipboard database directory is unavailable")?;
        fs::create_dir_all(directory).map_err(|error| error.to_string())?;
        let settings_path = directory.join("clipboard-settings.json");
        let settings = load_settings(&settings_path);
        let connection = Connection::open(directory.join("clipboard-history.sqlite3"))
            .map_err(|error| error.to_string())?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA auto_vacuum=INCREMENTAL;
             CREATE TABLE IF NOT EXISTS clipboard_items (
               id INTEGER PRIMARY KEY,
               content_hash TEXT NOT NULL UNIQUE,
               kind TEXT NOT NULL CHECK(kind IN ('text', 'image')),
               preview TEXT NOT NULL,
               text_content TEXT,
               image_png BLOB,
               byte_size INTEGER NOT NULL,
               width INTEGER,
               height INTEGER,
               captured_at INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS clipboard_items_captured_at ON clipboard_items(captured_at DESC);"
        ).map_err(|error| error.to_string())?;
        let history = Arc::new(Self {
            connection: Mutex::new(connection),
            settings: RwLock::new(settings),
            settings_path,
        });
        history.prune()?;
        Ok(history)
    }

    pub fn start(self: &Arc<Self>, app: AppHandle) {
        let history = Arc::clone(self);
        thread::spawn(move || {
            let mut last_hash = String::new();
            loop {
                thread::sleep(POLL_INTERVAL);
                if !history.settings().enabled {
                    last_hash.clear();
                    continue;
                }
                if let Ok(text) = app.clipboard().read_text()
                    && !text.is_empty()
                {
                    let hash = content_hash(b"text", text.as_bytes());
                    if hash != last_hash && history.record_text(&hash, &text).is_ok() {
                        last_hash = hash;
                    }
                    continue;
                }
                if let Ok(image) = app.clipboard().read_image() {
                    let hash = content_hash(b"image", image.rgba());
                    if hash != last_hash && history.record_image(&hash, &image).is_ok() {
                        last_hash = hash;
                    }
                }
            }
        });
    }

    pub fn settings(&self) -> ClipboardSettings {
        self.settings
            .read()
            .expect("clipboard settings poisoned")
            .clone()
    }

    pub fn update_settings(
        &self,
        settings: ClipboardSettings,
    ) -> Result<ClipboardSettings, String> {
        settings.validate()?;
        let serialized = serde_json::to_vec_pretty(&settings).map_err(|error| error.to_string())?;
        let temporary = self.settings_path.with_extension("json.tmp");
        fs::write(&temporary, serialized).map_err(|error| error.to_string())?;
        fs::rename(&temporary, &self.settings_path).map_err(|error| error.to_string())?;
        *self
            .settings
            .write()
            .map_err(|_| "clipboard settings are unavailable")? = settings.clone();
        self.prune()?;
        Ok(settings)
    }

    pub fn list(&self, limit: u32) -> Result<Vec<ClipboardItem>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "clipboard history is unavailable")?;
        let mut statement = connection.prepare(
            "SELECT id, kind, preview, byte_size, width, height, captured_at FROM clipboard_items ORDER BY captured_at DESC LIMIT ?1"
        ).map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([limit.clamp(1, 200)], |row| {
                Ok(ClipboardItem {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    preview: row.get(2)?,
                    byte_size: row.get::<_, i64>(3)? as u64,
                    width: row.get(4)?,
                    height: row.get(5)?,
                    captured_at: row.get(6)?,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn image_data_url(&self, id: i64) -> Result<String, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "clipboard history is unavailable")?;
        let png: Option<Vec<u8>> = connection
            .query_row(
                "SELECT image_png FROM clipboard_items WHERE id=?1 AND kind='image'",
                [id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .flatten();
        png.map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
            .ok_or_else(|| "clipboard image was not found".into())
    }

    pub fn restore(&self, app: &AppHandle, id: i64) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "clipboard history is unavailable")?;
        let item: Option<(String, Option<String>, Option<Vec<u8>>)> = connection
            .query_row(
                "SELECT kind, text_content, image_png FROM clipboard_items WHERE id=?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        drop(connection);
        match item {
            Some((kind, Some(text), _)) if kind == "text" => app
                .clipboard()
                .write_text(text)
                .map_err(|error| error.to_string()),
            Some((kind, _, Some(png))) if kind == "image" => {
                let decoded = image::load_from_memory(&png)
                    .map_err(|error| error.to_string())?
                    .into_rgba8();
                let (width, height) = decoded.dimensions();
                app.clipboard()
                    .write_image(&Image::new_owned(decoded.into_raw(), width, height))
                    .map_err(|error| error.to_string())
            }
            _ => Err("clipboard item was not found".into()),
        }
    }

    pub fn clear(&self) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "clipboard history is unavailable")?;
        connection
            .execute("DELETE FROM clipboard_items", [])
            .map_err(|error| error.to_string())?;
        connection
            .execute_batch("PRAGMA incremental_vacuum;")
            .map_err(|error| error.to_string())
    }

    fn record_text(&self, hash: &str, text: &str) -> Result<(), String> {
        let settings = self.settings();
        if text.len() as u64 > settings.max_total_bytes {
            return Ok(());
        }
        let preview: String = text.chars().take(180).collect();
        self.upsert(hash, "text", &preview, Some(text.as_bytes()), None, None)
    }

    fn record_image(&self, hash: &str, source: &Image<'_>) -> Result<(), String> {
        if source.rgba().len() > MAX_RAW_IMAGE_BYTES {
            return Ok(());
        }
        let mut png = Vec::new();
        PngEncoder::new(Cursor::new(&mut png))
            .write_image(
                source.rgba(),
                source.width(),
                source.height(),
                ColorType::Rgba8.into(),
            )
            .map_err(|error| error.to_string())?;
        if png.len() as u64 > self.settings().max_total_bytes {
            return Ok(());
        }
        self.upsert(
            hash,
            "image",
            &format!("Image · {} × {}", source.width(), source.height()),
            Some(&png),
            Some(source.width()),
            Some(source.height()),
        )
    }

    fn upsert(
        &self,
        hash: &str,
        kind: &str,
        preview: &str,
        content: Option<&[u8]>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<(), String> {
        let (text, image): (Option<String>, Option<&[u8]>) = if kind == "text" {
            (
                content.map(|bytes| String::from_utf8_lossy(bytes).into_owned()),
                None,
            )
        } else {
            (None, content)
        };
        let size = content.map_or(0, |bytes| bytes.len()) as i64;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "clipboard history is unavailable")?;
        connection.execute(
            "INSERT INTO clipboard_items(content_hash,kind,preview,text_content,image_png,byte_size,width,height,captured_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)
             ON CONFLICT(content_hash) DO UPDATE SET captured_at=excluded.captured_at",
            params![hash, kind, preview, text, image, size, width, height, now_seconds()],
        ).map_err(|error| error.to_string())?;
        drop(connection);
        self.prune()
    }

    fn prune(&self) -> Result<(), String> {
        let settings = self.settings();
        let cutoff = now_seconds() - i64::from(settings.retention_days) * 86_400;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "clipboard history is unavailable")?;
        connection
            .execute(
                "DELETE FROM clipboard_items WHERE captured_at < ?1",
                [cutoff],
            )
            .map_err(|error| error.to_string())?;
        connection.execute("DELETE FROM clipboard_items WHERE id NOT IN (SELECT id FROM clipboard_items ORDER BY captured_at DESC LIMIT ?1)", [settings.max_items]).map_err(|error| error.to_string())?;
        loop {
            let total: i64 = connection
                .query_row(
                    "SELECT COALESCE(SUM(byte_size),0) FROM clipboard_items",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            if total <= settings.max_total_bytes as i64 {
                break;
            }
            if connection.execute("DELETE FROM clipboard_items WHERE id=(SELECT id FROM clipboard_items ORDER BY captured_at ASC LIMIT 1)", []).map_err(|error| error.to_string())? == 0 { break; }
        }
        connection
            .execute_batch("PRAGMA incremental_vacuum(32);")
            .map_err(|error| error.to_string())
    }
}

fn load_settings(path: &Path) -> ClipboardSettings {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<ClipboardSettings>(&bytes).ok())
        .filter(|settings| settings.validate().is_ok())
        .unwrap_or_default()
}

fn content_hash(kind: &[u8], content: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(kind);
    digest.update(content);
    format!("{:x}", digest.finalize())
}

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[tauri::command]
pub fn get_clipboard_settings(
    history: tauri::State<'_, Arc<ClipboardHistory>>,
) -> ClipboardSettings {
    history.settings()
}

#[tauri::command]
pub fn set_clipboard_settings(
    settings: ClipboardSettings,
    history: tauri::State<'_, Arc<ClipboardHistory>>,
) -> Result<ClipboardSettings, String> {
    history.update_settings(settings)
}

#[tauri::command]
pub fn list_clipboard_items(
    limit: u32,
    history: tauri::State<'_, Arc<ClipboardHistory>>,
) -> Result<Vec<ClipboardItem>, String> {
    history.list(limit)
}

#[tauri::command]
pub fn clipboard_image(
    id: i64,
    history: tauri::State<'_, Arc<ClipboardHistory>>,
) -> Result<String, String> {
    history.image_data_url(id)
}

#[tauri::command]
pub fn restore_clipboard_item(
    id: i64,
    app: AppHandle,
    history: tauri::State<'_, Arc<ClipboardHistory>>,
) -> Result<(), String> {
    history.restore(&app, id)
}

#[tauri::command]
pub fn clear_clipboard_history(
    history: tauri::State<'_, Arc<ClipboardHistory>>,
) -> Result<(), String> {
    history.clear()
}
