// A Symbol handle must not outlive the Design it borrows.
fn main() {
    let session = sv_lang::Session::new();
    let sym = {
        let mut comp = sv_lang::Compilation::new(&session).unwrap();
        comp.add_source("module m; endmodule\n").unwrap();
        let design = comp.compile().unwrap();
        design.root()
    };
    let _ = sym.kind();
}
