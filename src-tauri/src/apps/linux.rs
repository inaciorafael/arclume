use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::provider::collect_files;
use super::{AppEntry, LaunchSpec};

pub fn discover() -> Vec<AppEntry> {
    let mut roots = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];
    if let Some(data_home) = env::var_os("XDG_DATA_HOME") {
        roots.push(PathBuf::from(data_home).join("applications"));
    } else if let Some(home) = env::var_os("HOME") {
        roots.push(PathBuf::from(home).join(".local/share/applications"));
    }
    collect_files(roots, "desktop", 4)
        .into_iter()
        .filter_map(|path| parse_desktop_file(&path))
        .collect()
}

fn parse_desktop_file(path: &Path) -> Option<AppEntry> {
    let content = fs::read_to_string(path).ok()?;
    parse_desktop_entry(&content)
}

fn parse_desktop_entry(content: &str) -> Option<AppEntry> {
    let mut in_entry = false;
    let mut name = None;
    let mut generic_name = None;
    let mut exec = None;
    let mut hidden = false;
    let mut no_display = false;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "Name" => name = Some(value.trim().to_owned()),
            "GenericName" => generic_name = Some(value.trim().to_owned()),
            "Exec" => exec = parse_exec(value),
            "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
            "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
            _ => {}
        }
    }
    if hidden || no_display {
        return None;
    }
    let title = name?;
    let (program, args) = exec?;
    Some(AppEntry::new(
        title,
        generic_name.unwrap_or_else(|| "Linux application".into()),
        Vec::new(),
        LaunchSpec::LinuxCommand { program, args },
    ))
}

fn parse_exec(value: &str) -> Option<(String, Vec<String>)> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '"' => quoted = !quoted,
            character if character.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }
    if escaped || quoted {
        return None;
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens.retain(|token| !token.starts_with('%'));
    let program = tokens.first()?.clone();
    Some((program, tokens.into_iter().skip(1).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_visible_desktop_entry_and_removes_field_codes() {
        let entry = parse_desktop_entry(
            "[Desktop Entry]\nName=Code\nGenericName=Editor\nExec=code --reuse-window %F\n",
        )
        .unwrap();
        assert_eq!(entry.title, "Code");
        let LaunchSpec::LinuxCommand { program, args } = entry.launch else {
            panic!("wrong launch kind")
        };
        assert_eq!(program, "code");
        assert_eq!(args, ["--reuse-window"]);
    }

    #[test]
    fn ignores_hidden_desktop_entry() {
        assert!(
            parse_desktop_entry("[Desktop Entry]\nName=Hidden\nExec=hidden\nNoDisplay=true\n")
                .is_none()
        );
    }
}
