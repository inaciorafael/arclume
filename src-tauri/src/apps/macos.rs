use std::env;
use std::path::PathBuf;

use super::provider::collect_files;
use super::{AppEntry, LaunchSpec};

pub fn discover() -> Vec<AppEntry> {
    let mut roots = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];
    if let Some(home) = env::var_os("HOME") {
        roots.push(PathBuf::from(home).join("Applications"));
    }
    collect_files(roots, "app", 3)
        .into_iter()
        .filter_map(|path| {
            let title = path.file_stem()?.to_string_lossy().trim().to_owned();
            Some(AppEntry::new(
                title,
                "macOS application".into(),
                Vec::new(),
                LaunchSpec::MacBundle(path),
            ))
        })
        .collect()
}
