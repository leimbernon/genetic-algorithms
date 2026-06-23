# Phase 66: Build-perf foundations (baseline + matrix + golden tests) - Context

**Gathered:** 2026-06-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish three measurement and regression-prevention infrastructure pieces — a build-perf shell script with a committed baseline JSON, a feature-matrix CI workflow, and golden regression tests for four reference examples — that gate Phases 67-69. No source-code behavioral change. No new library features. Pure tooling and CI infrastructure.

</domain>

<decisions>
## Implementation Decisions

### Feature-matrix CI trigger
- **D-01:** `feature-matrix.yml` runs on push to `main` and `milestone/**` branches only — NOT on every PR. Rationale: 8 parallel `cargo test` combinations (default, serde, visualization, benchmarks, observer-tracing, observer-metrics, all-features, wasm32) add 8-10 min of CI; that cost belongs at merge time, not during review.
- **D-02:** The wasm32 matrix entry uses `cargo check --target wasm32-unknown-unknown` — consistent with the existing `wasm-check.yml` pattern. No wasm-pack or browser test harness added.

### build-perf-gate cost and sccache interaction
- **D-03:** `build-perf-gate` CI job runs on EVERY PR unconditionally (per SC#5). This matches the BUILD-PERF.md spec exactly.
- **D-04:** sccache is DISABLED for the `build-perf-gate` job specifically. The gate does `cargo clean && cargo build` without `RUSTC_WRAPPER=sccache` to measure true cold build time. All other CI jobs retain sccache (Phase 67 decision).

### Golden test format
- **D-05:** Expected values stored as separate `.txt` files in `tests/golden/`: one file per example (`rastrigin.txt`, `nsga2_zdt1.txt`, `cma_es_rastrigin.txt`, `pso_rastrigin.txt`). Each file contains the best-fitness value to 12 decimal places. The Rust test reads the file and asserts equality. Updating expected values = editing text files with no Rust recompile.
- **D-06:** Each of the 4 examples gets a `--seed <u64>` CLI argument (via `std::env::args` parsing or a minimal clap arg) that calls `set_seed(seed)` when provided. Golden tests invoke `cargo run --example <name> --release -- --seed 42`. The seed flag is opt-in; existing behavior (random seed) is unchanged when `--seed` is absent.
- **D-07:** The 4 reference examples for golden tests are: `rastrigin`, `nsga2_zdt1`, `cma_es_rastrigin`, `pso_rastrigin` (one per engine family as specified in BUILD-PERF.md §CC-1).

### Baseline JSON schema
- **D-08:** `.planning/baselines/v3.0.0-baseline.json` uses a flat metrics object:
  ```json
  {
    "dev_build_s": 42.1,
    "wasm_check_s": 12.3,
    "test_suite_s": 67.4,
    "dep_count": 187,
    "public_api_hash": "abc123",
    "captured_at": "2026-06-13"
  }
  ```
  Simple, jq-diffable, one field per metric.
- **D-09:** Regression tolerance is split by metric type:
  - **Timing metrics** (`dev_build_s`, `wasm_check_s`, `test_suite_s`): 2% tolerance to absorb CI noise.
  - **Count/exact metrics** (`dep_count`, `public_api_hash`): 0% tolerance — any change is intentional and must fail the gate. The gate script encodes this as a per-metric comparison rule.

### build_perf.sh script
- **D-10:** `bench/build_perf.sh` runs the full measurement suite per BUILD-PERF.md §CC-1: `cargo clean && cargo build --timings` (default features), `cargo check --target wasm32-unknown-unknown`, `cargo test --quiet`, bench quick run, `cargo tree | sort -u | wc -l`, `cargo public-api` snapshot, and golden example runs.
- **D-11:** Script writes intermediate data to `target/build-perf/` (gitignored) and emits the final committed snapshot to `.planning/baselines/v3.0.0-baseline.json`. The `target/` output is ephemeral; the `.planning/baselines/` file is the canonical committed record.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary specification
- `.planning/v3.0.0-BUILD-PERF.md` §Cross-cutting work (CC-1, CC-2, CC-3) — Full script spec, workflow requirements, golden test requirements, baseline schema, and the "Non-negotiable guarantees" that ALL phases 66-69 must honor. MUST READ.
- `.planning/ROADMAP.md` §Phase 66 — Success criteria and 3-plan structure (Wave 0: script+baseline; Wave 1: matrix CI + golden+gate in parallel).

### Existing CI to understand and extend
- `.github/workflows/rust-unit-tests.yml` — The primary test workflow. `build-perf-gate` job is added here or as a separate workflow.
- `.github/workflows/wasm-check.yml` — The reference for `cargo check --target wasm32-unknown-unknown` pattern used by the matrix wasm32 entry.
- `.cargo/config.toml` — Contains existing `[target.wasm32-unknown-unknown]` rustflags. Feature-matrix wasm32 job must pass these same flags.

### Existing examples to instrument with --seed
- `examples/rastrigin.rs` — Add `--seed` arg; golden test baseline.
- `examples/nsga2_zdt1.rs` — Add `--seed` arg; golden test baseline.
- `examples/cma_es_rastrigin.rs` — Add `--seed` arg; golden test baseline.
- `examples/pso_rastrigin.rs` — Add `--seed` arg; golden test baseline.

### Existing RNG seeding API
- `src/rng.rs` — Contains `set_seed(seed: u64)` function; examples will call this.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `set_seed(seed: u64)` in `src/rng.rs` — Already exists. Examples call this after parsing the `--seed` arg. No new RNG infrastructure needed.
- Existing CI workflow structure in `.github/workflows/` — 5 workflows exist. `feature-matrix.yml` is a new file; `build-perf-gate` job can be added to an existing workflow or created as `build-perf-gate.yml`.

### Established Patterns
- `cargo check --target wasm32-unknown-unknown` with `getrandom_backend="wasm_js"` rustflag — already in `.cargo/config.toml` and `wasm-check.yml`. Feature-matrix wasm32 entry inherits this.
- `tests/` directory holds all unit and integration tests — `tests/golden/` follows the same convention.
- BUILD-PERF.md §Non-negotiable guarantees — must be re-read before planning any plan in this phase. All 6 guarantees apply.

### Integration Points
- `bench/build_perf.sh` → `.planning/baselines/v3.0.0-baseline.json` — script produces committed artifact. Plan 66-01 captures the initial baseline; Phase 67+ plans diff against it.
- `tests/golden/*.txt` → `tests/` test module — Golden tests are regular `#[test]` functions that run as part of `cargo test`. No separate test binary or harness needed.
- `build-perf-gate` CI job → must NOT have sccache (D-04) even though Phase 67 adds sccache to all other workflows.

</code_context>

<specifics>
## Specific Ideas

- Baseline JSON field names exactly as in D-08: `dev_build_s`, `wasm_check_s`, `test_suite_s`, `dep_count`, `public_api_hash`, `captured_at`.
- Tolerance rules in D-09: 2% for timing, 0% (exact) for dep_count and public_api_hash.
- `--seed 42` is the canonical seed value for all golden tests.
- `feature-matrix.yml` trigger: `on: push: branches: [main, "milestone/**"]`.
- `build-perf-gate` job runs on every PR but disables sccache via omitting `RUSTC_WRAPPER=sccache` (or setting it to empty) for that job specifically.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within Phase 66 scope. The tolerance-per-metric config (D-09) is implemented inline in `bench/build_perf.sh`; a full configurable-per-metric tolerance system belongs in a later iteration if needed.

</deferred>

---

*Phase: 66-build-perf-foundations-baseline-matrix-golden-tests*
*Context gathered: 2026-06-13*
