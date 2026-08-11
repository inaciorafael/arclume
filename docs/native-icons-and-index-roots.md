# Phase 19: native application icons and configurable index roots

Phase 19 closes two visible product gaps: generic application glyphs and files that were absent because their parent folder was never configured for indexing.

## Application icons

On Windows, application results request their icon lazily after search results are rendered. The core resolves the Start Menu shortcut's target/icon location and extracts its associated icon as PNG in a hidden, non-interactive PowerShell process.

- Search responses remain small and do not contain image data.
- Icon requests run sequentially in the frontend, outside the key-to-results path.
- Successful icons are cached by application ID for the process lifetime.
- Missing or failed icons keep the existing deterministic category fallback.
- The CSP allows `data:` only for images; scripts and connections remain unchanged.

macOS and Linux retain category fallbacks in this phase. Their native icon metadata formats require platform-specific acceptance and loaders before parity can be claimed.

## Index roots

Settings now lists the active folders and accepts absolute existing directories. Adding or removing a root:

1. validates and persists the native configuration;
2. starts a serialized background reconciliation;
3. updates the filesystem watcher within at most two seconds;
4. removes stale rows only from a complete reconciliation.

`C:\` can be selected explicitly, but Windows, Program Files, ProgramData, recycle/system metadata, VCS, dependency, build and cache directories are excluded by name. A narrower root such as `C:\Projects` is recommended because it reduces traversal time, database size and watcher load.

Exclusions are case-insensitive on Windows. Roots are never expanded silently; the original defaults remain Documents, Desktop and Downloads.

## Validation

- Vue/TypeScript production build, SDK and plugin tooling passed.
- Rust formatting and Clippy with warnings denied passed.
- 30 Rust tests passed after the first implementation; the exclusion assertion was then extended for Windows system directories.
- Icon extraction and live root reconciliation require interactive Windows confirmation in the updated application.
