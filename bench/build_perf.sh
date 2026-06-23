#!/usr/bin/env bash
# bench/build_perf.sh — Reproducible build-performance measurement harness
#
# Measures six canonical metrics for the genetic_algorithms crate and writes
# them to target/build-perf/results.json.
#
# Usage:
#   bash bench/build_perf.sh           # measure and write results.json
#   bash bench/build_perf.sh --commit  # as above, then copy results.json to
#                                      # .planning/baselines/v3.0.0-baseline.json
#
# Revert plan: delete bench/build_perf.sh
#
# Metrics (D-08 schema):
#   dev_build_s      — wall-clock seconds for cargo clean + cargo build (default features)
#   wasm_check_s     — wall-clock seconds for cargo clean + cargo check --target wasm32-unknown-unknown
#   test_suite_s     — wall-clock seconds for cargo clean + cargo test --quiet
#   dep_count        — unique transitive dependency crate count
#   public_api_hash  — sha256 of cargo public-api output ("unavailable" if not installed)
#   captured_at      — ISO date of measurement (YYYY-MM-DD)
#
# Outputs:
#   target/build-perf/results.json          — ephemeral measurement (gitignored)
#   target/build-perf/<name>-seed42.txt     — stdout of four reference examples (for Plan 66-03)
#
# Requirements: rustup, cargo, wasm32-unknown-unknown target installed.
# Optional: cargo-public-api (used when available; gracefully skipped when absent).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

COMMIT_BASELINE=false
if [[ "${1:-}" == "--commit" ]]; then
    COMMIT_BASELINE=true
fi

OUT_DIR="target/build-perf"
mkdir -p "$OUT_DIR"

# ---------------------------------------------------------------------------
# Helper: convert the bash `time` "real\tXmYYs" output to decimal seconds.
# Handles both macOS `time` format (e.g. "0m5.123s") and Linux format.
# ---------------------------------------------------------------------------
parse_time_to_seconds() {
    local raw="$1"
    # Extract the "real" line regardless of surrounding whitespace/tabs.
    local real_line
    real_line=$(printf '%s' "$raw" | grep -E '^real' | head -1 || true)
    if [[ -z "$real_line" ]]; then
        echo "0"
        return
    fi
    # The format after "real" is: \t<N>m<M.mmm>s  (BSD/macOS) or just <N>m<M.mmm>s
    # Extract minutes and seconds parts.
    local time_str
    time_str=$(printf '%s' "$real_line" | sed 's/^real[[:space:]]*//')
    local minutes seconds
    minutes=$(printf '%s' "$time_str" | sed 's/m.*//' | tr -d '[:space:]')
    seconds=$(printf '%s' "$time_str" | sed 's/^[0-9]*m//' | sed 's/s$//' | tr -d '[:space:]')
    # Use awk for floating-point arithmetic (minutes * 60 + seconds).
    awk -v m="$minutes" -v s="$seconds" 'BEGIN { printf "%.3f", m * 60 + s }'
}

echo "=== build_perf.sh — genetic_algorithms measurement harness ==="
echo "Repo root: $REPO_ROOT"
echo "Output:    $OUT_DIR/results.json"
echo ""

# ---------------------------------------------------------------------------
# Metric 1: dev_build_s — clean dev build (default features)
# ---------------------------------------------------------------------------
echo "[1/6] Measuring dev build (cargo clean + cargo build) ..."
cargo clean --quiet
TIME_OUTPUT=$( { time cargo build --quiet; } 2>&1 )
DEV_BUILD_S=$(parse_time_to_seconds "$TIME_OUTPUT")
echo "      dev_build_s = ${DEV_BUILD_S}s"

# ---------------------------------------------------------------------------
# Metric 2: wasm_check_s — WASM compatibility check
# ---------------------------------------------------------------------------
echo "[2/6] Measuring WASM check (cargo clean + cargo check --target wasm32-unknown-unknown --lib) ..."
cargo clean --quiet
TIME_OUTPUT=$( { time cargo check --target wasm32-unknown-unknown --lib --quiet 2>&1; } 2>&1 )
WASM_CHECK_S=$(parse_time_to_seconds "$TIME_OUTPUT")
echo "      wasm_check_s = ${WASM_CHECK_S}s"

# ---------------------------------------------------------------------------
# Metric 3: test_suite_s — full test suite
# ---------------------------------------------------------------------------
echo "[3/6] Measuring test suite (cargo clean + cargo test --quiet) ..."
cargo clean --quiet
TIME_OUTPUT=$( { time cargo test --quiet 2>&1; } 2>&1 )
TEST_SUITE_S=$(parse_time_to_seconds "$TIME_OUTPUT")
echo "      test_suite_s = ${TEST_SUITE_S}s"

# ---------------------------------------------------------------------------
# Metric 4: dep_count — unique transitive dependency count
# ---------------------------------------------------------------------------
echo "[4/6] Counting transitive dependencies (cargo tree) ..."
DEP_COUNT=$(cargo tree --prefix none 2>/dev/null | sort -u | wc -l | tr -d ' ')
echo "      dep_count = $DEP_COUNT"

# ---------------------------------------------------------------------------
# Metric 5: public_api_hash — SHA-256 of public API snapshot
# ---------------------------------------------------------------------------
echo "[5/6] Capturing public API hash ..."
if cargo public-api --help > /dev/null 2>&1; then
    PUBLIC_API_HASH=$(cargo public-api 2>/dev/null | sha256sum | awk '{print $1}')
    echo "      public_api_hash = $PUBLIC_API_HASH"
else
    PUBLIC_API_HASH="unavailable"
    echo "      WARNING: cargo-public-api not installed. Set public_api_hash='unavailable'."
    echo "               Install with: cargo install cargo-public-api"
fi

# ---------------------------------------------------------------------------
# Metric 6: captured_at — ISO date
# ---------------------------------------------------------------------------
CAPTURED_AT=$(date +%Y-%m-%d)

# ---------------------------------------------------------------------------
# Write results.json (six D-08 fields, no jq dependency)
# cargo clean may have removed target/; recreate the output directory.
# ---------------------------------------------------------------------------
mkdir -p "$OUT_DIR"
cat > "$OUT_DIR/results.json" << RESULTS_JSON
{
  "dev_build_s": ${DEV_BUILD_S},
  "wasm_check_s": ${WASM_CHECK_S},
  "test_suite_s": ${TEST_SUITE_S},
  "dep_count": ${DEP_COUNT},
  "public_api_hash": "${PUBLIC_API_HASH}",
  "captured_at": "${CAPTURED_AT}"
}
RESULTS_JSON

echo ""
echo "[6/6] Running four reference examples with --seed 42 (golden captures for Plan 66-03) ..."

EXAMPLES=("rastrigin" "nsga2_zdt1" "cma_es_rastrigin" "pso_rastrigin")
for EXAMPLE in "${EXAMPLES[@]}"; do
    echo "      Running: cargo run --example $EXAMPLE --release -- --seed 42"
    cargo run --example "$EXAMPLE" --release -- --seed 42 > "$OUT_DIR/${EXAMPLE}-seed42.txt" 2>&1 \
        || { echo "      WARNING: Example '$EXAMPLE' exited non-zero; output captured anyway."; }
    echo "      Captured: $OUT_DIR/${EXAMPLE}-seed42.txt"
done

echo ""
echo "Results written to: $OUT_DIR/results.json"
echo ""
echo "=== Measurement Summary ==="
printf "  dev_build_s   : %s s\n" "$DEV_BUILD_S"
printf "  wasm_check_s  : %s s\n" "$WASM_CHECK_S"
printf "  test_suite_s  : %s s\n" "$TEST_SUITE_S"
printf "  dep_count     : %s\n"   "$DEP_COUNT"
printf "  public_api_hash: %s\n"  "$PUBLIC_API_HASH"
printf "  captured_at   : %s\n"   "$CAPTURED_AT"

# ---------------------------------------------------------------------------
# Optional --commit: copy results.json to .planning/baselines/v3.0.0-baseline.json
# ---------------------------------------------------------------------------
if [[ "$COMMIT_BASELINE" == true ]]; then
    BASELINE_DIR=".planning/baselines"
    mkdir -p "$BASELINE_DIR"
    BASELINE_FILE="$BASELINE_DIR/v3.0.0-baseline.json"
    cp "$OUT_DIR/results.json" "$BASELINE_FILE"
    echo ""
    echo "Baseline committed to $BASELINE_FILE"
fi
