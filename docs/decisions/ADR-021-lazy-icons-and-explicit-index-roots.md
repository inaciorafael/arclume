# ADR-021: Load native icons lazily and expand indexing only by explicit roots

- Status: accepted
- Date: 2026-08-11

## Context

Application results displayed only category placeholders. File search covered Documents, Desktop and Downloads, so paths elsewhere on `C:` were correctly absent but the scope was not user-configurable. Indexing an entire system drive by default would create unacceptable privacy, startup, storage and watcher costs.

## Decision

Load Windows application icons after results render, sequentially and with an in-memory cache; preserve fallbacks on failure and on platforms without an accepted loader. Expose persisted add/remove operations for absolute index roots. Reconcile in a serialized background path and refresh watched roots dynamically. Permit drive roots only when explicitly selected and exclude known system/cache/build directories.

## Consequences

- Windows application results can display recognizable native icons without enlarging every search response.
- Users control search coverage and can add `C:\Projects` or another missing location.
- A full drive remains potentially expensive even with exclusions; narrower roots are recommended.
- Icon parity on macOS/Linux and native folder-picker UX remain future platform work.
