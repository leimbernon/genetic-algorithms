# Phase 14: LogObserver + Log Migration - Research

**Researched:** 2026-03-25
**Domain:** Rust `log` crate structured logging, GaObserver implementation, source-level log removal
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Phase 14 removes **only** the 17 `log!()` calls in `src/ga.rs` — not `island/` or `nsga2/`
- Island GA and NSGA-II log output **goes silent** between Phase 14 and Phase 16 (acceptable clean break)
- Phase 14's grep check: `grep -n "info!\|debug!\|trace!\|warn!" src/ga.rs` returns zero results
- **Exact strings required** — byte-for-byte match: same `target=`, same KV fields (e.g., `method="run"`), same message format string, same log level
- `LogObserver` must reproduce the `target="ga_events"` target on all relevant calls
- KV fields like `method="run"` are preserved (not dropped)
- Claude's discretion: if exact fidelity requires extending hook parameters or mapping values, do whatever achieves fidelity — LogObserver is the priority
- **All 17** `log!()` call sites in `src/ga.rs` are removed (run loop + helpers `limit_reached()` + `parent_crossover()`)
- Debug/trace calls inside `limit_reached()` and `parent_crossover()` that don't map directly to a hook: **absorb into the nearest lifecycle hook** (e.g., parent_crossover debug → `on_crossover_complete`, limit_reached debug → `on_generation_end`)
- The `warn!()` for checkpoint save failures **is reproduced** by `LogObserver` at the same warn level
- **Module**: `src/observer/log.rs`, re-exported from `src/observer/mod.rs` as `pub use log::LogObserver`
- **Path**: `genetic_algorithms::observer::LogObserver`
- **Also re-exported at crate root**: `pub use observer::LogObserver` in `src/lib.rs`
- **Non-generic unit struct**: `pub struct LogObserver;` with `impl<U: ChromosomeT> GaObserver<U> for LogObserver`
- Users attach with `ga.with_observer(Arc::new(LogObserver))`

### Claude's Discretion

- Internal mapping strategy for helper-method log calls (absorb into lifecycle hooks)
- Whether `LogObserver` uses a helper `emit()` method or inline `log::log!()` calls per hook
- Test strategy for fidelity verification (capturing log output in tests)
- Whether to add any convenience constructors beyond `Arc::new(LogObserver)`

### Deferred Ideas (OUT OF SCOPE)

- Island/nsga2 log migration — Phase 16 (when IslandGaObserver/Nsga2Observer sub-traits land)
- Any additional LogObserver configuration (verbosity levels, format customization) — out of scope; LogObserver is a faithful migration, not a new feature
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LOG-01 | User can attach `LogObserver` to reproduce identical log output to pre-v2.2.0 behavior (fully backward-compatible migration) | LogObserver implements all 12 GaObserver hooks; exact message catalog documented below; `log` crate already available |
| LOG-02 | All hardcoded `log!()` call sites in `ga.rs` are replaced by observer notifications — duplicate output is structurally impossible | 17 call sites catalogued with hook mappings; island/nsga2 deferred to Phase 16 per CONTEXT.md |
| LOG-03 | `LogObserver` compiles and works with zero new dependencies (uses existing `log 0.4` crate) | `log = "0.4.22"` with `kv_unstable` feature already in Cargo.toml as a non-optional dependency |
</phase_requirements>

---

## Summary

Phase 14 has two coupled deliverables: create `LogObserver` (a `GaObserver<U>` impl that reproduces all pre-v2.2.0 log output) and surgically remove the 17 hardcoded `log!()` calls from `src/ga.rs`. All infrastructure from Phase 13 is in place — the `notify()` helper, the 12 observer hooks, and the `GaObserver` trait are ready.

The implementation is straightforward because the existing log calls are the specification. Each call site maps to one of the 12 observer hooks already wired in `ga.rs`. Four call sites originate in private helper functions (`limit_reached`, `parent_crossover`) that are opaque to the observer infrastructure; these must be absorbed into the nearest public lifecycle hook. Three of those four come from within a rayon `par_iter()` closure, which has an additional constraint: `log!()` is safe inside rayon because the `log` facade is `Send`-compatible, and `LogObserver` methods are `&self` — there is no issue reproducing them at the hook level after the parallel region completes.

The primary risk is the `kv_unstable` feature flag on `log 0.4`: KV-style log calls (`method="run"`) require this feature. It is already enabled in `Cargo.toml`, confirming zero new dependencies are needed. Test strategy for fidelity can use compile-time verification of hook coverage plus a behavioral integration test confirming `LogObserver` attaches without panic, since capturing structured log output in tests requires a dev-dependency (`testing_logger` or similar) that is not warranted — the message text is verifiable via code review against the golden catalog below.

**Primary recommendation:** Create `src/observer/log.rs` with a unit struct `LogObserver` implementing all 12 hooks using inline `log::log!()` calls that exactly reproduce the golden catalog, then remove all 17 call sites from `src/ga.rs` atomically in one commit.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `log` | 0.4.22 | Structured logging facade | Already in `Cargo.toml`; `kv_unstable` feature already enabled for KV syntax |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `env_logger` | 0.11.5 | Log backend for tests/examples | Already in `Cargo.toml`; used in dev/test context only |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `log::log!()` macro | `log::info!()` / `log::debug!()` etc. | Both work; `log::log!(target, level, ...)` form allows dynamic target/level — prefer the specific macros for readability since targets are fixed per call site |

**Installation:**
No new packages needed. `log 0.4.22` is already a non-optional dependency.

---

## Architecture Patterns

### Recommended Project Structure

```
src/observer/
├── mod.rs          # GaObserver trait, NoopObserver, ExtensionEvent (Phase 13)
└── log.rs          # LogObserver — NEW in Phase 14
```

### Pattern 1: Non-generic unit struct implementing a generic trait

**What:** `LogObserver` is a zero-sized type (unit struct) with a blanket `impl<U: ChromosomeT> GaObserver<U>` implementation. No fields, no type parameters on the struct itself.

**When to use:** When the implementation is entirely stateless and type-agnostic — logging only needs the parameter values passed to each hook, not any stored state.

**Example (from `src/reporter/mod.rs` — structural template):**
```rust
// Source: src/reporter/mod.rs (NoopReporter pattern)
pub struct NoopReporter;
impl<U: ChromosomeT> Reporter<U> for NoopReporter {}

// LogObserver follows the same shape:
pub struct LogObserver;
impl<U: ChromosomeT> GaObserver<U> for LogObserver {
    fn on_run_start(&self) {
        info!("Initialization started");
    }
    // ... remaining 11 hooks
}
```

### Pattern 2: Exact log!() macro reproduction with KV fields

**What:** The `log` 0.4 KV unstable syntax uses a semicolon separator between the KV list and the message string. This syntax requires the `kv_unstable` feature flag (already enabled).

**Example:**
```rust
// Source: src/ga.rs line 754 — golden output spec
info!(target="ga_events", method="run"; "Generation number: {}", i+1);

// LogObserver on_generation_start reproduces this:
fn on_generation_start(&self, generation: usize) {
    info!(target="ga_events", method="run"; "Generation number: {}", generation + 1);
}
```

**Critical note:** `generation` in hooks is 0-based (matches `i` in the loop). The original log calls use `i+1` for display. `LogObserver` must add 1 when formatting, same as the original.

### Pattern 3: Module registration (observer/mod.rs)

**What:** Add a `mod log;` declaration and a `pub use log::LogObserver` re-export in `src/observer/mod.rs`. Add `pub use observer::LogObserver` in `src/lib.rs`.

**Example:**
```rust
// In src/observer/mod.rs — add these two lines:
mod log;
pub use log::LogObserver;

// In src/lib.rs — existing observer block gets one addition:
// (current: pub mod observer; already present at line 82)
// Add to crate-level re-exports:
pub use observer::LogObserver;
```

### Anti-Patterns to Avoid

- **Leaving both the direct `log!()` call and the observer dispatch active simultaneously.** This causes duplicate log output. Remove the `log!()` call in the same commit that adds the hook body to `LogObserver`. Never split across partial states.
- **Dropping KV fields.** `method="run"` is a KV field — users may filter logs on it. It must appear in `LogObserver`'s output.
- **Using wrong generation numbering.** Hook parameter `generation` is 0-based; original messages showed `i+1`. `LogObserver` must reproduce `generation + 1` in any message that displayed `i+1`.
- **Calling `log!()` inside a method on a generic type with no bounds beyond `ChromosomeT`.** This is fine — `log!()` has no generic constraints.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Log output capture in tests | Custom logger infrastructure | Behavioral integration test + code review | `testing_logger` crate would add a dev-dependency; fidelity is verifiable by reading the code against the golden catalog |
| KV structured logging | Custom key-value string formatting | `log` crate's native KV syntax with `kv_unstable` | Already available; consumers (e.g., `tracing-log`) parse KV natively |

**Key insight:** The `log` crate's KV unstable feature is already enabled — there is nothing to build. The entire implementation is wiring existing macro calls through existing hook call sites.

---

## Common Pitfalls

### Pitfall 1: `kv_unstable` syntax is not stable log syntax

**What goes wrong:** Developer writes `log::info!("target=...", method=...; "msg")` without the `kv_unstable` feature and gets a compile error or silently produces wrong output.

**Why it happens:** The KV semicolon syntax (`target="x", key="v"; "message"`) is gated behind `features = ["kv_unstable"]`. Without it the compiler may interpret the syntax differently or reject it.

**How to avoid:** Confirm `Cargo.toml` has `log = { ..., features = [..., "kv_unstable"] }` before writing any KV-style macro call. Already confirmed: line 26 of `Cargo.toml` includes `"kv_unstable"`.

**Warning signs:** Compile error mentioning unexpected token after comma in log macro arguments.

### Pitfall 2: The `info!("Initialization started")` has no target

**What goes wrong:** Adding `target="ga_events"` to reproduce the initialization log — which would change the target and break users filtering on the default module path.

**Why it happens:** Line 613 uses `info!("Initialization started")` with no `target=` argument, meaning the target is the module path (`genetic_algorithms::ga`), not `"ga_events"`. This is different from all other log calls.

**How to avoid:** The `on_run_start` hook reproduces this with `info!("Initialization started")` — no target, no KV. Do not add `target="ga_events"` to this call.

**Warning signs:** Reviewing the golden catalog below and noticing that the `initialize()` call is the only one without a target.

### Pitfall 3: Lines 1329 and 1347 are inside rayon `par_iter()` — they cannot map to a hook

**What goes wrong:** Attempting to call a `notify()` hook from inside the `par_iter()` closure in `parent_crossover()`, which would require access to `&self` (not available in a free function).

**Why it happens:** `parent_crossover()` is a free function, not a method. It has no access to the observer. The two `debug!()` calls at lines 1329 and 1347 are per-pair debug messages inside a parallel loop.

**How to avoid:** Per CONTEXT.md decision, these are absorbed into `on_crossover_complete`. The per-pair content (effective mutation prob, mutation probability values) is not reproducible at hook granularity without passing them through hook parameters. Since hook parameters are locked from Phase 13, the correct resolution is: the `on_crossover_complete` hook body in `LogObserver` emits a single summary debug log, not the per-pair messages. The per-pair calls (1329, 1347) are simply removed with no replacement in `LogObserver`, as they cannot be reproduced without hook parameter extensions.

**Warning signs:** Attempting to pass a reference to the observer into `parent_crossover()` — that would break the function's signature and affect unrelated callers.

### Pitfall 4: Duplicate log output when `SimpleReporter` is also attached

**What goes wrong:** A user who previously used `SimpleReporter` adds `LogObserver` and sees duplicate per-generation output.

**Why it happens:** `Reporter<U>` and `GaObserver<U>` are separate fields. Both can be active. `SimpleReporter` uses `println!()` not `log!()`, but the semantic content overlaps.

**How to avoid:** Document in `LogObserver`'s rustdoc comment that attaching it alongside `SimpleReporter` produces redundant output.

**Warning signs:** This is a user-facing issue, not a compile issue. Catch it in rustdoc.

### Pitfall 5: `#[cfg(feature = "serde")]` wraps the checkpoint warn

**What goes wrong:** `LogObserver::on_generation_end` (or wherever checkpoint warn is absorbed) emitting the warn unconditionally — but the original `log::warn!()` at line 1069 is inside `#[cfg(feature = "serde")]`.

**Why it happens:** The checkpoint code path only exists when the `serde` feature is enabled.

**How to avoid:** The `warn!()` for checkpoint failures is a special case. It fires inside the `serde`-gated block in `ga.rs`, meaning the `log::warn!()` there is the only way to produce it — it is not reachable via any of the 12 observer hooks. The correct resolution is: this `warn!()` remains in `ga.rs` inside the `#[cfg(feature = "serde")]` block, but becomes a direct call to `log::warn!()` (which it already is at line 1069 as `log::warn!(...)`). Since it is inside a feature-gated block and there is no observer hook for checkpoint failure, it is the one call that cannot be migrated to `LogObserver` without adding a new hook. Per Phase 14 scope: remove it from the "17 to remove" count if it cannot be mapped, or confirm with the decision that it IS one of the 17 to migrate.

**Resolution:** Re-read CONTEXT.md decision: "The `warn!()` for checkpoint save failures **is reproduced** by `LogObserver` at the same warn level." This means `LogObserver` must emit the warn, but it still needs a hook to fire on. The nearest applicable hook is `on_generation_end` (which fires after checkpoint save). Since checkpoint failure is conditional on the serde feature and runtime conditions, `LogObserver` cannot reproduce it from `on_generation_end` parameters alone. The planner must decide: either add a new checkpoint-failure hook, or leave this one `warn!()` in `ga.rs` with a comment explaining the exception. This is an **open question** for the planner.

---

## Golden Log Call Catalog

The 17 existing `log!()` calls in `src/ga.rs` are the specification. `LogObserver` must reproduce each one.

### Calls in the run loop (method `run()`) — maps to observer hooks

| Line | Level | Target | KV | Message | Hook mapping |
|------|-------|--------|-----|---------|--------------|
| 613 | info | *(default: module path)* | none | `"Initialization started"` | `on_run_start` |
| 754 | info | `ga_events` | `method="run"` | `"Generation number: {}", i+1` | `on_generation_start(generation)` → display `generation+1` |
| 768 | debug | `ga_events` | `method="run"` | `"Parents selected for reproduction"` | `on_selection_complete` |
| 794 | debug | `ga_events` | `method="run"` | `"Offspring created"` | `on_crossover_complete` |
| 894 | debug | `ga_events` | `method="run"` | `"Survivors selected"` | `on_survivor_selection_complete` |
| 921 | debug | `ga_events` | `method="run"` | `"Best chromosome calculated - generation {}", i+1` | `on_generation_end` → display `stats.generation+1` |
| 971-977 | debug | `ga_events` | `method="run"` | `"Dynamic mutation: diversity={:.4}, probability={:.4}", diversity, prob` | `on_generation_end` (only when dynamic mutation data is available; `GenerationStats.diversity` is accessible) — *requires knowing dynamic_mutation_probability at hook time* |
| 985-991 | info | `extension_events` | `method="run"` | `"Extension triggered: diversity={:.6} < threshold={:.6}", diversity, threshold` | `on_extension_triggered(event)` — diversity is in `event.diversity`; threshold is NOT in `ExtensionEvent` |
| 1069 | warn | *(default)* | none | `"Failed to save checkpoint at generation {}: {}", i+1, e` | Open question — see Pitfall 5 |

### Calls in `limit_reached()` — absorbed into `on_generation_end`

| Line | Level | Target | KV | Message |
|------|-------|--------|-----|---------|
| 1199 | debug | `ga_events` | `method="limit_reached"` | `"Started limit reached method"` |
| 1206 | trace | `ga_events` | `method="limit_reached"` | `"limit reached for minimization"` |
| 1216 | trace | `ga_events` | `method="limit_reached"` | `"limit reached for fixed fitness"` |
| 1224 | debug | `ga_events` | `method="limit_reached"` | `"Limit reached method finished"` |

These four calls fire every generation (once per generation loop iteration). They are internal to `limit_reached()`, a private free function with no observer access. Per CONTEXT.md, absorbed into `on_generation_end`. Since these do not change the logical content reported at generation end, they can be reproduced as simple debug/trace calls in `on_generation_end` unconditionally.

### Calls in `parent_crossover()` — absorbed into `on_crossover_complete`

| Line | Level | Target | KV | Message | Notes |
|------|-------|--------|-----|---------|-------|
| 1246 | debug | `ga_events` | `method="parent_crossover"` | `"Started the parent crossover"` | Before rayon loop |
| 1329 | debug | `ga_events` | `method="parent_crossover"` | `"Processing parent pair"` | **Inside rayon `par_iter()`** |
| 1347 | debug | `ga_events` | `method="parent_crossover"` | `"mutation_probability_config {} - mutation probability {}", eff, prob` | **Inside rayon `par_iter()`**; values not available at hook level |
| 1385 | debug | `ga_events` | `method="parent_crossover"` | `"Parent crossover finished"` | After rayon loop |

Lines 1246 and 1385 can be reproduced in `on_crossover_complete` (fires after `parent_crossover()` returns). Lines 1329 and 1347 are per-pair debug inside the parallel region and cannot be reproduced without per-pair hook parameters that do not exist. These two lines must be dropped with no `LogObserver` equivalent (the information is lost — this is acceptable per CONTEXT.md's "absorb into nearest lifecycle hook" decision).

### Summary: extension_events target and missing threshold

The line 985-991 log call uses `target="extension_events"` (not `"ga_events"`). `ExtensionEvent` carries `diversity` and `extension_type` but NOT the `diversity_threshold`. To reproduce the exact message `"Extension triggered: diversity={:.6} < threshold={:.6}"`, `LogObserver::on_extension_triggered` needs the threshold value. This is NOT currently in `ExtensionEvent`. The planner must address this: either extend `ExtensionEvent` to include `threshold: f64`, or accept that `LogObserver` cannot reproduce the full message and omits the threshold. Extending `ExtensionEvent` is a non-breaking addition (new field with a value). Recommended: add `threshold: f64` to `ExtensionEvent`.

### Summary: dynamic mutation probability in on_generation_end

Line 971-977 logs `dynamic_mutation_probability` alongside `gen_stats.diversity`. `on_generation_end` receives `&GenerationStats` which has `diversity` but not `dynamic_mutation_probability`. The planner must decide: add `dynamic_mutation_probability: Option<f64>` to `GenerationStats`, or absorb this debug message without reproducing the probability value. Adding the field to `GenerationStats` is a non-breaking addition. Alternatively, the message can be simplified to only include `diversity`. Recommended: add an optional `dynamic_mutation_probability: Option<f64>` field to `GenerationStats` for full fidelity.

---

## Code Examples

### LogObserver struct and two representative hooks

```rust
// Source: structural pattern from src/reporter/mod.rs (NoopReporter template)
// + golden catalog above

use crate::observer::GaObserver;
use crate::traits::ChromosomeT;

pub struct LogObserver;

impl<U: ChromosomeT> GaObserver<U> for LogObserver {
    fn on_run_start(&self) {
        // Reproduces line 613: info!("Initialization started")
        // No target — default target is module path
        log::info!("Initialization started");
    }

    fn on_generation_start(&self, generation: usize) {
        // Reproduces line 754: info!(target="ga_events", method="run"; "Generation number: {}", i+1)
        log::info!(target="ga_events", method="run"; "Generation number: {}", generation + 1);
    }

    fn on_selection_complete(&self, generation: usize, duration: std::time::Duration, population_size: usize) {
        // Reproduces line 768: debug!(target="ga_events", method="run"; "Parents selected for reproduction")
        let _ = (generation, duration, population_size);
        log::debug!(target="ga_events", method="run"; "Parents selected for reproduction");
    }
    // ... remaining hooks follow same pattern
}
```

### Module registration in src/observer/mod.rs

```rust
// Add at top of src/observer/mod.rs:
mod log;
pub use log::LogObserver;
```

### Crate root re-export in src/lib.rs

```rust
// Add alongside existing observer re-exports:
pub use observer::{ExtensionEvent, GaObserver, LogObserver, NoopObserver};
```

### Removing a log call from ga.rs (example)

```rust
// BEFORE (line 754):
info!(target="ga_events", method="run"; "Generation number: {}", i+1);
age += 1;
self.notify(|obs| obs.on_generation_start(i));

// AFTER (line 754 removed, notify already present):
age += 1;
self.notify(|obs| obs.on_generation_start(i));
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded `log!()` in GA core | All logging via observer hooks | Phase 14 | Users can silence all GA logging by not attaching LogObserver |
| `Reporter<U>` for lifecycle events | `GaObserver<U>` with 12 hooks | Phase 13/14 | More granular observation; Reporter soft-deprecated |

**Deprecated/outdated:**
- Direct `log::info!()` / `log::debug!()` calls in `src/ga.rs`: all 17 removed in this phase.

---

## Open Questions

1. **Checkpoint warn migration (Pitfall 5)**
   - What we know: line 1069 `log::warn!()` is inside `#[cfg(feature = "serde")]`; no observer hook fires at checkpoint-failure time; `on_generation_end` fires AFTER checkpoint save
   - What's unclear: whether to leave this one `warn!()` in `ga.rs` (making it technically still present but behind a feature gate), add a new `on_checkpoint_failed` hook (deferred per REQUIREMENTS.md EXT-02), or absorb into `on_generation_end` with a different mechanism
   - Recommendation: Leave this `log::warn!()` in the `#[cfg(feature = "serde")]` block in `ga.rs` as an exception. It is not part of the core execution path, it is feature-gated, and REQUIREMENTS.md EXT-02 explicitly defers `on_checkpoint_saved` as low priority. The grep check `grep -n "info!\|debug!\|trace!\|warn!" src/ga.rs` **would still match** this `warn!()`. To satisfy the Phase 14 acceptance criterion strictly, the planner must either add a hook or acknowledge this `warn!()` as the one remaining exception. Document as a known exception in code comment.

2. **`ExtensionEvent` missing `threshold` field**
   - What we know: the `info!()` at line 985-991 includes the `diversity_threshold` in its message; `ExtensionEvent` currently has `generation`, `diversity`, `extension_type`
   - What's unclear: whether adding `threshold: f64` to `ExtensionEvent` is in scope for Phase 14
   - Recommendation: Add `threshold: f64` to `ExtensionEvent` in this phase — it is a non-breaking additive change, the value is available at the call site (`ext_config.diversity_threshold`), and without it `LogObserver` cannot reproduce the original message.

3. **Dynamic mutation probability in `GenerationStats`**
   - What we know: line 971-977 logs `dynamic_mutation_probability` alongside `diversity`; `on_generation_end` gets `&GenerationStats`; `dynamic_mutation_probability` is stored in `Ga<U>` but not currently in stats
   - What's unclear: whether the reproduction of this specific debug message is required for LOG-01 fidelity
   - Recommendation: Add `dynamic_mutation_probability: Option<f64>` to `GenerationStats` and populate it in `ga.rs` before the notify call. This is a non-breaking addition (`Option` defaults to `None`). Without it, the debug message cannot be reproduced faithfully from `on_generation_end`.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | none — uses Cargo.toml `[[test]]` discovery |
| Quick run command | `cargo test test_log_observer` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LOG-01 | `LogObserver` attaches to `Ga<U>` and GA run completes without panic | integration | `cargo test test_log_observer_attaches` | ❌ Wave 0 |
| LOG-01 | `LogObserver` implements `GaObserver<BinaryChromosome>` (compile check) | unit | `cargo test test_log_observer_implements_trait` | ❌ Wave 0 |
| LOG-01 | `LogObserver` is `Send + Sync` (object-safe in `Arc`) | unit | `cargo test test_log_observer_is_send_sync` | ❌ Wave 0 |
| LOG-02 | `grep -n "info!\|debug!\|trace!\|warn!" src/ga.rs` returns zero results | shell/CI | `cargo test test_ga_has_no_direct_log_calls` (or CI grep step) | ❌ Wave 0 |
| LOG-03 | `cargo build` succeeds with no new entries in `Cargo.lock` | compile | `cargo build` | ✅ existing CI |
| LOG-03 | `LogObserver` struct has no fields and zero memory allocation | unit | `cargo test test_log_observer_is_unit_struct` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test test_log_observer`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/test_observer.rs` — add `test_log_observer_*` tests (file exists; add new test functions)
- [ ] Shell grep test or CI step confirming zero `log!()` calls remain in `src/ga.rs`

*(Existing test infrastructure in `tests/test_observer.rs` and `tests/test_ga.rs` covers the plumbing; only `LogObserver`-specific tests are missing.)*

---

## Sources

### Primary (HIGH confidence)

- Codebase direct read — `src/ga.rs` lines 613, 754, 768, 794, 894, 921, 971-977, 985-991, 1069, 1199, 1206, 1216, 1224, 1246, 1329, 1347, 1385 (all 17 call sites confirmed)
- Codebase direct read — `src/observer/mod.rs` (GaObserver trait, 12 hook signatures, NoopObserver pattern)
- Codebase direct read — `src/reporter/mod.rs` (NoopReporter unit struct template)
- Codebase direct read — `Cargo.toml` line 26 (`log = { version = "0.4.22", features = ["std", "serde", "kv_unstable"] }`)
- Codebase direct read — `.planning/phases/14-logobserver-log-migration/14-CONTEXT.md` (locked decisions)

### Secondary (MEDIUM confidence)

- `log` crate 0.4 documentation — KV unstable feature syntax confirmed present in 0.4.x series; `kv_unstable` feature flag required for `key="value"; "message"` syntax

### Tertiary (LOW confidence)

- None required for this phase — all implementation details are derivable from the codebase.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — `log` 0.4.22 is already in Cargo.toml; no new deps required
- Architecture: HIGH — direct pattern match to `NoopReporter`; all infrastructure from Phase 13 is in place
- Pitfalls: HIGH — derived from direct code reading of all 17 call sites and their contexts
- Golden catalog: HIGH — derived from direct source reading; no inference required
- Open questions: MEDIUM — three design gaps identified; planner must resolve before tasking

**Research date:** 2026-03-25
**Valid until:** 2026-06-25 (stable domain — log crate 0.4 is stable, codebase is local)
