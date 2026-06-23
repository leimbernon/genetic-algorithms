---
phase: 77
slug: extend-fitness-cache-to-more-engines-issue-260
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-19
---

# Phase 77 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| User → Engine Config | `with_fitness_cache_size(size)` builder method | `usize` cache capacity (user-controlled) |
| Engine → FitnessCache | `wrap_with_cache()` wraps `fitness_fn` with LRU cache | DNA hash → fitness value mapping |
| Engine → GenerationStats | Per-generation cache hit/miss delta | `u64` counters (hits, misses) |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-77-01 | Tampering | `src/fitness/cache.rs` (hash_dna) | accept | DefaultHasher u64 produces ~2^-64 collision probability per pair. Same risk exists in pre-phase GA/CMA engines — no new surface. | closed |
| T-77-02 | Denial of Service | `src/fitness/cache.rs` (wrap_with_cache) | accept | Lock held only for O(1) HashMap get/put + VecDeque ops. Lock released before `fitness_fn()` call (cache.rs:170-184). Matches existing CMA/GA pattern. | closed |
| T-77-03 | Denial of Service | All 3 engine configs | mitigate | LRU cache bounded by user-specified capacity. `FitnessCache::new(capacity)` stores limit; `put()` evicts LRU when full. Cache disabled by default (`None`). | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-77-01 | T-77-01 | Hash collision probability (~2^-64) is negligible for all practical purposes. Same risk exists in GA/CMA engines since project inception. | gsd-security-auditor | 2026-06-19 |
| AR-77-02 | T-77-02 | Mutex contention is minimal — lock held for O(1) operations only. No contention in sequential engines (PSO, DE). In EDA with rayon parallelism, each chromosome acquires/releases independently (~microseconds). | gsd-security-auditor | 2026-06-19 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-19 | 3 | 3 | 0 | gsd-security-auditor |

---

## Implementation Pattern Consistency

All new engines follow the established CMA pattern exactly:

| Pattern | PSO | EDA (Bernoulli) | EDA (Gaussian) | DE |
|---------|-----|------------------|----------------|-----|
| Config field `fitness_cache_size: Option<usize>` | `pso/configuration.rs:152` | `eda/configuration.rs:68` | `eda/configuration.rs:68` | `de/configuration.rs:88` |
| Default `None` | `pso/configuration.rs:169` | `eda/configuration.rs:79` | `eda/configuration.rs:79` | `de/configuration.rs:103` |
| Builder `with_fitness_cache_size()` | `pso/configuration.rs:233` | `eda/configuration.rs:124` | `eda/configuration.rs:124` | `de/configuration.rs:159` |
| Engine struct field `fitness_cache` | `pso/engine.rs:182` | `eda/engine.rs:140` | `eda/engine.rs:491` | `de/engine.rs:61` |
| `wrap_with_cache()` call in `run()` | `pso/engine.rs:320-327` | `eda/engine.rs:273-280` | `eda/engine.rs:636-643` | `de/engine.rs:97-104` |
| Cache snapshot before loop | `pso/engine.rs:367-373` | `eda/engine.rs:337-343` | `eda/engine.rs:697-703` | `de/engine.rs:131-137` |
| Delta stats with `saturating_sub` | `pso/engine.rs:481-487` | `eda/engine.rs:407-413` | `eda/engine.rs:765-771` | `de/engine.rs:239-245` |
| Lock error message `expect("fitness cache lock poisoned")` | ✓ (2 sites) | ✓ (4 sites) | ✓ (2 sites) | ✓ (2 sites) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-19
