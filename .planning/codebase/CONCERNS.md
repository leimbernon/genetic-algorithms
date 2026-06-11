# Codebase Concerns

**Focus:** Technical debt, known issues, security, performance, fragile areas
**Analyzed:** 2026-03-20

## Tech Debt

### Panic Risks
- **Order/PMX crossovers** (`src/operations/crossover/`) — Non-unique gene IDs cause panics. No validation that input chromosomes have unique gene IDs before running permutation crossovers.
- **Unwrap calls in initializers** — Several `unwrap()` calls in initialization paths that could panic on edge-case inputs.

### Error Handling
- **Error details lost in crossover dispatch** (`src/operations/crossover/`) — Factory dispatch swallows error context, making debugging harder.
- **Silent defaults in mutation parameters** — Mutation operators silently use defaults when sigma/step are 0.0 instead of returning an error.

### Cache Issues
- **Lock poisoning in fitness cache** (`src/fitness/`) — If a thread panics while holding the cache lock, the mutex becomes poisoned and subsequent calls will panic.

## Known Bugs

### Performance Bugs
- **PMX O(n²) on large permutations** (`src/operations/crossover/`) — Partially Mapped Crossover has quadratic complexity for large gene arrays.
- **O(n) cache lookup** (`src/fitness/`) — LRU cache lookup is O(n) instead of O(1) due to DNA hashing via Debug format strings.
- **Hash collision risk in fitness cache** — DNA hashed as Debug string; different chromosomes with same debug representation would collide.

### Validation Gaps
- **Island topology validation scattered** (`src/island/`) — Island migration configuration validated in multiple places inconsistently.
- **Test coverage gaps** — Some edge cases in crossover operators (empty chromosomes, single-gene chromosomes) not covered.

## Security

- **Mutex poison in multi-threaded fitness evaluation** (`src/fitness/`) — No recovery from poisoned mutex in parallel fitness evaluation.
- **No validation on custom island topologies** (`src/island/`) — Custom ring/mesh topologies don't validate that referenced island indices exist.

## Performance Bottlenecks

| Area | Location | Impact |
|------|----------|--------|
| Excessive cloning in crossovers | `src/operations/crossover/` | High — hot path |
| DNA hashing via Debug strings | `src/fitness/` | Medium — cache miss rate |
| O(n) LRU cache operations | `src/fitness/` | Medium — degrades with pop size |
| Large file complexity | `src/ga.rs` (1395 lines) | Low — maintainability |
| Rayon overhead for small pops | `src/ga.rs` | Low — only matters <100 individuals |

## Fragile Areas

- **Non-unique gene ID assumption** — Permutation crossovers (Order, PMX) silently produce wrong results if gene IDs are not unique. No precondition check.
- **Fitness cache correctness** — Cache keyed on Debug string; relies on deterministic Debug impl across versions.
- **Adaptive GA without bounds checking** — Adaptive parameter updates (`src/ga.rs`) can produce out-of-range values if fitness landscape is unusual.

## Missing Features / Test Gaps

- Integration tests for island model migration are sparse
- No benchmark regression tests to catch performance regressions
- NSGA-II crowding distance not tested with degenerate Pareto fronts
- No tests for checkpoint/restore round-trip with all operator types

---
*Mapped: 2026-03-20*
