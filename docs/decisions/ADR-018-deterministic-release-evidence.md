# ADR-018: Deterministic dependency and artifact evidence

- Status: accepted
- Date: 2026-08-10

## Context

Release readiness requires a reviewable dependency inventory and checksums tied to exact artifacts. The local npm installation does not expose a usable SBOM subcommand, and adding a large third-party generator would itself expand the supply chain.

## Decision

Generate a minimal deterministic JSON inventory directly from locked npm data, locked Cargo checksums and Cargo metadata from Rust 1.97.1. Generate conventional SHA-256 lines for bundle files with a separate standard-library-only tool. Keep outputs ignored and ephemeral; exercise both tools in CI and the manual package dry run without upload or publication.

## Consequences

- Reviewers can tie dependency metadata to exact lockfile hashes.
- Bundle integrity can be verified independently using standard SHA-256 tools.
- The inventory is an Arclume-specific schema, not CycloneDX or SPDX; it must not be represented as a standards-compliant SBOM.
- License presence is not legal approval, and signed release artifacts require new checksums after signing.
