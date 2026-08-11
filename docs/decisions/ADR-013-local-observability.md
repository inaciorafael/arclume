# ADR-013: Local-only performance observability

Status: accepted.

Return bounded provider timings with each search response and keep frontend percentile samples in memory only. Provide explicit, read-only diagnostic commands for index storage and Windows process memory.

Do not introduce an analytics SDK, remote collector or persistent query log. Do not label backend, IPC or Vue `nextTick` measurements as pixels-visible latency.

The per-response payload adds a small fixed set of numbers. This is preferred to hidden global instrumentation because the user can inspect provider cost at the point of use. If serialization overhead becomes measurable, gate detailed fields behind a local debug setting while retaining total time.
