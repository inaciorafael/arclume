# ADR-020: Gate declared license expressions without claiming legal approval

- Status: accepted
- Date: 2026-08-11

## Context

The dependency inventory contains compound SPDX-style expressions, approved exceptions and a few legacy slash forms. Substring matching would incorrectly reject valid permissive alternatives or accept an unapproved mandatory branch. Missing metadata must also be visible before packaging.

## Decision

Evaluate expressions as boolean policy using `AND`, `OR`, parentheses and `WITH`. Maintain a small explicit allowlist for identifiers/exceptions observed and reviewed in the current dependency set. Fail on missing, invalid, unknown or wholly unapproved expressions. Generate a deterministic summary with an explicit non-legal-advice disclaimer and run the gate after fresh inventory generation in CI and package dry runs.

## Consequences

- Metadata regressions and newly introduced license families require explicit review.
- Multi-license alternatives are evaluated according to their boolean meaning.
- A passing audit is technical evidence only; it does not satisfy attribution, source-offer, notice-text or legal-review obligations.
- Policy updates become reviewed source changes rather than silent acceptance of new identifiers.
