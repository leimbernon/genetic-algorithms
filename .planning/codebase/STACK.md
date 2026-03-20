# Technology Stack

**Analysis Date:** 2026-03-20

## Languages

**Primary:**
- Rust 1.81.0+ - Core library implementation

**Supported by:**
- TOML - Package manifest and configuration

## Runtime

**Environment:**
- Cargo (Rust package manager)

**Package Manager:**
- Cargo 1.81.0+
- Lockfile: `Cargo.lock` (present and committed)

## Frameworks

**Core:**
- None (library-centric, no web/async frameworks)

**Testing:**
- Criterion 0.8.2 - Benchmarking harness
  - Config: `[[bench]]` declarations in `Cargo.toml`
  - Benches located in `benches/` directory with custom harness (`harness = false`)

**Build/Dev:**
- Built-in Rust compiler and tooling
- Custom build script: `build.rs` - Registers tarpaulin code coverage cfg

## Key Dependencies

**Critical:**
- `rand` 0.9.2 - Pseudo-random number generation for population initialization, selection, crossover, mutation
  - Used throughout: `src/rng.rs`, operators, initializers
- `rayon` 1.10 - Data parallelism for fitness evaluation and reproduction
  - Parallel fitness batch evaluation and multi-operator crossover/mutation
- `log` 0.4.22 - Logging facade with `kv_unstable` support for structured logging
  - Features: `["std", "serde", "kv_unstable"]`
  - Used with target filtering throughout GA operations
- `env_logger` 0.11.5 - Runtime log configuration via environment variables
  - Optional initialization for development/debugging

**Serialization (conditional - feature: serde):**
- `serde` 1.x - Serialization/deserialization framework
  - Features: `["derive"]` for procedural macros
  - Optional dependency: `dep:serde`
- `serde_json` 1.x - JSON serialization
  - Optional dependency: `dep:serde_json`
  - Used in checkpoint save/load: `src/checkpoint.rs`

## Configuration

**Environment:**
- Runtime logging via `RUST_LOG` environment variable (env_logger integration)
- No `.env` file or environment secrets required for core functionality
- Feature flags enable/disable serde support

**Build:**
- `Cargo.toml` - Main manifest with workspace-style configuration
- `Cargo.lock` - Lock file ensuring reproducible builds
- `build.rs` - Pre-build script for cfg registration

## Platform Requirements

**Development:**
- Rust toolchain 1.81.0 or later
- Cargo package manager
- Supports: macOS, Linux, Windows (Rust is cross-platform)

**Production:**
- Standalone library (no external services or infrastructure required)
- Compilation to WASM or native binary depends on target
- No deployment infrastructure assumed (library for embedding in other applications)

## Feature Flags

| Flag | Dependencies | Purpose |
|------|-------------|---------|
| `serde` | `serde`, `serde_json` | Enable checkpoint serialization/deserialization for GA state persistence |
| (default) | None | Minimal core library without serialization support |

## Performance Characteristics

- `rayon` enables work-stealing thread pool for parallel fitness evaluation
- `rand` provides fast RNG with minimal overhead
- No async/await (synchronous, thread-based parallelism)
- Memory-efficient DNA representation via `Cow<[Gene]>` for zero-copy operations

---

*Stack analysis: 2026-03-20*
