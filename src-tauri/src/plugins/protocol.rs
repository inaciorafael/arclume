use serde::{Deserialize, Serialize};

pub const PLUGIN_API_VERSION: u32 = 1;
pub const MAX_REQUEST_BYTES: usize = 16 * 1024;
pub const MAX_RESPONSE_BYTES: usize = 32 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub plugin_api_version: u32,
    pub entrypoint: String,
    pub capabilities: Vec<String>,
    pub contributes: Contributions,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contributions {
    pub providers: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderRequest {
    pub plugin_api_version: u32,
    pub request_id: u64,
    pub query: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderResponse {
    pub plugin_api_version: u32,
    pub request_id: u64,
    pub results: Vec<PluginItem>,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub score: i64,
}
