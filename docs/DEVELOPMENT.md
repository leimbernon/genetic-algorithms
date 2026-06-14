<!-- generated-by: gsd-doc-writer -->
# Development Guide

This guide covers setting up a local development environment, building the project, running lints, and the contribution workflow for the `genetic_algorithms` crate.

## Local Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/leimbernon/rust_genetic_algorithms.git
   cd rust_genetic_algorithms
   ```

2. Ensure you have Rust >= 1.81.0 installed (check with `rustc --version`). Install via [rustup](https://rustup.rs/) if needed.

3. Install required Clippy and rustfmt components:
   ```bash
   rustup component add clippy rustfmt
   ```

4. Build the project to verify the setup:
   ```bash
   cargo build
   ```

No additional configuration files or environment variables are required for local development.

## Build Commands

| Command | Description |
|---------|-------------|
| `cargo build` | Compile the library (default features) |
| `cargo build --all-features` | Compile with all optional features enabled |
| `cargo build --features serde` | Compile with checkpoint serialization support |
| `cargo build --features visualization` | Compile with plotters-based visualization support |
| `cargo build --features observer-tracing` | Compile with tracing integration |
| `cargo build --features observer-metrics` | Compile with metrics integration |
| `cargo test` | Run the full test suite |
| `cargo test --features serde` | Run tests including serde-gated tests |
| `cargo test --features serde,observer-tracing,observer-metrics` | Run tests with all features |
| `cargo clippy --all-targets --all-features` | Lint all targets and features |
| `cargo doc --no-deps` | Generate crate documentation (zero rustdoc warnings required) |
| `cargo bench` | Run all criterion benchmarks |
| `cargo check --target wasm32-unknown-unknown` | Verify WASM compatibility (no extra flags needed — `.cargo/config.toml` handles `getrandom` backend) |
| `cargo check --target wasm32-unknown-unknown --features serde` | Verify WASM + serde |

## Feature Flags

| Flag | Optional Dependencies | Purpose |
|------|-----------------------|---------|
| `serde` | `serde`, `serde_json` | Checkpoint serialization/deserialization |
| `visualization` | `plotters` | Fitness/diversity visualization output |
| `observer-tracing` | `tracing` | Emit GA events via the `tracing` crate |
| `observer-metrics` | `metrics` | Emit GA metrics via the `metrics` crate |

The default feature set is empty — all features are opt-in.

## Code Style

**Clippy** enforces linting across all targets and features. Run it with:
```bash
cargo clippy --all-targets --all-features
```

**rustfmt** is available for formatting:
```bash
cargo fmt
```

Both tools are enforced in CI on every pull request via the `rust-clippy.yml` workflow.

Additional code conventions:
- Never use `panic!` in library code — return `Result<T, GaError>` instead.
- Use `rayon` (`par_iter`, `into_par_iter`) for all parallel work. Never use `std::thread::spawn` or manual `Arc<Mutex<>>`.
- Pass DNA as `Cow<[Gene]>` to avoid unnecessary copies.
- Document all public functions and types with `///` doc-comments. Zero rustdoc warnings are required.
- Use the `log` crate (`debug!`, `trace!`, `info!`) with descriptive targets (e.g., `target="ga_events"`).

## Adding a New Operator

Follow this pattern for any new genetic operator (crossover, mutation, selection, survivor, extension):

1. Create `src/operations/<type>/my_operator.rs` and implement the operator function.
2. Add a variant to the corresponding enum in `src/operations.rs`.
3. Register the variant in the factory `match` in `src/operations/<type>.rs`.
4. Re-export from `src/operations/<type>.rs` with `pub use self::my_operator::my_operator;`.
5. Create a test file at `tests/operations/test_<type>_my.rs`.
6. Register the test module inside the `mod operations { ... }` block in `tests/test_operations.rs`:
   ```rust
   mod operations {
       // ... existing entries ...
       mod test_<type>_my;
   }
   ```
7. Run `cargo test` — all tests must pass.

## Branch Conventions

Branches follow a milestone-scoped hierarchy:

```
main
 └── milestone/<milestone-name>     ← created from main
      ├── feat/<issue-number>-<description>
      └── fix/<issue-number>-<description>
```

- Never branch `feat/` or `fix/` directly from `main`.
- Check whether the target milestone branch exists before creating a feature branch; create it from `main` if it does not.
- PRs target the milestone branch, not `main`.

## PR Process

- Open the PR against the appropriate `milestone/<name>` branch.
- All of the following must pass before merge:
  - `cargo test`
  - `cargo test --features serde`
  - `cargo clippy`
  - `cargo doc --no-deps` (zero rustdoc warnings)
- Every change must include tests — no exceptions. Tests live in `tests/` (never inline with implementation code).
- Stochastic tests (mutation, selection) use retry loops to avoid flakiness.
- New public items require `///` doc-comments.
- No `panic!` in library code; no manual `thread::spawn`.

## Pre-Commit Checklist

- [ ] `cargo test` — all tests pass
- [ ] `cargo build` — no warnings
- [ ] `cargo check --target wasm32-unknown-unknown` — WASM compiles cleanly
- [ ] New or changed code has tests in `tests/`
- [ ] All public functions and types have doc-comments
- [ ] No `panic!` in library code (use `Result<T, GaError>`)
- [ ] No manual `thread::spawn` (use rayon)
- [ ] New `Instant::now()` calls gated with `#[cfg(not(target_arch = "wasm32"))]`
- [ ] New `par_iter()` calls duplicated with `iter()` fallback under `#[cfg(target_arch = "wasm32")]`

## CI Workflows

| Workflow | Trigger | What it runs |
|----------|---------|--------------|
| `rust-unit-tests.yml` | PR targeting `main` | `cargo build --verbose`, `cargo test --verbose` |
| `rust-clippy.yml` | All PRs | `cargo clippy --all-targets --all-features` (SARIF output) |
| `rust-publish.yml` | GitHub Release published | `cargo publish` to crates.io |

## Further Reading

- [GETTING-STARTED.md](GETTING-STARTED.md) — Prerequisites and first-run instructions
- [TESTING.md](TESTING.md) — Test structure, naming conventions, and coverage details
- [ARCHITECTURE.md](ARCHITECTURE.md) — Module map, core abstractions, and execution flow

## Cargo profiles

`Cargo.toml` declares three custom profile blocks — `[profile.dev]`, `[profile.dev.package."*"]`, and `[profile.test]` — that reduce local dev-build and test wall-clock without changing any compiled behaviour. They were introduced in Phase 67 (Plan 67-01) after a systematic build-performance analysis (`.planning/v3.0.0-BUILD-PERF.md` §Action #5/#6). See `.planning/intel/build-profile.md` for the AI-agent-facing rationale.

### [profile.dev]

```toml
[profile.dev]
debug = "line-tables-only"
split-debuginfo = "unpacked"
```

- **`debug = "line-tables-only"`**: Emits only line-number tables rather than full DWARF debug info. Backtraces still show file names and line numbers. The resulting debug section is significantly smaller, which speeds up the linker on every incremental build.
- **`split-debuginfo = "unpacked"`**: On macOS, the default `"packed"` mode spawns `dsymutil` to create a `.dSYM` bundle; this is the single largest contributor to link-time slowness on Apple Silicon and Intel Macs. Setting `"unpacked"` skips `dsymutil` for dev builds. On Linux this key is a no-op.

### [profile.dev.package."*"]

```toml
[profile.dev.package."*"]
opt-level = 1
debug = false
```

- **`opt-level = 1`**: Applies `-O1` optimisation to all third-party crates (rand, rayon, log, serde, …) while leaving your own code in `-O0` for fast iteration. The first clean build pays a one-time penalty of ~5-10 seconds; subsequent incremental builds are unaffected because compiled deps are cached in the Cargo artifact store.
- **`debug = false`**: Suppresses debug symbols for dependency crates. Combined with `opt-level = 1`, this makes the linker's job on deps dramatically smaller.

The result is noticeably faster test runtimes for anything that calls rand, rayon, or serde hot paths — the GA library itself uses all three heavily.

### [profile.test]

```toml
[profile.test]
opt-level = 1
```

- **`opt-level = 1`**: Test binaries run a lot of GA generations (crossover, mutation, survivor selection). Under `-O0` the inner loops are very slow; a single `-O1` pass cuts runtime by roughly 50 % with no additional rustc-time cost. This keeps `cargo test` fast enough to run in the inner development loop without sacrificing the debuggability of your own code.

### Reverting

If you need to undo the profile tuning (e.g., to diagnose a compiler bug or to measure unoptimised dep performance), simply delete the three blocks at the end of `Cargo.toml`:

```toml
# DELETE these three blocks:
[profile.dev]
...
[profile.dev.package."*"]
...
[profile.test]
...
```

Then delete `.planning/intel/build-profile.md` and remove this `§Cargo profiles` section from `docs/DEVELOPMENT.md`. No source-code change is required.

Before reverting, read `.planning/intel/build-profile.md` — it explains why the blocks were added and what CI baseline they are anchored to (Phase 66 / `.planning/baselines/v3.0.0-baseline.json`).

## Linker recommendations

For a medium-sized Rust library like `genetic_algorithms`, the link phase accounts for a meaningful slice of clean-build wall-clock time. Selecting a faster linker is one of the highest-leverage build-performance improvements available because it requires no code changes and applies to every build unconditionally. `.cargo/config.toml` encodes the recommended linker for each target platform so all developers and CI jobs benefit automatically.

### Linux (x86_64-unknown-linux-gnu)

`mold` is now the default linker on Linux via `.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

`linker = "clang"` is required because mold is invoked via `-fuse-ld=mold` and `gcc` on some Ubuntu versions does not forward that flag reliably to the linker. `clang` consistently honours `-fuse-ld`.

CI installs mold with:
```bash
sudo apt-get install -y mold
```
This step is present in `rust-unit-tests.yml`, `coverage.yml`, `rust-clippy.yml`, and `examples-smoke.yml` (Phase 67 / Plan 67-03).

For local development on Ubuntu/Debian:
```bash
sudo apt-get install -y mold
```
On Fedora/RHEL: `sudo dnf install mold`. On Arch: `sudo pacman -S mold`. The package is available in Ubuntu 22.04+ official repositories; no PPA needed.

### macOS (aarch64-apple-darwin)

A commented-out `lld` block is provided in `.cargo/config.toml` for opt-in use:

```toml
# Uncomment for ~5 % faster macOS builds (requires `brew install llvm`):
# [target.aarch64-apple-darwin]
# rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

To opt in, install LLVM and uncomment the block:
```bash
brew install llvm
```
Then uncomment the two lines in `.cargo/config.toml`. This yields approximately 5 % faster link times on Apple Silicon. The block is left commented by default because it requires a manual `brew install` step and is not enforced in CI.

### Windows

No automated linker configuration is provided for Windows targets because the project has no Windows CI. To opt in locally, add the following to your user-level `~/.cargo/config.toml` (do NOT commit this to the repository):

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-arg=-fuse-ld=lld-link"]
```

This uses `rust-lld` (bundled with the Rust toolchain) as the linker via `lld-link`. No additional installation is required.

### Reverting

To undo the Linux mold configuration:

1. Delete the `[target.x86_64-unknown-linux-gnu]` block from `.cargo/config.toml` (leave the `[target.wasm32-unknown-unknown]` block intact — it MUST remain).
2. Remove the `Install mold linker` step from `rust-unit-tests.yml`, `coverage.yml`, `rust-clippy.yml`, and `examples-smoke.yml`.
3. Remove this `§Linker recommendations` section from `docs/DEVELOPMENT.md`.

The `[target.wasm32-unknown-unknown]` block in `.cargo/config.toml` is unrelated to mold and must always remain present — it sets the `getrandom` backend required for WASM compilation.

## CI caching

CI uses `mozilla-actions/sccache-action@v0.0.9` to cache compiler output across runs via the GitHub Actions cache backend. `sccache` is set as `RUSTC_WRAPPER`, which intercepts every `rustc` invocation and serves the result from cache on cache hits. This typically delivers a 30–60 % wall-clock reduction on warm CI runs.

Affected workflows (Phase 67 / Plan 67-04):
- `rust-unit-tests.yml`
- `coverage.yml`
- `wasm-check.yml`
- `rust-clippy.yml`
- `examples-smoke.yml`

`build-perf-gate.yml` intentionally does **not** use sccache. That workflow measures cold-build wall-clock time to enforce a build-performance regression gate; enabling caching would invalidate the timing baseline (Pitfall 4 in `.planning/phases/67-build-perf-m1-config-only-quick-wins/67-RESEARCH.md`).

**Contributors do not need to install or configure anything locally.** sccache is CI-only — it is activated via the GitHub Actions action and is not required for local development.

Cache hit-rate is logged at the end of every affected CI job via `sccache --show-stats`. Look for the `Compile requests` / `Cache hits` lines in the job log to spot unexpected cache misses or regressions.

Version is pinned to `v0.0.9` (not `@latest` or `@main`) for supply-chain hygiene. To update, open a dedicated PR, update the `uses:` line in all five workflows, and measure before/after hit-rate in CI.
