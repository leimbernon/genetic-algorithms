---
phase: quick-260327-kkp
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - examples/onemax_binary.rs
  - examples/onemax_extension.rs
  - examples/knapsack_binary.rs
  - examples/feature_selection.rs
  - examples/job_scheduling.rs
  - examples/nqueens_range.rs
  - examples/niching.rs
  - examples/nsga2_zdt1.rs
  - examples/rastrigin.rs
  - examples/island_model.rs
  - README.md
autonomous: true
requirements: []
must_haves:
  truths:
    - "All 10 examples compile with plain `cargo build --examples`"
    - "All 10 examples show observer usage in code"
    - "MetricsObserver in examples is gated with #[cfg(feature = \"observer-metrics\")]"
    - "README has a GaObserver section replacing the Reporter section"
    - "README notes Reporter is deprecated"
  artifacts:
    - path: "examples/onemax_binary.rs"
      provides: "LogObserver usage"
    - path: "examples/rastrigin.rs"
      provides: "CompositeObserver with optional MetricsObserver"
    - path: "examples/island_model.rs"
      provides: "CompositeObserver with IslandGaObserver-aware LogObserver and optional MetricsObserver"
    - path: "examples/nsga2_zdt1.rs"
      provides: "Nsga2Observer-aware LogObserver usage"
    - path: "README.md"
      provides: "GaObserver documentation section"
      contains: "GaObserver"
  key_links:
    - from: "examples/rastrigin.rs"
      to: "CompositeObserver"
      via: "CompositeObserver::new().add(Arc::new(LogObserver))"
    - from: "examples/island_model.rs"
      to: "IslandGa::with_observer"
      via: "Arc<dyn IslandGaObserver>"
    - from: "examples/nsga2_zdt1.rs"
      to: "Nsga2::with_observer"
      via: "Arc<dyn Nsga2Observer>"
---

<objective>
Add observer usage to all 10 existing examples and replace the deprecated `Reporter` section in README with a comprehensive `GaObserver` API reference.

Purpose: Observable GA runs are the modern API. Examples and README must show users the canonical path — LogObserver for simple cases, CompositeObserver for composition, sub-traits for engine-specific hooks, MetricsObserver gated for production telemetry.
Output: 10 updated example files + updated README.md
</objective>

<execution_context>
@/Users/luis/.claude/get-shit-done/workflows/execute-plan.md
@/Users/luis/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md

<!-- Observer public API summary (extracted from src/) -->
<interfaces>
<!-- Re-exported from src/lib.rs — use these import paths in examples -->
```
use genetic_algorithms::GaObserver;
use genetic_algorithms::IslandGaObserver;
use genetic_algorithms::Nsga2Observer;
use genetic_algorithms::AllObserver;
use genetic_algorithms::CompositeObserver;
use genetic_algorithms::LogObserver;
#[cfg(feature = "observer-metrics")]
use genetic_algorithms::MetricsObserver;
```

Builder methods:
- `Ga::with_observer(Arc<dyn GaObserver<U> + Send + Sync>) -> Self`  — src/ga.rs:549
- `IslandGa::with_observer(Arc<dyn IslandGaObserver<U> + Send + Sync>) -> Self`  — src/island/mod.rs:170
- `Nsga2::with_observer(Arc<dyn Nsga2Observer<U> + Send + Sync>) -> Self`  — src/nsga2/mod.rs:103

Observer types:
- `LogObserver` — zero-sized struct, implements GaObserver+IslandGaObserver+Nsga2Observer+AllObserver. Construct: `LogObserver` (no ::new needed)
- `CompositeObserver::<U>::new().add(Arc::new(LogObserver))` — builder, implements all four observer traits
- `MetricsObserver::new("run_id")` — #[cfg(feature = "observer-metrics")], implements AllObserver

CompositeObserver implements both IslandGaObserver and Nsga2Observer (fan-out).
LogObserver implements AllObserver — satisfies `.add()` on CompositeObserver.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add observer to all 10 examples</name>
  <files>
    examples/onemax_binary.rs,
    examples/onemax_extension.rs,
    examples/knapsack_binary.rs,
    examples/feature_selection.rs,
    examples/job_scheduling.rs,
    examples/nqueens_range.rs,
    examples/niching.rs,
    examples/nsga2_zdt1.rs,
    examples/rastrigin.rs,
    examples/island_model.rs
  </files>
  <action>
For every example, add observer usage. Observer distribution (per decisions):

**LogObserver only** (simple Ga examples — 7 examples):
`onemax_binary`, `onemax_extension`, `knapsack_binary`, `feature_selection`, `job_scheduling`, `nqueens_range`, `niching`

Pattern for these:
1. Add import: `use std::sync::Arc;` and `use genetic_algorithms::LogObserver;`
2. Pass `.with_observer(Arc::new(LogObserver))` to the `Ga::new(...)` builder chain
3. Remove any `with_reporter()` calls if present (none currently exist, but check)
4. Update the top-of-file doc comment "Features demonstrated" bullet to include "LogObserver lifecycle hooks"

**CompositeObserver + optional MetricsObserver** (`rastrigin` and `island_model`):

`rastrigin.rs`:
1. Add imports: `use std::sync::Arc;`, `use genetic_algorithms::{LogObserver, CompositeObserver};`
2. Add cfg-gated import: `#[cfg(feature = "observer-metrics")] use genetic_algorithms::MetricsObserver;`
3. Build composite before `Ga::new`:
   ```rust
   let composite = CompositeObserver::new()
       .add(Arc::new(LogObserver));
   #[cfg(feature = "observer-metrics")]
   let composite = composite.add(Arc::new(MetricsObserver::new("rastrigin")));
   ```
4. Pass `.with_observer(Arc::new(composite))` to the Ga builder
5. Update doc comment features list

`island_model.rs` — uses `IslandGa`, so:
1. Add imports: `use std::sync::Arc;`, `use genetic_algorithms::{LogObserver, CompositeObserver, IslandGaObserver};`
2. Add cfg-gated import: `#[cfg(feature = "observer-metrics")] use genetic_algorithms::MetricsObserver;`
3. Build composite:
   ```rust
   let composite = CompositeObserver::new()
       .add(Arc::new(LogObserver));
   #[cfg(feature = "observer-metrics")]
   let composite = composite.add(Arc::new(MetricsObserver::new("island_model")));
   ```
4. Pass `.with_observer(Arc::new(composite))` (CompositeObserver implements IslandGaObserver)
5. Update doc comment

**Nsga2Observer-aware LogObserver** (`nsga2_zdt1`):
1. Add imports: `use std::sync::Arc;`, `use genetic_algorithms::{LogObserver, Nsga2Observer};`
2. Pass `.with_observer(Arc::new(LogObserver))` to the Nsga2 builder (LogObserver implements Nsga2Observer)
3. Note in comment above the call: `// LogObserver implements Nsga2Observer — logs pareto-front and crowding events`
4. Update doc comment features list

After editing all 10, verify compilation:
`cargo build --examples 2>&1`
If any example fails to compile, fix before proceeding.
  </action>
  <verify>
    <automated>cargo build --examples 2>&1 | grep -E "^error|finished"</automated>
  </verify>
  <done>All 10 examples compile without errors. Each example file contains at least one `.with_observer(` call. MetricsObserver usage is enclosed in `#[cfg(feature = "observer-metrics")]` blocks in rastrigin and island_model.</done>
</task>

<task type="auto">
  <name>Task 2: Update README — replace Reporter section with GaObserver section</name>
  <files>README.md</files>
  <action>
In README.md:

1. **Table of Contents** — change the `[Reporter](#reporter)` link to `[Observer (GaObserver)](#observer-gaobserver)`.

2. **Replace the `### Reporter` section** (lines 93–101) with the following content. Keep the `### Visualization` section that follows intact.

```markdown
### Observer (GaObserver)

> **Note:** `Reporter<U>` is deprecated since 2.2.0 and will be removed in v3.0.0. Use `GaObserver` instead.

Attach a lifecycle observer to any GA engine via `.with_observer(Arc::new(my_observer))`. All hooks use `&self` — safe for use in rayon parallel regions. Zero overhead when no observer is attached (stored as `Option<Arc<_>>`).

#### Core trait: `GaObserver<U>`

Hooks fired by `Ga<U>` on every generation cycle:

| Hook | When it fires |
|------|--------------|
| `on_selection_complete` | After parent selection |
| `on_crossover_complete` | After crossover batch |
| `on_mutation_complete` | After mutation batch |
| `on_fitness_evaluation_complete` | After fitness re-evaluation |
| `on_survivor_selection_complete` | After survivor selection |
| `on_new_best` | When a new best chromosome is found |
| `on_stagnation` | When no improvement for N generations |
| `on_extension_triggered` | When diversity extension fires |
| `on_generation_end` | End of each generation (with `GenerationStats`) |
| `on_run_start` | Before the first generation |
| `on_run_end` | After the last generation |

#### Engine-specific sub-traits

- `IslandGaObserver<U>` — additional hooks for island migration events; attach via `IslandGa::with_observer`.
- `Nsga2Observer<U>` — additional hooks for NSGA-II pareto-front and crowding events; attach via `Nsga2::with_observer`.

#### Built-in observers

**`LogObserver`** — logs every hook via the `log` crate. No feature flags required. Implements `GaObserver`, `IslandGaObserver`, and `Nsga2Observer`.

```rust
use std::sync::Arc;
use genetic_algorithms::{Ga, LogObserver};

let ga = Ga::new(config, population)
    .with_observer(Arc::new(LogObserver))
    .run();
```

**`CompositeObserver<U>`** — fan-out observer that forwards all hooks to a list of inner observers. Inner observers must implement `AllObserver<U>` (a supertrait marker).

```rust
use std::sync::Arc;
use genetic_algorithms::{Ga, CompositeObserver, LogObserver};

let composite = CompositeObserver::new()
    .add(Arc::new(LogObserver));

let ga = Ga::new(config, population)
    .with_observer(Arc::new(composite))
    .run();
```

**`MetricsObserver`** — emits `metrics`-crate gauges, counters, and histograms per generation. Requires feature flag:

```toml
genetic_algorithms = { version = "2.2.0", features = ["observer-metrics"] }
```

```rust
#[cfg(feature = "observer-metrics")]
use genetic_algorithms::MetricsObserver;

#[cfg(feature = "observer-metrics")]
let composite = composite.add(Arc::new(MetricsObserver::new("my_run")));
```

Emitted metrics: `ga.generation.best_fitness`, `ga.generation.mean_fitness`, `ga.generation.diversity`, `ga.operator.*_ms` histograms, `ga.event.new_best` / `ga.event.stagnation` / `ga.event.extension_triggered` counters.

**`TracingObserver`** — emits `tracing`-crate spans and events. Requires feature flag:

```toml
genetic_algorithms = { version = "2.2.0", features = ["observer-tracing"] }
```

#### Custom observer

Implement `GaObserver<U>` (or `IslandGaObserver<U>` / `Nsga2Observer<U>`) on your own type. Only override the hooks you care about — all hooks have default no-op implementations:

```rust
use genetic_algorithms::{GaObserver, stats::GenerationStats};

struct MyObserver;

impl<U: genetic_algorithms::traits::ChromosomeT> GaObserver<U> for MyObserver {
    fn on_generation_end(&self, stats: &GenerationStats) {
        println!("Gen {} best={:.4}", stats.generation, stats.best_fitness);
    }
}
```
```

3. **No other README sections change.** Do not touch the Quick Example, Full Example, or Roadmap sections.

After editing, verify the README renders without obvious markdown issues:
`cargo doc --no-deps 2>&1 | grep -c "^warning"` (should be same or fewer warnings than before)
  </action>
  <verify>
    <automated>cargo build --examples 2>&1 | tail -3 && grep -c "GaObserver" README.md</automated>
  </verify>
  <done>README.md contains a `### Observer (GaObserver)` section with snippets for LogObserver, CompositeObserver, MetricsObserver, TracingObserver, and custom observer. The old `### Reporter` section is removed. Table of contents link updated. `grep -c "GaObserver" README.md` returns >= 5.</done>
</task>

</tasks>

<verification>
```bash
# Examples build cleanly (no MetricsObserver without feature flag)
cargo build --examples

# Examples build with observer-metrics feature
cargo build --examples --features observer-metrics

# Full test suite unaffected
cargo test

# README contains the new section
grep "### Observer (GaObserver)" README.md
grep "Reporter.*deprecated" README.md
```
</verification>

<success_criteria>
- `cargo build --examples` exits 0 — all 10 examples compile without feature flags
- `cargo build --examples --features observer-metrics` exits 0 — MetricsObserver paths compile
- `cargo test` exits 0 — no regressions
- Every example file contains `.with_observer(`
- README `### Reporter` section replaced by `### Observer (GaObserver)`
- README notes `Reporter<U>` is deprecated
</success_criteria>

<output>
After completion, create `.planning/quick/260327-kkp-add-observability-to-examples-and-docume/260327-kkp-SUMMARY.md`
</output>
