# sv-lang API coverage

This file is the honest, reproducible answer to "how much of slang's public C++
API do these bindings reach?" — produced by `scripts/api_coverage.py`, which is
wired into CI as `python3 scripts/api_coverage.py --check`.

It exists because an earlier claim — "full semantic surface EXHAUSTIVELY bound,
857/857 Class::member targets / covers the whole slang C++ API" — was **not
backed by any script or committed artifact**. It was an unfalsifiable boast, and
a downstream request for `Compilation::getAttributes` (added in 0.1.10) exposed
that it was simply missing, with no coverage gate that could have caught it.
This replaces the boast with a number that can fail.

## Current numbers (slang 12.0)

Run `python3 scripts/api_coverage.py` to regenerate.

| bucket | count |
|---|---|
| Public `SLANG_EXPORT` member functions (targets) | 2532 |
| Bound by name (class-aware; see below) | 508 |
| Classified by structural rule (not a read-accessor surface) | 1880 |
| Residual, listed in `scripts/api_coverage_allow.txt` | 144 |
| **Unclassified gaps** (the gate fails if > 0) | **0** |
| **Stale allow-list entries** (the gate fails if > 0) | **0** |

So: the bindings bind **~508** public methods by name. The "857/857 = whole
API" figure was never a real denominator — the public member surface is ~2532,
most of which is deliberately **not** mirrored 1:1 (see below).

### Correction (2026-10-13): the first version of this gate over-counted

The first version of this file reported **717** bound methods. That number was
itself inflated: its "bound by name" matcher compared snake-case method names
**globally**, so `DefParamSymbol::getValue` counted as bound because *some*
unrelated `*_value` function existed, and every class with a `getName` matched
any `name`. Making the matcher class-aware dropped the true count to 445, and
surfaced ~30 real gaps that the global match had hidden. After adding the
family-scoped rule below and explicit `slang::…::method` references on generic
accessors, the honest figure is **508**. The lesson is the same one this file
exists for: a coverage number must be computed by a matcher that can't be
satisfied by coincidence.

## How "bound by name" is decided

A target `Class::method` counts as bound only if one of these holds:

1. **Explicit mirror** — the C header or Rust docs reference it verbatim as
   `slang::ast::Class::method` (or `slang::Class::method` for classes in the bare
   `slang` namespace). Generic accessors that serve several classes carry one
   reference per class they cover (e.g. `Symbol::constant_value` names
   `ParameterSymbol::getValue`, `EnumValueSymbol::getValue` and
   `SpecparamSymbol::getValue`).
2. **Class-tokened C function** — a `slang_*` C function whose name ends with the
   method and whose prefix carries the class token (`DefParamSymbol::getTarget`
   ↔ `slang_symbol_defparam_target`).
3. **Same-family Rust method** — a Rust method with exactly the method's name (or
   its `as_` view) on the class's own Rust handle. The bindings collapse each AST
   family into one handle (every `*Expression` is an `Expression`), so
   `BinaryExpression::left` is bound as `Expression::left`. Never a bare suffix
   match.

## What "classified by structural rule" means

`api_coverage.py` auto-classifies methods that are legitimately not part of the
frozen-design read surface a binding mirrors:

- **`syntax::` layer** — reached via generic node reflection (`member_*`/child),
  not per-method accessors.
- **`parsing::` / `util::` / `text::`** — parser and infrastructure internals.
- **factories / builders / mutators** — `fromSyntax`, `create*`, `add*`,
  `build*`, `set*` (a frozen design is read-only).
- **evaluation / binding / check internals** — `bind*`, `eval*`, `check*`,
  `require*`, `*Impl`, `isKind`, `*SlowCase`, `propagateType`, …
- **predicates and protocol** — `bad` (covered by `kind() == Invalid` / the
  `Bad` variant), `isEvaluating` (always false once frozen), `equals`/`next`
  (equality/iterator internals), `asSymbol` (Rust scope handles are already
  `Symbol`s).
- **infrastructure classes** — `Compilation`, `DiagnosticEngine`,
  `SourceManager`, `ASTContext`, `EvalContext`, `Bitstream`, `OpInfo`, … (bound
  selectively where it matters, never 1:1).

The classification rules live in the script and are reviewable; tune the rules
or the allow-list, not the parser, to resolve a misclassification.

## The residual backlog (`scripts/api_coverage_allow.txt`)

144 public methods are neither bound nor caught by a structural rule. Each is
tagged in the allow-list:

- **`TODO-BIND`** (~100) — read accessors worth exposing; the real backlog.
  Examples: `ClassType::getBaseClass` / `hasCycles`, `PortSymbol::getNetRanges`,
  `InstanceSymbol::isTopLevel`, `GenericClassDefSymbol::specializations`,
  `ConfigBlockSymbol::getCellOverrides` / `getTopCells`, `NetType::getDataType`,
  `EnumType::findDefinition`, `CaseStatement::getKnownBranch`.
  Six of them (`MinTypMaxExpression::min/typ/max`, `CallExpression::arguments`,
  `CopyClassExpression::sourceExpr`, `MemberAccessExpression::value`) are
  already **reachable via `children()`**; only a named accessor is missing.
- **niche / internal / triage** — rarely-needed helpers, structural-equivalence
  checks on non-expression families, and the diagnostics model (surfaced through
  `sv-lang-kinds`' generated tables rather than slang's `DiagGroup`/`DiagCode`).

These are a reviewed ledger, not a hidden gap: listing one here is a deliberate
"not bound, and here's why".

## The gate

`scripts/api_coverage.py --check` fails in **both** directions:

- a public method that is neither bound, rule-classified, nor allow-listed (a
  new gap — e.g. a slang version bump adding an accessor nobody triaged), and
- an allow-list entry that is no longer a target or is now bound /
  rule-classified (a stale ledger that would overstate the backlog).

Both directions are exercised: inserting a bound method into the allow-list and
removing a residual entry each make `--check` exit 1.

## Honest limitations

This is a **heuristic** C++ scan (no libclang), so:

- "Bound by name" is a careful lower bound, not an exact figure: a binding whose
  Rust/C name carries neither the class token nor an explicit `slang::…`
  reference will show up as a residual until it is annotated.
- The target parser can still miss or over-match an unusual declaration; it errs
  toward *reporting* a method (as a residual to classify) rather than hiding it.

When precision matters more than the gate, the next step is a libclang-based
target parser and a curated C++↔Rust name map.
