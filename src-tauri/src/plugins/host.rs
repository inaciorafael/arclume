use std::io::{self, Read};

use super::protocol::{
    MAX_REQUEST_BYTES, PLUGIN_API_VERSION, PluginItem, PluginManifest, ProviderRequest,
    ProviderResponse,
};

const MANIFEST: &str = include_str!("../../../plugins/hello-world/plugin.json");

pub fn run_plugin_host() -> Result<(), String> {
    let mut payload = Vec::new();
    io::stdin()
        .take((MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut payload)
        .map_err(|error| format!("cannot read plugin request: {error}"))?;
    if payload.len() > MAX_REQUEST_BYTES {
        return Err("plugin request exceeds size limit".into());
    }
    let request: ProviderRequest = serde_json::from_slice(&payload)
        .map_err(|error| format!("invalid plugin request: {error}"))?;
    let response = evaluate(request)?;
    serde_json::to_writer(io::stdout(), &response)
        .map_err(|error| format!("cannot write plugin response: {error}"))
}

fn evaluate(request: ProviderRequest) -> Result<ProviderResponse, String> {
    validate_manifest()?;
    if request.plugin_api_version != PLUGIN_API_VERSION {
        return Err("unsupported plugin API version".into());
    }
    if request.query.len() > 512 {
        return Err("query exceeds plugin limit".into());
    }
    let normalized = request.query.trim().to_lowercase();
    let matches = ["hello", "hello world", "ola", "olá"]
        .iter()
        .any(|candidate| normalized.contains(candidate));
    let results = matches
        .then(|| PluginItem {
            id: "plugin:hello-world:greeting".into(),
            title: "Hello from Arclume".into(),
            subtitle: "Isolated hello-world plugin · API v1".into(),
            score: 7_500,
        })
        .into_iter()
        .collect();
    Ok(ProviderResponse {
        plugin_api_version: PLUGIN_API_VERSION,
        request_id: request.request_id,
        results,
        diagnostic: None,
    })
}

fn validate_manifest() -> Result<(), String> {
    let manifest: PluginManifest = serde_json::from_str(MANIFEST)
        .map_err(|error| format!("invalid hello-world manifest: {error}"))?;
    let valid_identity = manifest.id == "hello-world"
        && manifest.name == "Hello World"
        && manifest.version == "0.1.0"
        && manifest.entrypoint == "builtin:hello-world";
    if !valid_identity
        || manifest.plugin_api_version != PLUGIN_API_VERSION
        || !manifest.capabilities.is_empty()
        || manifest.contributes.providers != ["greeting"]
    {
        return Err("hello-world manifest violates the POC contract".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_provider_returns_structured_result() {
        let response = evaluate(ProviderRequest {
            plugin_api_version: PLUGIN_API_VERSION,
            request_id: 7,
            query: "olá".into(),
        })
        .unwrap();
        assert_eq!(response.request_id, 7);
        assert_eq!(response.results[0].id, "plugin:hello-world:greeting");
    }

    #[test]
    fn rejects_incompatible_api_version() {
        let error = evaluate(ProviderRequest {
            plugin_api_version: 99,
            request_id: 1,
            query: "hello".into(),
        })
        .unwrap_err();
        assert!(error.contains("unsupported"));
    }

    #[test]
    fn embedded_manifest_is_valid_and_has_no_capabilities() {
        validate_manifest().unwrap();
    }
}
