# Migrating to v3.0.0

v3.0.0 is a breaking release that cleans up the public API before the v3.x feature window opens.
Every change has been **deprecated since v2.2.0** and is now removed or restructured.

This guide covers every breaking change with before/after code for each.
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

### Compiler error

```
error[E0277]: the trait bound `MyChromosome: LinearChromosome` is not satisfied
  --> src/main.rs:12:5
   |
12 |     .with_chromosome_length(ChromosomeLength::Fixed(20))
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `LinearChromosome` is not implemented for `MyChromosome`
   |
   = help: implement `LinearChromosome` for `MyChromosome`
```

> **Note:** In v3, all built-in operators (selection, crossover, mutation, survivor) bound on `LinearChromosome` instead of `ChromosomeT`. If your custom chromosome only implements `ChromosomeT`, every operator call site fails with the E0277 error above. Add an `impl LinearChromosome for YourChromosome { ... }` block (see the After example above) to restore compatibility. Tree-shaped chromosomes that intentionally do NOT support flat-slice access should implement `TreeChromosome: ChromosomeT` and use the `GpGa<U>` engine instead of `Ga<U>`.

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

### Compiler error

```
error[E0599]: no method named `default` found for type `MyChromosome` in the current scope
  --> src/main.rs:8:34
   |
8  |     let chromosome = my_chromosome.default();
   |                                    ^^^^^^^ method not found in `MyChromosome`
   |
   = help: the `LinearChromosome::default()` method was renamed to `reset()` in v3.0.0; `Default::default()` (the standard trait) is unaffected
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

### Compiler error

```
error[E0432]: unresolved import `genetic_algorithms::reporter`
  --> src/main.rs:1:5
   |
1  | use genetic_algorithms::reporter::SimpleReporter;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `reporter` in the root
   |
error[E0599]: no method named `with_reporter` found for struct `Ga`
  --> src/main.rs:8:10
   |
8  |     .with_reporter(Box::new(SimpleReporter::new(10)))
   |      ^^^^^^^^^^^^^ method not found in `Ga<_>`
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

### Compiler error

```
error[E0599]: no method named `with_genes_per_chromosome` found for struct `Ga`
  --> src/main.rs:5:10
   |
5  |     .with_genes_per_chromosome(8)
   |      ^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `Ga<_>`
   |
   = help: use `with_chromosome_length(ChromosomeLength::Fixed(8))` instead
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

### Compiler error

```
error[E0432]: unresolved import `genetic_algorithms::configuration::StoppingCriteria`
  --> src/main.rs:1:5
   |
1  | use genetic_algorithms::configuration::StoppingCriteria;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `StoppingCriteria` in `genetic_algorithms::configuration`
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

### Compiler error

```
error[E0559]: struct `LimitConfiguration` has no field named `needs_unique_ids`
  --> src/main.rs:5:10
   |
5  |     .with_needs_unique_ids(true)
   |      ^^^^^^^^^^^^^^^^^^^^^ method not found in `Ga<_>`
   |
   = help: the fields `needs_unique_ids` and `alleles_can_be_repeated` were removed in v3.0.0; remove these builder calls
```

**Fix:** Remove `.with_needs_unique_ids(...)` and `.with_alleles_can_be_repeated(...)` from your builder chain.

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

### Compiler error

```
error[E0616]: field `limit_configuration` of struct `GaConfiguration` is private
  --> src/main.rs:5:37
   |
5  |     let n = ga.configuration.limit_configuration.genes_per_chromosome;
   |                              ^^^^^^^^^^^^^^^^^^^ private field
   |
   = help: use `ga.configuration().limit()` to access limit configuration
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

### Compiler error

```
error[E0432]: unresolved import `genetic_algorithms::configuration::LogLevel`
  --> src/main.rs:1:5
   |
1  | use genetic_algorithms::configuration::LogLevel;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `LogLevel` in `genetic_algorithms::configuration`
   |
error[E0599]: no method named `with_logs` found for struct `Ga`
  --> src/main.rs:9:10
   |
9  |     .with_logs(LogLevel::Warn)
   |      ^^^^^^^^^ method not found in `Ga<_>`
   |
   = help: `LogLevel` and `with_logs()` were removed in v3.0.0; install your log subscriber in `main()` instead
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

---

## DeGene → RealGene rename

**What changed (Phase 56):** The `DeGene` trait was hard-renamed to `RealGene` and relocated to
`src/traits/real_gene.rs`. The import path changes from `genetic_algorithms::traits::DeGene` to
`genetic_algorithms::traits::RealGene`. The trait interface is identical except for the name; two
new methods `bounds() -> Option<(f64, f64)>` gained a default `None` implementation and do not
require changes in existing impls.

**Who is affected:** Anyone who implemented `DeGene` on a custom gene type for use with `DeEngine`,
`CmaEngine`, `ScatterEngine`, or any other real-valued engine.

### Before

```rust
use genetic_algorithms::traits::DeGene;

impl DeGene for MyGene {
    fn real_value(&self) -> f64 { self.value }
    fn with_real_value(&self, v: f64) -> Self { MyGene { value: v } }
}
```

### After

```rust
use genetic_algorithms::traits::RealGene;

impl RealGene for MyGene {
    fn real_value(&self) -> f64 { self.value }
    fn with_real_value(&self, v: f64) -> Self { MyGene { value: v } }
    // bounds() is optional — default returns None
}
```

### Compiler error

```
error[E0412]: cannot find trait `DeGene` in module `genetic_algorithms::traits`
  --> src/main.rs:3:34
   |
3  | use genetic_algorithms::traits::DeGene;
   |                                 ^^^^^^ not found in `genetic_algorithms::traits`
   |
   = help: there is a trait with a similar name: `RealGene`
```

**Fix:** Global search-and-replace `DeGene` → `RealGene` in all `use` statements and `impl` blocks.

---

## SelectionOperator::select — new num_parents parameter

**What changed (Phase 54):** `SelectionOperator::select` gained a fourth parameter `num_parents: usize`
to support N-ary (multi-parent) crossover operators. The return type changed from `Vec<(usize, usize)>`
(pairs) to `Vec<Vec<usize>>` (groups of arbitrary size). Built-in selection strategies were updated
automatically. Custom implementors of `SelectionOperator` must update their method signature.

**Who is affected:** Anyone who wrote a custom `SelectionOperator` implementation.

### Before

```rust
impl SelectionOperator for MySelection {
    fn select<U: ChromosomeT + Sync + Send + 'static + Clone>(
        &self,
        chromosomes: &[U],
        number_of_couples: usize,
        number_of_threads: usize,
    ) -> Vec<(usize, usize)> {
        vec![(0, 1), (2, 3)]
    }
}
```

### After

```rust
impl SelectionOperator for MySelection {
    fn select<U>(
        &self,
        chromosomes: &[U],
        number_of_couples: usize,
        number_of_threads: usize,
        num_parents: usize,   // new — number of parents per group
    ) -> Vec<Vec<usize>>      // pairs become Vec<Vec<usize>>
    where
        U: ChromosomeT + Sync + Send + 'static + Clone,
    {
        // For standard 2-parent crossover: return vec![vec![0, 1], vec![2, 3]]
        vec![vec![0, 1], vec![2, 3]]
    }
}
```

### Compiler error

```
error[E0053]: method `select` has an incompatible type for trait
  --> src/my_selection.rs:5:5
   |
5  |     fn select<U>(&self, chromosomes: &[U], number_of_couples: usize, number_of_threads: usize) -> Vec<(usize, usize)>
   |                                                                                                    ^^^^^^^^^^^^^^^^^^
   |                                            expected `fn(&Self, &[U], usize, usize, usize) -> Vec<Vec<usize>>`
   |                                                 found `fn(&Self, &[U], usize, usize) -> Vec<(usize, usize)>`
```

**Fix:** Add `num_parents: usize` as the fourth parameter; change return type to `Vec<Vec<usize>>`; wrap each pair `(a, b)` as `vec![a, b]`. For multi-parent operators, collect `num_parents` indices per group instead of two.

---

## Mutation enum variant parameter changes

**What changed (Phase 48 / variable-length support):** Several `Mutation` variants were restructured
between v2 and v3. The most impactful change is the renaming of the permutation-insert variant.
In v2, `Mutation::Insertion` referred to the permutation-preserving operation (remove a gene, reinsert
elsewhere). In v3, this variant is renamed to `Mutation::PermutationInsert`, and a new `Mutation::Insertion`
is added for variable-length chromosome growth. Using the old `Mutation::Insertion` for permutation
moves will silently misbehave — `Mutation::Insertion` in v3 has a **different meaning**.

Numeric-parameter variants (`step`, `sigma`, `eta`, `b`, `scale`, `alpha`) moved from being passed via
`mutation_step`/`mutation_sigma` builder fields to being inline struct fields on each variant.

**Who is affected:** Anyone using `Mutation::Insertion` for permutation chromosomes, or anyone who
configured `Gaussian`, `Creep`, `Polynomial`, or `NonUniform` mutations with `.with_mutation_sigma()`
or `.with_mutation_step()`.

### Before

```rust
use genetic_algorithms::operations::Mutation;

// Permutation insertion in v2:
.with_mutation_method(Mutation::Insertion)

// Gaussian with sigma via separate builder:
.with_mutation_method(Mutation::Gaussian)
.with_mutation_sigma(0.5)

// Creep with step via separate builder:
.with_mutation_method(Mutation::Creep)
.with_mutation_step(0.01)
```

### After

```rust
use genetic_algorithms::operations::Mutation;

// Permutation insertion in v3 (renamed):
.with_mutation_method(Mutation::PermutationInsert)
// Mutation::Insertion now means: insert a new gene (variable-length chromosomes only)

// Gaussian with inline sigma:
.with_mutation_method(Mutation::Gaussian { sigma: Some(0.5) })

// Creep with inline step:
.with_mutation_method(Mutation::Creep { step: Some(0.01) })
```

### Compiler error

```
error[E0599]: no variant or associated item named `Insertion` found for enum `Mutation` in the current scope
  --> src/main.rs:12:37
   |
12 |     .with_mutation_method(Mutation::Insertion)
   |                                     ^^^^^^^^^ variant not found in `Mutation` (for permutation use; `Insertion` now means variable-length growth)
   |
   = help: there is a variant with a similar name: `PermutationInsert`

error[E0308]: mismatched types
  --> src/main.rs:14:37
   |
14 |     .with_mutation_method(Mutation::Creep)
   |                                     ^^^^^ help: use struct syntax: `Mutation::Creep { step: None }`
```

**Fix table:**

| v2 code | v3 replacement |
|---------|---------------|
| `Mutation::Insertion` (permutation move) | `Mutation::PermutationInsert` |
| `Mutation::Gaussian` + `.with_mutation_sigma(x)` | `Mutation::Gaussian { sigma: Some(x) }` |
| `Mutation::Creep` + `.with_mutation_step(x)` | `Mutation::Creep { step: Some(x) }` |
| `Mutation::Polynomial` + eta via config | `Mutation::Polynomial { eta: Some(x) }` |
| `Mutation::NonUniform` + b via config | `Mutation::NonUniform { b: Some(x) }` |
| `.with_mutation_step(x)` (standalone) | Remove — inline in variant |
| `.with_mutation_sigma(x)` (standalone) | Remove — inline in variant |

---

## parallel feature — rayon is now optional

**What changed (Phase 69):** A new `parallel` feature flag (default-on) gates the `rayon` dependency.
Users who previously had `rayon` as a mandatory transitive dependency can now opt out and shed `rayon`
and `crossbeam` for embedded / wasm-only / ultra-lean builds. The default `cargo add genetic_algorithms`
experience is unchanged — `parallel` and `logging` are both default-on.

**Who is affected:** Users targeting `wasm32-unknown-unknown` or embedded environments who want to drop
the rayon transitive deps. Standard native users: no action required.

### Opting out

Disable both `default-features` and re-enable only the features you want:

```toml
[dependencies]
genetic_algorithms = { version = "3", default-features = false, features = ["logging"] }
```

When the `parallel` feature is disabled, every `par_iter()` / `par_chunks*()` call-site in the library
is replaced at compile time by a sequential `iter()` / `chunks*()` fallback. The canonical gate pattern
(also enforced for `wasm32`) is:

```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
let results: Vec<_> = items.par_iter().map(|x| process(x)).collect();
#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
let results: Vec<_> = items.iter().map(|x| process(x)).collect();
```

See `CLAUDE.md` §"WASM Compatibility" and `.planning/intel/parallel-feature.md` for the full rationale.

---

## logging feature — log crate is now optional

**What changed (Phase 68):** A new `logging` feature flag (default-on) gates the `log` crate dependency.
Users who do not want any logging machinery — and want to shed the `log` crate from their transitive dep
graph entirely — can disable it. This is distinct from the `## Logger setup (v2 auto-init → v3 explicit)`
section above, which covers the related removal of the implicit `env_logger::init()` call inside `Ga::run()`.

**Who is affected:** Users targeting `wasm32-unknown-unknown` or embedded environments who want to drop
the `log` crate. Standard users: no action required.

### Opting out

Disable both `default-features` and re-enable only the features you want:

```toml
[dependencies]
genetic_algorithms = { version = "3", default-features = false, features = ["parallel"] }
```

When the `logging` feature is disabled, every internal `crate::log_info!`, `crate::log_debug!`,
`crate::log_trace!`, `crate::log_warn!`, `crate::log_error!` macro expands to `()` instead of `::log::*`
— the library emits no log events and the `log` crate is dropped from the transitive dep graph.
`LogObserver` is also unavailable in this configuration (it depends on the `log` crate); construct your
own `GaObserver<U>` implementor if you need event hooks without the `log` crate.

See `.planning/intel/feature-flags.md` and `.planning/intel/logger-history.md` for the full rationale.
