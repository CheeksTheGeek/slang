//! Structured constant values — the folded/evaluated value of an expression as
//! owned Rust data, not just a printed string.
//!
//! Obtain one from [`Expression::constant_value`](crate::Expression::constant_value)
//! (the already-folded value, a pure read) or
//! [`EvalSession::eval_constant`](crate::EvalSession::eval_constant).

use sv_lang_sys as sys;

use crate::ffi;

/// A four-state bit: `0`, `1`, `X` (unknown) or `Z` (high-impedance).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bit {
    /// Logic 0.
    Zero,
    /// Logic 1.
    One,
    /// Unknown.
    X,
    /// High-impedance.
    Z,
}

impl Bit {
    fn from_raw(v: u8) -> Bit {
        match v {
            0 => Bit::Zero,
            1 => Bit::One,
            2 => Bit::X,
            _ => Bit::Z,
        }
    }
}

impl core::fmt::Display for Bit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Bit::Zero => "0",
            Bit::One => "1",
            Bit::X => "x",
            Bit::Z => "z",
        })
    }
}

/// An arbitrary-width, four-state SystemVerilog integer (slang's `SVInt`),
/// reconstructed as owned Rust data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SVInt {
    bit_width: u32,
    is_signed: bool,
    has_unknown: bool,
    as_i64: Option<i64>,
    as_u64: Option<u64>,
    /// Per-bit values, LSB-first. Populated for widths up to a cap; empty for
    /// very wide values (use [`to_string`](Self::to_string) then).
    bits: Vec<Bit>,
    text: String,
}

/// Widths above this are not expanded bit-by-bit (still available via string).
const MAX_BITS: u32 = 1024;

impl SVInt {
    /// The declared bit width.
    pub fn bit_width(&self) -> u32 {
        self.bit_width
    }
    /// Whether it is signed.
    pub fn is_signed(&self) -> bool {
        self.is_signed
    }
    /// Whether it contains any unknown (x/z) bits.
    pub fn has_unknown(&self) -> bool {
        self.has_unknown
    }
    /// The value as an `i64`, or `None` if it has unknown bits or does not fit.
    pub fn as_i64(&self) -> Option<i64> {
        self.as_i64
    }
    /// The value as a `u64`, or `None` if it has unknown bits or does not fit.
    pub fn as_u64(&self) -> Option<u64> {
        self.as_u64
    }
    /// The i'th bit (LSB = 0), or `None` if out of range or not expanded.
    pub fn bit(&self, index: usize) -> Option<Bit> {
        self.bits.get(index).copied()
    }
    /// The expanded per-bit values, LSB-first (empty for very wide values).
    pub fn bits(&self) -> &[Bit] {
        &self.bits
    }

    /// SAFETY: `v` must be a valid `slang_svint` borrowed from a live constant.
    unsafe fn from_raw(v: sys::slang_svint) -> SVInt {
        // SAFETY: all these read `v` which the caller keeps alive.
        unsafe {
            let bit_width = sys::slang_svint_bit_width(v);
            let is_signed = sys::slang_svint_is_signed(v);
            let has_unknown = sys::slang_svint_has_unknown(v);
            let mut i = 0i64;
            let as_i64 = sys::slang_svint_as_i64(v, &mut i).then_some(i);
            let mut u = 0u64;
            let as_u64 = sys::slang_svint_as_u64(v, &mut u).then_some(u);
            let bits = if bit_width <= MAX_BITS {
                (0..bit_width)
                    .map(|b| Bit::from_raw(sys::slang_svint_get_bit(v, b)))
                    .collect()
            } else {
                Vec::new()
            };
            let text = ffi::owned_str(sys::slang_svint_to_string(v));
            SVInt {
                bit_width,
                is_signed,
                has_unknown,
                as_i64,
                as_u64,
                bits,
                text,
            }
        }
    }
}

impl core::fmt::Display for SVInt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.text)
    }
}

impl TryFrom<&SVInt> for i64 {
    type Error = ();
    fn try_from(v: &SVInt) -> Result<i64, ()> {
        v.as_i64.ok_or(())
    }
}

/// A structured constant value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ConstantValue {
    /// An integer.
    Integer(SVInt),
    /// A real (`real` or `shortreal`).
    Real(f64),
    /// A string.
    String(String),
    /// The null class handle.
    Null,
    /// The unbounded `$` value.
    Unbounded,
    /// An unpacked array or queue.
    Array(Vec<ConstantValue>),
    /// An associative array (not expanded here; use the string form).
    Map(String),
    /// A tagged union (not expanded here; use the string form).
    Union(String),
    /// Not a constant / could not be evaluated.
    Bad,
}

impl ConstantValue {
    /// This value as an integer, if it is one.
    pub fn as_integer(&self) -> Option<&SVInt> {
        match self {
            ConstantValue::Integer(v) => Some(v),
            _ => None,
        }
    }
    /// This value as an `i64`, if it is a fitting integer.
    pub fn as_i64(&self) -> Option<i64> {
        self.as_integer().and_then(SVInt::as_i64)
    }
    /// This value as an `f64`, if it is real.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConstantValue::Real(r) => Some(*r),
            _ => None,
        }
    }
    /// This value as a string, if it is one.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConstantValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Reconstructs from a raw owned `slang_constant`, freeing it.
    ///
    /// SAFETY: `raw` must be a valid owned handle (or null); it is consumed.
    pub(crate) unsafe fn from_raw(raw: sys::slang_constant) -> Option<ConstantValue> {
        if raw.is_null() {
            return None;
        }
        // SAFETY: `raw` is a valid owned handle; freed before returning.
        let value = unsafe { Self::read(raw) };
        // SAFETY: we own `raw` and are done reading it.
        unsafe { sys::slang_constant_destroy(raw) };
        Some(value)
    }

    /// SAFETY: `raw` valid and live for the duration of the call.
    unsafe fn read(raw: sys::slang_constant) -> ConstantValue {
        // SAFETY: `raw` is valid.
        let kind = unsafe { sys::slang_constant_kind_of(raw) };
        match kind {
            sys::SLANG_CONSTANT_INTEGER => {
                // SAFETY: integer handle borrows `raw`, alive here.
                let v = unsafe { sys::slang_constant_integer(raw) };
                if v.is_null() {
                    ConstantValue::Bad
                } else {
                    // SAFETY: `v` valid, `raw` alive.
                    ConstantValue::Integer(unsafe { SVInt::from_raw(v) })
                }
            }
            sys::SLANG_CONSTANT_REAL | sys::SLANG_CONSTANT_SHORTREAL => {
                let mut d = 0.0f64;
                // SAFETY: out-param provided.
                unsafe { sys::slang_constant_real(raw, &mut d) };
                ConstantValue::Real(d)
            }
            sys::SLANG_CONSTANT_STRING => {
                // SAFETY: returns an owned/borrowed slang_str.
                ConstantValue::String(unsafe { ffi::owned_str(sys::slang_constant_string(raw)) })
            }
            sys::SLANG_CONSTANT_NULL => ConstantValue::Null,
            sys::SLANG_CONSTANT_UNBOUNDED => ConstantValue::Unbounded,
            sys::SLANG_CONSTANT_UNPACKED | sys::SLANG_CONSTANT_QUEUE => {
                // SAFETY: `raw` valid.
                let n = unsafe { sys::slang_constant_size(raw) };
                let mut out = Vec::with_capacity(n as usize);
                for i in 0..n {
                    let mut err = ffi::error();
                    // SAFETY: element returns a new owned handle or null.
                    let e = unsafe { sys::slang_constant_element(raw, i, &mut err) };
                    // SAFETY: `e` is a valid owned handle (or null), consumed.
                    if let Some(v) = unsafe { ConstantValue::from_raw(e) } {
                        out.push(v);
                    }
                }
                ConstantValue::Array(out)
            }
            sys::SLANG_CONSTANT_MAP => {
                // SAFETY: `raw` is valid; returns an owned slang_str.
                ConstantValue::Map(unsafe { ffi::owned_str(sys::slang_constant_to_string(raw)) })
            }
            sys::SLANG_CONSTANT_UNION => {
                // SAFETY: `raw` is valid; returns an owned slang_str.
                ConstantValue::Union(unsafe { ffi::owned_str(sys::slang_constant_to_string(raw)) })
            }
            _ => ConstantValue::Bad,
        }
    }
}

impl core::fmt::Display for ConstantValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConstantValue::Integer(v) => write!(f, "{v}"),
            ConstantValue::Real(r) => write!(f, "{r}"),
            ConstantValue::String(s) => write!(f, "\"{s}\""),
            ConstantValue::Null => f.write_str("null"),
            ConstantValue::Unbounded => f.write_str("$"),
            ConstantValue::Array(els) => {
                f.write_str("'{")?;
                for (i, e) in els.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{e}")?;
                }
                f.write_str("}")
            }
            ConstantValue::Map(s) | ConstantValue::Union(s) => f.write_str(s),
            ConstantValue::Bad => f.write_str("<bad>"),
        }
    }
}
