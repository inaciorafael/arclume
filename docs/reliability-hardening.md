# Phase 10: local reliability hardening

Phase 10 closes the known filesystem watcher consistency gap without adding product features.

## Reconciliation model

1. Register native watchers before the startup snapshot.
2. Walk configured roots once and prepare file records from the observed metadata.
3. Apply records in one SQLite transaction using a temporary observed-path table.
4. Delete stale rows only when the complete traversal reports zero directory/entry errors.
5. Preserve unobserved rows after an incomplete traversal.
6. Reconcile immediately after any watcher error and preventively every hour.

Create/modify events whose paths no longer exist are treated as removals, covering common rename sequences that previously left stale source paths.

## Availability

Reconciliation uses a dedicated SQLite WAL connection. The existing read connection remains available to search while the batch writes. Watcher events are processed on the indexer thread after reconciliation and cannot race the snapshot transaction inside that worker.

## Local diagnostics

Every reconciliation logs its reason, observed/indexed/removed counts, traversal errors, stale-removal decision and separate `scan_ms`, `apply_ms` and total elapsed time. No metric leaves the device.

## Safety properties

- excluded directories and symlinks are not traversed;
- a single unreadable directory disables global stale deletion for that pass;
- the transaction rolls back on database failure;
- unchanged records do not fire UPDATE/FTS triggers;
- no network, analytics or new dependency is involved.

## Windows development evidence

Single debug runs over approximately 163,900 observed items on 2026-08-10:

| Implementation | Total |
|---|---:|
| Previous per-item startup scan | 143,234 ms |
| Transactional reconciliation, first stale cleanup | 94,136 ms |
| Skip unchanged FTS updates | 72,083 ms |
| Single metadata classification | 58,220 ms |
| Prepared scan records passed to SQLite | 40,470 ms |

The final observed run reported `scan_ms=26,013` and `apply_ms=14,457`, about 72% lower total time than the recorded baseline. These are single-machine operational observations, not p50/p95/p99 benchmarks. macOS/Linux and controlled cold/warm macrobenchmarks remain required.
