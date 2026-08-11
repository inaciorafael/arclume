# ADR-008: Isolated plugin POC

Status: accepted for Phase 6 only.

Use a short-lived subprocess and bounded JSON-over-stdio protocol for the hello-world provider. This creates a measurable crash and timeout boundary without committing the public SDK to JavaScript, native ABI or WASM.

The trade-off is process startup overhead per non-empty query. This is acceptable for a single POC but not assumed acceptable for an ecosystem. Phase 7 must measure a persistent host and WASM/WASI, then supersede or retain this decision with evidence.

No manifest capability is automatically authorized. The POC grants none and cannot load an arbitrary entrypoint.
