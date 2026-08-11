# Ranking

The score combines text quality, exact/prefix bonuses, result-kind prior, logarithmic frequency, decaying recency and local query-to-item affinity. Adaptive contribution is capped at 1,200 points and covered by ordering tests.

Approximate matching accepts ordered abbreviations plus a bounded Damerau-Levenshtein distance: one edit for queries of three to six characters and two edits for longer queries. Exact, acronym, prefix and substring matches retain higher score bands. File lookup uses a trigram FTS candidate index before applying the same Rust ranker, avoiding a full scan of the local file catalog on every keystroke.

Personalization is local and clearable with the confirmed `clear history` action. Tests enforce that the maximum adaptive boost cannot displace an exact match. Ranking changes require a fixed relevance dataset and latency benchmark.
