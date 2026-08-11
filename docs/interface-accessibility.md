# Interface, preferences and accessibility

Phase 8 adds a contextual result preview and device-local presentation preferences without adding IPC calls to the search path.

## Preferences

Preferences are stored under the versioned browser key `arclume.preferences.v1`:

- theme: system, light or dark;
- result density: comfortable or compact;
- contextual preview visibility;
- search latency visibility;
- local performance panel visibility;
- reduced transparency.

Invalid or unavailable storage falls back to defaults and never blocks search. These settings contain no search history or result content.

## Keyboard behavior

- `Arrow Up` / `Arrow Down`: move through results;
- `Home` / `End`: select the first or last result;
- `Enter`: execute, or confirm a protected action;
- `Escape`: cancel confirmation, clear a query, then hide the launcher;
- `Ctrl+,` / `Command+,`: open settings;
- `Escape` inside settings: close the dialog and restore focus to search.

## Assistive technology

Search uses the combobox/listbox pattern with `aria-controls`, `aria-expanded`, `aria-activedescendant` and selected options. Result count, errors and confirmation prompts use a polite live region. Settings use a modal native dialog with labelled controls and managed focus.

The stylesheet honors `prefers-reduced-motion` and forced-colors mode. A reduce-transparency preference is available independently of the operating system.

## Preview boundary

The preview renders metadata already present in a bounded `SearchResult`; it does not read file contents or issue one IPC request per selection. Rich file previews remain future work because they require file-size, MIME, permission and cancellation policies.

## Evidence

The production frontend build passed. Browser validation at the native window viewport (`720 × 500`) covered dark/light rendering, compact density, preview visibility persistence, modal semantics and Escape focus restoration. Search results themselves require the Tauri IPC bridge and remain covered by the Rust payload tests and native app smoke test.
