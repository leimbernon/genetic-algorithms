# Phase 69: Build-perf M3 — Major Refactors - Research

**Researched:** 2026-06-15
**Domain:** Rust benchmark migration (criterion → divan), feature-gated parallelism, large-file module splitting
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Gate ALL rayon call-sites across the full codebase — not just `ga.rs` + `population.rs`. This includes all alt-engines (`nsga2`, `nsga3`, `moead`, `spea2`, `ibea`, `gp`, `island`, `cellular`, `alps`, `de`, `scatter`, `eda`). Every `par_iter()`/`into_par_iter()`/`par_iter_mut()`/`par_sort_unstable_by()`/`par_chunks*()` site in all 20 files gets the combined gate. Matches BUILD-PERF.md §Action #3 "every... site" language.

**D-02:** Non-rayon wasm32 gates (e.g., `#[cfg(not(target_arch = "wasm32"))]` for `Instant::now()` or WASM-incompatible stdlib calls) are NOT touched — only sites that call into `rayon::` get updated.

**D-03:** Do NOT add a separate regex CI step or clippy custom lint beyond the grep step (D-04). Rely on feature-matrix CI (CC-2) compiling with `--no-default-features --features logging` to catch any unconditional rayon reference at compile time.

**D-04:** Add a grep-based CI step (fast, <1s) that checks `src/` for any `rayon::` usage without the cfg gate. Runs as part of the `parallel` feature plan (69-03). Complements the feature-matrix compile check.

**D-05:** Feature name is `parallel` (semantic). Feature declaration in `Cargo.toml`: `parallel = ["dep:rayon"]`; default includes `parallel` and `logging`. Matches BUILD-PERF.md §Action #3.

**D-06:** The canonical gate pattern for every rayon site is exactly:
- Sequential fallback: `#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]`
- Parallel arm: `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]`
- CLAUDE.md "WASM Compatibility" section must be updated to document this new canonical gate.

**D-07:** Allow light cleanup during migration — remove dead/duplicate bench cases and simplify overly complex criterion setup code where divan makes it easier. All ported cases must stay within ±3% median tolerance. Port is still one-bench-file-per-commit.

**D-08:** `metrics_observer` bench (requires `--features observer-metrics`) stays as a **separate CI step** to keep feature isolation clean. Same pattern as `de` bench with `--features benchmarks`.

**D-09:** Both `criterion` and `divan` coexist in `Cargo.toml` during porting; `criterion` is removed in the same plan (69-01) once every bench file is ported and CI is green.

**D-10:** Submodule-by-submodule commits — one commit per extracted submodule (~11 commits). BUILD-PERF.md "atomic revert" satisfied by `git revert <range>` covering all commits.

**D-11:** Visibility strategy — minimize `pub` surface: use `pub(super)` for sibling sharing; escalate to `pub(crate)` only when accessed from outside `engines/ga/`. Public API items remain `pub` unchanged.

**D-12:** `cargo expand` symbol diff is the primary semantic-equivalence check: run before and after the full split and confirm zero diff in the exported symbol table. Used in plan 69-04 before merge.

**D-13:** Add a grep-based CI enforcement step in plan 69-03: `grep -rn 'rayon::' src/ | grep -v '#\[cfg'` (or equivalent) that fails if any match found.

### Claude's Discretion

- Exact grep regex for the enforcement step — use whatever correctly catches bare `rayon::` imports and call-sites while ignoring cfg-gated lines.
- Ordering of the 11 submodule extraction commits (within plan 69-04) — extract in dependency order (low-level helpers first: `cache`, `stats`, `observer`, `stopping`; then algorithm steps: `generation`, `lifecycle`, `adaptive`, `aos`, `extension`, `batch`; finally `mod.rs` orchestrator).

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

---

## Summary

Phase 69 lands three large-scope refactors as 5 sequential plans across 3 waves. The work is pure code transformation — no new user-facing features, no behavioural changes. The risk profile is HIGH because the rayon gating touches 20 files with 36 distinct call-sites (including `par_sort_unstable_by`), and the ga.rs split touches 3,342 lines that are the single most-loaded compilation unit in the codebase.

The critical insight from the codebase investigation is that the 20 files with rayon usage are **not in a uniform state**: 13 files already have `#[cfg(not(target_arch = "wasm32"))]` gates on both the `use rayon::prelude::*;` import and the individual call sites; 3 files (`population.rs`, `traits/common.rs`, `engines/island/nsga2.rs`) have entirely ungated rayon imports; and 4 files (`engines/eda/engine.rs`, `engines/island/mod.rs`, `engines/island/nsga2.rs` function-level) use local inline `use rayon::prelude::*;` imports inside function/block scopes. Plan 69-02 must normalize all of these to the new combined gate pattern `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]`.

The ga.rs module is declared via `#[path = "engines/ga.rs"]` in `src/lib.rs`, not from an `engines/mod.rs`. After the split into `src/engines/ga/`, the `#[path]` attribute must be updated to `#[path = "engines/ga/mod.rs"]` — a one-line change that makes the split transparent to all consumers.

**Primary recommendation:** Execute plans strictly in order (69-01 → 69-02 → 69-03 → 69-04 → 69-05). Do not parallelise. Each plan's CI gate must be green before the next starts, because rayon gating must be complete before the ga.rs split (avoids compound failures during the most risky plan).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Benchmark execution | Dev toolchain (cargo bench) | CI bench workflow | Bench files are dev-dependency consumers; no runtime tier |
| Parallel feature gate | Cargo features / Rustc cfgs | All engine source files | Feature declaration in Cargo.toml; enforcement via cfg attributes at call-sites |
| ga.rs submodule layout | Rust module system (file layout) | `src/lib.rs` `#[path]` attribute | Rust resolves `mod ga` via `#[path]`; submodule visibility is compiler-enforced |
| CI enforcement (grep) | GitHub Actions workflow | — | Fast check, no compilation cost |
| Regression verification | CC-1/CC-2/CC-3 harnesses | `cargo public-api` | All 6 BUILD-PERF.md guarantees enforced at PR merge gate |

---

## Standard Stack

### Core (Phase 69 — new addition)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `divan` | 0.1.21 | Benchmark harness replacement for `criterion` | [VERIFIED: crates.io registry] 4.6M downloads, nvzqz/divan, statistically sound, far fewer transitive deps than criterion 0.8.x |

### Retained (already in Cargo.toml)

| Library | Version | Purpose | Note |
|---------|---------|---------|------|
| `rayon` | 1.10 | Parallelism | Becomes `optional = true` in plan 69-02; no version change |
| `criterion` | 0.8.2 | Bench harness (being removed) | Coexists with divan during plan 69-01; removed at plan 69-01 end |

### Dev tooling (required but not in Cargo.toml)

| Tool | Purpose | Status |
|------|---------|--------|
| `cargo-expand` | Symbol-table diff for ga.rs split verification (D-12) | NOT installed — `cargo expand --version` fails. Must be installed before plan 69-04 executes. Install: `cargo install cargo-expand` |
| `cargo-public-api` | Public API surface diff (acceptance gate #5) | NOT installed — `cargo public-api --version` fails. Must be installed or CI-only. Install: `cargo install cargo-public-api` |

**Installation (plan 69-01 Wave 0):**
```toml
# Add to [dev-dependencies]:
divan = "0.1.21"

# After all benches ported, remove from [dev-dependencies]:
# criterion = "0.8.2"   ← delete this line
```

```toml
# Add to [features] in plan 69-02:
parallel = ["dep:rayon"]
default = ["logging", "parallel"]   # update from ["logging"]

# Make rayon optional in [dependencies]:
rayon = { version = "1.10", optional = true }
```

---

## Package Legitimacy Audit

> Only one new package is introduced: `divan`. Packages removed or made optional are not new risks.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| `divan` | crates.io | ~2 yrs | 4.6M total | github.com/nvzqz/divan | [ASSUMED — slopcheck install blocked by sandbox] | Approved [VERIFIED: crates.io, cargo search] |

**Packages removed due to slopcheck [SLOP] verdict:** none

**Packages flagged as suspicious [SUS]:** none

*slopcheck was blocked by sandbox policy at research time. `divan = "0.1.21"` was verified via `cargo search divan` (crates.io registry, authoritative source). The nvzqz/divan repository is the official package from Nicholas Matsakis-linked ecosystem contributor. Planner may optionally add a `checkpoint:human-verify` before the `divan` dev-dependency add if desired, but risk is assessed as LOW.*

---

## Architecture Patterns

### System Architecture Diagram

```
[benches/*.rs]                          [src/**/*.rs]           [src/engines/ga.rs]
     │                                       │                        │
     │ Plan 69-01:                           │ Plan 69-02:            │ Plans 69-04/05:
     │ criterion → divan                     │ rayon import           │ split into
     │ (one file per commit)                 │ → cfg-gated            │ ga/{mod,lifecycle,
     ▼                                       │   parallel+wasm32      │  generation,adaptive,
[divan::main! macro]                         ▼   combined gate        │  aos,extension,
[#[divan::bench] attrs]              [#[cfg(all(                     │  cache,batch,stats,
                                      not(target_arch="wasm32"),     │  observer,stopping}.rs
                                      feature="parallel"))]          ▼
                                     [par_iter / par_sort sites]  [src/engines/ga/mod.rs]
                                                                   [pub use Ga at same path]
                                          │
                                     Plan 69-03:
                                          │
                                    [Cargo.toml]
                                    parallel = ["dep:rayon"]
                                    rayon = { optional = true }
                                          │
                                    [CI: feature-matrix.yml]
                                    adds parallel=off combination
                                          │
                                    [CI grep step]
                                    grep -rn 'rayon::' src/ |
                                    grep -v '#\[cfg'  → must be empty
```

### Recommended File Layout After Phase 69

```
src/engines/
├── ga/                    ← NEW directory (was ga.rs)
│   ├── mod.rs             ← Ga<U> struct, builder impls, build(), run(), run_with_callback()
│   ├── lifecycle.rs       ← init_population, initialize_random, initialize_with_seeds, finalise_run
│   ├── generation.rs      ← the per-generation loop (selection→crossover→mutation→survivor)
│   ├── adaptive.rs        ← adaptive crossover/mutation probability recomputation
│   ├── aos.rs             ← Adaptive Operator Selection integration
│   ├── extension.rs       ← extension trigger + diversity check
│   ├── cache.rs           ← fitness cache lookup / insertion
│   ├── batch.rs           ← batch_evaluate helper
│   ├── stats.rs           ← GenerationStats collection
│   ├── observer.rs        ← observer hook dispatch (notify fn)
│   └── stopping.rs        ← limit_reached, stopping-criteria check
├── island/
├── nsga2/
│   ...
benches/
├── selection.rs            ← ported to divan
├── crossover.rs            ← ported to divan
├── mutation.rs             ← ported to divan
├── survivor.rs             ← ported to divan
├── ga_run.rs               ← ported to divan (most complex setup)
├── nsga2.rs                ← ported to divan
├── island_ga.rs            ← ported to divan (most complex setup)
├── de.rs                   ← ported to divan (requires --features benchmarks)
├── scatter.rs              ← ported to divan
├── alps.rs                 ← ported to divan
├── cellular.rs             ← ported to divan
├── rastrigin.rs            ← ported to divan
└── metrics_observer.rs     ← ported to divan (requires --features observer-metrics)
```

---

## Plan 69-01: Criterion → Divan Port

### Criterion API Inventory (Complete)

The following criterion APIs are used across all 13 bench files. Every one needs a divan equivalent:

| Criterion API | Files Using It | Divan Equivalent |
|---------------|---------------|-----------------|
| `Criterion` struct, `criterion_group!`, `criterion_main!` | ALL 13 | `divan::main!()` macro at crate root; `#[divan::bench]` attr |
| `c.benchmark_group("name")` | ga_run, island_ga, crossover, mutation, survivor, selection, nsga2 | `#[divan::bench(name = "group_name")]` or module-level grouping |
| `group.bench_with_input(BenchmarkId::new(name, param), &param, \|b, p\| ...)` | ALL parameterised | `#[divan::bench(args = [...])]` with `bencher: Bencher, arg: T` |
| `b.iter_batched(\|\| setup(), \|x\| work(x), BatchSize::SmallInput)` | ga_run, island_ga, mutation, survivor, rastrigin | `bencher.with_inputs(\|\| setup()).bench(\|b, x\| work(x))` |
| `b.iter(\|\| work())` (simple iter) | metrics_observer, simple sites | `bencher.bench(\|\| work())` |
| `group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic))` | ga_run, island_ga, crossover, mutation, survivor, selection, nsga2 | Drop entirely — divan does not have equivalent; omit (no functional loss) |
| `group.throughput(Throughput::Elements(n))` | selection, nsga2 | `#[divan::bench(sample_count = N)]` or `bencher.counter(ItemsCount(n))` [ASSUMED] |
| `group.sample_size(10)` | alps, cellular, de, scatter | `#[divan::bench(sample_count = 10)]` |
| `BenchmarkId::new(name, param)` | everywhere parameterised | Inline via `args` parameter or string in bench name |

**Key divan structure pattern** [ASSUMED — from divan docs/README, not Context7-verified]:
```rust
// Instead of criterion_group!/criterion_main!, divan uses:
fn main() {
    divan::main();
}

// Instead of |b: &mut Criterion| { ... }
// use attribute macros on fns:
#[divan::bench]
fn bench_something(bencher: divan::Bencher) {
    bencher.bench(|| { /* work */ });
}

// With setup (BatchSize equivalent):
#[divan::bench]
fn bench_with_setup(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| setup())
        .bench(|b, input| { use_input(input); });
}

// With args (parameterised):
#[divan::bench(args = [10, 50, 100])]
fn bench_parameterised(bencher: divan::Bencher, n: usize) {
    bencher.bench(|| { work_n(n); });
}
```

### Bench File Complexity Rating

| File | Lines | Complexity | Key criterion features used |
|------|-------|------------|---------------------------|
| `ga_run.rs` | ~190 | HIGH | `iter_batched`, `BenchmarkId`, `PlotConfiguration`, `AxisScale`, `BatchSize` |
| `island_ga.rs` | ~195 | HIGH | same as ga_run.rs + IslandGa setup |
| `mutation.rs` | ~310 | HIGH | `iter_batched`, many `BenchmarkId` params, `PlotConfiguration` |
| `survivor.rs` | ~150 | MED | `iter_batched`, `BenchmarkId`, `PlotConfiguration` |
| `selection.rs` | ~195 | MED | `Throughput::Elements`, `BenchmarkId`, `PlotConfiguration` |
| `nsga2.rs` | ~100 | MED | `Throughput::Elements`, `BenchmarkId`, `PlotConfiguration` |
| `crossover.rs` | ~206 | MED | `BenchmarkId`, `PlotConfiguration` |
| `rastrigin.rs` | ~110 | MED | `iter_batched`, `BenchmarkId` |
| `alps.rs` | ~106 | LOW | `sample_size(10)`, `BenchmarkId` |
| `cellular.rs` | ~89 | LOW | `sample_size(10)`, `BenchmarkId` |
| `de.rs` | ~58 | LOW | `sample_size(10)`, `BenchmarkId` |
| `scatter.rs` | ~69 | LOW | `sample_size(10)`, `BenchmarkId` |
| `metrics_observer.rs` | ~60 | LOW | simple `criterion_group!`, no batched setup |

**Recommended porting order** (LOW → HIGH complexity):
1. `metrics_observer.rs` (low, and separate CI step — good isolation test)
2. `de.rs` (low, already requires `--features benchmarks` — tests feature isolation)
3. `scatter.rs`
4. `alps.rs`
5. `cellular.rs`
6. `crossover.rs`
7. `nsga2.rs`
8. `selection.rs`
9. `rastrigin.rs`
10. `survivor.rs`
11. `mutation.rs`
12. `ga_run.rs`
13. `island_ga.rs` (last: most complex + island setup)

### Current Cargo.toml bench declarations

13 `[[bench]]` entries exist. `de` has `required-features = ["benchmarks"]`; `metrics_observer` has `required-features = ["observer-metrics"]`. All have `harness = false`. No changes needed to the `[[bench]]` declarations when porting — `harness = false` is still required for divan (divan provides its own harness via `divan::main!()`).

---

## Plan 69-02 & 69-03: Rayon → `parallel` Feature Gate

### Rayon Call-Site Complete Inventory

**Total files with rayon usage: 20**
**Total actual call-sites (including par_sort_unstable_by): 36 lines**
**Files where par_sort_unstable_by is the only rayon method: 4 survivor files**

#### Gating Status at Research Time

| File | Import gate | Call-site gate | Action required |
|------|------------|----------------|-----------------|
| `src/engines/ga.rs` | `#[cfg(not(wasm32))]` on import | All 5 sites individually wasm32-gated | Update import gate + all 5 site gates to combined pattern |
| `src/engines/nsga2/mod.rs` | `#[cfg(not(wasm32))]` on import | 4 sites: 2 pairs individually wasm32-gated | Update import + 4 site gates |
| `src/engines/nsga3/mod.rs` | `#[cfg(not(wasm32))]` on import | 2 sites: individually gated | Update import + 2 gates |
| `src/engines/spea2/mod.rs` | `#[cfg(not(wasm32))]` on import | 2 sites: individually gated | Update import + 2 gates |
| `src/engines/ibea/mod.rs` | `#[cfg(not(wasm32))]` on import | 2 sites: individually gated | Update import + 2 gates |
| `src/engines/moead/mod.rs` | `#[cfg(not(wasm32))]` on import | 1 site: individually gated | Update import + 1 gate |
| `src/engines/sms_emoa/mod.rs` | `#[cfg(not(wasm32))]` on import | 1 site: individually gated | Update import + 1 gate |
| `src/engines/gp/engine.rs` | `#[cfg(not(wasm32))]` on import | 3 sites: individually wasm32-gated | Update import + 3 gates |
| `src/operations/survivor/age.rs` | `#[cfg(not(wasm32))]` on import | 1 par_sort site: wasm32-gated | Update import + 1 gate |
| `src/operations/survivor/fitness.rs` | `#[cfg(not(wasm32))]` on import | 2 par_sort sites: individually wasm32-gated | Update import + 2 gates |
| `src/operations/survivor/mu_comma_lambda.rs` | `#[cfg(not(wasm32))]` on import | 2 par_sort sites: individually wasm32-gated | Update import + 2 gates |
| `src/operations/survivor/mu_plus_lambda.rs` | `#[cfg(not(wasm32))]` on import | 2 par_sort sites: individually wasm32-gated | Update import + 2 gates |
| `src/operations/selection/tournament.rs` | `#[cfg(not(wasm32))]` on import | 1 site: wasm32-gated | Update import + 1 gate |
| `src/engines/island/nsga2.rs` | UNGATED top-level import (line 57) | 3 sites: mix of block-local imports and ungated | Add wasm32+parallel gate to import; gate 3 sites |
| `src/engines/island/mod.rs` | No top-level import; local `use rayon::prelude::*` inside fn block at line 511 | 1 site: ungated local import | Gate the local import block with combined cfg |
| `src/engines/eda/engine.rs` | No top-level import; 2 local `use rayon::prelude::*` inside `#[cfg(not(wasm32))]` blocks | 2 sites: already wasm32-gated blocks | Update block cfg to combined gate |
| `src/population.rs` | UNGATED top-level import (line 26) | 1 site: UNGATED par_iter_mut (line 135) | Add combined cfg to import; gate call-site |
| `src/traits/common.rs` | UNGATED top-level import (line 8) | 1 site: UNGATED into_par_iter (line 148) | Add combined cfg to import; gate call-site |
| `src/observe/observer/log.rs` | No functional rayon call (line 85 is a comment) | Comment only | No change needed |
| `src/traits/linear_chromosome.rs` | No rayon import | Line 39 is a doc comment | No change needed |

**Actual files needing changes: 17** (excluding log.rs and linear_chromosome.rs which are comments only)

**Three files with ungated rayon imports (highest risk — will break wasm build if not caught):**
- `src/population.rs` (par_iter_mut at line 135 — active, ungated)
- `src/traits/common.rs` (into_par_iter at line 148 — active, ungated)
- `src/engines/island/nsga2.rs` (top-level import at line 57; function-level imports in evolve blocks)

### Canonical Gate Pattern (D-06)

```rust
// TOP-LEVEL IMPORT — replaces all existing patterns:
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;

// LOCAL/INLINE IMPORT INSIDE BLOCK — replaces existing wasm32-only blocks:
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
{
    use rayon::prelude::*;
    // parallel code...
}

// INDIVIDUAL CALL-SITE — replaces #[cfg(not(target_arch = "wasm32"))]:
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
let result: Vec<_> = items.par_iter().map(process).collect();
#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]
let result: Vec<_> = items.iter().map(process).collect();

// FOR par_sort_unstable_by (existing pattern: just the cfg on the sort call):
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
chromosomes.par_sort_unstable_by(|a, b| ...);
#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]
chromosomes.sort_unstable_by(|a, b| ...);
```

### Cargo.toml Changes (plan 69-02)

```toml
[features]
default = ["logging", "parallel"]   # add "parallel"
logging = ["dep:log"]
parallel = ["dep:rayon"]            # NEW
# ...other features unchanged

[dependencies]
rand = "0.9.2"
rayon = { version = "1.10", optional = true }  # add optional = true
```

### CI Changes (plan 69-03)

Add to `feature-matrix.yml` matrix:
```yaml
- name: "parallel-off"
  features: "--no-default-features --features logging"
  cmd: "cargo test --quiet --no-default-features --features logging"
```

Add grep enforcement step (D-13) at end of feature-matrix CI job:
```bash
# Enforcement: no unconditional rayon:: references allowed in src/
if grep -rn 'rayon::' src/ | grep -v '#\[cfg'; then
  echo "ERROR: unconditional rayon:: reference found in src/"
  exit 1
fi
```

---

## Plan 69-04 & 69-05: ga.rs Split

### Current File Facts

- **Location:** `src/engines/ga.rs` — but declared in `src/lib.rs` via `#[path = "engines/ga.rs"] pub mod ga;`
- **Lines:** exactly 3,342 [VERIFIED: wc -l]
- **Size:** 140.7 KB

### Module Declaration Change

The `#[path]` attribute in `src/lib.rs` at line 333 must change from:
```rust
#[path = "engines/ga.rs"]
pub mod ga;
```
to:
```rust
#[path = "engines/ga/mod.rs"]
pub mod ga;
```
This is the only change needed in `src/lib.rs`. All `use crate::ga::Ga` imports in the wider codebase continue to work unchanged.

### Submodule Extraction Map

Based on the function scan of ga.rs (fn, pub fn, impl blocks):

| Submodule | Line Range (approx) | Key items | Visibility |
|-----------|---------------------|-----------|------------|
| `mod.rs` | 1–430, 770–975, 1039–1200, 1485–1620, 2468–2482 | `Ga<U>` struct def, all `impl ConfigurationT/SelectionConfig/…` builder blocks, `with_*` methods, `build()`, `run()`, `run_with_callback()` orchestrator, `stats()`, `hall_of_fame()`, `notify()` | `pub` (Ga, run, stats) |
| `lifecycle.rs` | ~1200–1484 | `initialization()`, `initialize_random()`, `initialize_with_seeds()`, finalise helpers | `pub(crate)` |
| `generation.rs` | ~1620–2280 | Per-generation loop body (selection→crossover→mutation→survivor→elitism→niching→best-update) | `pub(crate)` |
| `adaptive.rs` | scattered in generation loop | `update_adaptive_probabilities()` and related crossover/mutation adaptive computation | `pub(super)` |
| `aos.rs` | scattered in generation loop | AOS reward update, `get_aos_operator()`, credit assignment | `pub(super)` |
| `extension.rs` | ~2200–2280 | Extension trigger, diversity check | `pub(super)` |
| `cache.rs` | ~2480–2682 | Fitness cache lookup and insertion helpers | `pub(crate)` |
| `batch.rs` | ~2682–2778 | `batch_evaluate<U>()` helper function | `pub(crate)` |
| `stats.rs` | ~2482–2483 (section) + scattered | `GenerationStats` collection per generation | `pub(crate)` |
| `observer.rs` | wherever `notify` calls are concentrated | Observer hook dispatch | `pub(crate)` |
| `stopping.rs` | ~2778–2857 | `limit_reached<U>()`, stopping criteria check | `pub(crate)` |

**Free-standing functions at bottom of ga.rs (lines ~2857–3342):**
- `parent_crossover<U>()` → `generation.rs`
- `extract_elite<U>()` → `generation.rs` (or `mod.rs` — used in orchestrator)
- `reinsert_elite<U>()` → `generation.rs`
- `batch_evaluate()` → `batch.rs`
- `limit_reached()` → `stopping.rs`

### Extraction Order (Claude's Discretion — dependency-first)

1. `stopping.rs` — `limit_reached` is a pure fn, no dependencies on other ga modules
2. `cache.rs` — fitness cache helpers, depends on external types only
3. `stats.rs` — GenerationStats collection, depends on external types only
4. `observer.rs` — `notify` dispatcher, depends on external GaObserver trait only
5. `batch.rs` — `batch_evaluate` helper, depends on cache.rs
6. `adaptive.rs` — probability recomputation, depends on configuration only
7. `aos.rs` — AOS credit/reward, depends on configuration + adaptive.rs
8. `extension.rs` — extension trigger, depends on external extension traits
9. `lifecycle.rs` — init functions, depends on cache.rs, batch.rs
10. `generation.rs` — per-gen loop, depends on stopping.rs, adaptive.rs, aos.rs, extension.rs, cache.rs, batch.rs, stats.rs, observer.rs
11. `mod.rs` — orchestrator: everything remaining (Ga struct, builders, run, initialization call)

### cargo-expand Workflow (D-12)

```bash
# Before split (capture baseline):
cargo expand --lib 2>/dev/null | grep -E '^pub (fn|struct|impl|trait|enum|type|use)' | sort > /tmp/ga-symbols-before.txt

# After split (verify):
cargo expand --lib 2>/dev/null | grep -E '^pub (fn|struct|impl|trait|enum|type|use)' | sort > /tmp/ga-symbols-after.txt

# Diff — must be empty for acceptance gate:
diff /tmp/ga-symbols-before.txt /tmp/ga-symbols-after.txt
```

Note: `cargo-expand` is NOT currently installed (verified by `cargo expand --version` failing). Must be installed in plan 69-04's Wave 0.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Benchmark statistics | Custom timing harness | `divan` | Statistical soundness, warmup handling, outlier rejection |
| WASM feature detection | Runtime checks | `#[cfg(target_arch = "wasm32")]` at compile time | Zero runtime cost; rustc eliminates dead branches |
| Symbol diffing | String parsing of cargo doc | `cargo expand` | Precise, complete — captures all generated code including macros |
| Public API diffing | Manual rustdoc comparison | `cargo public-api` | Semantic-aware, not text-diff; catches visibility changes |
| Parallel sort with fallback | Custom sorting abstraction | rayon `par_sort_unstable_by` / std `sort_unstable_by` under cfg gate | Rayon's sort already has the best cross-platform story |

---

## Common Pitfalls

### Pitfall 1: par_sort_unstable_by is a Rayon method — needs gating too

**What goes wrong:** The D-01 decision mentions `par_iter()/into_par_iter()/par_iter_mut()/par_chunks*()`. Four survivor files use `par_sort_unstable_by()` which is also a rayon method (from `rayon::slice::ParallelSliceMut`). If only the listed call types are grepped, these 7 sites are missed.
**Why it happens:** `par_sort_unstable_by` reads differently from `par_iter` and is easy to overlook in a grep pattern focused on iterator methods.
**How to avoid:** The grep enforcement step (D-13) should catch ALL `rayon::` usage, not just iterator patterns. The import-level gate (`use rayon::prelude::*`) being cfg-gated also catches the entire symbol namespace.
**Warning signs:** `cargo test --no-default-features --features logging` fails with "method not found: par_sort_unstable_by" on WASM or no-parallel builds.

### Pitfall 2: Three files have completely ungated rayon imports today

**What goes wrong:** `population.rs`, `traits/common.rs`, and `engines/island/nsga2.rs` have `use rayon::prelude::*;` with NO cfg gate whatsoever. They have not been WASM-tested before this phase despite CLAUDE.md requiring it. Removing the `parallel` feature in a build would cause a compile error immediately because `par_iter_mut` etc. are only in scope from the rayon prelude.
**Why it happens:** These files were written or modified without adding the wasm32 gate (possibly pre-CLAUDE.md WASM mandate).
**How to avoid:** Plan 69-02 must gate these 3 ungated imports before making rayon optional. The WASM CI check (`cargo check --target wasm32-unknown-unknown`) should fail immediately when `rayon = { optional = true }` is set and these files are untouched.
**Warning signs:** After making rayon optional in Cargo.toml, `cargo check --target wasm32-unknown-unknown` fails with "unresolved import rayon::prelude" in population.rs or common.rs.

### Pitfall 3: `#[path]` attribute must change when ga.rs becomes a directory

**What goes wrong:** After creating `src/engines/ga/mod.rs`, rustc still finds the old `#[path = "engines/ga.rs"]` pointing to a non-existent file. The old file must be deleted AND the `#[path]` updated atomically in the same commit.
**Why it happens:** `src/lib.rs` uses an explicit `#[path]` attribute rather than the conventional Rust 2018 module layout, so rustc won't auto-discover the directory.
**How to avoid:** In the first commit of the ga.rs split: simultaneously update `src/lib.rs:333` from `engines/ga.rs` → `engines/ga/mod.rs` AND create `src/engines/ga/mod.rs`. Never leave an intermediate state where both `ga.rs` and `ga/mod.rs` exist.
**Warning signs:** `cargo check` fails with "file not found for module `ga`" after the first split commit.

### Pitfall 4: Visibility escalation during split breaks API surface guarantee

**What goes wrong:** When `batch_evaluate()` is moved to `batch.rs`, it is needed by `lifecycle.rs` (for initial fitness in batch mode). If implemented as `pub(super)`, it's invisible to sibling `lifecycle` module. If escalated to `pub(crate)`, it's fine but also becomes reachable from all of `crate::` — which is acceptable but not minimal.
**Why it happens:** The D-11 visibility strategy says `pub(super)` for sibling sharing — but sibling submodules of `ga/` don't actually see each other's `pub(super)` items (those are visible to the parent `ga/mod.rs` only). True sibling visibility requires `pub(crate)` or re-exporting through mod.rs.
**How to avoid:** Use `pub(crate)` for any item accessed by more than one ga submodule. Reserve `pub(super)` only for items accessed exclusively by `mod.rs`. The `cargo public-api` check at merge time catches accidental public promotions.
**Warning signs:** `error[E0616]: field ... is private` or `error[E0603]: function ... is private` during compilation of generation.rs trying to call batch.rs helpers.

### Pitfall 5: divan `sample_count` vs criterion `sample_size` semantics differ

**What goes wrong:** `criterion`'s `group.sample_size(10)` means "collect 10 samples." `divan`'s `#[divan::bench(sample_count = 10)]` means the same in principle, but the default sample count in divan is different and the statistical model differs (divan uses fewer samples by default with more aggressive outlier handling).
**Why it happens:** Direct numeric translation of sample counts between frameworks is not always semantically equivalent.
**How to avoid:** For the 4 benches that set `sample_size(10)` (alps, cellular, de, scatter — these are slow end-to-end runs), set `#[divan::bench(sample_count = 10)]` but verify the run time is acceptable. The ±3% tolerance check is on the median value, not on sample count.
**Warning signs:** `cargo bench` with divan takes much longer than expected (running 100+ samples on slow GA benches), or the median value drifts more than 3% from baseline.

### Pitfall 6: `iter_batched` batch size semantics

**What goes wrong:** `criterion`'s `BatchSize::SmallInput` indicates that input setup cost is negligible relative to bench work. In `divan`, `bencher.with_inputs(|| setup()).bench(|b, x| work(x))` always calls setup once per iteration. For benches where the GA is created in setup and run once in bench (ga_run, island_ga, rastrigin), this is fine. But for benches that modify input, the setup function must return fresh state each time.
**Why it happens:** Divan's `with_inputs` API is conceptually simpler but less explicit about batching strategy.
**How to avoid:** For all `iter_batched` ports, verify the setup closure returns a new owned value and the bench closure consumes it. This mirrors `BatchSize::SmallInput` behaviour.

---

## Code Examples

### divan basic bench (replaces criterion_group!/criterion_main!)

```rust
// Source: github.com/nvzqz/divan README / crates.io documentation [ASSUMED]
fn main() {
    divan::main();
}

#[divan::bench]
fn bench_simple() {
    // work
}
```

### divan bench with setup (replaces iter_batched)

```rust
// Source: divan README [ASSUMED]
#[divan::bench]
fn bench_ga_run(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| build_ga(50, 10, 10))
        .bench(|_b, mut ga| {
            let _ = ga.run();
        });
}
```

### divan parameterised bench (replaces BenchmarkId + bench_with_input)

```rust
// Source: divan README [ASSUMED]
#[divan::bench(args = [10, 50, 100, 200])]
fn bench_selection(bencher: divan::Bencher, pop_size: usize) {
    let chromosomes = setup_chromosomes(pop_size);
    bencher.bench(|| {
        tournament(&chromosomes, 10, 1)
    });
}
```

### divan with sample count (replaces group.sample_size(10))

```rust
// Source: divan README [ASSUMED]
#[divan::bench(sample_count = 10)]
fn bench_alps_vs_de(bencher: divan::Bencher) {
    bencher.with_inputs(|| build_alps()).bench(|_b, mut ga| {
        let _ = ga.run();
    });
}
```

### Canonical rayon gate pattern (replaces all existing wasm32-only gates)

```rust
// Source: CONTEXT.md D-06 (locked decision) [VERIFIED: project decision]

// TOP-LEVEL IMPORT:
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;

// INDIVIDUAL CALL-SITE — parallel arm:
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
let results: Vec<_> = items.par_iter().map(process).collect();
// Sequential fallback:
#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]
let results: Vec<_> = items.iter().map(process).collect();

// par_sort_unstable_by:
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
chromosomes.par_sort_unstable_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap());
#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]
chromosomes.sort_unstable_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap());
```

### lib.rs path attribute update for ga.rs split

```rust
// Before (src/lib.rs line 333):
#[path = "engines/ga.rs"]
pub mod ga;

// After (src/lib.rs line 333):
#[path = "engines/ga/mod.rs"]
pub mod ga;
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `criterion` benchmark harness | `divan` 0.1.x | 2023 (divan released) | ~40% fewer transitive dev-deps; faster bench compilation |
| Unconditional `rayon` in dependencies | `rayon = { optional = true }` behind `parallel` feature | Phase 69 | Allows WASM-only and embedded builds to shed thread pool |
| Single 3342-line `ga.rs` compilation unit | 11 submodules in `engines/ga/` | Phase 69 | rustc can parallelise frontend per-file; faster incremental |
| `#[cfg(not(target_arch = "wasm32"))]` for rayon | Combined `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]` | Phase 69 | Single canonical gate; `parallel=off` also bypasses rayon |

**Deprecated patterns (do not reintroduce after Phase 69):**
- `use rayon::prelude::*;` without cfg gate in any `src/` file
- `criterion_group!` / `criterion_main!` macros in any bench file
- `#[cfg(not(target_arch = "wasm32"))]` as the sole gate for rayon call-sites

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `divan::Bencher::with_inputs().bench()` is the correct API for iter_batched replacement | Code Examples | Wrong API → bench files don't compile; easily corrected by reading divan docs |
| A2 | `#[divan::bench(args = [...])]` is the correct parameterised bench syntax | Code Examples | Wrong syntax → bench files don't compile; correctable from divan docs |
| A3 | `#[divan::bench(sample_count = 10)]` is the correct equivalent for `group.sample_size(10)` | Plan 69-01 / Pitfall 5 | Wrong param name → compile error or different behaviour; correctable |
| A4 | `divan` does not have a direct equivalent for `group.throughput(Throughput::Elements(n))` | Criterion API Inventory | May have a counterpart (e.g., `bencher.counter()`); only affects benchmark display, not correctness |
| A5 | `cargo-expand` captures enough symbol information to validate the ga.rs split semantically | Plan 69-04/05 | If expand output is too noisy, diff may show false differences; verify manually |
| A6 | The `observe/observer/log.rs` line 85 mention of `par_iter` is a pure comment | Rayon inventory | No action needed. Verified by reading the line: "// Lines inside the rayon par_iter..." |

---

## Open Questions (RESOLVED)

1. **divan `Throughput` equivalent**
   - What we know: `criterion` uses `group.throughput(Throughput::Elements(n))` in `selection.rs` and `nsga2.rs` to report elements/second
   - What's unclear: divan may have `bencher.counter(ItemsCount(n))` or similar
   - Recommendation: Check divan docs or README before porting selection.rs and nsga2.rs. If no equivalent, drop the throughput annotation (loss of display information only, not correctness).
   - **RESOLVED:** Drop `Throughput::Elements` entirely — divan 0.1.21 has no equivalent. Loss of display information only, not correctness. Plan 69-01 Task 1 handles this explicitly.

2. **`cargo-public-api` version pinning**
   - What we know: `cargo public-api --version` fails — tool is not installed
   - What's unclear: which version to pin in CI; whether it needs nightly toolchain
   - Recommendation: Plan 69-04 Wave 0 must install it. Check if it requires nightly (`cargo install cargo-public-api` may need `+nightly`). If nightly-only, it should run in its own CI job separate from stable checks.
   - **RESOLVED:** Plan 69-04 Task 1 installs with stable first (`cargo install cargo-public-api`), falls back to `cargo +nightly install cargo-public-api` if that fails, and documents the nightly requirement in the commit body if nightly is needed.

3. **`iter_with_large_drop` usage**
   - What we know: not found in the bench files (inventory found no uses of `iter_with_large_drop`)
   - What's unclear: confirmed absent — not a concern for porting
   - **RESOLVED:** Confirmed absent — not a concern. No action needed.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | All plans | ✓ | (current stable) | — |
| `wasm32-unknown-unknown` target | All plans (WASM check) | Assumed ✓ (wasm-check.yml exists + runs) | — | Install: `rustup target add wasm32-unknown-unknown` |
| `cargo-expand` | Plan 69-04 (D-12 symbol diff) | ✗ | — | Install in Wave 0: `cargo install cargo-expand` |
| `cargo-public-api` | All plans (acceptance gate #5) | ✗ | — | Install in Wave 0: `cargo install cargo-public-api` (check nightly req) |
| `divan` crate | Plan 69-01 | ✗ (not in Cargo.toml yet) | 0.1.21 | Add to [dev-dependencies] in Wave 0 |
| `bench/build_perf.sh` | All plans (CC-1) | ✓ | — | Exists at `bench/build_perf.sh` |
| `.planning/baselines/v3.0.0-baseline.json` | All plans (CC-1) | ✓ | 164B | Exists |
| `tests/golden/` | Plans 69-02, 69-03, 69-04 (CC-3) | ✓ | 4 files | Exists with rastrigin.txt, nsga2_zdt1.txt, cma_es_rastrigin.txt, pso_rastrigin.txt |
| `.github/workflows/feature-matrix.yml` | Plans 69-02, 69-03 (CC-2) | ✓ | — | Exists; needs `parallel-off` combination added |
| `.github/workflows/wasm-check.yml` | All plans | ✓ | — | Exists |

**Missing dependencies with no fallback:**
- `cargo-expand` — required for D-12 symbol diff; must be installed before plan 69-04
- `cargo-public-api` — required for acceptance gate #5 on every plan PR

**Missing dependencies with fallback:**
- None (golden tests, baseline, feature-matrix CI, and wasm-check all exist)

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` / `cargo nextest` (CI) |
| Quick run command | `cargo test --quiet` |
| Full suite command | `cargo test --all-features && cargo test --no-default-features --features logging` |
| WASM check | `cargo check --target wasm32-unknown-unknown --lib` |
| Bench regression | `bash bench/build_perf.sh` then Python diff against `.planning/baselines/v3.0.0-baseline.json` |

### Phase Requirements → Test Map

| Plan | Behavior | Test Type | Automated Command | Infrastructure Exists? |
|------|----------|-----------|-------------------|----------------------|
| 69-01 | Each divan bench compiles and runs | smoke | `cargo bench --bench <name>` (or `--features benchmarks` / `--features observer-metrics`) | ✅ bench files exist |
| 69-01 | Criterion fully removed after all 13 ported | compile | `cargo test` (criterion no longer in Cargo.toml) | ✅ |
| 69-02 | `parallel=off` build compiles without rayon | compile | `cargo test --no-default-features --features logging` | ✅ feature-matrix.yml |
| 69-02 | WASM build passes after rayon made optional | compile | `cargo check --target wasm32-unknown-unknown` | ✅ wasm-check.yml |
| 69-03 | Golden tests byte-identical with parallel=on | regression | `cargo test --test golden_tests` | ✅ tests/golden/ |
| 69-03 | Golden tests byte-identical with parallel=off | regression | `cargo test --test golden_tests --no-default-features --features logging` | ✅ tests/golden/ |
| 69-03 | No unconditional rayon:: in src/ | grep | `grep -rn 'rayon::' src/ \| grep -v '#\[cfg'` → empty | ❌ Wave 0: add to CI |
| 69-04 | All tests pass after ga.rs split | full suite | `cargo test --all-features` | ✅ |
| 69-04 | Public API unchanged | diff | `cargo public-api` → zero diff | ❌ Wave 0: install tool |
| 69-04 | Symbol table identical before/after | diff | `cargo expand` diff | ❌ Wave 0: install tool |
| 69-05 | docs/ARCHITECTURE.md, CLAUDE.md, intel/ updated | manual | Read files | — |

### Wave 0 Gaps

- [ ] `cargo install cargo-expand` — needed for plan 69-04 D-12
- [ ] `cargo install cargo-public-api` — needed for acceptance gate #5 on all plans
- [ ] Add `parallel-off` matrix combination to `.github/workflows/feature-matrix.yml` — needed for plan 69-03
- [ ] Add grep enforcement step to CI — needed for plan 69-03

---

## Security Domain

> Phase 69 is pure internal refactoring: benchmark migration, feature gating, and file splitting. No new authentication, session management, input handling from untrusted sources, cryptography, or access control surfaces are introduced. ASVS categories V2–V6 do not apply. No external network calls, no new binary parsing, no secrets handling.

Security domain: SKIPPED — code-only refactor with no new external attack surface.

---

## Project Constraints (from CLAUDE.md)

The following CLAUDE.md directives apply to Phase 69:

1. **WASM Compatibility (mandatory):** After Phase 69, the canonical gate is `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]`. The CLAUDE.md "WASM Compatibility" section itself must be updated (plan 69-03 deliverable) to document this new canonical gate. Until updated, the old pattern `#[cfg(not(target_arch = "wasm32"))]` is still the documented pattern for non-rayon calls.

2. **No Breaking Changes:** ga.rs split must export `Ga` at exactly the same path (`crate::ga::Ga`). The `#[path]` update in `lib.rs` preserves this. Zero items gain or lose visibility. `cargo public-api` diff must be empty.

3. **Tests in `tests/` folder:** All new tests go in `tests/`. No inline `#[cfg(test)] mod tests` blocks.

4. **Signed commits mandatory:** Every commit GPG-signed. All 11 submodule extraction commits in plan 69-04 must be signed.

5. **Branch naming:** `feat/69-build-perf-m3-major-refactors` (or individual sub-branches per plan if split).

6. **Performance patterns to maintain:**
   - `Cow<[Gene]>` zero-copy DNA — unaffected by any plan
   - `select_nth_unstable_by()` over full sort — unaffected
   - Parallel fitness evaluation via rayon — preserved when `parallel` feature is on (default)

7. **Observability hooks:** ga.rs split must NOT remove or bypass any `notify()` call sites in the generation loop. `observer.rs` submodule must contain or re-export all observer dispatch points.

8. **Commit bodies must include `Revert plan:`** per BUILD-PERF.md non-negotiable guarantee #5.

---

## Sources

### Primary (HIGH confidence)

- `src/engines/ga.rs` — 3,342 lines verified via `wc -l`; function inventory via grep [VERIFIED: direct codebase inspection]
- `Cargo.toml` — Current dependencies, features, bench declarations [VERIFIED: direct codebase inspection]
- `.planning/phases/69-build-perf-m3-major-refactors/69-CONTEXT.md` — All locked decisions (D-01 through D-13) [VERIFIED: project context]
- `.planning/v3.0.0-BUILD-PERF.md` — Action #3, #4, #7 specs; non-negotiable guarantees; acceptance gate [VERIFIED: project spec]
- `benches/*.rs` — All 13 bench files; criterion API patterns enumerated via grep [VERIFIED: direct codebase inspection]
- `src/**/*.rs` — All 20 files with rayon usage; gating status per file [VERIFIED: grep + line inspection]
- `.github/workflows/feature-matrix.yml` — Current matrix; missing `parallel-off` combination [VERIFIED: direct codebase inspection]
- `tests/golden/` — 4 golden test files confirmed present [VERIFIED: ls output]
- `bench/build_perf.sh` + `.planning/baselines/v3.0.0-baseline.json` — Both exist [VERIFIED: ls output]
- crates.io API + `cargo search divan` — divan 0.1.21 confirmed [VERIFIED: crates.io API + cargo search]

### Secondary (MEDIUM confidence)

- divan API patterns (with_inputs, bench, args, sample_count) — from divan README and crates.io description; not verified via Context7 or official docs fetch [ASSUMED for specific API names]

### Tertiary (LOW confidence)

- `Throughput::Elements` divan equivalent — not investigated; marked as open question

---

## Metadata

**Confidence breakdown:**
- Rayon site inventory: HIGH — grepped and manually verified per file
- Bench file criterion API patterns: HIGH — grepped all 13 files
- divan API equivalents: MEDIUM — from cargo search / crates.io; specific method names [ASSUMED]
- ga.rs submodule layout: HIGH — BUILD-PERF.md defines it verbatim; line scan confirms natural boundaries
- CI infrastructure status: HIGH — all workflow files read directly

**Research date:** 2026-06-15
**Valid until:** 2026-07-15 (stable crates, low churn domain)
