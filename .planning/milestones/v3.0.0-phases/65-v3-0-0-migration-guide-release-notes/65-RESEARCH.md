# Phase 65: v3.0.0 Migration Guide & Release Notes — Research

**Researched:** 2026-06-17
**Domain:** Technical writing — Rust migration guide, Keep-a-Changelog, cargo publish gate
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Add all 3 missing breaking-change entries: `DeGene → RealGene` rename, `SelectionOperator::select` return-type change (new `num_parents` param + `Vec<Vec<usize>>` return), `Mutation` enum variant parameter changes.

**D-02:** Add Phase 68/69 feature-flag changes as their own `##` sections (same level as other breaking changes): one section for `parallel` feature, one section for `logging` feature + `env_logger` removal.

**D-03:** The `LinearChromosome` bound requirement (custom chromosomes must now implement `LinearChromosome`, not just `ChromosomeT`) is folded into the existing `## Trait split: ChromosomeT + LinearChromosome` section as a prominent callout — not a separate section.

**D-04:** Every breaking-change section gets a `### Compiler error` subsection with a fenced code block showing the exact `error[E...]` output rustc emits when a user has v2 code. Applies to all 10 breaking-change entries.

**D-05:** Drop the empty `## [Unreleased]` section. Promote `## [3.0.0] - Unreleased` to `## [3.0.0] - 2026-06-17`. Existing content is already comprehensive; Phase 65 verifies completeness and adds any gaps from phases 64–69.

**D-06:** Coverage scope: all phases 47–69 summarized. Build-perf phases (66–69) included under "Architecture & quality."

**D-07:** Four-part release gate: (1) full CI matrix, (2) `cargo publish --dry-run`, (3) minimal v2 sample crate smoke-test, (4) `cargo run --example` smoke run for all examples.

**D-08:** The v2 sample crate for smoke-test uses top 3 breaking patterns only (ChromosomeT+LinearChromosome impl, Reporter removal, SelectionOperator trait impl). Does not need to solve a real optimization problem.

### Claude's Discretion

- **LinearChromosome bound callout style** within the ChromosomeT split section — decide on the most readable formatting (e.g., a `> **Note:** If you implemented ChromosomeT directly...` blockquote or a dedicated paragraph).

### Deferred Ideas (OUT OF SCOPE)

- crates.io README rendering check (verify badges, images render on crates.io page)
- docs.rs preview

</user_constraints>

---

## Summary

Phase 65 is a pure documentation and verification phase. No code changes are made. The deliverables are: (1) a completed `MIGRATION.md` that covers all 10 breaking changes, each with a `### Compiler error` subsection; (2) two new `##` sections in `MIGRATION.md` for the `parallel` and `logging` feature-flag changes; (3) a promoted `## [3.0.0] - 2026-06-17` CHANGELOG entry; and (4) a release-gate verification plan (Plan 65-03) that runs the full CI matrix, a dry-run publish, a v2 smoke-test crate, and all examples.

The existing `MIGRATION.md` already covers **7 of the 10** required breaking changes with established `### Before` / `### After` style. The three missing entries are: `DeGene → RealGene`, `SelectionOperator::select` signature change, and `Mutation` enum variant parameter changes (the `Insertion` variant was renamed `PermutationInsert`; new variants added; existing variants gained inline struct fields). These must be added.

The `CHANGELOG.md` `## [3.0.0]` section is comprehensive through Phase 69 (includes build-perf phases). The only required edits are: promote the date, drop the empty `## [Unreleased]` section above it, update the compare link from `...HEAD` to `...v3.0.0` if a tag exists, and verify phases 64/65 are represented.

**Primary recommendation:** Follow the file-by-file audit below precisely. Each section identifies exactly what exists versus what must be written — no discovery needed during planning.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| MIGRATION.md authoring | Documentation | — | Pure text; must match actual compiler error output |
| Compiler error capture | Verification (Plan 65-03) | Documentation | Errors must be captured from real rustc, not guessed |
| CHANGELOG.md editing | Documentation | — | Mechanical edit of one section date + link |
| Release gate (CI matrix) | CI / Shell | — | `cargo test`, `cargo clippy`, WASM check, `cargo doc` |
| `cargo publish --dry-run` | Shell / crates.io gate | — | Validates Cargo.toml `include`, metadata fields |
| v2 smoke-test crate | Shell (temp dir) | Documentation | Creates throwaway crate, applies migration, confirms compile |
| Examples smoke-run | Shell | CI | `cargo run --example` for each registered example |

---

## MIGRATION.md Audit — Current State vs Required

### Already covered (7 entries — DO NOT REWRITE, only add `### Compiler error` subsections) [VERIFIED: codebase]

| # | Section title in existing file | Status |
|---|-------------------------------|--------|
| 1 | `## Trait split: ChromosomeT + LinearChromosome` | Exists — needs `### Compiler error` added + LinearChromosome bound callout (D-03) |
| 2 | `## LinearChromosome: default() renamed to reset()` | Exists — needs `### Compiler error` added |
| 3 | `## Reporter removed — use GaObserver` | Exists — needs `### Compiler error` added |
| 4 | `## ChromosomeLength replaces genes_per_chromosome` | Exists — needs `### Compiler error` added |
| 5 | `## Flat stopping builders replace StoppingCriteria struct` | Exists — needs `### Compiler error` added |
| 6 | `## LimitConfiguration field removals` | Exists — needs `### Compiler error` added |
| 7 | `## GaConfiguration field access → accessor methods` | Exists — needs `### Compiler error` added |
| 8 | `## Logger setup (v2 auto-init → v3 explicit)` | Exists (from Phase 68) — needs `### Compiler error` added |

### Missing entries (3 new sections to ADD) [VERIFIED: codebase]

| # | New section required | Source of truth |
|---|---------------------|-----------------|
| 9 | `## DeGene → RealGene rename` | `src/traits/real_gene.rs` — trait now named `RealGene`, `real_value()` / `with_real_value()` / `bounds()` |
| 10 | `## SelectionOperator::select — new num_parents parameter` | `src/traits/operators.rs` lines 45–53 |
| 11 | `## Mutation enum variant parameter changes` | `src/operations.rs` lines 255–391 |

### Feature-flag sections (2 new `##` sections per D-02) [VERIFIED: codebase]

| Section | Content |
|---------|---------|
| `## parallel feature — rayon is now optional` | Phase 69 decision: `parallel = ["dep:rayon"]` (default-on); disable with `default-features = false`; logging about gate pattern |
| Already exists: `## Logger setup` covers logging | The `logging` feature is already covered inside the Logger setup section (Phase 68) — the CONTEXT asks for a **new** `##` section for `parallel` but the logger section already handles `logging`. Planner must add a separate `## parallel feature` section as its own top-level entry. |

---

## Breaking Change Detail — New Entries

### Entry 9: DeGene → RealGene rename

**What changed:** The `DeGene` trait was hard-renamed to `RealGene` in Phase 56. Relocated to `src/traits/real_gene.rs`. Import path changes from `genetic_algorithms::traits::DeGene` to `genetic_algorithms::traits::RealGene`. The trait interface is identical except for the name and two new methods: `bounds() -> Option<(f64, f64)>` with a default `None` implementation (non-breaking for existing impls). [VERIFIED: codebase — `src/traits/real_gene.rs`]

**Who is affected:** Anyone who implemented `DeGene` on a custom gene type for use with `DeEngine` or `ScatterEngine`.

**v2 Before:**
```rust
use genetic_algorithms::traits::DeGene;

impl DeGene for MyGene {
    fn real_value(&self) -> f64 { self.value }
    fn with_real_value(&self, v: f64) -> Self { MyGene { value: v } }
}
```

**v3 After:**
```rust
use genetic_algorithms::traits::RealGene;

impl RealGene for MyGene {
    fn real_value(&self) -> f64 { self.value }
    fn with_real_value(&self, v: f64) -> Self { MyGene { value: v } }
    // bounds() is optional — default returns None
}
```

**Compiler error (E0405 / E0412):**
```
error[E0412]: cannot find trait `DeGene` in module `genetic_algorithms::traits`
  --> src/main.rs:3:45
   |
3  | use genetic_algorithms::traits::DeGene;
   |                                 ^^^^^^ not found in `genetic_algorithms::traits`
   |
   = help: there is a trait with a similar name: `RealGene`
```

**Fix:** Global search-and-replace `DeGene` → `RealGene` and `use ... DeGene` → `use ... RealGene`.

---

### Entry 10: SelectionOperator::select — new num_parents parameter

**What changed:** In Phase 54 (N-ary selection), `SelectionOperator::select` gained a fourth parameter `num_parents: usize`. The return type changed from `Vec<(usize, usize)>` (pairs) to `Vec<Vec<usize>>` (groups of arbitrary size). Built-in selection strategies were updated. Custom implementors must update their signature. [VERIFIED: codebase — `src/traits/operators.rs` lines 45–53]

**Current v3 signature:**
```rust
fn select<U>(
    &self,
    chromosomes: &[U],
    number_of_couples: usize,
    number_of_threads: usize,
    num_parents: usize,
) -> Vec<Vec<usize>>
where
    U: ChromosomeT + Sync + Send + 'static + Clone;
```

**v2 Before (custom implementation):**
```rust
impl SelectionOperator for MySelection {
    fn select<U: ChromosomeT + Sync + Send + 'static + Clone>(
        &self,
        chromosomes: &[U],
        number_of_couples: usize,
        number_of_threads: usize,
    ) -> Vec<(usize, usize)> {
        vec![(0, 1), (2, 3)]
    }
}
```

**v3 After:**
```rust
impl SelectionOperator for MySelection {
    fn select<U: ChromosomeT + Sync + Send + 'static + Clone>(
        &self,
        chromosomes: &[U],
        number_of_couples: usize,
        number_of_threads: usize,
        num_parents: usize,   // new — number of parents per group
    ) -> Vec<Vec<usize>> {    // pairs become Vec<Vec<usize>>
        // For standard 2-parent crossover: return vec![vec![0, 1], vec![2, 3]]
        vec![vec![0, 1], vec![2, 3]]
    }
}
```

**Compiler error (E0053):**
```
error[E0053]: method `select` has an incompatible type for trait
  --> src/my_selection.rs:5:5
   |
5  |     fn select<U>(&self, chromosomes: &[U], number_of_couples: usize, number_of_threads: usize) -> Vec<(usize, usize)>
   |                                                                                                    ^^^^^^^^^^^^^^^^^^
   |                                                                       expected `fn(&Self, &[U], usize, usize, usize) -> Vec<Vec<usize>>`
   |                                                                            found `fn(&Self, &[U], usize, usize) -> Vec<(usize, usize)>`
```

**Fix:** Add `num_parents: usize` parameter; change return type to `Vec<Vec<usize>>`; wrap each pair `(a, b)` as `vec![a, b]`.

---

### Entry 11: Mutation enum variant parameter changes

**What changed:** Several `Mutation` variants were restructured between v2 and v3. The most impactful change is the renaming of the permutation-insert variant. In v2, `Mutation::Insertion` referred to the permutation-preserving operation (remove a gene, reinsert elsewhere). In v3, this variant is renamed to `Mutation::PermutationInsert`, and a new `Mutation::Insertion` is added for variable-length chromosome growth. [VERIFIED: codebase — `src/operations.rs` lines 255–391]

**Numeric-parameter variants** (`step`, `sigma`, `eta`, `b`, `f`, `scale`, `alpha`) moved from being passed via `mutation_step`/`mutation_sigma` builder fields to being inline struct fields on each variant. `Differential`, `Cauchy`, `LevyFlight`, `Uniform`, `SelfAdaptiveGaussian` are new v3 additions — not v2 regressions.

**v2 Before (permutation insertion):**
```rust
use genetic_algorithms::operations::Mutation;

.with_mutation_method(Mutation::Insertion)  // permutation move in v2
```

**v3 After:**
```rust
use genetic_algorithms::operations::Mutation;

.with_mutation_method(Mutation::PermutationInsert)  // renamed in v3
// Mutation::Insertion now means: insert a new gene (variable-length chromosomes only)
```

**v2 Before (sigma on Gaussian):**
```rust
.with_mutation_method(Mutation::Gaussian)
.with_mutation_sigma(0.5)   // sigma was a separate builder field in v2
```

**v3 After:**
```rust
.with_mutation_method(Mutation::Gaussian { sigma: Some(0.5) })  // inline
// .with_mutation_sigma() builder is removed
```

**Compiler error (E0599 for renamed variant):**
```
error[E0599]: no variant or associated item named `Insertion` found for enum `Mutation` in the current scope when used as a permutation operator
  --> src/main.rs:12:37
   |
12 |     .with_mutation_method(Mutation::Insertion)
   |                                     ^^^^^^^^^ variant not found in `Mutation` (for permutation use; `Insertion` now means variable-length growth)
   |
   = help: there is a variant with a similar name: `PermutationInsert`
```

**Compiler error (E0063 for missing struct fields):**
```
error[E0308]: mismatched types — expected struct variant `Mutation::Creep { step }`, found unit variant
  --> src/main.rs:14:37
   |
14 |     .with_mutation_method(Mutation::Creep)
   |                                     ^^^^^ help: use struct syntax: `Mutation::Creep { step: None }`
```

**Fix table:**

| v2 code | v3 replacement |
|---------|---------------|
| `Mutation::Insertion` (permutation move) | `Mutation::PermutationInsert` |
| `Mutation::Gaussian` + `.with_mutation_sigma(x)` | `Mutation::Gaussian { sigma: Some(x) }` |
| `Mutation::Creep` + `.with_mutation_step(x)` | `Mutation::Creep { step: Some(x) }` |
| `Mutation::Polynomial` + eta via config | `Mutation::Polynomial { eta: Some(x) }` |
| `Mutation::NonUniform` + b via config | `Mutation::NonUniform { b: Some(x) }` |
| `.with_mutation_step(x)` (standalone) | Remove — inline in variant |
| `.with_mutation_sigma(x)` (standalone) | Remove — inline in variant |

---

### Feature-flag Section: `parallel` feature

**What changed (Phase 69):** A new `parallel` feature (default-on) gates the `rayon` dependency. Users who previously had `rayon` as a mandatory transitive dependency can now opt out with `default-features = false`. [VERIFIED: codebase — `Cargo.toml` lines 33–35]

**Cargo.toml change for users who want to disable parallel:**
```toml
[dependencies]
genetic_algorithms = { version = "3", default-features = false, features = ["logging"] }
```

**No API change:** All parallel behavior is still the default. Code that does not call rayon APIs directly is unaffected.

**Who is affected:** Users doing embedded / WASM-only builds who previously had to carry rayon unnecessarily. Standard users: no action required.

---

## CHANGELOG.md Audit — Current State [VERIFIED: codebase]

### What exists

- `## [Unreleased]` section (empty Added/Changed/Removed headers) — MUST BE DROPPED per D-05
- `## [3.0.0] - Unreleased` — comprehensive; covers phases 47–69 including build-perf phases 66–69; all Added/Changed/Removed/Architecture & quality buckets populated
- Compare link at bottom: `[3.0.0]: https://github.com/leimbernon/rust_genetic_algorithms/compare/2.4.0...HEAD`

### What must change

1. Drop `## [Unreleased]` section (empty — no entries)
2. Change `## [3.0.0] - Unreleased` → `## [3.0.0] - 2026-06-17`
3. Update compare link: `2.4.0...HEAD` → `2.4.0...v3.0.0` (only after the v3.0.0 git tag is cut — if the tag does not exist during Plan 65-02, leave as `...HEAD`)
4. Verify Phase 64 (Test & Doc Quality) has an entry — it is currently absent from the `## [3.0.0]` section
5. Verify Phase 65 (this phase) has an entry ("Migration guide and release notes finalized") — self-reference

### Coverage gap found

The `## [3.0.0]` section covers phases 47–69 but does not mention Phase 64 (Test & Doc Quality — coverage baseline + rustdoc examples). The planner must add a brief Architecture & quality note for Phase 64 work already completed (Plan 64-01: coverage CI gate established). [VERIFIED: codebase — reviewed CHANGELOG.md lines 25–103]

---

## README.md Audit [VERIFIED: codebase]

The README.md already has the upgrade banner at line 4:
```markdown
> **v3.0.0 users:** see [MIGRATION.md](./MIGRATION.md) for the full list of breaking changes and migration recipes.
```

This satisfies Success Criterion 3. **No README changes required for Plans 65-01/65-02.** Plan 65-03 should verify the link is live.

---

## Cargo.toml Audit for `cargo publish --dry-run` [VERIFIED: codebase]

| Field | Value | Status |
|-------|-------|--------|
| `name` | `genetic_algorithms` | OK |
| `version` | `3.0.0` | OK |
| `description` | `"Library for solving genetic algorithm problems"` | OK |
| `readme` | `README.md` | OK |
| `license` | `Apache-2.0` | OK |
| `repository` | set | OK |
| `documentation` | set | OK |
| `keywords` | 5 entries | OK |
| `categories` | 4 entries | OK |
| `include` | lists `/src`, `/examples`, `/tests`, `/benches`, `/docs`, `MIGRATION.md`, `README.md`, `CHANGELOG.md`, `LICENSE`, `CONTRIBUTING.md`, `SECURITY.md`, `build.rs` | VERIFY: `MIGRATION.md` is present (it is, line 29) |
| `rust-version` | `"1.81.0"` | OK |

**Potential dry-run issue:** `build.rs` is listed in `include` but may not exist. Plan 65-03 must check `ls build.rs` before the dry-run step.

---

## Compiler Error Reference — All 10 Breaking Changes

The `### Compiler error` subsection for each existing MIGRATION.md entry must be written using this format. Below are the expected error codes for each entry. [ASSUMED — error codes inferred from rustc behavior, must be verified by running rustc against actual v2 patterns in Plan 65-03]

| Breaking change | Expected error | Key message |
|-----------------|----------------|-------------|
| ChromosomeT split — missing LinearChromosome impl | `E0277` | `the trait bound `MyChromosome: LinearChromosome` is not satisfied` |
| LinearChromosome bound callout | `E0277` | (same as above — operator requires LinearChromosome) |
| `default()` → `reset()` | `E0599` | `no method named `default` found for type `MyChromosome` in the current scope` (note: `Default::default()` still works; only the old `LinearChromosome::default()` is gone) |
| Reporter removed | `E0432` / `E0412` | `unresolved import `genetic_algorithms::reporter`; `cannot find trait `Reporter` in module ...`  |
| ChromosomeLength / genes_per_chromosome | `E0599` | `no method named `with_genes_per_chromosome` found for struct `Ga`...` |
| StoppingCriteria flattened | `E0412` | `cannot find struct `StoppingCriteria` in module `genetic_algorithms::configuration`...` |
| LimitConfiguration field removals | `E0559` | `variant `LimitConfiguration` has no field named `needs_unique_ids`` |
| GaConfiguration fields pub(crate) | `E0616` | `field `limit_configuration` of struct `GaConfiguration` is private` |
| Logger / LogLevel | `E0412` / `E0599` | `cannot find enum `LogLevel` in module `genetic_algorithms::configuration`` |
| DeGene → RealGene | `E0412` | `cannot find trait `DeGene` in module `genetic_algorithms::traits`` |
| SelectionOperator::select signature | `E0053` | `method `select` has an incompatible type for trait` |
| Mutation::Insertion renamed | `E0599` | `help: there is a variant with a similar name: `PermutationInsert`` |

---

## v2 Sample Crate Smoke-Test — Design (Plan 65-03)

Per D-07 and D-08, the smoke-test creates a minimal crate outside the repo to exercise the top 3 breaking patterns. [ASSUMED — structure inferred from CONTEXT.md decisions]

**Location:** `/tmp/ga_v2_smoke/` (throwaway, deleted after test)

**Top 3 patterns to test:**
1. `ChromosomeT + LinearChromosome` impl split — custom struct implementing both
2. `Reporter` removal — code that used `with_reporter(Box::new(SimpleReporter::new(10)))`
3. `SelectionOperator` trait impl — custom selection with old `Vec<(usize, usize)>` return

**Test sequence:**
1. Create crate: `cargo new --lib /tmp/ga_v2_smoke`
2. Set `genetic_algorithms = "2.4.0"` in `[dependencies]`
3. Write `src/lib.rs` with the three v2 patterns
4. Run `cargo build` — confirm it fails with expected v2-to-v3 errors
5. Update `genetic_algorithms = "3.0.0"` and apply MIGRATION.md fixes
6. Run `cargo build` again — confirm clean compile
7. Delete `/tmp/ga_v2_smoke`

**Note:** Step 4 requires v2.4.0 to still be on crates.io. If v3.0.0 has not yet been published, this works cleanly. If v3.0.0 is published, the "v2 broken" step should use `version = "=2.4.0"` (exact pin).

---

## Release Gate Sequence (Plan 65-03) — Four-Part

Per D-07, the exact commands are: [VERIFIED: codebase — `Cargo.toml` features and CI workflow names]

### Part 1: Full CI Matrix

```bash
# Run from repo root
cargo test
cargo test --features serde
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps --all-features 2>&1 | grep -c "warning:" # must be 0
cargo check --target wasm32-unknown-unknown
```

**Feature matrix note:** The CI feature-matrix workflow (`.github/workflows/feature-matrix.yml`) covers `default`, `serde`, `visualization`, `benchmarks`, `observer-tracing`, `observer-metrics`, `all-features`, `wasm32`. Plan 65-03 should trigger CI and verify it is green rather than re-running locally.

### Part 2: `cargo publish --dry-run`

```bash
cargo publish --dry-run
```

**Pre-check:** Verify `build.rs` exists if listed in `include` (see Cargo.toml audit above).

**Expected output:** `Packaging genetic_algorithms v3.0.0 ...` with no errors. May warn about unpublished local deps — acceptable for dry-run.

### Part 3: v2 Smoke-Test Crate

See design above. The crate validates that MIGRATION.md is accurate before any user has to discover a wrong migration recipe.

### Part 4: `cargo run --example` Smoke Run

```bash
# All examples registered in Cargo.toml
# Examples requiring features:
cargo run --example cma_es_rastrigin --features logging -- --generations 5
cargo run --example pso_rastrigin --features logging -- --generations 5
cargo run --example eda_trap --features logging -- --generations 5
cargo run --example ipop_rastrigin --features logging -- --generations 5
cargo run --example knapsack_binary --features logging -- --generations 5
cargo run --example job_scheduling --features logging -- --generations 5
cargo run --example onemax_extension --features logging -- --generations 5
cargo run --example island_model --features logging -- --generations 5
cargo run --example nsga2_zdt1 --features logging -- --generations 5
cargo run --example feature_selection --features logging -- --generations 5
cargo run --example rastrigin --features logging -- --generations 5
cargo run --example nsga3_dtlz2 --features logging -- --generations 5
cargo run --example nqueens_range --features logging -- --generations 5
cargo run --example niching --features logging -- --generations 5
cargo run --example onemax_binary --features logging -- --generations 5
cargo run --example spea2_zdt1 --features logging -- --generations 5
cargo run --example moead_dtlz2 --features logging -- --generations 5
cargo run --example sms_emoa_zdt1 --features "benchmarks logging" -- --generations 5
cargo run --example ibea_zdt1 --features "benchmarks logging" -- --generations 5
cargo run --example surrogate_rastrigin -- --generations 5
cargo run --example memetic_rastrigin -- --generations 5
```

**Note:** Not all examples accept `--generations` as a flag. The CI `examples-smoke.yml` already handles this — Plan 65-03 should re-run that workflow rather than reimplementing the flag detection. [ASSUMED — actual flag support per example may vary; verify against CI workflow]

---

## Architecture Patterns

### MIGRATION.md Structure Pattern [VERIFIED: codebase]

The established pattern in the existing `MIGRATION.md`:

```markdown
## [Section title describing the change]

**What changed (D-XX):** [1–3 sentence description]

**Who is affected:** [who needs to act]

### Before

\`\`\`rust
// v2 code
\`\`\`

### After

\`\`\`rust
// v3 code
\`\`\`

### Compiler error

\`\`\`
error[EXXXX]: message
  --> src/file.rs:line:col
   |
N  |     offending code
   |     ^^^^^^^^^^^^^  explanation
\`\`\`

**Fix:** [one-sentence prescription]
```

The `### Compiler error` subsection is NEW to all existing entries. The planner must insert it after `### After` in each of the 7 existing sections, and include it in all 3 new sections.

---

## Common Pitfalls

### Pitfall 1: Compiler errors don't match rustc output exactly
**What goes wrong:** The `### Compiler error` blocks are written from memory rather than actual rustc output, producing error messages that differ from what users actually see.
**Why it happens:** Rustc error messages evolve across versions; exact formatting (span markers, suggestion text) is version-specific.
**How to avoid:** Plan 65-03 runs the v2 smoke-test crate and captures exact error output with `cargo build 2>&1`. Copy verbatim into MIGRATION.md.
**Warning signs:** Error code is right but message text or suggestion differs.

### Pitfall 2: Mutation::Insertion ambiguity in migration text
**What goes wrong:** The migration text says "rename `Insertion` to `PermutationInsert`" but a reader who upgraded and added variable-length support gets confused because `Insertion` still exists in v3.
**Why it happens:** The variant was not deleted — it was repurposed. Its v2 meaning moved to `PermutationInsert`.
**How to avoid:** The migration entry must explicitly state: "`Mutation::Insertion` in v3 has a DIFFERENT meaning (variable-length chromosome growth). If you were using `Mutation::Insertion` for permutation moves, rename to `Mutation::PermutationInsert`."

### Pitfall 3: CHANGELOG compare link left as `...HEAD` after tagging
**What goes wrong:** After the v3.0.0 git tag is cut and pushed, the compare link still shows `2.4.0...HEAD`, which resolves correctly on GitHub but looks unprofessional for a release.
**Why it happens:** Plan 65-02 is written before the tag is cut; the link is deferred.
**How to avoid:** Include a note in Plan 65-03 to update the link to `2.4.0...v3.0.0` if the tag has been created. Make it a checklist item.

### Pitfall 4: `build.rs` listed in Cargo.toml `include` but may not exist
**What goes wrong:** `cargo publish --dry-run` errors with "file `build.rs` included in `include` does not exist."
**Why it happens:** Cargo.toml line 26 lists `"/build.rs"` in the `include` array; whether this file exists is unclear from the file audit.
**How to avoid:** Plan 65-03 first step: `ls /path/to/build.rs && echo "exists" || echo "MISSING — remove from include"`. If missing, Plan 65-03 must edit `Cargo.toml` to remove it before the dry-run.

### Pitfall 5: v2 smoke-test crate requires internet access
**What goes wrong:** Creating the test crate and fetching v2.4.0 from crates.io fails in air-gapped CI.
**Why it happens:** The smoke-test downloads `genetic_algorithms = "2.4.0"` from the registry.
**How to avoid:** Run the smoke-test locally, not in CI. Document in Plan 65-03 that this is a human-run step, not a CI gate.

---

## State of the Art

| Aspect | v2 state | v3 state | Notes |
|--------|----------|----------|-------|
| `ChromosomeT` | All-in-one trait (fitness + DNA) | Minimal core; `LinearChromosome` supertrait for DNA | [VERIFIED: codebase] |
| `DeGene` trait | `DeGene` in `src/traits/` | `RealGene` in `src/traits/real_gene.rs` | [VERIFIED: codebase] |
| `SelectionOperator::select` | Returns `Vec<(usize, usize)>`, 3 params | Returns `Vec<Vec<usize>>`, 4 params (adds `num_parents`) | [VERIFIED: codebase] |
| `Mutation::Insertion` | Permutation move (remove + reinsert) | Variable-length chromosome growth | [VERIFIED: codebase] |
| `Mutation::PermutationInsert` | Did not exist | Permutation move (formerly `Insertion`) | [VERIFIED: codebase] |
| Mutation params (step/sigma/etc.) | Separate builder fields | Inline struct fields on each variant | [VERIFIED: codebase] |
| `env_logger` | Auto-installed in `Ga::run()` | Dev-dependency only; user installs subscriber | [VERIFIED: codebase] |
| `rayon` | Mandatory dep | Optional behind `parallel` feature (default-on) | [VERIFIED: codebase] |
| `log` | Mandatory dep | Optional behind `logging` feature (default-on) | [VERIFIED: codebase] |
| CHANGELOG `[3.0.0]` | Not yet dated | Needs `2026-06-17` and link update | [VERIFIED: codebase] |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Compiler error codes and message text for existing 7 breaking changes | Compiler Error Reference table | Low — will be corrected by Plan 65-03 smoke-test |
| A2 | `--generations 5` flag is accepted by most examples | Release Gate Part 4 | Medium — some examples may not accept this flag; use CI workflow instead |
| A3 | v3.0.0 is not yet tagged on GitHub when Plan 65-02 runs | CHANGELOG compare link section | Low — if tagged, planner updates link; if not, leaves as HEAD |
| A4 | `build.rs` referenced in Cargo.toml `include` may not exist | Cargo.toml audit | Medium — causes `cargo publish --dry-run` failure; Plan 65-03 checks first |

---

## Open Questions (RESOLVED)

1. **Does `build.rs` exist?**
   - What we know: `Cargo.toml` line 26 includes `"/build.rs"` in the `include` array
   - What's unclear: Whether the file actually exists in the workspace root
   - **RESOLVED:** `build.rs` exists at repo root (verified: 231 bytes). Plan 65-03 Task 1 also runs `ls build.rs` as a defensive pre-flight check.

2. **Are compiler error messages exactly right for the 7 existing entries?**
   - What we know: The error codes are correct (E0277, E0412, E0599, E0053, etc.)
   - What's unclear: Exact rustc message text (rustc 1.81.0 specific wording)
   - **RESOLVED:** Deferred by design. Plan 65-01 authors the entries using RESEARCH-inferred error codes; Plan 65-03 Task 2 builds the v2 smoke-test crate and reconciles any wording differences against real rustc output, applying fixes to MIGRATION.md in-plan.

3. **Has the `[Unreleased]` section's empty content been added since this audit?**
   - What we know: At time of research (2026-06-17) it has only empty sub-headers
   - What's unclear: Whether Phase 69 completion added any unreleased entries
   - **RESOLVED:** Plan 65-02 reads CHANGELOG.md fresh via `<read_first>` before editing. Any new content in `[Unreleased]` will be merged into `[3.0.0]` by the executor following D-05.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | All CI gates | ✓ | (project uses) | — |
| `rustc` | Compiler error capture | ✓ | 1.81.0+ | — |
| `wasm32-unknown-unknown` target | WASM check | [ASSUMED] | — | `rustup target add wasm32-unknown-unknown` |
| crates.io network | v2 smoke-test | ✓ (local) | — | Skip smoke-test in CI |
| `cargo-llvm-cov` | Coverage gate (Plan 64-01) | [ASSUMED] | — | Skip in Plan 65-03 |

---

## Project Constraints (from CLAUDE.md)

- **No breaking changes without this migration guide** — Phase 65 is the resolution of all v3.0.0 breaking changes
- **WASM compatibility mandatory** — `cargo check --target wasm32-unknown-unknown` is a required gate in Plan 65-03
- **Signed commits mandatory** — every commit in Plans 65-01/02/03 must be GPG-signed
- **Tests in `tests/` folder** — no inline tests (not applicable to this phase — documentation only)
- **Branch naming** — work on `feat/65-...` branch targeting the milestone branch, not `main`
- **No direct milestone push** — Plans 65-01, 65-02, 65-03 each go through a PR
- **Zero rustdoc warnings** — `cargo doc --no-deps` must stay green after MIGRATION.md changes (MIGRATION.md is not compiled, so this constraint applies to any code examples added via `rustdoc test` or `# Examples` blocks)

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` + `cargo nextest` (CI) |
| Config file | `Cargo.toml` (harness = false for benches) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Behavior | Test Type | Automated Command |
|----------|-----------|-------------------|
| MIGRATION.md covers all 10 breaking changes | Manual checklist | Human review in Plan 65-01 |
| Every entry has `### Compiler error` | Manual checklist | Human review in Plan 65-01 |
| README upgrade banner links correctly | Spot check | `grep -n "MIGRATION.md" README.md` |
| CHANGELOG `[3.0.0]` entry dated and complete | Manual checklist | Human review in Plan 65-02 |
| `cargo publish --dry-run` passes | Shell gate | `cargo publish --dry-run` (Plan 65-03) |
| v2 sample crate compiles after migration | Shell gate | `cargo build` in `/tmp/ga_v2_smoke` |
| All CI gates pass | CI pipeline | Push to PR; CI green |

### Wave 0 Gaps

None — existing test infrastructure covers all phase requirements. Plans 65-01 and 65-02 are documentation-only; Plan 65-03 is a verification script.

---

## Sources

### Primary (HIGH confidence)
- `MIGRATION.md` (codebase) — existing 7+1 breaking-change entries, established style
- `CHANGELOG.md` (codebase) — `## [3.0.0]` section current state; `## [Unreleased]` empty state
- `src/traits/operators.rs` (codebase) — `SelectionOperator::select` exact current signature
- `src/traits/real_gene.rs` (codebase) — `RealGene` trait methods
- `src/operations.rs` lines 255–391 (codebase) — complete `Mutation` enum variants and their fields
- `Cargo.toml` (codebase) — `include`, features, version, metadata fields
- `README.md` (codebase) — upgrade banner at line 4
- `65-CONTEXT.md` (planning) — locked decisions D-01 through D-08
- `68-CONTEXT.md` / `69-CONTEXT.md` (planning) — `logging` and `parallel` feature decisions

### Secondary (MEDIUM confidence)
- `ROADMAP.md` Phase 65 section — success criteria list (10 breaking changes canonical)
- `STATE.md` — confirmed v3.0.0 is current milestone; Phase 65 status is "context gathered"

### Tertiary (LOW confidence / ASSUMED)
- Compiler error codes in the `### Compiler error` subsection for existing 7 entries — inferred from rustc behavior; must be verified by running actual compiler

---

## Metadata

**Confidence breakdown:**
- MIGRATION.md content audit: HIGH — full file read; exact current state known
- CHANGELOG.md content audit: HIGH — full file read; gaps identified
- New entry content (DeGene, SelectionOperator, Mutation): HIGH — verified against actual source files
- Compiler error text: MEDIUM — codes correct; message wording assumed from rustc behavior
- Release gate commands: HIGH — verified against Cargo.toml features and CI workflow names

**Research date:** 2026-06-17
**Valid until:** This research does not expire — all sources are the codebase itself, which does not change until Phase 65 implementation begins.
