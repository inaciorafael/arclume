# ADR-019: Fail packaging on release metadata drift

- Status: accepted
- Date: 2026-08-11

## Context

Arclume's version is represented in npm, Cargo and Tauri metadata. Dependency evidence is generated separately. A partial version update or stale inventory could produce installers whose filename, executable metadata and review evidence disagree.

## Decision

Add a standard-library-only preflight that validates application identity and SemVer across all manifests and lockfiles, then verifies inventory lock hashes against current files. Run it after fresh inventory generation in continuous validation and package dry runs. Report every mismatch before failing.

## Consequences

- Partial version bumps and stale evidence fail before packaging.
- Release preparation has an explicit machine-checkable identity contract.
- Inventory generation becomes a required predecessor of the preflight.
- The tool validates consistency, not release policy, signing, upgrade behavior or whether a chosen version is semantically appropriate.
