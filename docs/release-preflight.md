# Phase 17: release metadata preflight

Phase 17 prevents packaging when application identity, version or dependency evidence has drifted.

## Command

Generate fresh dependency evidence, then run:

```shell
npm run inventory
npm run release:preflight
```

The preflight verifies:

- `package.json` name and SemVer version;
- `package-lock.json` top-level and root-package identity/version;
- the `[package]` name/version in `src-tauri/Cargo.toml`;
- the Arclume package version in `src-tauri/Cargo.lock`;
- Tauri product name, version and application identifier;
- dependency-inventory project identity/version;
- inventory SHA-256 values against the current npm and Cargo lockfiles.

All mismatches are reported in one run. The command exits non-zero if any invariant fails or if fresh inventory evidence is absent.

## Automation

Cross-platform CI generates the dependency inventory, tests the preflight parser/validator and then validates release metadata before Rust compilation. The manual package dry run repeats the real preflight before producing checksums.

## Validation

- 4 preflight unit tests passed.
- Tests cover Cargo section isolation, synchronized state, aggregate mismatch reporting and invalid SemVer.
- The real project preflight passed for `Arclume 0.1.0` using freshly generated lockfile evidence.

This checks internal consistency. It does not choose the next version, authorize publication or prove upgrade compatibility.
