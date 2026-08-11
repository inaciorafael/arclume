# Phase 11: local observability and diagnostic snapshots

Phase 11 adds measurements without analytics, upload, identifiers or background reporting.

## Search diagnostics

Every search response contains local microsecond timings for:

- application catalog snapshot;
- file provider;
- action provider;
- history provider;
- plugin provider;
- ranking/merge;
- total Rust command time.

The optional performance panel keeps a memory-only ring of the latest 200 successful queries. It shows round-trip p50/p95/p99 and the latest Vue DOM commit approximation. Samples disappear when the webview closes and are never written to disk or sent over the network.

Enable it in `Ctrl+,` → **Show local performance panel**.

`render` ends after Vue `nextTick`; it does not measure WebView compositor pixels. Round-trip starts immediately before Tauri `invoke` and ends when the response resolves. Neither metric is equivalent to hotkey-to-visible.

## Index diagnostic

```shell
cargo run --manifest-path src-tauri/Cargo.toml --bin index-diagnostics
```

The read-only command prints JSON containing SQLite open/query time, indexed item count, database/WAL bytes, pages and freelist pages. It does not print indexed paths.

First Windows development snapshot:

- 164,345 indexed items;
- SQLite open: 4,914 µs on the first invocation;
- diagnostic queries: 2,399,801 µs on the first invocation;
- database: 325,488,640 bytes;
- WAL: 100,750,512 bytes.

Five immediately repeated warm samples:

| Metric | p50 | p95/p99 | Range |
|---|---:|---:|---:|
| SQLite open | 344 µs | 403 µs | 315–403 µs |
| diagnostic queries | 75,761 µs | 90,174 µs | 71,012–90,174 µs |

Five samples are a smoke distribution, not a statistically sufficient percentile benchmark. The cold/warm difference and ~426 MB database+WAL footprint are now explicit hardening targets.

## Windows process sampler

```powershell
powershell -NoProfile -File tools/process-diagnostics.ps1 -ProcessId <PID> -Samples 200 -IntervalMilliseconds 50
```

The script outputs p50/p95/p99/peak working-set and private bytes. A 20-sample validation against the warm development process observed approximately 21.9 MB working set and 15.5 MB private memory. It does not capture cold-start peak or GPU/WebView child processes and cannot be generalized to macOS/Linux.

## Remaining macrobenchmarks

- hotkey to pixels-visible with OS-level timestamps;
- cold and warm app startup p50/p95/p99;
- first query during reconciliation;
- full process-tree RSS including WebView;
- controlled index datasets and disk-state conditions;
- macOS and Linux equivalents.
