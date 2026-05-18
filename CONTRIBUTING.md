<!-- generated-by: gsd-doc-writer -->
# Contributing to genetic_algorithms

Thank you for your interest in contributing to `genetic_algorithms`. This guide covers everything you need to get started.

## Development Setup

The minimum supported Rust version is `1.81.0`. Ensure your toolchain meets this requirement before contributing.

Clone the repository and build the project:

```bash
git clone https://github.com/leimbernon/rust_genetic_algorithms.git
cd rust_genetic_algorithms
cargo build
```

Run the test suite to confirm your environment is working:

```bash
cargo test
cargo test --features serde
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for a map of the codebase and [docs/configuration.md](docs/configuration.md) for configuration details.

## Coding Standards

All contributions must pass the following checks before merging:

- **Tests** — `cargo test` and `cargo test --features serde` must both pass with zero failures. Every change must have corresponding tests.
- **Clippy** — `cargo clippy --all-targets --all-features` must produce zero warnings. CI runs Clippy automatically on every PR.
- **Rustdoc** — `cargo doc --no-deps` must produce zero warnings. All public API items require doc comments (`///`).
- **No `panic!` in library code** — return `Result<T, GaError>` instead. Tests may use `.unwrap()`.
- **No manual threading** — use `rayon` (`par_iter`, `into_par_iter`) for parallelism. Do not use `std::thread::spawn`, `Arc<Mutex<>>`, or `sync_channel`.
- **No breaking changes by default** — prefer additive changes: new enum variants, new builder methods, new traits, new modules, or `Option<T>` fields with a `None` default. Discuss breaking changes in the relevant issue before opening a PR.
- **Observability hooks** — all changes to the GA execution flow must preserve `GaObserver` notification points. Never remove or bypass observer callbacks.
- **Performance patterns** — maintain established conventions: `Cow<[Gene]>` for zero-copy DNA, in-place mutation via `dna_mut()` / `set_gene()`, `select_nth_unstable_by()` over full sort for top-k selection, and `Vec::with_capacity()` pre-allocation in hot paths.

## Feature Flags

The crate exposes the following optional feature flags:

| Flag | Purpose |
|------|---------|
| `serde` | Checkpoint serialization via serde/serde_json |
| `visualization` | Plotting support via plotters |
| `observer-tracing` | Tracing integration for the observer system |
| `observer-metrics` | Metrics integration for the observer system |

When adding or modifying features that touch optional dependencies, test with the relevant flag(s) enabled.

## PR Guidelines

- **Branch from the milestone branch**, not from `main`. The active milestone branch follows the pattern `milestone/<milestone-name>`. Feature branches use `feat/<issue-number>-<short-description>`; fix branches use `fix/<issue-number>-<short-description>`.
- **Target the milestone branch** in your PR, not `main`.
- **One concern per PR** — keep changes focused. Large refactors should be discussed in an issue first.
- **Tests go in `tests/`** — all tests must be placed in the `tests/` directory, never inline with implementation code. New operator tests go in `tests/operations/test_<operator>.rs` and must be registered in `tests/test_operations.rs`.
- **Operator implementations** go in their own file under `src/operations/<type>/`. Configuration structs belong in `src/configuration.rs` (or `<module>/configuration.rs` for sub-modules). New operators require a matching enum variant and factory registration.
- **Stochastic tests** must use retry loops (10 iterations) to avoid flakiness — never assert on a single random sample.
- **Commit message style** — use conventional commits format: `feat(scope): description`, `fix(scope): description`, `docs(scope): description`, etc.
- **CI must be green** — the Rust Unit Tests workflow runs `cargo test` on every PR targeting `main`; the Rust Clippy Check workflow runs on every PR. Address all failures before requesting review.

## Issue Reporting

Use GitHub Issues at `https://github.com/leimbernon/rust_genetic_algorithms/issues`.

**Bug reports** — use the `[BUG]` issue template and include:
- A clear description of the bug
- Steps to reproduce
- Expected vs actual behavior
- Any relevant error output or context

**Feature requests** — use the `[REQUEST]` issue template and include:
- The problem your feature solves
- Your proposed solution
- Alternatives you considered

## License

By contributing, you agree that your contributions will be licensed under the [Apache-2.0 License](LICENSE).
