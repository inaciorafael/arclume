# ADR-010: Local UI preferences and metadata preview

Status: accepted.

Keep presentation preferences in versioned frontend local storage because they affect only rendering and do not require Rust or SQLite coordination. Validate every loaded value and fall back safely when storage is unavailable.

Build the Phase 8 preview exclusively from the selected bounded search result. This preserves one request per query and avoids new filesystem disclosure, latency and cancellation paths.

The trade-off is that preferences do not roam between devices and the preview is metadata-only. Cross-device synchronization and rich content previews are deferred until they have explicit privacy and performance requirements.
