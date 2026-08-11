# Dependency baseline

Verified on 2026-08-10. Exact JavaScript versions are locked in `package-lock.json`; exact Rust transitive versions are locked in `Cargo.lock`.

| Dependency | Selected | Purpose |
|---|---:|---|
| Rust | 1.97.1 stable | Native core and OS integration |
| Node.js | 22.12.0 | Local build runtime; not shipped with the app |
| Vue | 3.5.40 | Presentation and transient UI state |
| Vite | 8.1.5 | Frontend build pipeline |
| `@vitejs/plugin-vue` | 6.0.8 | Official Vue SFC integration |
| TypeScript | 6.0.3 | Static frontend contracts |
| `vue-tsc` | 3.3.9 | Vue template type checking |
| `@tauri-apps/api` | 2.11.1 | Typed webview-to-core API |
| `@tauri-apps/cli` | 2.11.4 | Desktop build and packaging |
| `tauri-plugin-global-shortcut` | Cargo lock | Official native global shortcut support |
| `rusqlite` | 0.40.2 | SQLite FTS5 persistence with bundled SQLite |
| `notify` | 8.2.0 | Native cross-platform filesystem notifications |
| `directories` | 6.0.0 | OS-standard data, configuration and user directories |
| `criterion` | 0.8.2 | Development-only statistical benchmark harness |
| `tauri-plugin-clipboard-manager` | 2.3.2 | Official write-only clipboard integration for action results |
| `actions/checkout` | commit `3d3c42e` (v7.0.1) | CI-only repository checkout, pinned immutably |
| `actions/setup-node` | commit `8207627` (v7.0.0) | CI-only Node provisioning, pinned immutably |

TypeScript 7.0.2 was evaluated because it was the latest stable release. It is not selected: `vue-tsc` 3.3.9 accesses `typescript/lib/tsc`, which TypeScript 7 does not export. The resulting build fails with `ERR_PACKAGE_PATH_NOT_EXPORTED`. TypeScript 6.0.3 is the newest stable compatible line verified by an actual build attempt.

The development server uses fixed port 1437 (HMR 1438). The scaffold default 1420 was rejected after a real conflict with another local Vite project; Tauri and Vite configurations must remain synchronized.

The scaffold's opener plugin was removed because Phase 1 does not open external resources. This keeps the capability surface smaller.
