---
phase: 60
slug: batch-fitness-fitness-cache-extension
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-10
---

# Phase 60 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| User trait impl | `Arc<dyn BatchFitnessEvaluator<U>>` runs in-process; same trust posture as existing `Arc<dyn GaObserver<U>>` — user-controlled code, fully trusted | Chromosome references + `Vec<f64>` fitness values |
| Cache handle | `Arc<Mutex<FitnessCache>>` shared between engine and caller | Fitness scalar values (non-sensitive numeric metrics) |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-60-01 | Tampering | `evaluate_batch` returns wrong-length `Vec<f64>` | mitigate | `debug_assert_eq!(values.len(), chromosomes.len())` at both Ga (`ga.rs:2700, 2739`) and CMA (`cma/engine.rs:419, 450`) call sites; trait contract documented | closed |
| T-60-02 | Denial of Service | `FitnessCache` mutex poisoning if user code panics inside a lock critical section | accept | `.expect("fitness cache lock poisoned")` at all lock sites — matches existing `wrap_with_cache` pattern; panic surfaces immediately rather than corrupting state | closed |
| T-60-03 | Information Disclosure | Cache delta stats expose hit/miss counts to observers | accept | Counts are non-sensitive numeric metrics; no PII; intentional feature | closed |
| T-60-04 | Tampering | Caller sets both `fitness_fn` and `batch_evaluator` | mitigate | Ga: `build()` returns `GaError::ConfigurationError` (`ga.rs:798–799`). CMA: `batch_evaluator.is_some()` gate skips scalar path at `run()` — last-writer-wins semantics per D-03 CMA discretion | closed |
| T-60-05 | Denial of Service | Cache lock held across expensive `evaluate_batch` call (e.g. GPU latency) blocks all mutex waiters | mitigate | Lock scope explicitly closed before `evaluate_batch` call; re-acquired only for cache puts — comment `// Lock released (Pitfall 2)` at `ga.rs:2728` and `cma/engine.rs:444` | closed |
| T-60-SC | Tampering | Supply chain (cargo/npm/pip installs) | n/a | No new external dependencies introduced in Phase 60 — RESEARCH.md confirms zero packages added | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party) · n/a (not applicable)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-60-01 | T-60-02 | Mutex poisoning on user-code panic is the same risk posture as all other `Arc<Mutex<T>>` in the codebase. Panic-on-poison is idiomatic Rust for in-process state; the alternative (returning `PoisonError`) would require every caller to handle a case that only occurs when user code is already broken. | leimbernon | 2026-06-10 |
| AR-60-02 | T-60-03 | Hit/miss counts are aggregate performance metrics with no PII or sensitive business data. Exposing them is the explicit purpose of the cache handle; suppressing them would defeat the feature. | leimbernon | 2026-06-10 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-10 | 6 | 6 | 0 | gsd-secure-phase (short-circuit: register_authored_at_plan_time=true, threats_open=0) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer / n/a)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-10
