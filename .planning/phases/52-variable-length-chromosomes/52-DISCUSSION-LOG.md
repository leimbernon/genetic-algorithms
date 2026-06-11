# Phase 52: Variable-Length Chromosomes - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-24
**Phase:** 52-variable-length-chromosomes
**Areas discussed:** Mutation naming conflict, AlignmentStrategy variants, Parsimony pressure integration, Extension length sampling

---

## Mutation naming conflict

| Option | Description | Selected |
|--------|-------------|----------|
| Rename existing → Mutation::PermutationInsert | Rename current permutation-move operator. Use Insertion for new length-growing operator. Most semantically clear. | ✓ |
| New names: GeneGrow / GeneShrink | Keep existing Mutation::Insertion as-is. Name new operators GeneGrow and GeneShrink. No rename needed. | |
| You decide | Let the planner pick naming based on EA literature. | |

**User's choice:** Rename existing → `Mutation::PermutationInsert`

| Option | Description | Selected |
|--------|-------------|----------|
| Random from allele set | Sample new gene value from the allele set (same as initialization). | ✓ |
| Clone a random existing gene | Duplicate an existing gene at the insertion point. | |
| GeneT::new() | Use the gene's default constructor. | |

**User's choice:** Random from the chromosome's allele set

| Option | Description | Selected |
|--------|-------------|----------|
| Variable only — return GaError if Fixed | Return error if ChromosomeLength::Fixed is configured. Matches spec. | ✓ |
| Unconditional — no config check | Always apply regardless of ChromosomeLength. | |

**User's choice:** Variable only — return `GaError::MutationError` if `ChromosomeLength::Fixed`

---

## AlignmentStrategy variants

| Option | Description | Selected |
|--------|-------------|----------|
| Two variants: Trim + Pad | Trim aligns to min length; Pad fills shorter to max length. Covers the two most common approaches. | ✓ |
| One variant: Trim only | Only Trim. Simplest implementation. | |
| Three variants: Trim + Pad + RandomOffset | Also adds random offset alignment. More exploratory but higher complexity. | |

**User's choice:** Two variants: `AlignmentStrategy::Trim` and `AlignmentStrategy::Pad`

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed single-point within aligned region | After alignment, apply single-point crossover. Simple, well-understood. | ✓ |
| Uniform crossover within aligned region | Gene-by-gene coin flip after alignment. | |
| Configurable — pass a nested Crossover variant | VariableLength carries inner: Box<Crossover>. Heap allocation required. | |

**User's choice:** Fixed single-point within the aligned region

| Option | Description | Selected |
|--------|-------------|----------|
| Random from alleles | Sample padding genes from allele set. Consistent with Mutation::Insertion. | ✓ |
| Clone the nearest existing gene | Repeat last gene of shorter chromosome. | |
| Default gene value (GeneT::new()) | Fill with gene type's default. | |

**User's choice:** Random from alleles

---

## Parsimony pressure integration

| Option | Description | Selected |
|--------|-------------|----------|
| SurvivorConfiguration field | length_penalty: Option<f64> in SurvivorConfiguration. Matches CHR-02 spec. | ✓ |
| Top-level GaConfiguration field | New field at top level, applied each generation. More visible but separate concern. | |
| ExtensionConfiguration field | Attached to extension config, applies only during extension events. Unusual placement. | |

**User's choice:** `SurvivorConfiguration` field

| Option | Description | Selected |
|--------|-------------|----------|
| adjusted = fitness - (penalty × length) | Linear subtraction. Maximization penalizes longer; Minimization adds. | ✓ |
| adjusted = fitness × (1 - penalty × normalized_length) | Multiplicative penalty based on relative length. More complex. | |
| You decide | Let planner pick from EA literature. | |

**User's choice:** `adjusted_fitness = fitness ∓ (length_penalty × chromosome_length)` (sign per mode)

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — auto-adjust sign per ProblemSolving mode | Same positive length_penalty works for both Maximization and Minimization. | ✓ |
| Maximization only — users negate for Minimization | Always subtract; Minimization users pass negative value. | |

**User's choice:** Auto-adjust sign per `ProblemSolving` mode

---

## Extension length sampling

| Option | Description | Selected |
|--------|-------------|----------|
| Uniform from [min_observed, max_observed] of surviving population | Compute min/max DNA lengths of survivors; sample uniformly. Adaptive. | ✓ |
| Uniform from [config.min, config.max] (ChromosomeLength bounds) | Always use configured bounds regardless of population state. | |
| Sample length from a survivor (clone + remutate) | Clone a random survivor and apply Insertion/Deletion for length variation. | |

**User's choice:** Uniform from `[min_observed, max_observed]` of surviving population

| Option | Description | Selected |
|--------|-------------|----------|
| Pass sampled length through existing genes_per_chromosome parameter | Zero changes to init_fn signature. Variable-length sampling entirely in extension operator. | ✓ |
| Add new length parameter to ExtensionConfiguration | Thread ChromosomeLength through config struct. Requires config change. | |
| Add length_sampler closure to init_fn | Most flexible but changes init_fn signature and affects all callers. | |

**User's choice:** Pass sampled length as `genes_per_chromosome` to existing `init_fn`

---

## Claude's Discretion

None — all areas had clear user selections.

## Deferred Ideas

None — discussion stayed within phase scope.
