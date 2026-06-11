# Phase 34: WASM support — fix time-based panics for wasm32-unknown-unknown - Pattern Map

**Mapped:** 2026-05-07
**Files analyzed:** 7
**Analogs found:** 7 / 7

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/engines/ga.rs` | engine/orchestrator | event-driven (generation loop) | `src/engines/ga.rs` itself (existing cfg pattern at lines 59-66) | self-analog — cfg(feature) pattern already present |
| `src/engines/nsga2/mod.rs` | engine/orchestrator | event-driven (generation loop) | `src/engines/ga.rs` lines 764-807 | exact — same observer-gated Instant pattern |
| `src/engines/de/engine.rs` | engine | event-driven | `src/engines/ga.rs` (rayon par_iter pattern) | role-match — no rayon currently; CONTEXT confirms no rayon in DE |
| `src/engines/cellular/engine.rs` | engine | event-driven | `src/engines/ga.rs` (rayon par_iter pattern) | role-match — no rayon currently; CONTEXT confirms no rayon in Cellular |
| `src/engines/alps/engine.rs` | engine | event-driven | `src/engines/ga.rs` (rayon par_iter pattern) | role-match — no rayon currently; CONTEXT confirms no rayon in ALPS |
| `src/engines/scatter/engine.rs` | engine | event-driven | `src/engines/ga.rs` (rayon par_iter pattern) | role-match — no rayon currently; CONTEXT confirms no rayon in Scatter |
| `src/observe/reporter/duration.rs` | reporter/observer | request-response | `src/engines/ga.rs` lines 764-776 | role-match — same Instant::now() wrapping need |

---

## Investigation Finding: Rayon Scope is Smaller Than CONTEXT.md Suggests

After reading all six engine files, **only two engines actually use rayon**: `src/engines/ga.rs` and `src/engines/nsga2/mod.rs`. The engines listed in CONTEXT.md as having rayon (DE, Cellular, ALPS, Scatter) do **not** import or use rayon in their `engine.rs` files. Their imports begin with `use std::*`, `use crate::*`, and `use rand::*` only.

Rayon sites confirmed:
- `src/engines/ga.rs` line 46: `use rayon::prelude::*;` — used at lines 1030 and 1327
- `src/engines/nsga2/mod.rs` line 47: `use rayon::prelude::*;` — used at lines 379 and 476

The planner should treat DE, Cellular, ALPS, and Scatter as **no rayon change needed**.

---

## Pattern Assignments

### `src/engines/ga.rs` — Instant sites (4 sites)

**Analog:** The existing `#[cfg(feature = "serde")]` / `#[cfg(not(feature = "serde"))]` block pair already in this file (lines 59–66 and line 1076).

**Existing cfg pair pattern** (lines 59–66) — copy this structure:
```rust
#[cfg(feature = "serde")]
pub trait MaybeSerialize: serde::Serialize {}
#[cfg(feature = "serde")]
impl<T: serde::Serialize> MaybeSerialize for T {}

#[cfg(not(feature = "serde"))]
pub trait MaybeSerialize {}
#[cfg(not(feature = "serde"))]
impl<T> MaybeSerialize for T {}
```

**Existing inline cfg block** (line 1076) — copy this structure for inline cfg in function body:
```rust
#[cfg(feature = "serde")]
{
    // ... body that only runs when feature is enabled
}
```

**Site 1 — start_time** (line 750): `let start_time = Instant::now();`
Apply pattern:
```rust
#[cfg(not(target_arch = "wasm32"))]
let start_time = Instant::now();
```

**Sites 2, 3, 4 — observer-gated timing** (lines 765–768, 784–788, 824–828):
```rust
// CURRENT (e.g. line 765-768):
let t_sel = if self.observer.is_some() {
    Some(Instant::now())
} else {
    None
};

// AFTER:
let t_sel = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};
```

**max_duration_secs check** (lines 1191–1203) — wrap the entire time-limit block:
```rust
// CURRENT:
if let Some(max_secs) = self.configuration.stopping_criteria.max_duration_secs {
    if start_time.elapsed().as_secs_f64() >= max_secs {
        ...
        break;
    }
}

// AFTER:
#[cfg(not(target_arch = "wasm32"))]
if let Some(max_secs) = self.configuration.stopping_criteria.max_duration_secs {
    if start_time.elapsed().as_secs_f64() >= max_secs {
        ...
        break;
    }
}
```

**max_duration_secs WASM warning** — emit once at engine start (near line 750), following the `log::warn!` target convention from throughout the codebase:
```rust
#[cfg(target_arch = "wasm32")]
if self.configuration.stopping_criteria.max_duration_secs.is_some() {
    log::warn!(target: "ga_events",
        "max_duration_secs is not supported on wasm32 — time limit will be ignored");
}
```

**`use std::time::Instant` import** (line 50): Gate to avoid dead-code warning on wasm32:
```rust
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
```

---

### `src/engines/ga.rs` — rayon sites (2 sites)

**Site 1 — into_par_iter** (lines 1029–1047):
```rust
// CURRENT:
let new_chromosomes: Vec<U> = (0..deficit)
    .into_par_iter()
    .map(|_| { ... })
    .collect();

// AFTER:
#[cfg(not(target_arch = "wasm32"))]
let new_chromosomes: Vec<U> = (0..deficit)
    .into_par_iter()
    .map(|_| { ... })
    .collect();
#[cfg(target_arch = "wasm32")]
let new_chromosomes: Vec<U> = (0..deficit)
    .map(|_| { ... })
    .collect();
```

**Site 2 — par_iter in parent_crossover** (lines 1326–1340+):
```rust
// CURRENT:
let results: Vec<Result<Vec<U>, GaError>> = parents
    .par_iter()
    .map(|(key, value)| { ... })
    .collect();

// AFTER:
#[cfg(not(target_arch = "wasm32"))]
let results: Vec<Result<Vec<U>, GaError>> = parents
    .par_iter()
    .map(|(key, value)| { ... })
    .collect();
#[cfg(target_arch = "wasm32")]
let results: Vec<Result<Vec<U>, GaError>> = parents
    .iter()
    .map(|(key, value)| { ... })
    .collect();
```

**`use rayon::prelude::*` import** (line 46): Gate to avoid linker failures on wasm32:
```rust
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
```

---

### `src/engines/nsga2/mod.rs` — Instant sites (2 sites)

**Sites** (lines 233–236, 253–256): Both follow the same observer-gated pattern as ga.rs.

```rust
// CURRENT (line 233-236):
let t_sort = if self.observer.is_some() {
    Some(Instant::now())
} else {
    None
};

// AFTER:
let t_sort = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};
```

Apply the identical transformation to `t_crowd` at lines 253–256.

**`use std::time::Instant` import** (line 49): Gate same as ga.rs:
```rust
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
```

---

### `src/engines/nsga2/mod.rs` — rayon sites (2 sites)

**Site 1 — initialize_population** (lines 378–387):
```rust
// CURRENT:
let population = chromosomes
    .into_par_iter()
    .map(|chrom| { ... })
    .collect();

// AFTER:
#[cfg(not(target_arch = "wasm32"))]
let population = chromosomes
    .into_par_iter()
    .map(|chrom| { ... })
    .collect();
#[cfg(target_arch = "wasm32")]
let population = chromosomes
    .into_iter()
    .map(|chrom| { ... })
    .collect();
```

**Site 2 — create_offspring** (lines 475–484): Same pattern, `into_par_iter()` → `into_iter()` under wasm32 cfg.

**`use rayon::prelude::*` import** (line 47): Gate same as ga.rs:
```rust
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
```

---

### `src/observe/reporter/duration.rs` — Instant site (1 site)

**Full file is 74 lines.** The struct holds `start: Option<Instant>` (line 36), set at line 55, consumed at line 59.

**Pattern:** Replace the `Instant` field and call site with cfg-gated versions; keep the `Duration` return (using `Duration::ZERO` fallback) unchanged since `Duration` is always available.

**Import** (line 6): `use std::time::{Duration, Instant};`
After:
```rust
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
```

**Struct field** (line 36): `start: Option<Instant>,`
After:
```rust
#[cfg(not(target_arch = "wasm32"))]
start: Option<std::time::Instant>,
#[cfg(target_arch = "wasm32")]
start: Option<()>,   // zero-size placeholder; always None
```

Alternative (simpler, avoids phantom field): Keep field as `Option<std::time::Instant>` behind cfg, but keep the struct itself always present — use `Duration::ZERO` unconditionally on wasm32.

Recommended simpler approach (no struct layout change required):

```rust
// on_start (line 54-56):
fn on_start(&mut self) {
    #[cfg(not(target_arch = "wasm32"))]
    { self.start = Some(std::time::Instant::now()); }
}

// on_finish elapsed line (line 59):
#[cfg(not(target_arch = "wasm32"))]
let elapsed = self.start.map(|s| s.elapsed()).unwrap_or(Duration::ZERO);
#[cfg(target_arch = "wasm32")]
let elapsed = Duration::ZERO;
```

This requires the struct field to remain `Option<std::time::Instant>` (always present) but gated behind cfg so it compiles on wasm32 by never being written or read there. Since `std::time::Instant` itself is not available on wasm32, the field must be type-erased or the whole field must be cfg-gated:

```rust
pub struct DurationReporter {
    #[cfg(not(target_arch = "wasm32"))]
    start: Option<std::time::Instant>,
}
```

Then constructor:
```rust
pub fn new() -> Self {
    Self {
        #[cfg(not(target_arch = "wasm32"))]
        start: None,
    }
}
```

---

## Shared Patterns

### cfg-gating pattern for imports (apply to all Instant/rayon imports)
**Source:** `src/engines/ga.rs` lines 59–66 and line 46/50
**Apply to:** `src/engines/ga.rs`, `src/engines/nsga2/mod.rs`, `src/observe/reporter/duration.rs`
```rust
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
```

### Observer-gated Instant pattern (identical in ga.rs and nsga2)
**Source:** `src/engines/ga.rs` lines 764–776
**Apply to:** All 4 observer-timing Instant sites (2 in ga.rs, 2 in nsga2)
```rust
// Guard: only create a timer if there is an observer AND we are not on wasm32
let t_sel = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};
```

### par_iter → iter fallback pattern
**Source:** `src/engines/ga.rs` line 1327 / `src/engines/nsga2/mod.rs` line 379
**Apply to:** All 4 rayon call sites (2 in ga.rs, 2 in nsga2)
```rust
#[cfg(not(target_arch = "wasm32"))]
let results = collection.par_iter().map(|x| { ... }).collect::<Vec<_>>();
#[cfg(target_arch = "wasm32")]
let results = collection.iter().map(|x| { ... }).collect::<Vec<_>>();

// For into_par_iter:
#[cfg(not(target_arch = "wasm32"))]
let results = collection.into_par_iter().map(|x| { ... }).collect::<Vec<_>>();
#[cfg(target_arch = "wasm32")]
let results = collection.into_iter().map(|x| { ... }).collect::<Vec<_>>();
```

### log::warn! target convention
**Source:** Throughout codebase (e.g., `src/engines/ga.rs` multiple sites)
**Apply to:** The max_duration_secs WASM warning in ga.rs
```rust
log::warn!(target: "ga_events", "max_duration_secs is not supported on wasm32 — time limit will be ignored");
```

---

## No Analog Found

None — all files to modify already exist in the codebase and have self-analogs or close analogs.

---

## Corrections to CONTEXT.md Scope

| CONTEXT claim | Reality found in source | Impact |
|---|---|---|
| DE engine uses rayon | `src/engines/de/engine.rs` — no rayon import or par_iter | No rayon change needed for DE |
| Cellular engine uses rayon | `src/engines/cellular/engine.rs` — no rayon import | No rayon change needed for Cellular |
| ALPS engine uses rayon | `src/engines/alps/engine.rs` — no rayon import | No rayon change needed for ALPS |
| Scatter engine uses rayon | `src/engines/scatter/engine.rs` — no rayon import | No rayon change needed for Scatter |

The rayon WASM fix is scoped to exactly 2 files: `src/engines/ga.rs` and `src/engines/nsga2/mod.rs`.

---

## Metadata

**Analog search scope:** `src/engines/`, `src/observe/reporter/`
**Files read:** ga.rs, nsga2/mod.rs, de/engine.rs, cellular/engine.rs, alps/engine.rs, scatter/engine.rs, reporter/duration.rs
**Pattern extraction date:** 2026-05-07
