# Phase 25: Alternative Metaheuristics & Population Models - Context

**Gathered:** 2026-04-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 25 will implement four alternative metaheuristics: Differential Evolution, Scatter Search, Cellular GA, and ALPS engines as part of the genetic_algorithms library, following the existing architectural patterns of the library.

</domain>

<decisions>
## Implementation Decisions

### Engine Design
- Consistent Architecture: Follow existing patterns in the codebase
- Each engine should be implemented as a standalone module following the library's architectural patterns
- Engines should integrate with existing traits and interfaces

### Algorithms
- Full Implementation: Implement all mentioned algorithms and strategies from requirements
- JADE and L-SHADE variants for Differential Evolution
- Core Scatter Search algorithms with all optimization strategies
- Complete Cellular GA implementation with all neighborhood types
- Full ALPS implementation with all age schemes

### Integration
- Integration with existing traits and module structure
- Reuse existing configuration patterns and execution models
- Maintain compatibility with existing observer and reporting systems

### Performance
- Performance optimization is important but secondary to correctness
- Follow existing parallelization patterns from the GA engine
- Maintain consistency with existing performance characteristics

### Claude's Discretion
- Implementation details within each engine
- Specific algorithmic optimizations
- Internal architecture decisions

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core Architecture References
- `src/traits/*.rs` - Core trait definitions
- `src/operations/*` - Existing operation implementations
- `PROJECT.md` - Project level documentation

### Feature Documentation
- `docs/specs.md` - Feature requirements and specifications
- `docs/architecture.md` - System architecture documentation

</canonical_refs>

<specifics>
## Specific Ideas

- "Implementation should follow the patterns established by the existing GA engine"
- "Reuse existing trait system where possible"
- "Maintain consistency with existing codebase architecture"

</specifics>

<deferred>
## Deferred Ideas

None - all requirements are in scope for this phase

</deferred>

---
*Phase: 25-alternative-metaheuristics*
*Context gathered: 2026-04-24*