# Bench Harness: divan

**Created:** Phase 69-01 (2026-06-16)
**Action:** Wave M3 Action #7 — reduce dev-build dep weight and bench compile time

---

## Why divan

criterion pulled in ~20 transitive dependencies (plotters, ciborium, half, rayon fork, etc.)
that bloated dev-dependency compile times by 30-40 s on cold builds. divan 0.1.21 has
approximately 3 transitive dependencies and compiles in ~3 s. Both harnesses required
`harness = false` in Cargo.toml — that setting did not change.

Decision locked in D-09 of 69-CONTEXT.md: both harnesses coexist during porting;
criterion removed in the same plan once all 13 files are ported and CI is green.

---

## Canonical Patterns

### 1. Simple bench (no setup cost)

```rust
#[divan::bench]
fn my_bench(bencher: divan::Bencher) {
    bencher.bench(|| {
        // timed work here
    });
}
```

Replaces `c.bench_function("name", |b| b.iter(|| ...))`.

### 2. Bench with fresh owned input per iteration (replaces iter_batched)

```rust
#[divan::bench]
fn my_bench(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| build_expensive_setup())
        .bench_values(|mut owned| {
            let _ = owned.run();
        });
}
```

Note: the method is `bench_values` (not `bench`) when `with_inputs` is used.
The setup closure is called once per iteration; timing excludes setup time.

### 3. Parameterised bench (replaces BenchmarkId loop)

```rust
#[divan::bench(args = [10usize, 100, 1000])]
fn my_bench(bencher: divan::Bencher, n: usize) {
    bencher.bench(|| work(n));
}
```

For tuple args (multi-dimension combinations):
```rust
#[divan::bench(args = [(10usize, 2usize), (100, 5), (1000, 10)])]
fn my_bench(bencher: divan::Bencher, (pop, obj): (usize, usize)) {
    bencher.bench(|| work(pop, obj));
}
```

IMPORTANT: args elements must implement `Display`. Tuples of `(&str, EnumVariant)` do NOT
implement `Display` — use a module-level grouping with one fn per variant instead.

### 4. Controlled sample count (replaces group.sample_size(N))

```rust
#[divan::bench(sample_count = 10)]
fn slow_bench(bencher: divan::Bencher) {
    bencher.bench(|| slow_work());
}
```

### 5. Module-level grouping (replaces benchmark_group)

```rust
mod my_group {
    use super::*;

    #[divan::bench]
    fn variant_a(bencher: divan::Bencher) { ... }

    #[divan::bench]
    fn variant_b(bencher: divan::Bencher) { ... }
}
```

### 6. Entry point (required — replaces criterion_main!)

```rust
fn main() {
    divan::main();
}
```

---

## Dropped Criterion Features (loss of display only)

| criterion API | divan equivalent | Notes |
|---|---|---|
| `group.throughput(Throughput::Elements(n))` | none | No elements/sec display — correctness unaffected |
| `group.plot_config(PlotConfiguration::...)` | none | No logarithmic plot — display only |
| `BenchmarkId::new(name, param)` | `args = [...]` | Same parameterisation, different label format |
| `BatchSize::SmallInput` | implicit in `with_inputs` | divan always calls setup once per iteration |

---

## Do Not Reintroduce

- **No `criterion` in `benches/` or `Cargo.toml`** — zero tolerance. Verified by
  `grep -rn criterion benches/ Cargo.toml` returning zero matches (enforced in CI).
- **No `criterion_group!` / `criterion_main!` macros** in any bench file.
- **No `group.finish()` calls** — divan has no group object; grouping is via modules.
- **No `b.iter()` / `b.iter_batched()` closures** — divan uses `bencher.bench()` /
  `bencher.with_inputs().bench_values()`.

---

## How to Verify

```bash
# All 13 benches compile
cargo bench --no-run --all-features

# Feature-isolated benches
cargo bench --bench de --features benchmarks --no-run
cargo bench --bench metrics_observer --features observer-metrics --no-run

# No criterion references anywhere
grep -rn criterion benches/ Cargo.toml  # must return zero matches

# Lib is WASM-safe
cargo check --target wasm32-unknown-unknown --lib
```

---

## Feature-Isolated Benches

Two bench files require non-default features and are compiled separately in CI:

| Bench | Feature flag | Reason |
|---|---|---|
| `benches/de.rs` | `--features benchmarks` | Uses DE engine behind benchmarks feature |
| `benches/metrics_observer.rs` | `--features observer-metrics` | Uses metrics crate |

Both have `required-features` in their `[[bench]]` Cargo.toml entry AND `harness = false`.
Neither entry changed during the criterion → divan port (D-08: feature isolation preserved).

---

## References

- Decision D-07: Light cleanup allowed during port (dead bench cases may be removed; median stays within ±3%)
- Decision D-08: Feature-isolated benches (de, metrics_observer) stay separate
- Decision D-09: Both harnesses coexisted during port; criterion removed in same plan
- Phase 69 Action #7 in `.planning/v3.0.0-BUILD-PERF.md`
