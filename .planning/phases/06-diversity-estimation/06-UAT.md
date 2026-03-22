---
status: complete
phase: 06-diversity-estimation
source: 06-01-SUMMARY.md, 06-02-SUMMARY.md
started: 2026-03-20T20:00:00Z
updated: 2026-03-20T20:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. diversity field on GenerationStats
expected: After running a GA, each GenerationStats value in the history has a `diversity` field that is >= 0.0 and equals `fitness_std_dev`. You can access it as `stats.diversity`.
result: pass

### 2. cargo test passes (all features)
expected: Running `cargo test` and `cargo test --features serde` both complete with 0 failures. The new tests `test_ga_stats_diversity_populated` and `ga_extension_triggers_on_diversity` are included and pass.
result: pass

### 3. backward-compatible checkpoint loading
expected: A JSON checkpoint produced before the `diversity` field existed (i.e., without a `diversity` key in `GenerationStats`) deserializes without error, and the resulting struct has `diversity = 0.0`. The serde test `serde_generation_stats_backward_compat` covers this.
result: pass

### 4. extension triggers on low diversity
expected: When a population has uniform fitness (all chromosomes identical fitness → diversity = 0.0) and the extension threshold is set > 0.0 (e.g., 1.0), the extension operator fires during the run. The test `ga_extension_triggers_on_diversity` validates this path.
result: pass

### 5. dynamic mutation reads gen_stats.diversity
expected: The dynamic mutation code path no longer calls `compute_cardinality` — it reads `gen_stats.diversity` instead. The log message says "diversity" (not "cardinality"). This is an internal wiring change; `cargo test` passing is the observable confirmation.
result: pass

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0

## Gaps

[none yet]
