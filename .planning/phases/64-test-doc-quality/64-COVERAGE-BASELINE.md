# Phase 64 — Coverage Baseline Report

**Generated:** 2026-06-11
**Tool:** cargo-llvm-cov 0.8.7
**Command:** `cargo llvm-cov --all-features --ignore-filename-regex 'tests/' --summary-only`

---

## Tool Versions

```
cargo-llvm-cov 0.8.7
rustc 1.94.1 (e408947bf 2026-03-25)
```

---

## Per-File Coverage Summary

```
                                                           Lines             Branches              Regions
Filename                                               Count  Missed      %  Count  Missed      %  Count  Missed      %
...
TOTAL                                                  25942    3702  85.73%   2055     401  80.49%  17787    2957  83.38%
```

(Full per-file detail captured in the tables below. The `--summary-only` flag produces the table shown in the per-module breakdowns.)

---

## src/engines/ Per-Module Breakdown

| Module | Lines | Missed | % |
|--------|------:|-------:|----:|
| engines/alps/configuration.rs | 85 | 5 | 94.12% |
| engines/alps/engine.rs | 328 | 31 | 90.55% |
| engines/cellular/configuration.rs | 58 | 7 | 87.93% |
| engines/cellular/engine.rs | 303 | 18 | 94.06% |
| engines/cma/configuration.rs | 60 | 20 | 66.67% |
| engines/cma/engine.rs | 1129 | 118 | 89.55% |
| engines/de/configuration.rs | 42 | 0 | 100.00% |
| engines/de/crossover.rs | 63 | 0 | 100.00% |
| engines/de/engine.rs | 277 | 22 | 92.06% |
| engines/de/mutation.rs | 324 | 8 | 97.53% |
| engines/eda/configuration.rs | 23 | 8 | 65.22% |
| engines/eda/engine.rs | 733 | 95 | 87.04% |
| engines/ga.rs | 2581 | 839 | 67.49% |
| engines/gp/chromosome.rs | 118 | 60 | **49.15%** |
| engines/gp/configuration.rs | 135 | 62 | **54.07%** |
| engines/gp/crossover.rs | 138 | 1 | 99.28% |
| engines/gp/engine.rs | 408 | 56 | 86.27% |
| engines/gp/init.rs | 80 | 2 | 97.50% |
| engines/gp/mutation.rs | 228 | 7 | 96.93% |
| engines/gp/node.rs | 114 | 1 | 99.12% |
| engines/gp/primitives.rs | 129 | 65 | **49.61%** |
| engines/hill_climb/configuration.rs | 20 | 8 | 60.00% |
| engines/hill_climb/engine.rs | 161 | 20 | 87.58% |
| engines/ibea/configuration.rs | 32 | 0 | 100.00% |
| engines/ibea/mod.rs | 420 | 45 | 89.29% |
| engines/island/configuration.rs | 27 | 0 | 100.00% |
| engines/island/migration.rs | 366 | 14 | 96.17% |
| engines/island/mod.rs | 398 | 77 | 80.65% |
| engines/island/nsga2.rs | 356 | 29 | 91.85% |
| engines/island/topology.rs | 72 | 1 | 98.61% |
| engines/moead/configuration.rs | 69 | 0 | 100.00% |
| engines/moead/mod.rs | 534 | 37 | 93.07% |
| engines/multi_objective/indicators/generational_distance.rs | 147 | 7 | 95.24% |
| engines/multi_objective/indicators/hypervolume.rs | 141 | 5 | 96.45% |
| engines/multi_objective/indicators/inverted_generational_distance.rs | 132 | 6 | 95.45% |
| engines/multi_objective/indicators/mod.rs | 63 | 4 | 93.65% |
| engines/multi_objective/indicators/spread.rs | 180 | 7 | 96.11% |
| engines/multi_objective/non_dominated_sort.rs | 171 | 7 | 95.91% |
| engines/multi_objective/pareto.rs | 83 | 24 | 71.08% |
| engines/nsga2/configuration.rs | 32 | 0 | 100.00% |
| engines/nsga2/crowding_distance.rs | 71 | 0 | 100.00% |
| engines/nsga2/mod.rs | 478 | 43 | 91.00% |
| engines/nsga2/non_dominated_sort.rs | 171 | 15 | 91.23% |
| engines/nsga2/pareto.rs | 83 | 1 | 98.80% |
| engines/nsga3/configuration.rs | 56 | 0 | 100.00% |
| engines/nsga3/das_dennis.rs | 54 | 2 | 96.30% |
| engines/nsga3/mod.rs | 744 | 61 | 91.80% |
| engines/permutate/configuration.rs | 15 | 0 | 100.00% |
| engines/permutate/engine.rs | 137 | 7 | 94.89% |
| engines/pso/configuration.rs | 48 | 14 | 70.83% |
| engines/pso/engine.rs | 406 | 12 | 97.04% |
| engines/scatter/configuration.rs | 35 | 0 | 100.00% |
| engines/scatter/engine.rs | 357 | 11 | 96.92% |
| engines/sms_emoa/configuration.rs | 37 | 0 | 100.00% |
| engines/sms_emoa/mod.rs | 402 | 29 | 92.79% |
| engines/spea2/configuration.rs | 36 | 0 | 100.00% |
| engines/spea2/mod.rs | 578 | 53 | 90.83% |

---

## src/operations/ Per-Module Breakdown

| Module | Lines | Missed | % |
|--------|------:|-------:|----:|
| operations/crossover.rs | 355 | 176 | **50.42%** |
| operations/crossover/arithmetic.rs | 109 | 19 | 82.57% |
| operations/crossover/blend_alpha.rs | 127 | 13 | 89.76% |
| operations/crossover/clone.rs | 20 | 0 | 100.00% |
| operations/crossover/cycle.rs | 86 | 0 | 100.00% |
| operations/crossover/edge_recombination.rs | 246 | 29 | 88.21% |
| operations/crossover/multi_group_ox.rs | 93 | 1 | 98.92% |
| operations/crossover/multi_group_pmx.rs | 65 | 0 | 100.00% |
| operations/crossover/multipoint.rs | 97 | 0 | 100.00% |
| operations/crossover/order.rs | 116 | 3 | 97.41% |
| operations/crossover/pcx.rs | 157 | 12 | 92.36% |
| operations/crossover/pmx.rs | 110 | 0 | 100.00% |
| operations/crossover/rejuvenate.rs | 24 | 0 | 100.00% |
| operations/crossover/sbx.rs | 135 | 13 | 90.37% |
| operations/crossover/single_point.rs | 57 | 0 | 100.00% |
| operations/crossover/spx.rs | 136 | 12 | 91.18% |
| operations/crossover/undx.rs | 180 | 11 | 93.89% |
| operations/crossover/uniform_crossover.rs | 69 | 0 | 100.00% |
| operations/crossover/variable_length.rs | 113 | 17 | 84.96% |
| operations/extension/mass_deduplication.rs | 79 | 3 | 96.20% |
| operations/extension/mass_degeneration.rs | 44 | 13 | 70.45% |
| operations/extension/mass_extinction.rs | 77 | 2 | 97.40% |
| operations/extension/mass_genesis.rs | 45 | 1 | 97.78% |
| operations/extension/mod.rs | 26 | 0 | 100.00% |
| operations/local_search.rs | 195 | 7 | 96.41% |
| operations/mutation.rs | 270 | 74 | 72.59% |
| operations/mutation/bit_flip.rs | 24 | 0 | 100.00% |
| operations/mutation/cauchy.rs | 61 | 3 | 95.08% |
| operations/mutation/creep.rs | 47 | 1 | 97.87% |
| operations/mutation/differential.rs | 26 | 13 | **50.00%** |
| operations/mutation/gaussian.rs | 144 | 13 | 90.97% |
| operations/mutation/insertion.rs | 48 | 0 | 100.00% |
| operations/mutation/inversion.rs | 28 | 0 | 100.00% |
| operations/mutation/length_mutation.rs | 87 | 10 | 88.51% |
| operations/mutation/levy_flight.rs | 155 | 4 | 97.42% |
| operations/mutation/list_value.rs | 43 | 0 | 100.00% |
| operations/mutation/non_uniform.rs | 106 | 14 | 86.79% |
| operations/mutation/polynomial.rs | 95 | 15 | 84.21% |
| operations/mutation/scramble.rs | 28 | 0 | 100.00% |
| operations/mutation/self_adaptive_gaussian.rs | 89 | 14 | 84.27% |
| operations/mutation/swap.rs | 22 | 0 | 100.00% |
| operations/mutation/uniform.rs | 48 | 3 | 93.75% |
| operations/mutation/value.rs | 93 | 30 | 67.74% |
| operations/selection.rs | 121 | 27 | 77.69% |
| operations/selection/boltzmann.rs | 106 | 6 | 94.34% |
| operations/selection/clearing.rs | 103 | 11 | 89.32% |
| operations/selection/fitness_proportionate.rs | 145 | 2 | 98.62% |
| operations/selection/lexicase.rs | 209 | 16 | 92.34% |
| operations/selection/random.rs | 44 | 0 | 100.00% |
| operations/selection/rank.rs | 90 | 4 | 95.56% |
| operations/selection/tournament.rs | 61 | 1 | 98.36% |
| operations/selection/truncation.rs | 67 | 2 | 97.01% |
| operations/survivor.rs | 37 | 8 | 78.38% |
| operations/survivor/age.rs | 17 | 2 | 88.24% |
| operations/survivor/deterministic_crowding.rs | 104 | 6 | 94.23% |
| operations/survivor/fitness.rs | 42 | 2 | 95.24% |
| operations/survivor/mu_comma_lambda.rs | 51 | 17 | 66.67% |
| operations/survivor/mu_plus_lambda.rs | 42 | 16 | 61.90% |
| operations/survivor/parsimony.rs | 54 | 5 | 90.74% |

---

## Lowest 5 Modules (combined engines + operations)

These are the input targets for Plan 3 (coverage test writing). Listed in ascending coverage order:

1. `src/engines/gp/chromosome.rs` — **49.15%** (118 lines, 60 missed)
2. `src/engines/gp/primitives.rs` — **49.61%** (129 lines, 65 missed)
3. `src/operations/mutation/differential.rs` — **50.00%** (26 lines, 13 missed)
4. `src/operations/crossover.rs` — **50.42%** (355 lines, 176 missed)
5. `src/engines/gp/configuration.rs` — **54.07%** (135 lines, 62 missed)

---

## Exact File Paths Observed

The following absolute paths were recorded from the JSON output (`filename` field) for `src/engines/` and `src/operations/` files. These are used to validate the `--ignore-filename-regex` pattern in the CI workflow.

All paths share the common prefix:
`/Users/luis/RustroverProjects/genetic-algorithms/.claude/worktrees/agent-a663fceae3069ca21/`

The relative path portion (after `/src/`) is what the regex operates on. Key observation: **all files in the target subtrees contain the literal strings `/src/engines/` or `/src/operations/`** in their absolute paths.

Representative sample of observed paths:
```
.../src/engines/alps/configuration.rs
.../src/engines/gp/chromosome.rs
.../src/engines/ga.rs
.../src/operations/crossover.rs
.../src/operations/mutation/differential.rs
.../src/operations/survivor/mu_plus_lambda.rs
```

Files that must be **excluded** from the 80% gate (present in the full coverage run):
- `src/traits/` — trait definitions
- `src/types/` — built-in chromosome/gene types
- `src/configuration.rs` — config structs
- `src/population.rs` — population container
- `src/niching/` — fitness sharing
- `src/extension/` — extension configuration
- `src/stats.rs` — statistics
- `src/observe/` — checkpointing/observer
- `src/fitness/` — fitness function wrapper
- `src/rng.rs` — RNG
- `src/validators/` — configuration validation
- `src/error.rs` — error types
- `src/lib.rs` — crate root
- `src/aos/` — adaptive operator selection
- `src/benchmarks/` — benchmark functions
- `src/constraints/` — constraint handling
- `src/utils/` — utilities

---

## --ignore-filename-regex Validation

The CI workflow uses `--ignore-filename-regex` to exclude all files that are **not** in `src/engines/` or `src/operations/`. The regex must match paths to **exclude** them; files matching the regex are dropped from the coverage report before the threshold check.

The regex approach used: exclude files that do NOT contain `/src/engines/` or `/src/operations/`. Since llvm-cov's `--ignore-filename-regex` excludes matching files, we need to match everything we want to drop. This is done by matching everything EXCEPT `src/engines/` and `src/operations/` paths.

**Approach:** Rather than enumerate every excluded directory, we match file paths that do not contain `src/engines/` and do not contain `src/operations/`. Using `grep -v`-style logic in a single regex:

```
^(?!.*(src/engines/|src/operations/)).*$
```

This regex matches any file path that does NOT contain `src/engines/` or `src/operations/` — those matched files are excluded. Files under `src/engines/` and `src/operations/` do not match and thus are retained in the coverage gate.

**Included (retained for 80% gate):**
- All `src/engines/**` files
- All `src/operations/**` files

**Excluded (dropped from gate computation):**
- `src/traits/`, `src/types/`, `src/configuration.rs`, `src/population.rs`
- `src/niching/`, `src/extension/`, `src/stats.rs`, `src/observe/`
- `src/fitness/`, `src/rng.rs`, `src/validators/`, `src/error.rs`
- `src/lib.rs`, `src/aos/`, `src/benchmarks/`, `src/constraints/`, `src/utils/`
- `tests/` directory (separately excluded via `tests/`)

Note: The `tests/` directory exclusion is handled by a second `--ignore-filename-regex 'tests/'` flag in addition to the main pattern, or can be folded into the combined regex.

---

## Final CI Regex

Added after Task 2 workflow creation.

The `coverage.yml` workflow uses the following `--ignore-filename-regex` value:

```
^(?!.*(src/engines/|src/operations/)).*$
```

**Rationale:**
- The regex is a negative lookahead that matches any path NOT containing `src/engines/` or `src/operations/`.
- Files matching this regex are excluded from the coverage gate computation.
- Result: only `src/engines/**` and `src/operations/**` files contribute to the `--fail-under-lines 80` threshold.
- The `tests/` directory is naturally excluded because test files live outside `src/engines/` and `src/operations/` (they are in `tests/`), and the cargo-llvm-cov `--ignore-filename-regex` with the lookahead pattern already excludes them.

**Verification against observed file paths:**
- `src/engines/ga.rs` → does NOT match regex (contains `src/engines/`) → RETAINED ✓
- `src/operations/crossover.rs` → does NOT match regex → RETAINED ✓
- `src/traits/chromosome.rs` → matches regex (no `src/engines/` or `src/operations/`) → EXCLUDED ✓
- `src/lib.rs` → matches regex → EXCLUDED ✓
- `tests/test_engines.rs` → matches regex → EXCLUDED ✓
