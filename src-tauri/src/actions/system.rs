use std::process::Command;

#[derive(Clone, Copy, Debug)]
pub enum SystemAction {
    Lock,
    Sleep,
    Restart,
    Shutdown,
}

impl SystemAction {
    pub fn parse(query: &str) -> Option<Self> {
        match query
            .trim()
            .trim_start_matches('>')
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "lock" | "lock screen" => Some(Self::Lock),
            "sleep" | "suspend" => Some(Self::Sleep),
            "restart" | "reboot" => Some(Self::Restart),
            "shutdown" | "power off" => Some(Self::Shutdown),
            _ => None,
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Lock => "Lock screen",
            Self::Sleep => "Sleep",
            Self::Restart => "Restart computer",
            Self::Shutdown => "Shut down computer",
        }
    }
    pub fn subtitle(self) -> &'static str {
        if self.requires_confirmation() {
            "System action · confirmation required"
        } else {
            "System action"
        }
    }
    pub fn requires_confirmation(self) -> bool {
        !matches!(self, Self::Lock)
    }

    pub fn execute(self) -> Result<(), String> {
        let mut command = platform_command(self);
        command
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to execute system action: {error}"))
    }
}

#[cfg(target_os = "windows")]
fn platform_command(action: SystemAction) -> Command {
    match action {
        SystemAction::Lock => {
            let mut c = Command::new("rundll32.exe");
            c.args(["user32.dll,LockWorkStation"]);
            c
        }
        SystemAction::Sleep => {
            let mut c = Command::new("rundll32.exe");
            c.args(["powrprof.dll,SetSuspendState", "0,1,0"]);
            c
        }
        SystemAction::Restart => {
            let mut c = Command::new("shutdown.exe");
            c.args(["/r", "/t", "0"]);
            c
        }
        SystemAction::Shutdown => {
            let mut c = Command::new("shutdown.exe");
            c.args(["/s", "/t", "0"]);
            c
        }
    }
}

#[cfg(target_os = "macos")]
fn platform_command(action: SystemAction) -> Command {
    let mut command = Command::new("osascript");
    let script = match action {
        SystemAction::Lock => {
            "tell application \"System Events\" to keystroke \"q\" using {control down, command down}"
        }
        SystemAction::Sleep => "tell application \"System Events\" to sleep",
        SystemAction::Restart => "tell application \"System Events\" to restart",
        SystemAction::Shutdown => "tell application \"System Events\" to shut down",
    };
    command.args(["-e", script]);
    command
}

#[cfg(target_os = "linux")]
fn platform_command(action: SystemAction) -> Command {
    let (program, argument) = match action {
        SystemAction::Lock => ("loginctl", "lock-session"),
        SystemAction::Sleep => ("systemctl", "suspend"),
        SystemAction::Restart => ("systemctl", "reboot"),
        SystemAction::Shutdown => ("systemctl", "poweroff"),
    };
    let mut command = Command::new(program);
    command.arg(argument);
    command
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn destructive_actions_require_confirmation() {
        assert!(!SystemAction::Lock.requires_confirmation());
        assert!(SystemAction::Shutdown.requires_confirmation());
    }
}
