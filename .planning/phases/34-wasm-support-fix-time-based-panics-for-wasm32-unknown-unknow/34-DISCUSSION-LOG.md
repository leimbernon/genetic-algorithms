# Phase 34: WASM support — fix time-based panics for wasm32-unknown-unknown - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-07
**Phase:** 34-wasm-support-fix-time-based-panics-for-wasm32-unknown-unknow
**Areas discussed:** Detection mechanism, Time resolution strategy, Rayon parallelism, max_duration_secs behavior

---

## Detection Mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| cfg(target_arch = "wasm32") — auto | No user config needed. Works automatically when cross-compiling to wasm32. Follows Rust ecosystem conventions (same as getrandom, rand). Zero friction. | ✓ |
| wasm feature flag | Users add features = ["wasm"] in their Cargo.toml. Explicit opt-in, adds friction. Only useful if the feature also pulls in wasm-specific deps. | |
| Both: cfg(target_arch) for no-dep fixes, wasm flag for optional web-time | Auto-fix the panics unconditionally, but add a `wasm` feature for actual browser timing. More complex. | |

**User's choice:** cfg(target_arch = "wasm32") — automatic detection, no feature flag
**Notes:** Issue reporter suggested a feature flag, but the user preferred the automatic approach for zero friction.

---

## Time Resolution Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Stub zeros (Duration::ZERO) — no new deps | All elapsed() calls return Duration::ZERO on WASM. Simple, zero dependencies. | ✓ |
| Add web-time crate | WASM-compatible Instant using performance.now(). Actual timing data but adds a dependency. | |

**Sub-question — DurationReporter on WASM:**

| Option | Description | Selected |
|--------|-------------|----------|
| Compile, report Duration::ZERO | Same public API on all targets. DurationReporter reports 0ms on WASM. Non-breaking. | ✓ |
| cfg out DurationReporter on WASM | Cleaner semantics but IS a breaking change — any user referencing DurationReporter in WASM-targeted code gets a compile error. | |

**User's choice:** Stub zeros with DurationReporter kept in API (reports Duration::ZERO)
**Notes:** User explicitly asked about breaking changes before deciding; chose the non-breaking path. web-time deferred for future consideration.

---

## Rayon Parallelism

| Option | Description | Selected |
|--------|-------------|----------|
| Defer rayon — time panics only | Phase title says "time-based panics" — fix only Instant::now(). Rayon is a separate, larger problem. | |
| Address rayon too | Wrap rayon parallel loops with cfg so WASM falls back to sequential iteration. Broader WASM support. | ✓ |

**Sub-question — which engines:**

| Option | Description | Selected |
|--------|-------------|----------|
| GA + NSGA-II only | The two main engines users actually run in browsers. | |
| All 6 engines | GA, NSGA-II, DE, Scatter, Cellular, ALPS all get cfg-gated sequential fallback. | ✓ |

**User's choice:** Address rayon in all 6 engines
**Notes:** Despite the phase title focusing on time panics, user chose to also fix rayon for a complete WASM story.

---

## max_duration_secs Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Silent ignore | max_duration_secs is silently ignored on WASM. No surprises at runtime. | |
| log::warn at runtime | Emit a one-time log::warn! when the limit is configured but the target has no clock. | ✓ |

**User's choice:** log::warn! at runtime
**Notes:** User wants feedback without a panic.

---

## Claude's Discretion

- Exact placement of `cfg` guards (module-level `use` vs inline at each call site)
- Whether to restructure `use std::time::Instant` to avoid dead-code warnings on WASM

## Deferred Ideas

- `web-time` crate integration for real `performance.now()` timing in WASM
- Island model WASM support (thread-based channels — separate problem)
- wasm-bindgen JS bindings / public API surface
- WASM-specific example (`examples/wasm_onemax.rs`)
