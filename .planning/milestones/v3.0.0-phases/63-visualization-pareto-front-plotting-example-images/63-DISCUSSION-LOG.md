# Phase 63: Visualization — Pareto Front Plotting & Example Images - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-09
**Phase:** 63-visualization-pareto-front-plotting-example-images
**Areas discussed:** plot_pareto_front API shape, 3-objective rendering strategy, Which examples get --plot, README image links and docs layout

---

## plot_pareto_front API shape

| Option | Description | Selected |
|--------|-------------|----------|
| `&[(f64,f64)]` / `&[(f64,f64,f64)]` — two explicit functions | Caller extracts fitness coordinates; type-safe, no generic bounds, easy to test | ✓ |
| `&[U] where U: VectorFitness` | Generic over chromosome; more ergonomic, requires trait bound in viz module | |

**User's choice:** Two explicit functions: `plot_pareto_front_2d(&[(f64,f64)], path)` and `plot_pareto_front_3d(&[(f64,f64,f64)], path)`.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Two explicit functions (2d and 3d) | Type-safe, compile-time arity, consistent with existing plot_* functions | ✓ |
| Single function with enum/mode | `plot_pareto_front(&[Vec<f64>], path)` dispatches on `points[0].len()` | |

**User's choice:** Two separate functions.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Pareto points only — caller pre-filters | Function plots exactly what it receives; dominance filtering is caller's concern | ✓ |
| All points, Pareto highlighted in different color | Function accepts full population and runs dominance check internally | |

**User's choice:** Caller pre-filters; function is a pure plotter.

---

| Option | Description | Selected |
|--------|-------------|----------|
| No — stay focused on Pareto front + example images | ROADMAP scopes to Pareto front and fitness-progress charts | |
| Yes — add plot_true_fitness_calls | Phase 62 added true_fitness_calls; Phase 62 summary noted Phase 63 could plot it | ✓ |

**User's choice:** Add `plot_true_fitness_calls(stats: &[GenerationStats], path: &str)` to the module.

---

## 3-objective rendering strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Three 2D scatter subplots (f1×f2, f1×f3, f2×f3) | Three-panel 1200×400 image; fully readable, no projection math | ✓ |
| 2D projection (f1×f2 only, f3 ignored) | Simplest but loses information | |
| f1×f2 with f3 encoded as color gradient | Visually rich; requires color scale rendering | |

**User's choice:** Three-panel image, 1200×400.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Generic f1/f2/f3 labels — no extra argument | Simple signature; consistent with plot_fitness | ✓ |
| Optional axis labels as slice argument | `labels: Option<[&str;3]>` — flexible but rarely needed | |

**User's choice:** Generic labels, no additional argument.

---

## Which examples get --plot

| Option | Description | Selected |
|--------|-------------|----------|
| All 5 multi-objective examples | nsga2, spea2, sms_emoa, ibea (2-obj) + nsga3 (3-obj) | ✓ |
| nsga2_zdt1 only (as ROADMAP specifies) | Minimal — just what the success criterion names | |
| nsga2_zdt1 + nsga3_dtlz2 (one 2-obj + one 3-obj) | Representative without touching spea2/sms_emoa/ibea | |

**User's choice:** All 5 multi-objective examples.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Simple `std::env::args()` check — no clap | Consistent with existing examples; no new deps | ✓ |
| Add clap as dev-dependency | Proper CLI but adds dependency and complexity | |

**User's choice:** `std::env::args().any(|a| a == "--plot")`.

---

| Option | Description | Selected |
|--------|-------------|----------|
| `docs/images/[example_name].png` from cwd | Standard path; works with `cargo run` from repo root | ✓ |
| Output path as second CLI arg | Flexible but needs argument parsing | |

**User's choice:** Fixed path `docs/images/[example_name].png`.

---

## README image links and docs layout

| Option | Description | Selected |
|--------|-------------|----------|
| One representative image per algorithm (5 total) | All 5 multi-obj images + rastrigin.png | ✓ |
| One image only (nsga2_zdt1 as showpiece) | Minimal README | |
| Two images (one 2-obj, one 3-obj) | Balanced | |

**User's choice:** All 6 images (5 multi-obj + rastrigin).

---

| Option | Description | Selected |
|--------|-------------|----------|
| Under existing Visualization section | README already has ###Visualization; add sub-sections there | ✓ |
| Under a new Multi-Objective Examples section | New ## section | |
| Inline in algorithm descriptions | Embedded per-algorithm | |

**User's choice:** Sub-sections inside existing `### Visualization`.

---

| Option | Description | Selected |
|--------|-------------|----------|
| No — multi-objective Pareto plots only | Single-obj --plot is scope creep | |
| Yes — add --plot to one single-obj example | rastrigin.rs demonstrates plot_fitness | ✓ |

**User's choice:** Add `--plot` to `rastrigin.rs` as the single-objective representative.

| Option | Description | Selected |
|--------|-------------|----------|
| rastrigin.rs | Canonical benchmark, well-known | ✓ |
| onemax_binary.rs | Binary chromosome, maximization | |
| cma_es_rastrigin.rs | CMA-ES Phase 56 engine | |

**User's choice:** `rastrigin.rs`.

---

## Claude's Discretion

- Scatter plot marker style (circles vs dots, size) and grid line settings
- Whether `plot_true_fitness_calls` returns `InsufficientData` when all values are `None` or silently skips them
- Panel gap/margin in `plot_pareto_front_3d` three-panel layout
- Caption/alt text for README images

## Deferred Ideas

- `--plot` for CMA-ES / island model / other single-obj examples
- Optional axis label arguments for `plot_pareto_front_2d` / `_3d`
- f3-as-color-gradient alternative for 3-obj rendering
- True fitness calls chart for multi-objective observers (blocked until surrogate supports NSGA-II)
