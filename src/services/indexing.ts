import { invoke } from "@tauri-apps/api/core";

export function getIndexRoots() {
  return invoke<string[]>("get_index_roots");
}

export function addIndexRoot(root: string) {
  return invoke<string[]>("add_index_root", { root });
}

export function removeIndexRoot(root: string) {
  return invoke<string[]>("remove_index_root", { root });
}
