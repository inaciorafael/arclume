use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use directories::ProjectDirs;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IndexDiagnostics {
    captured_at_unix: u64,
    platform: &'static str,
    architecture: &'static str,
    open_micros: u128,
    query_micros: u128,
    indexed_items: i64,
    database_bytes: u64,
    wal_bytes: u64,
    page_count: i64,
    page_size: i64,
    freelist_pages: i64,
}

fn main() -> Result<(), String> {
    let project =
        ProjectDirs::from("com", "Arclume", "Arclume").ok_or("OS data directory is unavailable")?;
    let database_path = project.data_local_dir().join("search-index.sqlite3");
    let open_started_at = Instant::now();
    let connection =
        Connection::open_with_flags(&database_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|error| format!("cannot open index: {error}"))?;
    let open_micros = open_started_at.elapsed().as_micros();
    let query_started_at = Instant::now();
    let indexed_items = pragma_or_query(&connection, "SELECT count(*) FROM file_items")?;
    let page_count = pragma_or_query(&connection, "PRAGMA page_count")?;
    let page_size = pragma_or_query(&connection, "PRAGMA page_size")?;
    let freelist_pages = pragma_or_query(&connection, "PRAGMA freelist_count")?;
    let query_micros = query_started_at.elapsed().as_micros();
    let diagnostics = IndexDiagnostics {
        captured_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_secs(),
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        open_micros,
        query_micros,
        indexed_items,
        database_bytes: file_size(&database_path),
        wal_bytes: file_size(&database_path.with_extension("sqlite3-wal")),
        page_count,
        page_size,
        freelist_pages,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&diagnostics).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn pragma_or_query(connection: &Connection, sql: &str) -> Result<i64, String> {
    connection
        .query_row(sql, [], |row| row.get(0))
        .map_err(|error| error.to_string())
}

fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map_or(0, |metadata| metadata.len())
}
