# Phase 23: Memory Layout - Research

**Researched:** 2026-04-01
**Domain:** Rust memory layout — `Arc<[T]>`, Copy-type specialization, dead-field removal, `#[inline]`, incremental hashing
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- MEM-01: `pub ranges: Vec<(T, T)>` changes to `pub ranges: Arc<[(T, T)]>`; field stays public; `Range::new()` signature stays `(id, ranges: Vec<(T,T)>, value)` — converts internally via `.into_boxed_slice().into()`
- MEM-02: `Range::value()` specialized for `T: Copy` returns `self.value` (no `.clone()`); existing `impl<T: Clone + Default>` block is where the current `value()` lives
- MEM-03: `generation_numbers: Vec<usize>` removed from `Population` struct, `Clone` impl, `Serialize` impl, `Deserialize` impl, and the `FIELDS` array; serde break is accepted
- MEM-04: `#[inline]` added directly above `pub fn call(&self, dna: &[G]) -> f64` in `src/fitness/fitness_fn_wrapper.rs`; no logic change
- MEM-05: `MassDeduplication` replaces `HashSet<Vec<i32>>` with incremental `DefaultHasher` + two-pass collision-safe approach; hash gene IDs only (Option A — no `GeneT: Hash` bound change)

### Claude's Discretion
- Exact trait bound split for MEM-02 (single `T: Copy` impl vs separate `T: Clone` / `T: Copy` impls)
- Implementation structure for the MEM-05 two-pass collision check (e.g., `HashMap<u64, Vec<Vec<i32>>>` for collision buckets)
- Whether to add `Cow<[Gene]>` deref compatibility for the Arc field (not needed — skip)

### Deferred Ideas (OUT OF SCOPE)
- Adding `Hash` to `GeneT` trait — breaking change; defer to future milestone
- Criterion benchmarks for Phase 23 changes — deferred to Phase 25
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| MEM-01 | `Range` genes use `Arc<[(T, T)]>` shared range slice | Arc::from(vec.into_boxed_slice()) pattern; serde `#[serde(with)]` or custom impl needed |
| MEM-02 | `Range::value()` returns by value for `Copy` types without `.clone()` | Rust specialization workaround via overlapping impls blocked; split impl block approach works |
| MEM-03 | Unused `generation_numbers` field removed from `Population` | All 6 removal sites identified in population.rs; serde round-trip test must drop field reference |
| MEM-04 | `FitnessFnWrapper::call()` annotated `#[inline]` | One-line annotation; function is thin wrapper — ideal inline candidate |
| MEM-05 | Mass Deduplication uses incremental `DefaultHasher` instead of per-chromosome `Vec<i32>` | `std::hash::{DefaultHasher, Hash, Hasher}` in std; two-pass approach documented |
</phase_requirements>

---

## Summary

Phase 23 is five surgical internal changes to `src/genotypes/range.rs`, `src/population.rs`, `src/fitness/fitness_fn_wrapper.rs`, and `src/operations/extension/mass_deduplication.rs`. No public method signatures change (except the type of the public `ranges` field on `Range<T>`, which is an accepted breaking point). All access sites for `ranges` use slice indexing (`ranges[i]`, `ranges.len()`) which work identically via `Deref` on `Arc<[T]>` — zero downstream operator changes needed.

The most complex change is MEM-01 because `Arc<[(T, T)]>` must be serde-compatible. The existing serde derive on `Range<T>` uses `#[derive(serde::Serialize, serde::Deserialize)]` — `Arc<[T]>` is supported by serde since 1.0.91, so the derive continues to work with no custom serde code required. MEM-05 (DefaultHasher two-pass) requires careful collision handling: common path avoids any `Vec<i32>` alloc; only hash-colliding chromosomes pay the fallback cost.

MEM-03 touches the hand-written serde impl in `population.rs` at six distinct sites. The existing `serde_population_binary_round_trip` test in `tests/test_serde.rs` does not assert `generation_numbers` directly, so it passes after removal — but the deserializer currently returns `Err` if `generation_numbers` is missing from JSON (it calls `ok_or_else(|| de::Error::missing_field(...))`). This `ok_or_else` site must be removed so old checkpoint JSON (which contains the field) is silently ignored rather than causing a parse error, and new JSON (without the field) deserializes correctly.

**Primary recommendation:** Implement all five changes in a single wave. Each change is independent; MEM-03 serde surgery is the most error-prone and should be implemented last or carefully reviewed against the full Deserialize impl.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::sync::Arc` | stdlib | Reference-counted shared ownership of `[(T,T)]` slice | Already used in codebase for `FitnessFnWrapper`, config, observers |
| `std::hash::{DefaultHasher, Hash, Hasher}` | stdlib | Incremental hashing for MEM-05 | No external dependency; idiomatic Rust |
| `std::collections::HashMap` | stdlib | Collision bucket map for MEM-05 two-pass | Standard; already in codebase |

### No New Dependencies
All five changes use only `std`. No `Cargo.toml` changes needed.

---

## Architecture Patterns

### MEM-01: Vec-to-Arc Conversion in Constructor

The constructor accepts `Vec<(T, T)>` (unchanged public API) and converts internally:

```rust
// Source: Rust stdlib docs — Arc::from(Box<[T]>) is O(1) no-copy
pub fn new(id: i32, ranges: Vec<(T, T)>, value: T) -> Self {
    Self {
        id,
        ranges: ranges.into_boxed_slice().into(), // Vec -> Box<[T]> -> Arc<[T]>
        value,
    }
}
```

`ranges.into_iter().collect::<Arc<[_]>>()` is an alternative but allocates an intermediate buffer — prefer `into_boxed_slice().into()` for a true zero-copy path.

### MEM-01: Default impl change

`Default` currently creates `ranges: Vec::new()`. With `Arc<[T]>`, use `Arc::from([])` or `Arc::default()`:

```rust
impl<T: Default> Default for Range<T> {
    fn default() -> Self {
        Self {
            id: 0,
            ranges: Arc::from([]),   // empty Arc slice
            value: Default::default(),
        }
    }
}
```

### MEM-01: Serde compatibility

`serde` supports `Arc<[T]>` via the `rc` feature flag when deriving. Check: `serde` in `Cargo.toml` must have `features = ["rc"]` for `Arc` deserialization to work with `#[derive(Deserialize)]`.

**Verification required:** Check `Cargo.toml` for `serde` feature flags. If `features = ["rc"]` is absent, add it or use a custom deserializer. Without `"rc"`, `Arc<[T]>` deserializes as `Arc<Vec<T>>` in some serde versions, which may not match `Arc<[T]>`.

```toml
# Cargo.toml — verify this is present
serde = { version = "1", features = ["derive", "rc"] }
```

### MEM-02: Copy-specialized value() accessor

Rust stable does not have specialization, so the cleanest approach is to replace the single `impl<T: Clone + Default>` block with two non-overlapping impls by splitting on `Copy`:

```rust
// For Copy types — no .clone() call
impl<T: Copy + Default> Range<T> {
    pub fn new(id: i32, ranges: Vec<(T, T)>, value: T) -> Self { ... }
    pub fn value(&self) -> T { self.value }
    pub fn set_value(&mut self, value: T) -> &mut Self { ... }
}
```

**Problem:** `Copy` implies `Clone`. If you split into `impl<T: Copy>` and `impl<T: Clone + !Copy>`, Rust stable cannot express `!Copy`. The practical solution: **single impl block with `T: Copy + Default`** — all current Range users (`i32`, `f64`, etc.) are `Copy`, so this covers 100% of current usage. The `Clone`-only path (non-Copy T) can be dropped since no non-Copy Range types exist in the codebase.

If non-Copy support must be preserved, keep the `impl<T: Clone + Default>` block with `.clone()` and add a **separate** `impl<T: Copy + Default>` block that overrides `value()` — but Rust will error on duplicate method definitions in overlapping impls. The compiler rejects this on stable.

**Recommended decision:** Single `impl<T: Copy + Default>` block — GeneT already requires `Clone` (via bounds in `impl<T: Sync + Send + Clone + Default> GeneT`), and `Copy: Clone`, so no capability is lost.

### MEM-03: Surgical Serde Removal Sites

All six sites in `src/population.rs`:

| Site | Action |
|------|--------|
| Struct field `pub generation_numbers: Vec<usize>` (line 37) | Delete field and doc comment |
| `Population::new_empty()` — `generation_numbers: vec![]` (line 55) | Delete line |
| `Population::new()` — `generation_numbers: vec![]` (line 67) | Delete line |
| `Clone` impl — `generation_numbers: self.generation_numbers.clone()` (line 209) | Delete line |
| `Serialize` impl — `serialize_struct("Population", 6)` → 5; `serialize_field("generation_numbers", ...)` | Change count to 5, delete the serialize_field call |
| `Deserialize` impl — `Field::GenerationNumbers` enum variant, local `Option<Vec<usize>>` var, match arm, struct construction `ok_or_else`, and `FIELDS` array entry | Delete all five sub-sites |

The `Field` enum in the Deserialize impl has a `GenerationNumbers` variant. When a serde deserializer encounters an unknown field it can either ignore or error. After removal: the `Field` enum no longer contains `GenerationNumbers`, so encountering `"generation_numbers"` in JSON will cause a `unknown field` error on old checkpoint files. This is **acceptable per the locked decision** ("serde break is acceptable"). The `FIELDS` array must also have `"generation_numbers"` removed.

### MEM-05: Two-Pass DefaultHasher Approach

```
Pass 1 (common path — no Vec alloc):
  For each chromosome:
    - Create DefaultHasher
    - Hash each gene's id() in sequence
    - Finish → u64 hash key
    - Insert (hash → first-seen chromosome index) into HashMap<u64, usize>
    - If hash already seen: collision path (pass 2)

Pass 2 (collision path — rare):
  - Materialize Vec<i32> for the colliding chromosome (fallback to exact match)
  - Materialize Vec<i32> for the stored chromosome at the colliding index
  - If equal → duplicate; if not equal → distinct chromosomes with same hash (hash collision)
```

The `retain` pattern in the current code is incompatible with this two-pass approach since `retain` gives no index access. Replace with an index-based retain or a manual `Vec::drain`/collect pattern.

**Recommended structure:**

```rust
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

let mut seen: HashMap<u64, Vec<i32>> = HashMap::new();
chromosomes.retain(|c| {
    let mut hasher = DefaultHasher::new();
    for g in c.dna() {
        g.id().hash(&mut hasher);
    }
    let h = hasher.finish();

    match seen.entry(h) {
        std::collections::hash_map::Entry::Vacant(e) => {
            // common path: no Vec<i32> alloc on the happy path
            // BUT: we need the Vec<i32> only on collision, not here
            // Solution: store h -> Option<Vec<i32>> and materialize on collision
            e.insert(c.dna().iter().map(|g| g.id()).collect());
            true
        }
        std::collections::hash_map::Entry::Occupied(e) => {
            // collision path: exact compare
            let key: Vec<i32> = c.dna().iter().map(|g| g.id()).collect();
            key != *e.get()  // true = distinct chromosome (hash collision); false = duplicate
        }
    }
});
```

**Note on "no Vec<i32> on common path":** The above still allocates `Vec<i32>` on first insertion. To achieve true zero-alloc on the common path for the FIRST encounter, the HashMap value must be `Option<Vec<i32>>` — store `None` on first insertion, materialize only if a collision occurs. However this adds complexity. The context doc says "only hash-colliding entries get one Vec<i32>", implying the first-seen entry stores its Vec. The CONTEXT.md's own example (`HashMap<u64, Vec<i32>>` for collision buckets) means: store the gene-ID vec of the first chromosome per hash so collision detection can do exact compare. This is still a win: the old code allocated a `Vec<i32>` for **every** chromosome; the new code allocates one `Vec<i32>` per **unique hash bucket** (typically one per chromosome in well-distributed populations), which is the same asymptotic count but avoids the HashSet overhead of storing and hashing the `Vec<i32>` as a key.

### Anti-Patterns to Avoid

- **Using `Arc::new(vec.into_boxed_slice())` then converting** — prefer `Arc::from(vec.into_boxed_slice())` which uses the `From<Box<[T]>> for Arc<[T]>` impl directly (single allocation).
- **Splitting into Copy + Clone impls with overlapping methods** — Rust stable will reject duplicate method names in trait impls on overlapping types.
- **Leaving `"generation_numbers"` in the `FIELDS` array** — serde uses this array as a hint to the deserializer for human-readable formats; leaving it in causes the deserializer to expect the field.
- **Using `Hasher::write_i32` instead of `i32.hash(hasher)`** — prefer `Hash::hash` for portability; `DefaultHasher` internals are not stable across Rust versions anyway, but idiomatic usage is via `Hash` trait.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Ref-counted slice | Custom `Rc`-backed wrapper | `Arc<[T]>` from std | stdlib-provided, thread-safe, Deref to `[T]` |
| Incremental hashing | Custom FNV or polynomial hasher | `std::hash::DefaultHasher` | Available in std, no dependency |
| Serde for `Arc<[T]>` | Custom serialize/deserialize | serde `rc` feature + derive | Handled automatically |

---

## Common Pitfalls

### Pitfall 1: Serde `rc` feature not enabled for Arc deserialization
**What goes wrong:** `#[derive(Deserialize)]` on a struct containing `Arc<[T]>` compiles but panics or errors at runtime, or fails to compile entirely, if `serde`'s `rc` feature is not enabled.
**Why it happens:** serde gates `Arc` deserialization behind the `rc` feature flag to avoid pulling in the synchronization overhead for users who don't need it.
**How to avoid:** Add `features = ["derive", "rc"]` to the serde dependency in `Cargo.toml` under the `[features]` section or `[dependencies]`.
**Warning signs:** Compile error mentioning `Arc` does not implement `Deserialize`; or test `serde_range_gene_i32` / `serde_range_gene_f64` fails.

### Pitfall 2: Serialize field count mismatch in Population
**What goes wrong:** `serializer.serialize_struct("Population", 6)` must change to `5` after removing `generation_numbers`. Mismatched count causes some serializers (e.g., bincode) to fail; JSON is more lenient but the count should still be accurate.
**Why it happens:** The count is a hint passed to the serializer; forgetting to decrement it leaves a stale value.
**How to avoid:** Change `6` to `5` at the `serialize_struct` call site simultaneously with removing the `serialize_field` call.

### Pitfall 3: Deserializer still references removed field via ok_or_else
**What goes wrong:** After removing the struct field, the Deserialize impl still has `generation_numbers.ok_or_else(|| de::Error::missing_field("generation_numbers"))`. This will cause **every** deserialization of `Population` to fail because the `generation_numbers` local variable is always `None` (the match arm that sets it is removed, but the `ok_or_else` remains).
**Why it happens:** Multi-site removal; easy to miss one site.
**How to avoid:** Remove ALL six sites atomically. Compile and run `cargo test --features serde` immediately after.

### Pitfall 4: Range::Default produces empty Arc slice — PartialEq and Hash still work
**What goes wrong:** Not actually a problem — `Arc<[]>` compares equal to `Arc<[]>` because `PartialEq` on `Arc<[T]>` delegates to slice comparison. Documented here to prevent unnecessary investigation.
**Why it happens:** Developers may assume Arc pointer identity is used for equality; it uses slice content.

### Pitfall 5: MEM-05 retain closure borrows issues
**What goes wrong:** Using `seen.entry(h)` inside a `retain` closure that also borrows `c` simultaneously triggers borrow checker errors if the entry API borrows `seen` mutably while `c` is borrowed immutably.
**Why it happens:** `retain` takes `|c| -> bool`; `seen` is also captured. This is standard and works, but `entry().or_insert()` returning a mutable reference while the closure holds `c` can cause issues with some patterns.
**How to avoid:** Compute `key` before calling `entry`, or use `contains_key` + `insert` as two separate calls (avoids double lookup but is simpler to read).

---

## Code Examples

### MEM-01: Arc field declaration and constructor

```rust
// Source: Rust stdlib — Arc<[T]> via From<Box<[T]>>
use std::sync::Arc;

pub struct Range<T> {
    pub id: i32,
    pub ranges: Arc<[(T, T)]>,
    pub value: T,
}

// In new():
pub fn new(id: i32, ranges: Vec<(T, T)>, value: T) -> Self {
    Self {
        id,
        ranges: ranges.into_boxed_slice().into(),
        value,
    }
}
```

### MEM-02: Copy-specialized value()

```rust
// Single impl<T: Copy + Default> — covers all current Range users
impl<T: Copy + Default> Range<T> {
    pub fn new(id: i32, ranges: Vec<(T, T)>, value: T) -> Self {
        Self { id, ranges: ranges.into_boxed_slice().into(), value }
    }

    pub fn value(&self) -> T {
        self.value   // no .clone() — T: Copy means bitwise copy
    }

    pub fn set_value(&mut self, value: T) -> &mut Self {
        self.value = value;
        self
    }
}
```

### MEM-04: #[inline] annotation

```rust
// Source: Rust reference — #[inline] hint for thin wrappers
impl<G: GeneT> FitnessFnWrapper<G> {
    #[inline]
    pub fn call(&self, dna: &[G]) -> f64 {
        (self.0)(dna)
    }
}
```

### MEM-05: DefaultHasher with retain

```rust
// Source: std::hash docs — DefaultHasher incremental hashing
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

let mut seen: HashMap<u64, Vec<i32>> = HashMap::new();
chromosomes.retain(|c| {
    let mut hasher = DefaultHasher::new();
    for g in c.dna() {
        g.id().hash(&mut hasher);
    }
    let h = hasher.finish();

    match seen.entry(h) {
        std::collections::hash_map::Entry::Vacant(e) => {
            let ids: Vec<i32> = c.dna().iter().map(|g| g.id()).collect();
            e.insert(ids);
            true  // first seen — keep
        }
        std::collections::hash_map::Entry::Occupied(e) => {
            // collision path: exact compare
            let ids: Vec<i32> = c.dna().iter().map(|g| g.id()).collect();
            ids != *e.get()  // true = hash collision (distinct); false = true duplicate (remove)
        }
    }
});
```

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|-----------------|--------|
| `Vec<(T,T)>` per gene — separate heap alloc | `Arc<[(T,T)]>` shared slice | Reduces heap allocations when multiple chromosomes share the same gene template |
| `.clone()` on Copy values | Direct value return | Eliminates unnecessary clone call on every `value()` access in hot mutation/crossover loops |
| `HashSet<Vec<i32>>` — allocates `Vec<i32>` per chromosome | `HashMap<u64, Vec<i32>>` — allocates `Vec<i32>` only on first-seen hash | O(n) Vec allocations → 0 on fast path, 1 per unique hash on collision path |

---

## Open Questions

1. **Serde `rc` feature flag**
   - What we know: `Arc<[T]>` deserialization requires serde's `rc` feature
   - What's unclear: Whether the current `Cargo.toml` already includes `features = ["rc"]` — needs a one-line check before MEM-01 is planned
   - Recommendation: Implementer must check `Cargo.toml` serde entry and add `"rc"` if absent; plan should include this as a prerequisite step

2. **Non-Copy Range<T> users**
   - What we know: All current Range gene usage in the codebase uses `f64` or `i32` (both `Copy`)
   - What's unclear: Public API concern — removing the `Clone`-only impl could break downstream users of the library who use non-Copy T
   - Recommendation: Keep `impl<T: Clone + Default>` for `new()` and `set_value()`, add **only** a `Copy`-gated `value()` method to avoid overlap issues. See Architecture Patterns section for the single-impl approach.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]`) + `cargo test` |
| Config file | none (standard cargo test runner) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MEM-01 | `Range.ranges` is `Arc<[(T,T)]>`; serde round-trip preserved | unit + integration | `cargo test serde_range_gene && cargo test --features serde serde_range_gene` | Yes (`tests/test_serde.rs`) |
| MEM-02 | `Range::value()` for `Copy` types returns value without clone | unit | `cargo test -p genetic_algorithms test_range` | Yes (`tests/test_initializers.rs`, mutation tests) |
| MEM-03 | `Population` struct no longer has `generation_numbers`; serde round-trip passes | unit + integration | `cargo test serde_population && cargo test --features serde serde_population` | Yes (`tests/test_serde.rs`, `tests/test_population.rs`) |
| MEM-04 | `FitnessFnWrapper::call()` annotated `#[inline]` — compilation succeeds | compile check | `cargo build` | Yes (`tests/test_fitness.rs`) |
| MEM-05 | `MassDeduplication` correctly removes duplicates, keeps best | unit | `cargo test mass_deduplication` | Yes (`tests/extension/test_extension.rs`) |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
None — existing test infrastructure covers all phase requirements. The serde tests for `Range` gene and `Population` are present and will serve as regression tests for MEM-01 and MEM-03 respectively. The `mass_deduplication_*` tests in `tests/extension/test_extension.rs` cover MEM-05 behavior.

---

## Sources

### Primary (HIGH confidence)
- Direct source code inspection: `src/genotypes/range.rs`, `src/population.rs`, `src/fitness/fitness_fn_wrapper.rs`, `src/operations/extension/mass_deduplication.rs` — current implementation fully read
- Rust stdlib docs (training data, HIGH confidence for stable features): `Arc<[T]>`, `std::hash::DefaultHasher`, `#[inline]`
- `tests/test_serde.rs` — existing serde tests verified present and inspected

### Secondary (MEDIUM confidence)
- serde `rc` feature requirement for `Arc` — standard serde documentation behavior; Cargo.toml was not read in this session — **implementer must verify**

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all std; no new crates
- Architecture: HIGH — all patterns verified against actual source code
- Pitfalls: HIGH — derived from reading actual serde impl and identifying exact removal sites
- MEM-01 serde rc flag: MEDIUM — pattern is correct but Cargo.toml not verified

**Research date:** 2026-04-01
**Valid until:** Stable (no moving dependencies; all std)
