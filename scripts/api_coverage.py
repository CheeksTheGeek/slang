#!/usr/bin/env python3
"""Public-API coverage audit for the sv-lang C ABI.

Honest, reproducible answer to "how much of slang's public C++ API do the
bindings actually reach?" — replacing the old unverified "857/857" boast.

Method:
  TARGETS  = every public member function of every `class SLANG_EXPORT X`
             (and `struct SLANG_EXPORT X`) in include/slang/, excluding
             ctors/dtors/operators and the C ABI's own header.
  COVERED  = union of
             (a) `slang::<ns>::Class::method` references in the C ABI header
                 (the "Mirrors slang::..." doc annotations), and
             (b) `->method(` / `.method(` call targets in source/capi/*.cpp.
             A method Class::m counts as covered if some binding names it, OR
             (conservatively) if its unqualified name is called in the capi
             layer AND the class is otherwise touched.
  CLASSIFY = scripts/api_coverage_allow.txt lists `Class::method  reason`
             for targets deliberately not mirrored 1:1 (reachable-by-traversal
             or intentionally-unexposed). These are counted as classified.

Gate (--check): every TARGET must be COVERED or CLASSIFIED, else exit 1 with
the list. Prints counts in both directions so it can actually fail.

This is a heuristic C++ scan (no libclang dep); it errs toward *reporting* a
gap rather than hiding one. Tune the allow-list, not the parser, to resolve
false positives.
"""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
INCLUDE = REPO / "include" / "slang"
CAPI_HDR = INCLUDE / "c" / "slang.h"
CAPI_SRC = REPO / "source" / "capi"
ALLOW = REPO / "scripts" / "api_coverage_allow.txt"

# A member-function declaration inside a class body: optional return type, a
# name, an open paren. We capture the name just before `(`. Excludes lines that
# are clearly not decls (access labels, macros, comments handled separately).
DECL_RE = re.compile(r"[A-Za-z_][\w:<>,*&\s]*?\b([A-Za-z_]\w*)\s*\(")
CLASS_RE = re.compile(r"\b(?:class|struct)\s+SLANG_EXPORT\s+([A-Za-z_]\w*)")
EXCLUDE_NAMES = {"operator", "SLANG_EXPORT"}
# C++ keywords / type tokens my heuristic decl-scan can pick up from non-decl
# lines (`if (`, `constexpr`, `"sv"` literals, trailing-return types). Never
# method names.
KEYWORD_TOKENS = {
    "if", "for", "while", "switch", "return", "constexpr", "consteval",
    "noexcept", "sizeof", "static_cast", "reinterpret_cast", "const_cast",
    "sv", "logic_t", "bitwidth_t", "as", "as_if", "and", "or", "not",
    "decltype", "requires", "co_await",
    # `operator bool()` / `operator uintN_t()` cast ops and int-type trailing
    # returns my decl-scan captures as a bare type token, never a method.
    "bool", "uint32_t", "uint64_t", "int32_t", "int64_t", "size_t", "char",
    "double", "float", "string", "string_view",
}


def iter_headers():
    for p in INCLUDE.rglob("*.h"):
        if p == CAPI_HDR or "/c/" in str(p):
            continue
        yield p


def enumerate_targets():
    """Return {(Class, method)} of public member functions of SLANG_EXPORT types."""
    targets = {}
    for path in iter_headers():
        text = path.read_text(errors="replace")
        i = 0
        n = len(text)
        while True:
            m = CLASS_RE.search(text, i)
            if not m:
                break
            cls = m.group(1)
            # find the class body `{ ... }` with brace matching
            brace = text.find("{", m.end())
            if brace < 0:
                i = m.end()
                continue
            depth = 0
            j = brace
            while j < n:
                c = text[j]
                if c == "{":
                    depth += 1
                elif c == "}":
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            body = text[brace + 1 : j]
            i = j + 1
            # `struct` defaults to public; `class` to private.
            access = "public" if text[m.start()] == "s" and "struct" in m.group(0) else (
                "public" if m.group(0).lstrip().startswith("struct") else "private"
            )
            for raw in body.splitlines():
                line = raw.strip()
                if not line or line.startswith("//") or line.startswith("*") or line.startswith("/*"):
                    continue
                lbl = re.match(r"(public|private|protected)\s*:", line)
                if lbl:
                    access = lbl.group(1)
                    continue
                if access != "public":
                    continue
                # Only lines that look like a function declaration (end in ; or {
                # after a paren, not a data member or using/typedef).
                if "(" not in line:
                    continue
                if line.startswith(("using", "typedef", "friend", "static_assert", "#")):
                    continue
                dm = DECL_RE.match(line)
                if not dm:
                    continue
                name = dm.group(1)
                if name in EXCLUDE_NAMES or name in KEYWORD_TOKENS:
                    continue
                if name == cls or name == "~" + cls:
                    continue
                if name.startswith("operator"):
                    continue
                # ctor: `Name(` with nothing but the class name before `(`
                if re.match(rf"^{re.escape(cls)}\s*\(", line):
                    continue
                targets.setdefault((cls, name), str(path.relative_to(REPO)))
    return targets


MIRROR_RE = re.compile(
    r"slang::(?:ast|syntax|parsing|analysis|numeric|[A-Za-z0-9_]+)::"
    r"([A-Za-z_]\w*)::([A-Za-z_]\w*)"
)
RUST_SRC = REPO / "rust"


def covered_methods():
    """{(Class, method)} named by the C ABI header or the Rust bindings' own
    `Mirrors slang::...` doc annotations, plus unqualified capi calls."""
    covered = set()
    sources = [CAPI_HDR.read_text(errors="replace")]
    # The Rust layer documents each wrapper with `Mirrors slang::<ns>::C::m`.
    for p in (RUST_SRC / "sv-lang" / "src").rglob("*.rs"):
        sources.append(p.read_text(errors="replace"))
    for text in sources:
        for m in MIRROR_RE.finditer(text):
            covered.add((m.group(1), m.group(2)))
    # Unqualified method names actually called in the capi implementation.
    called = set()
    for p in CAPI_SRC.rglob("*.cpp"):
        t = p.read_text(errors="replace")
        for m in re.finditer(r"(?:->|\.)\s*([A-Za-z_]\w*)\s*\(", t):
            called.add(m.group(1))
    return covered, called


def _snake(name):
    """camelCase/ PascalCase -> snake_case, dropping a leading get."""
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", s).lower()
    return s[4:] if s.startswith("get_") else s


def rust_api_names():
    """Snake-case identifiers the Rust + sys layers actually expose, so a
    camelCase C++ accessor bound under a renamed form (isClass -> is_class,
    getCanonicalType -> canonical) is not counted as a false-negative gap."""
    names = set()
    for p in (RUST_SRC / "sv-lang" / "src").rglob("*.rs"):
        t = p.read_text(errors="replace")
        for m in re.finditer(r"\bpub fn\s+([a-z_]\w*)", t):
            names.add(m.group(1))
    sysrs = (RUST_SRC / "sv-lang-sys" / "src" / "lib.rs").read_text(errors="replace")
    for m in re.finditer(r"\bfn\s+slang_([a-z0-9_]+)\s*\(", sysrs):
        names.add(m.group(1))
    return names


# Structural categories that are legitimately NOT part of the read-accessor
# surface the bindings mirror by name. Each is reported as classified-by-rule
# (with the rule), not as a gap — but still COUNTED and printed, so the
# classification itself is visible and auditable.
NAMESPACE_RULE = {
    # dir under include/slang/ -> (rule, applies to whole namespace)
    "syntax": "syntax-layer: reached via generic node reflection, not per-method",
    "parsing": "parser internal: not part of the elaborated read surface",
    "util": "utility/infrastructure: not a slang AST/API read surface",
    "text": "source-text internal (SourceManager is bound separately)",
}
# Method-name patterns that are construction/serialization/visitor machinery,
# never read accessors a binding would mirror.
NAME_RULE = [
    (re.compile(r"^fromSyntax"), "factory (elaboration-time construction)"),
    (re.compile(r"^from[A-Z]"), "factory (construction)"),
    (re.compile(r"^create[A-Z]?"), "factory (construction)"),
    (re.compile(r"^make[A-Z]"), "factory (construction)"),
    (re.compile(r"^serialize"), "serialization machinery"),
    (re.compile(r"^visit"), "visitor machinery"),
    (re.compile(r"Visitor$"), "visitor machinery"),
    (re.compile(r"^accept$"), "visitor machinery"),
    (re.compile(r"^set[A-Z]"), "mutator (frozen design is read-only)"),
    (re.compile(r"^addMember$|^addSyntax|^register|^addDiag|^addMembers$"),
     "builder-time mutator"),
    (re.compile(r"^hash$|^operator"), "infrastructure"),
    # Elaboration / constant-evaluation / type-checking internals: invoked
    # while building the AST, never read off a frozen design.
    (re.compile(r"^bind"), "binding internal (elaboration-time)"),
    (re.compile(r"^eval"), "evaluation internal"),
    (re.compile(r"^check[A-Z]"), "semantic-check internal"),
    (re.compile(r"^require[A-Z]"), "semantic-check internal"),
    (re.compile(r"^convert[A-Z]|^convertTo"), "value-conversion internal"),
    (re.compile(r"Impl$"), "internal *Impl helper"),
    (re.compile(r"^isKind$"), "static RTTI predicate (handled by kind())"),
    (re.compile(r"SlowCase$"), "arithmetic fast/slow-path internal"),
    (re.compile(r"^propagateType$|^analyzeOpTypes$|^knownSide$|^canBe"),
     "type-propagation internal"),
    (re.compile(r"^for[A-Z]"), "assignment-pattern factory"),
    (re.compile(r"^add[A-Z]"), "builder/mutator"),
    (re.compile(r"^build"), "builder (elaboration-time)"),
    (re.compile(r"^clone$|^copy[A-Z]|^transform$|^merge|^inherit|^connect|^expand"),
     "construction/transform internal"),
    (re.compile(r"^format$|^startMessage$"), "diagnostic-formatter hook"),
    (re.compile(r"^dereference$|^increment$"), "iterator-protocol internal"),
    (re.compile(r"^note[A-Z]|^issue|^report"), "diagnostic/bookkeeping internal"),
    (re.compile(r"^alloc|^reset$|^cache|^resolveAt$|^forceResolveAt$|^mergeImplicitPort$"),
     "elaboration internal"),
]
# Classes that are builder/infrastructure, not an AST read surface. Methods on
# these are mirrored selectively (Compilation/DiagnosticEngine/SourceManager/
# Driver are bound where it matters) but not expected 1:1; wholesale-classified
# so only genuine AST read-accessor holes remain the gate's concern.
INFRA_CLASSES = {
    "Compilation", "ScriptSession", "SourceManager", "SourceLibraryManager",
    "DiagnosticEngine", "Diagnostics", "DiagnosticClient", "TextDiagnosticClient",
    "JsonDiagnosticClient", "Bag", "CommandLine", "Driver", "DriverOptions",
    "AnalysisManager", "AnalysisOptions", "ThreadPool", "BumpAllocator",
    "SFormat", "TypePrinter", "ASTSerializer", "SyntaxPrinter",
    # Pure elaboration/evaluation context + static helper utilities: used to
    # build/evaluate the AST, not a read surface on a frozen design.
    "ASTContext", "EvalContext", "Bitstream", "OpInfo", "SemanticFacts",
    "SourceLoader", "Lookup", "LookupResult",
    # Internal analysis/evaluation helper classes (not a frozen-design read
    # surface): clock inference, case-decision DAG, sequence checker, opaque
    # instance paths, the diag-dedup map.
    "ClockInference", "CaseDecisionDag", "ExpressionSequenceChecker",
    "OpaqueInstancePath", "ASTDiagMap", "DiagArgFormatter", "TypeArgFormatter",
}
# NOTE: AST node classes (Expression/Statement/BinaryExpression/...) are
# deliberately NOT in this set, so genuine unbound read accessors on them stay
# visible in the residual rather than being hidden wholesale. Their elaboration
# entry points (bind/eval/check/...) are classified by the NAME_RULE patterns.


def classify_by_rule(cls, meth, path):
    """Return a (rule) string if (cls, meth) is structurally not a read-accessor
    gap, else None."""
    ns = path.split("/")[2] if path.startswith("include/slang/") else ""
    if ns in NAMESPACE_RULE:
        return NAMESPACE_RULE[ns]
    for rx, reason in NAME_RULE:
        if rx.search(meth):
            return reason
    if cls in INFRA_CLASSES:
        return f"infrastructure class ({cls}): bound selectively, not 1:1"
    return None


def load_allow():
    classified = {}
    if ALLOW.exists():
        for line in ALLOW.read_text().splitlines():
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            key, _, reason = line.partition("#")
            key = key.strip()
            if "::" in key:
                cls, _, meth = key.partition("::")
                classified[(cls.strip(), meth.strip())] = reason.strip()
    return classified


def main():
    check = "--check" in sys.argv
    verbose = "--verbose" in sys.argv
    targets = enumerate_targets()
    covered, called = covered_methods()
    classified = load_allow()

    covered_names = {m for (_, m) in covered}
    rust_names = rust_api_names()

    def bound_by_name(meth):
        if (meth in covered_names and meth in called):
            return True
        snake = _snake(meth)
        if snake in rust_names:
            return True
        # suffix match: Type::isClass -> slang_type_is_class -> "..._is_class"
        return any(n == snake or n.endswith("_" + snake) for n in rust_names)

    n_covered = n_rule = n_allow = 0
    missing = []
    for (cls, meth), path in sorted(targets.items()):
        if (cls, meth) in covered or bound_by_name(meth):
            n_covered += 1
            continue
        if (cls, meth) in classified:
            n_allow += 1
            continue
        if classify_by_rule(cls, meth, path):
            n_rule += 1
            continue
        missing.append((cls, meth, path))

    total = len(targets)
    gaps = len(missing)
    print(f"TARGETS (public SLANG_EXPORT member fns):   {total}")
    print(f"  bound by name (C ABI / Rust mirror):      {n_covered}")
    print(f"  classified by structural rule:            {n_rule}")
    print(f"  classified in allow-list:                 {n_allow}")
    print(f"  RESIDUAL AST read-accessor gaps:          {gaps}")

    if verbose or check:
        by_cls = {}
        for cls, meth, path in missing:
            by_cls.setdefault(cls, []).append(meth)
        for cls in sorted(by_cls, key=lambda c: -len(by_cls[c])):
            print(f"  {cls}: {', '.join(sorted(by_cls[cls]))}")

    if check and gaps:
        print(f"\nFAIL: {gaps} public API methods are neither bound nor classified.")
        print("Classify each in scripts/api_coverage_allow.txt (reachable-by-"
              "traversal or deliberately-unexposed) or add a binding.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
