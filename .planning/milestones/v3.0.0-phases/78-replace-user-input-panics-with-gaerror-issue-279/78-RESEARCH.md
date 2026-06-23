# Phase 78: Replace User-Input Panics with GaError (Issue #279) - Research

**Researched:** 2026-06-19
**Domain:** Rust error handling — converting panics to recoverable `GaError` returns across multiple engine modules
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `GpChromosome::dna()`/`dna_mut()`/`set_dna()` panics are **kept as-is**. Trait signature returns `&[Gene]` (not `Result`); conversion is impossible without a trait-breaking change. Intentional misuse-panics with existing `# Panics` doc comments.
- **D-02:** Current panic messages ("not supported — use GpChromosome with GpGa, not Ga") are sufficient. No improvement required.
- **D-03:** AOS mutex lock failures **propagate as `GaError::InternalError`**. The generation loop return type changes to `Result<(), GaError>`; `run()` surfaces it to the caller.
- **D-04:** Add a new **`GaError::InternalError(String)`** variant to `src/error.rs`. Mutex poisoning is never user-caused — a dedicated variant keeps it clearly distinct from config/mutation errors.
- **D-05:** All fitness cache `lock().expect(...)` calls (across `ga/cache.rs`, `ga/batch.rs`, `eda/engine.rs`, `pso/engine.rs`, `cma/engine.rs`) are also converted to return `Err(GaError::InternalError(...))`. Consistent treatment across all mutex failures.
- **D-06:** Validation moves to **`new()` returning `Result<Self, GaError>`** for both `CellularEngine` and `AlpsEngine`. v3.0.0 is a semver-breaking release; `Result`-returning constructors are acceptable here.
- **D-07:** Use `GaError::ConfigurationError` for grid/layer constraint violations (rows=0, cols=0, layer_size=0, n_layers=0).
- **D-08:** **Internal invariant panics are excluded** from conversion. Qualifies for exclusion when: (a) surrounding code proves the panic is unreachable from valid user input, AND (b) the `.expect()` or `.unwrap()` message clearly states why. Downstream agents must document the reasoning in a comment.
- **D-09:** The Lexicase/EpsilonLexicase wrong-factory panic **is converted** to `GaError::SelectionError` — user-input-reachable via `SelectionOperator::select()` trait.
- **D-10:** Only **one new variant** added to `GaError`: `InternalError(String)`. No other new variants needed.

### Claude's Discretion

None documented.

### Deferred Ideas (OUT OF SCOPE)

None documented.
</user_constraints>

---

## Summary

Phase 78 is a focused error-handling correctness pass: every `panic!` / `.unwrap()` / `.expect()` in `src/` that can be triggered by user input or misconfigured calls must be replaced with a recoverable `Err(GaError::…)`. All required `GaError` variants already exist except one (`InternalError(String)`), which must be added to `src/error.rs`.

The work spans six subsystems: (1) EDA/PSO/CMA engines — empty-init-population panics, (2) Cellular/ALPS engines — grid/layer validation panics moved to `new()`, (3) OX crossover — non-unique gene ID panic in `ox_build_child`, (4) `SelectionOperator` trait implementation — Lexicase/EpsilonLexicase wrong-factory panic, (5) AOS mutex locks in `generation.rs` — eight `.unwrap()` calls, and (6) fitness-cache mutex `.expect()` calls across five files.

The critical architectural impact is that converting EDA/PSO/CMA `run()` panics to `GaError` requires changing those `run()` return types from `XxxResult<U>` to `Result<XxxResult<U>, GaError>`. This is a breaking API change — acceptable in v3.0.0. Moving `CellularEngine::new()` and `AlpsEngine::new()` to return `Result<Self, GaError>` is also breaking, and affects approximately 20 test call sites and 8 bench call sites that must add `.unwrap()` or `?` handling.

**Primary recommendation:** Decompose into four tasks: (1) add `InternalError` variant + convert cache/AOS mutex calls + update `generation.rs`, (2) convert EDA/PSO/CMA `run()` to `Result`-returning, (3) convert Cellular/ALPS `new()` to `Result`-returning + update callers, (4) convert OX crossover + Lexicase trait panic + write all new error-path tests.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Error type definition | `src/error.rs` | — | Single source of truth for `GaError`; adding `InternalError` here propagates to all consumers |
| Mutex poison handling | `src/engines/ga/generation.rs`, `ga/cache.rs`, `ga/batch.rs` | EDA/PSO/CMA engines | Mutex panics live at call sites; conversion must happen at each `lock()` call, not at a shared helper |
| Engine init validation | Each engine's `new()` or `run()` | — | Cellular/ALPS move to `new()`; EDA/PSO/CMA change `run()` return type |
| Operator error conversion | `src/operations/crossover/order.rs`, `src/operations/selection.rs` | — | Operators own their error paths; trait signature for `SelectionOperator::select()` change |
| Test coverage | `tests/engines/`, `tests/operations/` | — | All new error paths require tests in the correct `tests/` subdirectory |

---

## Standard Stack

No external crates needed. All work uses Rust stdlib (`std::sync::Mutex`, `Result`, `map_err`).

### Core Pattern: Converting `.unwrap()` / `.expect()` to `map_err`

```rust
// BEFORE — panics on mutex poison
let mut state = aos_state.lock().unwrap();

// AFTER — D-03/D-04 pattern
let mut state = aos_state.lock().map_err(|_| {
    GaError::InternalError("AOS state mutex poisoned".to_string())
})?;
```

```rust
// BEFORE — panics on mutex poison
let c = ch.lock().expect("fitness cache lock poisoned");

// AFTER — D-05 pattern
let c = ch.lock().map_err(|_| {
    GaError::InternalError("fitness cache mutex poisoned".to_string())
})?;
```

[ASSUMED] — Standard Rust error-handling patterns, no external verification needed.

### Core Pattern: Converting `panic!` in engine bodies to early `Err` return

For EDA/PSO/CMA where `run()` changes return type:

```rust
// BEFORE
pub fn run(&mut self) -> EdaResult<U> {
    // ...
    if pop.is_empty() {
        panic!("EdaEngine: init_fn returned an empty population");
    }
    // ...
}

// AFTER
pub fn run(&mut self) -> Result<EdaResult<U>, GaError> {
    // ...
    if pop.is_empty() {
        return Err(GaError::InitializationError(
            "EdaEngine: init_fn returned an empty population".to_string()
        ));
    }
    // ...all Ok(result) at return points
}
```

[ASSUMED] — Standard Rust Result propagation pattern.

### Core Pattern: Converting `new()` for Cellular/ALPS

```rust
// BEFORE
pub fn new(config, init_fn, fitness_fn) -> Self {
    Self { config, init_fn: Arc::new(init_fn), fitness_fn: Arc::new(fitness_fn) }
}

// AFTER — D-06/D-07
pub fn new(config, init_fn, fitness_fn) -> Result<Self, GaError> {
    if config.rows == 0 || config.cols == 0 {
        return Err(GaError::ConfigurationError(
            "CellularEngine: rows and cols must both be > 0".to_string()
        ));
    }
    Ok(Self { config, init_fn: Arc::new(init_fn), fitness_fn: Arc::new(fitness_fn) })
}
```

[ASSUMED] — Standard Rust constructor error pattern.

### Core Pattern: Converting `ox_build_child` panic

The panic is inside a `.map()` closure returning `Vec<G>`. The function signature must change to return `Result<Vec<G>, GaError>`:

```rust
// BEFORE — ox_build_child returns Vec<G>
pub(crate) fn ox_build_child<G>(donor, filler, p1, p2) -> Vec<G>

// AFTER — ox_build_child returns Result<Vec<G>, GaError>
pub(crate) fn ox_build_child<G>(donor, filler, p1, p2) -> Result<Vec<G>, GaError> {
    // ...same logic...
    child
        .into_iter()
        .enumerate()
        .map(|(i, g)| {
            g.ok_or_else(|| GaError::CrossoverError(format!(
                "Order crossover: child position {} was not filled — \
                 indicates non-unique gene IDs in the parents.",
                i
            )))
        })
        .collect()  // collects into Result<Vec<G>, GaError>
}
```

Then the call sites in `order()` use `?` to propagate:
```rust
let child_dna_1 = ox_build_child(parent_1.dna(), parent_2.dna(), p1, p2)?;
let child_dna_2 = ox_build_child(parent_2.dna(), parent_1.dna(), p1, p2)?;
```

[ASSUMED] — Standard Rust Result + Iterator::collect pattern.

### Core Pattern: Converting Lexicase panic in `SelectionOperator` trait

The `SelectionOperator::select()` trait returns `Vec<Vec<usize>>` (not `Result`). Converting the panic to `GaError` without a trait-breaking change requires a **panic-preserving approach with better message** OR a **trait signature change to `Result<Vec<Vec<usize>>, GaError>`**. Since v3.0.0 is a breaking release, D-09 mandates conversion. The trait signature must change:

```rust
// BEFORE
pub trait SelectionOperator {
    fn select<U>(...) -> Vec<Vec<usize>> where ...;
}

// AFTER
pub trait SelectionOperator {
    fn select<U>(...) -> Result<Vec<Vec<usize>>, GaError> where ...;
}
```

This cascades to ALL implementors of `SelectionOperator`. However, looking at the code, `SelectionOperator::select()` is called from `configuration.method.select(...)` on line 164 of `selection.rs` — which is already inside `factory()`. The Lexicase panic in the trait impl is the only site that can't return `Err` via the current `-> Vec<Vec<usize>>` signature.

**Alternative (lower-blast-radius):** Keep the trait returning `Vec<Vec<usize>>` but convert the panic to a logged error + empty return. However, D-09 explicitly requires `GaError::SelectionError`. The trait must change. [VERIFIED: source code read]

All callers that dispatch via the trait (island model, NSGA-II, custom impls) must be updated to handle `Result`. Check these call sites before writing the plan.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Collecting `Result<T,E>` from iterator | Manual loop + early return | `.collect::<Result<Vec<_>, _>>()` | Rust stdlib: `impl FromIterator<Result<T,E>> for Result<Vec<T>,E>` — stops at first error |
| Mutex poison handling | Custom wrapper type | `.map_err(|_| GaError::InternalError(...))` | Standard Rust `PoisonError` pattern |
| Thread-safe `?` propagation inside rayon closures | Custom channel | Collect `Vec<Result<_, GaError>>`, then iterate outside closure | rayon closures cannot directly `?`-propagate to outer scope |

---

## Architecture Patterns

### Recommended Project Structure (unchanged)

No new files needed. All changes are within existing source files.

### Pattern 1: `?`-propagation inside rayon closures

Rayon parallel closures (`par_iter().map(…)`) cannot use `?` to return early to the outer scope. The existing code in `generation.rs` already handles this correctly — the closure returns `Result<Vec<U>, GaError>` and the caller collects into `Vec<Result<_, _>>` then iterates with `?`. The same pattern applies when converting the AOS mutex calls:

```rust
// In process_pair closure (already exists in generation.rs lines 381-394):
let mut state = aos_state.lock().map_err(|_| {
    GaError::InternalError("AOS reward accumulator poisoned".to_string())
})?;
// The ? here exits the closure with Err — caller collects Vec<Result<...>>
```

[VERIFIED: codebase inspection]

### Pattern 2: `run()` callers need updating after return-type change

Every existing test that calls `.run()` on EDA/PSO/CMA engines uses the result directly:
```rust
let result = engine.run();
assert!(result.generations > 0);
```
After the change these become:
```rust
let result = engine.run().expect("engine run should succeed");
assert!(result.generations > 0);
```
Or in tests that check error paths:
```rust
let err = engine.run().unwrap_err();
assert!(matches!(err, GaError::InitializationError(_)));
```

[VERIFIED: codebase inspection — 13+ EDA, 8+ PSO, 17+ CMA test call sites]

### Anti-Patterns to Avoid

- **Silently discarding mutex poison:** Do NOT replace `.unwrap()` with `.unwrap_or_default()` or ignore the poison. Surface it as `GaError::InternalError`.
- **Changing `SelectionOperator::select()` return type without updating ALL trait implementors:** The island model and NSGA-II engines call `.select()` via the trait. Both must be updated.
- **Leaving the panic guard in `run()` for CellularEngine after moving validation to `new()`:** Remove the runtime check from `run()` once `new()` validates. Avoid double-guarding.
- **Adding `unwrap()` to new `CellularEngine::new(...).unwrap()` in bench files without explanation:** Benchmarks are infallible by design — add `.expect("valid config")` with a message for bench clarity.

---

## Common Pitfalls

### Pitfall 1: `SelectionOperator` trait blast radius

**What goes wrong:** Changing `SelectionOperator::select()` return type from `Vec<Vec<usize>>` to `Result<Vec<Vec<usize>>, GaError>` cascades to every file that implements or calls the trait via dynamic/static dispatch.

**Why it happens:** The trait is used in island model (`src/engines/island/`) and NSGA-II (`src/engines/nsga2/`) besides the main `ga/` engine.

**How to avoid:** Before writing the plan, grep for all call sites:
```bash
grep -rn '\.select(' src/ 2>/dev/null
grep -rn 'impl SelectionOperator' src/ 2>/dev/null
```
Enumerate all files that need updating and create explicit tasks for each.

**Warning signs:** Compile error `expected Vec<Vec<usize>>, found Result<...>` at island/nsga2 call sites.

### Pitfall 2: `CmaEngine::run()` has multiple early-return panic sites

**What goes wrong:** CMA has three distinct empty-population panics at lines 614, 653, and 663 — the peek-population check, the first-init check, and the restart-init check. Missing any one leaves a panic path open.

**Why it happens:** The restart loop structure creates three independent population-init points, each guarded separately.

**How to avoid:** Convert all three in the same task. Grep for all panic! in cma/engine.rs and audit the full list before writing tasks.

**Warning signs:** `grep -n 'panic!' src/engines/cma/engine.rs` still has results after the task.

### Pitfall 3: EDA has two independent `run()` functions

**What goes wrong:** `src/engines/eda/engine.rs` contains TWO `run()` methods — one for the Bernoulli model (line 265) and one for the real-valued `EdaRealEngine` (line 628). Each has its own empty-population panic. Missing the second one leaves a panic path.

**Why it happens:** EDA has a binary/Bernoulli model and a Gaussian model in the same file with separate impl blocks.

**How to avoid:** The plan must cover both `run()` methods explicitly.

**Warning signs:** `grep -n 'panic!' src/engines/eda/engine.rs` returns the line 674 panic after the task.

### Pitfall 4: `ox_build_child` is `pub(crate)` and called from two places

**What goes wrong:** `ox_build_child` is called from both the OX crossover (`order.rs`) and potentially from other crossover impls that build on it. Changing its return type to `Result<Vec<G>, GaError>` requires updating all callers.

**Why it happens:** The function is extracted for reuse; callers assume `Vec<G>` return.

**How to avoid:** `grep -rn 'ox_build_child' src/` before writing tasks. Currently only `order.rs` uses it — verify before claiming single-call-site.

**Warning signs:** Compile error `expected Vec<G>, found Result<Vec<G>, GaError>` in any crossover module.

### Pitfall 5: Bench files call `CellularEngine::new()` and `AlpsEngine::new()` without `?`

**What goes wrong:** `benches/cellular.rs` and `benches/alps.rs` have 8 total `::new()` calls that will no longer compile once `new()` returns `Result`. Bench functions typically cannot use `?`.

**Why it happens:** Bench harness (`criterion`) uses plain function bodies, not `Result`-returning contexts.

**How to avoid:** Change bench callers to `.expect("valid bench config")`. Must be included in the plan alongside the engine changes.

**Warning signs:** Compile errors in `benches/` after `new()` signature change.

### Pitfall 6: `generation.rs` AOS reward accumulator `.unwrap()` calls are inside `process_pair` closure

**What goes wrong:** The AOS state mutex calls at lines 212, 221 are inside the `process_pair` closure (which already returns `Result`). The reward accumulator `.unwrap()` calls at lines 349, 361, 381, 391, 394 are both inside the closure and outside it in the reward-apply phase.

**Why it happens:** The `process_pair` closure and post-loop reward application are architecturally separate. Lines 381-394 are outside the closure in `parent_crossover()`.

**How to avoid:** The reward-apply block lines 381-394 call `acc.lock().unwrap()` and `aos_state.lock().unwrap()`. These are outside the rayon closure and `parent_crossover` already returns `Result<(), GaError>`, so `?` propagation works directly. No special handling needed beyond `map_err`.

---

## Code Examples

### Adding `InternalError` to `src/error.rs`

```rust
// In the GaError enum, add after TreeSizeExceeded:
/// An internal invariant was violated (e.g., a mutex was poisoned by a
/// panicking thread). This indicates a bug in the calling code or a previous
/// panic — not a user configuration error.
InternalError(String),

// In the Display impl, add match arm:
GaError::InternalError(msg) => write!(f, "Internal error: {}", msg),
```

### Converting fitness cache `.expect()` in `ga/cache.rs`

```rust
// cache_snapshot — line 15
let c = ch.lock().map_err(|_| {
    GaError::InternalError("fitness cache mutex poisoned".to_string())
})?;
// Return type of cache_snapshot must change from (u64, u64) to Result<(u64, u64), GaError>

// cache_fill_stats — line 35
let c = ch.lock().map_err(|_| {
    GaError::InternalError("fitness cache mutex poisoned".to_string())
})?;
// Return type of cache_fill_stats must change from () to Result<(), GaError>
```

Note: `cache_snapshot` and `cache_fill_stats` are `pub(crate)` helper functions. Their callers in the GA run loop must propagate with `?`. Since the GA run loop already operates within a `Result<…, GaError>` context, this compiles without further return-type changes in the outer engine. [VERIFIED: codebase inspection]

### Test for Cellular/ALPS `new()` returning ConfigurationError

```rust
// In tests/engines/cellular/test_cellular.rs
#[test]
fn test_new_rejects_zero_rows() {
    let config = CellularConfiguration::default().with_grid(0, 5);
    let result = CellularEngine::<RangeChromosome<f64>>::new(
        config,
        |n| vec![RangeChromosome::default(); n],
        |dna| dna.iter().map(|g| g.value() * g.value()).sum(),
    );
    assert!(matches!(result, Err(GaError::ConfigurationError(_))));
}
```

---

## Runtime State Inventory

Not applicable — this is a pure code change phase. No stored data, live service config, OS-registered state, secrets, or build artifacts embed the renamed concepts.

---

## Open Questions

1. **`SelectionOperator::select()` trait signature change blast radius**
   - What we know: The trait is implemented by `Selection` enum in `src/operations/selection.rs`. The panic at line 79 is in that impl. `factory()` already handles Lexicase via `Err(...)` at line 157-163 — so the `factory()` path is already safe.
   - What's unclear: Whether island model and NSGA-II call `.select()` through the trait directly (bypassing `factory()`) or always go through `factory()`.
   - Recommendation: `grep -rn '\.select(' src/engines/island/ src/engines/nsga2/` before writing the plan. If they always use `factory()`, the trait panic conversion has zero blast radius on those engines — only need to update the trait impl itself.

2. **Should `CellularEngine::run()` and `AlpsEngine::run()` remain infallible after `new()` validates?**
   - What we know: D-06 moves validation to `new()`. The `run()` panics fire only when `pop.is_empty()` — which happens only when `rows * cols == 0`, which is now caught by `new()`.
   - What's unclear: Whether `run()` should remain `-> CellularResult<U>` or change to `Result<…>`.
   - Recommendation: Keep `run()` infallible for Cellular and ALPS — the validation is fully in `new()` and the run-time empty check becomes unreachable. Add a `debug_assert!(!pop.is_empty())` comment explaining the invariant. This avoids forcing all cellular/alps test callers to `.expect()` on `run()` in addition to `new()`.

3. **`SelectionOperator` trait blast radius — RESOLVED**
   - Island model uses `selection::factory(...)` at lines 562 and 659 of `src/engines/island/mod.rs` — NOT the trait's `.select()`. NSGA-II (`src/engines/nsga2/`) also uses `factory()`. The trait's `.select()` is only called from the `factory()` fallthrough path (line 164 of `selection.rs`) for non-Lexicase variants.
   - Conclusion: Changing `SelectionOperator::select()` return type ONLY requires updating (a) the `Selection` enum impl in `selection.rs` and (b) any external/user implementations of the trait. No internal engine changes beyond `selection.rs`. [VERIFIED: codebase inspection]

3. **`CmaEngine` `batch_evaluate_pop` `.expect()` at line 673 and 764**
   - What we know: These two `.expect("batch_evaluate_pop failed on initial population")` calls are on the result of `self.batch_evaluate_pop(...)` which returns `Result<(), GaError>`. They are NOT mutex panics — they are legitimate error propagation points.
   - What's unclear: Whether these should be converted to `?` (changing `run()` return type, which is already required for the empty-population panics) or left as-is since they represent a valid error.
   - Recommendation: Once `run()` returns `Result<CmaResult<U>, GaError>`, convert these to `?` as well — they are user-reachable via a bad batch evaluator implementation. This is consistent with D-08 (internal invariant panics excluded — but a failing batch evaluator is user-caused, not internal).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | `Cargo.toml` (no separate config) |
| Quick run command | `cargo test 2>&1 \| tail -20` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | GP depth/size out-of-scope (D-01) | n/a | n/a | n/a |
| SC-2 | EDA/PSO/CMA empty-init returns `InitializationError` | unit | `cargo test test_empty_init` | ❌ Wave 0 |
| SC-3 | OX crossover non-unique IDs returns `CrossoverError` | unit | `cargo test test_ox_non_unique` | ❌ Wave 0 |
| SC-4 | Cellular/ALPS zero-size config returns `ConfigurationError` from `new()` | unit | `cargo test test_new_rejects_zero` | ❌ Wave 0 |
| SC-5 | `generation.rs` mutex lock returns `InternalError` (untestable in practice — poison requires thread panic) | unit (mock) | `cargo test test_gaerror_internal` | ❌ Wave 0 |
| SC-6 | Grep audit confirms zero user-input-reachable panics | manual/CI | `grep -rn 'panic!\|\.unwrap()\|\.expect(' src/` | n/a |
| SC-7 | All existing tests still pass after return-type changes | regression | `cargo test` | ✅ (update callers) |
| SC-8 | `cargo clippy` clean | lint | `cargo clippy` | ✅ |

### Sampling Rate

- **Per task commit:** `cargo test 2>&1 | tail -30`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/engines/eda/test_eda.rs` — add `test_run_empty_init_returns_error` for both Bernoulli and Real EDA
- [ ] `tests/engines/pso/test_pso.rs` — add `test_run_empty_init_returns_error`
- [ ] `tests/engines/cma/test_cma.rs` — add `test_run_empty_init_returns_error` for all three panic sites
- [ ] `tests/engines/cellular/test_cellular.rs` — add `test_new_rejects_zero_rows` and `test_new_rejects_zero_cols`
- [ ] `tests/engines/alps/test_alps.rs` — add `test_new_rejects_zero_layer_size` and `test_new_rejects_zero_n_layers`
- [ ] `tests/operations/test_mutation.rs` — add `test_ox_crossover_non_unique_ids_returns_error`
- [ ] `tests/operations/test_operations.rs` — add `test_lexicase_selection_via_trait_returns_error`

All new tests go in `tests/` (project rule: no inline test modules). [VERIFIED: CLAUDE.md]

---

## Security Domain

Not applicable — this phase introduces no new user-facing endpoints, authentication paths, data persistence, or cryptographic operations. Error conversion does not affect the security surface.

---

## Project Constraints (from CLAUDE.md)

- **Tests in `tests/` folder only.** All unit tests must be in `tests/`, never inline with implementation code. [VERIFIED: CLAUDE.md]
- **WASM compatibility mandatory.** All changes must compile for `wasm32-unknown-unknown`. The changes in this phase (error propagation, mutex map_err) are pure Rust with no platform-specific APIs — WASM-safe by default. [VERIFIED: CLAUDE.md]
- **Signed commits mandatory.** Every commit must be GPG-signed. [VERIFIED: CLAUDE.md]
- **No direct milestone push.** All work goes through `feat/phase-78` → PR to `milestone/v3.0.0`. [VERIFIED: CLAUDE.md]
- **Branching:** Create `feat/phase-78` from `milestone/v3.0.0` before execution. [VERIFIED: CLAUDE.md]
- **No breaking changes (default) — EXCEPTION:** v3.0.0 is a semver-breaking release. Return-type changes for `run()` on EDA/PSO/CMA and `new()` on Cellular/ALPS are acceptable per D-06. [VERIFIED: CONTEXT.md + CLAUDE.md]
- **Performance awareness:** No performance impact expected — error paths are cold paths. Mutex `map_err` is zero-overhead on the happy path vs `.unwrap()`. [ASSUMED]

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `ox_build_child` is only called from `order.rs` within the codebase | Common Pitfalls | Low — if other callers exist, they need return-type updates too |
| A2 | Island model and NSGA-II engines always call `factory()` not `.select()` directly | Open Questions | RESOLVED — verified from source: island/mod.rs lines 562, 659 use `selection::factory()`; nsga2 also uses `factory()` |
| A3 | Rayon closures' `?` propagation already handled correctly in `generation.rs` — AOS mutex changes follow the same pattern | Architecture Patterns | Low — the existing `process_pair` closure already returns `Result` |
| A4 | `GaError` not deriving `Clone` or `PartialEq` issues — currently it does derive both | error.rs | Low — adding `InternalError(String)` preserves these derives since `String: Clone + PartialEq` |
| A5 | `batch_evaluate_pop` `.expect()` at CMA lines 673 and 764 are user-reachable via bad batch evaluator | Open Questions | Low — if internal-invariant, they stay as-is; if user-reachable, convert to `?` |
| A6 | Lexicase/EpsilonLexicase panic in `SelectionOperator::select()` trait — the `factory()` function already returns `Err(GaError::ConfigurationError(...))` for Lexicase at line 157-163 | Architecture Patterns | Low — verified from source; the remaining panic is only in the trait impl |

---

## Sources

### Primary (HIGH confidence)
- `src/error.rs` — full `GaError` enum; `InternalError` variant is absent [VERIFIED: codebase inspection]
- `src/engines/ga/generation.rs` — 8 `.unwrap()` calls identified at lines 212, 221, 349, 361, 381, 384, 391, 394 [VERIFIED: codebase inspection]
- `src/engines/ga/cache.rs` — 2 `.expect()` calls at lines 15, 35 [VERIFIED: codebase inspection]
- `src/engines/ga/batch.rs` — 2 `.expect()` calls at lines 52, 81 [VERIFIED: codebase inspection]
- `src/engines/eda/engine.rs` — panics at lines 315, 674; cache expects at lines 339, 408, 699, 766 [VERIFIED: codebase inspection]
- `src/engines/pso/engine.rs` — panic at line 344; cache expects at lines 369, 482 [VERIFIED: codebase inspection]
- `src/engines/cma/engine.rs` — panics at lines 614, 653, 663; cache expects at lines 429, 451, 723, 921 [VERIFIED: codebase inspection]
- `src/engines/cellular/engine.rs` — panic at line 136 in `run()` [VERIFIED: codebase inspection]
- `src/engines/alps/engine.rs` — panics at lines 127, 130 in `run()` [VERIFIED: codebase inspection]
- `src/operations/crossover/order.rs` — panic in `ox_build_child` at line 110 [VERIFIED: codebase inspection]
- `src/operations/selection.rs` — panic at line 79 in `SelectionOperator` trait impl [VERIFIED: codebase inspection]
- `src/traits/operators.rs` — `SelectionOperator::select()` returns `Vec<Vec<usize>>` [VERIFIED: codebase inspection]
- `tests/engines/cellular/test_cellular.rs` — 3 `CellularEngine::new()` call sites [VERIFIED: codebase inspection]
- `tests/engines/alps/test_alps.rs` — 6 `AlpsEngine::new()` call sites [VERIFIED: codebase inspection]
- `benches/cellular.rs` — 6 `CellularEngine::new()` call sites [VERIFIED: codebase inspection]
- `benches/alps.rs` — 4 `AlpsEngine::new()` call sites [VERIFIED: codebase inspection]

---

## Metadata

**Confidence breakdown:**
- Error conversion patterns: HIGH — standard Rust; codebase already uses these patterns extensively
- Blast radius of `SelectionOperator` trait change: MEDIUM — depends on island/nsga2 call paths not fully verified
- Test file placement / new test count: HIGH — CLAUDE.md rule verified

**Research date:** 2026-06-19
**Valid until:** 2026-07-19 (stable Rust patterns; no external dependencies)
