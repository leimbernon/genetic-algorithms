# Phase 42: Warm Starting & Population Seeding - Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can initialize populations from known solutions (seeded individuals plus random fill, with genotypic dedup), provide pre-evaluated individuals whose fitness is trusted, or resume from a deserialized checkpoint — enabling hot-start and transfer learning workflows.

**In scope:**
- `with_seeds(Vec<U>)` builder method on Ga — user-provided chromosomes mixed with random fill
- Genotypic deduplication: random fill avoids duplicating seed DNA
- `with_checkpoint(path)` builder method on Ga — deserializes checkpoint and populates state
- Checkpoint resumption: generation counter starts from checkpoint's saved generation (absolute mode)
- Hybrid config override: operator config (selection, crossover, mutation) from builder wins; state fields (population, generation, stats) from checkpoint
- Seed fitness is trusted — pre-evaluated seeds skip re-evaluation
- Hall of Fame compatibility: trusted seeds considered for archive admission during initialization
- Ga engine only (same pattern as Phase 41 Hall of Fame)
- WASM compatibility (pure math — no Instant/rayon needed for seeding or checkpoint loading)

**Out of scope:**
- Checkpoint resumption for non-Ga engines (De, Scatter, Cellular, Alps, Nsga2Ga) — deferred
- New GaObserver hooks for warm-start events — standard hooks fire correctly with absolute generation numbers
- Auto-configuration of `max_generations` from checkpoint (user controls this in builder)
- Mid-run checkpoint loading (checkpoint is loaded at build time, before run())
</domain>

<decisions>
## Implementation Decisions

### Seeding API
- **D-01:** Seeding via `with_seeds(Vec<U>)` builder method on Ga. Seeds are user-provided `U: ChromosomeT` instances. Remaining population slots are filled via the existing `initialization_fn`.
- **D-02:** Random fill deduplicates against seed DNA using genotypic uniqueness check (same approach as Hall of Fame dedup).

### Checkpoint Resumption API
- **D-03:** Resumption via `with_checkpoint(path)` builder method. Loads `Checkpoint<U>` at build time, sets population, generation counter, and stats from the checkpoint.
- **D-04:** Hybrid config override: user's builder config wins for operator settings (selection, crossover, mutation). Checkpoint wins for state fields (population, generation vector, accumulated stats). User must still provide non-serializable parts (fitness_fn, initialization_fn) in the normal builder chain.

### Generation Counter on Resume
- **D-05:** Absolute mode — generation loop starts from `checkpoint.generation` and runs for `max_generations` additional generations (i.e., upper bound is `checkpoint.generation + max_generations`). Observer hooks receive the correct absolute generation number.
- **D-06:** Accumulated stats vector from checkpoint is preserved and appended to during resumed run. `stats.clear()` is NOT called when checkpoint is loaded.

### Fitness Trust for Seeds
- **D-07:** Seed fitness is trusted — pre-evaluated seeds skip re-evaluation during initialization. User is responsible for providing chromosomes with correct fitness values.
- **D-08:** Seeds are eligible for Hall of Fame admission during initialization (if HOF is configured).

### Scope Boundaries
- **D-09:** Ga engine only. Other engines deferred (consistent with Phase 41 Hall of Fame pattern).

### Claude's Discretion
- Internal validation: if seeds exceed population_size, error or clamp? (planner decides)
- Seed injection timing: seeds placed first, then random fill generated to reach population_size
- Genotypic dedup algorithm for fill-vs-seeds: same approach as Hall of Fame (DNA slice comparison)
- Checkpoint loading at `.build()` time (before validation) vs deferred to `.run()` time
- Hall of Fame seed admission: whether to admit all seeds before run starts, or only during generation loop
- `with_seeds()` and `with_checkpoint()` mutual exclusivity validation

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### GA Engine — Primary Integration Target
- `src/engines/ga.rs` — Ga struct, builder methods, `initialization()`, `run_with_callback()` initialization block (line 810)
- `src/engines/ga.rs` §556 — `with_population()` builder method pattern
- `src/engines/ga.rs` §686 — `with_initialization_fn()` builder method pattern
- `src/engines/ga.rs` §717 — `initialization()` method (parallel random init flow)
- `src/engines/ga.rs` §863 — `self.stats.clear()` — MUST NOT call when checkpoint loaded (D-06)

### Checkpoint Module
- `src/observe/checkpoint.rs` — `Checkpoint<U>` struct, `load_checkpoint()`, `save_checkpoint()`. Existing foundation.

### Existing Patterns to Follow
- `src/engines/ga.rs` §103 — Ga struct fields: `Option<...>` pattern for optional features (zero overhead when None)
- `src/engines/ga.rs` §556 — `with_population()` consuming builder pattern
- `src/engines/ga.rs` §699 — `with_hall_of_fame()` builder method (Phase 41 integration) — most recent analogous feature

### Hall of Fame (Phase 41) — Seed admission
- `src/hall_of_fame.rs` — `HallOfFame<U>` struct: `try_add()`, genotypic dedup, capacity management
- `.planning/phases/41-hall-of-fame-solution-archive/41-CONTEXT.md` — HOF decisions (D-07: same-DNA dedup)

### Requirements and Roadmap
- `.planning/ROADMAP.md` §Phase 42 — Goal: "Users can initialize populations from known solutions, seeded individuals plus random fill, or deserialized checkpoints — enabling hot-start and transfer learning workflows"
- Issue #216 — Warm Starting & Population Seeding

### WASM Compatibility
- `CLAUDE.md` §WASM Compatibility — Must apply cfg-gating rules (seeding and checkpoint loading are pure data operations, no Instant/rayon needed)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`Checkpoint<U>` struct** (`src/observe/checkpoint.rs`) — Already has `population`, `configuration`, `generation`, `stats` fields + `load_checkpoint()` helper
- **`HallOfFame::try_add()`** (`src/hall_of_fame.rs`) — Genotypic dedup logic reusable for seed dedup during fill
- **`ChromosomeT::dna()`** — For genotypic distance comparison between seeds and random fill
- **`Population<U>`** — Existing type used by `with_population()`, seeds will target this type

### Established Patterns
- Ga stores optional features as `Option<...>` with builder methods — zero overhead when None
- Builder methods return `Self` for chaining
- `with_population()` is consuming (takes `Population<U>` by value) — seeds follow similar pattern
- Hall of Fame stores `Option<HallOfFame<U>>` — checkpoint field follows same pattern

### Integration Points
- `src/engines/ga.rs` — Add `seeds: Option<Vec<U>>` field on Ga struct
- `src/engines/ga.rs` — Add `with_seeds(Vec<U>)` builder method
- `src/engines/ga.rs` — Modify `initialization()` to handle seeds + random fill with dedup
- `src/engines/ga.rs` — Add `checkpoint_path: Option<PathBuf>` field on Ga struct
- `src/engines/ga.rs` — Add `with_checkpoint(path)` builder method
- `src/engines/ga.rs` — Modify `run_with_callback()` init block (line 810) to handle checkpoint loading and absolute generation counting
- `src/engines/ga.rs` — Modify generation loop bounds to support absolute generation counter (D-05)
- `src/engines/ga.rs` — Preserve stats vector when checkpoint loaded (D-06)

</code_context>

<specifics>
No specific references beyond standard warm-starting patterns. Open to standard approaches.

Key behaviors derived from discussion:
- Seeds placed before random fill in population vector (seeds = first N, fill = rest)
- Genotypic dedup: random DNA that matches any seed DNA is discarded and regenerated
- Checkpoint loaded at build-time validation (consistent with `build()` being the validation boundary)
- Seeds have `.fitness()` trusted — no re-evaluation. Random fill evaluated as normal.

</specifics>

<deferred>
None — discussion stayed within phase scope.

</deferred>

---

*Phase: 42-Warm Starting & Population Seeding*
*Context gathered: 2026-05-12*
