# Phase 41: Hall of Fame / Solution Archive - Context

**Gathered:** 2026-05-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can maintain an archive of top-N unique solutions across the entire GA run, with optional minimum-distance diversity filtering (fitness-space or genotypic), accessible after run completion via a public method on the Ga struct.

**In scope:**
- HallOfFame struct with bounded capacity, deduplication, and min-distance diversity filtering
- Two distance modes: Fitness-space (Euclidean, default) and Genotypic (DNA-level, configurable)
- Fixed distance threshold for diversity filtering (not relative/adaptive)
- Archive updated every generation: all offspring evaluated, top-N by fitness admitted
- Eviction policy: remove worst fitness when full
- Access via `.hall_of_fame()` on Ga<U> after run() completes
- Core API: `.solutions()`, `.top(k)`, `.would_qualify()`, `.len()`
- Extended API: serde support (behind feature flag), iterator with metadata (generation_added, fitness)
- Supplements existing best_chromosome tracking (non-breaking)
- Builder method on Ga only (`.with_hall_of_fame(...)`), not on ConfigurationT
- WASM compatibility (pure math, no time/thread requirements)

**Out of scope:**
- Hall of Fame integration in non-Ga engines (De, Scatter, Cellular, Alps, Nsga2Ga) — deferred to future phase
- GaObserver hooks for archive events — no new hooks this phase
- Mid-run archive access (post-run only)
</domain>

<decisions>
## Implementation Decisions

### Diversity Filtering
- **D-01:** Both Fitness-space and Genotypic distance metrics, configurable via an enum (e.g., `DistanceMetric::Fitness { min_distance: f64 }` and `DistanceMetric::Genotypic { min_distance: f64 }`)
- **D-02:** Default metric is Fitness-space (Euclidean distance in objective space)
- **D-03:** Distance threshold is a fixed f64 value, not relative/percentage-based
- **D-04:** When archive is full and a new solution qualifies, evict the solution with worst fitness

### Archive Update Strategy
- **D-05:** Archive is checked every generation — all offspring are evaluated for entry
- **D-06:** Entry criterion: top-N by fitness. A solution is admitted only if its fitness is >= the current worst in the archive (or archive not yet full)
- **D-07:** Deduplication: same-DNA entries are not added (genotypic uniqueness check)
- **D-08:** Post-run only — no observer hooks for archive events

### Access Pattern
- **D-09:** Accessed via `.hall_of_fame()` public method on `Ga<U>` after `run()` completes
- **D-10:** Core API: `solutions() -> &[U]`, `top(k: usize) -> &[U]`, `would_qualify(chromosome: &U) -> bool`, `len() -> usize`
- **D-11:** Extended: `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` for the HallOfFame struct, iterator yielding `(chromosome, generation_added, fitness_at_addition)`
- **D-12:** Hall of Fame supplements (not replaces) existing best_chromosome tracking

### Multi-Engine Support
- **D-13:** Ga only for this phase. Hall of Fame builder method goes on the Ga struct directly, not on ConfigurationT trait.
- **D-14:** Other engines (De, Scatter, Cellular, Alps, Nsga2Ga) deferred

### Claude's Discretion
- HallOfFame internal data structure: ordered Vec sorted by fitness is preferred (simple, O(n) insert)
- Ga stores `hall_of_fame: Option<HallOfFame<U>>` (zero overhead when None, consistent with GaObserver pattern)
- Archive capacity: usize parameter, no default (user must specify if they want archiving)
- Generation tracking: store u64 generation number when each solution was added, for iterator metadata
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### GA Engine Patterns
- `src/engines/ga.rs` — Ga struct, builder methods, run loop — Hall of Fame integration targets
- `src/configuration.rs` — GaConfiguration: may need hall_of_fame field if moving to shared config (per D-13, builder stays on Ga for now)
- `src/traits/configuration.rs` — ConfigurationT trait — NOT adding hall_of_fame here per D-13

### Existing Patterns to Follow
- `src/engines/ga.rs` — Observer wiring: `Option<Arc<dyn GaObserver<U>>>` with builder method `with_observer()` — Hall of Fame follows same Option pattern
- `src/traits/operators.rs` — Existing operator traits for reference patterns

### Requirements and Roadmap
- `.planning/ROADMAP.md` §Phase 41 — Goal: "Users can maintain an archive of top-N unique solutions across the entire run, with optional minimum-distance diversity filtering, accessible after run completion"
- Issue #217 — Hall of Fame / Solution Archive

### WASM Compatibility
- `CLAUDE.md` §WASM Compatibility — Must apply cfg-gating rules (archive management is pure math, no Instant/rayon needed)
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ChromosomeT::dna()` — For genotypic distance comparison (D-01), agents can compare DNA slices directly
- `GenerationStats` — For generation number tracking in archive metadata
- `GaObserver` pattern — `Option<Arc<dyn ...>>` with zero overhead when None — same pattern for HallOfFame field

### Established Patterns
- Ga stores optional features as `Option<...>` with builder methods — Hall of Fame follows same pattern
- Builder methods return `Self` for chaining
- `#[cfg_attr(feature = "serde", derive(...))]` for config structs
- Generation counter already tracked in Ga (self.generation or similar)

### Integration Points
- `src/engines/ga.rs` — Add `hall_of_fame: Option<HallOfFame<U>>` field on Ga struct
- `src/engines/ga.rs` — Add `with_hall_of_fame(config: HallOfFameConfig)` builder method
- `src/engines/ga.rs` — Insert archive update call in run loop after fitness evaluation (per D-05)
- `src/engines/ga.rs` — Add `pub fn hall_of_fame(&self) -> Option<&HallOfFame<U>>` accessor method
</code_context>

<specifics>
## Specific Ideas

- HallOfFame internal: ordered Vec (sorted by fitness descending), binary search insertion
- Genotypic distance: iterate DNA slices, count differing positions / sum absolute differences
- Fitness-space distance: Euclidean distance between fitness values (or objective vector for multi-objective, though multi-objective is deferred)
- Generation metadata: store u64 alongside each entry
- would_qualify(): check against current worst fitness AND min-distance to existing entries
</specifics>

<deferred>
## Deferred Ideas

- Nsga2Ga Hall of Fame integration — separate future phase
- De, Scatter, Cellular, Alps Hall of Fame integration — separate future phase
- Mid-run archive access via GaObserver hooks — no immediate demand, easy additive change
- Relative/adaptive distance thresholds — not requested, easy to add later
- Multi-objective Pareto-front archiving within HallOfFame — handled by Nsga2Ga's own Pareto front, separate concern
</deferred>

---

*Phase: 41-Hall of Fame / Solution Archive*
*Context gathered: 2026-05-11*
