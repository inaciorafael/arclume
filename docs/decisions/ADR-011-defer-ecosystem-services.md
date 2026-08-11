# ADR-011: Defer remote ecosystem services

Status: accepted.

Do not implement a plugin marketplace, community backend, account system, synchronization or telemetry in Phase 9 because the local cross-platform and untrusted-plugin security gates are not satisfied.

Preserve architectural compatibility by keeping local plugin installation independent of any future store, using versioned manifests and requiring future catalog artifacts to be content-addressed and signed. Do not select a cloud vendor before traffic, storage, region, retention, moderation and availability requirements are known.

This decision costs short-term ecosystem momentum but avoids operating a service that cannot safely deliver executable community content. It also prevents premature vendor lock-in and collection of personal launcher data without a demonstrated product requirement.
