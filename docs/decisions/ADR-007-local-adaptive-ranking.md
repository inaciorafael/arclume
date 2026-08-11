# ADR-007: Local adaptive ranking

Status: accepted.

Store query-to-item affinity, frequency and recency in local SQLite. Apply a bounded additive boost after provider candidate generation and before deduplication/top-N selection.

A bounded transparent model is preferred over an opaque learned model at this stage. It is deterministic, testable, cheap to update and easy to clear. The trade-off is limited context awareness. Any future statistical model must preserve offline operation, provide migration/rollback and demonstrate relevance gains on a fixed dataset without leaking personal history.
