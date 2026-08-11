# Phase 16: deterministic release evidence

Phase 16 adds local, reproducible evidence for dependency review and artifact integrity without introducing a publishing path.

## Dependency inventory

```shell
npm run inventory
```

The command writes ignored `artifacts/dependency-inventory.json` containing:

- project name and version;
- SHA-256 of `package-lock.json` and `src-tauri/Cargo.lock`;
- sorted npm package name, version, license, integrity and development/optional flags;
- sorted Cargo crate name, version, license, registry source and lockfile checksum.

The generator uses only Node.js standard-library modules. Rust metadata is read with the project toolchain through `rustup run 1.97.1 cargo metadata --locked`. It includes no timestamp, machine path or build identifier, so identical inputs produce identical JSON.

The Windows validation inventory contains 90 npm packages and 522 registry crates. No package lacked declared license metadata. This is evidence for review, not legal approval of the license set.

## Artifact checksums

```shell
npm run checksums -- --directory src-tauri/target/release/bundle
```

The command recursively hashes bundle files, sorts paths deterministically and writes ignored `artifacts/SHA256SUMS`. It fails when the directory is absent or empty.

Current unsigned Windows dry-run evidence:

| Bundle | SHA-256 |
|---|---|
| `Arclume_0.1.0_x64_en-US.msi` | `1f674f999308af50a1d2187a20ab2f0a42da2c0774f57cb33cb0688b06cf2704` |
| `Arclume_0.1.0_x64-setup.exe` | `0b9b082f68b2f8e6fbb9ab21d4df25005813955ce203911abb9e2b11933746f2` |

These hashes identify unsigned local dry-run artifacts only. They must not be reused for a later signed release because signing changes the files.

## Automation

The three-platform CI tests both evidence tools and generates the dependency inventory. The manual package dry run generates inventory plus checksums after bundling and includes checksums in the job summary. Neither workflow uploads or publishes artifacts.

## Validation

- 3 deterministic dependency-inventory tests passed.
- 1 recursive checksum test passed.
- A complete inventory was generated successfully.
- Current Windows MSI and NSIS bundles were rebuilt and hashed successfully.
