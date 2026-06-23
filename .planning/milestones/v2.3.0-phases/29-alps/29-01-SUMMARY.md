---
phase: 29-alps
plan: 01
tags: [alps, engine, benchmark]
completed: "2026-04-27"
---

# Plan 01: ALPS (Age-Layered Population Structure) Engine

**Result:** Complete — engine, tests, and benchmark shipped in one pass.

## What Was Done

- `src/engines/alps/configuration.rs` — `AlpsConfiguration` builder: number of layers, layer size, `AlpsAgeScheme` enum (Linear/Fibonacci/Polynomial), age gap, injection interval, crossover/mutation operators, fitness direction, fitness target, max generations
- `src/engines/alps/engine.rs` — `AlpsEngine<U>` generic over `ChromosomeT`:
  - Maintains N age-layered populations; individuals track their age across generations
  - 3 age schemes: `Linear` (threshold = layer × gap), `Fibonacci` (threshold = Fib(layer) × gap), `Polynomial` (threshold = layer² × gap)
  - Evolution per generation: random pairing within layers + 20% cross-layer mating with best elder from adjacent older layer; crossover + mutation → offspring (age=0); keep best `layer_size` survivors
  - Age promotion: individuals exceeding layer age threshold move to next older layer; oldest-layer overflow discarded
  - Periodic injection: layer 0 reseeded with fresh random individuals every `injection_interval` generations (`0` = disabled)
  - `AlpsResult` type: all layer populations, best individual, generations run
- `src/lib.rs` — public re-export of `alps` module
- `tests/test_alps.rs` — 11 integration tests: all 3 age schemes, threshold correctness, cross-layer mating, injection (enabled and disabled), early stopping, result consistency
- `benches/alps.rs` — ALPS vs DE comparison + all 3 age schemes on sphere(5D); `sample_size(10)`
- `Cargo.toml` — `alps` bench target added

## Key Decisions

- Cross-layer mating uses 20% probability to mate with the best individual from the adjacent older layer — balances exploration vs exploitation without full inter-layer panmixia
- Injection reseeds layer 0 completely every N generations (not just one individual) — ensures fresh genetic material enters regularly
- `injection_interval = 0` disables injection (clean opt-out with zero overhead)

## Verification

- `cargo test --test test_alps`: 11 tests passed
- `cargo bench --bench alps -- --test`: exits 0
- `cargo clippy`: 0 issues
- `cargo doc --no-deps`: 0 warnings
