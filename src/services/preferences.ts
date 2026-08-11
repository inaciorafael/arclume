export type ThemePreference = "system" | "light" | "dark";
export type DensityPreference = "comfortable" | "compact";

export interface Preferences {
  theme: ThemePreference;
  density: DensityPreference;
  showPreview: boolean;
  showLatency: boolean;
  showPerformancePanel: boolean;
  reduceTransparency: boolean;
}

const STORAGE_KEY = "arclume.preferences.v1";

export const DEFAULT_PREFERENCES: Readonly<Preferences> = Object.freeze({
  theme: "system",
  density: "comfortable",
  showPreview: true,
  showLatency: false,
  showPerformancePanel: false,
  reduceTransparency: false,
});

export function loadPreferences(): Preferences {
  try {
    const candidate = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null") as Partial<Preferences> | null;
    if (!candidate || typeof candidate !== "object") return { ...DEFAULT_PREFERENCES };
    return {
      theme: candidate.theme === "light" || candidate.theme === "dark" ? candidate.theme : "system",
      density: candidate.density === "compact" ? "compact" : "comfortable",
      showPreview: typeof candidate.showPreview === "boolean" ? candidate.showPreview : true,
      showLatency: typeof candidate.showLatency === "boolean" ? candidate.showLatency : false,
      showPerformancePanel: typeof candidate.showPerformancePanel === "boolean" ? candidate.showPerformancePanel : false,
      reduceTransparency: typeof candidate.reduceTransparency === "boolean" ? candidate.reduceTransparency : false,
    };
  } catch {
    return { ...DEFAULT_PREFERENCES };
  }
}

export function savePreferences(preferences: Preferences): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences));
  } catch {
    // Preferences are optional; an unavailable storage backend must not break search.
  }
}
