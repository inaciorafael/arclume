# ADR-016: Asynchronous search dispatch and opaque composition

- Status: accepted
- Date: 2026-08-10

## Context

Input remained perceptibly slow after redundant searches and plugin launches were bounded. Search still crossed a synchronous native command boundary, and the transparent window continuously blurred the desktop behind the complete launcher.

## Decision

Expose search as an asynchronous Tauri command using an owned query and `Result<SearchResponse, String>`. Disable native transparency and remove the full-window backdrop filter, using opaque theme surfaces instead.

## Consequences

- Search work no longer relies on the synchronous command form.
- Desktop composition cost becomes more predictable during typing and result replacement.
- The live glass effect is removed. Visual identity remains, but translucency can return only through a measured optional/platform-native implementation.
- SQLite and plugin work are still blocking operations inside the asynchronous task. The single-flight frontend scheduler bounds concurrency; a later general plugin runtime should adopt supervised persistent workers.
