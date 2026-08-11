# Search engine

Pipeline: normalize -> parse -> select providers -> collect bounded candidates -> fuzzy score -> adaptive rank -> deduplicate -> top N.

Providers expose structured candidates and never control the visual list. Exact and prefix matches precede fuzzy matches. The engine is UI-independent and queries are deterministic when history signals are disabled.

The first implementation must benchmark candidate generation before selecting an index. It must support updates and removals without a full disk scan.
