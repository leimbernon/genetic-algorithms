# Phase 49: Unified Strategy Trait + Alternative Strategy Engines - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-22
**Phase:** 49-unified-strategy-trait-alternative-strategy-engines
**Areas discussed:** Strategy<U> trait shape, Observer hook mapping, HillClimbEngine structure, PermutateEngine candidate enumeration

---

## Strategy<U> Trait Shape

### run() return type

| Option | Description | Selected |
|--------|-------------|----------|
| `()` (side effect only) | Engine stores result internally; user calls `best()` after `run()`. Simplest, dyn-safe. | |
| `Result<(), GaError>` | Same but propagates engine errors. Consistent with how `Ga::run()` currently signals problems. | ✓ |
| Associated type per engine | Each engine returns its own result type. NOT dyn-safe — breaks Box<dyn Strategy<U>>. | |

**User's choice:** `Result<(), GaError>`

### best() return type

| Option | Description | Selected |
|--------|-------------|----------|
| `Option<&U>` (reference, dyn-safe) | Borrow from internal state. Zero-copy. Works for Box<dyn Strategy<U>>. | ✓ |
| `Option<U>` (clone, dyn-safe) | Owned clone. Simpler caller ergonomics but heavier. | |
| `U` (panics if not run yet) | Forces engines to have dummy state before run(). Risky. | |

**User's choice:** `Option<&U>`

### with_observer() placement

| Option | Description | Selected |
|--------|-------------|----------|
| On individual engine structs only | Observer wired at build time before boxing. Trait stays minimal. | ✓ |
| On the Strategy<U> trait | Allows `strategy.with_observer(obs)` after boxing. Complicates dyn safety. | |

**User's choice:** On individual engine structs only (concrete types)

---

## Observer Hook Mapping

### Which hooks fire in HillClimb/Permutate

| Option | Description | Selected |
|--------|-------------|----------|
| `on_generation_start + on_new_best + on_generation_end` (reuse GA hooks) | Each iteration is a "generation". on_new_best fires on acceptance. on_run_start/on_run_end bracket. | ✓ |
| Subset only: on_run_start, on_new_best, on_run_end | Only lifecycle hooks with clear non-GA meaning. Less per-step visibility. | |
| New engine-specific hooks on GaObserver | Expands GaObserver contract — forces all existing observers to update. | |

**User's choice:** Reuse GA hooks — each iteration mapped as a generation.

### PermutateEngine gate overflow signal

| Option | Description | Selected |
|--------|-------------|----------|
| `log::warn!` macro | Uses existing `target = "ga_events"` logging pattern from ga.rs. Zero API surface. | ✓ |
| `on_stagnation` hook | Semantically imperfect reuse of existing infrastructure. | |
| Return `GaError::PermutationGateExceeded` from `run()` | Non-fatal error treatment — user decides how to handle. | |

**User's choice:** `log::warn!(target = "ga_events", ...)` — existing pattern, no new API.

---

## HillClimbEngine Structure

### Single struct vs two structs

| Option | Description | Selected |
|--------|-------------|----------|
| One struct with `HillClimbMode` enum field | Shares run(), neighbor_fn, observer wiring, stopping logic. Mode only changes neighbor selection. | ✓ |
| Two separate structs | Clearer type distinction but duplicates neighbor_fn storage, observer wiring, stopping logic. | |

**User's choice:** Single `HillClimbEngine<U>` with `mode: HillClimbMode` field.

### neighbor_fn storage type

| Option | Description | Selected |
|--------|-------------|----------|
| `Arc<dyn Fn(&U) -> Vec<U> + Send + Sync>` | Consistent with observer storage. Clonable. | ✓ |
| `Box<dyn Fn(&U) -> Vec<U>>` | Simpler, non-clonable. Less consistent with Arc pattern. | |

**User's choice:** `Arc<dyn Fn(&U) -> Vec<U> + Send + Sync>`

---

## PermutateEngine Candidate Enumeration

### How candidates are generated

| Option | Description | Selected |
|--------|-------------|----------|
| User provides `Vec<U>` at build time | Engine iterates lazily (.iter()), tracks running best. Decoupled from chromosome internals. | ✓ |
| Engine generates permutations from starting chromosome + alphabet | Tightly coupled — only works for permutation chromosomes. | |
| User provides a closure `Fn(usize) -> Option<U>` | Lazy, no Vec allocation. But callable state needs Box/Arc storage. | |

**User's choice:** `Vec<U>` at build time, engine iterates lazily.

### Materialization strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Evaluate lazily — iterate and compare, no full materialization | Only current best kept in memory. Gate counts iterations. | ✓ |
| Materialize into Vec<U> first, then evaluate | Holds all candidates in memory. Defeats safety gate purpose. | |

**User's choice:** Lazy iteration — evaluate one at a time.

---

## Claude's Discretion

None — all areas had clear user preferences.

## Deferred Ideas

None — discussion stayed within phase scope.
