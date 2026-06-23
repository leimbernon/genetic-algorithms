# Feature Flags Philosophy

## Date: 2026-06-15

## Default set

The following features are in the `default = [...]` set in `Cargo.toml`:

- `logging` — Enables the `log` crate dependency and `LogObserver`. Default-on for zero-regression for existing users.

Non-default features (opt-in):
- `serde` — Checkpoint serialization via serde_json
- `visualization` — PNG/SVG fitness plots via plotters
- `observer-tracing` — Structured tracing-crate spans
- `observer-metrics` — Per-generation metrics via metrics facade
- `benchmarks` — Standard benchmark functions (Sphere, Rastrigin, ZDT, DTLZ)

## Why `logging` is default-on but dep-optional

The `log` crate was previously an unconditional dependency. Making it optional via `optional = true`
and gating it behind a `logging` feature (which is in `default`) achieves:

1. **Zero regression on default**: existing `cargo add genetic_algorithms` users get identical behavior
2. **Opt-out path for ultra-lean builds**: users targeting embedded / WASM-only / minimal-footprint
   can set `default-features = false` and shed the `log` crate entirely
3. **Macro family encodes the gate**: `crate::log_info!` etc. expand to `::log::info!` when the
   feature is on, and to `()` when off — without any per-call-site `#[cfg]` annotation

The `log` crate feature list `["std", "serde", "kv_unstable"]` is preserved verbatim in Cargo.toml.
Only `optional = true` was added (Plan 68-02, D-02 constraint).

## Canonical pattern for new optional deps

When adding a new optional dependency:

1. In `Cargo.toml`: use `dep:foo` syntax in the feature definition
   ```toml
   [features]
   my-feature = ["dep:foo"]
   
   [dependencies]
   foo = { version = "...", optional = true }
   ```

2. Gate at the module/use boundary in `src/`, not at every call site:
   ```rust
   #[cfg(feature = "my-feature")]
   mod my_module;
   #[cfg(feature = "my-feature")]
   pub use my_module::MyType;
   ```

3. When the call-site count exceeds ~5, prefer a macro family over per-site `#[cfg]`:
   ```rust
   #[cfg(feature = "my-feature")]
   macro_rules! my_macro { ($($arg:tt)*) => { ::foo::macro!($($arg)*) }; }
   #[cfg(not(feature = "my-feature"))]
   macro_rules! my_macro { ($($arg:tt)*) => { () }; }
   pub(crate) use my_macro;
   ```

4. If a default-on feature is disabled, gate any types it exposes in `src/lib.rs`:
   ```rust
   #[cfg(feature = "my-feature")]
   pub use my_module::MyType;
   ```

5. Gate integration tests and examples that use those types with `#[cfg(feature = "my-feature")]`
   or `required-features = ["my-feature"]` in `[[example]]` / `[[test]]` Cargo entries.

## What MUST NOT happen

- Do not add a feature that is default-on without proving zero-regression via `cargo test` with
  default features AND proving the non-default path compiles via `cargo check --no-default-features`
- Do not gate items that are part of the stable public API without a `MIGRATION.md` entry and a
  `CHANGELOG.md` note in the `Changed (breaking)` section
- Do not use `features = ["dep:foo"]` in a downstream `use` path — `dep:foo` is a Cargo-level
  alias and cannot be referenced in Rust code directly
- Do not collapse feature lists when marking an existing dep optional: if `foo = { version = "...",
  features = [...] }` already has a feature list, keep it verbatim and only add `optional = true`

## How to verify a new flag

```bash
# Feature-enabled path
cargo check --features my-feature

# Feature-disabled path (no-default-features if it was default-on)
cargo check --no-default-features

# Explicit enable on top of no-default-features
cargo check --no-default-features --features my-feature

# WASM compatibility (mandatory per CLAUDE.md)
cargo check --target wasm32-unknown-unknown

# Full tests both ways
cargo test
cargo test --no-default-features

# CC-3 golden tests byte-identical (if applicable)
cargo test --test golden_tests --release

# Zero rustdoc warnings
cargo doc --no-deps 2>&1 | grep -i "^warning"
```

Feature-matrix CI enforces all states automatically once the matrix rows are added to
`.github/workflows/feature-matrix.yml`.
