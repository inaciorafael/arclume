# Architecture

## Boundaries

```text
Vue presentation -> one typed Tauri request -> Rust use case -> providers -> ranked results
                                                        -> platform adapters
                                                        -> local storage
                                                        -> permission broker
```

The UI renders results, owns transient selection state and rejects stale responses by `queryId`. Rust owns searchable entities, provider orchestration, ranking, execution, persistence and OS access.

## Core contracts

The future domain model contains `SearchItem`, `SearchResult`, `Action`, `ProviderResult`, `QueryContext` and `Preview`. These are Rust domain types, serialized into explicit IPC DTOs. Domain modules must not depend on Tauri window types or Vue concepts.

Provider failures are isolated and reported as diagnostics. A slow optional provider must not block local results. Results are bounded before IPC; the frontend never requests one item per IPC call.

## Concurrency

Every request carries a monotonic `queryId`. Providers receive a cancellation context when supported. The UI discards any response older than its latest query even if cancellation races.

## Evolution

Start as a modular monolith. Introduce a separate process only for plugin isolation or another measured reliability boundary. See the ADRs in `docs/decisions`.
