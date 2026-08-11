# ADR-001: Search index

Status: accepted for Phase 3.

Use SQLite FTS5 as the persistent file metadata index, with prefix indexes for two-, three- and four-character tokens. Keep final provider aggregation and relevance scoring in Rust.

Alternatives:

- Tantivy provides a mature inverted index and strong full-text facilities, but its architecture favors batched segment updates. That adds operational complexity for frequent single-path watcher events.
- A custom in-memory subsequence index supports fuzzy matching directly but duplicates persistent state, increases memory with dataset size and makes recovery/migrations our responsibility.
- SQLite provides transactions, WAL recovery, prefix search and incremental row updates in one embedded dependency.

The reproducible Criterion run on 2026-08-10 measured SQLite FTS5 prefix queries between approximately 11–45 microseconds across synthetic datasets of 10k, 100k and 1M names. SQLite no-result queries measured about 7–18 microseconds. A custom sequential no-result scan grew from approximately 0.61 ms at 10k to 63.45 ms at 1M.

These results support SQLite for candidate generation. They do not prove real-filesystem indexing speed, memory use, disk size or ranking quality. The custom benchmark stops after 20 matches, so broad early-match measurements such as `doc` must not be interpreted as full-scan throughput. Tantivy remains a future comparison if SQLite cannot meet real dataset requirements.
