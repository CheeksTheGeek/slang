// A syntax Node must not outlive the SyntaxTree it borrows.
fn main() {
    let session = sv_lang::Session::new();
    let node = {
        let tree = session.parse("module m; endmodule\n").unwrap();
        tree.root()
    };
    let _ = node.kind();
}
