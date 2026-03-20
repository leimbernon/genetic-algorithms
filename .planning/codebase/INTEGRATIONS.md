# External Integrations

**Analysis Date:** 2026-03-20

## APIs & External Services

**Not Applicable**

This is a library-only codebase. No external HTTP APIs, cloud services, or SaaS integrations are present. The GA framework is designed to be embedded within user applications.

## Data Storage

**Databases:**
- Not used - This is a library, not an application

**File Storage:**
- Local filesystem only (via `std::fs`)
  - Used by: `src/checkpoint.rs` for JSON checkpoint persistence
  - Method: Direct filesystem writes when `serde` feature enabled
  - No cloud storage integration

**Caching:**
- Fitness function result caching (LRU cache) available in-memory via `src/fitness/cache.rs`
  - Reduces redundant fitness evaluations within a GA run
  - No external cache service (Redis, Memcached, etc.)

## Authentication & Identity

**Auth Provider:**
- Not applicable - Library codebase with no external authentication

## Monitoring & Observability

**Error Tracking:**
- Not integrated - Error handling is internal via `src/error.rs` (`GaError` enum)
- Future initiative: GaObserver trait system (#182 in active milestones)

**Logs:**
- Structured logging via `log` crate with `kv_unstable` support
  - Configured at runtime via `RUST_LOG` environment variable and `env_logger`
  - Target filtering: logs use `target="ga_events"` convention
  - No centralized log aggregation or external observability system

## CI/CD & Deployment

**Hosting:**
- crates.io - Published as public crate `genetic_algorithms`
- GitHub - Repository at `https://github.com/leimbernon/rust_genetic_algorithms`
- docs.rs - Auto-generated documentation

**CI Pipeline:**
- GitHub Actions (`.github/workflows/`)
- Runs: `cargo test`, `cargo test --features serde`, `cargo clippy`, rustdoc lints
- Branch protection on `main` (PRs required)

## Environment Configuration

**Required env vars:**
- `RUST_LOG` - Optional, for runtime logging configuration (env_logger)
- No secrets or authentication tokens required

**Secrets location:**
- Not applicable - No external credentials or API keys needed

## Webhooks & Callbacks

**Incoming:**
- Not applicable - Library codebase

**Outgoing:**
- GitHub webhooks for CI/CD (configured in `.github/`)
- No outbound HTTP calls from library code

## Checkpointing & Persistence

**JSON Serialization:**
- Format: JSON via `serde_json` (when `serde` feature enabled)
- Location: User-specified filesystem path via `src/checkpoint.rs`
- Contains: Population state, GA configuration, generation index, per-generation statistics
- Use case: Resume GA runs from saved checkpoints
- Not persisted to external service (local filesystem only)

## RNG Seeding

**Random Number Generation:**
- `rand` crate provides cryptographically secure RNG
- Optional seeding available via `src/rng.rs` for reproducibility
- No external entropy source (uses system RNG or seed parameter)

---

*Integration audit: 2026-03-20*
