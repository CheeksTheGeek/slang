// An EvalSession grants interior mutation through &self and must not be Sync.
fn assert_sync<T: Sync>() {}
fn main() {
    assert_sync::<sv_lang::EvalSession<'static>>();
}
