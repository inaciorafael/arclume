# File search

Phase 3 indexes files and folders from the user's Documents, Desktop and Downloads directories when those locations exist. It never scans the entire disk by default.

## Lifecycle

1. The launcher registers its hotkey and creates the window.
2. A named background thread traverses configured roots without following symlinks.
3. Metadata is upserted into SQLite and an external-content FTS5 table maintained by triggers.
4. The platform-recommended `notify` watcher applies create, modify and remove events.
5. Search retrieves a bounded candidate set from SQLite; Rust performs final cross-provider ranking.

The database uses WAL and `synchronous=NORMAL`. It contains path, title, parent, kind, modification time and size—never file contents.

## Configuration

If present, `indexing.json` in the platform configuration directory accepts `roots`, `excludedNames` and `allowedExtensions`. An empty extension set allows every extension. Default exclusions include `.git`, `node_modules`, `target`, `dist`, `build`, `.cache` and `__pycache__`.

The settings format is currently an internal contract; the settings UI is scheduled for the polish phase. Invalid configuration fails index initialization visibly instead of silently scanning unintended locations.

## Recovery and limitations

- Watchers are advisory. Large bursts, network volumes and Linux inotify limits may lose events; scoped reconciliation is still required in a later hardening pass.
- Renames may arrive as remove/create or modify sequences depending on the backend.
- Initial indexing upserts current entries but does not yet prune stale rows missed while the app was stopped.
- File contents, icons, previews and full typo-tolerant candidate generation are out of scope for this phase.
