use std::collections::HashSet;
use std::path::{Path, PathBuf};

use directories::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};

const DEFAULT_EXCLUSIONS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".cache",
    "__pycache__",
    "windows",
    "program files",
    "program files (x86)",
    "programdata",
    "$recycle.bin",
    "system volume information",
];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexSettings {
    pub roots: Vec<PathBuf>,
    pub excluded_names: HashSet<String>,
    pub allowed_extensions: HashSet<String>,
    #[serde(skip)]
    data_dir: PathBuf,
    #[serde(skip)]
    config_path: PathBuf,
}

pub struct PathClassification {
    pub is_directory: bool,
    pub should_index: bool,
    pub metadata: std::fs::Metadata,
}

impl IndexSettings {
    pub fn load() -> Result<Self, String> {
        let project = ProjectDirs::from("com", "Arclume", "Arclume")
            .ok_or("OS data directory is unavailable")?;
        let data_dir = project.data_local_dir().to_path_buf();
        let config_path = project.config_dir().join("indexing.json");
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            let mut settings: Self = serde_json::from_str(&content)
                .map_err(|error| format!("invalid indexing settings: {error}"))?;
            settings
                .excluded_names
                .extend(DEFAULT_EXCLUSIONS.iter().map(|value| (*value).into()));
            settings.data_dir = data_dir;
            settings.config_path = config_path;
            return Ok(settings);
        }
        let user = UserDirs::new().ok_or("user directories are unavailable")?;
        let mut roots = [user.document_dir(), user.desktop_dir(), user.download_dir()]
            .into_iter()
            .flatten()
            .map(Path::to_path_buf)
            .collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        Ok(Self {
            roots,
            excluded_names: DEFAULT_EXCLUSIONS
                .iter()
                .map(|value| (*value).into())
                .collect(),
            allowed_extensions: HashSet::new(),
            data_dir,
            config_path,
        })
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("search-index.sqlite3")
    }

    pub fn classify(&self, path: &Path) -> std::io::Result<Option<PathClassification>> {
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| self.excluded_names.contains(&name.to_ascii_lowercase()))
        {
            return Ok(None);
        }
        let metadata = std::fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() {
            return Ok(None);
        }
        let is_directory = metadata.is_dir();
        let should_index = is_directory
            || self.allowed_extensions.is_empty()
            || path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|extension| {
                    self.allowed_extensions
                        .contains(&extension.to_ascii_lowercase())
                });
        Ok(Some(PathClassification {
            is_directory,
            should_index,
            metadata,
        }))
    }

    pub fn should_index(&self, path: &Path) -> bool {
        self.classify(path)
            .is_ok_and(|value| value.is_some_and(|value| value.should_index))
    }

    pub fn add_root(&mut self, root: PathBuf) -> Result<(), String> {
        if !root.is_absolute() {
            return Err("index root must be an absolute path".into());
        }
        if !root.is_dir() {
            return Err("index root must be an existing directory".into());
        }
        if self.roots.iter().any(|existing| same_path(existing, &root)) {
            return Ok(());
        }
        self.roots.push(root);
        self.roots.sort();
        self.save()
    }

    pub fn remove_root(&mut self, root: &Path) -> Result<(), String> {
        self.roots.retain(|existing| !same_path(existing, root));
        self.save()
    }

    fn save(&self) -> Result<(), String> {
        let parent = self
            .config_path
            .parent()
            .ok_or("invalid index configuration path")?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create index configuration directory: {error}"))?;
        let payload = serde_json::to_vec_pretty(self)
            .map_err(|error| format!("failed to encode index settings: {error}"))?;
        std::fs::write(&self.config_path, payload)
            .map_err(|error| format!("failed to save index settings: {error}"))
    }
}

#[cfg(target_os = "windows")]
fn same_path(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

#[cfg(not(target_os = "windows"))]
fn same_path(left: &Path, right: &Path) -> bool {
    left == right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_exclusions_cover_dependency_and_vcs_directories() {
        let excluded: HashSet<String> = DEFAULT_EXCLUSIONS
            .iter()
            .map(|value| (*value).into())
            .collect();
        assert!(excluded.contains("node_modules"));
        assert!(excluded.contains(".git"));
        assert!(excluded.contains("target"));
        assert!(excluded.contains("windows"));
        assert!(excluded.contains("program files"));
        assert!(excluded.contains("system volume information"));
    }
}
