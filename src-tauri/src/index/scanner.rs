use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use super::FileIndex;

pub struct ScanSnapshot {
    pub entries: Vec<ScanEntry>,
    pub directory_errors: usize,
}

pub struct ScanEntry {
    pub path: PathBuf,
    pub title: String,
    pub parent: String,
    pub kind: &'static str,
    pub modified: i64,
    pub size: i64,
}

impl ScanEntry {
    fn from_classification(
        path: PathBuf,
        classification: &super::settings::PathClassification,
    ) -> Option<Self> {
        let title = path.file_name()?.to_str()?.to_owned();
        let parent = path
            .parent()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        let modified = classification
            .metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |value| value.as_secs() as i64);
        Some(Self {
            path,
            title,
            parent,
            kind: if classification.is_directory {
                "folder"
            } else {
                "file"
            },
            modified,
            size: classification.metadata.len() as i64,
        })
    }
}

impl ScanSnapshot {
    pub fn is_complete(&self) -> bool {
        self.directory_errors == 0
    }
}

pub fn collect(index: &FileIndex) -> ScanSnapshot {
    let mut entries = Vec::new();
    let mut directory_errors = 0usize;
    let mut pending: Vec<PathBuf> = index.roots();
    while let Some(path) = pending.pop() {
        let classification = match index.classify_for_scan(&path) {
            Ok(Some(classification)) => classification,
            Ok(None) => continue,
            Err(_) => {
                directory_errors += 1;
                continue;
            }
        };
        if classification.should_index
            && let Some(entry) = ScanEntry::from_classification(path.clone(), &classification)
        {
            entries.push(entry);
        }
        if classification.is_directory {
            match std::fs::read_dir(&path) {
                Ok(children) => {
                    for child in children {
                        match child {
                            Ok(child) => pending.push(child.path()),
                            Err(_) => directory_errors += 1,
                        }
                    }
                }
                Err(_) => directory_errors += 1,
            }
        }
    }
    ScanSnapshot {
        entries,
        directory_errors,
    }
}
