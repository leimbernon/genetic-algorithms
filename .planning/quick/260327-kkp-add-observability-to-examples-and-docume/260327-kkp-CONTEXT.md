# Quick Task 260327-kkp: Add observability to examples and document observer API in README - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Task Boundary

Update all 10 existing examples to use the observer system. Distribute different observer types across examples to showcase the full API. Replace the deprecated `Reporter<U>` section in README with a comprehensive `GaObserver` section.

</domain>

<decisions>
## Implementation Decisions

### Examples scope
- Update all 10 existing examples (not create new ones)
- Every example must show observer usage

### Observer distribution
- Showcase all: LogObserver, CompositeObserver, MetricsObserver
- Most examples: LogObserver (simplest, no feature flags)
- 1-2 complex examples (e.g. rastrigin, island_model): CompositeObserver combining multiple observers
- MetricsObserver: gated with `#[cfg(feature = "observer-metrics")]` in 1-2 examples (e.g. nsga2_zdt1, island_model) so examples still compile without the flag

### Feature flag handling
- Use `#[cfg(feature = "observer-metrics")]` guards where MetricsObserver appears
- Examples must compile cleanly with plain `cargo run --example <name>`
- MetricsObserver usage activates when run with `--features observer-metrics`

### README update
- Replace the `Reporter` section with a new `GaObserver` section
- Cover: GaObserver trait, LogObserver, CompositeObserver, IslandGaObserver/Nsga2Observer sub-traits, MetricsObserver (feature flag), TracingObserver (feature flag)
- Add a note that `Reporter<U>` is deprecated and `GaObserver` is the replacement
- Keep it practical — code snippets showing `.with_observer(Arc::new(LogObserver))`

### Claude's Discretion
- Exact distribution of which observer goes in which example (keep LogObserver as default for simple examples, reserve Composite/Metrics for the richer GA mode examples)
- README section length and structure (should be comprehensive but not exhausting)

</decisions>

<specifics>
## Specific Ideas

- `rastrigin.rs` and/or `island_model.rs` → CompositeObserver (LogObserver + MetricsObserver gated)
- `nsga2_zdt1.rs` → Nsga2Observer-aware observer (LogObserver implements Nsga2Observer)
- `island_model.rs` → IslandGaObserver-aware observer (LogObserver implements IslandGaObserver)
- Simple examples (onemax_binary, onemax_extension, knapsack_binary, feature_selection, job_scheduling, nqueens_range, niching) → LogObserver
- Remove `with_reporter()` calls from examples if any exist (Reporter is deprecated)

</specifics>

<canonical_refs>
## Canonical References

- `src/observer/mod.rs` — GaObserver, IslandGaObserver, Nsga2Observer, AllObserver, NoopObserver
- `src/observer/log.rs` — LogObserver
- `src/observer/composite.rs` — CompositeObserver
- `src/observer/metrics_observer.rs` — MetricsObserver
- `src/lib.rs` — re-exports to check public API surface

</canonical_refs>
