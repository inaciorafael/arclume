# Phase 14: native input isolation and predictable composition

Phase 14 addresses latency where the typed character itself can appear late, rather than only delaying search results.

## Confirmed architectural risks

- The Tauri `search` command was synchronous while performing SQLite access, history lookup, ranking and potentially isolated-process IPC. That work shared the native command dispatch path with window interaction.
- The entire undecorated window was transparent and applied a 24 px backdrop blur plus saturation. Every result update could force expensive desktop recomposition, independently of Vue's measured DOM commit time.

## Changes

- `search` is now an asynchronous Tauri command with an owned query payload and an explicit `Result` contract.
- The frontend IPC response shape remains unchanged; command errors continue through the existing rejection path.
- Native window transparency is disabled.
- The launcher uses opaque dark/light surfaces and no continuous backdrop filter. Shape, spacing, hierarchy and theme behavior remain unchanged.

This deliberately trades live desktop translucency for predictable input and rendering latency. A future glass effect should use measured, platform-native composition and must remain optional.

## Validation

- Vue/TypeScript production build passed.
- SDK type checking, three plugin tooling tests and manifest validation passed.
- Rust formatting and Clippy with warnings denied passed.
- All 28 Rust tests passed.

Automated checks establish contract and regression safety. They do not prove perceived key-to-paint latency; that requires interactive use and OS-level frame measurement.
