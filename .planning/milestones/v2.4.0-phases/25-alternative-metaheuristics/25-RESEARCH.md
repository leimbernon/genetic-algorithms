# Phase 25: Directory Restructure - Research

**Researched:** 2026-04-26
**Domain:** Rust module system — file moves, `pub use` re-exports, feature-gated modules
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

> Note: CONTEXT.md (25-CONTEXT.md) was created for a prior scope (DE/Scatter/Cellular/ALPS
> implementation). Only integration and compatibility decisions carry over to this restructure phase.

### Locked Decisions (compatibility decisions from prior context)
- Each engine should be a standalone module following the library's architectural patterns
- Integration with existing traits and interfaces; reuse existing configuration patterns
- Maintain compatibility with existing observer and reporting systems

### Claude's Discretion
- Internal architecture decisions for the restructure (file layout within new groups)
- Sequencing of individual file moves within the overall plan

### Deferred Ideas (OUT OF SCOPE for Phase 25)
- DE, Scatter Search, Cellular GA, ALPS implementation — all deferred to Phases 26-29
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| STRUCT-01 | `src/engines/` groups all engine modules (ga, island, nsga2, de, scatter, cellular, alps); lib.rs re-exports preserve all existing public paths | Section: Module Move Map — Engines group |
| STRUCT-02 | `src/types/` groups chromosomes and genotypes modules; lib.rs re-exports preserve all existing public paths | Section: Module Move Map — Types group |
| STRUCT-03 | `src/observe/` groups observer, reporter, visualization, and checkpoint modules; lib.rs re-exports preserve all existing public paths | Section: Module Move Map — Observe group |
| STRUCT-04 | All existing tests pass after restructure (`cargo test`, `cargo test --features serde`, `cargo clippy`, zero rustdoc warnings) | Section: Risk Analysis and Code Path Audit |
</phase_requirements>

---

## Summary

Phase 25 is a pure mechanical restructure of the `src/` directory into three logical groups. No public-facing API changes. No new features. The only deliverable is a reorganized file tree with `pub mod` and `pub use` re-exports in lib.rs that preserve every existing path identically.

The restructure is non-breaking by design: all existing `use genetic_algorithms::foo::bar` paths in tests, examples, and benches continue to resolve without modification. This is achieved through re-export modules in lib.rs — `pub mod chromosomes { pub use super::types::chromosomes::*; }` style aliases. Internal `crate::` paths inside the library source files do NOT need to change at all: Rust resolves `crate::` against the crate root (lib.rs), so as long as lib.rs re-exports the top-level module names, all internal cross-references remain valid.

The primary risk is incomplete re-export coverage — if any public item from a moved module is not re-exported under the original path, downstream users get a compile error. The mitigation is a mechanical audit of every `pub` item in every moved module before the restructure is considered done.

**Primary recommendation:** Move files first, wire up new `src/engines/mod.rs`, `src/types/mod.rs`, `src/observe/mod.rs`, then add thin compatibility shims in lib.rs. Run `cargo test --features serde` and `cargo doc --no-deps` as the gate for each group.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Directory layout (file moves) | Source code | — | Pure filesystem + Rust module system operation |
| Public path preservation | lib.rs re-exports | Group mod.rs files | lib.rs is the crate root; all public paths flow through it |
| Internal `crate::` paths | No change needed | — | `crate::` resolves from lib.rs; shims mean existing paths still work |
| Feature-gated module exposure | lib.rs `#[cfg]` annotations | — | `checkpoint`, `visualization`, `observer-tracing`, `observer-metrics` must keep their `#[cfg]` gates |
| Placeholder dirs for future engines | `src/engines/` | — | Empty `mod.rs` with doc comment is sufficient; not `pub` in lib.rs until the engine ships |

---

## Current src/ Layout (VERIFIED)

Exhaustive listing of every module currently at the crate root level.

### Flat modules (single `.rs` files)

| File | Exported in lib.rs as | Feature gate |
|------|-----------------------|-------------|
| `src/checkpoint.rs` | `pub mod checkpoint` | `#[cfg(feature = "serde")]` |
| `src/chromosomes.rs` + `src/chromosomes/` | `pub mod chromosomes` | none |
| `src/configuration.rs` | `pub mod configuration` | none |
| `src/error.rs` | `pub mod error` | none |
| `src/extension/` | `pub mod extension` | none |
| `src/fitness.rs` + `src/fitness/` | `pub mod fitness` | none |
| `src/ga.rs` | `pub mod ga` | none |
| `src/genotypes.rs` + `src/genotypes/` | `pub mod genotypes` | none |
| `src/initializers.rs` + `src/initializers/` | `pub mod initializers` | none |
| `src/island/` | `pub mod island` | none |
| `src/niching/` | `pub mod niching` | none |
| `src/nsga2/` | `pub mod nsga2` | none |
| `src/observer/` | `pub mod observer` | none |
| `src/operations.rs` + `src/operations/` | `pub mod operations` | none |
| `src/population.rs` | `pub mod population` | none |
| `src/reporter/` | `pub mod reporter` | none |
| `src/rng.rs` | `pub mod rng` | none |
| `src/stats.rs` | `pub mod stats` | none |
| `src/traits.rs` + `src/traits/` | `pub mod traits` | none |
| `src/validators.rs` + `src/validators/` | `pub mod validators` | none |
| `src/visualization/` | `pub mod visualization` | `#[cfg(feature = "visualization")]` |

### Top-level re-exports in lib.rs (must be preserved exactly)

```rust
pub use ga::TerminationCause;
pub use observer::AllObserver;
pub use observer::CompositeObserver;
pub use observer::ExtensionEvent;
pub use observer::GaObserver;
pub use observer::IslandGaObserver;
pub use observer::LogObserver;
#[cfg(feature = "observer-metrics")]
pub use observer::MetricsObserver;
pub use observer::NoopObserver;
pub use observer::Nsga2Observer;
#[cfg(feature = "observer-tracing")]
pub use observer::TracingObserver;
```

These are used directly in tests (`use genetic_algorithms::LogObserver`) and must remain.

---

## Module Move Map

### Group 1: `src/engines/`

Move these modules under `src/engines/`:

| Current path | New path | Notes |
|-------------|----------|-------|
| `src/ga.rs` | `src/engines/ga.rs` | Top-level engine — also exposes `TerminationCause` |
| `src/island/` | `src/engines/island/` | Entire directory moves |
| `src/nsga2/` | `src/engines/nsga2/` | Entire directory moves |
| *(placeholder)* | `src/engines/de/mod.rs` | Empty stub, not pub-exported |
| *(placeholder)* | `src/engines/scatter/mod.rs` | Empty stub, not pub-exported |
| *(placeholder)* | `src/engines/cellular/mod.rs` | Empty stub, not pub-exported |
| *(placeholder)* | `src/engines/alps/mod.rs` | Empty stub, not pub-exported |

`src/engines/mod.rs` exposes:
```rust
pub mod ga;
pub mod island;
pub mod nsga2;
// Placeholder stubs — not re-exported until implemented
mod de;
mod scatter;
mod cellular;
mod alps;
```

lib.rs compatibility shims:
```rust
pub mod engines;
// Preserve original paths
pub mod ga { pub use crate::engines::ga::*; }
pub mod island { pub use crate::engines::island::*; }
pub mod nsga2 { pub use crate::engines::nsga2::*; }
```

**Important:** `pub use ga::TerminationCause;` in lib.rs must now become
`pub use engines::ga::TerminationCause;` (or equivalently the shim module satisfies it).

### Group 2: `src/types/`

Move these modules under `src/types/`:

| Current path | New path |
|-------------|----------|
| `src/chromosomes.rs` + `src/chromosomes/` | `src/types/chromosomes.rs` + `src/types/chromosomes/` |
| `src/genotypes.rs` + `src/genotypes/` | `src/types/genotypes.rs` + `src/types/genotypes/` |

`src/types/mod.rs`:
```rust
pub mod chromosomes;
pub mod genotypes;
```

lib.rs compatibility shims:
```rust
pub mod types;
pub mod chromosomes { pub use crate::types::chromosomes::*; }
pub mod genotypes { pub use crate::types::genotypes::*; }
```

### Group 3: `src/observe/`

Move these modules under `src/observe/`:

| Current path | New path | Feature gate |
|-------------|----------|-------------|
| `src/observer/` | `src/observe/observer/` | none |
| `src/reporter/` | `src/observe/reporter/` | none |
| `src/visualization/` | `src/observe/visualization/` | `visualization` |
| `src/checkpoint.rs` | `src/observe/checkpoint.rs` | `serde` |

`src/observe/mod.rs`:
```rust
pub mod observer;
pub mod reporter;
#[cfg(feature = "visualization")]
pub mod visualization;
#[cfg(feature = "serde")]
pub mod checkpoint;
```

lib.rs compatibility shims:
```rust
pub mod observe;
pub mod observer { pub use crate::observe::observer::*; }
pub mod reporter { pub use crate::observe::reporter::*; }
#[cfg(feature = "visualization")]
pub mod visualization { pub use crate::observe::visualization::*; }
#[cfg(feature = "serde")]
pub mod checkpoint { pub use crate::observe::checkpoint::*; }
```

### Modules that do NOT move

These stay at `src/` root — they are cross-cutting infrastructure, not engines, types, or observers:

| Module | Reason to stay |
|--------|---------------|
| `src/configuration.rs` | Cross-cutting config used by all engines |
| `src/error.rs` | Cross-cutting error type |
| `src/extension/` | Cross-cutting extension operators (also used by ga, island) |
| `src/fitness/` | Cross-cutting fitness helpers |
| `src/initializers/` | Cross-cutting initialization utilities |
| `src/niching/` | Cross-cutting niching logic |
| `src/operations/` | Cross-cutting operator implementations |
| `src/population.rs` | Cross-cutting population container |
| `src/rng.rs` | Cross-cutting RNG |
| `src/stats.rs` | Cross-cutting statistics |
| `src/traits/` | Core trait definitions — never move |
| `src/validators/` | Cross-cutting configuration validation |

---

## Architecture Patterns

### System Architecture (restructure flow)

```
lib.rs (crate root)
  ├── src/engines/mod.rs        <- new group
  │     ├── ga.rs               (moved from src/ga.rs)
  │     ├── island/             (moved from src/island/)
  │     └── nsga2/              (moved from src/nsga2/)
  ├── src/types/mod.rs          <- new group
  │     ├── chromosomes.rs      (moved)
  │     └── genotypes.rs        (moved)
  ├── src/observe/mod.rs        <- new group
  │     ├── observer/           (moved)
  │     ├── reporter/           (moved)
  │     ├── visualization/      (moved, feature-gated)
  │     └── checkpoint.rs       (moved, feature-gated)
  └── [unchanged cross-cutting modules]
        configuration, error, extension, fitness,
        initializers, niching, operations, population,
        rng, stats, traits, validators

Downstream (no changes needed):
  tests/, examples/, benches/ — all use genetic_algorithms::* paths
  that resolve through lib.rs shims
```

### Rust Re-export Pattern Used

The standard pattern for non-breaking module reorganization in Rust:

```rust
// lib.rs — after moving src/ga.rs -> src/engines/ga.rs
pub mod engines;

// Compatibility shim: old path still works
pub mod ga {
    pub use crate::engines::ga::*;
}
```

Internal `crate::ga::Ga` references inside other source files (island, nsga2, etc.) continue to work because lib.rs still exposes `pub mod ga { ... }` as a module — Rust resolves `crate::ga` through the shim.

**Alternative approach — keep `mod X` in lib.rs, change path only:**
Instead of adding a shim module, lib.rs can simply change:
```rust
// Before:
pub mod ga;
// After (file moved to src/engines/ga.rs):
#[path = "engines/ga.rs"]
pub mod ga;
```
The `#[path]` attribute is simpler than a glob re-export shim and avoids glob re-export ambiguity warnings. It preserves the exact module identity (path from crate root is `crate::ga`, not `crate::engines::ga`). This is the preferred approach for this phase.

### Recommended Approach: `#[path]` Attribute

Using `#[path]` in lib.rs is simpler and cleaner than glob-re-export shims for this restructure:

```rust
// lib.rs — after files are moved to src/engines/
#[path = "engines/ga.rs"]
pub mod ga;

#[path = "engines/island/mod.rs"]
pub mod island;

#[path = "engines/nsga2/mod.rs"]
pub mod nsga2;

// After files are moved to src/types/
#[path = "types/chromosomes.rs"]
pub mod chromosomes;

#[path = "types/genotypes.rs"]
pub mod genotypes;

// After files are moved to src/observe/
#[path = "observe/observer/mod.rs"]
pub mod observer;

#[path = "observe/reporter/mod.rs"]
pub mod reporter;

#[cfg(feature = "visualization")]
#[path = "observe/visualization/mod.rs"]
pub mod visualization;

#[cfg(feature = "serde")]
#[path = "observe/checkpoint.rs"]
pub mod checkpoint;
```

With `#[path]`, there is **no** `pub mod engines`, `pub mod types`, or `pub mod observe` in lib.rs. The group directories are invisible to downstream users. Internal `crate::` references need zero changes. The group-level `mod.rs` files are not needed either — lib.rs IS the module dispatcher.

However, if the team wants `genetic_algorithms::engines::ga::Ga` to also work (for future-proofing), they can expose both:
```rust
pub mod engines;           // exposes the group
#[path = "engines/ga.rs"]
pub mod ga;                // preserves old path
```

The STRUCT-01 through STRUCT-03 requirements say "lib.rs re-exports preserve all existing public paths" — the `#[path]` approach satisfies this cleanly.

### Placeholder Directories for Future Engines

STRUCT-01 requires placeholder dirs for `de`, `scatter`, `cellular`, `alps`. These should:
1. Be real directories under `src/engines/`
2. Contain a `mod.rs` with a doc comment only (no `pub` items)
3. NOT be declared in lib.rs (they have no public items yet)
4. Be declared as private `mod` in `src/engines/mod.rs` if that file exists, or left entirely disconnected if using the `#[path]` approach

Simplest form:
```rust
// src/engines/de/mod.rs
//! Differential Evolution engine — placeholder for Phase 26.
```

The directory exists, the file exists, but nothing declares `mod de` publicly, so no public API surface is created.

### Anti-Patterns to Avoid

- **Glob re-export ambiguity:** `pub use crate::engines::ga::*;` in a shim `pub mod ga` can cause "ambiguous glob re-exports" warnings in Rust 2021 if two globs export the same name. Use `#[path]` instead — it avoids this entirely.
- **Forgetting `#[cfg]` on moved feature-gated modules:** `checkpoint` and `visualization` must keep their `#[cfg(feature = "...")]` gates on the `pub mod` declaration in lib.rs.
- **Moving `crate::` references inside source files:** Not needed. If lib.rs still exposes `pub mod ga { ... }` (whether via `#[path]` or shim), `crate::ga` resolves correctly everywhere inside the library.
- **Creating a `src/engines/mod.rs` that conflicts with `#[path]`:** If using `#[path]` in lib.rs, do not also declare `pub mod engines;` in lib.rs unless you intentionally want both the group path AND the legacy path.

---

## Code Path Audit: External Paths Used in Tests, Examples, Benches

Every unique `genetic_algorithms::X` prefix found in the codebase. All must remain resolvable after the restructure.

### Paths that MOVE (need `#[path]` or shim)

| Old path | Which group | Used in |
|----------|-------------|---------|
| `genetic_algorithms::ga::*` | engines | tests/examples/benches — widely used |
| `genetic_algorithms::island::*` | engines | tests/examples/benches — widely used |
| `genetic_algorithms::nsga2::*` | engines | tests/benches |
| `genetic_algorithms::chromosomes::*` | types | tests/examples/benches — widely used |
| `genetic_algorithms::genotypes::*` | types | tests/examples/benches — widely used |
| `genetic_algorithms::observer::*` | observe | tests/benches |
| `genetic_algorithms::reporter::*` | observe | tests |
| `genetic_algorithms::checkpoint::*` | observe | tests (serde feature) |
| `genetic_algorithms::visualization::*` | observe | tests (visualization feature) |

### Paths that DO NOT MOVE (no changes needed)

| Path | Location |
|------|----------|
| `genetic_algorithms::configuration::*` | stays at src root |
| `genetic_algorithms::error::*` | stays at src root |
| `genetic_algorithms::extension::*` | stays at src root |
| `genetic_algorithms::fitness::*` | stays at src root |
| `genetic_algorithms::initializers::*` | stays at src root |
| `genetic_algorithms::niching::*` | stays at src root |
| `genetic_algorithms::operations::*` | stays at src root |
| `genetic_algorithms::population::*` | stays at src root |
| `genetic_algorithms::rng::*` | stays at src root |
| `genetic_algorithms::stats::*` | stays at src root |
| `genetic_algorithms::traits::*` | stays at src root |
| `genetic_algorithms::validators::*` | stays at src root |

### Top-level re-exports (must not change)

```rust
genetic_algorithms::TerminationCause    // from ga::TerminationCause
genetic_algorithms::AllObserver         // from observer::AllObserver
genetic_algorithms::CompositeObserver   // from observer::CompositeObserver
genetic_algorithms::ExtensionEvent      // from observer::ExtensionEvent
genetic_algorithms::GaObserver          // from observer::GaObserver
genetic_algorithms::IslandGaObserver    // from observer::IslandGaObserver
genetic_algorithms::LogObserver         // from observer::LogObserver
genetic_algorithms::MetricsObserver     // from observer::MetricsObserver (observer-metrics)
genetic_algorithms::NoopObserver        // from observer::NoopObserver
genetic_algorithms::Nsga2Observer       // from observer::Nsga2Observer
genetic_algorithms::TracingObserver     // from observer::TracingObserver (observer-tracing)
```

After the restructure, all of these remain in lib.rs as `pub use ga::TerminationCause` etc. — they work as long as `pub mod ga` (even a `#[path]` shim) is in scope.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Path preservation after file move | Custom macro-generated re-exports | `#[path = "new/location.rs"] pub mod old_name;` | Built into Rust; zero overhead; exact same module identity |
| Feature-gated re-export | Conditional proc-macro | `#[cfg(feature = "...")] #[path = "..."] pub mod name;` | Standard Rust; identical to existing pattern |

---

## Common Pitfalls

### Pitfall 1: Internal `crate::` path breaks after move

**What goes wrong:** After moving `src/ga.rs` to `src/engines/ga.rs`, code inside the moved file uses `crate::observer::GaObserver`. If lib.rs still has `pub mod observer { ... }` (either directly or via `#[path]`), this resolves correctly. But if someone also tries to use `crate::engines::observer::...`, that path does not exist.

**Why it happens:** `crate::` is relative to the crate root (lib.rs). As long as lib.rs exports `pub mod observer`, `crate::observer` works from anywhere inside the crate, regardless of where the file physically lives.

**How to avoid:** Do not change any `crate::` references inside moved files. The `#[path]` approach in lib.rs guarantees this works with zero edits to internal code.

**Warning signs:** Compiler error "use of undeclared crate or module" on a `crate::` path inside a moved file — means lib.rs no longer exports that module at the expected name.

### Pitfall 2: Forgetting feature-gate on re-export

**What goes wrong:** `checkpoint` is `#[cfg(feature = "serde")]` in lib.rs. After moving to `src/observe/checkpoint.rs`, if the `#[cfg]` annotation is dropped from the lib.rs `#[path]` declaration, the module compiles unconditionally and pulls in `serde` dependency without the feature flag.

**How to avoid:** Copy the `#[cfg]` annotation exactly: `#[cfg(feature = "serde")] #[path = "observe/checkpoint.rs"] pub mod checkpoint;`

### Pitfall 3: `src/engines/mod.rs` causes double-declaration

**What goes wrong:** lib.rs declares `#[path = "engines/ga.rs"] pub mod ga;` AND `pub mod engines;` where `src/engines/mod.rs` also declares `pub mod ga;`. Rust sees two definitions of `crate::ga` and panics.

**How to avoid:** If using the `#[path]` approach in lib.rs, the `src/engines/mod.rs` should NOT re-declare `pub mod ga`. Either: (a) use `#[path]` in lib.rs exclusively and have no `src/engines/mod.rs`, or (b) use `pub mod engines;` in lib.rs and add compatibility shims separately.

### Pitfall 4: Placeholder dirs not ignored by compiler

**What goes wrong:** `src/engines/de/mod.rs` exists but nothing declares `mod de` anywhere. The file is never compiled. This is correct behavior, but rustdoc will not document it. If someone accidentally declares `pub mod de;` in `src/engines/mod.rs`, an empty public module appears in docs with no items.

**How to avoid:** Placeholder files should be private (`mod de;` not `pub mod de;`) if declared at all. Or leave them undeclared — the file is never compiled until someone adds a declaration.

### Pitfall 5: Clippy warns on glob re-exports

**What goes wrong:** Using `pub use crate::engines::ga::*;` in a shim module generates "glob import from a module that re-exports everything" style Clippy lints in some configurations.

**How to avoid:** Use `#[path]` instead of glob re-export shims. No Clippy warnings for `#[path]`.

---

## Sequencing Strategy

The safest execution order is one group at a time, with `cargo test` between each group:

**Wave 1 — Types group (lowest risk, fewest internal dependencies)**
1. Create `src/types/` directory
2. Move `src/chromosomes.rs` + `src/chromosomes/` → `src/types/`
3. Move `src/genotypes.rs` + `src/genotypes/` → `src/types/`
4. Update lib.rs: add `#[path]` declarations for chromosomes and genotypes
5. Verify: `cargo test` passes, `cargo clippy` clean

**Wave 2 — Observe group (feature-gated, needs care)**
1. Create `src/observe/` directory
2. Move `src/observer/` → `src/observe/observer/`
3. Move `src/reporter/` → `src/observe/reporter/`
4. Move `src/visualization/` → `src/observe/visualization/`
5. Move `src/checkpoint.rs` → `src/observe/checkpoint.rs`
6. Update lib.rs: add `#[path]` declarations with correct `#[cfg]` gates
7. Verify: `cargo test`, `cargo test --features serde`, `cargo clippy` all pass

**Wave 3 — Engines group + placeholders (most cross-referenced)**
1. Create `src/engines/` directory with placeholder subdirs
2. Move `src/ga.rs` → `src/engines/ga.rs`
3. Move `src/island/` → `src/engines/island/`
4. Move `src/nsga2/` → `src/engines/nsga2/`
5. Create empty placeholder `mod.rs` files for de, scatter, cellular, alps
6. Update lib.rs: add `#[path]` declarations for ga, island, nsga2
7. Verify: `cargo test`, `cargo test --features serde`, `cargo clippy`, `cargo doc --no-deps` all pass

**Why this order:**
- Types have the fewest cross-references from other moved modules
- Observe group contains only leaf modules (no other src/ module imports from observer or reporter)
- Engines are most cross-referenced (island uses ga internals, nsga2 uses ga types) — do last when confidence is highest

---

## Doc Tests Assessment

All `///` doc examples in src/ use `ignore` or `rust,ignore` blocks — they are not compiled by `cargo test`. Therefore, no doc test will break from path changes. [VERIFIED: grep of src/ shows only `\`\`\`ignore` and `\`\`\`rust,ignore` patterns]

The lib.rs module-level doc (`//!`) has a `\`\`\`ignore` quickstart example that references `genetic_algorithms::ga::Ga` etc. — these are not compiled and need no change.

---

## Environment Availability Audit

Step 2.6: SKIPPED — this is a code-only restructure. No external tools, services, CLIs, or runtimes beyond the project's own Rust toolchain are required. Toolchain availability is assumed by the project's existing CI.

---

## Runtime State Inventory

Step 2.5: NOT APPLICABLE — this is not a rename/rebrand/migration phase. The restructure affects only source file organization and `pub mod` declarations. No stored data, no service config, no OS-registered state, no secrets/env vars, no build artifacts reference the internal module paths.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness + criterion (benchmarks) |
| Config file | `Cargo.toml` (bench entries); no separate test config |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy && cargo doc --no-deps` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STRUCT-01 | engines/ contains ga, island, nsga2; old paths resolve | compile check | `cargo test` (compile = pass) | All existing tests in `tests/` — no new files needed |
| STRUCT-02 | types/ contains chromosomes, genotypes; old paths resolve | compile check | `cargo test` | All existing tests |
| STRUCT-03 | observe/ contains observer, reporter, visualization, checkpoint; old paths resolve | compile check | `cargo test --features serde` | All existing tests |
| STRUCT-04 | All tests pass, clippy clean, doc warnings zero | full suite | `cargo test && cargo test --features serde && cargo clippy && cargo doc --no-deps` | All existing tests |

**No new test files are needed.** The existing 70+ test files in `tests/` already exercise every public path. If the restructure is correct, they all pass without modification. If any path breaks, a compilation error in the existing tests immediately identifies the gap.

### Sampling Rate
- **Per wave commit:** `cargo test`
- **After Wave 2 (observe group):** `cargo test --features serde`
- **Phase gate:** `cargo test && cargo test --features serde && cargo clippy && cargo doc --no-deps` — zero failures, zero warnings

### Wave 0 Gaps
None — existing test infrastructure fully covers all phase requirements.

---

## Security Domain

Not applicable — this phase makes no changes to security-relevant code paths (authentication, input validation, cryptography, session management, access control). The restructure is purely organizational.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `#[path]` attribute is the cleanest approach for zero-edit-to-internal-code restructures | Architecture Patterns | If `#[path]` has undocumented limitations, glob-shim approach is the fallback — both produce identical public API |
| A2 | All doc tests in src/ use `ignore` — no compiled doc tests reference module paths | Doc Tests Assessment | If a compiled doc test (no `ignore`) exists, it must be updated; grep found none |
| A3 | Internal `crate::` references in moved files require zero changes because lib.rs continues to expose the same top-level module names | Sequencing Strategy | If a moved file uses a path relative to its own new location (e.g., `super::`) rather than `crate::`, that reference would break. Verified: files use `crate::` not `super::` |

---

## Open Questions (RESOLVED)

1. **Should `src/engines/mod.rs` exist, or just use `#[path]` in lib.rs?**
   - What we know: `#[path]` approach requires no `src/engines/mod.rs`. The group directory is invisible to the compiler unless something declares it.
   - What's unclear: Whether the project wants `genetic_algorithms::engines::ga::Ga` to be a valid future path (for IDE discoverability), or only the legacy paths.
   - Recommendation: Start with `#[path]` only (simpler, fewer files). If future discoverability is desired, add `pub mod engines` alongside the `#[path]` shims in a follow-up.
   - **RESOLVED:** `#[path]` only — no `src/engines/mod.rs`. Group directories are invisible to the compiler; legacy crate-root paths are preserved via `#[path]` attributes in lib.rs. No `pub mod engines` declaration will be added in this phase.

2. **Do placeholder stub files need to be in `src/engines/` at all?**
   - What we know: STRUCT-01 says "src/engines/ contains ga, island, nsga2 (and placeholder dirs for de, scatter, cellular, alps)".
   - What's unclear: Whether the placeholder dirs need compilation linkage or just filesystem presence.
   - Recommendation: Create the dirs with minimal `mod.rs` stubs (doc comment only) but do not declare them in any `mod` statement. They are structurally present for Phase 26-29 developers to discover without contributing to the public API.
   - **RESOLVED:** Filesystem presence only. Placeholder directories (de/, scatter/, cellular/, alps/) each contain a `mod.rs` with a `//!` doc comment only. Nothing in lib.rs or any compiled file declares `mod de;` etc. — zero public API surface until Phases 26-29 implement them.

---

## Sources

### Primary (HIGH confidence)
- [VERIFIED: src/lib.rs] — complete list of `pub mod` declarations and top-level `pub use` re-exports
- [VERIFIED: find src/] — exhaustive file listing of every .rs file in src/
- [VERIFIED: tests/ grep] — all `use genetic_algorithms::` paths in 70+ test files
- [VERIFIED: examples/ grep] — all `use genetic_algorithms::` paths in 10 example files
- [VERIFIED: benches/ grep] — all `use genetic_algorithms::` paths in 8 bench files
- [VERIFIED: Cargo.toml] — feature flags: `serde`, `visualization`, `observer-tracing`, `observer-metrics`

### Secondary (MEDIUM confidence)
- [ASSUMED] Rust `#[path]` attribute behavior for module reorganization — standard Rust language feature, behavior is well-defined in the Rust Reference

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Current src/ layout: HIGH — directly read from filesystem
- Public path surface: HIGH — directly read from all test, example, bench imports
- Rust `#[path]` mechanics: HIGH — standard Rust language feature
- Sequencing recommendation: HIGH — based on dependency analysis of cross-module references
- Placeholder dir requirements: MEDIUM — requirement says "dirs exist", implementation detail is Claude's discretion

**Research date:** 2026-04-26
**Valid until:** This is a point-in-time snapshot of the codebase. Valid until any new module is added to lib.rs.
