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

## Current numbers (slang 11.0)

Run `python3 scripts/api_coverage.py` to regenerate.

| bucket | count |
|---|---|
| Public `SLANG_EXPORT` member functions (targets) | 2535 |
| Bound by name (C ABI / Rust `Mirrors slang::…` / snake-case rename) | 717 |
| Classified by structural rule (not a read-accessor surface) | 1701 |
| Residual, listed in `scripts/api_coverage_allow.txt` | 117 |
| **Unclassified gaps** (the gate fails if > 0) | **0** |

So: the bindings bind ~717 public methods by name. The "857/857 = whole API"
figure was never a real denominator — the actual public member surface is ~2535,
most of which is deliberately **not** mirrored 1:1 (see below).

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
- **infrastructure classes** — `Compilation`, `DiagnosticEngine`,
  `SourceManager`, `ASTContext`, `EvalContext`, `Bitstream`, `OpInfo`, … (bound
  selectively where it matters, never 1:1).

The classification rules live in the script and are reviewable; tune the rules
or the allow-list, not the parser, to resolve a misclassification.

## The residual backlog (`scripts/api_coverage_allow.txt`)

117 public methods are neither bound nor caught by a structural rule. Each is
tagged in the allow-list:

- **`TODO-BIND candidate accessor`** (~57) — genuine read accessors worth
  exposing. Real backlog. Examples: `ClassType::getBaseClass` /
  `getDeclaredInterfaces` / `hasCycles`, `PortSymbol::getNetRanges` /
  `getNetTypes` / `hasInitializer`, `InstanceSymbol::isTopLevel` /
  `getCanonicalBody`, `GenericClassDefSymbol::specializations`,
  `ConfigBlockSymbol::getCellOverrides` / `getInstanceOverrides`,
  `EnumType::findDefinition`, `CaseStatement::getKnownBranch`,
  `PackageSymbol::resolveExports`, `SpecparamSymbol::getPulse{Reject,Error}Limit`.
- **`triage`** (~60) — needs a human decision; includes **false-negatives**
  already reachable under a renamed form (e.g. `isNonBlocking` is bound as
  `is_nonblocking`; `getCanonicalType` as `canonical`), plus niche internals.

These are a reviewed ledger, not a hidden gap: listing one here is a deliberate
"not bound, and here's why" — and the gate still fails on anything *not* listed.

## The gate

`scripts/api_coverage.py --check` exits non-zero if any public method is neither
bound, structurally classified, nor in the allow-list. On a slang version bump,
new public methods that nobody has triaged will fail CI — which is exactly the
tripwire that was missing when `getAttributes` fell through.

## Honest limitations

This is a **heuristic** C++ scan (no libclang), so:

- "Bound by name" can over- or under-count across the C/Rust snake_case rename
  boundary; it is a lower bound on true functional reachability, not an exact
  figure.
- The target parser can still miss or over-match an unusual declaration; it errs
  toward *reporting* a method (as a residual to classify) rather than hiding it.

The number that matters is **unclassified gaps = 0 with a reviewed 117-entry
backlog**, not a single "coverage %". When precision matters more than the gate,
the next step is a libclang-based target parser and a curated C++↔Rust name map.
