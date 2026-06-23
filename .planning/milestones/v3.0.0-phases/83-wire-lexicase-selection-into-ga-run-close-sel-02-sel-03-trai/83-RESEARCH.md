# Phase 83 Research: Wire Lexicase Selection into Ga::run()

**Researched:** 2026-06-23
**Domain:** Rust genetic algorithm engine integration — selection operator dispatch
**Confidence:** HIGH

---

## Summary

Phase 50 (completed 2026-05-23) implemented the `VectorFitness` trait, `Selection::Lexicase`
and `Selection::EpsilonLexicase` enum variants, and the `selection::factory_lexicase` dispatch
function. However, `Ga::run_with_callback()` in `src/engines/ga/mod.rs` calls
`selection::factory()` unconditionally at line 1467, and `factory()` explicitly returns
`GaError::ConfigurationError` for `Lexicase`/`EpsilonLexicase`. A separate public method
`select_parents_lexicase()` was added to `Ga<U: VectorFitness>` as a workaround but is **never
called from `run()`**. Phase 83 closes this gap: the generation loop must detect lexicase
selection at runtime and route to `factory_lexicase` instead of `factory`, with the only
prerequisite being that `U: VectorFitness`.

**Primary recommendation:** Add a runtime branch inside `run_with_callback()`'s generation loop
that checks `configuration.selection_configuration.method` and calls `factory_lexicase` when it
matches `Lexicase` or `EpsilonLexicase`. No changes to `ChromosomeT`, no new trait bounds on
`run()`, no breaking changes. The `Ga<U>` struct's existing `run()` impl block already imports
`VectorFitness`; the fix requires the generic bound `U: VectorFitness` to be present when the
branch is taken — this is achievable via a helper function in the
`impl<U: LinearChromosome + VectorFitness + ...>` block already at line 2435.

---

## Project Constraints (from CLAUDE.md)

- All tests MUST live in `tests/`, never inline with implementation.
- WASM compatibility mandatory: no `Instant::now()` and no `par_iter()` in lexicase path
  (already satisfied by Phase 50 implementation — lexicase is intentionally sequential).
- No breaking changes: `ChromosomeT` and `Ga<U>` public API must be backward-compatible.
- Every commit must be GPG-signed.
- Branch must be `feat/phase-83` created from `milestone/v3.0.0`.
- PR targets `milestone/v3.0.0`, not `main`.

---

## Current State Analysis

### What Phase 50 Delivered (COMPLETE) [VERIFIED: codebase]

| Component | Location | Status |
|-----------|----------|--------|
| `VectorFitness` trait | `src/traits/vector_fitness.rs` | Complete — `fitness_values() -> &[f64]`, `set_fitness_values(Vec<f64>)` |
| `MultiCaseFitness` trait | `src/traits/multi_case_fitness.rs` | Complete — `case_fitness() -> &[f64]`, `set_case_fitness(Vec<f64>)` |
| `Selection::Lexicase` variant | `src/operations/mod.rs` (Selection enum) | Complete |
| `Selection::EpsilonLexicase` variant | `src/operations/mod.rs` (Selection enum) | Complete |
| `selection::factory_lexicase()` | `src/operations/selection.rs` lines 224–282 | Complete — takes `&mut [U: VectorFitness]`, returns `Vec<Vec<usize>>`, syncs scalar fitness to mean of case scores (D-04) |
| `lexicase_selection()` | `src/operations/selection/lexicase.rs` | Complete — Fisher-Yates shuffle, shrinking-pool cascade, WASM-safe |
| `epsilon_lexicase_selection()` | `src/operations/selection/lexicase.rs` | Complete — fixed `epsilon` or dynamic MAD |
| `Ga::select_parents_lexicase()` | `src/engines/ga/mod.rs` lines 2476–2482 | Present — but NOT wired into `run()` |
| `SelectionConfiguration.epsilon` | `src/configuration.rs` | Present |

### What is Missing (THE GAP) [VERIFIED: codebase]

`Ga::run_with_callback()` calls `selection::factory()` unconditionally:

```rust
// src/engines/ga/mod.rs line 1467
let parents = selection::factory(
    &self.population.chromosomes,
    self.configuration.selection_configuration,
    self.configuration.number_of_threads,
    num_parents,
)?;
```

`selection::factory()` contains:

```rust
// src/operations/selection.rs lines 170–175
Selection::Lexicase | Selection::EpsilonLexicase => {
    return Err(GaError::ConfigurationError(
        "Use selection::factory_lexicase for Lexicase/EpsilonLexicase; \
         standard factory() does not support VectorFitness bound."
            .into(),
    ));
}
```

This means: any user who sets `.with_selection_method(Selection::Lexicase)` and calls `ga.run()`
will receive `GaError::ConfigurationError` at runtime — even if their chromosome implements
`VectorFitness`. Requirements SEL-02, SEL-03, TRAITS-01 from Phase 50 are technically complete
in isolation, but the `Ga::run()` integration path is broken.

### The `SelectionOperator` Trait Path [VERIFIED: codebase]

The `SelectionOperator` trait dispatch (used by island model and NSGA-II) also rejects Lexicase
with a `SelectionError` ("use selection::factory_lexicase"). This is correct and intentional —
those engines do not support `VectorFitness`. The error message explicitly documents this. No
change needed for the island/NSGA-II path.

---

## Gap Analysis (SEL-02 / SEL-03 / TRAITS-01)

These issue codes come from Phase 50's requirements, confirmed in `ROADMAP.md`:

| Code | Requirement | Phase 50 Status | Phase 83 Gap |
|------|-------------|-----------------|--------------|
| TRAITS-01 | `MultiCaseFitness: ChromosomeT` with `case_fitness()` / `set_case_fitness()` | Complete — trait exists, publicly exported | None — no action needed |
| SEL-02 | `Selection::Lexicase` shuffles test cases per selection event; scalar fitness = mean case score | Complete in `lexicase_selection()` + `factory_lexicase()` | NOT wired into `run()` — Phase 83 wires it |
| SEL-03 | `Selection::EpsilonLexicase { epsilon }` retains candidates within epsilon of best per case | Complete in `epsilon_lexicase_selection()` | NOT wired into `run()` — Phase 83 wires it |

The Phase 50 UAT at `.planning/phases/50-lexicase-selection/50-UAT.md` documents:
> "Test 7: Ga engine integrates lexicase via select_parents_lexicase() method — pass"

The UAT considered the method's *existence* sufficient. The gap is that `run()` doesn't call it.

---

## Technical Approach

### Core Constraint: Two impl Blocks

`Ga::run_with_callback()` lives in the impl block at line 773 with bounds:

```rust
impl<U> Ga<U>
where
    U: LinearChromosome + Send + Sync + 'static + Clone + Debug
      + mutation::ValueMutable + MaybeSerialize + MaybeDeserialize
      + OperatorCompat + crate::traits::RealValuedMutation,
    U::Gene: 'static + Debug,
```

This block does NOT have `U: VectorFitness`. Adding `+ VectorFitness` here would be a **breaking
change** — it would force all existing `Ga<U>` users to implement `VectorFitness` on their
chromosomes. This is unacceptable.

### Recommended Approach: Runtime Match + Delegating Helper [ASSUMED]

The cleanest non-breaking approach uses a two-step pattern:

**Step 1** — inside `run_with_callback()`, replace the unconditional `selection::factory()` call
with a match on the selection method:

```rust
// in engines/ga/mod.rs run_with_callback() at line 1467
let parents = match self.configuration.selection_configuration.method {
    Selection::Lexicase | Selection::EpsilonLexicase => {
        self.select_parents_lexicase_dyn()?
    }
    _ => selection::factory(
        &self.population.chromosomes,
        self.configuration.selection_configuration,
        self.configuration.number_of_threads,
        num_parents,
    )?,
};
```

**Step 2** — add `select_parents_lexicase_dyn()` as a helper method on the base `impl` block
(line 773) that does a dynamic downcast or returns an error when `U` doesn't implement
`VectorFitness`. Since Rust doesn't support runtime trait introspection, this helper returns
`GaError::ConfigurationError` with a descriptive message when called on a non-`VectorFitness`
type.

However, this approach still won't compile: `select_parents_lexicase_dyn()` in the base impl
block cannot call `factory_lexicase()` without `U: VectorFitness`.

### Actually Correct Approach: Separate Lexicase Run Loop [ASSUMED]

The cleanest architectural solution is a separate `run_lexicase()` method on the `impl<U:
LinearChromosome + VectorFitness + ...>` block (which already exists at line 2435), which reuses
the full generation loop logic but replaces the single `selection::factory()` call with
`factory_lexicase()`.

However, duplicating the 800-line generation loop is not maintainable.

### Best Approach: Extracted Helper + Trait Object Dispatch [ASSUMED]

The cleanest low-duplication approach:

1. **Extract** the selection call in `run_with_callback()` into a closure or helper that accepts
   a mutable population slice and returns `Vec<Vec<usize>>`. The default implementation calls
   `selection::factory()`.

2. **Override** this helper in a `VectorFitness`-constrained impl block that routes to
   `factory_lexicase()`.

3. In `run_with_callback()`, detect lexicase at runtime via the configuration:

```rust
// Option A: explicit guard at top of run_with_callback()
if matches!(
    self.configuration.selection_configuration.method,
    Selection::Lexicase | Selection::EpsilonLexicase
) {
    return Err(GaError::ConfigurationError(
        "Selection::Lexicase requires U: VectorFitness. Use Ga<U: VectorFitness>::run_lexicase()".into()
    ));
}
```

4. Add `run_lexicase()` to the `VectorFitness`-constrained impl that is identical to
   `run_with_callback()` but calls `factory_lexicase()` at the selection step.

This avoids duplication of the full loop by factoring out the selection call into a named type:

```rust
// In the base impl block:
type SelectFn<'a, U> = dyn Fn(&mut [U], SelectionConfiguration, usize, usize)
    -> Result<Vec<Vec<usize>>, GaError> + 'a;

fn run_inner<F>(&mut self, select_fn: &F, ...) -> ...
where F: Fn(&mut [U], ...) -> ...
```

### Simplest Correct Approach (Recommended for Phase 83) [ASSUMED]

Given the single selection call in `run_with_callback()`, the pragmatic solution is:

1. Add a **build-time validation** in `Ga::build()` that rejects `Lexicase`/`EpsilonLexicase`
   unless the chromosome type implements `VectorFitness` — but Rust doesn't support this at build
   time without specialization.

2. The **actual recommended approach**: provide `run_lexicase()` in the `VectorFitness`-
   constrained `impl` block. This method is a thin wrapper over an internal helper that shares
   the full generation loop logic with `run_with_callback()`, with the selection call
   parameterized. The factoring:

   - Extract a `run_loop_inner` private function that takes a `select_fn: impl Fn(...)`
   - `run_with_callback` passes `selection::factory` as the select_fn
   - `run_lexicase` passes `selection::factory_lexicase` as the select_fn

3. **OR** (simpler, less refactor): Keep `run_with_callback` as-is but change the
   `Lexicase`/`EpsilonLexicase` arm in `selection::factory` to call `factory_lexicase` via a
   **different mechanism**: instead of returning an error, cast the slice to `&mut [dyn
   VectorFitness]` — which is not possible with Rust's type system without unsafe code.

### Final Recommended Approach (Confirmed Architecturally Sound) [ASSUMED]

The lowest-risk approach that requires minimal code changes:

**Step 1**: In `run_with_callback()` (line 1467), replace:
```rust
let parents = selection::factory(...)?;
```
with:
```rust
let parents = if matches!(
    self.configuration.selection_configuration.method,
    crate::operations::Selection::Lexicase | crate::operations::Selection::EpsilonLexicase
) {
    // Lexicase path: self must have U: VectorFitness — delegate to the VectorFitness impl block
    return Err(GaError::ConfigurationError(
        "Selection::Lexicase/EpsilonLexicase requires the chromosome to implement \
         VectorFitness. Use ga.run_lexicase() instead of ga.run(). See docs on VectorFitness.".into()
    ));
} else {
    selection::factory(
        &self.population.chromosomes,
        self.configuration.selection_configuration,
        self.configuration.number_of_threads,
        num_parents,
    )?
};
```

**Step 2**: Add `run_lexicase()` to the `VectorFitness`-constrained impl block at line 2435.
This method duplicates `run_with_callback()` but replaces the selection call with
`factory_lexicase()`. Given the cost of full duplication, a better alternative is:

**Step 2 (alternative)**: Add `run()` and `run_with_callback()` to the `VectorFitness`-
constrained impl block that override the base ones and include the lexicase dispatch. The Rust
compiler will prefer the more specific `VectorFitness`-constrained impl when `U: VectorFitness`,
but Rust does not support method specialization in stable Rust — both impl blocks would have the
same method names, causing a conflict.

**Step 2 (final clean answer)**: The correct Rust solution is to introduce a separate entry
point:

```rust
impl<U> Ga<U>
where
    U: LinearChromosome + VectorFitness + Send + Sync + 'static + Clone + Debug
      + mutation::ValueMutable + MaybeSerialize + MaybeDeserialize
      + OperatorCompat + crate::traits::RealValuedMutation,
    U::Gene: 'static + Debug,
{
    /// Runs the GA using lexicase or epsilon-lexicase selection.
    ///
    /// Use this instead of `run()` when `Selection::Lexicase` or
    /// `Selection::EpsilonLexicase` is configured. The chromosome type must
    /// implement `VectorFitness` with per-test-case scores populated in
    /// `calculate_fitness()`.
    pub fn run_lexicase(&mut self) -> Result<&Population<U>, GaError> {
        self.run_lexicase_with_callback(
            None::<fn(&usize, &Population<U>, &GenerationStats, &TerminationCause)
                -> ControlFlow<()>>,
            0,
        )
    }

    pub fn run_lexicase_with_callback<F>(&mut self, ...) { ... }
}
```

Where `run_lexicase_with_callback` is identical to `run_with_callback` except line 1467 calls
`factory_lexicase` instead of `factory`. The call signatures and the rest of the generation loop
are identical.

**Duplication concern**: `run_with_callback` is ~800 lines. Duplicating it is risky for
maintenance. The recommended phased approach:

1. **Phase 83 scope**: The minimum viable wiring — extract the selection step into a
   `fn do_selection(...)` helper that `run_with_callback` passes to an internal
   `run_loop(select_fn)`. Then `run_lexicase_with_callback` calls the same `run_loop` with a
   different `select_fn`. This is the correct long-term architecture.

2. If the refactor is too large for Phase 83, a minimal solution is acceptable: copy the
   `run_with_callback` body into `run_lexicase_with_callback` and change the one selection line.
   Leave a `// TODO: Phase N — consolidate with run_with_callback` comment.

---

## Key Files

| File | Action | Why |
|------|--------|-----|
| `src/engines/ga/mod.rs` | Modify `run_with_callback()` at line 1467; add `run_lexicase()` + `run_lexicase_with_callback()` to `VectorFitness` impl block | Core wiring change |
| `src/operations/selection.rs` | No change needed — `factory_lexicase` is already correct | Complete from Phase 50 |
| `src/operations/selection/lexicase.rs` | No change needed | Complete from Phase 50 |
| `src/traits/vector_fitness.rs` | No change needed | Complete from Phase 50 |
| `src/traits/multi_case_fitness.rs` | No change needed | Complete from Phase 50 |
| `tests/engines/lexicase/` (new directory) | New test module: `test_ga_run_lexicase.rs` | Integration test wiring `run_lexicase()` end-to-end |

---

## Risk Assessment

### Breaking Change Risk: NONE [VERIFIED: codebase]

- `ChromosomeT` is unchanged — no new methods required on the core trait.
- `VectorFitness` is an opt-in supertrait — existing chromosomes that do not implement it are
  unaffected.
- `Ga::run()` and `Ga::run_with_callback()` signatures are unchanged — existing callers compile
  as-is. The only change to existing behavior is that `run_with_callback()` now returns a clearer
  `ConfigurationError` for Lexicase/EpsilonLexicase (it returned this error before too, via
  `factory()` — no behavior change, better message).
- The new `run_lexicase()` method is additive.

### WASM Compatibility: SAFE [VERIFIED: codebase]

The lexicase operator (`src/operations/selection/lexicase.rs`) explicitly documents:
> "WASM note: This implementation uses sequential `.iter()` throughout. The shrinking-pool state
> in the filter cascade cannot be parallelised."

No `par_iter()`, no `Instant::now()`. The Phase 50 UAT confirms `cargo check --target
wasm32-unknown-unknown` passes with the lexicase code. Phase 83 adds no new WASM risk.

### Test Coverage Gap [VERIFIED: codebase]

There are NO existing tests that:
1. Create a `Ga<U: VectorFitness>` with `Selection::Lexicase`
2. Call `ga.run()` (or `run_lexicase()`) and observe convergence

Existing tests only test the `lexicase_selection()` function and `factory_lexicase()` in
isolation. Phase 83 must add integration tests.

### Multi-Parent Crossover Interaction [ASSUMED]

The lexicase operator clamps `num_parents` to 2 (see `factory_lexicase` and
`lexicase_selection`). If the user configures `Crossover::Undx { num_parents: 5 }` with
`Selection::Lexicase`, the selection produces 2-parent groups but crossover expects 5. This is an
existing design decision from Phase 50 (documented in `factory_lexicase` docstring: "Lexicase
always produces groups of 2"). The run loop should not attempt to pass `num_parents > 2` to
`factory_lexicase` — it always uses 2.

### AOS Interaction [ASSUMED]

The existing `run_with_callback()` applies AOS (Adaptive Operator Selection) rewards after
crossover. The lexicase path must preserve this behavior if AOS is configured. The AOS logic
lives entirely in `generation::parent_crossover()` which is called after selection — it does not
depend on which selection method was used. No change needed in `parent_crossover()`.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | `Cargo.toml` |
| Quick run command | `cargo test test_ga_run_lexicase` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy --all-targets -- -D warnings` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| SEL-02 | `Ga<U: VectorFitness>.run_lexicase()` completes without error | integration | `cargo test test_ga_run_lexicase_completes` | No — Wave 0 |
| SEL-02 | Lexicase produces diverse parent selection (specialists preserved) | behavioral | `cargo test test_run_lexicase_diversity` | No — Wave 0 |
| SEL-03 | `Selection::EpsilonLexicase` wired into `run_lexicase()` | integration | `cargo test test_ga_run_epsilon_lexicase_completes` | No — Wave 0 |
| SEL-02 | `ga.run()` with `Lexicase` returns `ConfigurationError` (clear message) | unit | `cargo test test_run_lexicase_on_non_vector_fitness_returns_error` | No — Wave 0 |
| TRAITS-01 | Scalar fitness synced to mean case score after lexicase selection | unit | `cargo test test_lexicase_mean_sync_in_run` | No — Wave 0 |

### Wave 0 Test File
- `tests/engines/lexicase/test_ga_run_lexicase.rs` — covers all 5 test cases above
- Requires a minimal `MultiCaseChromosome` fixture (reuse from `tests/structures.rs` or copy)

### Sampling Rate
- **Per task commit:** `cargo test test_ga_run_lexicase`
- **Per wave merge:** `cargo test && cargo test --features serde`
- **Phase gate:** Full CI matrix — `cargo test`, `cargo test --features serde`,
  `cargo clippy --all-targets -- -D warnings`, `cargo doc --no-deps`,
  `cargo check --target wasm32-unknown-unknown`

---

## Architecture Patterns

### Existing Pattern: Selection Dispatch in run_with_callback [VERIFIED: codebase]

```rust
// src/engines/ga/mod.rs line 1461-1472
let num_parents = match self.configuration.crossover_configuration.method {
    crate::operations::Crossover::Undx { num_parents }
    | crate::operations::Crossover::Spx { num_parents }
    | crate::operations::Crossover::Pcx { num_parents } => num_parents,
    _ => 2,
};
let parents = selection::factory(
    &self.population.chromosomes,
    self.configuration.selection_configuration,
    self.configuration.number_of_threads,
    num_parents,
)?;
```

Phase 83 needs to either branch before this call (in `run_lexicase_with_callback`) or replace
this call with a parameterized select function.

### Existing Pattern: VectorFitness-Constrained impl Block [VERIFIED: codebase]

```rust
// src/engines/ga/mod.rs line 2435-2483
impl<U> Ga<U>
where
    U: LinearChromosome + VectorFitness + Send + Sync + 'static + Clone + Debug
      + mutation::ValueMutable + MaybeSerialize + MaybeDeserialize
      + OperatorCompat,
    U::Gene: 'static + Debug,
{
    pub fn select_parents_lexicase(&mut self) -> Result<Vec<Vec<usize>>, GaError> {
        crate::operations::selection::factory_lexicase(
            &mut self.population.chromosomes,
            self.configuration.selection_configuration,
            self.configuration.number_of_threads,
        )
    }
}
```

This impl block is where `run_lexicase()` should be added. Note it currently lacks the
`+ crate::traits::RealValuedMutation` bound that the `run_with_callback` impl block requires.
Adding `run_lexicase_with_callback` here requires adding `+ crate::traits::RealValuedMutation`
to match the `run_with_callback` bound.

### Existing Pattern: factory_lexicase Sync [VERIFIED: codebase]

```rust
// src/operations/selection.rs line 273-279
// D-04: sync scalar fitness to mean of fitness values
for c in chromosomes.iter_mut() {
    let scores = c.fitness_values().to_vec();
    if !scores.is_empty() {
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        c.set_fitness(mean);
    }
}
```

The scalar fitness sync happens inside `factory_lexicase()` — no additional sync needed in the
run loop. However, `factory_lexicase` takes `&mut [U]` (mutable borrow to perform the sync),
while `factory` takes `&[U]` (immutable). The run loop must pass a mutable slice for the lexicase
path.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Lexicase algorithm | Custom filter cascade | `selection::factory_lexicase()` already exists | Phase 50 complete |
| VectorFitness trait | New trait | `VectorFitness` (Phase 50) | Already exported from `src/traits` |
| Test chromosome with case scores | New struct | `MultiCaseChromosome` from `tests/structures.rs` | Already used in Phase 50 tests |
| MAD epsilon computation | Custom stats | `compute_mad_epsilons()` in `lexicase.rs` | Already implemented |

---

## Common Pitfalls

### Pitfall 1: Adding VectorFitness Bound to run_with_callback
**What goes wrong:** Changing the `where` clause of the main `run_with_callback` impl to add
`+ VectorFitness` breaks all existing users who compile `Ga<BinaryChromosome>` etc.
**Why it happens:** Seems like the "obvious" fix.
**How to avoid:** Use a separate `run_lexicase_with_callback` in the VectorFitness-constrained
impl block. Do not modify the base impl block's type bounds.

### Pitfall 2: factory_lexicase Takes &mut [U] Not &[U]
**What goes wrong:** The run loop passes `&self.population.chromosomes` (immutable borrow) to
factory calls. `factory_lexicase` needs `&mut [U]` for the scalar-fitness sync step.
**Why it happens:** `factory()` is read-only; `factory_lexicase()` mutates chromosomes to sync
scalar fitness.
**How to avoid:** In `run_lexicase_with_callback`, pass
`&mut self.population.chromosomes` to `factory_lexicase`. Since `self` is `&mut self` in
`run_with_callback`, this is valid — but note that `run_with_callback` takes `&self.population
.chromosomes` currently (immutable). The lexicase-specific path needs the mutable reference.

### Pitfall 3: num_parents > 2 with Lexicase
**What goes wrong:** Passing `num_parents > 2` (from UNDX/SPX/PCX) to `factory_lexicase` which
clamps to 2 internally — silently producing 2-parent groups while crossover expects more.
**Why it happens:** `num_parents` is derived from the crossover method before the selection call.
**How to avoid:** In `run_lexicase_with_callback`, force `num_parents = 2` regardless of the
crossover method. Multi-parent crossover with lexicase selection is an unsupported combination —
consider returning a `ConfigurationError` at `build()` time if both are configured.

### Pitfall 4: Missing RealValuedMutation Bound on VectorFitness impl Block
**What goes wrong:** Compilation error: `the method run_lexicase_with_callback exists for struct
Ga<U>, but its trait bounds were not satisfied` when calling it with a concrete chromosome type.
**Why it happens:** `run_with_callback` requires `+ crate::traits::RealValuedMutation` because
`generation::parent_crossover` uses it. The VectorFitness-constrained impl at line 2435 currently
lacks this bound.
**How to avoid:** Add `+ crate::traits::RealValuedMutation` to the VectorFitness impl block's
where clause.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `run_lexicase_with_callback` can reuse `generation::parent_crossover` unchanged | Technical Approach | Low — `parent_crossover` doesn't depend on selection method |
| A2 | Multi-parent crossover (UNDX/SPX/PCX) + Lexicase is an unsupported combination | Technical Approach / Pitfall 3 | Low — could silently produce wrong group sizes; fix = validate at build() |
| A3 | The simplest viable approach is a separate `run_lexicase_with_callback` method rather than refactoring run_with_callback | Technical Approach | Medium — if the generation loop changes in future phases, the duplicate must be updated |
| A4 | Adding `+ RealValuedMutation` to the VectorFitness impl block is sound (no circular bound) | Technical Approach | Low — `RealValuedMutation` is a marker trait |

---

## Open Questions

1. **Duplication strategy**
   - What we know: `run_with_callback` is ~800 lines; duplicating it in `run_lexicase_with_callback` creates a maintenance burden.
   - What's unclear: Whether the user wants a full refactor (extract `run_loop_inner`) or accepts the duplication for Phase 83 with a TODO for a future cleanup phase.
   - Recommendation: For Phase 83, accept duplication with a `// TODO Phase N: consolidate` comment. Add a tracked issue for the cleanup.

2. **API naming: `run_lexicase()` vs `run()` routing**
   - What we know: Current API requires users to know they must call `run_lexicase()` instead of `run()` when using lexicase.
   - What's unclear: Whether the user wants `run()` to automatically detect `VectorFitness + Lexicase` config and delegate, or to keep `run_lexicase()` as a separate entry point.
   - Recommendation: Keep `run_lexicase()` as a distinct entry point for v3.0.0 (explicit is better); add a clear error message in `run()` pointing to `run_lexicase()` when lexicase is configured.

3. **Behavioral diversity test from Phase 50 UAT test 4**
   - What we know: `test_selection_lexicase_diversity.rs` confirms lexicase produces 1.2× more specialists than tournament in isolation.
   - What's unclear: Whether Phase 83 should add a similar diversity test at the `run_lexicase()` level (full GA run, not just selection call).
   - Recommendation: Yes — add a convergence/diversity integration test that runs 20+ generations.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| `cargo test` | Test suite | ✓ | (project standard) | — |
| `cargo clippy` | CI gate | ✓ | (project standard) | — |
| `wasm32-unknown-unknown` target | WASM CI check | ✓ | (Phase 34 installed it) | — |

---

## Sources

### Primary (HIGH confidence)
- `src/engines/ga/mod.rs` — direct codebase inspection of `run_with_callback()` at line 1467 and `select_parents_lexicase()` at line 2476
- `src/operations/selection.rs` — direct codebase inspection of `factory()` and `factory_lexicase()`
- `src/operations/selection/lexicase.rs` — complete lexicase implementation
- `src/traits/vector_fitness.rs` — trait definition
- `.planning/phases/50-lexicase-selection/50-UAT.md` — Phase 50 completion status

### Secondary (MEDIUM confidence)
- `ROADMAP.md` lines 595–613 — Phase 50 requirements SEL-02, SEL-03, TRAITS-01 specification
- `ROADMAP.md` lines 495–504 — Phase 83 goal definition

---

## Metadata

**Confidence breakdown:**
- Current state analysis: HIGH — directly read from source files
- Gap identification: HIGH — confirmed by reading both `run_with_callback` and `factory`
- Technical approach: MEDIUM — the Rust type system constraints (no method specialization) drive the design; specific impl details are ASSUMED until proven by compilation
- Pitfalls: HIGH — derived from direct type-system constraints visible in the code

**Research date:** 2026-06-23
**Valid until:** 2026-07-23 (stable codebase — no external dependencies to expire)
