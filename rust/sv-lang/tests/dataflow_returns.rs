//! `EvalSession::run_dataflow_including_returns`: the exit state also covers
//! paths that leave through `return` (or a task-level `disable`).
//!
//! slang's flow engine parks the state at a `return` and marks the rest of the
//! path unreachable, so plain `run_dataflow` reports only the paths that fall
//! off the end of the body. Requested by the Carcilator session: a "may"
//! analysis otherwise has to re-accumulate every write inside `transfer`.

use std::collections::BTreeSet;

use sv_lang::dataflow::{DfaEvent, DfaEventKind, Lattice};
use sv_lang::{Compilation, Session};

/// Every variable some path writes (union at joins; bottom is the empty set,
/// which is the identity for union).
#[derive(Clone, Debug, Default)]
struct MayWrite(BTreeSet<String>);

impl Lattice for MayWrite {
    fn top() -> Self {
        MayWrite::default()
    }
    fn join(&mut self, other: &Self) {
        self.0.extend(other.0.iter().cloned());
    }
    fn transfer(&mut self, ev: DfaEvent<'_>) {
        if ev.kind == DfaEventKind::Write
            && let Some(s) = ev.symbol
        {
            self.0.insert(s.name().to_string());
        }
    }
}

/// Variables written on EVERY path (intersection at joins); bottom is the
/// all-variables universe, the identity for intersection.
#[derive(Clone, Debug)]
struct MustWrite {
    names: BTreeSet<String>,
    unreachable: bool,
}

impl Lattice for MustWrite {
    fn top() -> Self {
        MustWrite {
            names: BTreeSet::new(),
            unreachable: false,
        }
    }
    fn bottom() -> Self {
        MustWrite {
            names: BTreeSet::new(),
            unreachable: true,
        }
    }
    fn join(&mut self, other: &Self) {
        if other.unreachable {
            return;
        }
        if self.unreachable {
            *self = other.clone();
            return;
        }
        self.names.retain(|k| other.names.contains(k));
    }
    fn transfer(&mut self, ev: DfaEvent<'_>) {
        if ev.kind == DfaEventKind::Write
            && let Some(s) = ev.symbol
        {
            self.names.insert(s.name().to_string());
        }
    }
}

/// Compiles `src` and runs `f` on the subroutine named `name` in `m`'s body.
fn with_subroutine<R>(
    src: &str,
    name: &str,
    f: impl FnOnce(&sv_lang::EvalSession<'_>, sv_lang::Symbol<'_>) -> R,
) -> R {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    let mut design = comp.compile().unwrap();
    let dfa = design.eval_session();
    let body = dfa
        .design()
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let sub = body.find(name).unwrap();
    f(&dfa, sub)
}

fn names(m: &MayWrite) -> Vec<&str> {
    m.0.iter().map(String::as_str).collect()
}

#[test]
fn early_return_inside_an_if_keeps_the_writes_before_it() {
    // Carcilator's case.
    let src = "module m; int a, b, d;\n\
               function automatic void f(input bit c);\n\
               \x20 a = 1;\n\
               \x20 if (c) begin b = 2; return; end\n\
               \x20 d = 3;\n\
               endfunction\nendmodule\n";
    with_subroutine(src, "f", |dfa, f| {
        let plain: MayWrite = dfa.run_dataflow(f).unwrap();
        assert_eq!(names(&plain), ["a", "d"], "plain run loses the return path");
        let all: MayWrite = dfa.run_dataflow_including_returns(f).unwrap();
        assert_eq!(names(&all), ["a", "b", "d"]);
    });
}

#[test]
fn every_path_ending_in_return_is_no_longer_bottom() {
    let src = "module m; int a, b, d;\n\
               function automatic int f(input bit c);\n\
               \x20 a = 1;\n\
               \x20 if (c) begin b = 2; return 1; end\n\
               \x20 d = 3;\n\
               \x20 return 0;\n\
               endfunction\nendmodule\n";
    with_subroutine(src, "f", |dfa, f| {
        let plain: MayWrite = dfa.run_dataflow(f).unwrap();
        assert!(plain.0.is_empty(), "plain run is bottom: {plain:?}");
        let all: MayWrite = dfa.run_dataflow_including_returns(f).unwrap();
        assert_eq!(names(&all), ["a", "b", "d"]);
    });
}

#[test]
fn a_body_without_return_is_unchanged() {
    let src = "module m; int a, b;\n\
               function automatic void f(input bit c);\n\
               \x20 a = 1;\n\
               \x20 if (c) b = 2;\n\
               endfunction\nendmodule\n";
    with_subroutine(src, "f", |dfa, f| {
        let plain: MayWrite = dfa.run_dataflow(f).unwrap();
        let all: MayWrite = dfa.run_dataflow_including_returns(f).unwrap();
        assert_eq!(names(&plain), names(&all));
        assert_eq!(names(&all), ["a", "b"]);
    });
}

#[test]
fn a_task_level_disable_is_a_return() {
    let src = "module m; int a, b, d;\n\
               task automatic t(input bit c);\n\
               \x20 a = 1;\n\
               \x20 if (c) begin b = 2; disable t; end\n\
               \x20 d = 3;\n\
               endtask\nendmodule\n";
    with_subroutine(src, "t", |dfa, t| {
        let plain: MayWrite = dfa.run_dataflow(t).unwrap();
        assert!(!plain.0.contains("b"), "{plain:?}");
        let all: MayWrite = dfa.run_dataflow_including_returns(t).unwrap();
        assert_eq!(names(&all), ["a", "b", "d"]);
    });
}

#[test]
fn a_break_needs_no_flag() {
    // The break state is merged at the loop's end, so plain already sees it.
    let src = "module m; int a, b, d;\n\
               function automatic void f(input bit c);\n\
               \x20 for (int i = 0; i < 4; i++) begin\n\
               \x20   a = 1;\n\
               \x20   if (c) begin b = 2; break; end\n\
               \x20   d = 3;\n\
               \x20 end\n\
               endfunction\nendmodule\n";
    with_subroutine(src, "f", |dfa, f| {
        let plain: MayWrite = dfa.run_dataflow(f).unwrap();
        let all: MayWrite = dfa.run_dataflow_including_returns(f).unwrap();
        assert!(plain.0.contains("b"), "{plain:?}");
        assert_eq!(names(&plain), names(&all));
    });
}

#[test]
fn a_must_analysis_is_unsound_without_the_flag() {
    // `b` is not assigned on the early-return path. The plain run only sees the
    // fall-through path and wrongly claims `b` is definitely assigned.
    let src = "module m; int a, b;\n\
               function automatic void f(input bit c);\n\
               \x20 a = 1;\n\
               \x20 if (c) return;\n\
               \x20 b = 2;\n\
               endfunction\nendmodule\n";
    with_subroutine(src, "f", |dfa, f| {
        let plain: MustWrite = dfa.run_dataflow(f).unwrap();
        assert!(plain.names.contains("b"), "plain overclaims: {plain:?}");
        let all: MustWrite = dfa.run_dataflow_including_returns(f).unwrap();
        assert!(all.names.contains("a"), "{all:?}");
        assert!(!all.names.contains("b"), "b skipped on the return: {all:?}");
    });
}

/// A lattice whose `join` panics: the run must come back as an error, never
/// abort or unwind through the C library, including the extra joins the flag
/// performs.
#[derive(Clone, Debug)]
struct PanicsOnJoin;

impl Lattice for PanicsOnJoin {
    fn top() -> Self {
        PanicsOnJoin
    }
    fn join(&mut self, _other: &Self) {
        panic!("join is not supported");
    }
    fn transfer(&mut self, _ev: DfaEvent<'_>) {}
}

#[test]
fn a_panicking_join_is_an_error_not_an_abort() {
    let src = "module m; int a;\n\
               function automatic void f(input bit c);\n\
               \x20 if (c) return;\n\
               \x20 a = 1;\n\
               endfunction\nendmodule\n";
    with_subroutine(src, "f", |dfa, f| {
        let r: Result<PanicsOnJoin, _> = dfa.run_dataflow_including_returns(f);
        assert!(r.is_err());
    });
}
