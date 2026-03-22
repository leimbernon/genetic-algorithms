# Phase 11: Advanced Mode Examples - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Three self-contained runnable examples in `examples/` using `Nsga2Ga` and `IslandGa`:
- `nsga2_zdt1.rs` — NSGA-II multi-objective on ZDT1 benchmark (EX-02)
- `island_model.rs` — island model parallel multi-population on Rastrigin (EX-03)
- `job_scheduling.rs` — permutation-based makespan minimization (EX-04)

Creating, editing, or moving other examples is out of scope.

</domain>

<decisions>
## Implementation Decisions

### Code structure and style (carried from Phase 10)
- Follow `examples/onemax_binary.rs` as reference template for structure
- Each example must have a `/*!` doc block at the top with: problem description, GA mode used, key configuration choices, and `cargo run --example <name>` command
- Constants section at the top of `main()` for all configurable parameters
- Final result via `match result { Ok(...) => ..., Err(...) => ... }`

### NSGA-II output format
- **API constraint (discovered during planning):** `Nsga2Ga::run()` has no callback hook — per-generation progress is not achievable without library changes. Decision revised: print final result only.
- At completion, sample ~10 evenly-spaced points from the Pareto front and print them as `(f1, f2)` pairs
- Print header: `Pareto front (10 points sampled from N non-dominated solutions):`
- Include a comment in the example doc block explaining that `Nsga2Ga` runs silently (no callback API)

### NSGA-II problem setup
- ZDT1 benchmark: 30 variables in [0, 1], two objectives: minimize f1 = x[0], minimize f2 = 1 - sqrt(x[0]/g)
- Both objectives use `ObjectiveDirection::Minimize`
- Use `RangeChromosome<f64>` with `range_random_initialization`

### Island model problem domain
- Problem: Rastrigin function minimization with 20 dimensions (harder than Phase 10's single-pop example — shows why island model helps)
- 4 islands, Ring topology (`MigrationTopology::Ring`), migration every 10 generations, 2 migrants per migration
- Heterogeneous configs: each island gets a different mutation probability (0.01, 0.05, 0.10, 0.20) via `with_heterogeneous_configs()` to demonstrate diversity-preserving evolution
- **API constraint (discovered during planning):** `IslandGa::evolve_islands_one_generation()` and `global_best()` are private — per-migration progress is not achievable without library changes. Decision revised: print final global best only.
- Output: after `run()` completes, print the global best fitness and chromosome
- Include a comment in the example doc block explaining that `IslandGa` evolves all islands internally with no mid-run observability via the current public API

### Job scheduling problem setup
- 15 jobs, 5 machines with hardcoded processing times (2D matrix)
- Chromosome: `RangeChromosome<i32>` (same pattern as `nqueens_range.rs`) — each gene is a job index, chromosome is a permutation representing job ordering
- Fitness function: simulate schedule using the job ordering (FIFO per machine), compute and minimize makespan
- Operators: Order crossover (`Crossover::Order`), Insertion mutation (`Mutation::Insertion`) — both designed for permutation encodings
- Output: best job ordering sequence and its makespan value
- Progress: print best makespan every N generations (consistent with Phase 10 pattern)

### Claude's Discretion
- NSGA-II: population size, max generations, exact ZDT1 variable count (standard is 30)
- Island model: exact Rastrigin variable count for 20D, population size per island, report interval
- Job scheduling: specific processing times in the 15×5 matrix (Claude's choice for interesting variety), report interval
- NSGA-II: how to sample evenly from the front (sort by f1, take every len/N-th point)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing examples (style reference)
- `examples/onemax_binary.rs` — Reference template for structure, doc block, callback, result handling
- `examples/nqueens_range.rs` — Reference for RangeChromosome<i32> permutation pattern (use for job scheduling)
- `examples/rastrigin.rs` — Reference for Rastrigin fitness function and RangeChromosome<f64> setup (reuse for island model)

### Library source — NSGA-II
- `src/nsga2/mod.rs` — Nsga2Ga API: new(), with_alleles(), with_initialization_fn(), with_objective_fns(), build(), run()
- `src/nsga2/configuration.rs` — Nsga2Configuration builder: with_num_objectives(), with_population_size(), with_max_generations(), with_objective_directions()
- `src/nsga2/pareto.rs` — ParetoFront and ParetoIndividual structs (individuals: Vec<ParetoIndividual<U>>, objectives: Vec<f64>)

### Library source — Island Model
- `src/island/mod.rs` — IslandGa API: new(), with_alleles(), with_initialization_fn(), with_fitness_fn(), with_heterogeneous_configs(), build(), run()
- `src/island/configuration.rs` — IslandConfiguration builder: with_num_islands(), with_migration_interval(), with_migration_count(), with_topology()
- `src/island/topology.rs` — MigrationTopology enum: Ring, FullyConnected, Grid, Hypercube, Custom

### Project requirements
- `.planning/REQUIREMENTS.md` — EX-02, EX-03, EX-04 acceptance criteria
- `.planning/ROADMAP.md` — Phase 11 success criteria

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `range_random_initialization` — direct use for NSGA-II (f64) and island model (f64) and job scheduling (i32)
- `examples/rastrigin.rs` — Rastrigin fitness fn can be copied/adapted for island model (20D version)
- `Mutation::Insertion` and `Crossover::Order` — already implemented in src/operations/, designed for permutation encodings

### Established Patterns
- NSGA-II does NOT use `run_with_callback` — it has its own `run()` returning `Result<ParetoFront<U>, GaError>`
- IslandGa uses `run()` returning `Result<U, GaError>` (returns best chromosome)
- IslandGa progress must be tracked differently — no built-in callback; may need to use migration_interval as reporting cadence
- `with_heterogeneous_configs()` takes `Vec<GaConfiguration>` — build N configs then pass them in

### Integration Points
- New files go in `examples/` directory — no changes to `src/`
- No new dependencies needed — all types already exist
- NSGA-II requires import of `genetic_algorithms::nsga2::{Nsga2Ga, configuration::Nsga2Configuration}`
- Island model requires import of `genetic_algorithms::island::{IslandGa, configuration::IslandConfiguration, topology::MigrationTopology}`

</code_context>

<specifics>
## Specific Ideas

- Island model example should document in its doc block WHY heterogeneous mutation rates help (exploration vs exploitation trade-off)
- The connection between Phase 10's single-pop Rastrigin (5D) and Phase 11's island-model Rastrigin (20D) makes a natural narrative: "single population struggles at 20D; island model handles it"
- Job scheduling output should clearly show the job sequence: `Best ordering: [3, 7, 1, 14, 0, ...]` and `Makespan: 42`

</specifics>

<deferred>
## Deferred Ideas

- None — discussion stayed within phase scope

</deferred>

---

*Phase: 11-advanced-mode-examples*
*Context gathered: 2026-03-22*
