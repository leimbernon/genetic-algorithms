---
phase: 68-build-perf-m2-dependency-hygiene
reviewed: 2026-06-15T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - tests/test_no_logger_installed.rs
  - src/engines/ga.rs
  - src/configuration.rs
  - src/configuration/builders.rs
  - src/traits/configuration.rs
  - docs/getting-started.md
  - src/lib.rs
  - src/observe/observer/mod.rs
  - src/observe/observer/log.rs
  - .github/workflows/feature-matrix.yml
findings:
  critical: 3
  warning: 4
  info: 2
  total: 9
status: issues_found
---

# Phase 68: Code Review Report

**Reviewed:** 2026-06-15T00:00:00Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

This phase adds the `test_no_logger_installed.rs` integration test, refactors configuration builder impls into `src/configuration/builders.rs`, expands `src/observe/observer/` with `log.rs`, and updates the feature-matrix workflow. The core files (`ga.rs`, `configuration.rs`, `traits/configuration.rs`) contain several pre-existing issues that are exposed or worsened by the new code added in this phase.

The most severe issues are: (1) doc-code examples in `src/lib.rs` and `src/engines/ga.rs` call a nonexistent `with_genes_per_chromosome` method — this is a public-facing API contract breakage; (2) the same examples pass three-argument closures to `with_initialization_fn` whose type only accepts two — every copy-paste of these examples fails to compile; (3) the adaptive penalty coefficient is silently multiplied starting from `0.0`, meaning `penalty_coefficient` will remain `0.0` forever when `initial_coefficient` is used and no initial assignment is made before the update window fires.

---

## Structural Findings (fallow)

No structural pre-pass was provided for this review.

---

## Narrative Findings (AI reviewer)

### Critical Issues

#### CR-01: `with_genes_per_chromosome` does not exist — all doc examples are broken

**File:** `src/lib.rs:38`, `src/engines/ga.rs:91`, `src/engines/ga.rs:769`

**Issue:** Both the crate-level `lib.rs` quick-start example and the `Ga` module-level doc
example call `.with_genes_per_chromosome(...)`, which is not a method on `Ga<U>` or any
configuration trait. The real API requires `.with_chromosome_length(ChromosomeLength::Fixed(n))`.
These doc examples are marked `rust,ignore`, so the compiler never catches the dead method — but
users who copy-paste the canonical quick-start snippet get an immediate compile error:

```
error[E0599]: no method named `with_genes_per_chromosome` found for struct `Ga<…>`
```

This invalidates the primary onboarding path for new users.

**Fix:**
```rust
// src/lib.rs line 38 — replace:
.with_genes_per_chromosome(5_usize)
// with:
.with_chromosome_length(genetic_algorithms::chromosomes::ChromosomeLength::Fixed(5))

// src/engines/ga.rs line 91 — replace:
.with_genes_per_chromosome(10)
// with:
.with_chromosome_length(crate::chromosomes::ChromosomeLength::Fixed(10))
```

---

#### CR-02: Three-argument closure passed to `with_initialization_fn`, which expects two

**File:** `src/lib.rs:40-42`, `src/engines/ga.rs:93-95`

**Issue:** The `InitializationFn<G>` type is defined as:
```rust
pub type InitializationFn<G> = dyn Fn(usize, Option<&[G]>) -> Vec<G> + Send + Sync;
```
Two parameters. Both doc examples pass a closure with **three** parameters:
```rust
.with_initialization_fn(move |genes_per_chromosome, _, _| {   // third _ is phantom
    range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
})
```
Additionally, `range_random_initialization` itself only accepts **two** arguments
(`src/initializers/range_initializer.rs:36-39`), so `Some(false)` as the third argument is
also fabricated. Because all examples are `rust,ignore`, this is invisible to CI but every
user who follows the quick-start guide encounters compile errors.

**Fix:**
```rust
// Replace the three-arg closure with the correct two-arg form:
.with_initialization_fn(move |n, _| {
    range_random_initialization(n, Some(&alleles_clone))
})
```

---

#### CR-03: Adaptive penalty coefficient never escapes `0.0` on the first update window

**File:** `src/engines/ga.rs:2595-2631`

**Issue:** The `PenaltyStrategy::Adaptive` branch in `apply_penalty_to_chromosomes` uses a
local snapshot `coeff` (which correctly substitutes `initial_coefficient` when
`self.penalty_coefficient == 0.0`) to apply the penalty. However, when the update window fires,
it multiplies `self.penalty_coefficient` directly — which is still `0.0`:

```rust
let coeff = if self.penalty_coefficient == 0.0 {
    initial_coefficient   // used to apply penalty (correct)
} else {
    self.penalty_coefficient
};
// ... at the update window:
if self.adaptive_penalty_counter > 0 {
    let new_coeff = self.penalty_coefficient * 1.1;  // 0.0 * 1.1 = 0.0 forever
    self.penalty_coefficient = new_coeff;
}
```

`self.penalty_coefficient` is initialized to `0.0` (line 419) and is never assigned
`initial_coefficient` before the multiplicative update. The coefficient remains `0.0`
permanently. The adaptive penalty is silently non-functional.

The identical dead-code pattern also exists in the offspring constraint loop at line 1834:
```rust
let coeff = if self.penalty_coefficient == 0.0 {
    0.0 // comment says "Will be initialized at generation boundary" — but it never is
```

**Fix:**
```rust
// Assign initial_coefficient to self.penalty_coefficient on first use:
if self.penalty_coefficient == 0.0 {
    self.penalty_coefficient = initial_coefficient;
}
let coeff = self.penalty_coefficient;
// Now window updates: self.penalty_coefficient * 1.1 is non-trivial
```

---

### Warnings

#### WR-01: `LogObserver::on_generation_end` emits unconditional spurious `limit_reached` trace messages

**File:** `src/observe/observer/log.rs:154-161`

**Issue:** The `on_generation_end` hook unconditionally emits two `trace!`-level messages:
```rust
log::trace!(target="ga_events", method="limit_reached"; "limit reached for minimization");
log::trace!(target="ga_events", method="limit_reached"; "limit reached for fixed fitness");
```
These fire on **every generation end**, even when neither limit condition is met. With
`RUST_LOG=trace`, users see thousands of spurious "limit reached" messages in normal runs.
This contradicts the stated design goal of reproducing pre-v2.2.0 log output faithfully —
the original code only emitted these messages when the condition was actually true.

**Fix:** Remove the unconditional trace emissions. If reproduction is desired, add an
`on_run_end` implementation that checks `TerminationCause`:
```rust
fn on_run_end(&self, cause: TerminationCause, _all_stats: &[GenerationStats]) {
    log::debug!(target="ga_events", method="limit_reached"; "Started limit reached method");
    if matches!(cause, TerminationCause::FitnessTargetReached) {
        log::trace!(target="ga_events", method="limit_reached"; "limit reached for fixed fitness");
    }
    log::debug!(target="ga_events", method="limit_reached"; "Limit reached method finished");
}
```

---

#### WR-02: `limit_reached` uses exact float equality — misses floating-point fitness targets

**File:** `src/engines/ga.rs:2754-2781`

**Issue:** The `limit_reached` function checks stopping conditions with `==` on `f64`:
```rust
if chromosome.fitness() == 0.0 {      // Minimization
if chromosome.fitness() == target {   // FixedFitness
```
For any fitness function that produces a result via floating-point arithmetic (e.g.,
`1e-15` instead of `0.0`, or `100.000000001` instead of `100.0`), the limit is never
triggered. The run exhausts `max_generations` silently. This is a correctness bug for all
callers that use a computed (non-integer) fitness target.

Additionally, `fitness_target` is never checked for `ProblemSolving::Maximization` — users
who call `with_fitness_target` with `Maximization` get no early stop at all.

**Fix:** Use epsilon comparison or document the exact-equality semantics explicitly:
```rust
// Minimization:
if chromosome.fitness().abs() < 1e-9 {
// FixedFitness:
if (chromosome.fitness() - target).abs() <= target.abs() * 1e-9 + 1e-12 {
```

---

#### WR-03: `initialize_with_seeds` dedup compares by `gene.id()` only — always fails for template-allele gene types

**File:** `src/engines/ga.rs:1393-1425`

**Issue:** The genotypic uniqueness check in `initialize_with_seeds` compares DNA by
`gene.id()` at each position:
```rust
let id_a = new_dna.get(i).map(|g| g.id()).unwrap_or(-1);
let id_b = seed_dna.get(i).map(|g| g.id()).unwrap_or(-1);
id_a == id_b
```
For `Range<T>` chromosomes initialized from a template allele, all genes share
`id = 0` (the template's id). Every generated chromosome appears identical to every seed
under this comparison, so `is_duplicate` is always `true`. The retry loop exhausts
`max_attempts` (line 1373: `fill_count * 10`) and returns:
```
GaError::InitializationError("Failed to generate N unique random chromosomes…")
```
This means `with_seeds` is silently broken for the most common real-valued use case.

**Fix:** Compare gene **values** (via `PartialEq` on gene), not just ids, or skip dedup
when all allele ids are non-unique:
```rust
// Check if gene IDs are unique before applying id-based dedup
let ids_are_unique = {
    let ids: std::collections::HashSet<i32> = self.alleles.iter().map(|g| g.id()).collect();
    ids.len() == self.alleles.len()
};
// Only run dedup when ids meaningfully distinguish genes
if ids_are_unique { /* run existing check */ }
```

---

#### WR-04: `test_no_logger_installed.rs` installs a `PanicLogger` that persists for the whole process

**File:** `tests/test_no_logger_installed.rs:85-89`

**Issue:** After the GA run, the test installs `PANIC_LOGGER` as the global logger:
```rust
log::set_logger(&PANIC_LOGGER).expect("...");
log::set_max_level(log::LevelFilter::Trace);
```
This sets the level to `Trace` permanently. Any log emission at any level by code running
after this test in the same process will call `PanicLogger::log()`, which panics. The
comment at line 15 correctly explains that integration test files are separate binaries,
but if a second test function is ever added to this file, it will inherit the poisoned
logger state from the first test (tests in the same binary share the process) and fail
unpredictably.

There is no protection against accidentally adding a second test to this file (e.g., a
future maintainer adds a variant that tests a different GA configuration).

**Fix:** Add a prominent invariant comment at the top of the test file:
```rust
// INVARIANT: This file MUST contain exactly ONE test function.
// `PANIC_LOGGER` is installed at Trace level and cannot be uninstalled.
// A second test in this file would panic on any subsequent log emission.
```
Also consider resetting the max level to `Off` after the assertion so accidental future
additions do not panic:
```rust
log::set_max_level(log::LevelFilter::Trace);
// Prove the slot is free, then immediately silence the logger
// so the PanicLogger cannot fire on any future code in this process.
log::set_max_level(log::LevelFilter::Off);
```

---

### Info

#### IN-01: `feature-matrix.yml` — `logging-explicit` step provides redundant but harmless coverage

**File:** `.github/workflows/feature-matrix.yml:43-45`

**Issue:** The `logging-explicit` step runs with `--no-default-features --features logging`.
This correctly exercises `test_no_logger_installed.rs`. However, the `default` matrix entry
(line 19-21) also exercises it because `logging` is in `default = ["logging"]`. The explicit
step adds coverage value only for testing that the feature compiles in isolation without other
defaults. No action required.

---

#### IN-02: Builder impls duplicated across `builders.rs` and `ga.rs`

**File:** `src/configuration/builders.rs:20-273`, `src/engines/ga.rs:430-734`

**Issue:** Every `XxxConfig for GaConfiguration` impl in `builders.rs` is structurally
duplicated by the corresponding `XxxConfig for Ga<U>` impl in `ga.rs`. Both implement the
same trait methods with identical logic, differing only in delegation depth. This is
~260 lines of near-identical code and creates a maintenance hazard: any new builder method
requires two identical implementations. A macro or blanket delegation impl would eliminate
the duplication.

No change required for this phase.

---

_Reviewed: 2026-06-15T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
