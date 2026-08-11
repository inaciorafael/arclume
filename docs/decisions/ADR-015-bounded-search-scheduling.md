# ADR-015: Bound search scheduling and prefilter isolated providers

- Status: accepted
- Date: 2026-08-10

## Context

Starting every provider for every key event created redundant concurrent work. The capability-free plugin POC was especially expensive because its isolation model intentionally starts a short-lived process. Empty-query ordering also lacked a database index and could delay the first typed query.

## Decision

Coalesce frontend query changes for at most 16 ms, allow one backend search in flight and retain only the latest pending query. Advance request identity at input time so stale responses cannot render. Apply a deterministic provider-domain prefilter before spawning the hello-world host, without weakening its process boundary for matching queries. Add an index for recent-file ordering.

## Consequences

- Typing cannot create an unbounded queue of process launches or IPC calls.
- Intermediate queries may be intentionally skipped, but the latest query always runs.
- Relevant plugin queries still pay the isolation cost; a future general plugin system will need persistent supervised hosts or an asynchronous provider protocol.
- The new SQLite index consumes additional disk space and adds a small write cost in exchange for bounded recent-item lookup.
