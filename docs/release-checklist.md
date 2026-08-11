# Release checklist

## Candidate preparation

- [ ] Choose the release version and update every version-bearing manifest consistently.
- [ ] Generate fresh inventory and pass `npm run release:preflight`.
- [ ] Review the diff, locked dependency changes, licenses and security-sensitive capabilities.
- [ ] Pass `npm run license:audit` and complete human review of applicable license texts/notices.
- [ ] Run `npm ci` from a clean checkout.
- [ ] Pass `npm run ci:frontend`, Rust formatting, Clippy with warnings denied and all tests.
- [ ] Record a release-specific dependency inventory or SBOM.

## Platform acceptance

- [ ] GitHub CI succeeds on Ubuntu 22.04, Windows and macOS.
- [ ] The manual package dry run succeeds on all three hosted operating systems.
- [ ] Install, first launch, global shortcut, search, indexing, actions and uninstall pass on clean machines.
- [ ] Upgrade from the previous supported version preserves compatible local state.
- [ ] macOS shortcut conflicts are resolved through a configurable shortcut before release.

## Trust and operations

- [ ] Signing identities are provisioned outside the repository with least-privilege access.
- [ ] Windows packages are signed and their signatures verified on a clean machine.
- [ ] macOS bundles are signed, notarized and Gatekeeper-verified.
- [ ] Linux package checksums and the selected trust/distribution model are documented.
- [ ] A private vulnerability disclosure channel and incident owner exist.
- [ ] Release notes cover compatibility, migrations, known limitations and rollback.

## Publication

- [ ] Compute and independently verify SHA-256 checksums for final artifacts.
- [ ] Publish only the exact tested, signed artifacts; do not rebuild after acceptance.
- [ ] Confirm download, signature, checksum and installation from the public location.
- [ ] Retain provenance and CI evidence for the released version.

No item in this checklist is implied complete merely because an unsigned local installer was generated.
