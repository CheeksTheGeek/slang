/* Guest-side trampoline for the custom DataFlow/Lattice bridge in wasm.
 *
 * slang runs the dataflow analysis inside the guest and drives a caller-defined
 * lattice through function pointers. Here those seven callbacks each forward to
 * a single imported host function (`env.dfa_dispatch`), so the actual lattice
 * logic lives in Rust on the host. The abstract "state" pointers slang passes
 * around are opaque to slang, so the host uses them as small integer handles
 * into a host-side slab of real Rust lattice values.
 *
 * Compiled and linked into the module by wasm/build.sh (guest only). */
#include <stdint.h>

#include "slang/c/slang.h"

__attribute__((import_module("env"), import_name("dfa_dispatch"))) extern int32_t
host_dfa_dispatch(int32_t op, void* user, const void* a, const void* b);

enum {
    OP_TOP = 0,
    OP_BOTTOM = 1,
    OP_CLONE = 2,
    OP_JOIN = 3,
    OP_MEET = 4,
    OP_TRANSFER = 5,
    OP_DROP = 6
};

static void* w_top(void* user) {
    return (void*)(intptr_t)host_dfa_dispatch(OP_TOP, user, 0, 0);
}
static void* w_bottom(void* user) {
    return (void*)(intptr_t)host_dfa_dispatch(OP_BOTTOM, user, 0, 0);
}
static void* w_clone(void* user, const void* state) {
    return (void*)(intptr_t)host_dfa_dispatch(OP_CLONE, user, state, 0);
}
static void w_join(void* user, void* into, const void* other) {
    host_dfa_dispatch(OP_JOIN, user, into, other);
}
static void w_meet(void* user, void* into, const void* other) {
    host_dfa_dispatch(OP_MEET, user, into, other);
}
static void w_transfer(void* user, void* state, const slang_dfa_event* event) {
    host_dfa_dispatch(OP_TRANSFER, user, state, event);
}
static void w_drop(void* user, void* state) {
    host_dfa_dispatch(OP_DROP, user, state, 0);
}

__attribute__((export_name("slang_wasm_dfa_run"))) void* slang_wasm_dfa_run(
    slang_compilation comp, slang_ast procedure, void* user, slang_error* err) {
    /* Zero-initialize so the observer hooks (on_case_begin / on_conditional_begin
     * / on_loop_begin) are null. slang null-checks each before calling, so null
     * means "skip" — exactly matching a native Lattice's default no-op observers.
     * WasmLattice exposes no observer methods, so there is nothing to forward
     * them to; leaving them as uninitialized stack garbage made slang indirect-
     * call a bogus table slot ("undefined element") on any design with an
     * if/case/loop. */
    slang_dfa_lattice lat = {0};
    lat.top = w_top;
    lat.bottom = w_bottom;
    lat.clone = w_clone;
    lat.join = w_join;
    lat.meet = w_meet;
    lat.transfer = w_transfer;
    lat.drop = w_drop;
    return slang_dfa_run(comp, procedure, &lat, user, err);
}
