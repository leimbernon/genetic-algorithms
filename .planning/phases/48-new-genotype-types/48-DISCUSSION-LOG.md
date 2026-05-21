# Phase 48: New Genotype Types - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-21
**Phase:** 48-new-genotype-types
**Areas discussed:** UniqueChromosome gene type, Invalid operator guard, MultiRangeChromosome gene type, MultiUniqueChromosome groups

---

## UniqueChromosome Gene Type

### Q1: What should UniqueChromosome<T> use for its gene type?

| Option | Description | Selected |
|--------|-------------|----------|
| New UniqueGenotype<T> | Thin `{ id: i32, value: T }` struct. Chromosome holds `alphabet: Arc<[T]>` once. Clean public API: `dna()[0].value` is the element. Follows existing pattern. | ✓ |
| Reuse List<T> | No new gene type. `List<T>` has alleles Arc per gene — O(1) alloc but larger per-gene struct. Less intuitive API for permutation problems. | |
| You decide | Claude picks. | |

**User's choice:** New UniqueGenotype<T>

---

### Q2: How should UniqueChromosome store its alphabet?

| Option | Description | Selected |
|--------|-------------|----------|
| Alphabet on the chromosome struct | `{ dna, alphabet: Arc<[T]>, fitness, age, fitness_fn }`. Initializer shuffles alphabet. | ✓ |
| Alphabet inferred from DNA only | No stored alphabet — derived at init time. Simpler struct but can't validate completeness post-mutation. | |

**User's choice:** Alphabet on the chromosome struct

---

### Q3: Where should the initializer live?

| Option | Description | Selected |
|--------|-------------|----------|
| New unique_initializer.rs in src/initializers/ | `unique_random_initialization(alphabet: &[T]) -> Vec<UniqueGenotype<T>>`. Follows established module pattern. | ✓ |
| Method on UniqueChromosome directly | `UniqueChromosome::random_from(alphabet)`. Breaks from initializer pattern. | |

**User's choice:** New unique_initializer.rs in src/initializers/

---

## Invalid Operator Guard

### Q1: Where should invalid crossover detection live?

| Option | Description | Selected |
|--------|-------------|----------|
| Build-time check in Ga::build() | `ConfigurationError` if chromosome's valid set excludes selected operator. Fails fast. Requires `compatible_crossovers()` method. | ✓ |
| Runtime method on ChromosomeT | Add `check_crossover_compatibility(crossover)` to trait. Called per crossover operation. Introduces coupling between traits and operations. | |

**User's choice:** Build-time check in Ga::build()

---

### Q2: How does Ga<U> know which crossovers are valid for U?

| Option | Description | Selected |
|--------|-------------|----------|
| New OperatorCompat trait | Optional trait with `valid_crossovers() -> Option<&'static [Crossover]>` and `valid_mutations()`. Default `None` = no restriction. | ✓ |
| Hardcode in validators with TypeId | TypeId check with generics is fragile in Rust. | |

**User's choice:** New `OperatorCompat` trait on ChromosomeT

---

### Q3: Should OperatorCompat cover mutation operators too?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — cover crossover and mutation | `valid_crossovers()` + `valid_mutations()`. UniqueChromosome rejects BitFlip, Gaussian. | ✓ |
| Crossover only (Phase 48 scope) | GEN-01 only specifies crossover restriction. Mutation deferred. | |

**User's choice:** Yes — cover crossover and mutation

---

## MultiRangeChromosome Gene Type

### Q1: What gene type should MultiRangeChromosome<T> use?

| Option | Description | Selected |
|--------|-------------|----------|
| New MultiRangeGenotype<T> | `{ id, lo, hi, value, mutation_rate }` — flat, explicit, no Arc overhead. Clean per-gene bounds API. | ✓ |
| Reuse Range<T> gene | `Range<T>` already has `ranges: Arc<[(T, T)]>` per gene. No new type but multi-allele semantics are a mismatch. | |

**User's choice:** New MultiRangeGenotype<T>

---

### Q2: How are per-gene bounds provided by the user?

| Option | Description | Selected |
|--------|-------------|----------|
| Vec<(T, T)> at initialization | `with_bounds(vec![(0.0,1.0), (-5.0,5.0)])` config API. Initializer maps each tuple to a gene's `(lo, hi)`. | ✓ |
| Stored on each gene at construction | User constructs `Vec<MultiRangeGenotype<T>>` manually. More verbose. | |

**User's choice:** Vec<(T, T)> at initialization

---

### Q3: Include per-gene mutation rate p_i (GEN-03)?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — include per-gene mutation rate (GEN-03 scope) | `mutation_rate: f64` field on gene. Gaussian mutation uses per-gene rate. Ships as specified. | ✓ |
| Defer per-gene rate, bounds-only | Deviates from GEN-03 spec. | |

**User's choice:** Yes — include per-gene mutation rate p_i

---

## MultiUniqueChromosome Groups

### Q1: How should group boundaries be represented?

| Option | Description | Selected |
|--------|-------------|----------|
| Vec<Vec<T>> alphabets per group | `groups: Vec<Arc<[T]>>` on chromosome. User provides `Vec<Vec<T>>`. Boundaries derived from alphabet lengths. Most semantic. | ✓ |
| Vec<usize> group sizes | Simpler but loses alphabet — can't validate permutation completeness. | |
| Vec<(usize,usize)> start/end pairs | Explicit but user must compute offsets manually. | |

**User's choice:** Vec<Vec<T>> alphabets per group

---

### Q2: Does MultiUniqueChromosome use UniqueGenotype<T>?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — reuse UniqueGenotype<T> | Same gene type. Group membership implicit from DNA position. | ✓ |
| New MultiUniqueGenotype<T> | Adds `group_id: usize` field — duplicates information already in chromosome's groups vec. | |

**User's choice:** Yes — reuse UniqueGenotype<T>

---

### Q3: How does the crossover operator know group boundaries?

| Option | Description | Selected |
|--------|-------------|----------|
| Chromosome exposes group_ranges() method | `fn group_ranges(&self) -> Vec<(usize, usize)>`. Operator calls this and applies PMX/OX within each range. | ✓ |
| MultiUnique-specific crossover variants | New `Crossover::MultiGroupPmx` / `Crossover::MultiGroupOx`. | |

**User's choice:** Chromosome exposes group_ranges() method
**Notes:** Implies need for `Crossover::MultiGroupPmx` and `Crossover::MultiGroupOx` variants that call `group_ranges()` internally.

---

## Claude's Discretion

None — user engaged with all questions.

## Deferred Ideas

None — discussion stayed within phase scope.
