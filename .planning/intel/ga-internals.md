# GA Engine Internals: src/engines/ga/

AI-readable note for future agents working on the single-objective GA engine.
Phase 69 Action #4 split `src/engines/ga.rs` (3342 lines, 139.2 KB) into 11 cohesive submodules.

## Why this layout exists

`src/engines/ga.rs` was the single most-loaded compilation unit in the entire codebase — 3342 lines,
139.2 KB, containing the GA struct definition, all builder impls, the per-generation hot loop, adaptive
operator selection, fitness caching, lifecycle management, observer dispatch, and stopping criteria.

Splitting it into a directory module (`src/engines/ga/`) with 11 submodules enables:

- **rustc frontend parallelisation** — the compiler can parse and lower each submodule concurrently.
- **Faster incremental compile** — a change to `stopping.rs` does not invalidate the code-gen unit for
  `generation.rs` or `lifecycle.rs`.
- **Focused diffs** — a PR touching only observer wiring affects `observer.rs` alone, not a 3342-line file.
- **AI context clarity** — agents and reviewers can read one submodule in isolation without scrolling past
  unrelated logic.

This split is a pure move refactor: zero semantic change, zero public API change. Verified via symbol diff,
`cargo public-api`, and 1661 passing tests.

## Submodule responsibilities

| File | Key items | Visibility |
|------|-----------|------------|
| `mod.rs` | `Ga<U>` struct definition; all `impl ConfigurationT / SelectionConfig / …` builder blocks; `with_*` builder methods; `build()`; `run()`; `run_with_callback()` orchestrator; `stats()`; `hall_of_fame()`; `notify()` dispatch; constraint helpers | `pub` (Ga, run, stats, hall_of_fame) |
| `lifecycle.rs` | `initialization()`, `initialize_random()`, `initialize_with_seeds()`, finalise helpers (separate `impl Ga<U>` block) | `pub(crate)` |
| `generation.rs` | `ParentCrossoverParams` struct; `parent_crossover()`; `extract_elite()`; `reinsert_elite()`; the per-generation loop body (selection → crossover → mutation → survivor → elitism → niching → best-update) | `pub(crate)` |
| `adaptive.rs` | `update_dynamic_mutation()` helper; adaptive crossover/mutation probability recomputation | `pub(crate)` |
| `aos.rs` | `init_aos_state()` helper; AOS credit assignment, reward update, `get_aos_operator()` | `pub(crate)` |
| `extension.rs` | `should_trigger_extension()` helper; extension trigger + diversity check | `pub(crate)` |
| `cache.rs` | `cache_snapshot()` and `cache_fill_stats()` helpers; fitness cache lookup and insertion | `pub(crate)` |
| `batch.rs` | `batch_evaluate<U>()` free function; batch fitness evaluation entry point | `pub(crate)` |
| `stats.rs` | `collect_generation_stats()` helper; `GenerationStats` collection per generation | `pub(crate)` |
| `observer.rs` | `dispatch()` free function; observer hook dispatch; called by `Ga::notify()` in `mod.rs` | `pub(crate)` |
| `stopping.rs` | `limit_reached<U>()` free function; stopping criteria check (generation limit, fitness target, stagnation, convergence, time limit) | `pub(crate)` |

## Visibility rules (D-11)

- Currently-`pub` items (`Ga`, `run`, `build`, `stats`, `hall_of_fame`, `TerminationCause`) **MUST remain `pub`**.
  Zero public surface change is an invariant of this split.
- Items called from 2+ ga submodules use `pub(crate)`. **Pitfall: `pub(super)` is NOT visible to siblings** —
  using `pub(super)` where `pub(crate)` is needed silently restricts access and will cause compilation errors.
- Items called only from `mod.rs` use `pub(super)`.
- Items used only within their own submodule use `pub(super)` or no visibility modifier (private to the module).

## What an agent must NOT reintroduce

- **Do NOT re-merge submodules into a single `ga.rs`**. The split is permanent. If you need to add a new
  lifecycle step, add it to the appropriate submodule.
- **Do NOT add new fields to `Ga<U>` or new `pub` items** without verifying `cargo public-api` diff is
  intentional and SemVer-additive.
- **Do NOT remove or bypass `notify()` call sites in `generation.rs` / `observer.rs`**. The `GaObserver`
  observability mandate in `CLAUDE.md` requires all lifecycle hooks to fire at the correct point.
  There are currently 13 `notify()` calls across the `ga/` directory (12 in `mod.rs` + 1 in `observer.rs`);
  this count must never decrease.
- **Do NOT escalate `pub(super)` → `pub` or `pub(crate)` → `pub`** without an explicit v3.x SemVer-additive
  review. The public API surface must remain stable within a major version.

## How to verify the invariant

```bash
# Symbol table diff against pre-split baseline
cargo expand --lib 2>/dev/null \
  | grep -E '^pub (fn|struct|impl|trait|enum|type|use)' \
  | sort \
  > /tmp/ga-symbols-after.txt
diff .planning/phases/69-build-perf-m3-major-refactors/ga-symbols-before.txt /tmp/ga-symbols-after.txt

# Public API diff (stable rustc sufficient — no nightly required)
cargo public-api > /tmp/ga-public-api-after.txt
diff .planning/phases/69-build-perf-m3-major-refactors/ga-public-api-before.txt /tmp/ga-public-api-after.txt

# Full test suite under both feature combinations
cargo test --all-features
cargo test --no-default-features --features logging

# WASM compatibility
cargo check --target wasm32-unknown-unknown --lib

# Rustdoc zero warnings
cargo doc --no-deps 2>&1 | grep warning
```

## Where to put new code

Decision tree for future agents adding features to the GA engine:

- **New stopping criterion** → `stopping.rs`
- **New cache strategy or cache invalidation logic** → `cache.rs`
- **New stat field on `GenerationStats`** → `stats.rs` (and update the `GenerationStats` struct)
- **New observer hook** → `observer.rs` (preserve existing `notify` call sites; add new hook to `GaObserver` trait in `src/observe/observer/mod.rs`)
- **New per-generation step** → `generation.rs` (within the existing loop body in `run_with_callback`)
- **New init strategy or population seeding logic** → `lifecycle.rs`
- **New adaptive parameter** → `adaptive.rs`
- **New AOS algorithm or credit assignment strategy** → `aos.rs`
- **New extension strategy** → `extension.rs` (the `ExtensionOperator` impl itself goes in `src/operations/extension/`)
- **New batch evaluation helper** → `batch.rs`
- **Builder API change, new top-level `Ga` method, or orchestrator change** → `mod.rs`
