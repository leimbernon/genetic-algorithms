# Phase 47: Architecture Audit & ChromosomeT Split - Context

**Gathered:** 2026-05-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 47 delivers the foundational breaking changes that shape all of v3.0.0: split `ChromosomeT` into a minimal core trait and a `LinearChromosome` supertrait, encapsulate `GaConfiguration`, introduce `ChromosomeLength` enum, flatten `StoppingCriteria`, and remove the deprecated `Reporter` trait. These changes MUST land and pass CI before any downstream feature branch (Phase 48+) opens.

</domain>

<decisions>
## Implementation Decisions

### ChromosomeT / LinearChromosome Trait Split (ARCH-01, ARCH-02)

- **D-01:** `ChromosomeT` retains only the minimal core contract: `fitness()`, `set_fitness()`, `calculate_fitness()`, `age()`, `set_age()`, `fitness_distance()`. No flat-slice methods and no fitness function installation.
- **D-02:** `LinearChromosome: ChromosomeT` is the supertrait for all flat-slice chromosomes. It adds: `dna()`, `dna_mut()`, `set_dna()`, `set_fitness_fn()`, `new_gene()`, and provides **default implementations** of `set_gene()` (index-bounds-checked, uses `dna_mut()`) and `reset()`.
- **D-03:** The `default(mut self) -> Self` reset helper is **renamed to `reset() -> &mut Self`** on `LinearChromosome`. This removes the shadowing ambiguity with the `Default` trait and adopts a builder-style `&mut Self` return. Existing `BinaryChromosome` and `RangeChromosome` implementors get it for free via the default impl.
- **D-04:** `ChromosomeT` does **not** have any fitness function installation method. `calculate_fitness()` is the only method — implementors wire up the fitness function however they like. `set_fitness_fn<F>()` stays on `LinearChromosome` only (its closure signature references `Self::Gene`, a flat-slice concept).
- **D-05:** The mechanical bound change (`U: ChromosomeT` → `U: LinearChromosome`) across ~30 operator files is done via **`sed` + `cargo check` loop** — not manual per-file edits. Executor uses: `sed -i 's/U: ChromosomeT/U: LinearChromosome/g'` on operator files, then `cargo check` to catch misses and over-broad hits.

### Configuration Cleanup (ARCH-04, ARCH-05, ARCH-06)

- **D-06:** `LimitConfiguration.needs_unique_ids` and `LimitConfiguration.alleles_can_be_repeated` are **removed without replacement** in Phase 47. Existing initializers (`BinaryInitializer`, `RangeInitializer`) simply drop the uniqueness enforcement logic — Phase 48's `UniqueChromosome` type enforces uniqueness at the type level. No bridge struct is needed.
- **D-07:** `ChromosomeLength` is a **standalone public type** — not inline in `configuration.rs`. Lives in a new file (e.g., `src/chromosomes/length.rs` or `src/types/length.rs`) and is re-exported from `lib.rs` as a first-class public type alongside `ChromosomeT`. Variants: `ChromosomeLength::Fixed(usize)` and `ChromosomeLength::Variable { min: usize, max: usize }`.
- **D-08:** `StoppingCriteria` struct is **removed entirely** from the type system. Its 3 fields (`stagnation_generations: Option<usize>`, `convergence_threshold: Option<f64>`, `max_duration_secs: Option<f64>`) become direct `pub(crate)` fields on `GaConfiguration`. Builder methods (`.with_stagnation_limit()`, `.with_convergence_threshold()`, `.with_max_duration_secs()`) set them. Serde serializes them as flat fields on the config object.
- **D-09:** `GaConfiguration` fields become `pub(crate)` with read-only public accessors. Accessor granularity: **sub-struct level** (e.g., `pub fn limit(&self) -> &LimitConfiguration`, `pub fn selection(&self) -> &SelectionConfiguration`).

### Reporter Removal (ARCH-03)

- **D-10:** `Reporter<U>` trait is **removed entirely** in v3.0.0. Users migrate to `GaObserver<U>` (available since v2.2.0, already the preferred API). `with_reporter()` builder method is removed from `Ga`.

### MIGRATION.md (ARCH-03 expanded)

- **D-11:** Publish a **full v3.0.0 breaking changes guide** at the crate root (`MIGRATION.md`, alongside `README.md`). Covers all Phase 47 breaking changes with before/after code examples: Reporter → GaObserver, `genes_per_chromosome` → `ChromosomeLength`, `StoppingCriteria` struct → flat builder methods, `LimitConfiguration` field removals, `LinearChromosome` trait requirement for custom chromosome types. Include in `Cargo.toml` `include` list. Link from `README.md`.

### Examples CI (ARCH-07)

- **D-12:** ARCH-07 examples smoke test lands as a **new CI workflow file**: `.github/workflows/examples-smoke.yml`. Triggers on pushes/PRs to the milestone branch only. Compiles and runs each of the 10 examples with a short generation count.

### PR Execution Strategy

- **D-13:** Phase 47 lands as **3 staged PRs** on the milestone branch:
  - **PR 1** — ChromosomeT split: ARCH-01 + ARCH-02 (trait definition + mechanical bound change across ~30 operator files)
  - **PR 2** — Config cleanup: ARCH-04 + ARCH-05 + ARCH-06 (GaConfiguration encapsulation, ChromosomeLength, StoppingCriteria flattening)
  - **PR 3** — Reporter removal + CI: ARCH-03 + ARCH-07 (Reporter removal, MIGRATION.md, examples-smoke.yml)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §ARCH — ARCH-01 through ARCH-07 requirements (the authoritative scope for this phase)

### Current Trait Definitions (to be changed)
- `src/traits/chromosome.rs` — current `ChromosomeT` trait; the entire contents move/split
- `src/traits/operators.rs` — operator trait signatures (all use `U: ChromosomeT` today)

### Current Configuration (to be changed)
- `src/configuration.rs` — `GaConfiguration`, `LimitConfiguration`, `StoppingCriteria`, all sub-configs; full contents must be read before planning ARCH-04/05/06

### Reporter (to be removed)
- `src/observe/reporter/` — `Reporter<U>` trait and `DurationReporter`, `SimpleReporter` impls
- `src/engines/ga.rs` lines ~280, ~850-856, ~1974 — `with_reporter()` builder and legacy fire point

### Operator Files (bound change targets)
- `src/operations/` — all subdirectories; ~30 files with `U: ChromosomeT` bounds to change to `U: LinearChromosome`

### Existing Chromosome Implementors (gain LinearChromosome defaults)
- `src/chromosomes/binary.rs` — `BinaryChromosome` implementation
- `src/chromosomes/range.rs` — `RangeChromosome<T>` implementation

### Examples (must still compile after ARCH-07)
- `examples/` — 10 examples; all must compile and run with a short generation count in CI

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/traits/chromosome.rs`: current ChromosomeT is 98 lines — the split creates two files; `LinearChromosome` inherits most content, `ChromosomeT` becomes ~20 lines (fitness + age only)
- `src/observe/observer/mod.rs`: `GaObserver<U>` is the replacement for `Reporter<U>` — already wired into all 4 alt-metaheuristic engines; MIGRATION.md can point directly to its usage examples
- Enum + factory pattern (used by all operators): consistent with `ChromosomeLength` enum placement decision

### Established Patterns
- `pub(crate)` field + public accessor: already used in some sub-configs; ARCH-04 extends this pattern to all of `GaConfiguration`
- `#[deprecated(since = "...", note = "...")]` attribute: already applied to `with_reporter()` in `src/engines/ga.rs:854` — removal in Phase 47 completes the deprecation lifecycle
- `#[cfg(not(target_arch = "wasm32"))]` gates: used throughout for `Instant` and `par_iter()` — `max_duration_secs` field in the flattened config must preserve this pattern (WASM mandatory constraint)

### Integration Points
- `src/engines/ga.rs`: primary integration point for all ARCH changes — ChromosomeT → LinearChromosome bounds, Reporter removal, GaConfiguration encapsulation
- `src/island/mod.rs`, `src/nsga2/mod.rs`, and the 4 alt-engine files in `src/engines/`: all use `U: ChromosomeT` at the orchestrator level → `U: LinearChromosome`
- `src/initializers/`: `BinaryInitializer` and `RangeInitializer` lose the `needs_unique_ids` / `alleles_can_be_repeated` checks

</code_context>

<specifics>
## Specific Ideas

- Sed approach for the mechanical bound change: `sed -i 's/U: ChromosomeT/U: LinearChromosome/g'` across `src/operations/` and orchestrator files, then `cargo check --all-features` to surface any remaining hits or false positives
- `examples-smoke.yml` should run examples with a short count (e.g., `--features ""` default, 5-10 generations) — not a full benchmark run; purpose is compile + minimal sanity check
- `MIGRATION.md` at crate root; add `"MIGRATION.md"` to the `include` array in `Cargo.toml` so it ships with the published crate

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 47-architecture-audit-chromosomet-split*
*Context gathered: 2026-05-19*
