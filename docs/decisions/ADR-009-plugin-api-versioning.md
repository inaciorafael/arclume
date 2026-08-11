# ADR-009: Plugin API versioning and tooling

Status: accepted for the pre-release SDK.

Use an integer `pluginApiVersion` for manifest/IPC compatibility and semantic versions for the SDK package and individual plugins. Unknown fields and unsupported API integers are rejected to prevent silent contract drift.

Keep the SDK private while runtime isolation, permissions and installation remain unresolved. Provide a dependency-free validator and scaffolder so the contract can be tested now without treating generated code as trusted or executable.

The CLI intentionally duplicates a small set of schema checks instead of adding a runtime JSON Schema dependency. Tests and the checked-in hello-world fixture guard the two representations. If the schema grows, replace this duplication with generated validation before a public release.
