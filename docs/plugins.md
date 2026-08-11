# Plugin architecture

An isolated hello-world provider POC is implemented for Phase 6. Phase 7 adds a private TypeScript SDK, JSON Schema, manifest validator and safe scaffolder around API v1. It is not yet a public or installable plugin ecosystem.

Plugins may contribute providers, actions, commands, previews and settings. They cannot mutate the result list or invoke privileged operations directly. All privileged calls pass through the Rust permission broker.

The Phase 6 subprocess protocol is accepted only as a measured POC in ADR-008. Runtime comparison, arbitrary entrypoint loading and permission grants remain blocked until ADR-002 is resolved with further evidence.
