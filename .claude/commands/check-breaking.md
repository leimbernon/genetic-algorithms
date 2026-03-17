Analyze the current uncommitted changes (or a specific branch diff) for breaking changes.

Usage: /check-breaking [branch-to-compare]

If no argument: analyze uncommitted changes. If argument provided: `git diff $ARGUMENTS...HEAD`

Check for these categories of breaking changes:

1. **Trait signature changes**: Modified method signatures in `ChromosomeT`, `GeneT`, `SelectionOperator`, `CrossoverOperator`, `MutationOperator`, `SurvivorOperator`, `ExtensionOperator`, `ConfigurationT` and sub-traits
2. **Enum variant removal/rename**: Changes to `Selection`, `Crossover`, `Mutation`, `Survivor`, `Extension`, `ProblemSolving`, `TerminationCause` enums
3. **Struct field changes**: Modified public fields in `GaConfiguration`, `Population`, `GenerationStats`, `LimitConfiguration`, and other public config structs
4. **Public function signature changes**: Any public fn whose parameters or return type changed
5. **Removed public items**: Any `pub` item that was removed or made private
6. **Type alias changes**: Changes to `FitnessFn`, `InitializationFn`, etc.

For each finding, report:
- What changed
- Whether it's a compile-breaking change for downstream users
- Suggested mitigation (deprecation, re-export, default implementation)

If no breaking changes are found, confirm that the changes are backward-compatible.
