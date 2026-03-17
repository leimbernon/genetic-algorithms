Add a new genetic algorithm operator to the library.

Operator type and name: $ARGUMENTS

Follow this checklist:

1. **Determine operator category**: selection, crossover, mutation, survivor, or extension
2. **Add enum variant** to the corresponding enum in `src/operations/<category>.rs` (e.g., `Selection::NewMethod`)
3. **Create implementation file** at `src/operations/<category>/<name>.rs`
4. **Implement the operator trait** (SelectionOperator, CrossoverOperator, MutationOperator, SurvivorOperator, or ExtensionOperator)
5. **Add factory branch** in the `factory()` function in `src/operations/<category>.rs`
6. **Add configuration parameters** (if needed) to the appropriate struct in `src/configuration.rs`
7. **Add builder methods** (if needed) to the trait in `src/traits/configuration.rs`
8. **Add mod declaration** and re-export in `src/operations/<category>.rs`
9. **Write unit tests** in the new file
10. **Run full test suite**: `cargo test && cargo test --features serde && cargo clippy`

Reference existing operators in the same category for patterns and conventions.

Performance rules:
- Use `dna_mut()` / `set_gene()` for in-place mutation, never `dna().to_vec()` for single-gene changes
- Pre-allocate Vecs with `with_capacity()`
- Use `partition_point()` instead of `position()` for sorted lookups
- Avoid cloning parents in crossover — build children directly
