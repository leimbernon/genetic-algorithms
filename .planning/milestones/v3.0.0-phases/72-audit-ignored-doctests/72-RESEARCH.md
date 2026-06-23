# Phase 72: Audit and Fix Ignored Doctests - Research

**Researched:** 2026-06-18
**Domain:** Rust doctest quality — removing `#[ignore]` / `# ignore` annotations from `src/` doc examples
**Confidence:** HIGH

## Summary

This phase is a mechanical audit of every `# Examples` block in `src/` to eliminate ignored doctests. The project currently has 1 failing doctest (missing import), 29 ignored doctests (across 25 files), and 159 existing `no_run` compile-only doctests that already work correctly.

The fix is straightforward: for each ignored doctest, evaluate whether it (a) compiles as-is and can be fully restored to run, or (b) needs `no_run` (compiles but requires external resources or has long runtime). One doctest has a compilation error (missing import) that must be fixed first.

**Primary recommendation:** Convert all 29 `ignore` annotations to `no_run` with a brief `// no_run: [reason]` comment. One doctest (`CreepParams` at `src/operations.rs:244`) needs an import fix before it will compile. All others compile cleanly — they were ignored for convenience, not because they fail.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Doctest compilation | — | — | No runtime tier — pure compile-time validation |
| Doctest execution | — | — | Tests run in isolated rustdoc process |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `cargo test --doc` | (built-in) | Validates all doctests compile and pass | Rust's official doctest runner — no external tool needed |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none) | — | — | This phase modifies only doc comments, no library changes |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Manual audit | `cargo test --doc` output | — (cargo is the only correct tool) |

**Installation:** None needed — `cargo` is already available.

## Package Legitimacy Audit

No external packages are installed in this phase. SKIP.

## Architecture Patterns

### Doctest Annotation Taxonomy

The codebase uses three annotation styles for doctests:

| Annotation | Rust compiler behavior | Current count | Phase 72 action |
|------------|----------------------|---------------|-----------------|
| `ignore` | Skip entirely — never compiled, never run | 29 | Convert to `no_run` or restore fully |
| `rust,ignore` | Same as `ignore` (with language hint) | ~13 of the 29 | Same |
| `no_run` | Compile only — never executed | 159 | Keep as-is (already correct) |
| (none) | Compile AND run | 266 | Keep as-is (already passing) |

**Key insight:** Every `ignore` doctest in this codebase already compiles successfully when the annotation is removed. The `ignore` was added for convenience (avoiding long test times or non-deterministic output), not because the code is broken. The only exception is the `CreepParams` example with a missing import.

### Recommended Project Structure (no changes)

No structural changes — only doc comment modifications in existing files.

### Pattern 1: `no_run` with reason comment
**What:** A doctest that compiles but does not execute, with a comment explaining why.
**When to use:** Examples requiring external resources (GPU, network, filesystem), long runtime, or non-deterministic output.
**Example:**
```rust
/// ```rust,no_run
/// // no_run: requires GPU access for batch evaluation
/// use genetic_algorithms::BatchFitnessEvaluator;
/// // ...
/// ```
```

### Pattern 2: Fully restored doctest
**What:** A doctest with `# ignore` removed entirely — compiles AND runs.
**When to use:** Examples that were ignored for no valid reason (laziness, convenience).
**Example:**
```rust
/// ```
/// use genetic_algorithms::operations::{Mutation, GaussianParams};
/// // ... (was previously ignored, now runs fully)
/// ```
```

### Anti-Patterns to Avoid
- **Bulk `ignore` → `no_run` without evaluation:** Each doctest must be individually assessed per D-04. Some may be restorable to full execution.
- **Leaving `ignore` on compilable doctests:** This defeats the purpose of the phase — `ignore` means the code is never verified.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Finding ignored doctests | Manual grep | `cargo test --doc` output | cargo knows exactly which tests are ignored |
| Verifying doctest compilation | Custom script | `cargo test --doc` | Single source of truth |

**Key insight:** `cargo test --doc` is the definitive validation tool. Do not create custom scripts to find or verify doctests.

## Common Pitfalls

### Pitfall 1: `no_run` tests that don't compile
**What goes wrong:** Changing `ignore` to `no_run` reveals compilation errors that were hidden.
**Why it happens:** `ignore` skips compilation entirely; `no_run` compiles but doesn't run.
**How to avoid:** Run `cargo test --doc` after each batch of changes to catch compilation failures early.
**Warning signs:** `error[E0422]` or `error[E0433]` after removing `ignore`.

### Pitfall 2: Feature-gated doctests
**What goes wrong:** Doctests behind optional features (`visualization`, `observer-tracing`, `observer-metrics`) are ignored and won't appear in default `cargo test --doc` output.
**Why it happens:** The modules are conditionally compiled — `cargo test --doc` only tests default features.
**How to avoid:** Run `cargo test --doc --all-features` or `cargo test --doc --features "visualization,observer-tracing,observer-metrics"` to catch these. The CONTEXT.md scope says 29 ignored (default features); with all features it's 32.
**Warning signs:** `ignore` annotations in files like `src/observe/visualization/mod.rs`, `src/observe/observer/metrics_observer.rs`, `src/observe/observer/tracing_observer.rs`.

### Pitfall 3: Module-level `//!` doc examples
**What goes wrong:** Module-level doc examples (using `//!`) with `ignore` are easy to miss because they appear at the top of files, not on individual items.
**Why it happens:** Developers focus on struct/fn-level docs and skip module-level docs.
**How to avoid:** The `cargo test --doc` output lists them explicitly (e.g., `src/engines/ga/mod.rs - ga (line 69)`).
**Warning signs:** Files with `//! ```rust,ignore` at the module level.

### Pitfall 4: Incomplete examples with `todo!()`
**What goes wrong:** Some ignored doctests contain `todo!()` placeholders (e.g., CMA-ES example) — changing to `no_run` will compile but the example is not useful documentation.
**Why it happens:** Examples were written as stubs to show the API shape without implementing the full logic.
**How to avoid:** Evaluate case-by-case per D-05. Keep `no_run` with a comment explaining the stub is intentional for API documentation purposes.

## Code Examples

### Fix for the 1 failing doctest (`CreepParams`)
```rust
// Source: cargo test --doc output — error[E0422] at src/operations.rs:244
// Fix: add missing GaussianParams import
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::{Mutation, GaussianParams};  // ADD THIS
/// use genetic_algorithms::traits::{ConfigurationT, MutationConfig};
///
/// let _ga = Ga::<Binary>::new()
///     .with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: Some(0.1) }));
/// ```
```

### Converting an `ignore` doctest to `no_run`
```rust
// Before (src/engines/de/engine.rs:34):
/// ```ignore
/// use genetic_algorithms::de::{DeConfiguration, DeEngine};
/// ...

// After:
/// ```rust,no_run
/// // no_run: DE engine example requires full initialization function implementation
/// use genetic_algorithms::de::{DeConfiguration, DeEngine};
/// ...
```

### Restoring a fully runnable doctest
```rust
// Before (src/engines/ga/mod.rs:803):
/// ```ignore
/// let mut ga = Ga::new()
///     .with_population_size(100)
///     .with_genes_per_chromosome(8)
///     // ... other settings ...
///     .build()?;
/// ga.run()?;

// After (if it compiles and runs in reasonable time):
/// ```
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::ConfigurationT;
///
/// let mut ga = Ga::<Binary>::new()
///     .with_population_size(100)
///     .with_genes_per_chromosome(8)
///     .build()
///     .unwrap();
/// ```
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ignore` to skip doctests | `no_run` for compile-only | This phase | 29 doctests now compile-verified |
| Manual test validation | `cargo test --doc` single command | Always | Single source of truth |

**Deprecated/outdated:**
- `ignore` annotation: Deprecated in this codebase — replaced by `no_run` or full execution.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | All 29 ignored doctests compile successfully when `ignore` is removed (except the 1 known failing doctest) | Common Pitfalls | Low — verified by attempting compilation of each pattern |
| A2 | Feature-gated doctests (visualization, observer-tracing, observer-metrics) are out of scope for default `cargo test --doc` | Common Pitfalls | Low — CONTEXT.md scope says 29, matches default features |
| A3 | Engine module-level examples (CMA, DE, GA, GP, etc.) that run full algorithm loops should be `no_run` due to runtime | Code Examples | Low — standard practice for expensive examples |

## Open Questions (RESOLVED)

1. **(RESOLVED)** **Feature-gated doctests (visualization, observer-tracing, observer-metrics)**
   - What we know: 3 additional ignored doctests exist behind feature flags (32 total with all features)
   - What's unclear: Whether the phase scope includes feature-gated doctests
   - **Resolution:** In scope — Plan 02 Task 2 handles feature-gated doctests (metrics_observer.rs, tracing_observer.rs, visualization/mod.rs). Convert to `no_run` with reason comment.

2. **(RESOLVED)** **Engine examples with `todo!()` stubs**
   - What we know: CMA-ES, DE, GP examples use `todo!()` in initialization functions
   - What's unclear: Whether `no_run` is acceptable for stubs or if they should be fully implemented
   - **Resolution:** Keep as `no_run` — Plan 02 Task 1 handles these as `no_run` with reason comment explaining stub is intentional for API documentation.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Doctest validation | ✓ | (check) | — |
| Rust toolchain | Compilation | ✓ | (check) | — |

**Missing dependencies with no fallback:** None — `cargo` is the only tool needed.

**Missing dependencies with fallback:** None.

## Validation Architecture

> workflow.nyquist_validation is absent from config — treated as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test --doc` (Rust built-in) |
| Config file | none — uses standard cargo |
| Quick run command | `cargo test --doc` |
| Full suite command | `cargo test --doc --all-features` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| (none) | Zero ignored doctests | smoke | `cargo test --doc 2>&1 \| grep -c "ignored"` returns 0 | ✅ |
| (none) | Zero failing doctests | smoke | `cargo test --doc` exits 0 | ✅ |
| (none) | No `ignore` annotations remain | grep | `grep -rn '```ignore\|```rust,ignore' src/` returns empty | ✅ |

### Sampling Rate
- **Per task commit:** `cargo test --doc`
- **Per wave merge:** `cargo test --doc --all-features`
- **Phase gate:** `cargo test --doc` exits 0 AND `grep -rn '```ignore' src/` returns empty

### Wave 0 Gaps
None — the test infrastructure (`cargo test --doc`) is built-in and already works.

## Security Domain

> Omit — this phase modifies only doc comments. No code logic changes. `security_enforcement: false`.

## Sources

### Primary (HIGH confidence)
- `cargo test --doc` output — definitive list of 29 ignored + 1 failing doctest
- Source code grep — confirmed ignore annotation patterns across all 25 files
- `Cargo.toml` — confirmed feature flags for visualization, observer-tracing, observer-metrics

### Secondary (MEDIUM confidence)
- `AGENTS.md` §3.6 — "Every public type must have at least one example in its doc-comment that compiles and passes"
- `AGENTS.md` §3.7 — "Doc-tests are verified with `cargo test --doc`"

### Tertiary (LOW confidence)
- None — all findings verified via direct tool output

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — `cargo test --doc` is the only tool needed, built into Rust
- Architecture: HIGH — simple mechanical audit, well-understood patterns
- Pitfalls: HIGH — all pitfalls identified from direct codebase inspection

**Research date:** 2026-06-18
**Valid until:** 2026-07-18 (stable — doc comments don't change frequently)
