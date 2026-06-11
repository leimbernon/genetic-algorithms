# Phase 50: Lexicase Selection - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-22
**Phase:** 50-lexicase-selection
**Areas discussed:** Case fitness eval flow, Lexicase factory dispatch, Epsilon default for ε-lexicase

---

## Case Fitness Eval Flow

| Option | Description | Selected |
|--------|-------------|----------|
| In calculate_fitness() | User calls set_case_fitness(vec![...]) inside calculate_fitness(). Single callback, no new fields. | ✓ |
| Separate case_fitness_fn closure | Second stored closure alongside scalar fitness_fn. More explicit but adds a new field per implementing chromosome. | |
| You decide | Pick whichever fits existing patterns best. | |

**User's choice:** In calculate_fitness()
**Notes:** Consistent with how all existing chromosomes handle fitness state.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Selection syncs it (SEL-02 spec) | lexicase selection calls chromosome.set_fitness(mean_case_score) before returning pairs. | ✓ |
| User sets it in calculate_fitness() | User computes mean manually. Selection never touches fitness(). | |
| You decide | Pick whatever is least surprising for users. | |

**User's choice:** Selection syncs it
**Notes:** Matches SEL-02 wording directly.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Read case_fitness() from chromosome | lexicase_selection reads chromosomes[0].case_fitness().len() to determine num cases. | ✓ |
| Caller passes num_cases explicitly | factory_lexicase() takes a num_cases: usize param. | |
| You decide | Pick whichever is simpler to implement. | |

**User's choice:** Read case_fitness() from chromosome
**Notes:** Info is already on the chromosome — no redundant parameter needed.

---

## Lexicase Factory Dispatch

| Option | Description | Selected |
|--------|-------------|----------|
| Separate factory_lexicase<U: MultiCaseFitness>() | ga.rs detects Lexicase/EpsilonLexicase in config and routes to separate factory. Standard factory() errors for these variants. | ✓ |
| Inline branch in ga.rs via specialization | ga.rs has explicit if/else, Ga<U> requires MultiCaseFitness bound when lexicase configured. Hard in stable Rust. | |
| You decide | Pick whatever integrates cleanest with existing dispatch. | |

**User's choice:** Separate factory_lexicase<U: MultiCaseFitness>()
**Notes:** Cleanest type-system approach without specialization.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Per-generation in run() | if/else branch in run(): Lexicase → factory_lexicase(), else → factory(). Minimal disruption. | ✓ |
| Validated at build() time | build() returns GaError if Lexicase selected without MultiCaseFitness. Requires runtime type check. | |
| You decide | Pick whichever avoids panics and keeps ga.rs readable. | |

**User's choice:** Per-generation in run()
**Notes:** Same location as all other selection dispatch — consistent.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Panic with a clear message | "Use factory_lexicase for Lexicase selection; SelectionOperator trait path does not support MultiCaseFitness" | ✓ |
| Return empty Vec (silent fail) | Returns vec![] silently. | |
| You decide | Pick whatever is safest for library users discovering this by mistake. | |

**User's choice:** Panic with a clear message
**Notes:** Guards island-model and NSGA-II paths (which go through SelectionOperator trait) from silently misbehaving.

---

## Epsilon Default for ε-lexicase

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed 0.01 | Simple, predictable, documented. | |
| Dynamic MAD | Median absolute deviation per case per generation. Standard academic default. O(n × cases) overhead. | ✓ |
| You decide | Pick whichever aligns better with library goals. | |

**User's choice:** Dynamic MAD
**Notes:** Helmuth et al. 2016 is the reference. Can still be overridden with fixed value.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Single scalar for all cases | epsilon: f64 in SelectionConfiguration. Same epsilon for every test case. SelectionConfiguration stays Copy. | ✓ |
| Per-case epsilon vector | epsilon: Vec<f64>. Breaks Copy derive on SelectionConfiguration. | |
| You decide | Pick whatever keeps SelectionConfiguration Copy-able. | |

**User's choice:** Single scalar for all cases
**Notes:** Consistent with boltzmann_temperature and niche_radius pattern.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Pre-computed once at selection start | Compute MAD for all cases from full population before shuffling. O(n × cases) once. Stable during filter cascade. | ✓ |
| Recomputed after each case filters pool | Dynamically adaptive but O(n × cases²) worst case. | |
| You decide | Pick whatever is simpler and consistent with literature. | |

**User's choice:** Pre-computed once at selection start
**Notes:** Epsilon is stable during the filtering cascade — avoids O(n²) overhead.

---

## Claude's Discretion

None — all gray areas had clear user choices.

## Deferred Ideas

None — discussion stayed within phase scope.
