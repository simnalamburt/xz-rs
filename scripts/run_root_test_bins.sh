#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  cat <<'EOF' >&2
Usage: scripts/run_root_test_bins.sh <cargo-target-dir> [repeats]

Run the prebuilt root crate unit and integration test binaries directly,
without going through `cargo test`. This avoids cargo/doctest overhead when
comparing backend runtime performance.
EOF
  exit 2
fi

TARGET_DIR="$1"
REPEATS="${2:-1}"
DEPS_DIR="$TARGET_DIR/release/deps"

if [[ "$REPEATS" -lt 1 ]]; then
  echo "repeats must be >= 1" >&2
  exit 2
fi

for ((repeat = 0; repeat < REPEATS; repeat++)); do
  for prefix in \
    "xz-" \
    "drop_incomplete-" \
    "standard_files-"
  do
    BIN=""
    while IFS= read -r candidate; do
      if [[ -x "$candidate" ]]; then
        BIN="$candidate"
        break
      fi
    done < <(find "$DEPS_DIR" -maxdepth 1 -type f -name "${prefix}*" | sort)
    if [[ -z "$BIN" ]]; then
      echo "missing test binary for prefix $prefix in $DEPS_DIR" >&2
      exit 1
    fi
    ARGS=(--test-threads=1)
    if [[ "$prefix" == "xz-" ]]; then
      # QuickCheck-based tests use a fresh RNG per process, so backend-to-backend
      # wall-clock comparisons on them are not stable. Cover those paths with
      # deterministic focused probes instead.
      ARGS+=(
        --skip read::tests::qc
        --skip read::tests::qc_lzma1
        --skip write::tests::qc
        --skip write::tests::qc_lzma1
        --skip tests::all
        --skip tests::copy
        --skip tests::size
      )
    fi
    "$BIN" "${ARGS[@]}"
  done
done
