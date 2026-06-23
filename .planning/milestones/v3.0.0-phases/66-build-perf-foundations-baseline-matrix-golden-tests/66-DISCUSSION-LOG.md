# Phase 66: Build-perf foundations — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-13
**Phase:** 66-build-perf-foundations-baseline-matrix-golden-tests
**Areas discussed:** Feature-matrix trigger scope, build-perf-gate CI cost, Golden test format, Baseline JSON schema

---

## Feature-matrix trigger scope

### When to run

| Option | Description | Selected |
|--------|-------------|----------|
| Push to main/milestone branches only | Runs after merge; keeps per-PR CI fast. | ✓ |
| On every PR | Maximum safety; 8-10 min per PR. | |
| On every PR, non-blocking | Informational only; doesn't fail PR checks. | |

### wasm32 matrix entry

| Option | Description | Selected |
|--------|-------------|----------|
| cargo check only (matches existing wasm-check.yml) | Standard project pattern, no new tooling. | ✓ |
| cargo test via wasm-pack | Requires wasm-pack + headless browser setup. | |

**User's choice:** Push to main/milestone only; wasm32 uses `cargo check`.
**Notes:** User specified "el que permita compilar y ejecutar los tests antes, y que sea el estándar" (the standard approach that allows compilation first) — interpreted as `cargo check`, matching the existing `wasm-check.yml` pattern.

---

## build-perf-gate CI cost

### When to run

| Option | Description | Selected |
|--------|-------------|----------|
| Run only on milestone/main branches | Gate after merge; PRs stay fast. | |
| Run on every PR with manual trigger | workflow_dispatch; opt-in. | |
| Run on every PR unconditionally | Maximum safety, as specified in SC#5. | ✓ |

### sccache interaction

| Option | Description | Selected |
|--------|-------------|----------|
| Disable sccache for build-perf-gate job | Measures true cold build time. Other jobs keep sccache. | ✓ |
| Keep sccache, measure warm-cache time | Faster but less representative as a cold baseline. | |
| Dedicated self-hosted runner | Most accurate; adds infrastructure complexity. | |

**User's choice:** Every PR, unconditional; sccache disabled for that job only.
**Notes:** SC#5 says "every PR" — user confirmed this is intentional despite the cost. Sccache disabled specifically for the gate job to preserve cold-build measurement accuracy.

---

## Golden test format

### Storage format

| Option | Description | Selected |
|--------|-------------|----------|
| Separate .txt files read at test runtime | One file per example; editing expected values = editing .txt. | ✓ |
| Inline as Rust constants | Requires Rust recompile to update expected values. | |
| JSON file per example | Structured but heavier than plain text. | |

### Seeding approach

| Option | Description | Selected |
|--------|-------------|----------|
| Add --seed CLI arg to each example | Standard binary interface; golden tests pass --seed 42. | ✓ |
| Use GA_SEED env var | No binary interface change; tests set env var. | |
| Dedicated golden test binaries | Don't touch existing examples. | |

**User's choice:** .txt files; --seed arg added to 4 examples.
**Notes:** --seed is opt-in; existing behavior (random seed when omitted) unchanged.

---

## Baseline JSON schema

### Field structure

| Option | Description | Selected |
|--------|-------------|----------|
| Flat metrics object | `{"dev_build_s": 42.1, "dep_count": 187, ...}` — simple, jq-diffable. | ✓ |
| Nested by category | `{"build": {"dev_s": 42.1}, ...}` — structured but harder to diff. | |
| Per-metric with tolerance inline | `[{"name": "dev_build_s", "tolerance_pct": 2}]` — flexible but heavy. | |

### Regression tolerance

| Option | Description | Selected |
|--------|-------------|----------|
| 2% for timing, 0% for counts | Split by metric type. Counts are exact; timing absorbs CI noise. | ✓ |
| 2% uniform | Simple rule; dep_count ±1 would pass (~0.5% of 187). | |

**User's choice:** Flat object; 2% timing / 0% counts.
**Notes:** public_api_hash uses 0% tolerance (exact match) — any API surface change must fail explicitly.

---

## Claude's Discretion

None — all areas had clear user selections.

## Deferred Ideas

None — discussion stayed within Phase 66 scope.
