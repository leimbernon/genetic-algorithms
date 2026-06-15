# Migrating to v3.0.0

v3.0.0 is a breaking release that cleans up the public API before the v3.x feature window opens.
Every change has been **deprecated since v2.2.0** and is now removed or restructured.

This guide covers all seven breaking changes with before/after code for each.
See [`CHANGELOG.md`](./CHANGELOG.md) for the full change list.

---

## Trait split: ChromosomeT + LinearChromosome

**What changed (D-01 / D-02):** `ChromosomeT` now defines only the minimal core contract (fitness
and age). The flat-slice DNA surface (`dna()`, `dna_mut()`, `set_dna()`, `set_fitness_fn()`,
`new_gene()`, `set_gene()`) moved to a new `LinearChromosome: ChromosomeT` supertrait.
All built-in engines, operators, and chromosomes already implement it.

**Who is affected:** Anyone who implemented a custom chromosome type.

### Before

```rust
use genetic_algorithms::traits::{ChromosomeT, GeneT};

#[derive(Clone, Default)]
struct MyChromosome {
    dna: Vec<MyGene>,
    fitness: f64,
    age: usize,
}

impl ChromosomeT for MyChromosome {
    type Gene = MyGene;

    fn dna(&self) -> &[MyGene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [MyGene] { &mut self.dna }
    fn set_dna(&mut self, dna: std::borrow::Cow<[MyGene]>) {
        self.dna = dna.into_owned();
    }
    fn set_fitness_fn<F>(&mut self, f: F) -> &mut Self
    where F: Fn(&[MyGene]) -> f64 + Send + Sync + 'static
    { /* ... */ self }
    fn new_gene(&self) -> MyGene { MyGene::default() }
    fn calculate_fitness(&mut self) { /* ... */ }
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, v: f64) { self.fitness = v; }
    fn age(&self) -> usize { self.age }
    fn set_age(&mut self, v: usize) { self.age = v; }
}
```

### After

```rust
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};

#[derive(Clone, Default)]
struct MyChromosome {
    dna: Vec<MyGene>,
    fitness: f64,
    age: usize,
}

// Core contract: fitness + age only.
impl ChromosomeT for MyChromosome {
    type Gene = MyGene;

    fn calculate_fitness(&mut self) { /* your logic */ }
    fn fitness(&self) -> f64 { self.fitness }
    fn set_fitness(&mut self, v: f64) { self.fitness = v; }
    fn age(&self) -> usize { self.age }
    fn set_age(&mut self, v: usize) { self.age = v; }
}

// Flat-slice surface: required for all built-in operators.
impl LinearChromosome for MyChromosome {
    fn dna(&self) -> &[MyGene] { &self.dna }
    fn dna_mut(&mut self) -> &mut [MyGene] { &mut self.dna }
    fn set_dna(&mut self, dna: std::borrow::Cow<[MyGene]>) {
        self.dna = dna.into_owned();
    }
    fn set_fitness_fn<F>(&mut self, f: F) -> &mut Self
    where F: Fn(&[MyGene]) -> f64 + Send + Sync + 'static
    { /* store closure */ self }
    fn new_gene(&self) -> MyGene { MyGene::default() }
    // set_gene() and reset() have default implementations — no override needed.
}
```

**Custom non-linear chromosomes:** If your chromosome is not flat-slice (e.g., a tree structure),
implement `ChromosomeT` only. Operators that work on fitness/age (`Survivor::Fitness`,
`selection::Tournament`) still accept it. Operators that call `dna()` require `LinearChromosome`
and are not usable with non-linear types by design.

---

## LinearChromosome: `default()` renamed to `reset()`

**What changed (D-03):** The `default(mut self) -> Self` reset helper on `LinearChromosome` is
renamed to `reset(&mut self) -> &mut Self`. This removes ambiguity with the `Default` trait and
adopts a builder-style mutable reference return.

### Before

```rust
let chromosome = my_chromosome.default(); // resets fitness, age, dna
```

### After

```rust
let chromosome = my_chromosome.reset(); // same effect, returns &mut Self
```

The `Default` trait is unaffected and still returns a zero-initialized chromosome.

---

## Reporter removed — use GaObserver

**What changed (D-10):** The `Reporter<U>` trait and its implementations (`SimpleReporter`,
`DurationReporter`, `NoopReporter`) are removed. The `with_reporter()` builder on `Ga<U>` is
also removed. Use `GaObserver<U>` instead — available since v2.2.0, it provides 11 lifecycle
hooks vs Reporter's 4, uses `Arc<dyn>` for thread-safe sharing, and uses `&self` instead of
`&mut self`.

### Before

```rust
use genetic_algorithms::reporter::SimpleReporter;

let mut ga = Ga::new()
    .with_population_size(100)
    .with_chromosome_length(ChromosomeLength::Fixed(20))
    // ...
    .with_reporter(Box::new(SimpleReporter::new(10)))
    .build()
    .unwrap();
```

### After

```rust
use genetic_algorithms::observer::LogObserver;
use std::sync::Arc;

let mut ga = Ga::new()
    .with_population_size(100)
    .with_chromosome_length(ChromosomeLength::Fixed(20))
    // ...
    .with_observer(Arc::new(LogObserver))
    .build()
    .unwrap();
```

**Built-in observers** (all in `genetic_algorithms::observer`):
- `LogObserver` — emits `log!()` calls matching the pre-v2.2.0 output (direct drop-in for `SimpleReporter`)
- `CompositeObserver` — composes multiple observers into one
- `MetricsObserver` — emits `metrics!()` counters/gauges (requires `observer-metrics` feature)
- `TracingObserver` — emits `tracing::event!()` spans (requires `observer-tracing` feature)
- `NoopObserver` — zero-overhead no-op (equivalent to no observer at all)

**Custom observer migration:**

```rust
use genetic_algorithms::observer::GaObserver;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::ChromosomeT;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Interior mutability via atomics (GaObserver uses &self, not &mut self)
struct CountingObserver {
    generation_count: AtomicUsize,
}

impl<U: ChromosomeT> GaObserver<U> for CountingObserver {
    fn on_generation_end(&self, _stats: &GenerationStats) {
        self.generation_count.fetch_add(1, Ordering::Relaxed);
    }
    fn on_run_end(&self, cause: TerminationCause, _all_stats: &[GenerationStats]) {
        println!("Finished ({:?}) after {} generations", cause,
                 self.generation_count.load(Ordering::Relaxed));
    }
}

let observer = Arc::new(CountingObserver {
    generation_count: AtomicUsize::new(0),
});
let mut ga = Ga::new()
    // ...
    .with_observer(observer)
    .build()
    .unwrap();
```

Full API reference: [docs.rs/genetic_algorithms — observer module](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/observer/index.html)

---

## ChromosomeLength replaces genes_per_chromosome

**What changed (D-07):** The `genes_per_chromosome: usize` field on `LimitConfiguration` is
replaced by `chromosome_length: ChromosomeLength`. The builder method changes accordingly.

### Before

```rust
let ga = Ga::new()
    .with_genes_per_chromosome(8)
    // ...
    .build()
    .unwrap();
```

### After

```rust
use genetic_algorithms::ChromosomeLength;

let ga = Ga::new()
    .with_chromosome_length(ChromosomeLength::Fixed(8))
    // ...
    .build()
    .unwrap();
```

`ChromosomeLength::Variable { min, max }` is reserved for variable-length chromosome support
arriving in a future phase. Using it currently returns an error from `build()`.

---

## Flat stopping builders replace StoppingCriteria struct

**What changed (D-08):** The `StoppingCriteria` struct is removed. Its three fields become
direct flat builders on `Ga<U>`.

### Before

```rust
use genetic_algorithms::configuration::StoppingCriteria;

let ga = Ga::new()
    .with_stopping_criteria(StoppingCriteria {
        stagnation_generations: Some(50),
        convergence_threshold: Some(0.001),
        max_duration_secs: Some(60.0),
    })
    // ...
    .build()
    .unwrap();
```

### After

```rust
let ga = Ga::new()
    .with_stagnation_limit(50)
    .with_convergence_threshold(0.001)
    .with_max_duration_secs(60.0)
    // ...
    .build()
    .unwrap();
```

Each builder is independent — set only the ones you need. Omitting a stopping criterion
means it is not enforced (same as passing `None` in the old struct).

---

## LimitConfiguration field removals

**What changed (D-06):** `LimitConfiguration.needs_unique_ids` and
`LimitConfiguration.alleles_can_be_repeated` are removed without replacement.
These flags existed to trigger uniqueness enforcement inside the built-in initializers, but the
enforcement was incomplete and did not propagate through crossover or mutation.

### Before

```rust
let ga = Ga::new()
    .with_needs_unique_ids(true)         // removed
    .with_alleles_can_be_repeated(false) // removed
    // ...
    .build()
    .unwrap();
```

### After

Remove the two builder calls. The initializers no longer enforce uniqueness.

**If you need permutation chromosomes** (TSP, scheduling, ordering problems):
- Use a custom `with_initialization_fn` that returns a shuffled permutation.
- Apply a custom `RepairFn` (`.with_repair_operator()`) after crossover/mutation to
  restore validity.
- Phase 48 introduces `UniqueChromosome<T>` which enforces permutation invariants
  at the type level with compatible crossover operators (PMX, OX).

---

## GaConfiguration field access → accessor methods

**What changed (D-09):** `GaConfiguration` fields are now `pub(crate)`. External read access
uses sub-struct accessor methods.

### Before

```rust
let n = ga.configuration.limit_configuration.genes_per_chromosome;
let method = ga.configuration.selection_configuration.method;
```

### After

```rust
use genetic_algorithms::ChromosomeLength;

// Access the ChromosomeLength (replaces genes_per_chromosome)
let length = ga.configuration().limit().chromosome_length;
// or pattern-match:
if let ChromosomeLength::Fixed(n) = ga.configuration().limit().chromosome_length {
    println!("Fixed length: {n}");
}

let method = ga.configuration().selection().method;
```

Available sub-struct accessors on `Ga::configuration() -> &GaConfiguration`:
- `.limit()` → `&LimitConfiguration`
- `.selection()` → `&SelectionConfiguration`
- `.crossover()` → `&CrossoverConfiguration`
- `.mutation()` → `&MutationConfiguration`
- `.survivor()` → `Survivor` (returns enum value, not reference)

---

## Logger setup (v2 auto-init → v3 explicit)

**What changed (Phase 68 / Plan 68-01):** The library no longer installs `env_logger` automatically
during `Ga::run()`. In v2, the GA called `env_logger::Builder::from_default_env().try_init()` at
the start of every run — silently competing with the application's own logger installation and
dragging `env_logger` (and ~12 transitive crates) into every library consumer's dependency graph.
In v3, `env_logger` is a dev-dependency only; the library emits `log!()` events and the application
chooses the subscriber.

**Who is affected:** Any code that relied on the implicit `env_logger` initialization inside the GA
to receive log output. Also any code that called `.with_logs(LogLevel::Warn)` (or any other
`LogLevel` variant) on the builder.

### Before

```rust
use genetic_algorithms::configuration::LogLevel;
use genetic_algorithms::traits::ConfigurationT;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Binary;

// No logger installed in main() — the GA installs env_logger automatically.
let mut ga = Ga::<Binary>::new()
    .with_logs(LogLevel::Warn)   // configures env_logger filter level
    // ...
    .build()
    .unwrap();

ga.run().unwrap(); // env_logger was installed here; RUST_LOG=warn output appeared
```

### After

```rust
use genetic_algorithms::traits::ConfigurationT;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Binary;

fn main() {
    env_logger::init(); // application installs its own subscriber — first statement in main()
    // or any other log subscriber: tracing-subscriber, simplelog, fern, etc.

    let mut ga = Ga::<Binary>::new()
        // .with_logs() is gone — control log verbosity via RUST_LOG or your subscriber's config
        // ...
        .build()
        .unwrap();

    ga.run().unwrap(); // the GA emits log!() events; env_logger above handles them
}
```

Control verbosity the idiomatic way: set `RUST_LOG=genetic_algorithms=warn cargo run`, or configure
your subscriber programmatically. The GA emits events on the `ga_events`, `population_events`,
and related log targets.

See `.planning/intel/logger-history.md` for the full rationale and a list of what must never be
reintroduced.

### Removed: `LogLevel` enum and `with_logs()` builder method

Both `configuration::LogLevel` and `ConfigurationT::with_logs()` are removed in v3.0.0. They
only existed to configure the now-removed auto-installer. Remove all calls to `.with_logs()`
and any `use ... LogLevel` imports from your code. Filter log verbosity via `RUST_LOG` or your
own subscriber configuration instead.
