# Phase 78: Replace User-Input Panics with GaError (Issue #279) - Context

**Gathered:** 2026-06-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Audit all `panic!` / `.unwrap()` / `.expect()` calls in `src/` that are reachable through user input or configuration, and replace them with recoverable `GaError` returns. All relevant `GaError` variants already exist. Internal invariant panics (proven unreachable by surrounding logic) are explicitly out of scope.

**In scope:**
- `engines/eda/engine.rs` — empty init population panics (lines 311, 667) → `GaError::InitializationError`
- `engines/pso/engine.rs` — empty init population panic (line 344) → `GaError::InitializationError`
- `engines/cma/engine.rs` — empty init population panics (lines 614, 653, 663) → `GaError::InitializationError`
- `engines/cellular/engine.rs` — grid rows=0/cols=0 panic (line 136) → move validation to `new()` → `GaError::ConfigurationError`
- `engines/alps/engine.rs` — layer_size=0 / n_layers=0 panics (lines 127, 130) → move validation to `new()` → `GaError::ConfigurationError`
- `operations/crossover/order.rs` — non-unique gene IDs panic (line 111) → `GaError::CrossoverError`
- `operations/selection.rs` — Lexicase/EpsilonLexicase wrong-factory panic → `GaError::SelectionError`
- `engines/ga/generation.rs` — AOS mutex lock().unwrap() calls (8 occurrences) → `GaError::InternalError`
- All fitness cache `lock().expect("fitness cache lock poisoned")` calls (ga/cache.rs, ga/batch.rs, eda/engine.rs, pso/engine.rs, cma/engine.rs) → `GaError::InternalError`
- Add new `GaError::InternalError(String)` variant to `error.rs`
- Tests for each former panic feeding bad input and asserting the correct `GaError` variant

**Out of scope:**
- `engines/gp/chromosome.rs` `dna()`/`dna_mut()`/`set_dna()` panics — intentional misuse-panics; trait signature returns `&[Gene]` not `Result`; cannot be converted
- `operations/crossover.rs` downcast expects — guarded by `downcast_ref` immediately above; internal invariant
- `engines/gp/crossover.rs` index expects (`i1`/`i2` valid) — indices computed from known-valid node counts; internal invariant
- `engines/ga/mod.rs` `stats.last().unwrap()` — stats vec populated at gen 0 before this code runs; internal invariant
- `operations/crossover/multipoint.rs` array-index unwraps — bounds-checked iteration; internal invariant

</domain>

<decisions>
## Implementation Decisions

### GP chromosome trait panics
- **D-01:** `GpChromosome::dna()`/`dna_mut()`/`set_dna()` panics are **kept as-is**. The trait signature returns `&[Gene]` (not `Result`), so conversion is impossible without a trait-breaking change. These are intentional misuse-panics with existing `# Panics` doc comments. No changes needed here.
- **D-02:** Current panic messages ("not supported — use GpChromosome with GpGa, not Ga") are sufficient. No improvement required.

### Mutex poison handling
- **D-03:** AOS mutex lock failures **propagate as `GaError::InternalError`**. The generation loop return type changes to `Result<(), GaError>`; `run()` surfaces it to the caller.
- **D-04:** Add a new **`GaError::InternalError(String)`** variant to `src/error.rs`. Mutex poisoning is never user-caused — a dedicated variant keeps it clearly distinct from config/mutation errors and makes it grep-able.
- **D-05:** All fitness cache `lock().expect(...)` calls (across `ga/cache.rs`, `ga/batch.rs`, `eda/engine.rs`, `pso/engine.rs`, `cma/engine.rs`) are also converted to return `Err(GaError::InternalError(...))`. Consistent treatment across all mutex failures.

### Cellular/ALPS validation placement
- **D-06:** Validation moves to **`new()` returning `Result<Self, GaError>`** for both `CellularEngine` and `AlpsEngine`. v3.0.0 is a semver-breaking release; `Result`-returning constructors are acceptable here.
- **D-07:** Use `GaError::ConfigurationError` for grid/layer constraint violations (rows=0, cols=0, layer_size=0, n_layers=0). Fail-fast at construction time, before any run starts.

### Audit scope boundary
- **D-08:** **Internal invariant panics are excluded** from conversion. An invariant qualifies for exclusion when: (a) there is a proof in the surrounding code that the panic is unreachable from valid user input, AND (b) the `.expect()` or `.unwrap()` message clearly states why. Downstream agents should document the reasoning in a comment when leaving such a call in place.
- **D-09:** The Lexicase/EpsilonLexicase wrong-factory panic **is converted** to `GaError::SelectionError` — the user triggered it by configuring Lexicase and calling the wrong factory function; that's user-input-reachable.

### Error variant additions
- **D-10:** Only **one new variant** added to `GaError`: `InternalError(String)`. No other new variants are needed — all other panic replacements map to existing variants.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Error types
- `src/error.rs` — `GaError` enum; add `InternalError(String)` variant here; all existing variants mapped in `Display` impl

### Target engines (panics to fix)
- `src/engines/eda/engine.rs` — lines 311, 667 (empty pop panics); lines 335, 404, 692, 759 (cache lock expects)
- `src/engines/pso/engine.rs` — line 344 (empty pop panic); lines 369, 482 (cache lock expects)
- `src/engines/cma/engine.rs` — lines 614, 653, 663 (empty pop panics); lines 429, 451 (cache lock expects)
- `src/engines/cellular/engine.rs` — line 104: `new()` signature to change; line 136: validation to move to `new()`
- `src/engines/alps/engine.rs` — line 107: `new()` signature to change; lines 127, 130: validation to move to `new()`

### GA engine mutation handling (reference for generation loop Result type)
- `src/engines/ga/generation.rs` — AOS mutex calls at lines 212, 221, 349, 361, 381, 384, 391, 394; generation loop return type must change
- `src/engines/ga/cache.rs` — line 15, 35: cache lock expects to convert
- `src/engines/ga/batch.rs` — lines 52, 81: cache lock expects to convert

### Operator panics
- `src/operations/crossover/order.rs` — line 111: panic on non-unique gene IDs → `GaError::CrossoverError`
- `src/operations/selection.rs` — line 79: Lexicase wrong-factory panic → `GaError::SelectionError`

### GP chromosome (out of scope — kept as panics)
- `src/engines/gp/chromosome.rs` — lines 274, 284, 294: intentional misuse-panics, not converted

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `GaError` enum in `src/error.rs` — all required variants already exist except `InternalError`; add one variant, update `Display` match arm
- `GaResult<T>` alias already used throughout — `?` operator works everywhere engines return `Result`

### Established Patterns
- EDA/PSO/CMA run loops already return `Result<GaResult<U>, GaError>` — the plumbing for propagating `GaError` from within the run loop is already in place
- `CellularEngine::new()` and `AlpsEngine::new()` currently return `Self` — both need signature change to `Result<Self, GaError>`; callers in examples/tests will need `.unwrap()` or `?`
- The AOS state mutex is acquired via `aos_state.lock().unwrap()` — change to `aos_state.lock().map_err(|_| GaError::InternalError("AOS state mutex poisoned".to_string()))?`
- Fitness cache expects follow pattern: `ch.lock().expect("fitness cache lock poisoned")` — change to `ch.lock().map_err(|_| GaError::InternalError("fitness cache mutex poisoned".to_string()))?`

### Integration Points
- Each engine's run loop return type — already `Result<..., GaError>` so `?` propagation works
- `CellularEngine::new()` and `AlpsEngine::new()` — callers in `examples/`, `tests/` and internal code must handle `Result`
- `src/operations/crossover/order.rs` — `fn order_crossover<U>` return type must propagate `GaError::CrossoverError`
- `src/operations/selection.rs` — `SelectionOperator::select()` return type; check if it already returns `Result` or if this is a new `Err` path

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches following the established `GaError` pattern.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 78-replace-user-input-panics-with-gaerror-issue-279*
*Context gathered: 2026-06-19*
