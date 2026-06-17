# parallel Feature Gate — AI Intel

## Why this feature exists

The `parallel` feature decouples rayon from the binary output. Users targeting
`wasm32-unknown-unknown` (where rayon does not compile), embedded environments,
or ultra-lean builds can disable parallelism without losing the rest of the library.
It also makes the dependency cost explicit: enabling `parallel` pulls in `rayon`
and `crossbeam`; disabling it sheds them entirely.

## Canonical gate pattern (D-06)

Every rayon call-site in this library uses the **combined** cfg form. Two arms
are always emitted together:

```rust
// Parallel arm — enabled when NOT wasm32 AND the "parallel" feature is on
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;

// Sequential fallback — enabled when wasm32 OR "parallel" feature is off
#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
use std::iter; // or plain iterator
```

The single-condition `#[cfg(not(target_arch = "wasm32"))]` form (without
`feature = "parallel"`) is **forbidden** — it would make rayon unconditional
when `parallel` is off on non-wasm targets.

## What an agent must NOT reintroduce

- `use rayon::prelude::*;` without both cfg arms
- `rayon = "1.10"` (non-optional) in Cargo.toml
- `.par_iter()` / `.into_par_iter()` / `.par_sort_unstable_by()` without the
  combined gate pattern
- Any form of `#[cfg(not(target_arch = "wasm32"))]` on rayon imports that omits
  `feature = "parallel"`

## How to verify the invariant (CI enforcement)

The feature-matrix CI runs `cargo check --no-default-features --features logging`
(parallel disabled) on every PR. A local grep can surface violations:

```bash
# Must return 0 matches (any match = forbidden bare wasm32-only gate on rayon)
grep -rn 'use rayon' src/ \
  | grep -v 'all(not(target_arch = "wasm32"), feature = "parallel")'
```

Also run:
```bash
cargo check --no-default-features --features logging
cargo check --target wasm32-unknown-unknown
```

## Why the name is `parallel` and not `rayon`

Using `rayon` as the feature name would be a semver footgun: if the underlying
parallelism library is ever swapped for `crossbeam-deque` or `threadpool`, the
public feature name must not change. `parallel` describes the *capability* rather
than the *implementation*, keeping the public API stable through future dependency
changes.
