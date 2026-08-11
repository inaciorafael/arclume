import { invoke } from "@tauri-apps/api/core";

export interface ClipboardSettings {
  enabled: boolean;
  maxItems: number;
  maxTotalBytes: number;
  retentionDays: number;
}

export interface ClipboardItem {
  id: number;
  kind: "text" | "image";
  preview: string;
  byteSize: number;
  width?: number;
  height?: number;
  capturedAt: number;
}

export const getClipboardSettings = () => invoke<ClipboardSettings>("get_clipboard_settings");
export const setClipboardSettings = (settings: ClipboardSettings) => invoke<ClipboardSettings>("set_clipboard_settings", { settings });
export const listClipboardItems = (limit = 100) => invoke<ClipboardItem[]>("list_clipboard_items", { limit });
export const getClipboardImage = (id: number) => invoke<string>("clipboard_image", { id });
export const restoreClipboardItem = (id: number) => invoke<void>("restore_clipboard_item", { id });
export const clearClipboardHistory = () => invoke<void>("clear_clipboard_history");
