# Arclume

Arclume is a fast, keyboard-first, cross-platform launcher built with Tauri 2, Rust, Vue 3 and TypeScript. It borrows the universal-search concept—not the branding or visual identity—of existing launchers.

## Current scope

Arclume searches applications, indexed files and actions, keeps an optional bounded clipboard history, captures screenshots and supports explicit online Portuguese dictionary lookups without sending keystrokes over the network.

Dictionary usage, privacy and cache limits are documented in [Portuguese dictionary](docs/portuguese-dictionary.md).

## Prerequisites

- Node.js 22.12 or newer supported by the selected Vite release
- Rust stable and the platform prerequisites from the [Tauri documentation](https://v2.tauri.app/start/prerequisites/)
- Windows: Microsoft C++ Build Tools and WebView2
- macOS: Xcode command-line tools
- Linux: the WebKitGTK and system packages required by Tauri

This Windows machine also has an obsolete Chocolatey Rust 1.73 installation before rustup in `PATH`. If `cargo --version` does not report 1.97.1, put `%USERPROFILE%\.cargo\bin` before `C:\ProgramData\chocolatey\bin` or invoke the rustup toolchain explicitly. Do not build with Cargo 1.73; current dependencies use Rust edition 2024.

## Development

```shell
npm install
npm run tauri dev
```

The default shortcut is `Ctrl+Space` on Windows/Linux and `Command+Space` on macOS. Change it in `Ctrl+,` → **Global shortcut** if it conflicts with the operating system.

## Validation

```shell
npm run build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run inventory:test
npm run checksums:test
npm run release:preflight:test
npm run inventory
npm run release:preflight
npm run license:audit:test
npm run license:audit
```

See [architecture](docs/architecture.md), [application search](docs/application-search.md), [file search](docs/file-search.md), [native icons and index roots](docs/native-icons-and-index-roots.md), [action engine](docs/action-engine.md), [history and ranking](docs/history-ranking.md), [interface and accessibility](docs/interface-accessibility.md), [isolated plugin POC](docs/plugin-poc.md), [plugin SDK workflow](docs/plugin-sdk.md), [ecosystem readiness](docs/ecosystem-readiness.md), [reliability hardening](docs/reliability-hardening.md), [local observability](docs/local-observability.md), [input responsiveness](docs/input-responsiveness.md), [native input isolation](docs/native-input-isolation.md), [configurable shortcut](docs/configurable-shortcut.md), [release evidence](docs/release-evidence.md), [release preflight](docs/release-preflight.md), [automatic releases](docs/automatic-releases.md), [license review](docs/license-review.md), [release readiness](docs/release-readiness.md), [release checklist](docs/release-checklist.md), [dependency baseline](docs/dependencies.md), [roadmap](docs/roadmap.md), and [performance budgets](docs/performance.md).
