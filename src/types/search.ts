export interface SearchResult {
  id: string;
  kind: "application" | "file" | "folder" | "action" | "plugin";
  title: string;
  subtitle: string;
  score: number;
  requiresConfirmation: boolean;
}

export interface SearchResponse {
  queryId: number;
  elapsedMicros: number;
  results: SearchResult[];
  diagnostics: SearchDiagnostics;
}

export interface DictionaryEntry {
  word: string;
  definitions: string[];
  source: string;
  cached: boolean;
}

export interface SearchDiagnostics {
  catalogSnapshotMicros: number;
  fileProviderMicros: number;
  actionProviderMicros: number;
  historyProviderMicros: number;
  pluginProviderMicros: number;
  rankingMicros: number;
}
