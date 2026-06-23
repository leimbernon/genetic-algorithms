# Phase 68: Build-perf M2 — dependency hygiene - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-15
**Phase:** 68-build-perf-m2-dependency-hygiene
**Areas discussed:** LogLevel / with_logs() fate, log kv_unstable feature, LogObserver gating strategy

---

## LogLevel / with_logs() fate

| Option | Description | Selected |
|--------|-------------|----------|
| Remove both entirely | Drop LogLevel enum, with_logs() method, and the log_level field from Configuration. Cleanest v3.0 break. MIGRATION.md documents the removal. 1 example affected (memetic_rastrigin.rs). | ✓ |
| Keep for forward-compat | Leave LogLevel and with_logs() in place as silent no-ops. Confusing but avoids any API break beyond env_logger removal. | |

**User's choice:** Remove both entirely
**Notes:** v3.0.0 is the correct moment to remove dead code. Only 1 example uses with_logs() so migration surface is minimal.

---

## log kv_unstable feature

| Option | Description | Selected |
|--------|-------------|----------|
| Keep kv_unstable | Leave LogObserver call sites unchanged. The kv_unstable API has been stable-enough in practice. | ✓ |
| Drop kv_unstable, use standard syntax | Change all LogObserver log calls to standard log::info!(target: "...", "...") format. Removes unstable feature but touches ~20 call sites in log.rs. | |

**User's choice:** Keep kv_unstable
**Notes:** No reformatting work needed in LogObserver. log feature list stays as ["std", "serde", "kv_unstable"].

---

## LogObserver gating strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Gate behind #[cfg(feature = "logging")] | LogObserver disappears when logging is off. Correct since users without logging feature have no log subscriber anyway. | ✓ |
| No-op LogObserver when logging=off | Keep LogObserver always available but all methods become empty. More complex and misleadingly named. | |

**User's choice:** Gate LogObserver behind #[cfg(feature = "logging")]
**Notes:** Cleaner approach. Users building with default-features=false won't have LogObserver — this is intentional and documented.

---

## Claude's Discretion

- Internal macro approach for log call sites (D-04): BUILD-PERF.md recommends `crate::log_info!()` macro family over 109+ inline `#[cfg]` gates. User did not select this gray area for discussion — Claude applied the spec recommendation directly.

## Deferred Ideas

None — discussion stayed within phase scope.
