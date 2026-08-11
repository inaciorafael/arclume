# Release readiness

Phase 12 establishes reproducible validation and unsigned packaging. It does not authorize public distribution.

## Automated workflows

- `.github/workflows/ci.yml` runs on pushes, pull requests and manual dispatch across Ubuntu 22.04, Windows and macOS. It validates the frontend, private plugin SDK, plugin manifest, Rust formatting, Clippy, tests and a release compilation without bundles.
- `.github/workflows/package-dry-run.yml` is manual-only. It builds native unsigned bundles on the same operating-system matrix, records their paths in the job summary and deliberately does not upload, sign, publish or create a release.
- Workflow permissions are restricted to `contents: read`.
- External actions are pinned to immutable commit SHAs. Package-manager caching is disabled to avoid sharing a mutable dependency cache across untrusted pull-request validation.
- CI tests and generates a deterministic dependency inventory; the manual package dry run also generates SHA-256 checksums after bundling.

Node.js 22.12.0 and Rust 1.97.1 are explicit in both workflows. JavaScript and Rust dependency graphs remain locked by `package-lock.json` and `src-tauri/Cargo.lock`.

## Local evidence (2026-08-10)

The Windows validation completed with:

- frontend production build and Vue/TypeScript type checking;
- private SDK type checking;
- 3 plugin tooling tests and validation of the checked-in hello-world manifest;
- `cargo fmt --check`;
- Clippy for all targets and features with warnings denied;
- 26 Rust tests passing;
- optimized Tauri executable at `src-tauri/target/release/arclume.exe`;
- unsigned MSI: `Arclume_0.1.0_x64_en-US.msi` (5,267,456 bytes);
- unsigned NSIS installer: `Arclume_0.1.0_x64-setup.exe` (3,026,718 bytes).

These files are local test outputs under ignored `target` storage. Their existence proves the Windows packaging path, not installation safety, upgrade behavior, signing or compatibility on another machine.

## Local commands

```shell
npm ci
npm run ci:frontend
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
npm run tauri build -- --ci --no-bundle
npm run tauri build -- --ci --no-sign
```

The selected toolchain must resolve before any obsolete system Cargo installation. On this development machine, Rust 1.73 cannot parse the project's Rust 2024 edition.

## Gates before public distribution

1. Run both GitHub workflows on hosted runners and retain evidence for all three operating systems.
2. Perform clean-machine installation, launch, shortcut, indexing, uninstall and upgrade acceptance tests per OS.
3. Define platform signing custody and rotation: Authenticode for Windows, Developer ID/notarization for macOS, and the chosen Linux package trust model.
4. Configure a private vulnerability disclosure channel and incident owner.
5. Review third-party notices and produce a release-specific dependency/SBOM record.
6. Keep versions synchronized across `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
7. Publish only immutable, signed artifacts with checksums and release notes after the previous gates pass.

Until these gates pass, package dry runs are engineering evidence only and must not be distributed as production builds.
