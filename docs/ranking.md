# Ranking

The score combines text quality, exact/prefix bonuses, result-kind prior, logarithmic frequency, decaying recency and local query-to-item affinity. Adaptive contribution is capped at 1,200 points and covered by ordering tests.

Personalization is local and clearable with the confirmed `clear history` action. Tests enforce that the maximum adaptive boost cannot displace an exact match. Ranking changes require a fixed relevance dataset and latency benchmark.
