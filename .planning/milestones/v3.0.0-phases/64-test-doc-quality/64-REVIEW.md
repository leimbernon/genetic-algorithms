---
phase: 64-test-doc-quality
reviewed: 2026-06-17T16:00:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - .github/workflows/coverage.yml
  - src/engines/de/mutation.rs
  - src/engines/de/engine.rs
  - src/engines/de/configuration.rs
  - src/engines/cma/engine.rs
  - src/engines/ga/mod.rs
  - src/engines/ga/generation.rs
  - src/engines/ga/batch.rs
  - src/engines/gp/chromosome.rs
  - src/engines/gp/configuration.rs
  - src/engines/gp/primitives.rs
  - src/observe/observer/composite.rs
  - src/operations/mutation.rs
  - src/engines/multi_objective/non_dominated_sort.rs
  - src/fitness/batch.rs
findings:
  critical: 2
  warning: 3
  info: 2
  total: 7
status: issues_found
---

# Phase 64: Code Review Report

**Reviewed:** 2026-06-17T16:00:00Z
**Depth:** standard
**Files Reviewed:** 15
**Status:** issues_found

## Summary

Phase 64 spans 4 plans: coverage infrastructure, Clippy suppression removal, coverage tests, and rustdoc examples. The review focused on the source files modified during this phase. Two BLOCKER findings involve `panic!` usage in library code (violating AGENTS.md §2.5), and several quality issues were found in doc examples and CI configuration.

## Critical Issues

### CR-01: `panic!` in library code — `src/engines/cma/engine.rs` lines 638, 678, 688, 1053

**File:** `src/engines/cma/engine.rs:638, 678, 688, 1053`
**Issue:** The `CmaEngine::run()` method uses `panic!` in four locations for error conditions that can occur during normal operation (empty init_fn return, empty population after restart). AGENTS.md §2.5 explicitly states: "Never use `panic!` in library code. Use `Result<T, GaError>` instead." The `run()` method already returns `CmaResult<U>` — it should return `Result<CmaResult<U>, GaError>` or handle empty populations gracefully.

- Line 638: `panic!("CmaEngine: init_fn returned an empty population");`
- Line 678: `panic!("CmaEngine: init_fn returned an empty population (first init)");`
- Line 688: `panic!("CmaEngine: init_fn returned an empty population (restart)");`
- Line 1053: `panic!("CmaEngine: no best chromosome found (empty run)")`

**Fix:**
```rust
// Change the return type of run() from CmaResult<U> to Result<CmaResult<U>, GaError>
pub fn run(&mut self) -> Result<CmaResult<U>, GaError>
where
    U::Gene: Debug,
{
    // ...
    if peek_pop.is_empty() {
        return Err(GaError::ConfigurationError(
            "CmaEngine: init_fn returned an empty population".to_string(),
        ));
    }
    // Same pattern for lines 678, 688, 1053
}
```

### CR-02: `assert!` panics in library code — `src/engines/de/mutation.rs` line 33

**File:** `src/engines/de/mutation.rs:33`
**Issue:** `pick_distinct()` uses `assert!(pop_size > n, "population too small for mutation")` which panics on invalid input. AGENTS.md §2.5 says: "Never use `panic!` in library code." This function is called from `mutate()` which returns `Vec<U::Gene>` (no `Result`), so propagating the error requires a signature change.

**Fix:**
```rust
pub(crate) fn pick_distinct(
    rng: &mut impl Rng,
    pop_size: usize,
    exclude: usize,
    n: usize,
) -> Result<Vec<usize>, GaError> {
    if pop_size <= n {
        return Err(GaError::SelectionError(format!(
            "Population size {} too small for mutation (need > {})",
            pop_size, n
        )));
    }
    // ... rest of function
    Ok(chosen)
}
```
Then update `mutate()` to propagate: `let rs = pick_distinct(rng, pop.len(), *i, 3)?;`

## Warnings

### WR-01: Misleading doc comment in `assign_ranks` example — `non_dominated_sort.rs` line 193

**File:** `src/engines/multi_objective/non_dominated_sort.rs:193`
**Issue:** The doc comment for `assign_ranks` says `let b = vec![0.6, 0.6]; // dominated by a (a[0]<b[0] AND a[1]>b[1] — not dominated actually)`. The comment is internally contradictory: it claims `b` is "dominated by a" but the parenthetical correctly notes they are "not dominated actually." For minimization objectives, `a=[0.1,0.9]` and `b=[0.6,0.6]` are non-dominated (a wins on f1, b wins on f2). The actual dominance relationship is `b` dominates `c` (b[0]=0.6 < c[0]=0.7 AND b[1]=0.6 < c[1]=0.8).

**Fix:**
```rust
/// let a = vec![0.1, 0.9];
/// let b = vec![0.6, 0.6]; // non-dominated with a
/// let c = vec![0.7, 0.8]; // dominated by b
```

### WR-02: `coverage.yml` missing `--locked` for reproducible builds

**File:** `.github/workflows/coverage.yml:41`
**Issue:** The `cargo install cargo-llvm-cov --locked` command is correct. However, the `--locked` flag only locks dependencies — it does not pin the `cargo-llvm-cov` version itself. Over time, `cargo install` will pull the latest published version, which may introduce behavioral differences. The plan specified `--locked` to honor `Cargo.lock`, but the `coverage.yml` workflow uses `cargo install` (not a binary from a lockfile), so version drift is possible.

**Fix:** Consider pinning the version explicitly:
```yaml
- name: Install cargo-llvm-cov
  run: cargo install cargo-llvm-cov --locked --version 0.8.7
```

### WR-03: `JadeState::draw_f` has unbounded retry loop — `de/mutation.rs` line 257

**File:** `src/engines/de/mutation.rs:257`
**Issue:** `draw_f()` uses a `loop` that retries until `f > 0.0`. While mathematically guaranteed to terminate eventually (Cauchy has non-zero probability of positive values), there is no iteration bound. If `mu_f` drifts negative due to a numerical edge case in the Lehmer mean update, this could spin for a very long time. Same pattern in `LShadeState::draw_f` at line 326.

**Fix:** Add a safety bound:
```rust
pub fn draw_f(&self, rng: &mut impl Rng) -> f64 {
    for _ in 0..100 {
        let f = cauchy_sample(rng, self.mu_f, 0.1);
        if f > 0.0 {
            return f.min(1.0);
        }
    }
    // Fallback: return mu_f clamped to (0, 1]
    self.mu_f.clamp(f64::EPSILON, 1.0)
}
```

## Info

### IN-01: `CompositeObserver::add` deprecated alias has `#[allow(clippy::should_implement_trait)]` — intentional but documented

**File:** `src/observe/observer/composite.rs:83`
**Issue:** The `#[allow(clippy::should_implement_trait)]` suppression on the deprecated `add` method is intentional (kept for backward compatibility per Plan 02 decision). The suppression is correctly scoped to only the deprecated wrapper. No action needed — the finding is recorded for completeness.

### IN-02: Type aliases `ConstraintFn`, `RepairFn`, `RewardAccumulator` are private — verify no downstream breakage

**File:** `src/engines/ga/mod.rs:180-186`
**Issue:** The three type aliases introduced in Plan 02 are private (`type ConstraintFn<G> = ...`). If any downstream code was accessing the underlying `Arc<dyn Fn(...)>` types directly, the aliases are transparent. However, if the `Ga` struct fields were `pub`, the field types changed (from anonymous `Arc<dyn Fn(...)>` to named `ConstraintFn`). Since `Ga` fields are not `pub`, this is not a breaking change. Recorded for completeness.

---

_Reviewed: 2026-06-17T16:00:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
