# Phase 76: Parallelize Survivor Selection and Non-Dominated Sorting - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-19
**Phase:** 76-parallelize-survivor-selection-and-non-dominated-sorting
**Areas discussed:** Non-dominated sorting parallel strategy, Engine-specific parallelization scope, Deterministic crowding + parsimony, Population size threshold

---

## Non-dominated sorting parallel strategy

### Deduplication approach

| Option | Description | Selected |
|--------|-------------|----------|
| Parallelize shared copy only | Keep both copies identical but add rayon to multi_objective/; nsga2/ re-exports from there | |
| Deduplicate first, then parallelize | Delete nsga2/non_dominated_sort.rs, re-export from multi_objective/ | ✓ |
| Agent decides | You decide the best approach | |

**User's choice:** Deduplicate first, then parallelize
**Notes:** Cleaner long-term; eliminates the maintenance burden of keeping two copies in sync.

### Parallelization strategy for O(N^2) loop

| Option | Description | Selected |
|--------|-------------|----------|
| Parallelize outer i-loop | Each i processes in parallel via par_iter; inner j-loop stays sequential per i | ✓ |
| Parallelize inner j-loop | Inner j-loop parallelized per i; more fine-grained but higher overhead | |
| Both loops | Both loops parallelized; maximum parallelism but complex | |
| Agent decides | You decide | |

**User's choice:** Parallelize outer i-loop
**Notes:** Simpler, good cache locality per i.

### assign_ranks() parallelization

| Option | Description | Selected |
|--------|-------------|----------|
| Skip | assign_ranks() is O(N) after the O(N^2) sort; leave sequential | ✓ |
| Parallelize assign_ranks | Parallelize the front-processing loop for completeness | |

**User's choice:** Skip
**Notes:** The bottleneck is the sort, not rank assignment.

---

## Engine-specific parallelization scope

### Which engines get parallel non-dominated sorting

| Option | Description | Selected |
|--------|-------------|----------|
| All 6 engines | NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA all benefit | ✓ |
| Skip SMS-EMOA and MOEA/D | Only NSGA-II, NSGA-III, SPEA2, IBEA — skip steady-state/sequential engines | |
| Agent decides | You decide per-engine | |

**User's choice:** All 6 engines
**Notes:** Since the sorting function is shared, all engines benefit automatically after parallelization.

### Crowding distance parallelization

| Option | Description | Selected |
|--------|-------------|----------|
| Skip | Crowding distance is O(N log N) per front; less critical than O(N^2) sort | ✓ |
| Parallelize too | Parallelize crowding distance computation for completeness | |

**User's choice:** Skip
**Notes:** The O(N^2) non-dominated sort is the primary bottleneck.

---

## Deterministic crowding + parsimony

### deterministic_crowding parallelization

| Option | Description | Selected |
|--------|-------------|----------|
| Skip | Pairwise parent-offspring comparison (O(N)); low benefit from parallelization | ✓ |
| Parallelize | Parallelize the pairwise comparison loop | |
| Agent decides | You decide | |

**User's choice:** Skip
**Notes:** O(N) is not the bottleneck.

### parsimony pressure parallelization

| Option | Description | Selected |
|--------|-------------|----------|
| Skip | Delegates to factory(); parsimony pressure loop is O(N) | ✓ |
| Parallelize | Parallelize the parsimony pressure application | |
| Agent decides | You decide | |

**User's choice:** Skip
**Notes:** O(N) wrapper, low benefit.

---

## Population size threshold

| Option | Description | Selected |
|--------|-------------|----------|
| >=100 (ROADMAP default) | Conservative, matches typical small-population use cases | ✓ |
| >=200 | Only parallelize at >=200; more aggressive | |
| Always parallel | No threshold; simplest code but may hurt small populations | |
| Agent decides | You decide based on profiling | |

**User's choice:** >=100 (ROADMAP default)
**Notes:** Matches the ROADMAP success criterion.

---

## the agent's Discretion

- Whether to add a brief comment explaining the parallelization strategy at the par_iter() call site
- Exact par_iter() vs into_par_iter() choice (depends on whether ownership is needed)
- Whether to benchmark before/after as part of this phase or rely on Phase 74 benchmarks

## Deferred Ideas

None — discussion stayed within phase scope.
