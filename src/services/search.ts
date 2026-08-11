import { invoke } from "@tauri-apps/api/core";
import type { DictionaryEntry, SearchResponse } from "../types/search";

export function searchApplications(queryId: number, query: string) {
  return invoke<SearchResponse>("search", { queryId, query });
}

export function executeResult(id: string, query: string, title: string, kind: string, confirmed = false) {
  return invoke<DictionaryEntry | null>("execute_result", { id, query, title, kind, confirmed });
}
