use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::protocol::{
    MAX_RESPONSE_BYTES, PLUGIN_API_VERSION, PluginItem, ProviderRequest, ProviderResponse,
};

const PROVIDER_DEADLINE: Duration = Duration::from_millis(100);

#[derive(Default)]
pub struct PluginManager;

impl PluginManager {
    pub fn search(&self, request_id: u64, query: &str) -> Vec<PluginItem> {
        if query.trim().is_empty() || query.len() > 512 || !query_may_match(query) {
            return Vec::new();
        }
        match self.call_host(request_id, query) {
            Ok(results) => results,
            Err(error) => {
                eprintln!("hello-world plugin unavailable: {error}");
                Vec::new()
            }
        }
    }

    pub fn execute(&self, id: &str) -> Result<(), String> {
        match id {
            "plugin:hello-world:greeting" => Ok(()),
            _ => Err("unknown plugin result".into()),
        }
    }

    fn call_host(&self, request_id: u64, query: &str) -> Result<Vec<PluginItem>, String> {
        let executable = std::env::current_exe()
            .map_err(|error| format!("cannot locate plugin host: {error}"))?;
        let mut child = Command::new(executable)
            .arg("--plugin-host")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("cannot start plugin host: {error}"))?;
        let request = ProviderRequest {
            plugin_api_version: PLUGIN_API_VERSION,
            request_id,
            query: query.into(),
        };
        let payload = serde_json::to_vec(&request)
            .map_err(|error| format!("cannot encode plugin request: {error}"))?;
        let mut stdin = child.stdin.take().ok_or("plugin stdin unavailable")?;
        stdin
            .write_all(&payload)
            .and_then(|_| stdin.flush())
            .map_err(|error| format!("cannot send plugin request: {error}"))?;
        drop(stdin);

        let started_at = Instant::now();
        loop {
            if child
                .try_wait()
                .map_err(|error| format!("cannot inspect plugin host: {error}"))?
                .is_some()
            {
                break;
            }
            if started_at.elapsed() >= PROVIDER_DEADLINE {
                let _ = child.kill();
                let _ = child.wait();
                return Err("provider deadline exceeded".into());
            }
            thread::sleep(Duration::from_millis(2));
        }
        let mut payload = Vec::new();
        child
            .stdout
            .take()
            .ok_or("plugin stdout unavailable")?
            .take((MAX_RESPONSE_BYTES + 1) as u64)
            .read_to_end(&mut payload)
            .map_err(|error| format!("cannot read plugin response: {error}"))?;
        if payload.len() > MAX_RESPONSE_BYTES {
            return Err("plugin response exceeds size limit".into());
        }
        let response: ProviderResponse = serde_json::from_slice(&payload)
            .map_err(|error| format!("invalid plugin response: {error}"))?;
        if response.plugin_api_version != PLUGIN_API_VERSION || response.request_id != request_id {
            return Err("plugin response identity mismatch".into());
        }
        if response.results.len() > 8 {
            return Err("plugin returned too many results".into());
        }
        Ok(response.results)
    }
}

fn query_may_match(query: &str) -> bool {
    let normalized = query.trim().to_lowercase();
    ["hello", "hello world", "ola", "olá"]
        .iter()
        .any(|candidate| normalized.contains(candidate))
}

#[cfg(test)]
mod tests {
    use super::query_may_match;

    #[test]
    fn skips_process_start_for_queries_outside_the_provider_domain() {
        assert!(!query_may_match("visual studio code"));
        assert!(!query_may_match("calculator"));
        assert!(query_may_match("hello"));
        assert!(query_may_match("olá mundo"));
    }
}
