# Ecosystem readiness gate

Phase 9 evaluates optional services; it does not assume that a marketplace, account system, synchronization or telemetry is justified.

## Decision

**No-go for remote ecosystem services.** The local platform is functional on the Windows development machine, but it is not stable enough to distribute or execute untrusted community packages.

No backend, vendor account, network dependency, analytics SDK or remote data flow was added.

## Blocking evidence

| Gate | Current evidence | Status |
|---|---|---|
| Windows, macOS and Linux acceptance | Windows development only; macOS/Linux manual and CI acceptance absent | Blocked |
| Launcher latency | synchronous show request logged; pixels-visible p50/p95/p99 absent | Blocked |
| Index reliability | persistent FTS and watcher exist; overflow reconciliation remains | Blocked |
| Resource budgets | query microbench exists; cold startup, RSS and controlled disk macrobench absent | Blocked |
| Public plugin runtime | one embedded isolated POC; arbitrary entrypoints intentionally disabled | Blocked |
| Permission broker | capability declarations exist; grants, audit and privileged host calls absent | Blocked |
| Supply chain | package signature, artifact hash policy, key rotation and revocation absent | Blocked |
| Security operations | no private vulnerability disclosure channel or incident process | Blocked |

## Service order after the gate passes

### 1. Read-only plugin registry

Start with signed, immutable catalog metadata and downloadable artifacts. Installation must also work from a local package without the registry. The client must verify manifest compatibility, artifact digest and publisher signature; HTTPS alone is not package trust.

Required catalog fields should include plugin identity, version, Plugin API range, artifact digest, signature, publisher key ID, declared capabilities and platform/runtime compatibility. The exact schema must wait for the runtime artifact format.

### 2. Updates

Updates reuse the signed registry. They are opt-in during the pre-release period, show capability changes before installation and support rollback. A removed or compromised package needs a signed revocation mechanism.

### 3. Community features

Ratings, comments and publisher accounts add authentication, moderation, abuse handling, privacy obligations and operational cost. They are independent of installing local plugins and should not block the registry.

### 4. Optional synchronization

Sync must be separately enabled. Search history, clipboard content and indexed filenames are excluded by default. If preferences or non-sensitive plugin configuration later sync, use explicit field allowlists, encryption in transit and at rest, deletion/export controls and conflict semantics.

### 5. Telemetry

Do not add product analytics automatically. Development diagnostics remain local. Any future crash or usage reporting requires an explicit opt-in screen, a documented event allowlist, retention policy and a way to inspect data before sending.

## Data classification

| Data | Remote default | Reason |
|---|---|---|
| Search queries and usage history | Never | Reveals user intent and behavior |
| Clipboard content | Never | Frequently sensitive |
| Indexed paths and filenames | Never | Reveals local filesystem contents |
| UI preferences | Local | No current need for an account |
| Installed plugin IDs/versions | Local | Remote update checks can send only the requested plugin/version later |
| Crash diagnostics | Local | Upload requires separate informed opt-in |

## Release criteria

Remote registry work may begin only when all blocking gates above have owners, repeatable tests and recorded evidence. Community accounts and sync require an additional privacy/security review and concrete product demand. Vendor selection happens after workload, region, retention, availability and cost requirements exist.
