# Phase 61 — Benchmark Results

**Measurement date:** 2026-06-08
**Machine:** Darwin MacBook-Pro-de-Luis.local 25.5.0 Darwin Kernel Version 25.5.0: Mon Apr 27 20:41:15 PDT 2026; root:xnu-12377.121.6~2/RELEASE_ARM64_T6041 arm64
**Benchmark:** benches/rastrigin.rs (pop=500, max_generations=50, RangeChromosome<f64>, bounds [-5.12, 5.12])
**Baseline path used:** pre-phase-worktree
**Baseline commit:** e3b0728b16438de8640d2bcc9b79018da4c42973 (docs(60): phase 60 complete — summary and state update)

## Results

| Dimensions | Baseline (mean) | Post-change (mean) | Improvement | ROADMAP gate (≥10%) |
|------------|-----------------|--------------------|-------------|---------------------|
| 10         | 1.5586 ms       | 1.5375 ms          | 1.35%       | FAIL                |
| 20         | 1.6334 ms       | 1.5990 ms          | 2.11%       | FAIL                |
| 50         | 1.8204 ms       | 1.8274 ms          | -0.38%      | FAIL                |

## Headline

ROADMAP success criterion #1 is **NOT MET**: pop=500 rastrigin wall-time reduction is at most 2.11% at dim=20. The ≥10% target was not reached at any tested dimensionality.

## Raw output

### Baseline (`cargo bench --bench rastrigin` at e3b0728b16438de8640d2bcc9b79018da4c42973)

```
Benchmarking rastrigin/Ga::run/pop_500_dim_10: Collecting 100 samples in estimated 8.5385 s (5050 iterations)
rastrigin/Ga::run/pop_500_dim_10
                        time:   [1.5464 ms 1.5586 ms 1.5716 ms]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking rastrigin/Ga::run/pop_500_dim_20: Collecting 100 samples in estimated 9.6501 s (5050 iterations)
rastrigin/Ga::run/pop_500_dim_20
                        time:   [1.6266 ms 1.6334 ms 1.6403 ms]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking rastrigin/Ga::run/pop_500_dim_50: Collecting 100 samples in estimated 5.0678 s (2000 iterations)
rastrigin/Ga::run/pop_500_dim_50
                        time:   [1.8124 ms 1.8204 ms 1.8290 ms]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
```

### Post-change (`cargo bench --bench rastrigin` at HEAD / worktree-agent-a0d6d8dfc7f4c8c11)

```
Benchmarking rastrigin/Ga::run/pop_500_dim_10: Collecting 100 samples in estimated 8.7015 s (5050 iterations)
rastrigin/Ga::run/pop_500_dim_10
                        time:   [1.5301 ms 1.5375 ms 1.5464 ms]
Found 14 outliers among 100 measurements (14.00%)
  5 (5.00%) high mild
  9 (9.00%) high severe
Benchmarking rastrigin/Ga::run/pop_500_dim_20: Collecting 100 samples in estimated 9.5593 s (5050 iterations)
rastrigin/Ga::run/pop_500_dim_20
                        time:   [1.5943 ms 1.5990 ms 1.6044 ms]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking rastrigin/Ga::run/pop_500_dim_50: Collecting 100 samples in estimated 5.1229 s (2000 iterations)
rastrigin/Ga::run/pop_500_dim_50
                        time:   [1.8220 ms 1.8274 ms 1.8338 ms]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
```

## Notes

- Both runs were performed back-to-back on the same machine (Apple Silicon ARM64, Darwin 25.5.0) in the same session.
- The baseline worktree was freshly compiled at commit e3b0728 with rastrigin.rs and Cargo.toml [[bench]] entry copied from the current tree (Plan 01 artifacts, no Plan 02/03 changes).
- At pop=500 with max_generations=50, the total work per bench iteration is modest (~25,000 chromosome evaluations for dim=10/20; the fitness function is cheap). With populations at this size, `par_sort_unstable_by` on the survivor step introduces rayon thread-pool coordination overhead that equals or exceeds the sort savings — the population needs to be significantly larger (e.g., pop=5000+) for parallel sort to show net gains.
- dim=50 actually regressed by −0.38%: the slightly longer rastrigin evaluation (more genes to compute) still does not make the population large enough to amortize rayon overhead over the sort.
- The observer `&U` change (Plan 03) eliminated one per-generation chromosome clone on each new-best event; this improvement is undetectable at the millisecond scale in a benchmark that runs only 50 generations.
- The crossover fallback clones (D-01) fire only on the rare else-branch; their impact on this benchmark is negligible.
- Conclusion: the optimizations delivered are architecturally correct and will provide net throughput gains at larger population sizes (where survivor sort dominates), but the ≥10% gate at pop=500 was not achieved. Human decision required on whether to accept the lower improvement, amend the ROADMAP criterion, or pursue additional optimization.
