# Indexing

Indexing is implemented with SQLite FTS5. The source-of-truth record contains path, kind, title, parent, size and modification timestamp; an external-content FTS table is synchronized by triggers.

Initial discovery creates the persistent index in a background thread. Native watchers are registered before the initial snapshot and apply incremental changes. Watcher errors trigger transactional configured-root reconciliation; a preventive pass runs hourly. Stale rows are removed only after an error-free traversal, and reconciliation uses a dedicated WAL connection so search reads remain available.

Default exclusions include `.git`, `node_modules`, `target`, `dist`, build caches and platform system directories where appropriate. Symlink traversal, network volumes and removable drives require explicit policy.

Reconciliation logs local reason/count/timing diagnostics. See `reliability-hardening.md` and ADR-012. Scoped reconciliation and platform journals remain possible optimizations, not current correctness requirements.
