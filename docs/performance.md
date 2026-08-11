# Performance

Budgets are targets, not achieved measurements.

| Path | Ideal | Acceptable |
|---|---:|---:|
| hotkey to visible window | <50 ms | <100 ms |
| key to first common local results | <10 ms | <30 ms |
| final ranking | <2 ms | <8 ms |
| first result render | <8 ms | <16 ms |
| persistent index open | <75 ms | <200 ms |

Record p50, p95 and p99, peak RSS, disk size and hardware. Datasets contain 10k, 100k and 1M items; queries cover exact, prefix, fuzzy and no-result cases. Cold and warm runs are separate. Phase 1 currently logs the synchronous show-request duration; this is not equivalent to pixels-visible latency and must not be reported as such.

## Phase 3 quick benchmark — 2026-08-10

Criterion release profile, synthetic filename dataset, Windows development machine:

| Query path | 10k | 100k | 1M |
|---|---:|---:|---:|
| SQLite prefix, `document999` | 35.0 µs | 42.7 µs | 44.3 µs |
| SQLite prefix, `doc` | 11.4 µs | 19.1 µs | 22.1 µs |
| SQLite no result | 7.0 µs | 15.6 µs | 17.5 µs |
| Custom sequential no result | 0.61 ms | 5.87 ms | 63.45 ms |

Command: `cargo bench --manifest-path src-tauri/Cargo.toml --bench search_index -- --quick`.

This quick run is evidence for relative candidate-query behavior, not a production SLA. Index creation time, RSS, disk usage and cold reopen still require a controlled macrobenchmark.

## Phase 10 reconciliation observation — 2026-08-10

On the Windows development dataset (~163,900 paths), the startup consistency pass decreased from 143,234 ms to 40,470 ms after batching, unchanged-row suppression, single metadata classification and prepared scan records. The final run split into 26,013 ms filesystem traversal and 14,457 ms SQLite application. Reconciliation now uses a dedicated WAL connection, so this writer duration does not hold the search connection mutex.

This is a single debug observation affected by filesystem cache and concurrent machine activity. It is not a percentile benchmark or proof of the startup budget.

## Phase 11 diagnostic snapshot — 2026-08-10

The read-only index diagnostic observed 164,345 items, a 325,488,640-byte database and a 100,750,512-byte WAL. The first invocation opened SQLite in 4,914 µs and ran count/page queries in 2,399,801 µs. Five warm invocations measured open p50 344 µs and query p50 75,761 µs; the largest warm observations were 403 µs and 90,174 µs respectively.

A 20-sample Windows process check observed approximately 21.9 MB working set and 15.5 MB private memory for the warm native process. It excludes WebView child processes and startup peak. See `local-observability.md` for commands and limitations.

## Phase 13 input-path controls — 2026-08-10

The frontend now coalesces changes for at most 16 ms, permits one search in flight and retains only the newest pending query. Irrelevant queries no longer start the isolated hello-world process, whose deadline remains 100 ms for matching queries. The initial recent-file query now has a dedicated `modified DESC` index; its query plan is asserted by a test.

These are structural upper bounds and eliminated-work evidence, not perceived-latency percentiles. Interactive p50/p95/p99 measurements remain required after running the updated app.

## Phase 14 dispatch and composition — 2026-08-10

Search now uses Tauri's asynchronous command form with an owned query payload. Native window transparency and the 24 px full-window backdrop blur were removed, preventing continuous desktop-behind-window recomposition during result updates.

This reduces two architectural sources of key-to-paint delay. It remains improvement-by-design rather than a measured percentile claim; OS-level key event and presented-frame timestamps are still needed.
