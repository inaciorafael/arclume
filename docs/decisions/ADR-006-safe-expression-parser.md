# ADR-006: Safe expression parser

Status: accepted.

Use a small Pratt parser owned by the core for the initial calculator grammar. Reject JavaScript evaluation, shell evaluation and general scripting runtimes.

The internal parser has no dependency cost, deterministic precedence and a deliberately narrow attack surface. The trade-off is responsibility for numeric semantics and tests. If future requirements include units inside expressions, constants, complex numbers or localization, compare mature parser crates before extending the grammar ad hoc.
