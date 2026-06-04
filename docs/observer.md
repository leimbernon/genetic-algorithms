# Observer System

> Structured lifecycle hooks for the GA execution loop — all engines support attaching an observer via `Option<Arc<dyn GaObserver<U>>>`.

## Overview

The `GaObserver` trait provides **12 lifecycle hooks** that fire at precise points during engine execution. Unlike the legacy `Reporter` trait, observers are stored as `Arc<dyn GaObserver + Send + Sync>` (sharable across island model threads) and receive `&self` (not `&mut self`), enabling interior mutability patterns.

All trait methods have default no-op implementations — implement only the hooks you need. The `Send + Sync` supertraits are required for safe sharing across rayon threads via `Arc`.

Built-in observers include `LogObserver`, `CompositeObserver`, `NoopObserver`, and feature-gated `MetricsObserver` and `TracingObserver`.

Engine-specific sub-traits exist for `IslandGaObserver`, `Nsga2Observer`, `Nsga3Observer`, `Spea2Observer`, `MoeaDObserver`, `SmsEmoaObserver`, and `IbeaObserver`.

## Key Concepts

### GaObserver Trait Hooks

| Hook | When it fires |
|------|--------------|
| `on_run_start` | Once before the first generation |
| `on_generation_start` | Start of each generation, before any operators |
| `on_selection_complete` | After parent selection |
| `on_crossover_complete` | After crossover produces offspring |
| `on_mutation_complete` | After mutation is applied |
| `on_fitness_evaluation_complete` | After fitness evaluation of new population |
| `on_survivor_selection_complete` | After survivor selection prunes population |
| `on_new_best` | When the population's best fitness improves |
| `on_stagnation` | Each time the stagnation counter increments |
| `on_extension_triggered` | When an extension strategy fires (with `ExtensionEvent` payload) |
| `on_generation_end` | End of each generation, after statistics collected |
| `on_run_end` | Once after the GA loop exits |

### Built-in Observers

| Observer | Description | Features required |
|----------|-------------|-------------------|
| `LogObserver` | Reproduces structured `log!()` output for each generation | None |
| `CompositeObserver` | Fan-out to multiple observers via `.add(Arc::new(observer))` | None |
| `NoopObserver` | Zero-overhead placeholder | None |
| `MetricsObserver` | Per-generation metrics (gauges, counters, histograms) via the `metrics` facade | `observer-metrics` |
| `TracingObserver` | Structured `tracing`-crate spans and events | `observer-tracing` |

### Engine-Specific Sub-Traits

Each multi-objective engine defines its own observer sub-trait for engine-specific hooks:

- `Nsga2Observer<U>` — `on_non_dominated_sort_complete`, `on_pareto_front_assigned`
- `Nsga3Observer<U>` — `on_non_dominated_sort_complete`, `on_pareto_front_assigned`
- `MoeaDObserver<U>` — `on_non_dominated_sort_complete`, `on_pareto_front_assigned`
- `Spea2Observer<U>` — Engine-specific hooks
- `SmsEmoaObserver<U>` — Engine-specific hooks
- `IbeaObserver<U>` — Engine-specific hooks
- `IslandGaObserver<U>` — `on_migration_complete`

### GP Engine Observers

`GpGa<N>` reuses the core `GaObserver<GpChromosome<N>>` trait — no GP-specific sub-trait exists. All standard hooks fire (`on_run_start`, `on_generation_end`, `on_new_best`, `on_stagnation`, etc.). The `GenerationStats` struct passed to `on_generation_end` includes the `avg_node_count` field which is populated by `GpGa` for bloat monitoring (set to `0.0` by all other engines).

```rust
use std::sync::Arc;
use genetic_algorithms::LogObserver;

let mut engine = GpGa::<MathNode>::with_ramped_half_and_half(config, fitness_fn)
    .with_observer(Arc::new(LogObserver));
```

### AllObserver

`AllObserver<U>` is a blanket impl: any type implementing all three core observer traits (GaObserver + IslandGaObserver + Nsga2Observer) automatically gets the `AllObserver` trait, enabling a single observer to be used across all engine types.

## API Quick Reference

```rust,ignore
use std::sync::Arc;
use genetic_algorithms::observer::{
    GaObserver, LogObserver, CompositeObserver, NoopObserver,
    Nsga3Observer, Spea2Observer,
};

// Attach LogObserver to the GA engine
let ga = Ga::new()
    .with_observer(Arc::new(LogObserver))
    // ... other configuration
    .build().expect("valid config");

// CompositeObserver for fan-out
let composite = CompositeObserver::new()
    .add(Arc::new(LogObserver))
    .add(Arc::new(MyCustomObserver));
```

## Usage Example

```rust,ignore
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::traits::ChromosomeT;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct MyObserver {
    generation_count: AtomicUsize,
    best_fitness: std::sync::Mutex<Option<f64>>,
}

impl<U: ChromosomeT> GaObserver<U> for MyObserver {
    fn on_run_start(&self) {
        println!("GA run started");
    }

    fn on_generation_start(&self, generation: usize) {
        self.generation_count.store(generation, Ordering::Relaxed);
    }

    fn on_new_best(&self, generation: usize, best: U) {
        let fitness = best.fitness();
        println!("New best at gen {}: {:.6}", generation, fitness);
        let mut best_fitness = self.best_fitness.lock().unwrap();
        *best_fitness = Some(fitness);
    }

    fn on_generation_end(&self, stats: &GenerationStats) {
        println!("Gen {}: best={:.4}, avg={:.4}, diversity={:.4}",
            stats.generation, stats.best_fitness, stats.avg_fitness, stats.diversity);
    }

    fn on_run_end(&self, cause: TerminationCause, _all_stats: &[GenerationStats]) {
        println!("GA run ended: {:?}", cause);
    }
}
```

## Configuration

Attach an observer to any engine using the `.with_observer()` builder method:

```rust,ignore
use std::sync::Arc;
use genetic_algorithms::LogObserver;

let mut ga = Ga::new()
    .with_observer(Arc::new(LogObserver))
    // ... configure operators, etc.
    .build()?;
```

For multi-objective engines, the observer must implement the engine-specific sub-trait:

```rust,ignore
let mut nsga3 = Nsga3Ga::new(nsga3_config, ga_config)
    .with_observer(Arc::new(MyNsga3Observer))
    .build()?;
```

## Performance Considerations

- Zero overhead when no observer is attached (stored as `Option<Arc<dyn GaObserver<U>>>` — the `None` branch is eliminated by the compiler).
- Observer hooks fire inside the generation loop. Expensive observer implementations (disk I/O, network calls) will slow down execution.
- Use `Arc<dyn GaObserver>` for shared ownership across rayon threads (island model).
- `ExtensionEvent` is stack-allocated and `Copy` — zero heap allocation per hook.

## See Also

- [Constraints](constraints.md) — Constraint handling subsystem
- [Hall of Fame](hall_of_fame.md) — Elite solution archive
- [Engines Overview](engines.md) — All engine configuration options
- [docs.rs/genetic_algorithms::observer](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/observer/index.html) — Module API reference
