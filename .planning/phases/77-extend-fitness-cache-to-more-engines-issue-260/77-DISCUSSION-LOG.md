# Phase 77: Extend Fitness Cache to More Engines - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-19
**Phase:** 77-extend-fitness-cache-to-more-engines-issue-260
**Areas discussed:** Engine wiring pattern, Cache stats reporting, Engine-specific behavior, Cache lifecycle

---

## Engine wiring pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Ga pattern: build() wrap | Cache wraps fitness_fn inside build() — keeps builder API consistent | ✓ |
| CMA pattern: engine-level wrap | Cache wraps fitness_fn inside engine struct constructor — more explicit but inconsistent | |
| You decide | Pick the pattern that best fits each engine's architecture | |

**User's choice:** Ga pattern: build() wrap (Recommended)
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Optional builder method | Same as Ga: with_fitness_cache_size() builder method. Users opt-in. | ✓ |
| Always enabled | Automatically wrap fitness_fn with cache whenever provided | |
| You decide | Pick what fits the library's philosophy | |

**User's choice:** Optional builder method (Recommended)
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Automatic via FitnessCache | FitnessCache is already WASM-compatible. No cfg-gating needed. | ✓ |
| Per-engine cfg-gate | Each engine wraps cache in #[cfg] blocks. More explicit but redundant. | |
| You decide | Pick the safest approach | |

**User's choice:** Automatic via FitnessCache (Recommended)
**Notes:** None

---

## Cache stats reporting

| Option | Description | Selected |
|--------|-------------|----------|
| Per-generation via GenerationStats | Same as Ga: call cache_snapshot() each generation | ✓ |
| Final result only | Only expose total hits/misses in engine Result type | |
| Both: per-gen + final summary | Populate GenerationStats AND include totals in Result | |
| You decide | Pick what fits the library's pattern | |

**User's choice:** Per-generation via GenerationStats (Recommended)
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Optional: None when disabled | cache_hits/misses are Option<u64>. None = no cache configured. | ✓ |
| Always present: 0 when disabled | Change to u64 with 0 default. Simpler but loses 'no cache' signal. | |
| You decide | Pick what's clearest for users | |

**User's choice:** Optional: None when disabled (Recommended)
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Only GenerationStats | Keep it simple — stats are in the generation log. | ✓ |
| Also on Result type | Add cache_stats() to each engine's Result for quick access. | |
| You decide | Pick what's most useful | |

**User's choice:** Only GenerationStats (Recommended)
**Notes:** None

---

## Engine-specific behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Identical for all | Same cache_size builder, same wrap_with_cache() call for all three engines | ✓ |
| Engine-specific defaults | Different default cache sizes per engine based on expected duplicate rate | |
| Skip for low-duplicate engines | Don't add cache to EDA if duplicates are rare | |
| You decide | Pick what fits the library's philosophy | |

**User's choice:** Identical for all (Recommended)
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Only main evaluations | Cache wraps primary fitness_fn call; personal_best benefits automatically | ✓ |
| Explicitly wrap personal_best too | Add separate cache check in personal_best update | |
| You decide | Pick the simpler approach | |

**User's choice:** Only main evaluations (Recommended)
**Notes:** None

---

## Cache lifecycle

| Option | Description | Selected |
|--------|-------------|----------|
| LRU eviction only | Let cache accumulate across generations. Hot entries stay warm. | ✓ |
| Clear between generations | Reset cache each generation. Prevents stale values but loses hot entries. | |
| You decide | Pick what's simplest | |

**User's choice:** LRU eviction only (Recommended)
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Persistent across generations | Same cache instance for entire run. Hot entries stay warm. | ✓ |
| Fresh each generation | New cache each generation. No stale values but loses all warm-up. | |
| You decide | Pick what's most efficient | |

**User's choice:** Persistent across generations (Recommended)
**Notes:** None

---

## the agent's Discretion

- Whether to add a brief comment at each engine's cache wiring explaining the pattern
- Exact order of builder methods in each engine's impl block
- Whether to add a benchmark demonstrating cache benefit on a deterministic problem per engine

## Deferred Ideas

None — discussion stayed within phase scope.
