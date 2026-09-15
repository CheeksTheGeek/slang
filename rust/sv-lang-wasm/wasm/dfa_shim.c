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
    OP_DROP = 6,
    OP_ON_CASE = 7,
    OP_ON_CONDITIONAL = 8,
    OP_ON_LOOP = 9
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

/* Observer hooks. slang passes an opaque `ctx` and the statement being entered.
 * The guest resolves the two cheap scalars the host FlowContext exposes —
 * `slang_dfa_ctx_state` (the current state's opaque handle, which in this bridge
 * IS the host slab id the lattice operates on) and `slang_dfa_ctx_is_bad` — here
 * in the guest, so the host needs no re-entrant call back into the sandbox. The
 * statement value-struct is passed by pointer to the guest-stack copy; the host
 * reads its four words while this frame is live. `eval_constant` is deliberately
 * not bridged (it would require host->guest re-entry mid-lattice-callback); it is
 * a native-only FlowContext method. */
static void w_observe(int32_t op, slang_dfa_ctx ctx, slang_ast stmt) {
    int32_t state_id = (int32_t)(intptr_t)slang_dfa_ctx_state(ctx);
    int32_t is_bad = slang_dfa_ctx_is_bad(ctx) ? 1 : 0;
    host_dfa_dispatch(op, (void*)(intptr_t)state_id, (const void*)&stmt,
                      (const void*)(intptr_t)is_bad);
}
static void w_on_case(void* user, slang_dfa_ctx ctx, slang_ast stmt) {
    (void)user;
    w_observe(OP_ON_CASE, ctx, stmt);
}
static void w_on_conditional(void* user, slang_dfa_ctx ctx, slang_ast stmt) {
    (void)user;
    w_observe(OP_ON_CONDITIONAL, ctx, stmt);
}
static void w_on_loop(void* user, slang_dfa_ctx ctx, slang_ast stmt) {
    (void)user;
    w_observe(OP_ON_LOOP, ctx, stmt);
}

__attribute__((export_name("slang_wasm_dfa_run"))) void* slang_wasm_dfa_run(
    slang_compilation comp, slang_ast procedure, void* user, slang_error* err) {
    /* Zero-initialize first: any field left unset must read as a null pointer,
     * not uninitialized stack garbage. slang null-checks each observer before
     * calling it, and an indirect call through a garbage table slot traps
     * ("undefined element") on any design with an if/case/loop. */
    slang_dfa_lattice lat = {0};
    lat.top = w_top;
    lat.bottom = w_bottom;
    lat.clone = w_clone;
    lat.join = w_join;
    lat.meet = w_meet;
    lat.transfer = w_transfer;
    lat.drop = w_drop;
    lat.on_case_begin = w_on_case;
    lat.on_conditional_begin = w_on_conditional;
    lat.on_loop_begin = w_on_loop;
    return slang_dfa_run(comp, procedure, &lat, user, err);
}
