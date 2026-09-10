#!/usr/bin/env bash
# Rebuilds wasm/slang_c.wasm.gz: slang's C API compiled to wasm32-wasip1 and
# linked into a reactor module that exports every slang_* function.
#
# Requires a wasi-sdk (https://github.com/WebAssembly/wasi-sdk) and a slang
# checkout. Usage:
#   WASI_SDK=/path/to/wasi-sdk SLANG_SRC=/path/to/slang ./build.sh
set -euo pipefail

WASI_SDK="${WASI_SDK:?set WASI_SDK to your wasi-sdk directory}"
SLANG_SRC="${SLANG_SRC:-$(cd "$(dirname "$0")/../../.." && pwd)}"
HERE="$(cd "$(dirname "$0")" && pwd)"
BUILD="$(mktemp -d)"

echo "Configuring slang for wasm32-wasip1 in $BUILD ..."
cmake -S "$SLANG_SRC" -B "$BUILD" -G Ninja \
  -DCMAKE_TOOLCHAIN_FILE="$WASI_SDK/share/cmake/wasi-sdk-p1.cmake" \
  -DCMAKE_BUILD_TYPE=Release \
  -DSLANG_INCLUDE_CAPI=ON -DSLANG_INCLUDE_TESTS=OFF -DSLANG_INCLUDE_TOOLS=OFF \
  -DSLANG_USE_MIMALLOC=OFF -DSLANG_USE_THREADS=OFF

echo "Building slang_c ..."
cmake --build "$BUILD" --target slang_c

# The guest-side dataflow trampoline (forwards slang's lattice callbacks to the
# host import env.dfa_dispatch).
echo "Compiling dfa_shim.c ..."
"$WASI_SDK/bin/clang" --target=wasm32-wasip1 --sysroot="$WASI_SDK/share/wasi-sysroot" \
  -DSLANG_STATIC_DEFINE -DSLANG_C_STATIC -I"$SLANG_SRC/include" \
  -O2 -c "$HERE/dfa_shim.c" -o "$BUILD/dfa_shim.o"

# Every exported slang_* function, minus the unstable ones (compiled out).
grep -oE 'slang_[a-z_]+\(' "$SLANG_SRC/include/slang/c/slang.h" \
  | tr -d '(' | sort -u | grep -v slang_unstable_native > "$BUILD/exports.txt"

echo "Linking reactor module ..."
"$WASI_SDK/bin/clang++" --target=wasm32-wasip1 --sysroot="$WASI_SDK/share/wasi-sysroot" \
  -mexec-model=reactor -O2 -fno-exceptions \
  -Wl,--export=malloc -Wl,--export=free -Wl,--export=slang_wasm_dfa_run \
  -Wl,--allow-undefined \
  $(sed 's/^/-Wl,--export=/' "$BUILD/exports.txt" | tr '\n' ' ') \
  "$BUILD/dfa_shim.o" "$BUILD/lib/libslang-c.a" "$BUILD/lib/libsvlang.a" \
  -o "$BUILD/slang_c.wasm"

gzip -9 -c "$BUILD/slang_c.wasm" > "$HERE/slang_c.wasm.gz"
echo "Wrote $HERE/slang_c.wasm.gz ($(du -h "$HERE/slang_c.wasm.gz" | cut -f1))"
rm -rf "$BUILD"
