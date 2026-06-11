# Phase 32: Crossover & Differential Mutation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-04
**Phase:** 32-Crossover & Differential Mutation
**Areas discussed:** Differential mutation: population access, F scale factor config, ERX degenerate cases, ERX chromosome constraints

---

## Differential Mutation: Population Access

| Option | Description | Selected |
|--------|-------------|----------|
| Engine branch | GA engine detects Mutation::Differential before the mutation loop and calls a population-aware function directly, bypassing factory. No trait change. | ✓ |
| New factory with population slice | Add factory_with_population() for all mutations; non-Differential operators ignore extra args. | |
| Restrict to GA config only | Only valid through the GA engine; error at runtime on other engines. | |

**User's choice:** Engine branch (Recommended)
**Notes:** None — accepted the recommended approach.

---

## Differential Mutation: Population Too Small

| Option | Description | Selected |
|--------|-------------|----------|
| Return GaError | Fail fast with clear error when population_size < 4. | ✓ |
| Fall back to Gaussian | Silently degrade to Gaussian when population too small. | |
| Clone target unchanged | Return individual unmodified; disables mutation for small populations. | |

**User's choice:** Return GaError (Recommended)

---

## Differential Mutation: Chromosome Type

| Option | Description | Selected |
|--------|-------------|----------|
| Range<T> only | Require ValueMutable + Range chromosome; error clearly on Binary/List. | ✓ |
| Any ChromosomeT | Attempt generic support via gene IDs — fragile, not recommended. | |

**User's choice:** Range<T> only (Recommended)

---

## F Scale Factor Config

| Option | Description | Selected |
|--------|-------------|----------|
| New differential_f field | Add differential_f: Option<f64> to MutationConfiguration; with_differential_f() builder. Default 0.5. | ✓ |
| Reuse sigma | Reuse MutationConfiguration.sigma for F — semantically confusing. | |
| New DifferentialConfiguration struct | Separate struct — overkill for one field. | |

**User's choice:** New differential_f field (Recommended)

---

## ERX Degenerate: Exhausted Neighbor List

| Option | Description | Selected |
|--------|-------------|----------|
| Random unvisited gene | Canonical ERX algorithm (Whitley 1989) fallback — rare, keeps offspring valid. | ✓ |
| Return GaError | Error out on exhausted neighbors. | |

**User's choice:** Random unvisited gene (Recommended)

---

## ERX Minimum Length

| Option | Description | Selected |
|--------|-------------|----------|
| len >= 2 | Error for len < 2. Consistent with PMX's floor. | ✓ |
| len >= 3 | Stricter, matches Order crossover. | |

**User's choice:** len >= 2 (Recommended)

---

## ERX Chromosome Constraints

| Option | Description | Selected |
|--------|-------------|----------|
| Trust the user, no validation | Lazy pattern like Order, PMX, Cycle. | |
| Validate uniqueness at factory | Error if gene IDs not unique — O(n) HashSet check. | ✓ |

**User's choice:** Validate uniqueness at factory
**Notes:** User explicitly chose validation here, breaking from the pattern set by other permutation operators. ERX's behavior is more undefined than Order/PMX when genes repeat, so the early error is helpful.

---

## Claude's Discretion

- ERX adjacency-list data structure (HashMap<gene_id, HashSet<gene_id>> or Vec-based)
- ERX tie-breaking when multiple neighbors have equal fewest remaining neighbors
- Whether ERX produces 1 or 2 children per call
- Log target names: `crossover_events`, `mutation_events`
- Internal helper function names and loop structure

## Deferred Ideas

None — discussion stayed within phase scope.
