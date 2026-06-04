# Phase 57: PSO Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-01
**Phase:** 57-pso-engine
**Areas discussed:** Velocity storage, Topology / neighborhood, Inertia weight strategy, Boundary enforcement

---

## Velocity Storage

| Option | Description | Selected |
|--------|-------------|----------|
| Engine-internal state | PsoEngine holds Vec<Vec<f64>> parallel to population. No chromosome API changes — same LinearChromosome + RealGene bound as CmaEngine. CmaState precedent. | ✓ |
| Chromosome wrapper trait | Define PsoChromosome: LinearChromosome { velocity / set_velocity }. Cleaner encapsulation, but adds a new trait. | |
| You decide | Delegate to Claude. | |

**User's choice:** Engine-internal state  
**Follow-up — velocity initialization:**

| Option | Description | Selected |
|--------|-------------|----------|
| Derived from gene bounds | v_init ∈ [-(hi-lo), +(hi-lo)] per gene — standard PSO practice. No extra config. | ✓ |
| User-specified v_max | Single v_max in PsoConfiguration; uniform across genes. | |
| Zero velocity | Start at rest; statistically biased. | |

**User's choice:** Derived from gene bounds

---

## Topology / Neighborhood

| Option | Description | Selected |
|--------|-------------|----------|
| Global best (gbest) only | All particles toward single best ever. Fastest convergence, simplest. | |
| gbest + ring (lbest) | Both via PsoTopology enum. Ring: each particle uses best in k-nearest neighbors. | ✓ |
| You decide | Claude picks gbest only. | |

**User's choice:** gbest + ring (lbest) — both from day one  
**Follow-up — ring neighborhood size:**

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed k neighbors | neighborhood_size: usize in PsoConfiguration. Particle i's neighbors: (i ± k/2) mod n. | ✓ |
| Fraction of swarm | neighborhood_fraction: f64. Scales automatically but less intuitive. | |
| You decide | Delegate to Claude. | |

**User's choice:** Fixed k neighbors

---

## Inertia Weight Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Both: static + linear decay as enum | PsoInertia::Constant(w) and PsoInertia::LinearDecay { w_start, w_end }. Covers demo and production use cases. | ✓ |
| Static w only | Single f64 field. Linear decay is future enhancement. | |
| Linear decay only | Always decays; less flexible. | |

**User's choice:** Both as PsoInertia enum  
**Follow-up — v_max in inertia config?:**

| Option | Description | Selected |
|--------|-------------|----------|
| Separate — handle in boundary enforcement | Keep inertia config focused on w only. v_max belongs with boundary. | ✓ |
| Include v_max in PsoConfiguration alongside w | One location for all velocity-limiting behavior. | |

**User's choice:** Separate — handle in boundary enforcement

---

## Boundary Enforcement

| Option | Description | Selected |
|--------|-------------|----------|
| Clamp + zero velocity component | Position clamped to [lo, hi]; velocity zeroed at wall. Simple, stable, most common default. | ✓ |
| Reflect (elastic wall) | Position reflected; velocity negated. More exploration but can oscillate. | |
| Both as PsoBoundary enum | PsoBoundary::Absorb and PsoBoundary::Reflect. More config complexity. | |

**User's choice:** Clamp + zero velocity component (absorbing)  
**Follow-up — v_max configurable?:**

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-derived, not configurable | v_max_i = hi_i - lo_i per gene. Standard practice. No config field. | ✓ |
| Optional config field | v_max: Option<f64> in PsoConfiguration. None = auto. | |

**User's choice:** Auto-derived, not configurable

---

## Claude's Discretion

- `PsoState` internal field names and struct layout
- Personal best update rule (strict improvement vs. ≥)
- `PsoResult<U>` field set
- Whether `GenerationStats` gets PSO-specific fields (`swarm_velocity_norm`)
- Default `neighborhood_size` for ring topology
- Whether `c1` and `c2` are exposed in config or hardcoded defaults

## Deferred Ideas

- **Constriction factor** — Alternative to inertia weight (Clerc-Kennedy); future `PsoInertia::Constriction` variant
- **Velocity-based stagnation stopping** — Stop when swarm velocity norm < threshold
- **Discrete PSO (BPSO)** — Binary/categorical variants; separate concern
- **Adaptive c1/c2** — APSO variants with time-varying coefficients
