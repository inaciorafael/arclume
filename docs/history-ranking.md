# Local history and adaptive ranking

Phase 5 records successful executions locally in the existing SQLite database. No event is transmitted, synchronized or exposed to plugins.

Stored fields are normalized query, item ID, last displayed title, result kind, use count and last-used timestamp. Recent raw query display text is stored separately for future UX, but is not currently rendered.

## Ranking

Adaptive boost combines logarithmic frequency and exponentially decaying recency with a hard ceiling of 1,200 points. Exact text match starts at 10,000 and prefix match at 8,000, so history cannot make a fuzzy result displace an exact result. Query-specific affinity is used for non-empty queries; aggregate item history is used for the empty launcher.

Recent items are injected as candidates before deduplication, allowing an older file to appear even when it is not among the most recently modified files.

## Privacy controls

Search `clear history` and press Enter twice to delete both usage history and recent queries in one transaction. The core enforces confirmation. Clearing history does not delete the file index or application catalog.

History recording validates field lengths and occurs only after successful execution. Failed opens and cancelled confirmations are not recorded.
