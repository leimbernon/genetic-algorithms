# Phase 31: Selection & Survivor Diversity Operators - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-04
**Phase:** 31-Selection & Survivor Diversity Operators
**Areas discussed:** Clearing: internal pairing, Clearing: niche distance metric, DeterministicCrowding: offspring detection, DeterministicCrowding: distance metric

---

## Clearing: Internal Pairing

| Option | Description | Selected |
|--------|-------------|----------|
| Random pairing | Pair eligible individuals randomly — consistent with existing Random selector, minimal selective pressure beyond clearing itself | ✓ |
| Tournament on eligible pool | Run tournament selection within the eligible subset — adds selective pressure on top of clearing | |
| You decide | Leave to planner/implementer | |

**User's choice:** Random pairing

---

| Option | Description | Selected |
|--------|-------------|----------|
| Standard clearing | One winner per niche survives, everyone else in radius is cleared. Eligible pool = winners + anyone not in any niche | ✓ |
| Just filter within-radius | Simpler: if within radius of any fitter individual, ineligible — no explicit niche winner concept | |

**User's choice:** Standard clearing semantics

---

| Option | Description | Selected |
|--------|-------------|----------|
| SelectionConfiguration | Add niche_radius: f64 to existing struct — consistent with Boltzmann temperature co-location | ✓ |
| New ClearingConfiguration | Dedicated struct — more isolated but more surface area | |

**User's choice:** SelectionConfiguration

---

## Clearing: Niche Distance Metric

| Option | Description | Selected |
|--------|-------------|----------|
| Fitness distance | \|f_a - f_b\| — generic, no type constraints, same scale as fitness function | ✓ |
| Genetic distance (Hamming) | Count positions where gene IDs differ — structural, but radius unit is "differing genes" | |
| Genetic distance (Euclidean) | √Σ(id_a - id_b)² on gene IDs — sensitive to magnitude, less portable | |

**User's choice:** Fitness distance

---

## DeterministicCrowding: Offspring Detection

| Option | Description | Selected |
|--------|-------------|----------|
| age() == 0 means offspring | Generic signal already on ChromosomeT — no API changes | ✓ |
| Ordering convention | First N entries are parents — fragile if merge order changes | |
| Split by population_size | First population_size entries are old population — same fragility | |

**User's choice:** age() == 0

---

| Option | Description | Selected |
|--------|-------------|----------|
| Keep the offspring | Unpaired offspring survive unconditionally — conservative | ✓ |
| Discard the offspring | Unpaired offspring dropped — stricter but can shrink pool | |
| You decide | Leave to implementer | |

**User's choice:** Keep the offspring

---

## DeterministicCrowding: Distance Metric

| Option | Description | Selected |
|--------|-------------|----------|
| Hamming on gene IDs | Count positions where gene_a.id() != gene_b.id() — works for any chromosome type | ✓ |
| Euclidean on gene IDs | √Σ(id_a - id_b)² — sensitive to magnitude, less portable | |
| Fitness distance | \|f_parent - f_offspring\| — fast but ignores genome structure | |

**User's choice:** Hamming on gene IDs

---

| Option | Description | Selected |
|--------|-------------|----------|
| Compare up to min length | Compare gene IDs at positions 0..min(len_a, len_b) — simple, safe | ✓ |
| Treat length difference as extra mismatches | distance = mismatches + \|len_a - len_b\| — more accurate but unusual case | |
| You decide | Leave to implementer | |

**User's choice:** Compare up to min length

---

## Claude's Discretion

- Internal implementation structure within clearing.rs and deterministic_crowding.rs
- Whether niche winner identification iterates sorted-by-fitness or uses another order
- Log target naming (follow existing patterns)

## Deferred Ideas

None.
