# Phase 18: declared-license technical review

Phase 18 adds a deterministic technical gate over license metadata from the dependency inventory.

## Commands

```shell
npm run inventory
npm run license:audit:test
npm run license:audit
```

The audit parses boolean license expressions with `AND`, `OR`, parentheses and approved `WITH` exceptions. Known legacy slash forms from current metadata are normalized. This matters because `MIT OR Apache-2.0 OR LGPL-2.1-or-later` has an approved permissive alternative and must not be treated like a mandatory LGPL-only dependency.

The command fails when:

- license metadata is absent;
- an expression is syntactically invalid;
- an identifier or exception is not in the reviewed technical policy;
- every branch requires an unapproved license, including strong-copyleft-only expressions.

It writes ignored `artifacts/THIRD_PARTY_LICENSE_SUMMARY.md` with expression counts and a mandatory disclaimer.

## Current evidence

- 612 packages reviewed: 90 npm packages and 522 Cargo registry crates;
- no missing license metadata;
- no expression requiring an unapproved license under the technical policy;
- 4 evaluator/audit tests passed.

The current set includes MPL-2.0 dependencies and several multi-license expressions. Their obligations still require release-specific review and inclusion of applicable full license texts/notices.

## Boundaries

This gate is not legal advice, does not determine copyright ownership, does not inspect source headers and does not generate complete third-party notices. A passing result means only that declared expressions match the reviewed machine policy. Publication remains blocked until a qualified release review confirms obligations and assembles required texts.
