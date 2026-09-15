//! Run [slang](https://sv-lang.com) inside a WebAssembly sandbox.
//!
//! This crate embeds slang's C API compiled to `wasm32-wasip1` and drives it
//! through [`wasmtime`], so you can parse and inspect SystemVerilog with **no
//! native slang and no C++ toolchain** — the whole compiler is a `.wasm` file
//! loaded at runtime, sandboxed from the host.
//!
//! It is the `backend-wasm` proof: the same slang C ABI the native
//! [`sv-lang`](https://crates.io/crates/sv-lang) crate links directly is here
//! reached across the wasm boundary instead, marshalling arguments and results
//! through the guest's 32-bit linear memory.
//!
//! ```no_run
//! let mut slang = sv_lang_wasm::Slang::new()?;
//! assert!(slang.version().starts_with("11."));
//!
//! let tree = slang.parse("module top; endmodule\n")?;
//! assert_eq!(slang.root_kind_name(&tree)?, "CompilationUnit");
//! assert_eq!(slang.module_count(&tree)?, 1);
//! # Ok::<(), sv_lang_wasm::Error>(())
//! ```
#![warn(missing_docs)]

use std::fmt;

use std::any::Any;
use std::collections::BTreeMap;
use std::sync::OnceLock;

use wasmtime::{
    Cache, Caller, Config, Engine, Instance, Linker, Memory, Module, Store, StoreLimits,
    StoreLimitsBuilder, TypedFunc, Val,
};
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::p1::WasiP1Ctx;

// The generated raw marshalling for every non-callback slang_* C function
// (`raw_*` methods on `Slang`); the ergonomic API below is written on top.
mod generated {
    pub mod bridge;
}

/// An event delivered to [`WasmLattice::transfer`] as slang walks a procedure.
#[derive(Clone, Copy, Debug)]
pub struct WasmDfaEvent {
    /// `0` = read, `1` = write, `2` = call.
    pub kind: u32,
    /// The value symbol's guest `slang_ast` words `[ptr, comp, kind, domain]`;
    /// `symbol[0] == 0` for a call.
    pub symbol: [u32; 4],
}

/// A caller-defined lattice for the in-sandbox custom dataflow analysis — the
/// wasm analogue of the native `sv_lang::dataflow::Lattice`. Implement it and
/// pass to [`Slang::run_lattice`]; slang drives it from inside the guest,
/// calling back out to the host for each operation.
pub trait WasmLattice: Clone + Send + 'static {
    /// The entry (top) state.
    fn top() -> Self;
    /// The unreachable (bottom) state; defaults to [`top`](Self::top).
    fn bottom() -> Self {
        Self::top()
    }
    /// Merge `other` into `self` where branches rejoin.
    fn join(&mut self, other: &Self);
    /// Merge `other` into `self` at a loop back-edge; defaults to `join`.
    fn meet(&mut self, other: &Self) {
        self.join(other);
    }
    /// Apply a read/write/call event to the state.
    fn transfer(&mut self, event: &WasmDfaEvent);

    /// Called when the analysis begins visiting a case statement, before any of
    /// its branches. Mirrors `sv_lang::dataflow::Lattice::on_case_begin`. The
    /// default is a no-op.
    fn on_case_begin(&mut self, _stmt: Node, _ctx: &WasmFlowContext) {}

    /// Called when the analysis begins visiting a conditional (`if`/`else`)
    /// statement, before its branches. Mirrors
    /// `sv_lang::dataflow::Lattice::on_conditional_begin`. The default is a
    /// no-op.
    fn on_conditional_begin(&mut self, _stmt: Node, _ctx: &WasmFlowContext) {}

    /// Called when the analysis begins visiting any loop statement, before its
    /// body. Mirrors `sv_lang::dataflow::Lattice::on_loop_begin`. The default
    /// is a no-op.
    fn on_loop_begin(&mut self, _stmt: Node, _ctx: &WasmFlowContext) {}
}

/// Context handed to a [`WasmLattice`]'s observer hooks (`on_case_begin`,
/// `on_conditional_begin`, `on_loop_begin`).
///
/// The guest resolves the two cheap scalars this exposes before crossing back
/// to the host, so no re-entrant call into the sandbox is needed. Unlike the
/// native `sv_lang::dataflow::FlowContext`, this does **not** offer
/// `eval_constant`: evaluating an arbitrary expression from an observer would
/// require host→guest re-entry in the middle of a lattice callback, so
/// observer-time constant evaluation is a native-backend-only capability.
#[derive(Clone, Copy, Debug)]
pub struct WasmFlowContext {
    is_bad: bool,
    state_addr: usize,
}

impl WasmFlowContext {
    /// True if the analysis has recorded an unrecoverable error (an
    /// `InvalidStatement`/`InvalidExpression` was visited). Mirrors
    /// `sv_lang::dataflow::FlowContext::is_bad`.
    pub fn is_bad(&self) -> bool {
        self.is_bad
    }

    /// The opaque identity of the current flow state (the same state the
    /// lattice methods operate on). Mirrors
    /// `sv_lang::dataflow::FlowContext::current_state_addr` — exposed for
    /// identity only; not dereferenceable.
    pub fn current_state_addr(&self) -> usize {
        self.state_addr
    }
}

/// A type-erased lattice state (a `Box<L>` seen as `dyn Any`), plus the erased
/// lattice operations that act on it.
type Erased = Box<dyn Any + Send>;
type ConstructFn = Box<dyn Fn() -> Erased + Send>;
type CloneFn = Box<dyn Fn(&dyn Any) -> Erased + Send>;
type MergeFn = Box<dyn Fn(&mut dyn Any, &dyn Any) + Send>;
type TransferFn = Box<dyn Fn(&mut dyn Any, &WasmDfaEvent) + Send>;
type ObserveFn = Box<dyn Fn(&mut dyn Any, Node, &WasmFlowContext) + Send>;

/// A type-erased in-flight dataflow run: the host-side lattice states slang
/// operates on (addressed by the opaque integer "state pointers" it passes),
/// plus the monomorphized lattice operations for the specific `L`.
struct DfaRun {
    slab: Vec<Option<Erased>>,
    top: ConstructFn,
    bottom: ConstructFn,
    clone_: CloneFn,
    join: MergeFn,
    meet: MergeFn,
    transfer: TransferFn,
    on_case: ObserveFn,
    on_conditional: ObserveFn,
    on_loop: ObserveFn,
}

impl DfaRun {
    fn new<L: WasmLattice>() -> DfaRun {
        DfaRun {
            slab: Vec::new(),
            top: Box::new(|| Box::new(L::top()) as Box<dyn Any + Send>),
            bottom: Box::new(|| Box::new(L::bottom()) as Box<dyn Any + Send>),
            clone_: Box::new(|s| {
                Box::new(s.downcast_ref::<L>().unwrap().clone()) as Box<dyn Any + Send>
            }),
            join: Box::new(|into, other| {
                let o = other.downcast_ref::<L>().unwrap().clone();
                into.downcast_mut::<L>().unwrap().join(&o);
            }),
            meet: Box::new(|into, other| {
                let o = other.downcast_ref::<L>().unwrap().clone();
                into.downcast_mut::<L>().unwrap().meet(&o);
            }),
            transfer: Box::new(|st, ev| {
                st.downcast_mut::<L>().unwrap().transfer(ev);
            }),
            on_case: Box::new(|st, stmt, ctx| {
                st.downcast_mut::<L>().unwrap().on_case_begin(stmt, ctx);
            }),
            on_conditional: Box::new(|st, stmt, ctx| {
                st.downcast_mut::<L>()
                    .unwrap()
                    .on_conditional_begin(stmt, ctx);
            }),
            on_loop: Box::new(|st, stmt, ctx| {
                st.downcast_mut::<L>().unwrap().on_loop_begin(stmt, ctx);
            }),
        }
    }

    fn push(&mut self, state: Box<dyn Any + Send>) -> i32 {
        for (i, slot) in self.slab.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(state);
                return (i + 1) as i32;
            }
        }
        self.slab.push(Some(state));
        self.slab.len() as i32
    }

    fn get(&self, id: u32) -> Option<&(dyn Any + Send)> {
        (id != 0)
            .then(|| self.slab.get((id - 1) as usize))
            .flatten()
            .and_then(Option::as_ref)
            .map(Box::as_ref)
    }

    fn merge(&mut self, a: u32, b: u32, is_join: bool) {
        let Some(other) = self.get(b).map(|s| (self.clone_)(s)) else {
            return;
        };
        if a != 0
            && let Some(Some(into)) = self.slab.get_mut((a - 1) as usize)
        {
            if is_join {
                (self.join)(into.as_mut(), other.as_ref());
            } else {
                (self.meet)(into.as_mut(), other.as_ref());
            }
        }
    }

    /// Handles one lattice callback. For lattice ops `a`/`b` are state ids and
    /// `ev` is the event's `slang_ast` (only meaningful for a WRITE transfer).
    /// For observer ops (7/8/9) `user` is the current state id, `ev` is the
    /// observed statement's `slang_ast`, and `b` is the `is_bad` flag.
    fn dispatch(&mut self, op: i32, user: u32, a: u32, b: u32, ev_kind: u32, ev: [u32; 4]) -> i32 {
        match op {
            0 => {
                let s = (self.top)();
                self.push(s)
            }
            1 => {
                let s = (self.bottom)();
                self.push(s)
            }
            2 => match self.get(a) {
                Some(s) => {
                    let c = (self.clone_)(s);
                    self.push(c)
                }
                None => 0,
            },
            3 => {
                self.merge(a, b, true);
                0
            }
            4 => {
                self.merge(a, b, false);
                0
            }
            5 => {
                if a != 0
                    && let Some(Some(st)) = self.slab.get_mut((a - 1) as usize)
                {
                    let event = WasmDfaEvent {
                        kind: ev_kind,
                        symbol: ev,
                    };
                    (self.transfer)(st.as_mut(), &event);
                }
                0
            }
            6 => {
                if a != 0
                    && let Some(slot) = self.slab.get_mut((a - 1) as usize)
                {
                    *slot = None;
                }
                0
            }
            7..=9 => {
                if user != 0
                    && let Some(Some(st)) = self.slab.get_mut((user - 1) as usize)
                {
                    let stmt = Node(Ast {
                        ptr: ev[0],
                        comp: ev[1],
                        kind: ev[2],
                        domain: ev[3],
                    });
                    let ctx = WasmFlowContext {
                        is_bad: b != 0,
                        state_addr: user as usize,
                    };
                    let observe = match op {
                        7 => &self.on_case,
                        8 => &self.on_conditional,
                        _ => &self.on_loop,
                    };
                    observe(st.as_mut(), stmt, &ctx);
                }
                0
            }
            _ => 0,
        }
    }

    /// Extracts the exit state (by id) as the concrete lattice, consuming it.
    fn take_state<L: WasmLattice>(&mut self, id: u32) -> Option<L> {
        if id == 0 {
            return None;
        }
        let boxed = self.slab.get_mut((id - 1) as usize)?.take()?;
        boxed.downcast::<L>().ok().map(|b| *b)
    }
}

/// The built-in reaching-writes lattice behind [`Slang::reaching_writes`].
#[derive(Clone, Default)]
struct WriteSetLattice(BTreeMap<u32, [u32; 4]>);

impl WasmLattice for WriteSetLattice {
    fn top() -> Self {
        WriteSetLattice::default()
    }
    fn join(&mut self, other: &Self) {
        self.0.extend(other.0.iter().map(|(k, v)| (*k, *v)));
    }
    fn meet(&mut self, other: &Self) {
        self.0.retain(|k, _| other.0.contains_key(k));
    }
    fn transfer(&mut self, ev: &WasmDfaEvent) {
        if ev.kind == 1 && ev.symbol[0] != 0 {
            self.0.insert(ev.symbol[0], ev.symbol);
        }
    }
}

/// The store's host data: the WASI context, the resource limiter, and the
/// in-flight custom dataflow run (if any).
struct Host {
    wasi: WasiP1Ctx,
    limits: StoreLimits,
    dfa: Option<DfaRun>,
}

/// Resource limits for a sandboxed instance (see [`Slang::with_limits`]).
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum linear-memory size in bytes. `None` = the wasm 4 GiB ceiling.
    pub max_memory: Option<usize>,
    /// Fuel budget: the guest traps (returning [`Error::Runtime`]) once it
    /// executes this many instructions. `None` = unlimited, i.e. a hostile
    /// input can hang. The budget is shared across all calls on one instance,
    /// so use a fresh instance per untrusted input.
    pub fuel: Option<u64>,
}

impl Default for Limits {
    fn default() -> Self {
        // 2 GiB memory, ~100e9 instructions — ample for real files, but bounded.
        Limits {
            max_memory: Some(2 << 30),
            fuel: Some(100_000_000_000),
        }
    }
}

/// The slang C API compiled to `wasm32-wasip1`, gzip-compressed (it is ~14 MB
/// raw, ~2.8 MB compressed); decompressed once in [`Slang::new`].
const SLANG_WASM_GZ: &[u8] = include_bytes!("../wasm/slang_c.wasm.gz");

fn decompress_wasm() -> Result<Vec<u8>, Error> {
    use std::io::Read;
    let mut out = Vec::with_capacity(16 << 20);
    flate2::read::GzDecoder::new(SLANG_WASM_GZ)
        .read_to_end(&mut out)
        .map_err(map(Error::Setup))?;
    Ok(out)
}

/// An error from the wasm backend.
#[derive(Debug)]
pub enum Error {
    /// The wasm engine or module could not be set up.
    Setup(String),
    /// A trap or error occurred while calling into the module.
    Runtime(String),
    /// slang reported a failure (its message).
    Slang(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Setup(m) => write!(f, "wasm setup error: {m}"),
            Error::Runtime(m) => write!(f, "wasm runtime error: {m}"),
            Error::Slang(m) => write!(f, "slang error: {m}"),
        }
    }
}

impl std::error::Error for Error {}

fn map<E: fmt::Display>(kind: fn(String) -> Error) -> impl Fn(E) -> Error {
    move |e| kind(e.to_string())
}

/// The compiled wasm module and its engine, shared across the whole process so
/// the ~14 MB module is JIT-compiled only once. `Engine` and `Module` are
/// `Send + Sync`, so every `Slang::new`/[`Slang::fork`] after the first is cheap
/// — it only builds a fresh store and instance. Fuel accounting is enabled here
/// unconditionally so this one module serves both fuel-limited and unlimited
/// instances (the latter run with a `u64::MAX` budget); the per-instruction cost
/// is negligible next to recompiling the module.
fn shared_engine_module() -> Result<&'static (Engine, Module), Error> {
    init_shared(None)
}

/// Initializes the process-wide engine + module, once. With `precompiled` bytes
/// (from [`precompile`]) it deserializes native code directly — no compilation,
/// no cache directory needed — which is what makes a cold start on ephemeral or
/// read-only infrastructure (serverless, fresh containers) instant. Without
/// them it JIT-compiles the embedded module, using wasmtime's on-disk cache so
/// the compile is paid once per machine. The cell is set once by whichever path
/// runs first; a deserialize that fails validation (wrong target or wasmtime
/// version) falls back to compiling, so a stale `.cwasm` never breaks anything.
fn init_shared(precompiled: Option<&[u8]>) -> Result<&'static (Engine, Module), Error> {
    static CELL: OnceLock<Result<(Engine, Module), String>> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut config = Config::new();
        config.consume_fuel(true);
        // The on-disk cache only helps the compile path; deserialize is already
        // near-instant, so skip it there.
        if precompiled.is_none()
            && let Ok(cache) = Cache::from_file(None)
        {
            config.cache(Some(cache));
        }
        let engine = Engine::new(&config).map_err(|e| e.to_string())?;
        let compile = |engine: &Engine| -> Result<Module, String> {
            let wasm = decompress_wasm().map_err(|e| e.to_string())?;
            Module::from_binary(engine, &wasm).map_err(|e| e.to_string())
        };
        let module = match precompiled {
            // SAFETY: `bytes` are trusted to come from `precompile()` of this
            // crate built against this wasmtime version (the documented contract
            // of the `unsafe fn from_precompiled`). `deserialize` validates a
            // compatibility header and returns `Err` (not UB) on a version/target
            // mismatch, and we fall back to compiling on any error.
            Some(bytes) => match unsafe { Module::deserialize(&engine, bytes) } {
                Ok(m) => m,
                Err(_) => compile(&engine)?,
            },
            None => compile(&engine)?,
        };
        Ok((engine, module))
    })
    .as_ref()
    .map_err(|e| Error::Setup(e.clone()))
}

/// A loaded slang-in-wasm instance. Cheap to keep; one per thread.
pub struct Slang {
    store: Store<Host>,
    instance: Instance,
    memory: Memory,
    limits: Limits,
}

// Guest (wasm32) layout constants.
const SLANG_STR_SIZE: u32 = 12; // { data:i32, len:i32, owner:i32 }
const SLANG_ERROR_SIZE: u32 = 252; // { status:i32, message:[u8;248] }
const SLANG_NODE_SIZE: u32 = 16; // { ptr:i32, tree:i32, kind:u32, reserved:u32 }

impl Slang {
    /// Loads the embedded slang wasm module with the [default](Limits::default)
    /// resource limits and initializes it.
    pub fn new() -> Result<Slang, Error> {
        Slang::with_limits(Limits::default())
    }

    /// Loads the module with explicit resource [`Limits`]. Use this (with a
    /// fresh instance per input) to parse untrusted SystemVerilog safely: a
    /// runaway input traps on fuel or memory exhaustion instead of hanging the
    /// host thread or exhausting host memory.
    pub fn with_limits(limits: Limits) -> Result<Slang, Error> {
        // Reuse the process-wide compiled module (see `shared_engine_module`);
        // only the store and instance are per-`Slang`.
        let (engine, module) = shared_engine_module()?;
        Self::instantiate(engine, module, limits)
    }

    /// Precompiles the embedded module to portable native code (a wasmtime
    /// `.cwasm`) for the current target, returning the bytes to embed. Intended
    /// for a build script: run it once at build time, save the bytes, and load
    /// them with [`from_precompiled`](Slang::from_precompiled) so a cold start
    /// does no compilation — the answer for serverless / read-only deployments
    /// where the on-disk cache never persists between runs.
    ///
    /// The bytes are specific to this crate's wasmtime version and the build
    /// host's target; regenerate them whenever either changes.
    pub fn precompile() -> Result<Vec<u8>, Error> {
        let (_engine, module) = shared_engine_module()?;
        module.serialize().map_err(map(Error::Setup))
    }

    /// Loads the sandbox from precompiled bytes produced by
    /// [`precompile`](Slang::precompile), skipping compilation entirely — a cold
    /// start becomes a fast `deserialize` with no cache directory required. If
    /// the bytes are incompatible (a different wasmtime version or target) it
    /// transparently falls back to compiling the embedded module, so a stale
    /// artifact never breaks the sandbox.
    ///
    /// # Safety
    /// `cwasm` must be the unmodified output of [`precompile`](Slang::precompile)
    /// from the same crate version and wasmtime build; deserializing arbitrary or
    /// corrupted bytes is undefined behavior (wasmtime runs the embedded native
    /// code). A version/target mismatch is detected and rejected safely.
    pub unsafe fn from_precompiled(cwasm: &[u8], limits: Limits) -> Result<Slang, Error> {
        let (engine, module) = init_shared(Some(cwasm))?;
        Self::instantiate(engine, module, limits)
    }

    /// Builds a fresh store and instance over an already-loaded engine/module.
    fn instantiate(engine: &Engine, module: &Module, limits: Limits) -> Result<Slang, Error> {
        let mut linker: Linker<Host> = Linker::new(engine);
        wasmtime_wasi::p1::add_to_linker_sync(&mut linker, |h: &mut Host| &mut h.wasi)
            .map_err(map(Error::Setup))?;

        // The guest DFA trampoline calls back out to this import; it drives the
        // host-side lattice ([`DfaRun`]) for the in-flight `reaching_writes` run.
        linker
            .func_wrap(
                "env",
                "dfa_dispatch",
                |mut caller: Caller<'_, Host>, op: i32, user: i32, a: i32, b: i32| -> i32 {
                    // Read any struct the callback references out of guest memory
                    // before touching the (mutably-borrowed) run: for a transfer
                    // (op 5) the event's `slang_ast` at `b`; for an observer
                    // (ops 7/8/9) the observed statement's `slang_ast` at `a`.
                    let (ev_kind, ev) = if op == 5 || (7..=9).contains(&op) {
                        match caller.get_export("memory").and_then(|e| e.into_memory()) {
                            Some(mem) => {
                                let data = mem.data(&caller);
                                let base = if op == 5 { b as usize } else { a as usize };
                                let rd = |off: usize| -> u32 {
                                    match data.get(base + off..base + off + 4) {
                                        Some(s) => u32::from_le_bytes([s[0], s[1], s[2], s[3]]),
                                        None => 0,
                                    }
                                };
                                if op == 5 {
                                    // slang_dfa_event { kind:u32@0, symbol:slang_ast@4 }
                                    (rd(0), [rd(4), rd(8), rd(12), rd(16)])
                                } else {
                                    // slang_ast { ptr@0, comp@4, kind@8, domain@12 }
                                    (0, [rd(0), rd(4), rd(8), rd(12)])
                                }
                            }
                            None => (0, [0; 4]),
                        }
                    } else {
                        (0, [0; 4])
                    };
                    match caller.data_mut().dfa.as_mut() {
                        Some(run) => run.dispatch(op, user as u32, a as u32, b as u32, ev_kind, ev),
                        None => 0,
                    }
                },
            )
            .map_err(map(Error::Setup))?;

        let mut lb = StoreLimitsBuilder::new();
        if let Some(m) = limits.max_memory {
            lb = lb.memory_size(m);
        }
        let host = Host {
            wasi: WasiCtxBuilder::new().build_p1(),
            limits: lb.build(),
            dfa: None,
        };
        let mut store = Store::new(engine, host);
        store.limiter(|h| &mut h.limits);
        // Fuel is always accounted (see `shared_engine_module`); an unlimited
        // instance runs with the maximum budget.
        store
            .set_fuel(limits.fuel.unwrap_or(u64::MAX))
            .map_err(map(Error::Setup))?;

        let instance = linker
            .instantiate(&mut store, module)
            .map_err(map(Error::Setup))?;

        // Reactor modules initialize through `_initialize` rather than `_start`.
        if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "_initialize") {
            init.call(&mut store, ()).map_err(map(Error::Runtime))?;
        }

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| Error::Setup("module has no exported memory".into()))?;

        Ok(Slang {
            store,
            instance,
            memory,
            limits,
        })
    }

    /// Creates another independent sandbox instance with the same [`Limits`],
    /// reusing the process-wide compiled module — a cheap way to get a second
    /// worker (e.g. to parse files in parallel on other threads) without
    /// recompiling the ~14 MB module. Each `Slang` is fully isolated; nothing is
    /// shared but the read-only compiled code.
    ///
    /// ```
    /// let a = sv_lang_wasm::Slang::new()?;
    /// let mut b = a.fork()?;
    /// let tree = b.parse("module m; endmodule\n")?;
    /// assert_eq!(b.module_count(&tree)?, 1);
    /// # Ok::<(), sv_lang_wasm::Error>(())
    /// ```
    pub fn fork(&self) -> Result<Slang, Error> {
        Slang::with_limits(self.limits)
    }

    fn func0_i32(&mut self, name: &str) -> Result<i32, Error> {
        let f: TypedFunc<(), i32> = self
            .instance
            .get_typed_func(&mut self.store, name)
            .map_err(map(Error::Runtime))?;
        f.call(&mut self.store, ()).map_err(map(Error::Runtime))
    }

    fn call(&mut self, name: &str, args: &[Val]) -> Result<Vec<Val>, Error> {
        let func = self
            .instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| Error::Runtime(format!("no export `{name}`")))?;
        let ty = func.ty(&self.store);
        let mut results = vec![Val::I32(0); ty.results().len()];
        func.call(&mut self.store, args, &mut results)
            .map_err(map(Error::Runtime))?;
        Ok(results)
    }

    fn malloc(&mut self, len: u32) -> Result<u32, Error> {
        let out = self.call("malloc", &[Val::I32(len as i32)])?;
        match out.first() {
            Some(Val::I32(p)) if *p != 0 => Ok(*p as u32),
            _ => Err(Error::Runtime("malloc failed".into())),
        }
    }

    fn free(&mut self, ptr: u32) {
        let _ = self.call("free", &[Val::I32(ptr as i32)]);
    }

    fn read(&self, ptr: u32, len: u32) -> Result<Vec<u8>, Error> {
        let data = self.memory.data(&self.store);
        let (start, end) = (ptr as usize, ptr as usize + len as usize);
        data.get(start..end)
            .map(<[u8]>::to_vec)
            .ok_or_else(|| Error::Runtime("out-of-bounds read".into()))
    }

    fn read_u32(&self, ptr: u32) -> Result<u32, Error> {
        let data = self.memory.data(&self.store);
        let start = ptr as usize;
        let b = data
            .get(start..start + 4)
            .ok_or_else(|| Error::Runtime("out-of-bounds read".into()))?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn write(&mut self, ptr: u32, bytes: &[u8]) -> Result<(), Error> {
        let data = self.memory.data_mut(&mut self.store);
        let (start, end) = (ptr as usize, ptr as usize + bytes.len());
        data.get_mut(start..end)
            .ok_or_else(|| Error::Runtime("out-of-bounds write".into()))?
            .copy_from_slice(bytes);
        Ok(())
    }

    /// Copies a Rust string into freshly `malloc`d guest memory; returns
    /// `(ptr, len)`. Caller frees `ptr`.
    fn push_str(&mut self, s: &str) -> Result<(u32, u32), Error> {
        if s.is_empty() {
            return Ok((0, 0));
        }
        let ptr = self.malloc(s.len() as u32)?;
        self.write(ptr, s.as_bytes())?;
        Ok((ptr, s.len() as u32))
    }

    /// Reads a NUL-terminated C string from guest memory.
    fn read_cstr(&self, ptr: u32) -> Result<String, Error> {
        if ptr == 0 {
            return Ok(String::new());
        }
        let data = self.memory.data(&self.store);
        let tail = data
            .get(ptr as usize..)
            .ok_or_else(|| Error::Runtime("out-of-bounds read".into()))?;
        let len = tail.iter().position(|&b| b == 0).unwrap_or(tail.len());
        Ok(String::from_utf8_lossy(&tail[..len]).into_owned())
    }

    /// Reads a `slang_str` struct at `ptr` (data:i32, len:i32, owner:i32) into a
    /// Rust `String`, then frees it through `slang_str_free` if it is owned.
    fn take_slang_str(&mut self, ptr: u32) -> Result<String, Error> {
        let data = self.read_u32(ptr)?;
        let len = self.read_u32(ptr + 4)?;
        let s = if data == 0 || len == 0 {
            String::new()
        } else {
            String::from_utf8_lossy(&self.read(data, len)?).into_owned()
        };
        // Free the owned string (slang_str_free is a no-op for borrowed ones).
        // A struct passed by value uses the wasm indirect ABI: a pointer to the
        // struct. `ptr` already points at the slang_str, so pass it directly.
        let _ = self.call("slang_str_free", &[Val::I32(ptr as i32)]);
        Ok(s)
    }

    /// The slang version string.
    pub fn version(&mut self) -> String {
        // slang_version_string() -> const char* (a guest pointer to static data).
        match self.call("slang_version_string", &[]) {
            Ok(v) => match v.first() {
                Some(Val::I32(p)) => self.read_cstr(*p as u32).unwrap_or_default(),
                _ => String::new(),
            },
            Err(_) => String::new(),
        }
    }

    /// The number of syntax kinds slang knows.
    pub fn syntax_kind_count(&mut self) -> u32 {
        self.func0_i32("slang_syntax_kind_count").unwrap_or(0) as u32
    }

    /// The name of a syntax kind (e.g. `"ModuleDeclaration"`).
    ///
    /// `slang_syntax_kind_name` returns a `slang_str` by value, which on wasm32
    /// is passed through a hidden first pointer argument (the "sret" pointer).
    pub fn syntax_kind_name(&mut self, kind: u32) -> Result<String, Error> {
        let sret = self.malloc(SLANG_STR_SIZE)?;
        let r = self.call(
            "slang_syntax_kind_name",
            &[Val::I32(sret as i32), Val::I32(kind as i32)],
        );
        let out = r.and_then(|_| self.take_slang_str(sret));
        self.free(sret);
        out
    }

    /// Parses SystemVerilog source text and returns a handle to the tree.
    pub fn parse(&mut self, text: &str) -> Result<Tree, Error> {
        // slang_source_manager_create(slang_error* err) -> handle
        let err = self.malloc(SLANG_ERROR_SIZE)?;
        self.zero(err, SLANG_ERROR_SIZE)?;
        let sm = match self
            .call("slang_source_manager_create", &[Val::I32(err as i32)])?
            .first()
        {
            Some(Val::I32(p)) => *p as u32,
            _ => 0,
        };
        self.check_err(err)?;
        if sm == 0 {
            self.free(err);
            return Err(Error::Slang("failed to create source manager".into()));
        }

        // slang_syntax_tree_from_text(sm, text, tlen, name, nlen, path, plen, options, err)
        let (tp, tl) = self.push_str(text)?;
        let (np, nl) = self.push_str("source")?;
        self.zero(err, SLANG_ERROR_SIZE)?;
        let tree = match self
            .call(
                "slang_syntax_tree_from_text",
                &[
                    Val::I32(sm as i32),
                    Val::I32(tp as i32),
                    Val::I32(tl as i32),
                    Val::I32(np as i32),
                    Val::I32(nl as i32),
                    Val::I32(0),
                    Val::I32(0),
                    Val::I32(0), // options = null
                    Val::I32(err as i32),
                ],
            )?
            .first()
        {
            Some(Val::I32(p)) => *p as u32,
            _ => 0,
        };
        self.free(tp);
        self.free(np);
        let check = self.check_err(err);
        self.free(err);
        check?;

        if tree == 0 {
            return Err(Error::Slang("failed to parse".into()));
        }
        Ok(Tree { sm, tree })
    }

    fn zero(&mut self, ptr: u32, len: u32) -> Result<(), Error> {
        let data = self.memory.data_mut(&mut self.store);
        let (start, end) = (ptr as usize, ptr as usize + len as usize);
        data.get_mut(start..end)
            .ok_or_else(|| Error::Runtime("out-of-bounds write".into()))?
            .fill(0);
        Ok(())
    }

    /// Checks a `slang_error` at `ptr`; returns `Err(Slang(msg))` on failure.
    fn check_err(&self, ptr: u32) -> Result<(), Error> {
        let status = self.read_u32(ptr)? as i32;
        if status <= 0 {
            return Ok(());
        }
        let msg = self.read_cstr(ptr + 4).unwrap_or_default();
        Err(Error::Slang(msg))
    }
}

/// A parsed syntax tree living in the wasm instance.
pub struct Tree {
    #[allow(dead_code)] // the owning source manager; kept alive with the instance
    sm: u32,
    tree: u32,
}

impl Slang {
    /// The kind name of a tree's root node (e.g. `"CompilationUnit"`).
    pub fn root_kind_name(&mut self, tree: &Tree) -> Result<String, Error> {
        // slang_syntax_tree_root(tree) -> slang_node (16 bytes, sret).
        let sret = self.malloc(SLANG_NODE_SIZE)?;
        self.call(
            "slang_syntax_tree_root",
            &[Val::I32(sret as i32), Val::I32(tree.tree as i32)],
        )?;
        // slang_node { ptr:i32, tree:i32, kind:u32, reserved:u32 } — kind at offset 8.
        let kind = self.read_u32(sret + 8)?;
        self.free(sret);
        self.syntax_kind_name(kind)
    }

    /// The number of top-level module declarations (walks the tree's children).
    ///
    /// A `slang_node` is a 16-byte struct; on wasm32 it is passed by value using
    /// the indirect ABI — a pointer to the struct in linear memory. So the node
    /// lives at a guest pointer throughout, and functions taking it by value get
    /// that pointer.
    pub fn module_count(&mut self, tree: &Tree) -> Result<u32, Error> {
        let module_struct = self.module_declaration_kind()?;

        // root -> a 16-byte node written to `root_ptr` (sret).
        let root_ptr = self.malloc(SLANG_NODE_SIZE)?;
        self.call(
            "slang_syntax_tree_root",
            &[Val::I32(root_ptr as i32), Val::I32(tree.tree as i32)],
        )?;

        // child_count(node) — node passed indirectly as `root_ptr`.
        let count = match self
            .call("slang_node_child_count", &[Val::I32(root_ptr as i32)])?
            .first()
        {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        };

        let mut modules = 0;
        for i in 0..count {
            let node_out = self.malloc(SLANG_NODE_SIZE)?;
            let tok_out = self.malloc(24)?;
            // slang_node_child(node, index, node_out, token_out) -> child tag.
            let tag = match self
                .call(
                    "slang_node_child",
                    &[
                        Val::I32(root_ptr as i32),
                        Val::I32(i as i32),
                        Val::I32(node_out as i32),
                        Val::I32(tok_out as i32),
                    ],
                )?
                .first()
            {
                Some(Val::I32(t)) => *t,
                _ => 0,
            };
            if tag == 1 {
                // SLANG_CHILD_NODE: kind is at offset 8 in the node struct.
                if self.read_u32(node_out + 8)? == module_struct {
                    modules += 1;
                }
            }
            self.free(node_out);
            self.free(tok_out);
        }
        self.free(root_ptr);
        Ok(modules)
    }

    fn module_declaration_kind(&mut self) -> Result<u32, Error> {
        // The ordinal is a property of the (process-wide, shared) compiled
        // module's kind table, so scan for it once and cache it.
        static KIND: OnceLock<Option<u32>> = OnceLock::new();
        if let Some(cached) = KIND.get() {
            return cached.ok_or_else(|| Error::Slang("ModuleDeclaration kind not found".into()));
        }
        let count = self.syntax_kind_count();
        let mut found = None;
        for k in 0..count {
            if self.syntax_kind_name(k)? == "ModuleDeclaration" {
                found = Some(k);
                break;
            }
        }
        let _ = KIND.set(found);
        found.ok_or_else(|| Error::Slang("ModuleDeclaration kind not found".into()))
    }
}

// ---- Semantic layer: compilation, symbols, types, and the AST tree ----------

const SLANG_AST_SIZE: u32 = 16;
const FREEZE_ALL: u32 = 7; // ELABORATE_ALL | PREFOLD | SEAL
const DISABLE_INSTANCE_CACHING: u32 = 1 << 11;

/// A guest `slang_ast` value (16 bytes: `{ptr, compilation, kind, domain}`).
#[derive(Clone, Copy, Debug)]
struct Ast {
    ptr: u32,
    comp: u32,
    kind: u32,
    domain: u32,
}

impl Ast {
    fn is_null(self) -> bool {
        self.ptr == 0
    }
}

/// An elaborated, frozen design inside the sandbox.
pub struct Design {
    comp: u32,
    #[allow(dead_code)]
    sm: u32,
}

/// An opaque handle to a semantic node — a symbol, type, statement, or
/// expression — inside a [`Design`].
#[derive(Clone, Copy, Debug)]
pub struct Node(Ast);

impl Slang {
    // -- guest slang_ast marshalling --

    fn read_ast(&self, p: u32) -> Result<Ast, Error> {
        Ok(Ast {
            ptr: self.read_u32(p)?,
            comp: self.read_u32(p + 4)?,
            kind: self.read_u32(p + 8)?,
            domain: self.read_u32(p + 12)?,
        })
    }

    fn write_ast(&mut self, a: Ast) -> Result<u32, Error> {
        let p = self.malloc(SLANG_AST_SIZE)?;
        self.write(p, &a.ptr.to_le_bytes())?;
        self.write(p + 4, &a.comp.to_le_bytes())?;
        self.write(p + 8, &a.kind.to_le_bytes())?;
        self.write(p + 12, &a.domain.to_le_bytes())?;
        Ok(p)
    }

    /// Calls `fn(slang_ast, extra...) -> scalar`, passing the ast indirectly.
    fn call_ast(&mut self, name: &str, a: Ast, extra: &[Val]) -> Result<Vec<Val>, Error> {
        let ap = self.write_ast(a)?;
        let mut args = vec![Val::I32(ap as i32)];
        args.extend_from_slice(extra);
        let r = self.call(name, &args);
        self.free(ap);
        r
    }

    /// Calls `fn(slang_ast, extra..., [err]) -> slang_ast`: sret pointer first,
    /// then the indirect ast pointer, then extra args, then an optional error.
    fn call_ast_to_ast(
        &mut self,
        name: &str,
        a: Ast,
        extra: &[Val],
        with_err: bool,
    ) -> Result<Ast, Error> {
        let sret = self.malloc(SLANG_AST_SIZE)?;
        let ap = self.write_ast(a)?;
        let err = if with_err {
            let e = self.malloc(SLANG_ERROR_SIZE)?;
            self.zero(e, SLANG_ERROR_SIZE)?;
            e
        } else {
            0
        };
        let mut args = vec![Val::I32(sret as i32), Val::I32(ap as i32)];
        args.extend_from_slice(extra);
        if with_err {
            args.push(Val::I32(err as i32));
        }
        let called = self.call(name, &args);
        self.free(ap);
        let out = called.and_then(|_| {
            if with_err {
                self.check_err(err)?;
            }
            self.read_ast(sret)
        });
        if with_err {
            self.free(err);
        }
        self.free(sret);
        out
    }

    /// Calls `fn(slang_ast, [err]) -> slang_str` (sret first).
    fn call_ast_to_str(&mut self, name: &str, a: Ast, with_err: bool) -> Result<String, Error> {
        let sret = self.malloc(SLANG_STR_SIZE)?;
        let ap = self.write_ast(a)?;
        let err = if with_err {
            let e = self.malloc(SLANG_ERROR_SIZE)?;
            self.zero(e, SLANG_ERROR_SIZE)?;
            e
        } else {
            0
        };
        let mut args = vec![Val::I32(sret as i32), Val::I32(ap as i32)];
        if with_err {
            args.push(Val::I32(err as i32));
        }
        let called = self.call(name, &args);
        self.free(ap);
        let out = called.and_then(|_| self.take_slang_str(sret));
        if with_err {
            self.free(err);
        }
        self.free(sret);
        out
    }

    // -- public semantic API --

    /// Elaborates and freezes a parsed tree into a [`Design`] — the semantic
    /// layer, running entirely inside the sandbox.
    pub fn compile(&mut self, tree: &Tree) -> Result<Design, Error> {
        let err = self.malloc(SLANG_ERROR_SIZE)?;
        self.zero(err, SLANG_ERROR_SIZE)?;

        // Options with DisableInstanceCaching (required for full totalization).
        let options = match self
            .call("slang_options_create", &[Val::I32(err as i32)])?
            .first()
        {
            Some(Val::I32(p)) => *p as u32,
            _ => 0,
        };
        self.call(
            "slang_options_set_compilation_flags",
            &[
                Val::I32(options as i32),
                Val::I32(DISABLE_INSTANCE_CACHING as i32),
            ],
        )?;

        self.zero(err, SLANG_ERROR_SIZE)?;
        let comp = match self
            .call(
                "slang_compilation_create",
                &[Val::I32(options as i32), Val::I32(err as i32)],
            )?
            .first()
        {
            Some(Val::I32(p)) => *p as u32,
            _ => 0,
        };
        self.check_err(err)?;

        self.zero(err, SLANG_ERROR_SIZE)?;
        self.call(
            "slang_compilation_add_tree",
            &[
                Val::I32(comp as i32),
                Val::I32(tree.tree as i32),
                Val::I32(err as i32),
            ],
        )?;
        self.check_err(err)?;

        self.zero(err, SLANG_ERROR_SIZE)?;
        self.call(
            "slang_compilation_freeze",
            &[
                Val::I32(comp as i32),
                Val::I32(FREEZE_ALL as i32),
                Val::I32(0), // report = null
                Val::I32(err as i32),
            ],
        )?;
        let check = self.check_err(err);
        self.free(err);
        check?;

        Ok(Design { comp, sm: tree.sm })
    }

    /// The top-level module instances of a design.
    pub fn top_instances(&mut self, design: &Design) -> Result<Vec<Node>, Error> {
        let count = match self
            .call(
                "slang_compilation_top_instance_count",
                &[Val::I32(design.comp as i32)],
            )?
            .first()
        {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        };
        let mut out = Vec::new();
        for i in 0..count {
            let sret = self.malloc(SLANG_AST_SIZE)?;
            self.call(
                "slang_compilation_top_instance",
                &[
                    Val::I32(sret as i32),
                    Val::I32(design.comp as i32),
                    Val::I32(i as i32),
                ],
            )?;
            let a = self.read_ast(sret)?;
            self.free(sret);
            if !a.is_null() {
                out.push(Node(a));
            }
        }
        Ok(out)
    }

    /// A symbol's name.
    pub fn name(&mut self, node: Node) -> Result<String, Error> {
        self.call_ast_to_str("slang_symbol_name", node.0, false)
    }

    /// A node's kind name (e.g. `"Instance"`, `"BinaryExpression"`).
    pub fn kind_name(&mut self, node: Node) -> Result<String, Error> {
        let sret = self.malloc(SLANG_STR_SIZE)?;
        let r = self.call(
            "slang_ast_kind_name",
            &[
                Val::I32(sret as i32),
                Val::I32(node.0.domain as i32),
                Val::I32(node.0.kind as i32),
            ],
        );
        let out = r.and_then(|_| self.take_slang_str(sret));
        self.free(sret);
        out
    }

    /// The instance body (scope) of a module/interface/program instance.
    pub fn instance_body(&mut self, node: Node) -> Result<Option<Node>, Error> {
        let a = self.call_ast_to_ast("slang_instance_body", node.0, &[], false)?;
        Ok((!a.is_null()).then_some(Node(a)))
    }

    /// The direct members of a scope symbol, in declaration order.
    pub fn members(&mut self, node: Node) -> Result<Vec<Node>, Error> {
        let first = self.call_ast_to_ast("slang_scope_first_member", node.0, &[], true)?;
        let mut out = Vec::new();
        let mut cur = first;
        while !cur.is_null() {
            out.push(Node(cur));
            cur = self.call_ast_to_ast("slang_symbol_next_sibling", cur, &[], false)?;
        }
        Ok(out)
    }

    /// Finds a direct member by name.
    pub fn find(&mut self, node: Node, member: &str) -> Result<Option<Node>, Error> {
        let (np, nl) = self.push_str(member)?;
        let a = self.call_ast_to_ast(
            "slang_scope_find",
            node.0,
            &[Val::I32(np as i32), Val::I32(nl as i32)],
            true,
        );
        self.free(np);
        let a = a?;
        Ok((!a.is_null()).then_some(Node(a)))
    }

    /// The declared type of a value symbol.
    pub fn value_type(&mut self, node: Node) -> Result<Option<Node>, Error> {
        // Mirror the native backend: a non-value symbol has no type (rather than
        // being an error), so guard on `is_value` before asking for the type.
        if !self.is_value(node)? {
            return Ok(None);
        }
        let a = self.call_ast_to_ast("slang_value_type", node.0, &[], true)?;
        Ok((!a.is_null()).then_some(Node(a)))
    }

    /// Whether a symbol is a value symbol (has a type and can appear in an
    /// expression) — a net, variable, port, parameter, enum value, and so on.
    pub fn is_value(&mut self, node: Node) -> Result<bool, Error> {
        let r = self.call_ast("slang_symbol_is_value", node.0, &[])?;
        Ok(matches!(r.first(), Some(Val::I32(n)) if *n != 0))
    }

    /// A type's bit width.
    pub fn type_bit_width(&mut self, node: Node) -> Result<u64, Error> {
        let r = self.call_ast("slang_type_bit_width", node.0, &[])?;
        Ok(match r.first() {
            Some(Val::I64(n)) => *n as u64,
            Some(Val::I32(n)) => *n as u64,
            _ => 0,
        })
    }

    /// A type printed as SystemVerilog (e.g. `"logic[7:0]"`).
    pub fn type_string(&mut self, node: Node) -> Result<String, Error> {
        self.call_ast_to_str("slang_type_to_string", node.0, true)
    }

    /// The root statement of a procedural block or subroutine symbol.
    pub fn body(&mut self, node: Node) -> Result<Option<Node>, Error> {
        let a = self.call_ast_to_ast("slang_symbol_body", node.0, &[], false)?;
        Ok((!a.is_null()).then_some(Node(a)))
    }

    /// The immediate semantic children of a statement or expression node.
    ///
    /// Uses the bulk `slang_ast_sem_children` (which collects the children once
    /// and fills a caller buffer) rather than the per-child accessor, whose
    /// re-collection on every index makes an N-child node cost O(N²). The guest
    /// returns the true total, so a node wider than the initial guess is filled
    /// by a single retry with the exact capacity.
    pub fn sem_children(&mut self, node: Node) -> Result<Vec<Node>, Error> {
        let mut cap = 16u32;
        loop {
            let out = self.malloc(cap * SLANG_AST_SIZE)?;
            // `call_ast` marshals the node handle indirectly (and frees it); the
            // `out` buffer is ours to allocate and read back.
            let r = self.call_ast(
                "slang_ast_sem_children",
                node.0,
                &[Val::I32(out as i32), Val::I32(cap as i32)],
            );
            let total = match r {
                Ok(vals) => match vals.first() {
                    Some(Val::I32(n)) => *n as u32,
                    _ => 0,
                },
                Err(e) => {
                    self.free(out);
                    return Err(e);
                }
            };
            if total > cap {
                // The buffer was too small; retry once with the exact size.
                self.free(out);
                cap = total;
                continue;
            }
            let mut children = Vec::with_capacity(total as usize);
            for i in 0..total {
                let a = self.read_ast(out + i * SLANG_AST_SIZE)?;
                if !a.is_null() {
                    children.push(Node(a));
                }
            }
            self.free(out);
            return Ok(children);
        }
    }

    /// The binary-operator ordinal of a `BinaryOp` expression (0 otherwise).
    pub fn binary_op(&mut self, node: Node) -> Result<u32, Error> {
        let r = self.call_ast("slang_expr_binary_op", node.0, &[])?;
        Ok(match r.first() {
            Some(Val::I32(n)) => *n as u32,
            _ => 0,
        })
    }

    /// Runs a built-in reaching-writes dataflow analysis over a procedure symbol
    /// (an `always`/`initial`/`final` block or a subroutine) **entirely inside
    /// the sandbox**, returning the names of the value symbols written on the
    /// procedure's exit path.
    ///
    /// This exercises slang's custom DataFlow/Lattice bridge across the wasm
    /// boundary: slang runs the analysis in the guest and drives the lattice by
    /// calling back out to the host through the `env.dfa_dispatch` import, where
    /// the abstract state (a set of written symbols) lives in Rust.
    pub fn reaching_writes(
        &mut self,
        design: &Design,
        procedure: Node,
    ) -> Result<Vec<String>, Error> {
        let exit = self.run_lattice::<WriteSetLattice>(design, procedure)?;

        let mut names = Vec::new();
        for ast in exit.0.into_values() {
            let node = Node(Ast {
                ptr: ast[0],
                comp: ast[1],
                kind: ast[2],
                domain: ast[3],
            });
            names.push(self.name(node)?);
        }
        names.sort();
        names.dedup();
        Ok(names)
    }

    /// Runs an arbitrary caller-defined [`WasmLattice`] over a procedure symbol
    /// inside the sandbox and returns its exit state — the generic form of
    /// [`reaching_writes`](Self::reaching_writes), and the wasm analogue of the
    /// native `sv_lang::dataflow` bridge.
    ///
    /// slang drives the analysis in the guest, calling out to the host through
    /// `env.dfa_dispatch` for every lattice operation (`top`/`bottom`/`clone`/
    /// `join`/`meet`/`transfer`); the states live host-side in Rust as `L`.
    pub fn run_lattice<L: WasmLattice>(
        &mut self,
        design: &Design,
        procedure: Node,
    ) -> Result<L, Error> {
        self.store.data_mut().dfa = Some(DfaRun::new::<L>());

        let err = self.malloc(SLANG_ERROR_SIZE)?;
        self.zero(err, SLANG_ERROR_SIZE)?;
        let ap = self.write_ast(procedure.0)?;
        let out = self.call(
            "slang_wasm_dfa_run",
            &[
                Val::I32(design.comp as i32),
                Val::I32(ap as i32),
                Val::I32(0), // user (unused; state lives in host data)
                Val::I32(err as i32),
            ],
        );
        self.free(ap);
        let exit_id = match &out {
            Ok(v) => match v.first() {
                Some(Val::I32(id)) => *id as u32,
                _ => 0,
            },
            Err(_) => 0,
        };
        let check = self.check_err(err);
        self.free(err);

        // Take the run out (ending the analysis) before extracting the state.
        let mut run = self.store.data_mut().dfa.take();
        out?;
        check?;

        run.as_mut()
            .and_then(|r| r.take_state::<L>(exit_id))
            .ok_or_else(|| Error::Slang("dataflow produced no exit state".into()))
    }
}

impl Drop for Slang {
    fn drop(&mut self) {
        // The whole instance is dropped; guest memory goes with it, so there is
        // nothing to free explicitly.
    }
}

impl Tree {
    /// The raw guest handle (for advanced use).
    pub fn raw(&self) -> u32 {
        self.tree
    }
}
