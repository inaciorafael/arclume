use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct AppEntry {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub keywords: Vec<String>,
    pub launch: LaunchSpec,
}

#[derive(Clone, Debug)]
pub enum LaunchSpec {
    #[cfg(target_os = "windows")]
    WindowsShortcut(PathBuf),
    #[cfg(target_os = "macos")]
    MacBundle(PathBuf),
    #[cfg(target_os = "linux")]
    LinuxCommand { program: String, args: Vec<String> },
}

impl AppEntry {
    pub fn new(title: String, subtitle: String, keywords: Vec<String>, launch: LaunchSpec) -> Self {
        let mut hasher = DefaultHasher::new();
        launch.hash_key().hash(&mut hasher);
        let id = format!("app:{:016x}", hasher.finish());
        Self {
            id,
            title,
            subtitle,
            keywords,
            launch,
        }
    }

    pub fn launch(&self) -> Result<(), String> {
        let result = match &self.launch {
            #[cfg(target_os = "windows")]
            LaunchSpec::WindowsShortcut(path) => Command::new("explorer.exe").arg(path).spawn(),
            #[cfg(target_os = "macos")]
            LaunchSpec::MacBundle(path) => Command::new("open").arg("-a").arg(path).spawn(),
            #[cfg(target_os = "linux")]
            LaunchSpec::LinuxCommand { program, args } => Command::new(program).args(args).spawn(),
        };
        result
            .map(|_| ())
            .map_err(|error| format!("failed to launch {}: {error}", self.title))
    }
}

impl LaunchSpec {
    fn hash_key(&self) -> String {
        match self {
            #[cfg(target_os = "windows")]
            Self::WindowsShortcut(path) => format!("windows:{}", path.to_string_lossy()),
            #[cfg(target_os = "macos")]
            Self::MacBundle(path) => format!("macos:{}", path.to_string_lossy()),
            #[cfg(target_os = "linux")]
            Self::LinuxCommand { program, args } => format!("linux:{program}\0{}", args.join("\0")),
        }
    }
}
