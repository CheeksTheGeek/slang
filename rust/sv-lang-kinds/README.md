# sv-lang-kinds

Kind enumerations for [slang](https://sv-lang.com), the SystemVerilog compiler frontend:
`SyntaxKind`, `SyntaxStruct`, `TokenKind`, `TriviaKind`, `DiagSubsystem`, `DiagCode` and
the named diagnostic constants in `diag::*`.

Plain Rust, `no_std`, no dependencies, no native code. Generated from slang's own schema
files so discriminants match the C++ enums exactly. Depend on this crate when you only
need to *name* kinds; depend on `sv-lang` when you need the compiler itself.

```rust
use sv_lang_kinds::{SyntaxKind, diag};

assert_eq!(SyntaxKind::from_raw(0), Some(SyntaxKind::Unknown));
assert_eq!(diag::UnknownModule.name(), "UnknownModule");
```

Regenerate after a slang update with `cargo xtask codegen` from the `rust/` workspace.
