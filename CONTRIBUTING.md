# Contributing

1. Keep changes inside the active roadmap phase.
2. Add behavior to Rust when it involves search, indexing, ranking, storage, filesystem, execution or OS integration.
3. Keep Vue responsible for presentation and input orchestration.
4. Document platform-specific behavior and test it on the affected operating system.
5. Include measurements for performance claims and tests for ranking or parser changes.
6. Run the validation commands from the README before opening a change.

Dependencies require a short justification, a stable version, an official source and a note about bundle, memory and maintenance impact.

For Plugin API changes, also run `npm run sdk:typecheck`, `npm run plugin:test` and `npm run plugin:validate -- plugins/hello-world/plugin.json`. Create contract-development fixtures with `npm run plugin:create -- <kebab-case-id>`; generated entrypoints are not executable by the desktop app yet. See `docs/plugin-sdk.md`.
