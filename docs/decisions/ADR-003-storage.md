# ADR-003: Storage

Status: proposed.

Separate settings, behavioral history and search index lifecycles. SQLite is the baseline candidate for transactional local data. Secrets must use OS credential storage rather than ordinary rows. Schema migrations are forward-only, versioned and tested against a copied database.

The selected search index may use a separate optimized store if benchmark evidence justifies the additional recovery and migration complexity.
