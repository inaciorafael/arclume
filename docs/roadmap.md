# Roadmap

0. Research, architecture and ADRs.
1. Resident launcher POC with mock results and measurable window path.
2. Native application discovery, fuzzy lookup and open action. **Implemented as a cross-platform baseline; OS acceptance testing remains required.**
3. File index, persistence, watcher and ranking baseline. **Implemented; overflow reconciliation and OS macrobenchmarks remain hardening work.**
4. Safe calculator, deterministic conversions and system actions. **Implemented with core-enforced confirmation.**
5. Local history and adaptive ranking. **Implemented with bounded weights and transactional clearing.**
6. Isolated hello-world plugin POC. **Implemented with bounded JSON IPC, a short-lived host and deny-by-default capabilities.**
7. Versioned SDK, manifest tooling and developer workflow. **Implemented as a private API v1 contract with schema, validator, scaffolder and tests; third-party loading remains intentionally disabled.**
8. Preview, settings, accessibility, themes and polish. **Implemented with metadata-only preview, validated local preferences, keyboard/focus semantics, light/dark themes and reduced-motion/forced-colors support.**
9. Optional ecosystem services only after the local platform is stable. **Evaluated: no-go until cross-platform, performance, permission-broker, supply-chain and security-operation gates pass; no remote service or telemetry added.**
10. Local reliability hardening. **Implemented transactional watcher recovery, incomplete-scan deletion safety, hourly reconciliation, rename cleanup, dedicated WAL writing and local timing diagnostics.**
11. Macrobenchmarks and local observability. **Implemented provider timings, an opt-in in-memory p50/p95/p99 panel, read-only index diagnostics and a Windows process sampler; pixels-visible and cross-platform controlled benchmarks remain.**
12. Cross-platform CI and packaging readiness. **Implemented least-privilege three-OS validation, a manual unsigned package dry run, immutable action pins and a release checklist. Windows release compilation plus MSI/NSIS generation passed locally; hosted macOS/Linux runs, signing and clean-machine acceptance remain.**
13. Input-path responsiveness. **Implemented frame-sized query coalescing, a single in-flight search, stale-response suppression at input time, plugin-domain prefiltering and indexed recent-file ordering. Automated checks pass; interactive p50/p95/p99 confirmation remains.**
14. Native input isolation and predictable composition. **Moved search to the asynchronous Tauri command form and replaced transparent full-window backdrop blur with opaque theme surfaces. Automated validation passes; interactive key-to-paint measurement remains.**
15. Configurable global shortcut. **Implemented core-owned allowlist validation, OS registration, local persistence, rollback on failed replacement and startup fallback. Automated validation passes; manual OS conflict acceptance remains.**
16. Deterministic release evidence. **Implemented dependency inventory with lockfile hashes/licenses/checksums, recursive artifact SHA-256 generation, tests and CI/package-dry-run integration. Current unsigned Windows bundles were rebuilt and hashed; signed release evidence remains future work.**
17. Release metadata preflight. **Implemented aggregate identity/version validation across npm, Cargo, Tauri and dependency evidence, including lockfile hash freshness. Real preflight passes for 0.1.0 and is enforced in CI/package dry runs.**
18. Declared-license technical review. **Implemented boolean SPDX-style policy evaluation, missing/unknown license gates, deterministic summary generation and CI/package integration. All 612 inventoried packages pass the technical policy; legal review and complete notice texts remain required.**
19. Native icons and explicit file-search coverage. **Implemented lazy cached Windows application icons, persisted add/remove index roots, serialized background reconciliation, dynamic watcher refresh and system/cache exclusions. macOS/Linux icon parity and interactive Windows acceptance remain.**

Each phase requires acceptance criteria and evidence before the next begins.
# Phase 20 — Bounded local clipboard history

- Opt-in capture for copied text and images while Arclume is running.
- Dedicated local SQLite storage with count, age, and byte limits.
- Clipboard browser, selected image preview, restore, pause, and clear controls.
- Raw oversized images are rejected and list queries never load image payloads.
