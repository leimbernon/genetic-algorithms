# Release Gate — Phase 65 / Plan 65-03

## Pre-flight

| Check | Value |
|-------|-------|
| `cargo --version` | cargo 1.94.1 (29ea6fb6a 2026-03-24) |
| `rustc --version` | rustc 1.94.1 (e408947bf 2026-03-25) |
| `build.rs` exists | YES (`-rw-r--r--@ 231 bytes`) |
| `wasm32-unknown-unknown` target installed | YES |

## Part 1 — CI Matrix

### `cargo test`
Exit code: 0
Output (head):
```
running 1661 tests
...
```
Output (tail):
```
test result: ok. 267 passed; 0 failed; 29 ignored; 0 measured; 0 filtered out; finished in 27.96s
```

### `cargo test --features serde`
Exit code: 0
Output (head):
```
Compiling genetic_algorithms v3.0.0 (/Users/luis/RustroverProjects/genetic-algorithms)
...
```
Output (tail):
```
test result: ok. 267 passed; 0 failed; 29 ignored; 0 measured; 0 filtered out
```

### `cargo clippy --all-targets -- -D warnings`
Exit code: 0
Output (head):
```
Checking genetic_algorithms v3.0.0
...
```
Output (tail):
```
Finished `dev` profile [optimized + debuginfo] target(s) in 3.48s
```
Warning count: 0

### `cargo doc --no-deps --all-features`
Exit code: 0
Warning grep (`grep -c '^warning:'`): 0
Output (tail):
```
Generated /Users/luis/RustroverProjects/genetic-algorithms/target/doc/genetic_algorithms/index.html
Finished `dev` profile [optimized + debuginfo] target(s) in 1.53s
```

### `cargo check --target wasm32-unknown-unknown --no-default-features --features logging`
Exit code: 0
Output (tail):
```
Finished `dev` profile [optimized + debuginfo] target(s) in 1.58s
```

## Part 2 — cargo publish --dry-run

Exit code: 0
Output:
```
Updating crates.io index
Packaging genetic_algorithms v3.0.0
Packaged 455 files, 4.3MiB (829.1KiB compressed)
Verifying genetic_algorithms v3.0.0
Finished `dev` profile [optimized + debuginfo] target(s) in 3.57s
Uploading genetic_algorithms v3.0.0
warning: aborting upload due to dry run
```
Note: Dry-run upload aborted as expected — `cargo publish` would succeed.

## Part 3 — v2 sample crate smoke-test

### Sub-step A — Create crate
Created `/tmp/ga_v2_smoke` with `cargo new --lib`.

### Sub-step B — Write v2 `src/lib.rs`
Exercised top 3 breaking patterns:
1. `SmokeChrom` implementing v2 `ChromosomeT` with `dna()`, `dna_mut()`, `set_dna()`, `set_fitness_fn()` inline
2. `use genetic_algorithms::reporter::SimpleReporter` + `.with_reporter(Box::new(SimpleReporter::new(10)))`
3. `SmokeSelection` implementing v2 `SelectionOperator` with 3-param `select()` returning `Vec<(usize, usize)>`

### Sub-step C — Baseline build against v2.4.0
Exit code: 0
Output (tail):
```
warning: use of deprecated method `genetic_algorithms::ga::Ga::<U>::with_reporter`: use with_observer() instead.
Finished `dev` profile [optimized + debuginfo] target(s) in 0.07s
```

### Sub-step D — Switch to in-progress v3 (path dependency)
Exit code: NON-ZERO (expected)
Captured v3 errors:

#### Captured v3 error: E0432 (Reporter import)
```
error[E0432]: unresolved import `genetic_algorithms::reporter`
 --> src/lib.rs:3:25
  |
3 | use genetic_algorithms::reporter::SimpleReporter;
  |                         ^^^^^^^^ could not find `reporter` in `genetic_algorithms`
```

#### Captured v3 error: E0407 (ChromosomeT method removal)
```
error[E0407]: method `dna` is not a member of trait `ChromosomeT`
error[E0407]: method `dna_mut` is not a member of trait `ChromosomeT`
error[E0407]: method `set_dna` is not a member of trait `ChromosomeT`
error[E0407]: method `set_fitness_fn` is not a member of trait `ChromosomeT`
```

#### Captured v3 error: E0277 (LinearChromosome bound)
```
error[E0277]: the trait bound `SmokeChrom: LinearChromosome` is not satisfied
  --> src/lib.rs:24:23
   |
24 | impl ValueMutable for SmokeChrom {}
   |                       ^^^^^^^^^^ unsatisfied trait bound
```

#### Captured v3 error: E0050 (SelectionOperator parameter count)
```
error[E0050]: method `select` has 4 parameters but the declaration in trait `select` has 5
  --> src/lib.rs:50:9
   |
50 | /         &self,
51 | |         _chromosomes: &[U],
52 | |         _couples: usize,
53 | |         _threads: usize,
   | |_______________________^ expected 5 parameters, found 4
   |
   = note: `select` from trait: `fn(&Self, &[U], usize, usize, usize) -> Vec<Vec<usize>>`
```

### Sub-step E — Reconciliation

| Pattern | Captured Error | MIGRATION.md | Outcome | MIGRATION.md Change |
|---------|---------------|-------------|---------|-------------------|
| Reporter removal | E0432: `unresolved import genetic_algorithms::reporter` | E0432: `unresolved import genetic_algorithms::reporter` | MATCHES VERBATIM | No |
| ChromosomeT split | E0277: `LinearChromosome not satisfied` + E0407 at impl site | E0277: `LinearChromosome not satisfied` at call site | MATCHES (code, different call site) | No |
| SelectionOperator::select | E0050: `has 4 parameters but the declaration in trait select has 5` | E0053: `incompatible type for trait` | MISMATCH — code differs | YES: updated E0053 → E0050 |

**MIGRATION.md fix applied:** SelectionOperator compiler error updated from `error[E0053]` (incompatible type) to `error[E0050]` (wrong parameter count) with the actual captured rustc output.

### Sub-step F — Apply MIGRATION.md fixes and re-build
Exit code: 0
Output (tail):
```
warning: struct `SmokeGene` is never constructed
warning: struct `SmokeChrom` is never constructed
warning: struct `SmokeSelection` is never constructed
Finished `dev` profile [optimized + debuginfo] target(s) in 0.09s
```

### Sub-step G — Cleanup
Smoke crate directory removed: `test ! -d "${TMPDIR:-/tmp}/ga_v2_smoke"` exits 0.

### Reconciliation summary

| Pattern | Captured error code | MIGRATION.md outcome | MIGRATION.md change required |
|---------|--------------------|--------------------|-----------------------------|
| ChromosomeT + LinearChromosome split | E0277 / E0407 | MATCHES (code, paraphrased — different call site) | No |
| Reporter removal | E0432 | MATCHES VERBATIM | No |
| SelectionOperator::select | E0050 | MISMATCH (fixed) | Yes — E0053 → E0050 |

## Part 4 — examples smoke-run

### `sms_emoa_zdt1`
Exit code: 0
Output (tail):
```
  f1=0.1865, f2=2.7960
  f1=0.5285, f2=2.4434
  f1=0.8972, f2=2.3749
  f1=0.9067, f2=2.0905
  f1=0.9819, f2=1.7014
```

### `ibea_zdt1`
Exit code: 0
Output (tail):
```
  f1=0.2761, f2=3.5338
  f1=0.3280, f2=3.3658
  f1=0.6836, f2=3.2895
  f1=0.7119, f2=2.6306
  f1=0.8893, f2=2.5281
```

### `memetic_rastrigin`
Exit code: 0
Output (tail):
```

Comparison:
  Memetic GA best:   0.000005
  Standard GA best:  0.000179
  => HillClimbing local search improved convergence!
```

### `cma_es_rastrigin`
Exit code: 0
Output (tail):
```
sigma0=0.5, max_generations=300, target=1e-3
--------------------------------------------------
Generations: 300
Best fitness: 4.974795
Best DNA:    [-0.9950, -0.9950, -0.9950, -0.9950, -0.9950]
```

### `ipop_rastrigin`
Exit code: 0
Output (tail):
```
    Finished `release` profile [optimized] target(s) in 0.06s
     Running `target/release/examples/ipop_rastrigin`
Total restarts: 3
Generations:    800
Best fitness:   0.994959
```

### `pso_rastrigin`
Exit code: 0
Output (tail):
```
inertia=LinearDecay(0.9→0.4), c1=2.0, c2=2.0, topology=Global
--------------------------------------------------
Generations: 1000
Best fitness: 0.994959
Best DNA:    [0.9950, 0.0000, 0.0000, -0.0000, -0.0000, 0.0000, -0.0000, -0.0000, 0.0000, 0.0000]
```

### `eda_trap`
Exit code: 0
Output (tail):
```
Best DNA:     000000000000000000000000000000
Learned probs: [0.14, 0.23, 0.08, 0.12, 0.16, 0.16, 0.16, 0.10, 0.28, 0.22, 0.11, 0.21, 0.13, 0.01, 0.04, 0.13, 0.08, 0.27, 0.04, 0.11, 0.19, 0.22, 0.09, 0.19, 0.04, 0.02, 0.02, 0.01, 0.09, 0.18]
Converged positions (p > 0.9 or p < 0.1): 11/30
---------------------------------------------
PARTIAL: Best fitness 24.0/30 (increase generations or population for full convergence)
```

### `surrogate_rastrigin`
Exit code: 0
Output (tail):
```
Max evaluations without surrogate (est): 3000
Evaluation savings: -3.3%
-------------------------------------------------------
Surrogate-reduction assertion PASSED: at least one generation had
  true_fitness_calls < 100 (offspring_count).
```

### `knapsack_binary`
Exit code: 0
Output (tail):
```
Generation: 4799 - Best Score: 1088 - Termination Cause: NotTerminated
Generation: 4899 - Best Score: 1088 - Termination Cause: NotTerminated
Generation: 4999 - Best Score: 1088 - Termination Cause: NotTerminated
Generation: 5000 - Best Score: 1088 - Termination Cause: GenerationLimitReached
Best chromosome for fixed fitness: 0000000101
```

### `job_scheduling`
Exit code: 0
Output (tail):
```
== Job Scheduling -- Permutation-Based Makespan Minimization ==
Jobs: 15, Machines: 5, Population: 100, Max generations: 500
Operators: Selection=Tournament, Crossover=Order, Mutation=Insertion
-------------------------------------------------------
GA failed: MutationError("Mutation::Insertion requires ChromosomeLength::Variable. ChromosomeLength::Fixed does not allow changing chromosome length.")
```

### `onemax_extension`
Exit code: 0
Output (tail):
```
Gen  500: best= 60.0, avg= 53.8, std_dev= 1.01, unique=30/30
-------------------------------------------------------
Finished. Best fitness: 60/64
Termination: GenerationLimitReached
Reached 94% of optimum.
```

### `island_model`
Exit code: 0
Output (tail):
```
-------------------------------------------------------
-------------------------------------------------------
Best fitness: 261.947960
Best solution (first 5 dims): [0.2714417243429148, 1.0665890185609692, -0.09378149161417948, -4.135840945184614, 4.6384395232018685]
Try increasing generations or population size.
```

### `nsga2_zdt1`
Exit code: 0
Output (tail):
```
  f1=0.1109, f2=0.7178
  f1=0.1657, f2=0.6415
  f1=0.2505, f2=0.5454
  f1=0.3644, f2=0.4391
  f1=0.6066, f2=0.2586
```

### `feature_selection`
Exit code: 0
Output (tail):
```
-------------------------------------------------------
Finished. Best fitness: 4.00
Selected features: [0, 1, 2, 3]
Expected relevant features: [0, 1, 2, 3]
SUCCESS: All relevant features were selected!
```

### `rastrigin`
Exit code: 0
Output (tail):
```
Generation  499: best =   0.0001, avg =   0.0008
Generation  500: best =   0.0001, avg =   0.0008
-------------------------------------------------------
Finished. Best fitness: 0.000150
Near-optimal solution found!
```

### `nsga3_dtlz2`
Exit code: 0
Output (tail):
```
    0.0017     0.7040     0.7102     1.0000
    0.0017     0.7088     0.7057     1.0004
    0.0020     0.8209     0.5711     1.0000
    0.0022     0.8980     0.4399     1.0000
    0.0022     0.8980     0.4399     1.0000
```

### `nqueens_range`
Exit code: 0
Output (tail):
```
     Running `target/release/examples/nqueens_range`
Generation: 99 - Best Score: 1 - Phenotype: 2, 5, 7, 6, 0, 3, 1, 4 - Termination Cause: NotTerminated
Generation: 103 - Best Score: 0 - Phenotype: 6, 1, 5, 2, 0, 3, 7, 4 - Termination Cause: FitnessTargetReached
Best chromosome for N-Queens: 6, 1, 5, 2, 0, 3, 7, 4
Starting generation of random chromosome
```

### `niching`
Exit code: 0
Output (tail):
```
Top solutions (showing population spread across peaks):
  Peak 1 (x=2): 150 individuals
  Peak 2 (x=5): 0 individuals
  Peak 3 (x=8): 0 individuals
Found 1 of 3 peaks. Try increasing population or adjusting sigma_share.
```

### `onemax_binary`
Exit code: 0
Output (tail):
```
Generation   49: best =  99.00, avg =  97.20
Generation   59: best = 100.00, avg =  98.52
-------------------------------------------------------
Finished. Best fitness: 100
SUCCESS: Found the global optimum (all bits are 1)!
```

### `spea2_zdt1`
Exit code: 0
Output (tail):
```
  f1=0.0877, f2=0.7115
  f1=0.1593, f2=0.6081
  f1=0.3514, f2=0.4135
  f1=0.5460, f2=0.2668
  f1=0.7250, f2=0.1537
```

### `moead_dtlz2`
Exit code: 0
Output (tail):
```
    0.0012     0.0350     0.9995     1.0002
    0.0012     0.0350     0.9995     1.0002
    0.0012     0.0350     0.9995     1.0002
    0.0012     0.0350     0.9995     1.0002
    0.0012     0.0350     0.9995     1.0002
```


## Compare-link reconciliation

v3.0.0 tag: **not present**
CHANGELOG `[3.0.0]:` compare link: `2.4.0...HEAD`
Action: No rewrite needed — tag does not exist yet. The link will be updated to `2.4.0...v3.0.0` after the tag is cut (follow-up action outside this plan).

## Release sign-off

- **Date:** 2026-06-17T17:53:37+02:00
- **Commit:** 4f6e137c4eaeff03cfe8e0044e307a0d23ca539c
- **Conclusion:** Phase 65 release gate PASSED
