#!/usr/bin/env bash
# Fetch large real-world SystemVerilog designs and run the sv-lang round-trip
# test over them. Nothing here is committed; clones go to a scratch dir.
#
#   ./scripts/fetch-corpus.sh [dest-dir]
#
# then it runs `cargo test -p sv-lang --test corpus external_corpus_round_trips`
# with SV_LANG_CORPUS pointed at the fetched sources.
set -euo pipefail

DEST="${1:-${TMPDIR:-/tmp}/sv-lang-corpus}"
HERE="$(cd "$(dirname "$0")/.." && pwd)" # rust/
mkdir -p "$DEST"

clone() { # url dir
    if [ -d "$DEST/$2/.git" ]; then
        echo "have $2"
    else
        echo "cloning $2 ..."
        git clone --depth 1 "$1" "$DEST/$2"
    fi
}

clone https://github.com/lowRISC/ibex.git ibex
clone https://github.com/openhwgroup/cva6.git cva6
# OpenTitan is large; clone only if explicitly requested (WITH_OT=1).
if [ "${WITH_OT:-0}" = "1" ]; then
    clone https://github.com/lowRISC/opentitan.git opentitan
fi

echo "running round-trip over $DEST ..."
SV_LANG_CORPUS="$DEST" cargo test --manifest-path "$HERE/Cargo.toml" \
    -p sv-lang --test corpus external_corpus_round_trips -- --nocapture
