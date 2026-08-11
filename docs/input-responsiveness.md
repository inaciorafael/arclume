# Phase 13: input responsiveness

Phase 13 removes avoidable work from the key-to-results path while preserving the existing search and plugin security contracts.

## Confirmed causes

1. Every query change immediately invoked the complete backend pipeline. Fast typing could therefore create multiple concurrent searches whose results were later discarded.
2. The isolated hello-world POC started a short-lived native process for every non-empty query, despite only matching `hello`, `hello world`, `ola` or `olá`. Each process was allowed a 100 ms deadline.
3. Empty-query file results sorted the entire `file_items` table by `modified` without a supporting index. That query held the shared search connection while a typed query waited.

## Changes

- Query changes are coalesced within a 16 ms frame-sized window.
- At most one backend search is in flight. If the query changes during it, only the newest pending value runs next.
- Query IDs advance when input changes, so an older response cannot update the UI while a newer query is pending.
- The hello-world provider checks its declared matching domain before starting the isolated host. Relevant queries retain the same process isolation and 100 ms deadline.
- `file_items_modified` supports the empty-query `ORDER BY modified DESC LIMIT 40` path.

The scheduler does not debounce for hundreds of milliseconds: the maximum intentional coalescing delay is one 16 ms window, within the existing 30 ms acceptable key-to-common-results budget before backend and render work.

## Validation

- Vue/TypeScript production build passed.
- Private SDK and plugin tooling validation passed.
- Clippy passed for all targets/features with warnings denied.
- 28 Rust tests passed.
- A test verifies irrelevant plugin queries are rejected before process startup.
- A SQLite query-plan test verifies the recent-items query uses `file_items_modified`.

The automated evidence proves bounded request scheduling and query-plan selection. Perceived latency still needs interactive confirmation on this machine and p50/p95/p99 collection from the local performance panel.
