import { invoke } from "@tauri-apps/api/core";

export const GLOBAL_SHORTCUT_OPTIONS = [
  { value: "CmdOrControl+Space", label: "Ctrl/Command + Space" },
  { value: "CmdOrControl+Shift+Space", label: "Ctrl/Command + Shift + Space" },
  { value: "Alt+Space", label: "Alt/Option + Space" },
  { value: "CmdOrControl+Alt+Space", label: "Ctrl/Command + Alt/Option + Space" },
] as const;

export function getGlobalShortcut() {
  return invoke<string>("get_global_shortcut");
}

export function setGlobalShortcut(shortcut: string) {
  return invoke<void>("set_global_shortcut", { shortcut });
}
