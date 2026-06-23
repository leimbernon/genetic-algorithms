# Phase 67: Build-perf M1 — config-only quick wins - Context

**Gathered:** 2026-06-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Land four zero-risk, config-only build improvements — Cargo profile tuning, cargo-nextest CI swap, mold linker config, and sccache CI caching — that shave 5-15% off clean dev wall-clock and 30-60% off CI wall-clock. No source-code behavioral change. Four plans run in parallel (Wave 0 in ROADMAP).

**BLOCKER: Phase 67 MUST NOT start until Phase 66 is complete.** Phase 66 delivers the baseline harness (CC-1), feature-matrix CI (CC-2), and golden tests (CC-3) that Phase 67 needs for success criterion #5 verification. Plan Phase 66 first, then return here.

</domain>

<decisions>
## Implementation Decisions

### Phase Ordering
- **D-01:** Plan and execute Phase 66 before Phase 67. Phase 67 CONTEXT.md is ready, but planning (PLAN.md files) and execution MUST wait until Phase 66 is shipped. SC#5 (build-perf-gate ≥5% improvement vs Phase 66 baseline) is unverifiable without it.

### Nextest scope
- **D-02:** Add `cargo nextest run` to ALL three test-running CI workflows: `rust-unit-tests.yml`, `coverage.yml`, and `wasm-check.yml`.
- **D-03:** `coverage.yml` uses `cargo-llvm-cov` — compatible with nextest via `cargo llvm-cov nextest`. Safe to include.
- **D-04:** Local `cargo test` remains unchanged — no developer install requirement.

### sccache scope + version
- **D-05:** Add `mozilla-actions/sccache-action@v0.0.9` (latest release, not v0.0.4 from BUILD-PERF.md which is stale) to ALL CI workflows that invoke rustc: `rust-unit-tests.yml`, `wasm-check.yml`, `coverage.yml`, `rust-clippy.yml`, `examples-smoke.yml`.
- **D-06:** Set `RUSTC_WRAPPER=sccache` env var in each workflow step. Log cache hit-rate per BUILD-PERF.md spec.

### mold linker installation
- **D-07:** Install mold via `apt-get install -y mold` in EVERY Linux CI workflow (not just `rust-unit-tests.yml`). The 5s overhead is paid per job; the net win is still positive and the approach is consistent.
- **D-08:** `.cargo/config.toml` gets the mold block for `x86_64-unknown-linux-gnu` AND a commented-out lld block for `aarch64-apple-darwin` (documents the opt-in macOS path per BUILD-PERF.md spec). The existing WASM `rustflags` block is preserved unchanged.

### Cargo profiles
- **D-09:** Add to `Cargo.toml` exactly the three blocks from BUILD-PERF.md §Action #5/#6:
  - `[profile.dev]` — `debug = "line-tables-only"`, `split-debuginfo = "unpacked"`
  - `[profile.dev.package."*"]` — `opt-level = 1`, `debug = false`
  - `[profile.test]` — `opt-level = 1`
  No deviation from the spec.

### Documentation deliverables (per plan)
- **D-10:** 67-01 writes `docs/DEVELOPMENT.md` "Cargo profiles" section + `CHANGELOG.md` Changed entry + `.planning/intel/build-profile.md`.
- **D-11:** 67-02 writes `docs/TESTING.md` nextest opt-in instructions + `CHANGELOG.md` Changed (internal) entry.
- **D-12:** 67-03 writes `docs/DEVELOPMENT.md` "Linker recommendations" section + `CHANGELOG.md` Changed (CI) entry.
- **D-13:** 67-04 writes `docs/DEVELOPMENT.md` "CI caching" subsection (no user action needed; informational).
- **D-14:** Every plan commit body MUST include a `Revert plan:` line per BUILD-PERF.md non-negotiable guarantee #5.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary specification
- `.planning/v3.0.0-BUILD-PERF.md` §Wave M1 — Exact TOML blocks, doc deliverables, revert plans, and acceptance gates for all 4 plans. This is the authoritative spec. MUST READ.
- `.planning/ROADMAP.md` §Phase 67 — Success criteria, plan list (4 parallel plans), dependency on Phase 66.

### Existing config to extend (not replace)
- `.cargo/config.toml` — Already has `[target.wasm32-unknown-unknown]` rustflags block. New linker blocks are ADDITIVE. Do not remove or modify the existing WASM entry.

### CI workflows to modify
- `.github/workflows/rust-unit-tests.yml` — Gets nextest + sccache + mold (Linux)
- `.github/workflows/coverage.yml` — Gets nextest (`cargo llvm-cov nextest`) + sccache + mold
- `.github/workflows/wasm-check.yml` — Gets nextest + sccache
- `.github/workflows/rust-clippy.yml` — Gets sccache + mold (Linux)
- `.github/workflows/examples-smoke.yml` — Gets sccache + mold (Linux)

### Non-negotiable guarantees
- `.planning/v3.0.0-BUILD-PERF.md` §Non-negotiable guarantees — All 6 guarantees apply to every plan. Zero behavioral regression, zero public-API regression, all feature combos green, zero rustdoc warnings, reversibility, measurement before and after.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `.cargo/config.toml` — Exists with WASM rustflags. Linker plan (67-03) adds to it, does not replace it.

### Established Patterns
- WASM cfg gate pattern (`#[cfg(not(target_arch = "wasm32"))]`) — Profile tuning and linker changes are config-level; they don't affect WASM source gating. WASM check CI must stay green after every plan.
- All CI workflows use `ubuntu-latest` (which is `ubuntu-22.04` at the time of writing) for Linux jobs — mold must be installed via `apt-get` in each workflow that needs it.

### Integration Points
- Phase 66 baseline (`bench/build_perf.sh` + `.planning/baselines/v3.0.0-baseline.json`) — Phase 67 plans diff against it for SC#5. Do not execute Phase 67 plans before this file exists.
- Golden tests (`tests/golden/`) — CC-3 from Phase 66. Every Phase 67 plan must run these and confirm byte-identical output.

</code_context>

<specifics>
## Specific Ideas

- `cargo-llvm-cov nextest` is the correct invocation for coverage.yml after the nextest swap — not `cargo nextest run` standalone.
- sccache action version: `v0.0.9` (not the `v0.0.4` listed in BUILD-PERF.md).
- mold install in every Linux CI job — consistent pattern, not just the unit-test workflow.
- Commented-out macOS lld block in `.cargo/config.toml` — document the opt-in but don't activate it by default.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. Phase 68 (dep hygiene) and Phase 69 (major refactors) take the next build-perf steps.

</deferred>

---

*Phase: 67-build-perf-m1-config-only-quick-wins*
*Context gathered: 2026-06-13*
