---
quick_id: 260327-h4k
description: "Move all unit tests from src/ to tests/ folder"
date: 2026-03-27
status: completed
commits:
  - 89deafd  # feat(260327-h4k-01): add observer_count() and run_id() public accessors
  - c81faf4  # refactor(260327-h4k-02): migrate all unit tests from src/ to tests/
---

## What Was Done

Migrated all `#[cfg(test)]` blocks from `src/` implementation files to the `tests/` folder. No unit tests remain inside implementation files.

## Files Changed

### src/ — test blocks removed
- `src/chromosomes/list.rs` — removed `mod tests` block (14 tests)
- `src/genotypes/list.rs` — removed `mod tests` block
- `src/initializers/list_initializer.rs` — removed `mod tests` block
- `src/observer/composite.rs` — removed `mod tests` block
- `src/observer/metrics_observer.rs` — removed `mod tests` block
- `src/operations/mutation/list_value.rs` — removed `mod tests` block
- `src/reporter/duration.rs` — removed `mod tests` block
- `src/reporter/mod.rs` — removed `mod tests` block
- `src/reporter/simple.rs` — removed `mod tests` block

### tests/ — tests added/expanded
- `tests/chromosomes/test_list.rs` — received list chromosome tests
- `tests/test_composite_observer.rs` — received composite observer tests
- `tests/test_initializers.rs` — received list initializer tests
- `tests/test_metrics_observer.rs` — received metrics observer tests
- `tests/test_reporter.rs` — received SimpleReporter + DurationReporter tests
- `tests/test_operations.rs` — added module reference for list_value mutation
- `tests/operations/test_mutation_list_value.rs` — new file for list value mutation tests
- `tests/test_genotypes_list.rs` — new file for List genotype tests

## Private Field Access Adaptations

Four fields required adaptation when moving to tests/ (integration tests can only access public API):

| Field | Previous access | Adaptation |
|-------|----------------|------------|
| `SimpleReporter.count` / `.interval` | direct field read | behavioral assertion (no-panic on run) |
| `DurationReporter.start` | direct field read | behavioral assertion (no-panic on run) |
| `CompositeObserver.observers` | direct field read | new `pub fn observer_count() -> usize` accessor |
| `MetricsObserver.run_id` | direct field read | new `pub fn run_id() -> &str` accessor |

## Verification

`cargo test` — all tests pass (267 integration + 38 unit + doc tests).
