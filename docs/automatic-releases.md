# Automatic releases

Arclume uses Release Please to turn conventional commits on `main` into a reviewed release pull request. Publication is automatic after that pull request is merged; ordinary feature commits never publish installers directly.

## Flow

1. Merge commits using Conventional Commit prefixes such as `fix:`, `feat:` and `feat!:`.
2. The `Automatic Release` workflow creates or updates one release pull request with the next beta version and `CHANGELOG.md`.
3. Review its version, changelog, synchronized manifests and normal CI checks.
4. Merge the release pull request.
5. Release Please creates the `v*` tag and GitHub prerelease.
6. The same workflow invokes the reusable Tauri packaging matrix and attaches Linux, Windows and macOS artifacts to that prerelease.

Version changes are synchronized across npm, Cargo, Tauri and the dependency lockfile. `npm run release:preflight` remains the packaging gate that detects drift.

## Version rules

- `fix:` increments the patch portion.
- `feat:` increments the minor portion.
- `!` or a `BREAKING CHANGE:` footer increments the major portion.
- Releases remain beta prereleases until signing and clean-machine acceptance gates are complete.

The workflow uses the repository `GITHUB_TOKEN`; no personal access token or additional secret is required. The packaging job is called directly after release creation because GitHub does not recursively trigger tag workflows created with that token. A manually pushed `v*` tag still invokes the same packaging workflow as a fallback.
