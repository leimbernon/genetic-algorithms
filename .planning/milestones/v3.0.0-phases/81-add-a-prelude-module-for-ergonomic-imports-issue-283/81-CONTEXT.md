# Phase 81: Add a Prelude Module for Ergonomic Imports (Issue #283) - Context

**Gathered:** 2026-06-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Create `src/prelude.rs` and declare `pub mod prelude;` in `src/lib.rs` so users can write `use genetic_algorithms::prelude::*;` and have all high-frequency items available without 8–11 separate import lines. This is purely additive — no existing public API changes.

</domain>

<decisions>
## Implementation Decisions

### Prelude Breadth — Engines
- **D-01:** Include ALL engine entry points: `Ga`, `IslandGa`, all multi-objective engines (`Nsga2Engine`, `Nsga3Engine`, `MoeaDEngine`, `Spea2Engine`, `SmsEmoaEngine`, `IbeaEngine`), all alt-metaheuristic engines (`DeEngine`, `ScatterEngine`, `CellularEngine`, `AlpsEngine`, `CmaEngine`, `PsoEngine`, `EdaEngine`), plus `HillClimbEngine`, `PermutateEngine`, `GpGa`.
- **D-02:** Include ALL engine-specific config structs (`CmaConfiguration`, `PsoConfiguration`, `EdaConfiguration`, `AlpsConfiguration`, `HillClimbConfiguration`, `PermutateConfiguration`, etc.) — if an engine is in the prelude, its config struct is too.

### Prelude Breadth — Core Types
- **D-03:** Include from ROADMAP.md: `ConfigurationT` + per-area config traits, operator enums (`Selection`, `Crossover`, `Mutation`, `Survivor`), `ProblemSolving`, `ChromosomeLength`, `GaError`, core traits (`ChromosomeT`, `GeneT`, `LinearChromosome`).
- **D-04:** Observers: include `GaObserver` trait + `NoopObserver` only. Concrete observers (`LogObserver`, `CompositeObserver`, engine-specific observer traits like `CmaObserver`, `PsoObserver`) are excluded — they're optional/advanced and users importing them already know what they need.

### Feature-Gated Items
- **D-05:** Re-export feature-gated types behind the same `#[cfg]` gates in `prelude.rs`:
  - `#[cfg(feature = "logging")] pub use ...::LogObserver;`
  - `#[cfg(feature = "observer-metrics")] pub use ...::MetricsObserver;`
  - `#[cfg(feature = "observer-tracing")] pub use ...::TracingObserver;`
  The prelude mirrors `lib.rs` behavior — if you have the feature, you get the type in the glob.

### Example Showcase
- **D-06:** Update `examples/rastrigin.rs` to use `use genetic_algorithms::prelude::*;`. It currently has 11 import lines and is the canonical simple-GA example — the diff makes the ergonomic benefit obvious. Concrete chromosome/genotype types (`RangeChromosome`, `RangeGenotype`) remain explicit imports (they are not in the prelude by design).

### Documentation
- **D-07:** Document in both `README.md` (Quick Start / Ergonomic Imports section) and the getting-started guide (docs/getting-started.md or equivalent). Rustdoc on the `prelude` module itself is also required.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope
- `.planning/ROADMAP.md` §Phase 81 — success criteria and explicit list of required prelude contents

### Existing Public API Surface
- `src/lib.rs` lines 395–437 — current `pub use` re-exports; prelude content must be a superset of the roadmap-listed items and must not introduce name collisions with these
- `src/lib.rs` lines 320–395 — current `pub mod` declarations; all engine modules already public

### Examples
- `examples/rastrigin.rs` — target file for prelude showcase update (D-06)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/lib.rs` existing `pub use` block (lines 397–437): prelude can re-use the same re-export paths; no new module resolution needed
- All engine entry types (`Ga`, `DeEngine`, `CmaEngine`, etc.) already publicly re-exported from `src/lib.rs` — prelude just needs `pub use crate::<type>;` or `pub use super::<type>;`

### Established Patterns
- Feature-gated re-exports already used in `lib.rs` (`#[cfg(feature = "logging")]`, `#[cfg(feature = "observer-metrics")]`, `#[cfg(feature = "observer-tracing")]`) — same pattern applies in `prelude.rs`
- No `prelude.rs` exists yet — this is a new file
- `pub mod prelude;` must be added to `src/lib.rs` alongside existing `pub mod` declarations

### Integration Points
- `src/lib.rs`: add `pub mod prelude;` declaration
- `src/prelude.rs`: new file, re-exports only (no new types defined)
- `examples/rastrigin.rs`: replace 11 explicit imports with `use genetic_algorithms::prelude::*;` + minimal concrete type imports
- `README.md`: add prelude section
- `docs/getting-started.md` (or equivalent): add prelude guidance

</code_context>

<specifics>
## Specific Ideas

- The prelude should enable writing a minimal GA with only `use genetic_algorithms::prelude::*;` plus concrete chromosome/genotype types (e.g. `use genetic_algorithms::chromosomes::Range as RangeChromosome;`) — this is the success criterion from the roadmap.
- Collision check is mandatory: run `cargo check` with a fresh file using only the glob import to verify no name conflicts.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 81-add-a-prelude-module-for-ergonomic-imports-issue-283*
*Context gathered: 2026-06-22*
