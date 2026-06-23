---
phase: 53-tree-chromosome-gpga-engine
plan: "04"
subsystem: engines/gp
tags:
  - genetic-programming
  - serde
  - serde_stacker
  - checkpoint
  - wasm-safe

dependency_graph:
  requires:
    - phase: 53-01
      provides: Node<N>, GpChromosome<N> with serde derive support
    - phase: 53-03
      provides: GpGa engine, test_serde_deep_tree stub in tests/gp.rs
  provides:
    - serde_stacker optional dep wired into serde feature
    - test_serde_deep_tree: depth-64 right-spine tree round-trips through JSON
    - TestNode derives Serialize/Deserialize under serde feature
    - Node<N> rustdoc explaining serde_stacker usage pattern
  affects:
    - tests/gp.rs (test_serde_deep_tree activated, 16 tests total)

tech_stack:
  added:
    - "serde_stacker 0.1.14 (dtolnay) — stack-safe serde wrappers for deep trees"
    - "serde_json unbounded_depth feature (dev-dependency) — enables disable_recursion_limit()"
  patterns:
    - "serde_stacker pattern: wrap serde_json::Serializer with serde_stacker::Serializer at the call site (not inside Serialize impl)"
    - "Call json_de.disable_recursion_limit() before wrapping with serde_stacker::Deserializer — requires unbounded_depth feature on serde_json"
    - "Optional dep pattern: serde_stacker = { version = '0.1', optional = true } wired via serde feature"

key_files:
  created: []
  modified:
    - Cargo.toml
    - src/engines/gp/node.rs
    - tests/gp.rs

key-decisions:
  - "serde_stacker wrappers applied at call site (test), not inside Serialize/Deserialize trait impls — Node<N> retains #[derive(Serialize, Deserialize)] and serde_stacker::Serializer/Deserializer wrap the underlying serde_json serializer at the usage point"
  - "serde_json unbounded_depth feature required in dev-dependencies to enable disable_recursion_limit() — without it serde_json's internal recursion check fires at 128 nesting levels before serde_stacker can intervene"
  - "serde_stacker wired as optional dep under the existing serde feature — no standalone feature flag, no unconditional dep; fully backward compatible"
  - "wasm32 check confirmed passing: serde_stacker 0.1.14 compiles for wasm32-unknown-unknown (stacker crate is a noop on wasm32)"

requirements-completed:
  - CHR-06

duration: 18min
completed: 2026-05-25
---

# Phase 53 Plan 04: GP Serde Checkpoint (Wave 3) Summary

**serde_stacker 0.1.14 wired into the serde feature; depth-64 right-spine tree round-trips through JSON without stack overflow using serde_stacker::Serializer/Deserializer at the call site**

## Performance

- **Duration:** 18 min
- **Started:** 2026-05-25T09:34:09Z
- **Completed:** 2026-05-25T09:52:00Z
- **Tasks:** 1 (Task 1 was the human-verify checkpoint, already approved; Task 2 is the implementation)
- **Files modified:** 3

## Accomplishments

- Added `serde_stacker = { version = "0.1", optional = true }` to Cargo.toml and wired it into the serde feature
- Added `serde_json` dev-dep with `unbounded_depth` feature, enabling `disable_recursion_limit()` used in the deep-tree test
- Activated `test_serde_deep_tree` with full implementation: builds depth-64 right-spine `Node<TestNode>` tree, serializes via `serde_stacker::Serializer`, deserializes via `serde_stacker::Deserializer` + `disable_recursion_limit`, asserts round-trip depth preserved
- Added `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` to `TestNode` to satisfy the serde bounds on `GpChromosome<TestNode>`
- Extended `Node<N>` rustdoc with a Serde section documenting the call-site serde_stacker pattern and the security note about allocation limits
- wasm32 check passes: `cargo check --target wasm32-unknown-unknown --features serde` compiles 16 crates with zero errors

## Task Commits

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 2 | Add serde_stacker + activate test_serde_deep_tree | bc04d1d | Cargo.toml, src/engines/gp/node.rs, tests/gp.rs |

## Files Created/Modified

- `Cargo.toml` — added `serde_stacker = { version = "0.1", optional = true }` to `[dependencies]`; updated `serde` feature line to include `"dep:serde_stacker"`; updated `serde_json` dev-dep to add `features = ["unbounded_depth"]`
- `src/engines/gp/node.rs` — extended `Node<N>` rustdoc: added Serde section explaining serde_stacker call-site pattern and security note; derive attrs unchanged
- `tests/gp.rs` — added `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` to `TestNode`; replaced `#[ignore] todo!()` stub in `test_serde_deep_tree` with full depth-64 round-trip test using serde_stacker wrappers

## Decisions Made

- **serde_stacker applied at call site, not inside impls.** The `serde_stacker::Serializer` and `serde_stacker::Deserializer` are adapter wrappers around an underlying format serializer (serde_json). They cannot be used inside a `Serialize`/`Deserialize` trait impl because the impl receives only the abstract `Serializer`/`Deserializer` trait object — not the concrete serde_json type. The correct pattern is to wrap serde_json's concrete types at the usage site, then pass the serde_stacker adapter as the serializer. This means the test owns the call-site pattern; library users who checkpoint deep trees must apply the same pattern.

- **serde_json unbounded_depth required.** Without this feature on serde_json, `disable_recursion_limit()` doesn't exist as a method, and serde_json's internal recursion guard fires at 128 nesting levels before serde_stacker can take over. A depth-64 `Node` tree produces ~128 JSON nesting levels (struct + array per layer), exactly hitting the default limit. Adding `features = ["unbounded_depth"]` to the dev-dep is the correct fix — this is a dev-dep change only, not visible to library users.

- **Node<N> retains derived serde.** The plan initially suggested replacing derive with manual impls calling serde_stacker. Investigation showed serde_stacker exports `Serializer` and `Deserializer` struct wrappers, not free functions. Manual impls inside the trait cannot call these wrappers because they require the concrete serde_json type. Keeping derive is correct and the call-site pattern is the canonical approach per dtolnay's documentation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] serde_stacker API is wrappers, not free functions**
- **Found during:** Task 2 (first `cargo check --features serde`)
- **Issue:** The plan's interfaces section described `serde_stacker::serialize(self, serializer)` and `serde_stacker::deserialize(deserializer)` as free functions. The actual crate exports `serde_stacker::Serializer` and `serde_stacker::Deserializer` wrapper structs — no free functions exist. The planned approach of replacing `#[derive(Serialize, Deserialize)]` with manual impls calling these free functions would not compile.
- **Fix:** Kept `#[derive(Serialize, Deserialize)]` on `Node<N>`. Implemented the call-site wrapper pattern in `test_serde_deep_tree`: wrap `serde_json::Serializer` with `serde_stacker::Serializer`, and wrap `serde_json::Deserializer` (after `disable_recursion_limit()`) with `serde_stacker::Deserializer`.
- **Files modified:** `src/engines/gp/node.rs` (rustdoc updated), `tests/gp.rs` (test implemented with correct API)
- **Verification:** `cargo test --features serde --test gp test_serde_deep_tree` passes
- **Committed in:** bc04d1d

**2. [Rule 3 - Blocking] serde_json requires unbounded_depth for disable_recursion_limit()**
- **Found during:** Task 2 (test compilation, then test execution)
- **Issue:** `json_de.disable_recursion_limit()` is gated behind `#[cfg(feature = "unbounded_depth")]` in serde_json. Without it, the method doesn't exist (compile error on `--tests`). After adding the method, the test at depth 64 hit serde_json's 128-level recursion limit at runtime (error: "recursion limit exceeded").
- **Fix:** Updated dev-dependency from `serde_json = "1"` to `serde_json = { version = "1", features = ["unbounded_depth"] }`. This is a dev-dep only change — library users are unaffected.
- **Files modified:** `Cargo.toml`
- **Verification:** `cargo check --features serde --tests` exits 0; `test_serde_deep_tree` passes
- **Committed in:** bc04d1d (same commit)

---

**Total deviations:** 2 auto-fixed (1 API mismatch in research, 1 missing feature flag)
**Impact on plan:** Both auto-fixes required for correctness. The serde_stacker API mismatch was a research error — the actual crate is correctly implemented and the call-site pattern is canonical per dtolnay's documentation. No scope creep.

## WASM Compatibility

`cargo check --target wasm32-unknown-unknown --features serde` compiled 16 crates with zero errors. The `stacker` crate (serde_stacker's internal dependency for stack growth) is a noop on wasm32 — no OS stack manipulation is performed on that target.

## Known Stubs

None. `test_serde_deep_tree` is fully implemented and passing.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes. The serde_stacker dependency is gated behind the `serde` optional feature — it is not compiled in default builds. Threat model items from the plan:

- **T-53-11 (DoS via malicious JSON):** Documented in `Node<N>` rustdoc — checkpoint files should be treated as trusted input; deserialization does not bound allocation.
- **T-53-12 (supply chain):** Human verify checkpoint was approved by the user before this task executed. Package confirmed as dtolnay's work.
- **T-53-SC:** Resolved by human checkpoint approval.

## Self-Check: PASSED

Files exist:
- FOUND: Cargo.toml (modified — serde_stacker dep added)
- FOUND: src/engines/gp/node.rs (modified — rustdoc updated)
- FOUND: tests/gp.rs (modified — test_serde_deep_tree activated)

Commits exist:
- FOUND: bc04d1d (Task 2 — serde_stacker + test)

Tests:
- `cargo test --features serde --test gp`: 16 passed, 0 failed, 0 ignored
- `cargo check --target wasm32-unknown-unknown --features serde`: 0 errors
- `cargo clippy --features serde -- -D warnings`: clean

## Next Phase Readiness

- Phase 53 (all 4 waves) is complete: `Node<N>`, `GpChromosome<N>`, `GpCrossover`, `GpMutation`, `GpGa`, and serde checkpoint support are all implemented and tested
- CHR-06 satisfied: GP runs can be checkpointed and restored; depth-64 trees validated in CI
- The pre-existing flaky test `observe::test_serde::ga_run_with_save_progress_creates_checkpoint_files` was observed to fail when run in the full suite (with `--features serde`) but pass in isolation — this is unrelated to Wave 3 changes and was failing before any Wave 3 modifications
