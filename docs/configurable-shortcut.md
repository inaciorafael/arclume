# Phase 15: configurable global shortcut

Phase 15 removes the fixed launcher shortcut as a release blocker, particularly the default `Command+Space` conflict with Spotlight on macOS.

## Behavior

The settings dialog offers four bounded, cross-platform combinations:

- `Ctrl/Command + Space`;
- `Ctrl/Command + Shift + Space`;
- `Alt/Option + Space`;
- `Ctrl/Command + Alt/Option + Space`.

The core, not the webview, owns validation, OS registration and persistence. Arbitrary shortcut strings are rejected.

## Safe replacement

When the user selects a different shortcut, the core:

1. validates it against the allowlist;
2. releases the current shortcut;
3. registers the requested shortcut;
4. persists it only after registration succeeds;
5. restores the previous registration if registration or persistence fails.

On startup, the persisted shortcut is attempted first. If it is unavailable, Arclume attempts the default, updates local configuration and reports the failure in stderr. If both are unavailable, the launcher is shown so settings remain reachable.

Configuration is local at the OS-standard Arclume configuration directory in `shortcut.json`. Unknown fields and unsupported values do not become active.

## Validation

- Vue/TypeScript production build and SDK checks passed.
- Plugin tooling and manifest validation passed.
- Rust formatting and Clippy with warnings denied passed.
- 30 Rust tests passed, including shortcut allowlist and strict configuration-contract tests.

Actual conflict behavior remains OS-owned and needs manual acceptance on Windows, macOS and representative Linux desktop environments.
