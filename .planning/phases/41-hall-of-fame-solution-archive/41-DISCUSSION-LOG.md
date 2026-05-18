# Phase 41 Discussion Log

**Date:** 2026-05-11
**Phase:** 41 — Hall of Fame / Solution Archive
**Mode:** Default (interactive)

## Areas Discussed

### 1. Diversity Filtering
- **Q:** What distance metric? → **Both modes configurable** (Fitness-space + Genotypic)
- **Q:** Default metric? → **Fitness-space (Euclidean)**
- **Q:** How threshold specified? → **Fixed f64 value**
- **Q:** Eviction policy? → **Remove worst fitness**

### 2. Archive Update Strategy
- **Q:** Update timing? → **Every generation (all offspring)**
- **Q:** Entry criteria? → **Top-N by fitness**
- **Q:** Mid-run access? → **Post-run only**
- **Q:** Relationship to best_chromosome? → **Supplement (both exist)**

### 3. Access Pattern
- **Q:** How to access? → **Public method on Ga**
- **Q:** Core API methods? → `.solutions()`, `.top(k)`, `.would_qualify()`, `.len()`
- **Q:** Extended API? → **Both serde + iter metadata**

### 4. Multi-Engine Support
- **Q:** Which engines? → **Ga only**
- **Q:** Builder placement? → **Ga builder only**

## Deferred Ideas
- Nsga2Ga Hall of Fame — separate phase
- De, Scatter, Cellular, Alps integration — future
- Mid-run access via observer hooks — future
- Relative distance thresholds — future
