//! Small helpers over the raw `sv-lang-sys` surface: error translation and
//! string ownership. Nothing here is exported.

use core::ffi::c_char;

use sv_lang_sys as sys;

use crate::Error;

/// A fresh, cleared error record.
pub(crate) fn error() -> sys::slang_error {
    sys::slang_error::INIT
}

/// Translates a C error record into a [`Result`]. Warnings (negative status)
/// are treated as success.
pub(crate) fn check(err: &sys::slang_error) -> Result<(), Error> {
    if err.status <= sys::SLANG_OK {
        return Ok(());
    }
    Err(Error::from_status(err.status, message(err)))
}

/// The human-readable message from an error record.
pub(crate) fn message(err: &sys::slang_error) -> String {
    // SAFETY: `message` is a NUL-terminated buffer written by the library.
    let bytes = unsafe { core::ffi::CStr::from_ptr(err.message.as_ptr()) };
    bytes.to_string_lossy().into_owned()
}

/// Copies a borrowed `slang_str` into an owned `String`. Does not free it
/// (borrowed strings have no owner; this must not be called on an owned one
/// without also freeing).
pub(crate) fn borrowed_str(s: sys::slang_str) -> String {
    if s.data.is_null() || s.len == 0 {
        return String::new();
    }
    // SAFETY: `data`/`len` describe valid UTF-8 bytes owned by a live slang object.
    let slice = unsafe { core::slice::from_raw_parts(s.data.cast::<u8>(), s.len) };
    String::from_utf8_lossy(slice).into_owned()
}

/// Copies an owned `slang_str` into a `String` and frees the original.
pub(crate) fn owned_str(s: sys::slang_str) -> String {
    let out = borrowed_str(s);
    // SAFETY: taking ownership of a string the library allocated for us.
    unsafe { sys::slang_str_free(s) };
    out
}

/// A borrowed `slang_str` as a `&str` without copying, valid for `'a`.
///
/// # Safety
/// The bytes must remain valid and borrowed for all of `'a` (i.e. the owning
/// slang object outlives `'a`), and must be valid UTF-8 (slang identifiers and
/// source text are).
pub(crate) unsafe fn str_ref<'a>(s: sys::slang_str) -> &'a str {
    if s.data.is_null() || s.len == 0 {
        return "";
    }
    // SAFETY: guaranteed by the caller's contract.
    let slice = unsafe { core::slice::from_raw_parts(s.data.cast::<u8>(), s.len) };
    core::str::from_utf8(slice).unwrap_or("")
}

/// Passes a `&str` to a C function as a (ptr, len) pair. Empty strings pass a
/// null pointer with zero length, which the API accepts.
pub(crate) fn as_ptr_len(s: &str) -> (*const c_char, usize) {
    if s.is_empty() {
        (core::ptr::null(), 0)
    } else {
        (s.as_ptr().cast::<c_char>(), s.len())
    }
}
