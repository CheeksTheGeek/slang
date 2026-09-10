//! A codemod: rename every occurrence of an identifier in a file, preserving
//! all formatting, and verify the result still parses.
//!
//!     cargo run --example rename -- old_name new_name design.sv
//!
//! Prints the rewritten source to stdout (does not modify the file).

use std::process::ExitCode;

use sv_lang::Session;
use sv_lang::green::SyntaxEditor;

const DEMO: &str = "\
module m;
  logic old_sig;
  assign old_sig = 1'b0;   // old_sig in a comment is NOT renamed
endmodule
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (from, to, src) = match args.as_slice() {
        [from, to, path] => match std::fs::read_to_string(path) {
            Ok(s) => (from.clone(), to.clone(), s),
            Err(e) => {
                eprintln!("error reading {path}: {e}");
                return ExitCode::FAILURE;
            }
        },
        [from, to] => (from.clone(), to.clone(), DEMO.to_string()),
        _ => {
            eprintln!("usage: rename <from> <to> [file.sv]");
            return ExitCode::FAILURE;
        }
    };

    let session = Session::new();
    let tree = match session.parse(&src) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("parse error: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Rename identifier tokens only — not matching text in comments or strings.
    let mirror = tree.mirror();
    let mut editor = SyntaxEditor::new(&mirror);
    let mut count = 0;
    for token in mirror
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
    {
        if token.kind() == sv_lang::green::Kind::Token(sv_lang::kinds::TokenKind::Identifier)
            && token.text() == from
        {
            editor.replace(token, to.as_str());
            count += 1;
        }
    }
    let rewritten = editor.finish();

    // Confirm the codemod produced valid source.
    match session.parse(&rewritten) {
        Ok(t) if !t.diagnostics().has_errors() => {}
        _ => {
            eprintln!("warning: rewritten source has parse errors");
        }
    }

    eprintln!("renamed {count} occurrence(s) of `{from}` -> `{to}`");
    print!("{rewritten}");
    ExitCode::SUCCESS
}
