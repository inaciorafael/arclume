use std::env;
use std::path::PathBuf;
use std::process::Command;

use std::os::windows::process::CommandExt;

use super::provider::collect_files;
use super::{AppEntry, LaunchSpec};

pub fn discover() -> Vec<AppEntry> {
    let mut roots = Vec::new();
    if let Some(app_data) = env::var_os("APPDATA") {
        roots.push(PathBuf::from(app_data).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    if let Some(program_data) = env::var_os("PROGRAMDATA") {
        roots.push(PathBuf::from(program_data).join(r"Microsoft\Windows\Start Menu\Programs"));
    }

    collect_files(roots, "lnk", 8)
        .into_iter()
        .filter_map(|path| {
            let title = path.file_stem()?.to_string_lossy().trim().to_owned();
            if title.is_empty() || title.eq_ignore_ascii_case("uninstall") {
                return None;
            }
            Some(AppEntry::new(
                title,
                "Windows application".into(),
                parent_keywords(&path),
                LaunchSpec::WindowsShortcut(path),
            ))
        })
        .collect()
}

fn parent_keywords(path: &std::path::Path) -> Vec<String> {
    path.parent()
        .and_then(|parent| parent.file_name())
        .map(|name| vec![name.to_string_lossy().into_owned()])
        .unwrap_or_default()
}

pub fn load_icon(shortcut_path: &std::path::Path) -> Option<String> {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const SCRIPT: &str = r#"
Add-Type -AssemblyName System.Drawing
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($env:ARCLUME_ICON_PATH)
$source = $shortcut.IconLocation
if ($source) { $source = $source.Split(',')[0].Trim('"') }
if (!$source -or !(Test-Path -LiteralPath ([Environment]::ExpandEnvironmentVariables($source)))) { $source = $shortcut.TargetPath }
$source = [Environment]::ExpandEnvironmentVariables($source)
if ($source -and (Test-Path -LiteralPath $source)) {
  $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($source)
  if ($icon) {
    $stream = New-Object System.IO.MemoryStream
    $bitmap = $icon.ToBitmap()
    $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
    [Convert]::ToBase64String($stream.ToArray())
    $bitmap.Dispose(); $stream.Dispose(); $icon.Dispose()
  }
}
"#;
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .env("ARCLUME_ICON_PATH", shortcut_path)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.len() > 1024 * 1024 {
        return None;
    }
    let encoded = String::from_utf8(output.stdout).ok()?;
    let encoded = encoded.trim();
    (!encoded.is_empty()).then(|| format!("data:image/png;base64,{encoded}"))
}
