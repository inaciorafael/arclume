use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rusqlite::{Connection, OptionalExtension, params};

use super::scanner::ScanEntry;

#[derive(Debug, PartialEq, Eq)]
pub struct ReconcileStats {
    pub indexed: usize,
    pub removed: usize,
    pub stale_removal_skipped: bool,
}

#[derive(Clone, Debug)]
pub struct IndexedItem {
    pub id: i64,
    pub title: String,
    pub subtitle: String,
    pub kind: String,
}

pub fn open(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    connection.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         CREATE TABLE IF NOT EXISTS file_items (
           id INTEGER PRIMARY KEY,
           path TEXT NOT NULL UNIQUE,
           title TEXT NOT NULL,
           parent TEXT NOT NULL,
           kind TEXT NOT NULL,
           modified INTEGER NOT NULL,
           size INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS file_items_modified ON file_items(modified DESC);
         CREATE VIRTUAL TABLE IF NOT EXISTS file_items_fts USING fts5(
           title, parent, content='file_items', content_rowid='id',
           tokenize='unicode61 remove_diacritics 2', prefix='2 3 4'
         );
         CREATE TRIGGER IF NOT EXISTS file_items_ai AFTER INSERT ON file_items BEGIN
           INSERT INTO file_items_fts(rowid, title, parent) VALUES (new.id, new.title, new.parent);
         END;
         CREATE TRIGGER IF NOT EXISTS file_items_ad AFTER DELETE ON file_items BEGIN
           INSERT INTO file_items_fts(file_items_fts, rowid, title, parent) VALUES('delete', old.id, old.title, old.parent);
         END;
         CREATE TRIGGER IF NOT EXISTS file_items_au AFTER UPDATE ON file_items BEGIN
           INSERT INTO file_items_fts(file_items_fts, rowid, title, parent) VALUES('delete', old.id, old.title, old.parent);
           INSERT INTO file_items_fts(rowid, title, parent) VALUES (new.id, new.title, new.parent);
         END;"
    ).map_err(|error| format!("failed to initialize file index: {error}"))?;
    Ok(connection)
}

pub fn upsert(connection: &Connection, path: &Path) -> rusqlite::Result<()> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
    let Some(title) = path.file_name().and_then(|value| value.to_str()) else {
        return Ok(());
    };
    let parent = path
        .parent()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |value| value.as_secs() as i64);
    let kind = if metadata.is_dir() { "folder" } else { "file" };
    connection.execute(
        "INSERT INTO file_items(path,title,parent,kind,modified,size) VALUES(?1,?2,?3,?4,?5,?6)
         ON CONFLICT(path) DO UPDATE SET title=excluded.title,parent=excluded.parent,kind=excluded.kind,modified=excluded.modified,size=excluded.size
         WHERE file_items.title<>excluded.title
            OR file_items.parent<>excluded.parent
            OR file_items.kind<>excluded.kind
            OR file_items.modified<>excluded.modified
            OR file_items.size<>excluded.size",
        params![path.to_string_lossy(), title, parent, kind, modified, metadata.len() as i64],
    )?;
    Ok(())
}

fn upsert_scanned(connection: &Connection, entry: &ScanEntry) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT INTO file_items(path,title,parent,kind,modified,size) VALUES(?1,?2,?3,?4,?5,?6)
         ON CONFLICT(path) DO UPDATE SET title=excluded.title,parent=excluded.parent,kind=excluded.kind,modified=excluded.modified,size=excluded.size
         WHERE file_items.title<>excluded.title
            OR file_items.parent<>excluded.parent
            OR file_items.kind<>excluded.kind
            OR file_items.modified<>excluded.modified
            OR file_items.size<>excluded.size",
        params![
            entry.path.to_string_lossy(),
            entry.title,
            entry.parent,
            entry.kind,
            entry.modified,
            entry.size
        ],
    )?;
    Ok(())
}

pub fn remove(connection: &Connection, path: &Path) -> rusqlite::Result<()> {
    let value = path.to_string_lossy();
    connection.execute(
        "DELETE FROM file_items WHERE path=?1 OR path LIKE ?2",
        params![value, format!("{}{}%", value, std::path::MAIN_SEPARATOR)],
    )?;
    Ok(())
}

pub fn reconcile(
    connection: &mut Connection,
    entries: &[ScanEntry],
    remove_stale: bool,
) -> rusqlite::Result<ReconcileStats> {
    let transaction = connection.transaction()?;
    transaction.execute_batch(
        "CREATE TEMP TABLE IF NOT EXISTS reconcile_paths(path TEXT PRIMARY KEY);
         DELETE FROM reconcile_paths;",
    )?;
    let mut indexed = 0usize;
    {
        let mut remember =
            transaction.prepare_cached("INSERT OR IGNORE INTO reconcile_paths(path) VALUES(?1)")?;
        for entry in entries {
            upsert_scanned(&transaction, entry)?;
            remember.execute([entry.path.to_string_lossy().as_ref()])?;
            indexed += 1;
        }
    }
    let removed = if remove_stale {
        transaction.execute(
            "DELETE FROM file_items WHERE path NOT IN (SELECT path FROM reconcile_paths)",
            [],
        )?
    } else {
        0
    };
    transaction.commit()?;
    Ok(ReconcileStats {
        indexed,
        removed,
        stale_removal_skipped: !remove_stale,
    })
}

pub fn search(
    connection: &Connection,
    query: &str,
    limit: usize,
) -> rusqlite::Result<Vec<IndexedItem>> {
    if query.trim().is_empty() {
        return query_rows(
            connection,
            "SELECT id,title,parent,kind FROM file_items ORDER BY modified DESC LIMIT ?1",
            params![limit as i64],
        );
    }
    let fts_query = query
        .split_whitespace()
        .map(|token| format!("\"{}\"*", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ");
    let mut statement = connection.prepare(
        "SELECT f.id,f.title,f.parent,f.kind FROM file_items_fts
         JOIN file_items f ON f.id=file_items_fts.rowid
         WHERE file_items_fts MATCH ?1 ORDER BY rank LIMIT ?2",
    )?;
    let rows = statement.query_map(params![fts_query, limit as i64], map_item)?;
    rows.collect()
}

fn query_rows<P: rusqlite::Params>(
    connection: &Connection,
    sql: &str,
    params: P,
) -> rusqlite::Result<Vec<IndexedItem>> {
    let mut statement = connection.prepare(sql)?;
    statement.query_map(params, map_item)?.collect()
}

fn map_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<IndexedItem> {
    Ok(IndexedItem {
        id: row.get(0)?,
        title: row.get(1)?,
        subtitle: row.get(2)?,
        kind: row.get(3)?,
    })
}

pub fn path_for_id(connection: &Connection, id: i64) -> Result<PathBuf, String> {
    connection
        .query_row("SELECT path FROM file_items WHERE id=?1", [id], |row| {
            row.get::<_, String>(0)
        })
        .optional()
        .map_err(|error| error.to_string())?
        .map(PathBuf::from)
        .ok_or_else(|| "file was not found".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_fts_schema() {
        let connection = open(Path::new(":memory:")).unwrap();
        let exists: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='file_items_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(exists, 1);
    }

    #[test]
    fn recent_items_use_the_modified_index() {
        let connection = open(Path::new(":memory:")).unwrap();
        let plan: String = connection
            .query_row(
                "EXPLAIN QUERY PLAN SELECT id,title,parent,kind FROM file_items ORDER BY modified DESC LIMIT 40",
                [],
                |row| row.get(3),
            )
            .unwrap();
        assert!(
            plan.contains("file_items_modified"),
            "unexpected query plan: {plan}"
        );
    }

    #[test]
    fn complete_reconciliation_removes_stale_items() {
        let mut connection = open(Path::new(":memory:")).unwrap();
        let directory = unique_test_directory("complete");
        std::fs::create_dir_all(&directory).unwrap();
        let file = directory.join("document.txt");
        std::fs::write(&file, "test").unwrap();
        let entry = scan_entry(&file);
        let first = reconcile(&mut connection, std::slice::from_ref(&entry), true).unwrap();
        assert_eq!(first.indexed, 1);
        let second = reconcile(&mut connection, &[], true).unwrap();
        assert_eq!(second.removed, 1);
        assert_eq!(item_count(&connection), 0);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn incomplete_reconciliation_preserves_unobserved_items() {
        let mut connection = open(Path::new(":memory:")).unwrap();
        let directory = unique_test_directory("incomplete");
        std::fs::create_dir_all(&directory).unwrap();
        let file = directory.join("document.txt");
        std::fs::write(&file, "test").unwrap();
        let entry = scan_entry(&file);
        reconcile(&mut connection, std::slice::from_ref(&entry), true).unwrap();
        let stats = reconcile(&mut connection, &[], false).unwrap();
        assert!(stats.stale_removal_skipped);
        assert_eq!(item_count(&connection), 1);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn unchanged_upsert_does_not_rewrite_fts_rows() {
        let connection = open(Path::new(":memory:")).unwrap();
        let directory = unique_test_directory("unchanged");
        std::fs::create_dir_all(&directory).unwrap();
        let file = directory.join("document.txt");
        std::fs::write(&file, "test").unwrap();
        upsert(&connection, &file).unwrap();
        let changes_before = connection.total_changes();
        upsert(&connection, &file).unwrap();
        assert_eq!(connection.total_changes(), changes_before);
        std::fs::remove_dir_all(directory).unwrap();
    }

    fn item_count(connection: &Connection) -> i64 {
        connection
            .query_row("SELECT count(*) FROM file_items", [], |row| row.get(0))
            .unwrap()
    }

    fn unique_test_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "arclume-index-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn scan_entry(path: &Path) -> ScanEntry {
        let metadata = std::fs::metadata(path).unwrap();
        ScanEntry {
            path: path.to_path_buf(),
            title: path.file_name().unwrap().to_string_lossy().into_owned(),
            parent: path.parent().unwrap().to_string_lossy().into_owned(),
            kind: if metadata.is_dir() { "folder" } else { "file" },
            modified: metadata
                .modified()
                .unwrap()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            size: metadata.len() as i64,
        }
    }
}
