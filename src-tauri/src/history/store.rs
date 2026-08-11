use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::RecentItem;
use rusqlite::{Connection, params};

pub struct UsageSignal {
    pub item_id: String,
    pub use_count: i64,
    pub last_used: i64,
}

pub fn open(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    connection
        .execute_batch(
            "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         CREATE TABLE IF NOT EXISTS usage_history (
           item_id TEXT NOT NULL,
           normalized_query TEXT NOT NULL,
           title TEXT NOT NULL,
           kind TEXT NOT NULL,
           use_count INTEGER NOT NULL DEFAULT 1,
           last_used INTEGER NOT NULL,
           PRIMARY KEY(item_id, normalized_query)
         );
         CREATE INDEX IF NOT EXISTS usage_history_recent ON usage_history(last_used DESC);
         CREATE TABLE IF NOT EXISTS recent_queries (
           normalized_query TEXT PRIMARY KEY,
           query TEXT NOT NULL,
           use_count INTEGER NOT NULL DEFAULT 1,
           last_used INTEGER NOT NULL
         );",
        )
        .map_err(|error| format!("failed to initialize history: {error}"))?;
    Ok(connection)
}

pub fn record(
    connection: &Connection,
    query: &str,
    item_id: &str,
    title: &str,
    kind: &str,
) -> rusqlite::Result<()> {
    let normalized = normalize(query);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_secs() as i64);
    connection.execute(
        "INSERT INTO usage_history(item_id,normalized_query,title,kind,use_count,last_used) VALUES(?1,?2,?3,?4,1,?5)
         ON CONFLICT(item_id,normalized_query) DO UPDATE SET title=excluded.title,kind=excluded.kind,use_count=usage_history.use_count+1,last_used=excluded.last_used",
        params![item_id, normalized, title, kind, now],
    )?;
    if !query.trim().is_empty() {
        connection.execute(
            "INSERT INTO recent_queries(normalized_query,query,use_count,last_used) VALUES(?1,?2,1,?3)
             ON CONFLICT(normalized_query) DO UPDATE SET query=excluded.query,use_count=recent_queries.use_count+1,last_used=excluded.last_used",
            params![normalized, query.trim(), now],
        )?;
    }
    Ok(())
}

pub fn signals(connection: &Connection, query: &str) -> rusqlite::Result<Vec<UsageSignal>> {
    let normalized = normalize(query);
    if normalized.is_empty() {
        let mut statement = connection.prepare("SELECT item_id,SUM(use_count),MAX(last_used) FROM usage_history GROUP BY item_id ORDER BY MAX(last_used) DESC LIMIT 200")?;
        return statement.query_map([], map_signal)?.collect();
    }
    let mut statement = connection.prepare("SELECT item_id,use_count,last_used FROM usage_history WHERE normalized_query=?1 ORDER BY last_used DESC LIMIT 200")?;
    statement.query_map([normalized], map_signal)?.collect()
}

fn map_signal(row: &rusqlite::Row<'_>) -> rusqlite::Result<UsageSignal> {
    Ok(UsageSignal {
        item_id: row.get(0)?,
        use_count: row.get(1)?,
        last_used: row.get(2)?,
    })
}

pub fn clear(connection: &Connection) -> rusqlite::Result<()> {
    connection
        .execute_batch("BEGIN; DELETE FROM usage_history; DELETE FROM recent_queries; COMMIT;")
}

pub fn recent_items(connection: &Connection) -> rusqlite::Result<Vec<RecentItem>> {
    let mut statement = connection.prepare(
        "SELECT item_id,MAX(title),MAX(kind) FROM usage_history GROUP BY item_id ORDER BY MAX(last_used) DESC LIMIT 40"
    )?;
    statement
        .query_map([], |row| {
            Ok(RecentItem {
                id: row.get(0)?,
                title: row.get(1)?,
                kind: row.get(2)?,
            })
        })?
        .collect()
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .flat_map(str::chars)
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn records_increments_and_clears_history() {
        let connection = open(Path::new(":memory:")).unwrap();
        record(
            &connection,
            "vsc",
            "app:1",
            "Visual Studio Code",
            "application",
        )
        .unwrap();
        record(
            &connection,
            "vsc",
            "app:1",
            "Visual Studio Code",
            "application",
        )
        .unwrap();
        assert_eq!(signals(&connection, "vsc").unwrap()[0].use_count, 2);
        clear(&connection).unwrap();
        assert!(signals(&connection, "vsc").unwrap().is_empty());
    }
}
