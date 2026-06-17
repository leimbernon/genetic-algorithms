# Phase 65: v3.0.0 Migration Guide & Release Notes - Context

**Gathered:** 2026-06-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 65 delivers the complete v3.0.0 release documentation package:
- A fully audited `MIGRATION.md` covering all 10 breaking changes (3 entries missing from the existing draft must be added), each with before/after code and a `### Compiler error` subsection showing the exact `error[E...]` rustc output
- Two new `## sections` in `MIGRATION.md` for the `parallel` and `logging` feature flag changes introduced by Phases 69/68
- An audited `CHANGELOG.md` where `[3.0.0] - Unreleased` is promoted to `[3.0.0] - 2026-06-17` (the empty `[Unreleased]` section is dropped/merged)
- A Plan 65-03 release-gate verification: full CI matrix + `cargo publish --dry-run` + minimal v2 sample crate smoke-test (top 3 breaking patterns) + `cargo run --example` smoke run

No new code or API changes. Documentation and verification only.

</domain>

<decisions>
## Implementation Decisions

### MIGRATION.md completeness
- **D-01:** Add all 3 missing breaking-change entries: `DeGene → RealGene` rename, `SelectionOperator::select` return-type change (new `num_parents` param + `Vec<Vec<usize>>` return), `Mutation` enum variant parameter changes.
- **D-02:** Add Phase 68/69 feature-flag changes as their own `##` sections (same level as other breaking changes): one section for `parallel` feature, one section for `logging` feature + `env_logger` removal.
- **D-03:** The `LinearChromosome` bound requirement (custom chromosomes must now implement `LinearChromosome`, not just `ChromosomeT`) is folded into the existing `## Trait split: ChromosomeT + LinearChromosome` section as a prominent callout — not a separate section (closely related concepts, keeps navigation coherent).

### Compiler error format
- **D-04:** Every breaking-change section gets a `### Compiler error` subsection with a fenced code block showing the exact `error[E...]` output rustc emits when a user has v2 code. Use stable rustc format (error code + message + file pointer line). This applies to all 10 breaking-change entries (including the 3 new ones).

### CHANGELOG date & structure
- **D-05:** Drop the empty `## [Unreleased]` section. Promote `## [3.0.0] - Unreleased` to `## [3.0.0] - 2026-06-17`. The existing content (phases 47–69, Added/Changed/Removed) is already comprehensive; Phase 65 verifies completeness and adds any gaps from phases 64–69 (doc quality, build-perf phases worth noting in "Architecture & quality" bucket).
- **D-06:** Coverage scope: all phases 47–69 summarized. Build-perf phases (66–69) are included under "Architecture & quality" — internal improvements that affect compile time are worth noting even if they don't change the public API.

### Release gate (Plan 65-03)
- **D-07:** Four-part release gate:
  1. Full CI matrix: `cargo test`, `cargo test --features serde`, `cargo clippy --all-targets -D warnings`, `cargo doc --no-deps --all-features` (zero warnings), `cargo check --target wasm32-unknown-unknown`
  2. `cargo publish --dry-run` to catch Cargo.toml issues, missing files, license/readme fields
  3. Minimal v2 sample crate smoke-test: create a small crate using the top 3 breaking patterns (`ChromosomeT+LinearChromosome` impl, `Reporter` removal, `SelectionOperator` trait impl), apply MIGRATION.md, confirm it compiles after migration
  4. `cargo run --example` smoke run for all examples in `examples/`
- **D-08:** The v2 sample crate for smoke-test should use top 3 breaking patterns only (ChromosomeT+LinearChromosome impl, Reporter removal, SelectionOperator trait impl). Does not need to solve a real optimization problem.

### Claude's Discretion
- **LinearChromosome bound callout style** within the ChromosomeT split section — decide on the most readable formatting (e.g., a `> **Note:** If you implemented ChromosomeT directly...` blockquote or a dedicated paragraph).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing release artifacts to update
- `MIGRATION.md` — already covers 7 of 10 breaking changes; planner must audit against D-01–D-03 for the 3 missing entries
- `CHANGELOG.md` — `## [3.0.0]` section exists; planner checks gaps for phases 64–69
- `README.md` — already has upgrade banner and MIGRATION.md links; verify they're correct

### Breaking-change source of truth
- `.planning/ROADMAP.md` §Phase 65 Success Criteria — canonical list of 10 required breaking-change entries
- `src/traits/operators.rs` — `SelectionOperator::select` current signature (for compiler error example)
- `src/traits/real_gene.rs` — `RealGene` trait (former `DeGene`)
- `src/operations/mutation.rs` — `Mutation` enum current variant structure

### Phase 68/69 feature changes
- `.planning/phases/68-build-perf-m2-dependency-hygiene/68-CONTEXT.md` — `logging` feature + env_logger removal decisions
- `.planning/phases/69-build-perf-m3-major-refactors/69-CONTEXT.md` — `parallel` feature + rayon gating decisions

### Release gate tooling
- `CHANGELOG.md` Keep-a-Changelog format — follow existing sections for consistency
- `Cargo.toml` — check `readme`, `license`, `description` fields for `cargo publish --dry-run`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `MIGRATION.md` (7 entries): established before/after style with `### Before` / `### After` H3 subsections; add `### Compiler error` as a third subsection to each entry
- `CHANGELOG.md` `## [3.0.0]` section: already comprehensive for phases 47–67; gaps to fill are phases 64 (doc quality) and 65 (this phase itself)
- `README.md` upgrade banner: already exists at line 4 (`> **v3.0.0 users:** see MIGRATION.md`); no changes needed unless links are stale

### Established Patterns
- MIGRATION.md uses H2 (`##`) for each breaking change and H3 (`###`) for Before/After subsections — follow this for the 5 new sections (3 missing API changes + 2 feature flags)
- CHANGELOG.md uses Keep-a-Changelog bucketing (Added / Changed (breaking) / Removed / Architecture & quality)

### Integration Points
- Plan 65-03 creates a temporary v2 sample crate (outside repo, in `/tmp` or a throwaway dir); verify it compiles after applying MIGRATION.md

</code_context>

<specifics>
## Specific Ideas

- For `### Compiler error` subsections: use real rustc error format — `error[E0412]: cannot find trait 'DeGene' in module 'traits'` with a `-->` pointer. Run `rustc` against actual v2 code patterns to capture exact error text where possible.
- The `[3.0.0]` compare link at the bottom of CHANGELOG.md currently points to `2.4.0...HEAD`; Phase 65 should update it to `2.4.0...v3.0.0` once the tag exists — or leave `HEAD` if no tag is cut during this phase.

</specifics>

<deferred>
## Deferred Ideas

- crates.io README rendering check (verify badges, images render on crates.io page) — not required for Phase 65; left to the human before manual publish
- docs.rs preview — left to the human post-publish

</deferred>

---

*Phase: 65-v3-0-0-migration-guide-release-notes*
*Context gathered: 2026-06-17*
