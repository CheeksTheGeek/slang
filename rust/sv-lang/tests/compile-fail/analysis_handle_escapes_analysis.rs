// A live analysis handle (here a ValueDriver from `driver_handles`) must not
// outlive the `Analysis` it borrows. The handle's C++ backing is owned by the
// AnalysisManager, which is freed on `Analysis::drop` (slang_analysis_destroy);
// letting the Copy handle escape the analysis borrow would be a use-after-free.
use sv_lang::AnalysisFlags;

fn main() {
    let session = sv_lang::Session::new();
    let mut comp = sv_lang::Compilation::new(&session).unwrap();
    comp.add_source("module m(input logic a); logic z; assign z = a; endmodule\n")
        .unwrap();
    let design = comp.compile().unwrap();
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();
    let z = body.find("z").unwrap();
    let d = {
        let a = design.analyze(AnalysisFlags::NONE, 1).unwrap();
        a.driver_handles(z).next().unwrap()
    };
    let _ = d.symbol();
}
