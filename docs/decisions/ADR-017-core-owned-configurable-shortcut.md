# ADR-017: Core-owned configurable global shortcut

- Status: accepted
- Date: 2026-08-10

## Context

The fixed `CmdOrControl+Space` shortcut conflicts with common OS behavior, especially Spotlight on macOS. Letting the webview persist arbitrary strings would create inconsistent state and could leave the launcher unreachable after a failed registration.

## Decision

Keep shortcut validation, registration and persistence in Rust. Expose a small IPC contract for reading and selecting one of four allowed combinations. Replace registrations transactionally and restore the previous shortcut on failure. At startup, fall back to the default and show the launcher if no shortcut can be registered.

## Consequences

- Users can resolve platform conflicts without editing files.
- A failed change does not intentionally leave the app without its previous shortcut.
- The bounded allowlist is simpler and safer than a free-form shortcut recorder, but supports fewer combinations.
- Manual compositor/desktop-environment acceptance remains necessary because global-shortcut availability is controlled by the OS.
