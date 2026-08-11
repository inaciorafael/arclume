# Application search

Phase 2 builds an in-memory catalog once during startup and sends at most 12 ranked results per IPC response. Each response repeats the monotonically increasing `queryId`; Vue discards stale responses.

## Discovery

- Windows scans per-user and all-user Start Menu `Programs` known locations for `.lnk` entries. Launch uses `explorer.exe` with the shortcut path as a separate argument. Packaged applications without Start Menu shortcuts are a documented gap.
- macOS scans `/Applications`, `/System/Applications` and `~/Applications` for `.app` bundles. Launch uses the system `open -a` interface. Localized bundle display names are not yet extracted.
- Linux scans XDG application directories for visible `.desktop` entries, parses `Name`, `GenericName`, `Exec`, `Hidden` and `NoDisplay`, and removes desktop field codes before process execution. Full desktop-entry locale and `TryExec` handling remain pending.

Discovery failures are scoped to unreadable directories or malformed entries. The catalog remains available with successfully parsed applications.

## Relevance baseline

The deterministic score orders exact, prefix, substring and ordered-subsequence matches. It is intentionally dependency-free and covered by ranking tests. History and recency are not included until Phase 5.

## Security

Application identifiers are opaque hashes. The frontend cannot provide a path or executable; `launch_application` resolves the ID against the catalog created by Rust. Arguments from Linux desktop entries are passed directly to `Command`, never interpolated into a shell string.
