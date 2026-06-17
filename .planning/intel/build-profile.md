# Build profile rationale (Phase 67 / Action #5/#6)

This file explains why `Cargo.toml` declares three `[profile.*]` blocks and provides
context for AI agents so the tuning is not silently reverted in a future refactor.

Source authority: `.planning/v3.0.0-BUILD-PERF.md §Action #5` and `§Action #6`.

---

## DO NOT REMOVE these profile blocks

The three `[profile.dev]`, `[profile.dev.package."*"]`, and `[profile.test]` sections at
the end of `Cargo.toml` were added in Phase 67 / Plan 67-01 after a systematic
build-performance baseline measurement (`.planning/baselines/v3.0.0-baseline.json`,
established in Phase 66). They delivered a measured improvement of ≥5 % on dev-build
wall-clock and approximately 50 % reduction in `cargo test` runtime.

If you are an AI agent performing a refactor and you are tempted to:
- Clean up "unused" sections at the bottom of `Cargo.toml`
- Normalise the TOML file by removing blocks that look redundant
- Replace the profile blocks with some other mechanism

STOP. Read this file first. The blocks are intentional, measured, and anchored to CI.

---

## Why each key is set the way it is

### [profile.dev] — `debug = "line-tables-only"`

Cargo's default `debug = true` (or `debug = 2`) emits full DWARF info. Full DWARF causes
the linker to process and emit a large `.debug_*` section on every incremental build, even
for tiny changes. `"line-tables-only"` emits only DWARF line-number tables, which are
sufficient for file/line backtraces and `RUST_BACKTRACE=1` but a fraction of the size.

Impact: linker is faster on every incremental build. No loss of debuggability for most
development workflows.

Source: `.planning/v3.0.0-BUILD-PERF.md §Action #5`.

### [profile.dev] — `split-debuginfo = "unpacked"`

On macOS, Cargo's default `split-debuginfo = "packed"` causes cargo to invoke `dsymutil`
after every link step. `dsymutil` walks all object files and consolidates debug info into a
`.dSYM` bundle. This is the single most expensive non-compile step on Apple Silicon and
Intel Macs. Setting `"unpacked"` skips `dsymutil` for dev builds — debug info remains in
the object files, which is fine for local debugging (LLDB can still find symbols).

On Linux, this key has no effect (the default is already `"unpacked"`).

Source: `.planning/v3.0.0-BUILD-PERF.md §Action #5`.

### [profile.dev.package."*"] — `opt-level = 1` and `debug = false`

This section applies only to dependencies (the glob `"*"` matches all crate names except
the root crate). It compiles third-party code at `-O1` instead of `-O0`.

Why this matters: the GA library uses `rand` for all random sampling, `rayon` for parallel
iteration, and `log`/`serde` in hot paths. Under `-O0`, these crates emit unoptimised code
that is roughly 3-8x slower at runtime than their optimised counterparts. Because tests
exercise many GA generations (hundreds to thousands), slow deps inflate `cargo test` time
significantly.

`debug = false` for deps suppresses dep debug symbols, reducing the linker's input size.

One-time cost: the first clean build with these settings takes ~5-10 s longer than without
them, because deps must be recompiled with `-O1`. After that, Cargo caches the optimised
dep artifacts and incremental builds are no slower than before.

Source: `.planning/v3.0.0-BUILD-PERF.md §Action #6`.

### [profile.test] — `opt-level = 1`

Test binaries run the full GA execution loop (selection, crossover, mutation, survivor,
stats). Many test cases run for hundreds of generations. Under `-O0`, the test binary itself
is slow. A single `-O1` pass cuts test runtime by approximately 50 % with negligible
rustc-time overhead (the optimisation passes at `-O1` are cheap: inlining, constant folding,
dead-code elim).

This setting does NOT affect debuggability of the code under test in any meaningful way for
the typical test-failure debugging workflow — `cargo test -- --nocapture` output is still
fully readable, and panics still show line numbers via the `line-tables-only` setting above.

Source: `.planning/v3.0.0-BUILD-PERF.md §Action #6`.

---

## CI anchor

The Phase 66 baseline is stored at `.planning/baselines/v3.0.0-baseline.json`. The
`build-perf-gate.yml` CI workflow enforces that no PR regresses the dep count above the
baseline. The profile blocks themselves are pure configuration — they do not add crate
dependencies and are invisible to `build-perf-gate`'s dep-count check.

---

## Cross-references

- Canonical TOML spec: `.planning/v3.0.0-BUILD-PERF.md §Action #5` and `§Action #6`
- Contributor-facing rationale: `docs/DEVELOPMENT.md §Cargo profiles`
- Phase 67 plan that added these blocks: `.planning/phases/67-build-perf-m1-config-only-quick-wins/67-01-PLAN.md`
- Phase 67 execution summary: `.planning/phases/67-build-perf-m1-config-only-quick-wins/67-01-SUMMARY.md`
