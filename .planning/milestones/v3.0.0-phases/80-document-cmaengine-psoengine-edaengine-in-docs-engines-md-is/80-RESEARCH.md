# Phase 80: Document CmaEngine, PsoEngine, EdaEngine in docs/engines.md — Research

**Researched:** 2026-06-22
**Domain:** Markdown documentation authoring for an existing Rust crate
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** CMA-ES, PSO, and EDA each get a dedicated docs page (`docs/cma.md`, `docs/pso.md`, `docs/eda.md`) following the `nsga3.md`/`moead.md` pattern (~130–150 lines each).
- **D-02:** `docs/engines.md` gets a short stub section per engine (when-to-use bullet list + key params + link to dedicated page), NOT a full inline section. Same pattern as the NSGA-III inline section that links to `nsga3.md`.
- **D-03:** `docs/index.md` gets entries linking the three new pages under the "Engines" section.
- **D-04:** The engine overview table at the top of `docs/engines.md` is updated to add rows for `CmaEngine`, `PsoEngine`, and `EdaEngine`/`EdaRealEngine`.
- **D-05 (snippets):** Code snippets in dedicated pages cover key differentiating config only:
  - CMA: show `sigma0` heuristic (1/3 of search range), `RestartStrategy::Ipop` for multimodal, and `lambda = 0` (auto-compute)
  - PSO: show `PsoInertia::LinearDecay { w_start: 0.9, w_end: 0.4 }` + `PsoTopology::Ring { neighborhood_size: 2 }` as the typical recommended config
  - EDA: show the Bernoulli model path (`EdaEngine` for binary) vs Gaussian model path (`EdaRealEngine` for continuous) as two contrasting snippets

### Claude's Discretion

- Parameter table formatting: follow existing `nsga3.md` / `moead.md` Markdown table style
- "When PSO beats GA" / "When EDA beats crossover-based GAs" content: derive from config docs and engine logic in `src/engines/`
- Section ordering within each page: Description → When to Use → Configuration (param table) → Key Snippets → See Also (links to example + engines.md)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

## Summary

Phase 80 is a pure documentation phase — no Rust source code changes. Three engines already fully implemented (`CmaEngine`, `PsoEngine`, `EdaEngine`/`EdaRealEngine`) have zero coverage in `docs/engines.md` or any dedicated page. The task is to create three new Markdown pages (`docs/cma.md`, `docs/pso.md`, `docs/eda.md`) and update two existing files (`docs/engines.md`, `docs/index.md`).

All source-of-truth material exists in the codebase right now: the configuration structs have thorough doc-comments covering every parameter, the examples are runnable and tested, and the existing dedicated pages (`nsga3.md`, `moead.md`, `spea2.md`) provide exact structural templates to follow. The research phase has confirmed every field name, type, default, and constraint directly from source.

One discrepancy from CONTEXT.md was found and is documented below: `PsoInertia` has only two variants (`Constant` and `LinearDecay`), not three — there is no `RandomRange` variant. Similarly `PsoTopology` has only `Global` and `Ring`, not `VonNeumann`. The planner must use only the variants that exist in source.

**Primary recommendation:** Treat this as a transcription task — copy structure from `nsga3.md`/`moead.md`, populate with verified fields from configuration source files, then run `cargo doc --no-deps` to confirm zero rustdoc warnings.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| New docs pages (cma.md, pso.md, eda.md) | Filesystem / docs/ | — | Pure Markdown files, no code tier |
| engines.md stub sections + overview table | Filesystem / docs/ | — | Edit to existing Markdown file |
| index.md engine links | Filesystem / docs/ | — | Edit to existing Markdown file |
| Snippet correctness | Rust crate API | — | Snippets reference real public types; must compile |
| Zero rustdoc warnings | Rust compiler | — | `cargo doc --no-deps` gate |

---

## Standard Stack

### Core

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| Markdown | — | Documentation format | Project-wide convention; all existing docs use Markdown |
| `cargo doc --no-deps` | current toolchain | rustdoc warning check | Required by success criteria; currently passes (0 warnings) |

No external packages are installed by this phase.

---

## Package Legitimacy Audit

No external packages are introduced by this phase. Section not applicable.

---

## Architecture Patterns

### Recommended Project Structure

New files to create:
```
docs/
├── cma.md     ← NEW (follows nsga3.md/moead.md pattern)
├── pso.md     ← NEW
└── eda.md     ← NEW
```

Files to edit:
```
docs/
├── engines.md  ← add 3 overview table rows + 3 stub sections
└── index.md    ← add 3 engine links under "Engines" section
```

### Pattern 1: Dedicated Page Structure (from nsga3.md / moead.md)

**What:** Each dedicated page follows this section order, matching the existing template exactly.
**When to use:** All three new pages must follow this pattern verbatim.

```markdown
# EngineName

> One-line tagline.

## Description

[Algorithm narrative — how it works, what makes it different]

## When to Use

- **Problem type:** [continuous / binary / ...]
- **Variable type:** [real-valued / binary / ...]
- **Key strength:** [what it does better than alternatives]
- **Key weakness:** [where it struggles]

## Quick Reference

### [Parameter group]

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
...

## Complete Example

```rust,ignore
[full working snippet]
```

## Configuration Tips

[Bullet list of practical advice]

## When to Choose This vs [Alternative]

| Factor | ThisEngine | Alternative |
...

## References

- [Citation]

## See Also

- [engines.md link]
- [example file link]
- [docs.rs link]
```

**Source:** `docs/nsga3.md`, `docs/moead.md`, `docs/spea2.md` — [VERIFIED: codebase read]`

### Pattern 2: engines.md Stub Section (from existing NSGA-III section)

**What:** A short inline section in `docs/engines.md` that summarises the engine and links to the dedicated page. Approximately 20–25 lines.
**When to use:** Add this for CMA, PSO, and EDA after the existing engine sections.

Structure observed in the existing `## Nsga3Ga<U> — NSGA-III` section (lines 372–428 of engines.md):
1. One-line description paragraph
2. `**Entry point:**` line
3. `### When to Use` bullet list (4 bullets)
4. `### Configuration` code snippet (key config only, not full boilerplate)
5. `### Key Parameters` table
6. `### See Also` with link to full dedicated page + example

**Source:** `docs/engines.md` lines 372–428 — [VERIFIED: codebase read]

### Pattern 3: index.md Engine Link Format

**What:** Each engine gets a bullet under `### Engines (12 total)` in index.md. Currently 14 entries (including dedicated pages and engines.md anchors). The new entries follow the same format.

Example existing entry:
```markdown
- [NSGA-III](nsga3.md) — Reference-point based many-objective optimization (3+ objectives)
```

New entries should be:
```markdown
- [CMA-ES](cma.md) — Covariance Matrix Adaptation Evolution Strategy for continuous optimization
- [PSO](pso.md) — Particle Swarm Optimization with configurable inertia and topology
- [EDA](eda.md) — Univariate Marginal Distribution Algorithm (Bernoulli and Gaussian models)
```

**Source:** `docs/index.md` lines 19–34 — [VERIFIED: codebase read]

### Anti-Patterns to Avoid

- **Inventing API variants that don't exist:** `PsoInertia::RandomRange` and `PsoTopology::VonNeumann` are referenced in CONTEXT.md's "Specifics" but do NOT exist in source. Only `Constant` and `LinearDecay` for inertia; only `Global` and `Ring` for topology. [VERIFIED: codebase grep]
- **Using `use genetic_algorithms::*` in snippets:** Existing docs use precise module paths only (observed in all existing dedicated pages). [VERIFIED: codebase read]
- **Inline sections instead of stubs:** D-02 locks that `engines.md` gets short stubs with links, not full inline documentation.
- **Missing `## See Also` section:** Every existing dedicated page ends with a `## See Also` block. Omitting it breaks the navigation convention.

---

## Engine-Specific Findings (Source of Truth)

### CMA-ES (`CmaEngine`)

**Source files read:** `src/engines/cma/configuration.rs`, `src/engines/cma/restart.rs`, `examples/cma_es_rastrigin.rs` — [VERIFIED: codebase read]

**Module path:** `genetic_algorithms::cma`
**Public exports:** `CmaEngine`, `CmaConfiguration`, `CmaResult`, `RestartStrategy`, `RestartEvent`, `RestartKind`

**Configuration fields (all verified from source):**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `sigma0` | `f64` | `0.3` | Initial step size. Heuristic: 1/5 to 1/3 of search range. |
| `population_size` | `usize` | `0` (auto) | λ. If 0, auto-computes `4 + floor(3·ln(n))` at `run()`. |
| `max_generations` | `usize` | `1000` | Stop after this many generations. |
| `problem_solving` | `ProblemSolving` | `Minimization` | Minimize or maximize. |
| `fitness_target` | `Option<f64>` | `None` | Early-stop when reached. |
| `cc` | `Option<f64>` | `None` | Covariance cumulation rate. None = Hansen auto-formula. |
| `cs` | `Option<f64>` | `None` | Step-size cumulation rate. None = Hansen auto-formula. |
| `c1` | `Option<f64>` | `None` | Rank-one update rate. None = Hansen auto-formula. |
| `cmu` | `Option<f64>` | `None` | Rank-mu update rate. None = auto. Constraint: `c1 + cmu ≤ 1`. |
| `restart_strategy` | `Option<RestartStrategy>` | `None` | IPOP or BIPOP; None = no restarts. |
| `fitness_cache_size` | `Option<usize>` | `None` | LRU cache size for fitness evaluations. |

**RestartStrategy variants (verified from source):**

- `Ipop { population_scale: f64, stagnation_threshold: usize, max_restarts: usize }` — doubles population on stagnation. Typical: `population_scale=2.0`, `stagnation_threshold=50`, `max_restarts=9`.
- `Bipop { population_scale: f64, small_population_size: usize, stagnation_threshold: usize, max_restarts: usize }` — alternates large (IPOP-style) and small restarts.

**Helper constructor:** `CmaConfiguration::default_for_dim(n: usize)` — sets `population_size = 4 + floor(3·ln(n))`.

**Key snippet from example (cma_es_rastrigin.rs):**
```rust,ignore
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine};
use genetic_algorithms::configuration::ProblemSolving;

let config = CmaConfiguration::default_for_dim(DIMENSIONS)
    .with_sigma0(0.5)
    .with_max_generations(300)
    .with_fitness_target(1e-3)
    .with_problem_solving(ProblemSolving::Minimization);

let mut engine = CmaEngine::new(config, init_population, rastrigin)
    .with_observer(Arc::new(LogObserver));
let result = engine.run().expect("engine run should succeed");
```

**Sigma0 heuristic to document:** "Set `sigma0 ≈ 1/3 of the expected search range`. For a search domain of `[-5.12, 5.12]` (range 10.24), use `sigma0 ≈ 3.4`."

**When CMA beats standard GA:**
- Continuous real-valued problems (sphere, Rosenbrock, Rastrigin)
- Problems where the fitness landscape has correlations between dimensions (non-separable)
- Black-box / expensive fitness functions where evaluation budget matters
- Dimensions up to ~40 (CMA covariance matrix is O(n²))

---

### PSO (`PsoEngine`)

**Source files read:** `src/engines/pso/configuration.rs`, `examples/pso_rastrigin.rs` — [VERIFIED: codebase read]

**Module path:** `genetic_algorithms::pso`
**Public exports (from engine pattern):** `PsoEngine`, `PsoConfiguration`, `PsoInertia`, `PsoTopology`

**IMPORTANT CORRECTION:** CONTEXT.md mentions `PsoInertia::RandomRange` as a third inertia variant and `PsoTopology::VonNeumann` as a third topology. **These do not exist in the source.** Only two inertia variants and two topology variants are implemented. [VERIFIED: grep confirmed no matches]

**PsoInertia variants (verified):**
- `Constant(f64)` — fixed weight every generation. Common: `0.729` (Clerc).
- `LinearDecay { w_start: f64, w_end: f64 }` — decreasing from `w_start` to `w_end`. Default: `0.9 → 0.4` (Shi & Eberhart 1998).

**PsoTopology variants (verified):**
- `Global` — gbest: all particles share the single global best. Fast convergence, premature convergence risk on multimodal.
- `Ring { neighborhood_size: usize }` — lbest: each particle attracted to best within k nearest neighbors. Slower convergence, better exploration.

**Configuration fields (all verified from source):**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `population_size` | `usize` | `30` | Number of particles. |
| `max_generations` | `usize` | `1000` | Stop after this many generations. |
| `problem_solving` | `ProblemSolving` | `Minimization` | Minimize or maximize. |
| `fitness_target` | `Option<f64>` | `None` | Early-stop when reached. |
| `inertia` | `PsoInertia` | `LinearDecay { 0.9, 0.4 }` | Velocity retention strategy. |
| `c1` | `f64` | `2.0` | Cognitive coefficient (personal-best pull). |
| `c2` | `f64` | `2.0` | Social coefficient (neighborhood-best pull). |
| `topology` | `PsoTopology` | `Global` | Neighborhood topology. |
| `fitness_cache_size` | `Option<usize>` | `None` | LRU cache size. |

**Key snippet from example (pso_rastrigin.rs):**
```rust,ignore
use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoTopology};
use genetic_algorithms::configuration::ProblemSolving;

let config = PsoConfiguration {
    population_size: 200,
    max_generations: 1000,
    problem_solving: ProblemSolving::Minimization,
    fitness_target: Some(1e-3),
    inertia: PsoInertia::LinearDecay { w_start: 0.9, w_end: 0.4 },
    c1: 2.0,
    c2: 2.0,
    topology: PsoTopology::Global,
    fitness_cache_size: None,
};
```

**D-05 prescribed snippet for the dedicated page:**
```rust,ignore
// Typical recommended config: LinearDecay + Ring topology for multimodal problems
let config = PsoConfiguration::default()
    .with_population_size(50)
    .with_max_generations(1000)
    .with_inertia(PsoInertia::LinearDecay { w_start: 0.9, w_end: 0.4 })
    .with_topology(PsoTopology::Ring { neighborhood_size: 2 })
    .with_c1(1.49445)
    .with_c2(1.49445);
```

**When PSO beats GA:**
- Continuous real-valued problems where gradient information is unavailable
- Moderate dimensionality (works well up to ~100 dims without covariance overhead)
- When fewer hyperparameters vs CMA is desirable
- Problems with smooth, unimodal landscapes (Global topology converges fast)

---

### EDA (`EdaEngine` / `EdaRealEngine`)

**Source files read:** `src/engines/eda/configuration.rs`, `src/engines/eda/engine.rs`, `src/engines/eda/mod.rs`, `examples/eda_trap.rs` — [VERIFIED: codebase read]

**Module path:** `genetic_algorithms::eda`
**Public exports:** `EdaEngine`, `EdaRealEngine`, `EdaConfiguration`, `EdaModel`, `EdaResult`

**Key architecture:** Single `EdaConfiguration` struct used by BOTH `EdaEngine` (Bernoulli/binary) and `EdaRealEngine` (Gaussian/continuous). The engine type selects the model — not a config flag.

**EdaModel enum (public, returned in EdaResult):**
- `Bernoulli(Vec<f64>)` — probability per position
- `Gaussian { means: Vec<f64>, stds: Vec<f64> }` — mean + std per position

**Configuration fields (verified from source):**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `population_size` | `usize` | `100` | Population. EDA needs larger pops than GA to estimate model. |
| `max_generations` | `usize` | `500` | Stop after this many generations. |
| `problem_solving` | `ProblemSolving` | `Maximization` | Note: default is **Maximization** (unlike CMA/PSO which default to Minimization). |
| `fitness_target` | `Option<f64>` | `None` | Early-stop when reached. |
| `selection_ratio` | `f64` | `0.5` | Fraction of population used to estimate model. Range: `(0.0, 1.0]`. |
| `fitness_cache_size` | `Option<usize>` | `None` | LRU cache size. |

**D-05 prescribed snippet for the dedicated page:**
```rust,ignore
// Binary genes → Bernoulli UMDA model
use genetic_algorithms::eda::{EdaConfiguration, EdaEngine};
use genetic_algorithms::configuration::ProblemSolving;

let config = EdaConfiguration::default()
    .with_population_size(300)
    .with_max_generations(500)
    .with_selection_ratio(0.3)
    .with_problem_solving(ProblemSolving::Maximization);

let mut engine = EdaEngine::<BinaryChromosome>::new(
    config,
    |n| init_population(n),
    trap_fitness,
);

// Continuous genes → Gaussian univariate model
use genetic_algorithms::eda::{EdaConfiguration, EdaRealEngine};

let config = EdaConfiguration::default()
    .with_population_size(100)
    .with_max_generations(300)
    .with_selection_ratio(0.5)
    .with_problem_solving(ProblemSolving::Minimization);

let mut engine = EdaRealEngine::<RangeChromosome<f64>>::new(
    config,
    |n| init_population(n),
    sphere_fitness,
);
```

**When EDA beats crossover-based GAs:**
- Binary problems with epistasis/deception (trap functions) where crossover disrupts block structure
- Problems where variable dependencies are captured by marginal distributions (linkage-learning advantage)
- Feature selection / combinatorial problems on binary chromosomes
- Note: UMDA (univariate) does not model inter-variable dependencies — for that, use multivariate EDAs (BOA, MIMIC) if available

---

## engines.md: Overview Table Additions

Current table has 13 rows (12 engines + note). Add three rows between `GpGa<N>` and `Nsga2Ga<U>`:

```markdown
| `CmaEngine<U>` | `cma` | Best f64 vector | Continuous optimization — self-adaptive covariance matrix |
| `PsoEngine<U>` | `pso` | Best vector | Swarm-based continuous optimization — few hyperparameters |
| `EdaEngine<U>` / `EdaRealEngine<U>` | `eda` | Best individual | Probabilistic model-building — binary (Bernoulli) or continuous (Gaussian) |
```

**Placement rationale:** These are single-objective continuous/combinatorial engines, fitting between GP (also single-obj) and the multi-objective engines.

**Header update:** Change "twelve engines" in the intro paragraph to "fifteen engines" (or similar accurate count).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Parameter tables | Custom format | `| Field | Type | Default | Description |` | Existing convention in engines.md DE/Scatter sections |
| Snippet imports | Glob imports `use genetic_algorithms::*` | Precise module paths | Observed pattern in all existing dedicated pages |
| CMA algorithm explanation | Original prose | Paraphrase from config doc-comments | Doc-comments are already precise; paraphrase to avoid rustdoc conflicts |

---

## Common Pitfalls

### Pitfall 1: Non-Existent PSO Variants
**What goes wrong:** Documentation mentions `PsoInertia::RandomRange` or `PsoTopology::VonNeumann` — neither exists in source.
**Why it happens:** CONTEXT.md Specifics listed them as variants to document, but they were not implemented.
**How to avoid:** Only document variants confirmed in `src/engines/pso/configuration.rs` — `Constant`, `LinearDecay`, `Global`, `Ring`.
**Warning signs:** Any snippet using `RandomRange` or `VonNeumann` will fail to compile.

### Pitfall 2: EDA Default is Maximization
**What goes wrong:** Snippet assumes `ProblemSolving::Minimization` is the default for EdaConfiguration — it is NOT.
**Why it happens:** CMA and PSO both default to Minimization; EDA defaults to Maximization (binary fitness problems are typically maximized).
**How to avoid:** Always explicitly call `.with_problem_solving(...)` in EDA snippets, or note the default clearly in the parameter table.

### Pitfall 3: Incorrect sigma0 Heuristic
**What goes wrong:** Documenting "sigma0 = 1/3 of search range" as an exact formula when it is a guideline.
**Why it happens:** The doc-comment says "1/5 to 1/3 of search range" — using only 1/3 is accurate but omits the range.
**How to avoid:** Document as "typically 1/5 to 1/3 of the search range per dimension."

### Pitfall 4: Rustdoc Warning from Broken Links
**What goes wrong:** A `See Also` link or `[EngineName](path.md)` points to a page that does not exist yet.
**Why it happens:** Pages are created in a different wave than `engines.md` / `index.md` edits.
**How to avoid:** Create all three dedicated pages before editing `engines.md` and `index.md`, or create all files in a single wave.

### Pitfall 5: EdaRealEngine vs EdaEngine Confusion
**What goes wrong:** Using `EdaEngine` for real-valued chromosomes or `EdaRealEngine` for binary chromosomes.
**Why it happens:** Both use the same `EdaConfiguration` — the model selection is determined by the engine type, not a config field.
**How to avoid:** Make the distinction the opening sentence of the EDA page description.

---

## Code Examples

### CMA-ES: Sigma0 Heuristic + IPOP Restart
```rust,ignore
// Source: src/engines/cma/configuration.rs, src/engines/cma/restart.rs
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine, RestartStrategy};
use genetic_algorithms::configuration::ProblemSolving;

// Search domain [-5.12, 5.12] → range = 10.24 → sigma0 ≈ 10.24 / 3 ≈ 3.4
// For multimodal landscapes, add IPOP restarts
let config = CmaConfiguration::default_for_dim(10)
    .with_sigma0(3.4)
    .with_max_generations(1000)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_restart_strategy(RestartStrategy::Ipop {
        population_scale: 2.0,
        stagnation_threshold: 50,
        max_restarts: 9,
    });
```

### PSO: LinearDecay + Ring Topology
```rust,ignore
// Source: src/engines/pso/configuration.rs
use genetic_algorithms::pso::{PsoConfiguration, PsoInertia, PsoTopology};

let config = PsoConfiguration::default()
    .with_inertia(PsoInertia::LinearDecay { w_start: 0.9, w_end: 0.4 })
    .with_topology(PsoTopology::Ring { neighborhood_size: 2 })
    .with_c1(1.49445)
    .with_c2(1.49445);
```

### EDA: Bernoulli (binary) vs Gaussian (continuous)
```rust,ignore
// Source: src/engines/eda/engine.rs, src/engines/eda/mod.rs
use genetic_algorithms::eda::{EdaConfiguration, EdaEngine, EdaRealEngine};

// Binary chromosomes → Bernoulli UMDA
let config = EdaConfiguration::default()
    .with_population_size(300)
    .with_selection_ratio(0.3);
let mut bernoulli = EdaEngine::<BinaryChromosome>::new(config, init_fn, fitness_fn);

// Real-valued chromosomes → Gaussian univariate model
let config2 = EdaConfiguration::default()
    .with_population_size(100)
    .with_selection_ratio(0.5)
    .with_problem_solving(ProblemSolving::Minimization);
let mut gaussian = EdaRealEngine::<RangeChromosome<f64>>::new(config2, init_fn2, fitness_fn2);
```

---

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|------------------|-------|
| No docs for CMA/PSO/EDA | Dedicated page per engine | This phase adds the pages |
| engines.md lists 12 engines in table | Table will list 15 engines | 3 new rows |
| index.md "Engines (12 total)" | Will become "15 total" | Heading update needed |

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo doc --no-deps` | Zero-warning gate | ✓ | current toolchain | — |
| `cargo test` | Snippet validation | ✓ | current toolchain | — |

Current state: `cargo doc --no-deps` produces 0 warnings. Phase must maintain this.

---

## Validation Architecture

**nyquist_validation:** Not explicitly set in `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust compiler + cargo doc |
| Config file | Cargo.toml |
| Quick run command | `cargo doc --no-deps 2>&1 \| grep -c warning` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy && cargo doc --no-deps` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DOC-01 | docs/cma.md exists and is complete | manual | `ls docs/cma.md` | No — Wave 0 |
| DOC-02 | docs/pso.md exists and is complete | manual | `ls docs/pso.md` | No — Wave 0 |
| DOC-03 | docs/eda.md exists and is complete | manual | `ls docs/eda.md` | No — Wave 0 |
| DOC-04 | engines.md overview table has CMA/PSO/EDA rows | manual | `grep -c "CmaEngine\|PsoEngine\|EdaEngine" docs/engines.md` | ✅ (partial — grep target) |
| DOC-05 | index.md links three new pages | manual | `grep -c "cma.md\|pso.md\|eda.md" docs/index.md` | ✅ (grep target) |
| DOC-06 | Zero rustdoc warnings | automated | `cargo doc --no-deps 2>&1 \| grep warning \| wc -l` (must be 0) | ✅ |

### Sampling Rate

- **Per task commit:** `cargo doc --no-deps 2>&1 | grep warning | wc -l` (must output 0)
- **Per wave merge:** `cargo test && cargo doc --no-deps`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `docs/cma.md` — must be created (DOC-01)
- [ ] `docs/pso.md` — must be created (DOC-02)
- [ ] `docs/eda.md` — must be created (DOC-03)

---

## Security Domain

This phase modifies only Markdown documentation files. No authentication, session management, access control, cryptography, or input validation is involved. Security domain is not applicable.

---

## Open Questions

1. **index.md engine count heading**
   - What we know: Current text says "Engines (12 total)" and lists 14 entries (including multi-objective dedicated pages).
   - What's unclear: Whether the heading count should be updated (it's already inaccurate at 12 vs 14).
   - Recommendation: Update heading to "Engines (15 total)" when adding the 3 new entries, or remove the count entirely to avoid future staleness.

2. **engines.md intro count**
   - What we know: First paragraph says "twelve engines."
   - What's unclear: Whether to update to "fifteen engines."
   - Recommendation: Update to match the table row count (15), or phrase as "fifteen engine types."

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `PsoInertia::RandomRange` and `PsoTopology::VonNeumann` do not exist | Engine-Specific Findings (PSO) | If they were added after last read, snippets would compile but docs would miss them |
| A2 | The doc/index.md heading "12 total" should be updated to "15 total" | Open Questions | Minor: cosmetic inconsistency if left as-is |

**Note:** A1 was verified by grep over the PSO configuration source — zero matches for `RandomRange` or `VonNeumann`. This is HIGH confidence. Tagging A1 as `[ASSUMED]` only for the case that source changed after this research ran.

---

## Sources

### Primary (HIGH confidence)
- `src/engines/cma/configuration.rs` — all CmaConfiguration fields, builder methods, defaults
- `src/engines/cma/restart.rs` — RestartStrategy variants (Ipop, Bipop) with all fields
- `src/engines/pso/configuration.rs` — all PsoConfiguration fields, PsoInertia variants, PsoTopology variants
- `src/engines/eda/configuration.rs` — all EdaConfiguration fields, builder methods, defaults
- `src/engines/eda/engine.rs` — EdaEngine vs EdaRealEngine distinction, EdaModel enum
- `src/engines/eda/mod.rs` — public export surface
- `docs/nsga3.md` — structural template (Description/When to Use/Config/Example/See Also)
- `docs/moead.md` — second structural template
- `docs/spea2.md` — third structural template
- `docs/engines.md` — NSGA-III stub section pattern; overview table structure
- `docs/index.md` — engine link format in navigation section
- `examples/cma_es_rastrigin.rs` — runnable CMA-ES example (link target)
- `examples/pso_rastrigin.rs` — runnable PSO example (link target)
- `examples/eda_trap.rs` — runnable EDA example (link target)

### Secondary (MEDIUM confidence)
- `README.md` engine decision matrix — confirms CMA/PSO/EDA are published and documented at README level

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Documentation structure: HIGH — directly read from existing template files
- API field names / types / defaults: HIGH — read from Rust source with doc-comments
- PSO variant correction: HIGH — confirmed by grep returning 0 matches
- Example snippets: HIGH — read from working example files

**Research date:** 2026-06-22
**Valid until:** 2026-09-22 (stable library; no known plans to change CMA/PSO/EDA APIs)
