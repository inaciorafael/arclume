# ADR-005: Phase 2 fuzzy ranking

Status: accepted for baseline.

Use an internal deterministic scorer for application names: exact > prefix > substring > ordered subsequence. No fuzzy-search dependency is added.

This keeps the first relevance baseline explainable, small and benchmarkable. The cost is limited typo tolerance and no advanced token-boundary model. Revisit after collecting fixed relevance cases and 10k/100k latency measurements; replacement requires measurable relevance or latency improvement.
