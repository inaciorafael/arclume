# ADR-014: CI matrix and unsigned packaging dry runs

- Status: accepted
- Date: 2026-08-10

## Context

Arclume has a cross-platform implementation baseline, but local Windows success does not prove compilation or packaging on macOS and Linux. Publishing installers would also be premature because signing, clean-machine acceptance and a private disclosure process are not ready.

## Decision

Use two least-privilege GitHub Actions workflows:

1. continuous validation on Ubuntu 22.04, Windows and macOS, ending in a release compilation without bundles;
2. a manual, unsigned packaging dry run on the same matrix with no artifact upload or release step.

Pin Node.js, Rust and external GitHub actions. Deny Clippy warnings in CI. Keep package publication and signing out of these workflows until the release gates are designed and accepted.

## Consequences

- Cross-platform regressions can be detected before release work begins.
- Packaging can be exercised without presenting unsigned artifacts as distributable software.
- Hosted-runner execution and OS acceptance are still required; workflow definitions alone are not evidence that other platforms pass.
- A later ADR must define signing, provenance, secret custody and publication before production distribution is enabled.
