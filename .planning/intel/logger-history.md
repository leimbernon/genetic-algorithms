# Logger History

## Date: 2026-06-15

## Why the library no longer installs env_logger

Libraries must never install loggers. Doing so forces a specific backend on every downstream
user and makes it impossible for the application to control its own logging pipeline.

Phase 68 shed `env_logger` from `[dependencies]`, eliminating ~12 transitive crates
(`humantime`, `regex`, `regex-automata`, `regex-syntax`, `memchr`, `aho-corasick`, etc.).
The result was ≥15% improvement in clean-build wall-clock time and a meaningfully smaller
dependency graph for every consumer of the library.

The library continues to emit structured log events via the `log` facade (gated behind the
`logging` Cargo feature, which is default-on). Application code is responsible for installing
a log subscriber — e.g. `env_logger::init()` in `fn main()` — before running the GA.

## What MUST NOT be reintroduced

- `env_logger` in `[dependencies]` — it is only allowed in `[dev-dependencies]` (for tests
  and examples). Adding it back to the non-dev section re-installs the logger for all users.
- Auto-init block in `Ga::run()` or any engine entry point — the four-line
  `env_logger::Builder::try_init()` block removed in Phase 68 must never be restored.
- `LogLevel` enum — removed in v3.0.0. Users filter log output via their own subscriber
  (e.g. `RUST_LOG=warn cargo run`).
- `with_logs()` builder method on `ConfigurationT` / `Configuration` — removed in v3.0.0
  alongside `LogLevel`. All implementations of `ConfigurationT` must NOT add it back.

## Canonical pattern for emitting log events

The GA uses the internal macro family defined in `src/lib.rs`:

```rust
crate::log_info!(target: "ga_events", "generation {gen} best={best:.4}");
crate::log_debug!(target: "ga_events", "crossover produced {n} offspring");
crate::log_trace!(target: "ga_events", "selection: {pair:?}");
crate::log_warn!(target: "ga_events", "population below minimum size");
crate::log_error!(target: "ga_events", "fitness function returned NaN");
```

When the `logging` Cargo feature is enabled (the default), these macros delegate to
`::log::info!`, `::log::debug!`, `::log::trace!`, `::log::warn!`, and `::log::error!`
respectively. When the feature is disabled (e.g. `--no-default-features`), they compile
to no-ops with zero runtime cost.

New code in `src/` MUST use `crate::log_*!` exclusively — never import `log::*` or call
`log::info!` etc. directly. This ensures the feature gate works uniformly.

Application code installs the subscriber:

```rust
fn main() {
    env_logger::init(); // or tracing_subscriber::fmt::init(), etc.
    let ga = Ga::new(config).run();
}
```

## How to verify

```bash
# 1. Integration test — asserts the library never installs a logger
cargo test --test test_no_logger_installed
# Expected: 1 test passes

# 2. No env_logger in src/
grep -rn "env_logger" src/
# Expected: 0 matches

# 3. env_logger only in [dev-dependencies] in Cargo.toml
grep -n "env_logger" Cargo.toml
# Expected: only lines under [dev-dependencies], not [dependencies]

# 4. No-default-features build must pass
cargo build --no-default-features
# Expected: exit 0

# 5. With-logging feature build must pass
cargo build --features logging
# Expected: exit 0
```
