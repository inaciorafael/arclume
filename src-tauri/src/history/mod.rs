mod ranking;
mod store;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

pub struct HistoryStore {
    connection: Mutex<Connection>,
}

#[derive(Clone, Debug)]
pub struct RecentItem {
    pub id: String,
    pub title: String,
    pub kind: String,
}

impl HistoryStore {
    pub fn open(database_path: &Path) -> Result<Self, String> {
        let connection = store::open(database_path)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn record(
        &self,
        query: &str,
        item_id: &str,
        title: &str,
        kind: &str,
    ) -> Result<(), String> {
        validate_field(query, 512, "query")?;
        validate_field(item_id, 256, "item id")?;
        validate_field(title, 512, "title")?;
        validate_field(kind, 32, "kind")?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "history is unavailable")?;
        store::record(&connection, query, item_id, title, kind).map_err(|error| error.to_string())
    }

    pub fn boosts(&self, query: &str) -> HashMap<String, i64> {
        let Ok(connection) = self.connection.lock() else {
            return HashMap::new();
        };
        store::signals(&connection, query)
            .unwrap_or_default()
            .into_iter()
            .map(|signal| {
                let boost = ranking::adaptive_boost(signal.use_count, signal.last_used);
                (signal.item_id, boost)
            })
            .collect()
    }

    pub fn clear(&self) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "history is unavailable")?;
        store::clear(&connection).map_err(|error| error.to_string())
    }

    pub fn recent_items(&self) -> Vec<RecentItem> {
        let Ok(connection) = self.connection.lock() else {
            return Vec::new();
        };
        store::recent_items(&connection).unwrap_or_default()
    }
}

fn validate_field(value: &str, maximum: usize, name: &str) -> Result<(), String> {
    if value.len() > maximum {
        Err(format!("{name} is too long"))
    } else {
        Ok(())
    }
}
