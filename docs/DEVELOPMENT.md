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
- [ ] New or changed code has tests in `tests/`
- [ ] All public functions and types have doc-comments
- [ ] No `panic!` in library code (use `Result<T, GaError>`)
- [ ] No manual `thread::spawn` (use rayon)

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
