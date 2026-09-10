// A Compilation (still building) must not be Sync.
fn assert_sync<T: Sync>() {}
fn main() {
    assert_sync::<sv_lang::Compilation>();
}
