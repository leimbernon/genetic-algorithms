---
phase: 68-build-perf-m2-dependency-hygiene
fixed_at: 2026-06-15T00:00:00Z
review_path: .planning/phases/68-build-perf-m2-dependency-hygiene/68-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 68: Code Review Fix Report

**Fixed at:** 2026-06-15
**Source review:** `.planning/phases/68-build-perf-m2-dependency-hygiene/68-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 7 (CR-01, CR-02, CR-03, WR-01, WR-02, WR-03, WR-04)
- Fixed: 7
- Skipped: 0

## Fixed Issues

### CR-01 + CR-02: Doc examples use nonexistent method and wrong closure/function signatures

**Files modified:** `src/lib.rs`, `src/engines/ga.rs`
**Commit:** 59dfc81
**Applied fix:**
- Replaced `.with_genes_per_chromosome(n)` with `.with_chromosome_length(ChromosomeLength::Fixed(n))` in both files. Added `ChromosomeLength` to the import in `src/lib.rs`; used `crate::chromosomes::ChromosomeLength::Fixed(10)` in `src/engines/ga.rs`.
- Changed three-argument closures `|n, _, _|` to correct two-argument form `|n, _|` in both doc examples.
- Removed fabricated third argument `Some(false)` from `range_random_initialization` calls — the function only accepts two arguments (`genes_per_chromosome`, `alleles`).

---

### CR-03: Adaptive penalty coefficient never escapes 0.0

**Files modified:** `src/engines/ga.rs`
**Commit:** 5ba1af4
**Applied fix:**
In `apply_penalty_to_chromosomes` (`PenaltyStrategy::Adaptive` branch): added bootstrap assignment `if self.penalty_coefficient == 0.0 { self.penalty_coefficient = initial_coefficient; }` before reading `coeff = self.penalty_coefficient`. The multiplicative window updates now operate on a non-trivial starting value.

Also fixed the offspring constraint loop (line ~1832): destructured `initial_coefficient` from `PenaltyStrategy::Adaptive { initial_coefficient, .. }` and used it as the coefficient fallback when `self.penalty_coefficient` has not yet been bootstrapped by the generation-boundary update.

Note: this fix involves coefficient comparison/update logic and should be verified manually to confirm the adaptive update semantics are correct for the intended algorithm.

---

### WR-01: Unconditional spurious `limit_reached` trace messages

**Files modified:** `src/observe/observer/log.rs`
**Commit:** 6bcd06a
**Applied fix:**
Removed the two unconditional `trace!` "limit reached" emissions from `on_generation_end`. Added a comment explaining they have moved to `on_run_end`. Updated `on_run_end` to emit the bookend `debug!` messages and the `trace!` "limit reached for fixed fitness" message only when `cause == TerminationCause::FitnessTargetReached`.

---

### WR-02: Exact float equality in `limit_reached`, missing Maximization branch

**Files modified:** `src/engines/ga.rs`
**Commit:** ed329f7
**Applied fix:**
- Minimization check: `chromosome.fitness() == 0.0` → `chromosome.fitness().abs() < 1e-9`
- Added missing `ProblemSolving::Maximization` branch: stops when `chromosome.fitness() >= target` (early exit when fitness_target is set)
- FixedFitness check: `chromosome.fitness() == target` → relative epsilon `(fitness - target).abs() <= target.abs() * 1e-9 + 1e-12`

Updated doc comment to reflect new semantics.

Note: the Maximization branch uses `>=` (greater-or-equal) rather than epsilon comparison since maximization targets are typically "at least X" semantics. If exact-equality semantics are desired for Maximization FixedFitness cases, this requires human verification.

---

### WR-03: `initialize_with_seeds` dedup compares by `gene.id()` only

**Files modified:** `src/engines/ga.rs`
**Commit:** 06411f0
**Applied fix:**
Before running the id-based duplicate check, compute `ids_are_unique` by inserting all allele IDs into a `HashSet` and checking that the set size equals the allele count. The duplicate check (both against seed DNAs and against already-generated fill chromosomes) is now only executed when `ids_are_unique == true`. When allele IDs are non-unique (e.g., all Range alleles share `id=0`), dedup is skipped entirely, allowing `with_seeds` to work correctly for real-valued chromosomes.

---

### WR-04: `test_no_logger_installed.rs` — PanicLogger teardown

**Files modified:** `tests/test_no_logger_installed.rs`
**Commit:** e755265
**Applied fix:**
- Added a prominent `# INVARIANT` comment at the top of the file documenting that the file must contain exactly one test function, explaining that `PANIC_LOGGER` cannot be uninstalled, and warning maintainers not to add more tests to this file.
- After calling `log::set_max_level(LevelFilter::Trace)` (which proves the slot is free), immediately set `log::set_max_level(LevelFilter::Off)` to silence `PanicLogger` for any code running later in the same process.

---

_Fixed: 2026-06-15_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
