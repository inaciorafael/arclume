# ADR-012: Transactional index reconciliation

Status: accepted.

Recover watcher uncertainty with a full configured-root metadata snapshot and transactional SQLite reconciliation. Remove stale rows only after a complete traversal; otherwise favor stale results over destructive false deletion.

Use a dedicated WAL writer connection so reconciliation does not hold the search connection mutex. Trigger recovery on watcher errors and hourly as a safety net. Do not add a platform-specific journal dependency until cross-platform evidence shows the portable strategy is insufficient.

The trade-off is periodic filesystem I/O and temporary memory proportional to indexed metadata. On the Windows development dataset, prepared records reduced the observed recovery path from 143.2 s to 40.5 s. Future work should benchmark memory and consider scoped or journal-based reconciliation for very large roots.
