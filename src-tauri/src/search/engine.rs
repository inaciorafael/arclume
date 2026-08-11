use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;

use serde::Serialize;

use super::normalize::normalize;
use super::ranking::{acronym_score, text_score};
use crate::actions::ActionItem;
use crate::apps::AppEntry;
use crate::history::RecentItem;
use crate::index::IndexedItem;
use crate::plugins::PluginItem;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    query_id: u64,
    elapsed_micros: u128,
    results: Vec<SearchResult>,
    diagnostics: SearchDiagnostics,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchDiagnostics {
    pub catalog_snapshot_micros: u128,
    pub file_provider_micros: u128,
    pub action_provider_micros: u128,
    pub history_provider_micros: u128,
    pub plugin_provider_micros: u128,
    pub ranking_micros: u128,
}

impl SearchResponse {
    pub fn with_diagnostics(
        mut self,
        elapsed_micros: u128,
        mut diagnostics: SearchDiagnostics,
    ) -> Self {
        diagnostics.ranking_micros = self.elapsed_micros;
        self.elapsed_micros = elapsed_micros;
        self.diagnostics = diagnostics;
        self
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    id: String,
    kind: &'static str,
    title: String,
    subtitle: String,
    score: i64,
    requires_confirmation: bool,
}

#[expect(
    clippy::too_many_arguments,
    reason = "the ranking boundary receives one explicit collection per search provider"
)]
pub fn search_applications(
    query_id: u64,
    query: &str,
    applications: &[AppEntry],
    files: &[IndexedItem],
    actions: &[ActionItem],
    boosts: &HashMap<String, i64>,
    recent_items: &[RecentItem],
    plugin_items: &[PluginItem],
    limit: usize,
) -> SearchResponse {
    let started_at = Instant::now();
    let normalized_query = normalize(query);
    let mut results: Vec<SearchResult> = applications
        .iter()
        .filter_map(|application| {
            let title_score = text_score(&normalized_query, &normalize(&application.title));
            let acronym_score = acronym_score(&normalized_query, &application.title);
            let keyword_score = application
                .keywords
                .iter()
                .filter_map(|keyword| text_score(&normalized_query, &normalize(keyword)))
                .max();
            title_score
                .into_iter()
                .chain(acronym_score)
                .chain(keyword_score)
                .max()
                .map(|score| SearchResult {
                    id: application.id.clone(),
                    kind: "application",
                    title: application.title.clone(),
                    subtitle: application.subtitle.clone(),
                    score: score + boosts.get(&application.id).copied().unwrap_or_default(),
                    requires_confirmation: false,
                })
        })
        .collect();
    results.extend(files.iter().filter_map(|file| {
        let score = text_score(&normalized_query, &normalize(&file.title))? - 100;
        let id = format!("file:{}", file.id);
        Some(SearchResult {
            id: id.clone(),
            kind: if file.kind == "folder" {
                "folder"
            } else {
                "file"
            },
            title: file.title.clone(),
            subtitle: file.subtitle.clone(),
            score: score + boosts.get(&id).copied().unwrap_or_default(),
            requires_confirmation: false,
        })
    }));
    results.extend(actions.iter().map(|action| SearchResult {
        id: action.id.clone(),
        kind: "action",
        title: action.title.clone(),
        subtitle: action.subtitle.clone(),
        score: action.score + boosts.get(&action.id).copied().unwrap_or_default(),
        requires_confirmation: action.requires_confirmation,
    }));
    results.extend(plugin_items.iter().map(|item| SearchResult {
        id: item.id.clone(),
        kind: "plugin",
        title: item.title.clone(),
        subtitle: item.subtitle.clone(),
        score: item.score + boosts.get(&item.id).copied().unwrap_or_default(),
        requires_confirmation: false,
    }));
    if normalize(query) == "clearhistory" {
        results.push(SearchResult {
            id: "history:clear".into(),
            kind: "action",
            title: "Clear local history".into(),
            subtitle: "Privacy action · confirmation required".into(),
            score: 9_800,
            requires_confirmation: true,
        });
    }
    results.extend(recent_items.iter().map(|item| SearchResult {
        id: item.id.clone(),
        kind: match item.kind.as_str() {
            "application" => "application",
            "folder" => "folder",
            "file" => "file",
            _ => "action",
        },
        title: item.title.clone(),
        subtitle: "Recent item".into(),
        score: boosts.get(&item.id).copied().unwrap_or_default(),
        requires_confirmation: false,
    }));
    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.title.cmp(&right.title))
    });
    let mut seen = HashSet::new();
    results.retain(|result| seen.insert(result.id.clone()));
    results.truncate(limit);
    SearchResponse {
        query_id,
        elapsed_micros: started_at.elapsed().as_micros(),
        results,
        diagnostics: SearchDiagnostics::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apps::LaunchSpec;
    use std::path::PathBuf;

    fn app(title: &str) -> AppEntry {
        #[cfg(target_os = "windows")]
        let launch = LaunchSpec::WindowsShortcut(PathBuf::from(title));
        #[cfg(target_os = "macos")]
        let launch = LaunchSpec::MacBundle(PathBuf::from(title));
        #[cfg(target_os = "linux")]
        let launch = LaunchSpec::LinuxCommand {
            program: title.into(),
            args: Vec::new(),
        };
        AppEntry::new(title.into(), "Application".into(), Vec::new(), launch)
    }

    #[test]
    fn ranks_visual_studio_code_for_vsc() {
        let response = search_applications(
            7,
            "vsc",
            &[app("vscode-config"), app("Visual Studio Code")],
            &[],
            &[],
            &HashMap::new(),
            &[],
            &[],
            10,
        );
        assert_eq!(response.query_id, 7);
        assert_eq!(response.results[0].title, "Visual Studio Code");
    }

    #[test]
    fn adaptive_boost_cannot_displace_exact_match() {
        let exact = app("Code");
        let fuzzy = app("Codec Settings");
        let mut boosts = HashMap::new();
        boosts.insert(fuzzy.id.clone(), 1_200);
        let response =
            search_applications(8, "code", &[fuzzy, exact], &[], &[], &boosts, &[], &[], 10);
        assert_eq!(response.results[0].title, "Code");
    }

    #[test]
    fn includes_structured_plugin_result_in_search_payload() {
        let plugin = PluginItem {
            id: "plugin:hello-world:greeting".into(),
            title: "Hello from Arclume".into(),
            subtitle: "Isolated plugin".into(),
            score: 7_500,
        };
        let response = search_applications(
            9,
            "hello",
            &[],
            &[],
            &[],
            &HashMap::new(),
            &[],
            &[plugin],
            10,
        );
        assert_eq!(response.results[0].kind, "plugin");
        assert_eq!(response.results[0].id, "plugin:hello-world:greeting");
    }

    #[test]
    fn attaches_provider_diagnostics_and_total_elapsed_time() {
        let response = search_applications(10, "", &[], &[], &[], &HashMap::new(), &[], &[], 10)
            .with_diagnostics(
                123,
                SearchDiagnostics {
                    file_provider_micros: 7,
                    ..SearchDiagnostics::default()
                },
            );
        assert_eq!(response.elapsed_micros, 123);
        assert_eq!(response.diagnostics.file_provider_micros, 7);
    }
}
