use std::fs;
use std::path::{Path, PathBuf};

pub fn collect_files(
    roots: impl IntoIterator<Item = PathBuf>,
    extension: &str,
    max_depth: usize,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending: Vec<_> = roots.into_iter().map(|path| (path, 0usize)).collect();
    while let Some((directory, depth)) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && depth < max_depth {
                pending.push((path, depth + 1));
            } else if has_extension(&path, extension) {
                files.push(path);
            }
        }
    }
    files
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}
