---
phase: 69-build-perf-m3-major-refactors
reviewed: 2026-06-16T00:00:00Z
depth: standard
files_reviewed: 46
files_reviewed_list:
  - .github/workflows/feature-matrix.yml
  - benches/alps.rs
  - benches/cellular.rs
  - benches/crossover.rs
  - benches/de.rs
  - benches/ga_run.rs
  - benches/island_ga.rs
  - benches/metrics_observer.rs
  - benches/mutation.rs
  - benches/nsga2.rs
  - benches/rastrigin.rs
  - benches/scatter.rs
  - benches/selection.rs
  - benches/survivor.rs
  - docs/ARCHITECTURE.md
  - docs/benchmarks.md
  - examples/rastrigin.rs
  - src/engines/eda/engine.rs
  - src/engines/ga/adaptive.rs
  - src/engines/ga/aos.rs
  - src/engines/ga/batch.rs
  - src/engines/ga/cache.rs
  - src/engines/ga/extension.rs
  - src/engines/ga/generation.rs
  - src/engines/ga/lifecycle.rs
  - src/engines/ga/mod.rs
  - src/engines/ga/observer.rs
  - src/engines/ga/stats.rs
  - src/engines/ga/stopping.rs
  - src/engines/gp/engine.rs
  - src/engines/ibea/mod.rs
  - src/engines/island/mod.rs
  - src/engines/island/nsga2.rs
  - src/engines/moead/mod.rs
  - src/engines/nsga2/mod.rs
  - src/engines/nsga3/mod.rs
  - src/engines/sms_emoa/mod.rs
  - src/engines/spea2/mod.rs
  - src/lib.rs
  - src/operations/selection/tournament.rs
  - src/operations/survivor/age.rs
  - src/operations/survivor/fitness.rs
  - src/operations/survivor/mu_comma_lambda.rs
  - src/operations/survivor/mu_plus_lambda.rs
  - src/population.rs
  - src/traits/common.rs
  - tests/observe/observer/test_observer.rs
findings:
  critical: 3
  warning: 3
  info: 1
  total: 7
status: issues_found
---

# Phase 69: Code Review Report

**Reviewed:** 2026-06-16
**Depth:** standard
**Files Reviewed:** 46
**Status:** issues_found

## Summary

This phase covers the ga.rs module split into `src/engines/ga/` submodules, a full port of
benchmarks from Criterion to Divan, and the introduction of a CI feature-matrix workflow with
a rayon cfg-gate enforcement step. Three blockers were found: the CI rayon enforcement step
is always-failing due to a grep pattern that matches correctly-gated `use` statements; the
`mu_comma_lambda` survivor incorrectly discards all offspring in an actual GA run because
the engine's age counter starts at 1 for offspring while the survivor retains only age==0;
and the `age_based` survivor keeps the **oldest** individuals (highest age) despite docs and
module description claiming the opposite. Two further warnings cover a "parallel-off" CI
matrix duplicate entry and a division-by-zero (NaN) in `recalculate_aga` on empty population.

## Critical Issues

### CR-01: CI rayon enforcement step always fails — blocks entire feature-matrix workflow

**File:** `.github/workflows/feature-matrix.yml:69-74`

**Issue:** The enforcement step runs:
```bash
if grep -rn 'rayon::' src/ | grep -v '#\[cfg'; then
  exit 1
fi
```
Every file that correctly gates rayon usage writes a `use rayon::prelude::*;` line **below**
the `#[cfg(...)]` attribute — the use statement itself does not contain the text `#[cfg`. The
second `grep -v '#\[cfg'` therefore does NOT filter it out, so every valid rayon import line
is returned by the pipeline. grep exits 0 (matches found), the `if` branch fires, and the
step exits 1. This causes **all** matrix jobs to fail unconditionally. The CI never reports a
passing run, making the enforcement useless and blocking the entire workflow.

**Fix:** The pattern must match the `#[cfg]` line that precedes the `use` rather than the
use line itself. The idiomatic approach is to check for the `rayon::` *call-site* patterns
(method calls, not import declarations):

```bash
- name: Enforce no unconditional rayon references in src/
  run: |
    # Detect rayon call-sites (par_iter, par_sort, into_par_iter, etc.) that are not
    # preceded by a #[cfg(...)] line on the immediately previous non-empty line.
    # Strategy: flag any line containing a rayon *method* invocation without a cfg gate.
    if grep -rn '\.\(par_iter\|par_sort\|into_par_iter\|par_iter_mut\|par_extend\)' src/ \
         | grep -v '#\[cfg'; then
      echo "ERROR: rayon call-site without cfg gate found"
      exit 1
    fi
```

Alternatively, use a script that checks for `use rayon` imports whose immediately preceding
line is not `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]`.

---

### CR-02: `mu_comma_lambda` discards all offspring in the actual GA run — population never evolves

**File:** `src/operations/survivor/mu_comma_lambda.rs:44`

**Issue:** The function retains chromosomes with `age() == 0` (treating them as "offspring"):
```rust
chromosomes.retain(|c| c.age() == 0);
```
However, in `Ga::run_with_callback` (`src/engines/ga/mod.rs:1346-1398`), the `age` counter
starts at `0` and is incremented to `1` **before** `parent_crossover` is called each
generation. Initial chromosomes are assigned `age = 0` during initialization. The result:

- Generation 0 of the loop: `age = 1`. Offspring get `set_age(1)`. Initial pop has `age = 0`.
- `mu_comma_lambda` retains `age == 0` → keeps the **initial population**, discards offspring.
- Generation 1: surviving chromosomes still have `age = 0`. `age` counter = 2. Offspring get `age = 2`.
- `mu_comma_lambda` again retains `age == 0` → same initial individuals survive forever.

The unit tests in `tests/operations/test_survivor_mu_comma_lambda.rs` manually assign `age = 0`
to "offspring" and `age > 0` to "parents" — the inverse of what the engine actually sets.
The tests pass but do not cover the integration. With `Survivor::MuCommaLambda`, the GA never
evolves: the initial random population persists across all generations.

**Fix:** The survivor function's contract must match what the engine actually sets. Options:

1. **Change the engine** to assign offspring `age = 0` and parents `age > 0` (requires
   incrementing parent ages each generation or resetting offspring age differently).

2. **Change the survivor** to use the maximum parent age as the threshold instead of
   hardcoding `== 0`. The cleaner fix is option 1: in `parent_crossover`, set `child_1.set_age(0)`
   and `child_2.set_age(0)`, and add an explicit parent-age-increment step before crossover:

```rust
// In run_with_callback, before parent_crossover:
for c in self.population.chromosomes.iter_mut() {
    c.set_age(c.age() + 1);
}
// Then offspring are set_age(0) in parent_crossover:
child_1.set_age(0);
child_2.set_age(0);
```

---

### CR-03: `age_based` survivor keeps oldest individuals, not youngest — inverted selection pressure

**File:** `src/operations/survivor/age.rs:31-39`

**Issue:** The function is documented as "Retains the youngest individuals (lowest age)" and
the module-level doc says "Retains the youngest individuals (lowest age) after parents and
offspring have been merged." However the sort is **descending** by age:

```rust
chromosomes.par_sort_unstable_by(|a, b| b.age().cmp(&a.age()));
// sorted: [age=5, age=4, age=3, age=2, age=1]
chromosomes.truncate(population_size);
// keeps: [age=5, age=4, age=3] — the THREE OLDEST
```

`truncate` keeps the front of the vector. After descending sort the front holds the **highest**
ages (oldest). The documentation claims the opposite. This inverts selection pressure:
age-based survivor selection is supposed to favour fresh genetic material (youngest = lowest
age), but the implementation eliminates fresh material and preserves stagnant individuals.

**Fix:** Sort ascending to put youngest at the front, then truncate:
```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
chromosomes.par_sort_unstable_by(|a, b| a.age().cmp(&b.age()));
#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
chromosomes.sort_unstable_by(|a, b| a.age().cmp(&b.age()));
// After ascending sort: [age=1, age=2, age=3, age=4, age=5]
// truncate(3) keeps [age=1, age=2, age=3] — the three YOUNGEST. Correct.
```

## Warnings

### WR-01: "parallel-off" CI matrix entry is a duplicate of "logging-explicit" — missed coverage

**File:** `.github/workflows/feature-matrix.yml:46-48`

**Issue:** The `parallel-off` matrix entry and `logging-explicit` run the identical command:
```yaml
- name: "logging-explicit"
  cmd: "cargo test --quiet --no-default-features --features logging"
- name: "parallel-off"
  cmd: "cargo test --quiet --no-default-features --features logging"
```
Both disable default features (which includes `parallel`) and re-enable only `logging`. There
is no matrix entry that tests with `--no-default-features` and **neither** logging nor parallel
(i.e., the absolute minimum feature set). Additionally, there is no entry testing
`--no-default-features --features parallel` (parallel without logging), which is a valid
user configuration. The duplicate wastes CI minutes without adding coverage.

**Fix:** Replace the duplicate with a genuinely distinct combination:
```yaml
- name: "parallel-only"
  features: "--no-default-features --features parallel"
  cmd: "cargo test --quiet --no-default-features --features parallel"
```

---

### WR-02: `recalculate_aga()` divides by zero when population is empty — produces NaN f_avg

**File:** `src/population.rs:110`

**Issue:** `recalculate_aga` computes the population average fitness without a guard:
```rust
self.f_avg /= self.chromosomes.len() as f64;  // 0.0 / 0.0 = NaN when empty
```
This does not panic (IEEE 754 division produces NaN), but NaN propagates into the adaptive
GA crossover and mutation probability computations (`aga_probability`), poisoning all
probabilities for the generation. The adaptive GA path calls `recalculate_aga` at two points
in `run_with_callback` (line 1315 and 1742 of mod.rs) and could reach this with an empty
population if a catastrophic extension strategy wipes the population (e.g., MassExtinction
combined with a failed regrowth).

**Fix:**
```rust
pub fn recalculate_aga(&mut self) {
    self.f_max = f64::NEG_INFINITY;
    self.f_avg = 0.0;
    if self.chromosomes.is_empty() {
        return;
    }
    for chromosome in self.chromosomes.as_slice() {
        self.f_max = self.f_max.max(chromosome.fitness());
        self.f_avg += chromosome.fitness();
    }
    self.f_avg /= self.chromosomes.len() as f64;
}
```

---

### WR-03: Observer timing hooks share a single `t_cx` elapsed value for crossover, mutation, and fitness — misleading durations

**File:** `src/engines/ga/mod.rs:1515-1523`

**Issue:** A single timer `t_cx` is started before `parent_crossover` and its elapsed value
is passed to three separate observer hooks:
```rust
self.notify(|obs| obs.on_crossover_complete(i, elapsed, offspring_count));
self.notify(|obs| obs.on_mutation_complete(i, elapsed, pop_size));
self.notify(|obs| obs.on_fitness_evaluation_complete(i, elapsed, pop_size));
```
The comment acknowledges this: "NOTE: elapsed covers combined crossover+mutation+fitness time
(EXT-01)". Observers that track per-operator latency (e.g., `MetricsObserver`) will record
the combined duration three times under three different metric labels, making them look three
times slower than they are, and it is impossible to distinguish crossover vs mutation vs
fitness evaluation cost. The `test_mutation_timing_nonzero` and `test_fitness_eval_timing_nonzero`
tests only assert `is_some()`, not that the values are independent, so this goes uncaught.

**Fix:** Either introduce per-operator timers around the distinct operations within
`parent_crossover` and expose them in a richer return type, or collapse the three hooks into
a single `on_generation_phase_complete` hook with an enum tag until EXT-01 is implemented.
At minimum, document in each hook's rustdoc that the `duration` is the combined phase time.

## Info

### IN-01: Benchmark setup data reused across bench iterations in crossover benches — may inflate repeatability

**File:** `benches/crossover.rs:126-139`

**Issue:** The crossover benchmarks (e.g., `cycle`, `order`, `single_point`) create the
parent pair **once** outside the `bencher.bench(|| {...})` closure and reuse it across all
iterations. This is correct for functions that take `&self` (read-only), but if any crossover
implementation were ever changed to mutate its inputs (e.g., for in-place crossover variants),
all iterations after the first would operate on already-modified data. The pattern in the
mutation benches (`with_inputs(|| chromosome.clone()).bench_values(...)`) is the safer idiom
for benchmarks where input state matters.

**Fix:** For defensive correctness, consider using `bencher.with_inputs` even for read-only
operations, or add a comment explaining why reuse is safe:
```rust
#[divan::bench(args = [10usize, 100, 1000])]
fn cycle(bencher: divan::Bencher, gene_length: usize) {
    // cycle takes &parent_1, &parent_2 (read-only) — pair reuse across iterations is safe.
    let (perm_p1, perm_p2) = setup_permutation_pair(gene_length);
    bencher.bench(|| {
        let _ = super::cycle(&perm_p1, &perm_p2);
    });
}
```

---

_Reviewed: 2026-06-16_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
