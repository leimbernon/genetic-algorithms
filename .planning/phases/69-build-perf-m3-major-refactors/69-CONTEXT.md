# Phase 69: Build-perf M3 — major refactors - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Land the three highest-risk, highest-payoff build-perf refactors under zero regression: (1) port all 12 bench files from `criterion` to `divan` then remove `criterion` from `[dev-dependencies]`; (2) gate every `rayon` call-site across the full codebase (~27 files) behind a new `parallel` feature (default-on); (3) split `src/engines/ga.rs` (3342 lines) into 10 cohesive submodules under `src/engines/ga/`. Five sequential plans (Waves 0 → 1 → 2) as defined in ROADMAP.md.

**What this phase delivers:**
- `criterion` removed from `[dev-dependencies]`; all 12 bench files use `divan` API
- New `parallel` feature (default-on) gating `dep:rayon`; every `par_iter()`/`into_par_iter()`/`par_iter_mut()`/`par_chunks*()` site in all engines uses the combined gate `#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]`
- `src/engines/ga.rs` → `src/engines/ga/{mod,lifecycle,generation,adaptive,aos,extension,cache,batch,stats,observer,stopping}.rs`
- A grep-based CI step that fails if any unconditional `rayon::` reference remains in `src/`
- Feature-matrix CI green on every combination including `parallel=off`
- CC-3 golden tests byte-identical with `parallel=on` AND `parallel=off`
- Documentation sweep: `CHANGELOG.md`, `README.md`, `src/lib.rs`, `docs/benchmarks.md`, `docs/ARCHITECTURE.md`, `CLAUDE.md`, and `.planning/intel/bench-harness.md`, `.planning/intel/parallel-feature.md`, `.planning/intel/ga-internals.md`

</domain>

<decisions>
## Implementation Decisions

### rayon gating scope (plan 69-03)
- **D-01:** Gate ALL rayon call-sites across the full codebase — not just `ga.rs` + `population.rs`. This includes all alt-engines (`nsga2`, `nsga3`, `moead`, `spea2`, `ibea`, `gp`, `island`, `cellular`, `alps`, `de`, `scatter`, `eda`). Every `par_iter()`/`into_par_iter()`/`par_iter_mut()`/`par_chunks*()` site in all 27 files gets the combined gate. Matches BUILD-PERF.md §Action #3 "every... site" language.
- **D-02:** Non-rayon wasm32 gates (e.g., `#[cfg(not(target_arch = "wasm32"))]` for `Instant::now()` or WASM-incompatible stdlib calls) are NOT touched — only sites that call into `rayon::` get updated.
- **D-03:** Do NOT add a separate regex CI step or clippy custom lint. Rely on feature-matrix CI (CC-2) compiling with `--no-default-features --features logging` to catch any unconditional rayon reference at compile time. This is faster CI overall. HOWEVER — user requested a grep CI step for enforcement (see D-04).
- **D-04:** Add a grep-based CI step (fast, <1s) that checks `src/` for any `rayon::` usage without the cfg gate. Runs as part of the `parallel` feature plan (69-03). Complements the feature-matrix compile check by making the invariant explicitly visible in CI output.

### `parallel` feature definition
- **D-05:** Feature name is `parallel` (semantic, not `rayon` — lets implementation be swapped later). Feature declaration in `Cargo.toml`: `parallel = ["dep:rayon"]`; default includes `parallel` and `logging`. Matches BUILD-PERF.md §Action #3 and the open-questions resolution at the bottom of that spec.
- **D-06:** The canonical gate pattern for every rayon site is exactly: `#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]` for the sequential fallback. The parallel arm uses `#[cfg(not(any(target_arch = "wasm32", not(feature = "parallel"))))]` or equivalently `#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]`. CLAUDE.md "WASM Compatibility" section must be updated to document this new canonical gate.

### divan port (plan 69-01)
- **D-07:** Allow light cleanup during migration — remove dead/duplicate bench cases and simplify overly complex criterion setup code where divan makes it easier. All ported cases must stay within ±3% median tolerance. Port is still one-bench-file-per-commit as ROADMAP specifies.
- **D-08:** `metrics_observer` bench (requires `--features observer-metrics`) stays as a **separate CI step** to keep feature isolation clean, mirroring how the `de` bench uses `--features benchmarks`. Same pattern applied consistently.
- **D-09:** Both `criterion` and `divan` coexist in `Cargo.toml` during porting; `criterion` is removed in the same plan (69-01) once every bench file is ported and CI is green.

### ga.rs submodule split (plan 69-04)
- **D-10:** Submodule-by-submodule commits — one commit per extracted submodule (approximately 11 commits). Easier to review and bisect if something breaks in a 3342-line refactor. BUILD-PERF.md says "atomic revert" — that's satisfied by `git revert <range>` covering all 11 commits if needed.
- **D-11:** Visibility strategy — minimize `pub` surface: use `pub(super)` where two sibling submodules share an item; escalate to `pub(crate)` only when the item is accessed from outside `engines/ga/`. Public API items (currently `pub`) remain `pub` unchanged — BUILD-PERF.md guarantees zero API surface change. No items gain visibility relative to what they have today.
- **D-12:** `cargo expand` symbol diff is the primary semantic-equivalence check: run before and after the full split and confirm zero diff in the exported symbol table. Used in plan 69-04 before merge.

### Clippy / grep enforcement
- **D-13:** Add a grep-based CI enforcement step in plan 69-03: `grep -rn 'rayon::' src/ | grep -v '#\[cfg'` (or equivalent) that fails if any match found. Fast (<1s), no clippy custom lint setup, no additional CI compilation cost. Documents the invariant enforceably.

### Claude's Discretion
- Exact grep regex for the enforcement step — use whatever correctly catches bare `rayon::` imports and call-sites while ignoring cfg-gated lines. Planner has full flexibility here.
- Ordering of the 11 submodule extraction commits (within plan 69-04) — extract in dependency order (low-level helpers like `cache`, `stats`, `observer` first; then `generation`, `lifecycle`, `adaptive`; finally `mod.rs` orchestrator). Claude's call.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary specification
- `.planning/v3.0.0-BUILD-PERF.md` §Wave M3 — Exact change descriptions, verification criteria, doc deliverables, and revert plans for all three actions (#7, #3, #4). MUST READ before writing any plan.
- `.planning/v3.0.0-BUILD-PERF.md` §Non-negotiable guarantees — All 6 guarantees apply to every plan in this phase.
- `.planning/v3.0.0-BUILD-PERF.md` §Acceptance gate per action — 9-gate merge checklist. Every plan PR must satisfy all 9.
- `.planning/ROADMAP.md` §Phase 69 — Success criteria (6 items), plan list (5 plans in 3 waves), dependency on Phase 68.

### Prior build-perf phases (carry-forward constraints)
- `.planning/phases/68-build-perf-m2-dependency-hygiene/68-CONTEXT.md` — `logging` feature gate pattern (`#[cfg(feature = "logging")]`), `dep:log` in Cargo.toml, log macro family in `src/lib.rs`. The `parallel` feature in Phase 69 follows the same `dep:` pattern.
- `.planning/phases/67-build-perf-m1-config-only-quick-wins/67-CONTEXT.md` — sccache, nextest, mold patterns already in CI; Phase 69 CI changes must not conflict with these.

### Baseline / regression infrastructure (Phase 66, must exist before execution)
- `bench/build_perf.sh` — CC-1 measurement script. Run before and after each plan.
- `.planning/baselines/v3.0.0-baseline.json` — Canonical baseline numbers to diff against.
- `tests/golden/` — CC-3 golden tests. Must be byte-identical with `parallel=on` AND `parallel=off` after plan 69-03.
- `.github/workflows/feature-matrix.yml` — CC-2 feature-matrix CI. Must be green on all combinations including `parallel=off`.

### Files modified by plan 69-01 (criterion → divan)
- `benches/*.rs` — all 12 bench files
- `Cargo.toml` — remove `criterion`, add `divan`
- `docs/benchmarks.md` — updated invocation snippets
- `.planning/intel/bench-harness.md` — AI-readable rationale

### Files modified by plan 69-02/03 (parallel feature)
- `Cargo.toml` — add `parallel = ["dep:rayon"]` to `[features]`; `rayon` becomes `optional = true`; `default` updated
- `src/**` — all 27 files with rayon call-sites; existing `#[cfg(not(target_arch = "wasm32"))]` rayon gates replaced with combined gate
- `CLAUDE.md` — "WASM Compatibility" rule updated to document new canonical gate pattern
- `README.md` + `src/lib.rs` — Features table: add `parallel` row
- `CHANGELOG.md` — Added entry for `parallel` feature
- `.planning/intel/parallel-feature.md` — AI-readable rationale + canonical gate pattern
- `.github/workflows/feature-matrix.yml` — Add `parallel=off` combination

### Files modified by plan 69-04/05 (ga.rs split)
- `src/engines/ga.rs` → `src/engines/ga/{mod,lifecycle,generation,adaptive,aos,extension,cache,batch,stats,observer,stopping}.rs`
- `src/engines/mod.rs` or equivalent — update module declaration from `mod ga` (file) to `mod ga` (directory)
- `docs/ARCHITECTURE.md` — updated module map reflecting the split
- `CHANGELOG.md` — Changed (internal) entry
- `.planning/intel/ga-internals.md` — AI-readable submodule responsibilities

### WASM compatibility
- `.cargo/config.toml` — existing `[target.wasm32-unknown-unknown]` rustflags block; do not modify
- `.github/workflows/wasm-check.yml` — must remain green after every plan

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engines/ga.rs` (3342 lines) — the file to be split in plan 69-04. Existing section comments in the file provide natural submodule boundaries; use them as the extraction guide.
- `benches/` — 12 criterion bench files. `ga_run.rs` and `island_ga.rs` have the most complex setups (population + config construction). `metrics_observer.rs` requires `--features observer-metrics`.
- `.planning/v3.0.0-BUILD-PERF.md` §Action #4 proposed split — 11 submodule names and their responsibilities are defined verbatim. Use as the canonical layout.

### Established Patterns
- WASM cfg gate: `#[cfg(not(target_arch = "wasm32"))]` for sequential fallback — being extended to `#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]` in this phase. Non-rayon wasm32 gates are unchanged.
- `dep:` prefix in Cargo.toml features (`logging = ["dep:log"]` from Phase 68) — same pattern for `parallel = ["dep:rayon"]`.
- `pub(crate)` for intra-crate sharing, `pub(super)` for sibling submodule sharing — both used in existing `src/engines/` submodules.
- Feature-gated CI (e.g., `--features benchmarks` for `de` bench) — same pattern for `--features observer-metrics` on `metrics_observer` bench.

### Integration Points
- `Cargo.toml` `[features]` section — already has `logging` (Phase 68). Add `parallel` beside it; update `default = ["logging", "parallel"]`.
- `src/lib.rs` Feature Flags table — already documents `logging` (Phase 68). Add `parallel` row in the same format.
- `src/engines/ga.rs` re-export surface — after split, `src/engines/mod.rs` must still export `Ga` at the same path (`crate::engines::Ga`). Zero public API surface change.
- Rayon sites in 27 files — the 5 in `ga.rs` (lines ~1277, ~1941, ~2246, ~3173, and within `evaluate_initial_fitness`) are gated during plans 69-02/03; the remaining sites in alt-engines are gated in the same plans (full-crate scope, D-01).

### Build Verification Checklist (per plan)
- `cargo check --target wasm32-unknown-unknown` — must pass after every plan
- `cargo test --all-features` and `cargo test --no-default-features --features logging` — both must pass
- CC-3 golden tests — byte-identical
- `cargo doc --no-deps` — zero warnings
- `cargo public-api` — zero unintended changes

</code_context>

<specifics>
## Specific Ideas

- The grep enforcement check (D-13) should be positioned in CI after the feature-matrix compile check so it's only reached if compilation succeeds — fail fast on compile errors first.
- The 11 submodule extraction commits (plan 69-04) should extract in dependency order: low-level helpers first (`cache`, `stats`, `observer`, `stopping`), then algorithm steps (`generation`, `lifecycle`, `adaptive`, `aos`, `extension`, `batch`), and finally `mod.rs` as the orchestrator. This order ensures each intermediate state compiles cleanly.
- Commit bodies for all plans MUST include a `Revert plan:` line per BUILD-PERF.md non-negotiable guarantee #5.
- `.planning/intel/parallel-feature.md` must include: why `parallel` not `rayon` as the feature name, the canonical gate pattern verbatim, what an agent must NOT reintroduce (unconditional rayon:: imports), and how to verify the invariant (CI step + compile check).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 69-build-perf-m3-major-refactors*
*Context gathered: 2026-06-15*
