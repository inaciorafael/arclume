import { invoke } from "@tauri-apps/api/core";

export function getApplicationIcon(id: string) {
  return invoke<string | null>("application_icon", { id });
}
