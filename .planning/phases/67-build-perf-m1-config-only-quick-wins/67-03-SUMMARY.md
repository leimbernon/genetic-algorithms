---
phase: 67-build-perf-m1-config-only-quick-wins
plan: 03
subsystem: infra
tags: [build-perf, ci, linker, mold, lld, cargo-config]

# Dependency graph
requires:
  - phase: 67-build-perf-m1-config-only-quick-wins plan 01
    provides: docs/DEVELOPMENT.md with Cargo profiles section (this plan appends Linker recommendations after it)
  - phase: 67-build-perf-m1-config-only-quick-wins plan 02
    provides: nextest CI step in rust-unit-tests.yml (mold install added after nextest install)
provides:
  - .cargo/config.toml Linux mold block + commented macOS lld block; existing WASM block preserved verbatim
  - mold apt-install step in rust-unit-tests.yml, coverage.yml, rust-clippy.yml, examples-smoke.yml
  - docs/DEVELOPMENT.md Linker recommendations section (Linux/macOS/Windows/Reverting)
  - CHANGELOG.md Changed entry for mold CI adoption
affects: [67-04-sccache, future-ci-plans]

# Tech tracking
tech-stack:
  added: [mold (system package via apt), lld (documented macOS opt-in via brew)]
  patterns:
    - .cargo/config.toml target-specific linker config (linker + rustflags per target)
    - apt-install step placed before first cargo invocation in each Linux CI workflow

key-files:
  created: []
  modified:
    - .cargo/config.toml
    - .github/workflows/rust-unit-tests.yml
    - .github/workflows/coverage.yml
    - .github/workflows/rust-clippy.yml
    - .github/workflows/examples-smoke.yml
    - docs/DEVELOPMENT.md
    - CHANGELOG.md

key-decisions:
  - "linker = clang required because gcc on some Ubuntu versions does not reliably forward -fuse-ld=mold"
  - "wasm-check.yml intentionally NOT modified (wasm32 uses LLVM cross-compile chain, not ELF mold — Anti-Pattern per RESEARCH.md)"
  - "build-perf-gate.yml intentionally NOT modified (Pitfall 4: adding mold install would corrupt cold-build timing measurements)"
  - "macOS lld block left commented — requires brew install llvm; not enforced in CI; opt-in only"
  - "Windows: documented local ~/.cargo/config.toml override only; no Windows CI to enforce it"

patterns-established:
  - "Target-specific linker: .cargo/config.toml [target.X] block with linker + rustflags keys"
  - "CI mold install: single apt-get step before first cargo invocation, no version pinning (Ubuntu 22.04+ ships mold)"

requirements-completed: []

# Metrics
duration: 18min
completed: 2026-06-14
---

# Phase 67 Plan 03: Mold Linker Configuration Summary

**mold set as the Linux linker via .cargo/config.toml [target.x86_64-unknown-linux-gnu] and apt-installed in four CI workflows; macOS lld opt-in and developer guide appended**

## Performance

- **Duration:** 18 min
- **Started:** 2026-06-14T15:30:00Z
- **Completed:** 2026-06-14T15:48:00Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- `.cargo/config.toml` extended with Linux mold block (`linker = "clang"`, `-fuse-ld=mold`) and commented-out macOS lld block; existing `[target.wasm32-unknown-unknown]` rustflags block preserved verbatim (D-08)
- Four Linux CI workflows (`rust-unit-tests.yml`, `coverage.yml`, `rust-clippy.yml`, `examples-smoke.yml`) each gained an `Install mold linker` step before the first `cargo` invocation (D-07)
- `docs/DEVELOPMENT.md` gained a `## Linker recommendations` section with four subsections: Linux, macOS, Windows, Reverting
- `CHANGELOG.md` updated with a Changed entry referencing mold adoption in CI (Phase 67 / Plan 67-03)

## Task Commits

Each task was committed atomically:

1. **Task 1: .cargo/config.toml Linux mold + macOS lld blocks** - `93f4735` (chore)
2. **Task 2: mold apt-install in four Linux CI workflows** - `a0b79e5` (ci)
3. **Task 3: DEVELOPMENT.md Linker recommendations + CHANGELOG** - `c684acf` (docs)

## Files Created/Modified

- `.cargo/config.toml` - Added [target.x86_64-unknown-linux-gnu] block (linker=clang, fuse-ld=mold) and commented [target.aarch64-apple-darwin] lld block; WASM block preserved
- `.github/workflows/rust-unit-tests.yml` - Added `Install mold linker` step after nextest install, before Build
- `.github/workflows/coverage.yml` - Added `Install mold linker` step after cache, before cargo-llvm-cov install
- `.github/workflows/rust-clippy.yml` - Added `Install mold linker` step after rust-cache, before clippy-sarif install
- `.github/workflows/examples-smoke.yml` - Added `Install mold linker` step after cache, before Run example
- `docs/DEVELOPMENT.md` - Appended ## Linker recommendations section (after ## Cargo profiles) with Linux/macOS/Windows/Reverting subsections
- `CHANGELOG.md` - Appended Changed bullet referencing mold and Phase 67 / Plan 67-03

## Intentional Non-Touches (important for audit)

- **`wasm-check.yml` NOT modified** — wasm32-unknown-unknown compiles via LLVM cross-compile chain, not ELF mold. Adding mold install would be pointless and could confuse future maintainers. Anti-Pattern per RESEARCH.md.
- **`build-perf-gate.yml` NOT modified** — this workflow measures cold-build timing baselines. Adding mold would skew all future timing measurements, invalidating the baseline established in Phase 66. Pitfall 4 per RESEARCH.md.

## Decisions Made

- `linker = "clang"` required (not `gcc`) because gcc on some Ubuntu versions silently ignores the `-fuse-ld` flag; clang consistently forwards it.
- macOS lld block kept commented-out by default: requires `brew install llvm`, no macOS CI to enforce it, and the performance gain (~5 %) is opt-in for developers who want it locally.
- Windows documented as a user-local `~/.cargo/config.toml` override only (no Windows CI); no repository-committed Windows config to avoid surprises for developers who do not have `lld-link`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Threat Surface Scan

No new network endpoints, auth paths, or trust boundaries introduced. The `apt-get install -y mold` step uses the Ubuntu apt-signed repository — same trust model as `apt-get install gcc`. The T-67-03-T acceptance criteria (exact key/value strings + WASM block preservation) were verified by grep assertions.

## User Setup Required

None — no external service configuration required. macOS developers who want the lld opt-in can run `brew install llvm` and uncomment the block in `.cargo/config.toml`.

## Next Phase Readiness

- Plan 67-04 (sccache) can proceed; it touches different workflow steps (sccache install + env vars, not mold)
- WASM build confirmed unaffected (cargo check exits 0)
- All five workflow files validated as syntactically correct YAML

---
*Phase: 67-build-perf-m1-config-only-quick-wins*
*Completed: 2026-06-14*
