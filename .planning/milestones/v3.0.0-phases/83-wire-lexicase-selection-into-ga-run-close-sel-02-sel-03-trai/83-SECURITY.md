---
phase: 83
slug: wire-lexicase-selection-into-ga-run-close-sel-02-sel-03-trai
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-23
---

# Phase 83 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| user config → run path | User-supplied `Selection` enum value crosses into the run loop and selects the dispatch branch | Selection enum (non-sensitive, enum variant) |
| test harness → engine API | Tests exercise the public `run_lexicase` / `run` surface | Test-only fixture data, no production secrets |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-83-01 | Tampering | `run_with_callback` selection dispatch | mitigate | `matches!` guard at `src/engines/ga/mod.rs:1279` rejects `Lexicase`/`EpsilonLexicase` on the non-VectorFitness path with a clear `ConfigurationError` naming `run_lexicase`; unreachable arm in `factory()` eliminated | closed |
| T-83-02 | Denial of Service | `factory_lexicase` num_parents | mitigate | `factory_lexicase` API enforces the 2-parent constraint internally (does not accept `num_parents`); comment at `src/engines/ga/mod.rs:2717` documents the Pitfall 3 rationale; multi-parent crossover config cannot silently desync | closed |
| T-83-03 | Information Disclosure | error messages | accept | Error text names the public method `run_lexicase`; this is intentional API guidance with no sensitive data exposure | closed |
| T-83-T1 | Repudiation | test coverage | mitigate | All 5 integration test functions present in `tests/engines/lexicase/test_ga_run_lexicase.rs`: `test_ga_run_lexicase_completes`, `test_ga_run_epsilon_lexicase_completes`, `test_run_lexicase_on_non_vector_fitness_returns_error`, `test_lexicase_mean_sync_in_run`, `test_run_lexicase_diversity` — closes the Phase 50 "method existence is sufficient" gap | closed |
| T-83-T2 | Denial of Service | flaky diversity test | accept | Diversity test uses small populations and a tolerant fallback assertion; `rng_seed` applied for determinism where available | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| R-83-01 | T-83-03 | Error messages are intentional API guidance; no PII or internal implementation details are exposed — method names are part of the public API surface | Phase 83 security audit | 2026-06-23 |
| R-83-02 | T-83-T2 | Diversity test is non-deterministic by nature of random selection; small population + tolerant assertion bounds the flakiness surface; deterministic seed applied where API permits | Phase 83 security audit | 2026-06-23 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-23 | 5 | 5 | 0 | gsd-secure-phase (orchestrator, register_authored_at_plan_time: true) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-23
