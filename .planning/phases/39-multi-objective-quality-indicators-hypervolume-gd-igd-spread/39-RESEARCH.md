# Phase 39: Multi-objective quality indicators — Hypervolume, GD, IGD, Spread - Research

**Researched:** 2026-05-10
**Domain:** Multi-objective optimization quality indicators
**Confidence:** HIGH

## Summary

This phase adds a shared `src/engines/multi_objective/indicators/` directory exposing Hypervolume (2D exact Lebesgue), Generational Distance (GD), Inverted Generational Distance (IGD), and Spread (Deb et al. 2002) as pure functions. These are consumed by Phase 38 engines (SMS-EMOA, IBEA) and callable from user code for post-run Pareto-front analysis.

All four indicators are pure functions taking `&[Vec<f64>]` point sets and returning `Result<f64, GaError>`. No structs, no state, no pre-computation. The module sits in `src/engines/multi_objective/indicators/` following the shared-MOO-utility pattern established in Phase 35.

The algorithms are well-established in the multi-objective optimization literature (Zitzler & Thiele 1999 for hypervolume, Van Veldhuizen & Lamont 2000 for GD, Coello Coello & Cruz Cortes 2005 for IGD, Deb et al. 2002 for Spread). No new dependencies, WASM-compatible by construction (pure math, no `std::time` or `rayon`).

**Primary recommendation:** Create four files + mod.rs + two public validation helpers, one GaError variant, modify the multi_objective mod.rs, and four test files. No structs, no feature flags, no cfg-gating.

### Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MOO-05 | User can compute Hypervolume, GD, IGD, and Spread from any set of Pareto-front solutions via a shared quality-indicator module | Four pure functions in `src/engines/multi_objective/indicators/`, each returning `Result<f64, GaError>`, with integration tests using analytically-known ZDT/DTLZ reference fronts |

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Indicators live in `src/engines/multi_objective/indicators/` — a directory with one file per indicator + a `mod.rs` that re-exports all four public functions. All multi-objective engines import via `crate::multi_objective::indicators::*`.
- **D-02:** All four indicators are pure functions with signatures as specified in CONTEXT.md. No structs, no pre-computation. Stateless.
- **D-03:** 2-objective exact hypervolume only. Algorithm: sort points by first objective, then accumulate rectangular slices (Lebesgue measure). O(n log n). Returns `GaError::InvalidIndicatorConfiguration` if objectives != 2 or if reference point is not strictly dominated by all points.
- **D-04:** No hardcoded Pareto-front data in the library. GD and IGD require the true front as a `&[Vec<f64>]` parameter. Tests define analytically-known ZDT/DTLZ reference fronts inline in test code only.
- **D-05:** All indicator functions return `Result<f64, GaError>`. New `GaError` variant: `InvalidIndicatorConfiguration(String)`.

### Claude's Discretion

- Exact algorithm selection for each indicator: 2D hypervolume uses sort-then-sweep (Lebesgue measure). GD/IGD use standard Euclidean-distance-to-nearest formulas. Spread uses the Deb et al. 2002 definition (extreme-point distance + uniformity measure).
- Internal validation helpers for common checks (empty sets, dimension consistency, reference point dominance).
- `power` parameter on GD/IGD defaults to 2.0 (standard p=2 Euclidean norm).
- WASM: no `Instant` or `rayon` needed — pure functions compile for wasm32 without cfg-gating.
- No new feature flags — indicators are always available.
- `#[path]` re-exports for any backward-compat nsga2::indicators paths if needed.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

### Explicit Out of Scope
- 3+ objective hypervolume (exact WFG or Monte Carlo)
- Hardcoded Pareto-front reference sets in the library
- Epsilon-indicator function (I_eps+) — Phase 38's domain
- Indicator-based engine integration (SMS-EMOA/IBEA wiring) — Phase 38
- `GaObserver` hooks or observer integration
- `AllObserver<U>` updates
- WASM-specific examples
</user_constraints>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Hypervolume computation | Utility module | — | Pure mathematical computation, no engine or observer dependency |
| GD computation | Utility module | — | Pure distance-based calculation, no side effects |
| IGD computation | Utility module | — | Mirror of GD computation (symmetrical), same tier |
| Spread computation | Utility module | — | Distribution metric, pure function |
| Error handling | Error module | Utility module | `GaError` already owns all error types; new variant lives in `src/error.rs` |
| Pareto front extraction | Engine layer (user code) | — | User extracts `Vec<Vec<f64>>` from `ParetoFront<U>` before calling indicators |

All indicators are stateless utility functions with no engine or lifecycle dependencies. They sit in the utility/layer alongside `non_dominated_sort.rs` and `pareto.rs`.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust std | 1.81+ | All computation | Pure math — no external deps needed |
| genetic_algorithms::GaError | 2.4.0 | Error return type | Single error enum for the entire crate |

### Dependencies: NONE
No new crates are needed. Hypervolume, GD, IGD, and Spread are implemented entirely with `std::f64` operations (`f64::sqrt`, `f64::powf`, `f64::abs`, sorting).

**Installation:** None. No Cargo.toml changes.

**Version verification:** No new packages to verify. The existing `genetic_algorithms` crate and its dependencies (`rand`, `rayon`, `log`, etc.) are unchanged.

## Architecture Patterns

### System Architecture Diagram

```
User code / Phase 38 engines
        |
        | calls indicator functions with &[Vec<f64>]
        v
+---------------------------------------+
| multi_objective::indicators module    |
|                                       |
|  hypervolume()    generational_       |
|  (2D Lebesgue)    distance()          |
|                   (Euclidean p-norm)  |
|                                       |
|  inverted_generational_    spread()   |
|  distance()               (Deb 2002)  |
|  (Euclidean p-norm)                   |
|                                       |
|  Internal helpers:                    |
|  - validate_non_empty()               |
|  - validate_dimensions()              |
|  - nearest_distance()                 |
+---------------------------------------+
        |
        | returns Result<f64, GaError>
        v
User code / Phase 38 engines
```

Data flow for all indicators:
1. User extracts `Vec<Vec<f64>>` from `ParetoFront<U>.individuals[i].objectives`
2. User calls `indicators::hypervolume(&points, &reference_point)` (or GD/IGD/Spread)
3. Function validates inputs (empty sets, dimension consistency)
4. Function computes algorithm via pure math
5. Returns `Ok(f64)` or `Err(GaError::InvalidIndicatorConfiguration(...))`

### Recommended Project Structure

```
src/engines/multi_objective/
├── mod.rs                      # + pub mod indicators;
├── pareto.rs                   # unchanged
├── non_dominated_sort.rs       # unchanged
└── indicators/
    ├── mod.rs                  # Re-exports: pub use hypervolume::*, etc.
    ├── hypervolume.rs          # 2D exact Lebesgue measure
    ├── generational_distance.rs # GD with configurable power
    ├── inverted_generational_distance.rs # IGD with configurable power
    └── spread.rs               # Deb 2002 spread metric

tests/engines/multi_objective/
└── indicators/                 # NEW directory
    ├── test_hypervolume.rs     # Inline ZDT/DTLZ reference fronts
    ├── test_generational_distance.rs
    ├── test_inverted_generational_distance.rs
    └── test_spread.rs
```

### Module Entry Point Pattern (mod.rs)

The mod.rs re-exports all public functions using `pub use` for individual functions, following the same pattern as `src/engines/multi_objective/mod.rs`.

### Pattern 1: Pure Function API
**What:** All indicators are stateless `fn` items. No structs, no trait implementations, no builder pattern.
**When to use:** Always — this is the locked decision D-02.

### Pattern 2: Error Handling with GaError
**What:** All functions return `Result<f64, GaError>`. Only two error types: `InvalidIndicatorConfiguration(String)` for validation failures, and `Ok(f64)` for successful computation.
**When to use:** Every indicator function follows this contract.

### Anti-Patterns to Avoid
- **Struct-based indicator wrappers:** CONTEXT.md locks D-02 against this. No `HypervolumeCalculator` struct, no `IndicatorConfig`. Phase 38 adds that layer if needed.
- **Pre-computation caching:** No lazy sorting, no memoized distance matrices. Cold call every time.
- **Mutation of input data:** Input points are `&[Vec<f64>]`. Never sort in place on the reference.
- **Returning bare f64:** Always wrap in `Result`. Invalid inputs (empty sets, dimension mismatches) return errors, not NaN or infinity.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Euclidean distance | Hand-rolled loop | `f64::powf(_, 2.0).sum().sqrt()` | Rust std handles this — no distance crate needed |
| Sorting by first objective | Custom sort | `points.sort_by_key()` | std sort is O(n log n) and stable |

**Key insight:** All four indicators are simple enough to implement directly with std operations. No math or optimization crate (ndarray, nalgebra, etc.) is justified. The indicators are O(n*m) for GD/IGD (pairwise distance computations) and O(n log n) for HV/Spread (sorting). No algorithmic complexity that benefits from external crates.

## Common Pitfalls

### Pitfall 1: Dimension Mismatch Between Point Sets
**What goes wrong:** GD or IGD receives `approx_front` with 2 objectives but `true_front` with 3 objectives. The distance computation silently produces wrong results.
**Why it happens:** Plain `&[Vec<f64>]` inputs have no compile-time dimension guarantee.
**How to avoid:** Validate dimension consistency at the start of each function. If inner `Vec<f64>` lengths differ between or within sets, return `GaError::InvalidIndicatorConfiguration`.
**Warning signs:** Tests with mismatched dimensions should return error, not compute.

### Pitfall 2: Hypervolume Reference Point Not Dominated
**What goes wrong:** User passes `reference_point = [0.0, 0.0]` for a minimization problem where all points have values in [1.0, 2.0]. The computed hypervolume is 0 or negative.
**Why it happens:** The Lebesgue measure formula requires reference point to be strictly greater (worse) than all points in every objective.
**How to avoid:** Require `reference_point[i] > max_point[i]` for all i (minimization) or `reference_point[i] < min_point[i]` (maximization). Since indicator functions don't know objective directions, default to minimization (reference point must dominate all points: all components strictly greater).
**Warning signs:** Hypervolume of 0.0 or negative is always wrong.

### Pitfall 3: GD Root vs. Power Confusion
**What goes wrong:** The GD formula has both a per-point power (distance^p) and an outer root (^(1/p)). Mixing these up produces wrong results.
**Why it happens:** The standard formula `GD = (1/|P| * sum min_dist^p)^{1/p}` uses the power both inside and outside the sum.
**How to avoid:** Compute sum of `min_dist.powf(power)`, divide by len, then call `.powf(1.0 / power)`. Document this clearly. When power=2.0, p=2 means Euclidean squared then sqrt.

### Pitfall 4: Spread Formula Edge Cases
**What goes wrong:** The Deb 2002 spread formula has edge cases when all distances are equal (division by near-zero denominator) or when the front has only 1-2 points.
**Why it happens:** `d_bar` is the mean of consecutive distances. With 1 point there are 0 consecutive distances, causing division by zero.
**How to avoid:** Validate front length >= 2 for spread. For the edge case where df+dl = 0 and all d_i are equal (producing delta = 0/0), check if df+dl+(n-1)*d_bar == 0 and return 0.0 (perfect spread).

### Pitfall 5: IGD/GD Reverse Semantics
**What goes wrong:** User swaps approx_front and true_front arguments, getting GD when they expected IGD.
**Why it happens:** GD and IGD have the same formula but different argument order. GD averages distances FROM approx TO true. IGD averages FROM true TO approx.
**How to avoid:** Name the parameters clearly (`approx_front`, `true_front`). Document the semantics. Tests verify that GD(approx, true) != IGD(approx, true) for unequal-sized sets.

## Code Examples

### 2D Hypervolume (sort-then-sweep Lebesgue measure)

```rust
// Source: [ASSUMED — well-established algorithm, Zitzler & Thiele 1999]
pub fn hypervolume(points: &[Vec<f64>], reference_point: &[f64]) -> Result<f64, GaError> {
    validate_non_empty("points", points)?;
    validate_dimension_consistency(points)?;
    validate_dimension("reference_point", reference_point, points[0].len())?;

    if points[0].len() != 2 {
        return Err(GaError::InvalidIndicatorConfiguration(
            "Hypervolume requires exactly 2 objectives".to_string(),
        ));
    }

    // Validate reference point is strictly dominated by all points
    for point in points {
        for (p, r) in point.iter().zip(reference_point.iter()) {
            if p >= r {
                return Err(GaError::InvalidIndicatorConfiguration(
                    "Reference point must be strictly dominated by all points".to_string(),
                ));
            }
        }
    }

    // Sort by first objective ascending
    let mut sorted: Vec<&[f64]> = points.iter().map(|v| v.as_slice()).collect();
    sorted.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal));

    let mut hv = 0.0;
    let mut prev_f1 = sorted[0][0];

    for point in &sorted {
        let f1 = point[0];
        let f2 = point[1];
        let width = if f1 > prev_f1 { f1 - prev_f1 } else { 0.0 };
        let height = reference_point[1] - f2;
        hv += width * height;
        prev_f1 = f1;
    }

    Ok(hv)
}
```

### Generational Distance (GD)

```rust
// Source: [ASSUMED — well-established, Van Veldhuizen & Lamont 2000]
const DEFAULT_POWER: f64 = 2.0;

fn squared_euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum()
}

fn nearest_distance(point: &[f64], front: &[Vec<f64>], power: f64) -> f64 {
    front
        .iter()
        .map(|ref_point| squared_euclidean_distance(point, ref_point))
        .fold(f64::INFINITY, f64::min)
        .powf(power / 2.0) // sqrt(squared_dist)^power = squared_dist^(power/2)
}

pub fn generational_distance(
    approx_front: &[Vec<f64>],
    true_front: &[Vec<f64>],
    power: f64,
) -> Result<f64, GaError> {
    validate_non_empty("approx_front", approx_front)?;
    validate_non_empty("true_front", true_front)?;
    validate_dimension_consistency(approx_front)?;
    validate_dimension_consistency(true_front)?;

    if approx_front[0].len() != true_front[0].len() {
        return Err(GaError::InvalidIndicatorConfiguration(
            "Dimension mismatch between approx_front and true_front".to_string(),
        ));
    }

    if power <= 0.0 {
        return Err(GaError::InvalidIndicatorConfiguration(
            "Power must be positive".to_string(),
        ));
    }

    let sum: f64 = approx_front
        .iter()
        .map(|point| nearest_distance(point, true_front, power))
        .sum();

    let mean = sum / approx_front.len() as f64;
    Ok(mean.powf(1.0 / power))
}
```

### Inverted Generational Distance (IGD)

```rust
// Source: [ASSUMED — well-established, Coello Coello & Cruz Cortes 2005]
pub fn inverted_generational_distance(
    approx_front: &[Vec<f64>],
    true_front: &[Vec<f64>],
    power: f64,
) -> Result<f64, GaError> {
    // Same validation as GD...
    // Same formula, but summing nearest distances FROM true_front TO approx_front
    let sum: f64 = true_front
        .iter()
        .map(|point| nearest_distance(point, approx_front, power))
        .sum();

    let mean = sum / true_front.len() as f64;
    Ok(mean.powf(1.0 / power))
}
```

### Spread (Deb et al. 2002)

```rust
// Source: [ASSUMED — Deb et al. 2002, A Fast and Elitist Multiobjective GA: NSGA-II]
pub fn spread(
    approx_front: &[Vec<f64>],
    extreme_points: &[Vec<f64>],
) -> Result<f64, GaError> {
    validate_non_empty("approx_front", approx_front)?;
    validate_non_empty("extreme_points", extreme_points)?;
    validate_dimension_consistency(approx_front)?;

    if approx_front.len() < 2 {
        return Err(GaError::InvalidIndicatorConfiguration(
            "Spread requires at least 2 points in approx_front".to_string(),
        ));
    }

    // Sort by first objective
    let mut sorted: Vec<&[f64]> = approx_front.iter().map(|v| v.as_slice()).collect();
    sorted.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    let mut d = Vec::with_capacity(n - 1);

    // Consecutive Euclidean distances
    for i in 0..(n - 1) {
        d.push(squared_euclidean_distance(sorted[i], sorted[i+1]).sqrt());
    }

    let d_bar: f64 = d.iter().sum::<f64>() / d.len() as f64;

    // df and dl: distances from extreme points to closest front endpoints
    let df = extreme_points.iter()
        .map(|ep| squared_euclidean_distance(ep, sorted[0]).sqrt())
        .fold(f64::INFINITY, f64::min); // extreme_point[0] (first objective extreme)

    // Extreme point for last objective
    // In 2D, sorted by f1 means endpoints are first and last
    let dl = extreme_points.iter()
        .map(|ep| squared_euclidean_distance(ep, sorted[n-1]).sqrt())
        .fold(f64::INFINITY, f64::min);

    let numerator = df + dl + d.iter().map(|di| (di - d_bar).abs()).sum::<f64>();
    let denominator = df + dl + (n as f64 - 1.0) * d_bar;

    if denominator == 0.0 {
        return Ok(0.0); // Perfect spread
    }

    Ok(numerator / denominator)
}
```

### Validation Helpers (factored out)

```rust
fn validate_non_empty(name: &str, points: &[Vec<f64>]) -> Result<(), GaError> {
    if points.is_empty() {
        return Err(GaError::InvalidIndicatorConfiguration(
            format!("{} must not be empty", name),
        ));
    }
    Ok(())
}

fn validate_dimension_consistency(points: &[Vec<f64>]) -> Result<(), GaError> {
    if points.is_empty() {
        return Ok(());
    }
    let dim = points[0].len();
    if dim == 0 {
        return Err(GaError::InvalidIndicatorConfiguration(
            "Points must have at least 1 dimension".to_string(),
        ));
    }
    for point in points.iter().skip(1) {
        if point.len() != dim {
            return Err(GaError::InvalidIndicatorConfiguration(
                format!("Dimension mismatch: expected {} dimensions, got {}", dim, point.len()),
            ));
        }
    }
    Ok(())
}
```

### Inline Test Data: ZDT1 Reference Front

```rust
// Source: [ASSUMED — analytically known ZDT1 Pareto front]
fn zdt1_reference_front(n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| {
            let f1 = i as f64 / (n - 1) as f64;
            let f2 = 1.0 - f1.sqrt();
            vec![f1, f2]
        })
        .collect()
}

// Hypervolume for ZDT1 front with reference_point = [1.0, 1.0]
// Expected: 0.666... (approximately)
// (The exact value depends on resolution n; with n=1000, it's close to 2/3)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| None (new module) | Shared `multi_objective/indicators/` directory | Phase 39 | All MOO engines can import indicators without coupling to nsga2 |

No deprecated patterns to track — this is a new module.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `GaError::InvalidIndicatorConfiguration(String)` follows existing naming convention | Error Handling | If convention differs, rename variant before merge |
| A2 | The `search` (spread) function uses the standard Deb 2002 delta formula with extreme point distances | Code Examples | Non-standard spread definition would need different computation |
| A3 | ZDT1 Pareto front is f2 = 1 - sqrt(f1) for f1 in [0, 1] | Test Data | Wrong front would cause test assertions with wrong expected values |
| A4 | GD power parameter semantics: `(1/n * sum min_dist^p)^{1/p}` | Code Examples | Some formulations use `(1/n * sum min_dist^p)^{1/p}` consistently; others omit the outer root. Verify against cited references. |

## Open Questions (RESOLVED)

1. **[Spread extreme-point semantics] [RESOLVED]**
   - What we know: Spread takes `extreme_points: &[Vec<f64>]` — the user provides the ideal point / nadir extreme for each objective.
   - What's unclear: Whether the standard Deb 2002 definition uses the extreme of the APPROXIMATE front or the TRUE front. The parameter name `extreme_points` implies user supplies them (consistent with D-04).
   - Recommendation: [RESOLVED] Proceed with `extreme_points` parameter as specified in D-02. The function computes df and dl as distances from extreme_points[0] to the first front member and extreme_points[1] (or the last objective extreme) to the last front member.

2. **[Power semantics edge case — power=1 vs power=2] [RESOLVED]**
   - What we know: power=2.0 is the default (Euclidean).
   - What's unclear: When power=1.0, `nearest_distance` computes `sqrt(squared_dist)^1 = Manhattan distance`, and the outer root is ^1.0 (no-op). When power goes to infinity, the indicator approaches the maximum distance (Chebyshev/limit).
   - Recommendation: [RESOLVED] Document that `power=2.0` gives standard Euclidean GD/IGD. `power=1.0` gives Manhattan. Very large powers approximate the Hausdorff metric.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | Build, test, lint | yes | 1.81+ (MSRV) | — |

**No external tools needed.** New Rust files compile with existing `cargo build` / `cargo test` / `cargo clippy`. No new crates, no new binaries, no services.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` |
| Config file | None — `Cargo.toml` has no test-specific config (no harness = false needed) |
| Quick run command | `cargo test -p genetic_algorithms --test '*' indicators 2>&1 | head -30` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MOO-05 | hypervolume returns correct value for known 2D front | integration | `cargo test test_hypervolume_basic_zdt1 -- --exact` | Wave 0 |
| MOO-05 | hypervolume returns error for 3D points | integration | `cargo test test_hypervolume_rejects_3d -- --exact` | Wave 0 |
| MOO-05 | hypervolume returns error for empty points | integration | `cargo test test_hypervolume_empty -- --exact` | Wave 0 |
| MOO-05 | GD returns correct distance for identical fronts | integration | `cargo test test_gd_identical_fronts -- --exact` | Wave 0 |
| MOO-05 | GD returns positive distance for shifted fronts | integration | `cargo test test_gd_shifted -- --exact` | Wave 0 |
| MOO-05 | GD returns error for dimension mismatch | integration | `cargo test test_gd_dimension_mismatch -- --exact` | Wave 0 |
| MOO-05 | IGD returns positive distance | integration | `cargo test test_igd -- --exact` | Wave 0 |
| MOO-05 | IGD > GD for unequal front sizes (coverage) | integration | `cargo test test_igd_gt_gd_sparse -- --exact` | Wave 0 |
| MOO-05 | Spread returns 0.0 for uniformly spaced points | integration | `cargo test test_spread_perfect -- --exact` | Wave 0 |
| MOO-05 | Spread returns >0 for non-uniform points | integration | `cargo test test_spread_nonuniform -- --exact` | Wave 0 |
| MOO-05 | Spread returns error for single point | integration | `cargo test test_spread_single_point -- --exact` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test` (all tests, including new indicator tests)
- **Per wave merge:** `cargo test` + `cargo test --features serde` + `cargo clippy`
- **Phase gate:** Full suite green + `cargo doc --no-deps --no-deps` (zero rustdoc warnings) + `cargo check --target wasm32-unknown-unknown`

### Wave 0 Gaps
- [ ] `tests/engines/multi_objective/indicators/mod.rs` — test module entry (or rely on Cargo test discovery of individual files in `tests/`) 
- [ ] `tests/engines/multi_objective/indicators/test_hypervolume.rs` — covers MOO-05 hypervolume cases
- [ ] `tests/engines/multi_objective/indicators/test_generational_distance.rs` — covers MOO-05 GD cases
- [ ] `tests/engines/multi_objective/indicators/test_inverted_generational_distance.rs` — covers MOO-05 IGD cases
- [ ] `tests/engines/multi_objective/indicators/test_spread.rs` — covers MOO-05 Spread cases

## Security Domain

Skip — `security_enforcement` is not part of this project's workflow configuration. All computations are pure math with no I/O, no network, no file access, no user-controlled input beyond `Vec<f64>` data. No ASVS categories apply.

## Sources

### Primary (HIGH confidence)
- [CONTEXT.md] — Phase 39 locked decisions D-01 through D-05, API signatures, scope boundaries
- [CLAUDE.md] — Project conventions: WASM compatibility, code style, test placement, GaError pattern

### Secondary (MEDIUM confidence)
- [Codebase: `src/engines/multi_objective/mod.rs`] — Confirmed module layout, `pub mod` pattern, `ObjectiveDirection` enum
- [Codebase: `src/error.rs`] — Confirmed `GaError` naming conventions: `InvalidXxxConfiguration(String)` pattern
- [Codebase: `src/engines/multi_objective/pareto.rs`] — Confirmed `ParetoIndividual{objectives: Vec<f64>}` data structure users will extract from
- [Codebase: `src/lib.rs`] — Confirmed `pub mod multi_objective;` re-export path `crate::multi_objective`
- [Codebase: `tests/engines/nsga2/test_pareto.rs`] — Confirmed test pattern: imports using crate root, `#[test]` functions

### Tertiary (LOW confidence) — Algorithm references (well-established, no verification needed beyond training knowledge)
- Hypervolume: Zitzler & Thiele 1999, "Multiobjective evolutionary algorithms: A comparative case study and the strength Pareto approach"
- GD: Van Veldhuizen & Lamont 2000, "On measuring multiobjective evolutionary algorithm performance"
- IGD: Coello Coello & Cruz Cortes 2005, "Solving multiobjective optimization problems using an artificial immune system"
- Spread: Deb et al. 2002, "A fast and elitist multiobjective genetic algorithm: NSGA-II"

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — No new dependencies needed. Pure Rust std + GaError.
- Architecture: HIGH — Module layout, file structure, and API signed off in CONTEXT.md.
- Pitfalls: MEDIUM — Algorithm edge cases (spread division by zero, power semantics) are standard but need implementation care.

**Research date:** 2026-05-10
**Valid until:** No time sensitivity — algorithms are well-established and don't change.
