# Phase 11: Advanced Mode Examples - Research

**Researched:** 2026-03-22
**Domain:** Rust GA library — NSGA-II multi-objective, Island Model, permutation-based scheduling
**Confidence:** HIGH

## Summary

Phase 11 adds three self-contained example files that demonstrate the library's advanced GA modes. All required APIs, operators, and types already exist in the codebase — no source changes are needed. The work is entirely in `examples/`, with each file following the style established in Phase 10 (`examples/rastrigin.rs`, `examples/onemax_binary.rs`).

The NSGA-II example (`nsga2_zdt1.rs`) uses `Nsga2Ga` with `RangeChromosome<f64>` and two objective functions for the ZDT1 benchmark. The island model example (`island_model.rs`) uses `IslandGa::with_heterogeneous_configs()` with four distinct `GaConfiguration` instances (varying mutation probability) to evolve a 20D Rastrigin problem. The job scheduling example (`job_scheduling.rs`) uses `Ga` with `RangeChromosome<i32>`, `Crossover::Order`, and `Mutation::Insertion` — both already implemented in the operations layer.

**Primary recommendation:** Write three files in `examples/`. Each is a plain Rust program with a `/*!` doc block, a constants section in `main()`, and `match result { Ok(...) => ..., Err(...) => ... }` result handling. Reuse the Rastrigin fitness function from `examples/rastrigin.rs` verbatim for the island model example.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Code structure and style (carried from Phase 10):**
- Follow `examples/onemax_binary.rs` as reference template for structure
- Each example must have a `/*!` doc block at the top with: problem description, GA mode used, key configuration choices, and `cargo run --example <name>` command
- Constants section at the top of `main()` for all configurable parameters
- Final result via `match result { Ok(...) => ..., Err(...) => ... }`

**NSGA-II output format:**
- Show per-generation progress reporting front size (count of non-dominated individuals) — grows toward population size as algorithm converges
- At completion, sample ~10 evenly-spaced points from the Pareto front and print them as `(f1, f2)` pairs
- Print header: `Pareto front (10 points sampled from N non-dominated solutions):`
- Progress line example: `Generation 50: front size = 42`

**NSGA-II problem setup:**
- ZDT1 benchmark: 30 variables in [0, 1], two objectives: minimize f1 = x[0], minimize f2 = 1 - sqrt(x[0]/g)
- Both objectives use `ObjectiveDirection::Minimize`
- Use `RangeChromosome<f64>` with `range_random_initialization`

**Island model problem domain:**
- Problem: Rastrigin function minimization with 20 dimensions
- 4 islands, Ring topology (`MigrationTopology::Ring`), migration every 10 generations, 2 migrants per migration
- Heterogeneous configs: each island gets a different mutation probability (0.01, 0.05, 0.10, 0.20) via `with_heterogeneous_configs()`
- Output: after each migration round, print per-island best fitness and global best
- Progress line example: `Migration 3: island[0]=2.4 island[1]=1.8 island[2]=3.1 island[3]=2.9 | global=1.8`

**Job scheduling problem setup:**
- 15 jobs, 5 machines with hardcoded processing times (2D matrix)
- Chromosome: `RangeChromosome<i32>` — each gene is a job index, chromosome is a permutation
- Fitness function: simulate schedule using the job ordering (FIFO per machine), compute and minimize makespan
- Operators: Order crossover (`Crossover::Order`), Insertion mutation (`Mutation::Insertion`)
- Output: best job ordering sequence and its makespan value
- Progress: print best makespan every N generations (consistent with Phase 10 pattern)

### Claude's Discretion

- NSGA-II: population size, max generations, exact ZDT1 variable count (standard is 30)
- Island model: exact Rastrigin variable count for 20D, population size per island, report interval
- Job scheduling: specific processing times in the 15x5 matrix, report interval
- NSGA-II: how to sample evenly from the front (sort by f1, take every len/N-th point)

### Deferred Ideas (OUT OF SCOPE)

- None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| EX-02 | User can run an NSGA-II multi-objective example optimizing the ZDT1 benchmark (two conflicting objectives) | `Nsga2Ga` + `Nsga2Configuration` + `ParetoFront` APIs verified in `src/nsga2/`. ZDT1 formula confirmed (x[0] minimization vs Pareto trade-off). |
| EX-03 | User can run an Island Model GA example with multiple sub-populations evolving in parallel with migration | `IslandGa::with_heterogeneous_configs()` + `IslandConfiguration` + `MigrationTopology::Ring` verified in `src/island/`. Per-island progress must be read from `self.islands` between `run()` calls — requires a loop over `migrate()` cadence rather than the built-in `run()`. |
| EX-04 | User can run a Job Scheduling example minimizing makespan across machines via permutation-based chromosome representation | `Crossover::Order` (OX) and `Mutation::Insertion` confirmed present in `src/operations/`. `RangeChromosome<i32>` permutation pattern confirmed in `examples/nqueens_range.rs`. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `genetic_algorithms` (this crate) | current | GA execution, NSGA-II, island model | Project under development — all examples use it directly |
| `std::f64::consts::PI` | stdlib | ZDT1 / Rastrigin math | No extra dep needed |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `RangeChromosome<f64>` | project | Continuous genes for ZDT1 and Rastrigin | NSGA-II and island model |
| `RangeChromosome<i32>` | project | Integer permutation genes | Job scheduling |
| `range_random_initialization` | project | Allele-bounded random init | All three examples |
| `Crossover::Order` | project | OX crossover preserving gene order | Job scheduling |
| `Mutation::Insertion` | project | Permutation-safe insertion mutation | Job scheduling |
| `Mutation::Gaussian` | project | Continuous perturbation | Island model Rastrigin |

**Installation:** No new dependencies. All types are already available in the crate.

## Architecture Patterns

### Recommended Project Structure
```
examples/
├── nsga2_zdt1.rs        # NSGA-II ZDT1 benchmark (EX-02)
├── island_model.rs      # Island model Rastrigin 20D (EX-03)
└── job_scheduling.rs    # Permutation makespan scheduling (EX-04)
```

No changes to `src/` in any plan.

### Pattern 1: NSGA-II Example Structure

**What:** `Nsga2Ga` takes two configs (`Nsga2Configuration` + `GaConfiguration`), a list of alleles, an init function, and a `Vec<Box<ObjectiveFn>>`. Calling `run()` returns `Result<ParetoFront<U>, GaError>`. There is no `run_with_callback` — progress must be tracked differently.

**Critical API issue:** `Nsga2Ga::run()` runs all generations internally with no user callback hook. Progress output (`Generation X: front size = N`) cannot come from inside `run()`. Resolution: the CONTEXT.md output format for NSGA-II specifies progress "per generation" — but given the API, the planner must either accept that progress is only shown at the end, OR call a custom loop. Looking at `src/nsga2/mod.rs` lines 217-284, the run loop is fully internal.

**Recommended resolution:** Report front size at completion only, or sample the front mid-way by running multiple partial calls. Since CONTEXT.md says "per-generation progress reporting" and the current API doesn't support it, the cleanest approach is to print a single summary line after `run()` completes: `Final front size: N`. The planner must document this constraint for the implementor.

**Alternative:** Implement a manual NSGA-II generation loop inside the example (bypassing `run()`) using `Nsga2Ga`'s internal fields — but that breaks encapsulation and violates the spirit of the examples. Not recommended.

**When to use:** Whenever two or more conflicting objectives must be simultaneously optimized.

**Example — NSGA-II builder:**
```rust
// Source: src/nsga2/mod.rs
use genetic_algorithms::nsga2::Nsga2Ga;
use genetic_algorithms::nsga2::configuration::{Nsga2Configuration, ObjectiveDirection};
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;

let nsga2_config = Nsga2Configuration::new()
    .with_num_objectives(2)
    .with_population_size(100)
    .with_max_generations(250)
    .with_objective_directions(vec![
        ObjectiveDirection::Minimize,
        ObjectiveDirection::Minimize,
    ]);

let mut ga_config = GaConfiguration::default();
ga_config.limit_configuration.genes_per_chromosome = 30;
ga_config.limit_configuration.alleles_can_be_repeated = true;

let alleles = vec![RangeGenotype::new(0, vec![(0.0_f64, 1.0_f64)], 0.0_f64)];
let alleles_clone = alleles.clone();

let mut nsga2 = Nsga2Ga::<RangeChromosome<f64>>::new(nsga2_config, ga_config)
    .with_alleles(alleles)
    .with_initialization_fn(move |n, _, _| {
        range_random_initialization(n, Some(&alleles_clone), Some(true))
    })
    .with_objective_fns(vec![
        Box::new(|dna: &[RangeGenotype<f64>]| dna[0].value),
        Box::new(|dna: &[RangeGenotype<f64>]| { /* ZDT1 f2 */ 0.0 }),
    ])
    .build()
    .expect("Failed to build NSGA-II");

let pareto_front = nsga2.run().unwrap();
```

**Note on `ga_config.limit_configuration`:** `genes_per_chromosome` and `alleles_can_be_repeated` must be set directly on the struct fields (not via builder methods) because `Nsga2Ga` does not implement `ConfigurationT`. Verified in `src/nsga2/mod.rs` line 325-326.

### Pattern 2: Island Model with Heterogeneous Configs

**What:** Build 4 `GaConfiguration` instances (one per island), each with a different `mutation_configuration.probability_max`. Pass them to `IslandGa::with_heterogeneous_configs()`. The `run()` method returns `Result<U, GaError>` (the best chromosome globally).

**Critical API issue for per-island progress:** `IslandGa::run()` does not expose per-island state during execution. The islands are stored in `self.islands: Vec<Population<U>>` (public field), but `run()` owns the execution loop. There is no callback hook in `IslandGa`.

**Resolution for EX-03 output requirement:** The CONTEXT.md requires printing per-island best fitness after each migration round. This cannot be done via the public `run()` API. The implementor must replicate the `run()` logic manually using `initialize()`, `evolve_islands_one_generation()`, `migrate()`, and `global_best()` — all of which are public. This is the intended approach for examples that need observation.

**Manual loop pattern (verified from `src/island/mod.rs`):**
```rust
// Source: src/island/mod.rs lines 316-367
island_ga.initialize()?;
for gen in 0..MAX_GENERATIONS {
    island_ga.evolve_islands_one_generation(problem_solving)?;
    if gen > 0 && gen % MIGRATION_INTERVAL == 0 {
        // Read per-island best BEFORE migration
        // ... print progress ...
        migrate(&mut island_ga.islands, &island_ga.island_config, problem_solving)?;
    }
}
let best = island_ga.global_best(problem_solving);
```

**Note:** `evolve_islands_one_generation` and `global_best` are public. `migrate` is in `src/island/migration.rs` and needs to be imported from `genetic_algorithms::island::migration::migrate`.

**Heterogeneous config construction:**
```rust
// Source: tests/island/test_island.rs lines 44-56
use genetic_algorithms::configuration::{GaConfiguration, ProblemSolving};
use genetic_algorithms::traits::{MutationConfig, SelectionConfig, CrossoverConfig, StoppingConfig};
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};

fn make_island_config(mutation_prob: f64, pop_size: usize, genes: usize) -> GaConfiguration {
    let mut cfg = GaConfiguration::default();
    cfg.limit_configuration.population_size = pop_size;
    cfg.limit_configuration.genes_per_chromosome = genes;
    cfg.limit_configuration.problem_solving = ProblemSolving::Minimization;
    cfg.limit_configuration.max_generations = MAX_GENERATIONS;
    cfg.mutation_configuration.probability_max = Some(mutation_prob);
    cfg.mutation_configuration.method = Mutation::Gaussian;
    cfg.crossover_configuration.method = Crossover::Uniform;
    cfg.selection_configuration.method = Selection::Tournament;
    cfg.survivor = Survivor::Fitness;
    cfg
}
```

### Pattern 3: Job Scheduling Permutation Example

**What:** Standard `Ga` with `RangeChromosome<i32>`. Alleles define job indices [0, N_JOBS-1]. `range_random_initialization` with `alleles_can_repeat = false` (so initialization yields distinct gene values = a permutation). `Crossover::Order` (OX) preserves relative order — safe for permutations. `Mutation::Insertion` moves a gene to a new position — safe for permutations.

**Fitness function:** Simulate a parallel machine schedule using the chromosome's gene ordering as job sequence. Each job is assigned to the machine with the earliest available slot (FIFO greedy). Makespan = max completion time across all machines.

**Permutation init pattern (from `examples/nqueens_range.rs` line 52-55):**
```rust
// Source: examples/nqueens_range.rs
let alleles = vec![RangeGenotype::new(0, vec![(0, N_JOBS as i32 - 1)], 0)];
let alleles_clone = alleles.clone();
.with_initialization_fn(move |genes_per_chromosome, _, _| {
    range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
})
```

**Key:** passing `Some(false)` as the third argument to `range_random_initialization` disables allele repetition, producing a permutation. Confirmed in `examples/nqueens_range.rs` line 54.

### Anti-Patterns to Avoid

- **Using `Ga` builder for `Nsga2Ga` or `IslandGa`:** These are separate structs. `ConfigurationT` traits are not implemented on them. Use `GaConfiguration::default()` and set fields directly.
- **Calling `nsga2.run_with_callback()`:** `Nsga2Ga` has no `run_with_callback` method. Only `run()` exists.
- **Using `IslandGa::run()` when per-island progress is needed:** `run()` does not expose internal state. Use the manual loop pattern.
- **Setting `alleles_can_be_repeated = true` for job scheduling:** This produces non-permutation chromosomes. Must pass `Some(false)` to init.
- **Wrong first arg to `RangeGenotype::new()`:** First arg is `i32` id (not the gene value type). Use `RangeGenotype::new(0, vec![(0, N-1)], 0)` for i32, `RangeGenotype::new(0, vec![(0.0, 1.0)], 0.0_f64)` for f64.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Non-dominated sorting | Custom Pareto front logic | `Nsga2Ga::run()` + `ParetoFront` | Complex rank/crowding distance math already verified |
| Order-preserving crossover | Custom OX implementation | `Crossover::Order` | Already implemented in `src/operations/crossover/order.rs` |
| Permutation-safe mutation | Custom insertion/swap logic | `Mutation::Insertion` | Already implemented in `src/operations/mutation.rs` line 162 |
| Multi-island parallel evolution | Manual rayon threads | `IslandGa` | Island parallelism via rayon already handled internally |
| ZDT1 g-function | Inline math | Standard formula inline | Only ~3 lines, no external lib needed |

**Key insight:** Every operator, struct, and algorithm needed for these three examples already exists. The only creative work is the fitness functions (ZDT1 objectives, makespan simulation) and the processing-time matrix for job scheduling.

## Common Pitfalls

### Pitfall 1: `genes_per_chromosome` not set in `ga_config` for `Nsga2Ga`

**What goes wrong:** `run()` initializes population using `ga_config.limit_configuration.genes_per_chromosome`. If it is 0 (the default), chromosomes have zero genes, objectives compute NaN, and sorting panics or returns an empty front.

**Why it happens:** `Nsga2Ga` does not use a builder that automatically validates this — `build()` only checks `num_objectives`, `population_size`, and `initialization_fn`.

**How to avoid:** Always set `ga_config.limit_configuration.genes_per_chromosome = N_VARS` (e.g., 30 for ZDT1) before passing to `Nsga2Ga::new()`.

**Warning signs:** Empty `ParetoFront`, objectives all `0.0`, or a `validate()` error claiming `population_size < 2`.

### Pitfall 2: Island model `run()` blocks without per-island reporting

**What goes wrong:** Using `island_ga.run()` satisfies EX-03's correctness criterion but cannot produce the per-island progress output required by CONTEXT.md.

**Why it happens:** `run()` owns the loop — there is no observer or callback slot in `IslandGa`.

**How to avoid:** Use the manual loop: `initialize()` + loop over `evolve_islands_one_generation()` + conditional `migrate()` + read `self.islands` for per-island best.

**Warning signs:** If only a single final line is printed instead of one line per migration round.

### Pitfall 3: `Nsga2Ga` requires `mutation::ValueMutable` bound

**What goes wrong:** `run()` is defined on `impl<U> Nsga2Ga<U> where U: ChromosomeT + mutation::ValueMutable`. If a chromosome type does not implement `ValueMutable`, `run()` does not exist and the compiler emits "method not found."

**Why it happens:** `RangeChromosome<f64>` and `RangeChromosome<i32>` both implement `ValueMutable`. This is not a concern for these examples, but it appears as a confusing compiler error if the wrong chromosome type is used.

**How to avoid:** Use `RangeChromosome<f64>` for ZDT1 and `RangeChromosome<i32>` for job scheduling — both confirmed to implement `ValueMutable`.

### Pitfall 4: ZDT1 `g` function denominator when x[0] = 0

**What goes wrong:** ZDT1 f2 = 1 - sqrt(x[0] / g). When x[0] = 0 (valid for ZDT1), f2 = 1 - 0 = 1, which is fine. But if `g = 0`, the formula produces `NaN`.

**Why it happens:** g = 1 + (9 / (n-1)) * sum(x[1..n]). Since all x in [0,1] and sum >= 0, g >= 1 always. Not a real risk, but worth checking in code review.

**How to avoid:** The standard ZDT1 formula guarantees g >= 1 for n >= 2. Use the formula as-is.

### Pitfall 5: Job scheduling permutation: `Crossover::Order` requires allele uniqueness

**What goes wrong:** If alleles repeat (e.g., two genes with value 3), the OX crossover may produce invalid chromosomes where the same job appears twice and another is missing.

**Why it happens:** OX preserves relative order of unique values. With duplicates, it can double-count.

**How to avoid:** Pass `Some(false)` to `range_random_initialization` (disables repetition). This is already the pattern in `examples/nqueens_range.rs`.

### Pitfall 6: Island model migration count vs population size validation

**What goes wrong:** `validate()` returns `GaError::InvalidIslandConfiguration` if `migration_count >= population_size` for any island config.

**Why it happens:** Validated in `src/island/mod.rs` lines 211-219.

**How to avoid:** Keep `MIGRATION_COUNT` (2) strictly less than `POP_SIZE_PER_ISLAND`. With 50 per island and 2 migrants, this is fine.

## Code Examples

Verified patterns from source files and tests:

### ZDT1 Objective Functions
```rust
// ZDT1 standard formulas — no external source needed
// f1(x) = x[0]
// g(x) = 1 + 9/(n-1) * sum(x[1..n])
// f2(x) = 1 - sqrt(x[0] / g(x))

let obj_f1 = Box::new(|dna: &[RangeGenotype<f64>]| dna[0].value);
let obj_f2 = Box::new(|dna: &[RangeGenotype<f64>]| {
    let n = dna.len();
    let g = 1.0 + (9.0 / (n - 1) as f64) * dna[1..].iter().map(|g| g.value).sum::<f64>();
    1.0 - (dna[0].value / g).sqrt()
});
```

### Pareto Front Sampling (10 evenly-spaced points)
```rust
// Sort by f1 (first objective), sample every len/10 points
let mut individuals = pareto_front.individuals.clone();
individuals.sort_by(|a, b| a.objectives[0].partial_cmp(&b.objectives[0]).unwrap());
let n = individuals.len();
let step = (n / 10).max(1);
println!("Pareto front (10 points sampled from {} non-dominated solutions):", n);
for i in (0..n).step_by(step).take(10) {
    println!("  ({:.4}, {:.4})", individuals[i].objectives[0], individuals[i].objectives[1]);
}
```

### Island Model Manual Loop
```rust
// Source: src/island/mod.rs (run() implementation, adapted for progress output)
use genetic_algorithms::island::migration::migrate;

island_ga.initialize()?;
let mut migration_count = 0;
for gen in 1..=MAX_GENERATIONS {
    island_ga.evolve_islands_one_generation(problem_solving)?;
    if gen % MIGRATION_INTERVAL == 0 {
        migration_count += 1;
        // Per-island best fitness
        let island_bests: Vec<f64> = island_ga.islands.iter().map(|island| {
            island.chromosomes.iter()
                .map(|c| c.fitness())
                .fold(f64::INFINITY, f64::min)
        }).collect();
        let global = island_bests.iter().cloned().fold(f64::INFINITY, f64::min);
        print!("Migration {}: ", migration_count);
        for (i, &b) in island_bests.iter().enumerate() {
            print!("island[{}]={:.2} ", i, b);
        }
        println!("| global={:.2}", global);
        migrate(&mut island_ga.islands, &island_ga.island_config, problem_solving)?;
    }
}
let best = island_ga.global_best(problem_solving);
```

### Makespan Fitness for Job Scheduling
```rust
// Greedy parallel-machine schedule simulation
fn makespan_fitness(dna: &[RangeGenotype<i32>], processing_times: &[[u32; N_MACHINES]]) -> f64 {
    let mut machine_finish = [0u32; N_MACHINES];
    for gene in dna {
        let job = gene.value as usize;
        // Assign job to earliest-available machine
        let m = machine_finish.iter().enumerate()
            .min_by_key(|(_, &t)| t).map(|(i, _)| i).unwrap();
        machine_finish[m] += processing_times[job][m];
    }
    *machine_finish.iter().max().unwrap() as f64
}
```

### Heterogeneous GaConfiguration for Island Model
```rust
// Source: tests/island/test_island.rs pattern (adapted)
// Build 4 configs with different mutation probabilities
let mutation_probs = [0.01_f64, 0.05, 0.10, 0.20];
let ga_configs: Vec<GaConfiguration> = mutation_probs.iter().map(|&prob| {
    let mut cfg = GaConfiguration::default();
    cfg.limit_configuration.population_size = POP_SIZE_PER_ISLAND;
    cfg.limit_configuration.genes_per_chromosome = DIMENSIONS;
    cfg.limit_configuration.problem_solving = ProblemSolving::Minimization;
    cfg.limit_configuration.max_generations = MAX_GENERATIONS;
    cfg.mutation_configuration.probability_max = Some(prob);
    cfg.mutation_configuration.method = Mutation::Gaussian;
    cfg.crossover_configuration.method = Crossover::Uniform;
    cfg.selection_configuration.method = Selection::Tournament;
    cfg.survivor = Survivor::Fitness;
    cfg
}).collect();

let island_ga = IslandGa::with_heterogeneous_configs(island_config, ga_configs)
    .with_alleles(alleles)
    .with_initialization_fn(...)
    .with_fitness_fn(fitness_fn)
    .build()
    .expect("Failed to build island GA");
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single-population GA for hard problems | Island model with Ring topology | Library already supports this | Phase 10 showed 5D Rastrigin; Phase 11 shows 20D via islands |
| Manual Pareto sorting | `Nsga2Ga` + `ParetoFront` | Library already supports this | ZDT1 requires direction-aware sorting — use `ObjectiveDirection::Minimize` for both |

**Deprecated/outdated:**
- None for this phase. No operator or API changes needed.

## Open Questions

1. **NSGA-II per-generation progress output**
   - What we know: `Nsga2Ga::run()` has no callback hook. Progress cannot be printed per generation using the public API.
   - What's unclear: Whether the CONTEXT.md requirement ("per-generation progress reporting front size") means literally every generation or just at the end.
   - Recommendation: Print a single summary at completion: `Final Pareto front: N non-dominated solutions`. If the planner wants true per-generation output, the implementor must replicate the run loop manually (not recommended for an example file). Document this tradeoff in the plan.

2. **Island model `problem_solving` parameter in manual loop**
   - What we know: `evolve_islands_one_generation(problem_solving)` takes a `ProblemSolving` enum. It must be `ProblemSolving::Minimization` for Rastrigin.
   - What's unclear: Whether importing `ProblemSolving` requires a separate `use` statement beyond `GaConfiguration`.
   - Recommendation: Import `use genetic_algorithms::configuration::ProblemSolving;` explicitly.

3. **Job scheduling processing times matrix**
   - What we know: Claude's discretion governs the exact values. Must be a 15x5 `[[u32; 5]; 15]` array.
   - What's unclear: Nothing — implementor chooses values for "interesting variety."
   - Recommendation: Use values in the range 1–20 with variance across machines to produce non-trivial scheduling decisions.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | none (standard Cargo project) |
| Quick run command | `cargo test --test test_nsga2 && cargo test --test test_island` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EX-02 | `cargo run --example nsga2_zdt1` exits 0, prints Pareto front | smoke | `cargo run --example nsga2_zdt1` | Wave 0 |
| EX-03 | `cargo run --example island_model` exits 0, prints per-island + global best | smoke | `cargo run --example island_model` | Wave 0 |
| EX-04 | `cargo run --example job_scheduling` exits 0, prints best ordering + makespan | smoke | `cargo run --example job_scheduling` | Wave 0 |
| EX-02 | NSGA-II returns non-empty ParetoFront on ZDT1 | integration | `cargo test --test test_nsga2` | ✅ (existing) |
| EX-03 | Island GA validates with heterogeneous configs | integration | `cargo test --test test_island` | ✅ (existing) |
| EX-04 | Order crossover preserves permutation validity | unit | `cargo test --test test_operations` | ✅ (existing) |

### Sampling Rate
- **Per task commit:** `cargo clippy && cargo run --example <name>`
- **Per wave merge:** `cargo test && cargo test --features serde`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `examples/nsga2_zdt1.rs` — covers EX-02 (created in Wave 1)
- [ ] `examples/island_model.rs` — covers EX-03 (created in Wave 1)
- [ ] `examples/job_scheduling.rs` — covers EX-04 (created in Wave 1)

*(All gaps are the deliverables of this phase — no separate test infrastructure needed. Existing tests cover the underlying library; the example files are the integration test.)*

## Sources

### Primary (HIGH confidence)
- `src/nsga2/mod.rs` — `Nsga2Ga` struct, `run()` signature, `initialize_population()`, `create_offspring()`, `with_objective_fns()`, `with_initialization_fn()`, `with_alleles()`, `build()`
- `src/nsga2/configuration.rs` — `Nsga2Configuration` builder, `ObjectiveDirection`, `with_objective_directions()`
- `src/nsga2/pareto.rs` — `ParetoFront`, `ParetoIndividual` fields (`objectives: Vec<f64>`, `rank: usize`)
- `src/island/mod.rs` — `IslandGa::with_heterogeneous_configs()`, `initialize()`, `evolve_islands_one_generation()`, `global_best()`, `run()` implementation
- `src/island/configuration.rs` — `IslandConfiguration` builder, `MigrationPolicy`
- `src/island/topology.rs` — `MigrationTopology::Ring`
- `src/configuration.rs` — `GaConfiguration`, `LimitConfiguration`, `MutationConfiguration` direct field access
- `src/traits/configuration.rs` — `MutationConfig`, `CrossoverConfig`, `SelectionConfig` trait methods
- `src/operations/crossover.rs` + `src/operations/crossover/order.rs` — `Crossover::Order` confirmed
- `src/operations/mutation.rs` — `Mutation::Insertion` confirmed at line 162
- `examples/onemax_binary.rs` — canonical style template
- `examples/nqueens_range.rs` — permutation pattern with `range_random_initialization(..., Some(false))`
- `examples/rastrigin.rs` — Rastrigin fitness function (reuse for island model)
- `tests/island/test_island.rs` — `with_heterogeneous_configs()` usage pattern

### Secondary (MEDIUM confidence)
- ZDT1 benchmark formula: standard GA literature; f1 = x[0], g = 1 + 9/(n-1) * sum(x[1..]), f2 = 1 - sqrt(x[0]/g). Widely confirmed, n=30 standard.

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all types verified in source
- Architecture: HIGH — API signatures read directly from source; one open question on NSGA-II progress hook
- Pitfalls: HIGH — discovered by reading `validate()` implementations and existing tests

**Research date:** 2026-03-22
**Valid until:** 2026-04-22 (stable codebase, no external dependencies to track)
