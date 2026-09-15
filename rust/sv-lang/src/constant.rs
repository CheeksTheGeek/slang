//! Structured constant values — the folded/evaluated value of an expression as
//! owned Rust data, not just a printed string.
//!
//! Obtain one from [`Expression::constant_value`](crate::Expression::constant_value)
//! (the already-folded value, a pure read) or
//! [`EvalSession::eval_constant`](crate::EvalSession::eval_constant).

use sv_lang_sys as sys;

use crate::Error;
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
    pub(crate) fn from_raw(v: u8) -> Bit {
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

// ---- slang::logic_t: four-state bitwise operators on `Bit` -----------------
//
// `Bit` (above) is this crate's safe representation of a single four-state
// value everywhere else in the API (`get_bit`, `reduction_and`, etc.), which
// all use the C API's compact 2-bit wire encoding (0/1/2=x/3=z). `logic_t`
// itself is a distinct wire type (`slang_logic_value`) that instead mirrors
// slang's own in-memory byte for the type (0/1, or the sentinels
// `X_VALUE = 0x80` / `Z_VALUE = 0x40`) — the same encoding `Digit::to_raw`
// above already uses for `SVInt::fromDigits`. The two encodings meet here:
// `Bit` stays the one public four-state type, and `slang_logic_value` exists
// only as this module's private wire format for calling into
// `slang::logic_t`'s own methods.
impl Bit {
    fn to_logic_raw(self) -> sys::slang_logic_value {
        sys::slang_logic_value {
            value: match self {
                Bit::Zero => 0,
                Bit::One => 1,
                Bit::X => 0x80,
                Bit::Z => 0x40,
            },
        }
    }

    fn from_logic_raw(v: sys::slang_logic_value) -> Bit {
        match v.value {
            0 => Bit::Zero,
            1 => Bit::One,
            0x80 => Bit::X,
            _ => Bit::Z,
        }
    }

    /// The raw four-state value byte slang itself stores — mirrors the
    /// `slang::logic_t::value` field (`0`/`1` for a known bit, `0x80` for
    /// unknown `x`, `0x40` for high-impedance `z`; note this differs from
    /// [`Bit`]'s own compact 2-bit wire encoding used elsewhere in this
    /// crate). A pure, allocation-free read.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(sv_lang::Bit::Zero.logic_t_value(), 0);
    /// assert_eq!(sv_lang::Bit::One.logic_t_value(), 1);
    /// assert_eq!(sv_lang::Bit::X.logic_t_value(), 0x80);
    /// assert_eq!(sv_lang::Bit::Z.logic_t_value(), 0x40);
    /// ```
    pub fn logic_t_value(&self) -> u8 {
        // SAFETY: `slang_logic_value` is a plain value struct; this is a
        // pure function of its argument.
        unsafe { sys::slang_logic_value_value(self.to_logic_raw()) }
    }

    /// The four-state unknown `x` constant — mirrors the static
    /// `slang::logic_t::x`. A pure function with no arguments.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(sv_lang::Bit::x(), sv_lang::Bit::X);
    /// ```
    pub fn x() -> Bit {
        // SAFETY: pure function with no arguments.
        Bit::from_logic_raw(unsafe { sys::slang_logic_value_x() })
    }

    /// The four-state high-impedance `z` constant — mirrors the static
    /// `slang::logic_t::z`. A pure function with no arguments.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(sv_lang::Bit::z(), sv_lang::Bit::Z);
    /// ```
    pub fn z() -> Bit {
        // SAFETY: pure function with no arguments.
        Bit::from_logic_raw(unsafe { sys::slang_logic_value_z() })
    }

    /// True if this bit is `X` or `Z` — mirrors `slang::logic_t::isUnknown`.
    /// A pure, allocation-free read.
    ///
    /// # Examples
    /// ```
    /// assert!(!sv_lang::Bit::One.is_unknown());
    /// assert!(sv_lang::Bit::X.is_unknown());
    /// assert!(sv_lang::Bit::Z.is_unknown());
    /// ```
    pub fn is_unknown(&self) -> bool {
        // SAFETY: `slang_logic_value` is a plain value struct; this is a
        // pure function of its argument.
        unsafe { sys::slang_logic_value_is_unknown(self.to_logic_raw()) }
    }
}

impl core::ops::BitAnd for Bit {
    type Output = Bit;

    /// Four-state bitwise AND — mirrors `slang::logic_t::operator&`.
    ///
    /// # Examples
    /// ```
    /// use sv_lang::Bit;
    /// assert_eq!(Bit::Zero & Bit::X, Bit::Zero);
    /// assert_eq!(Bit::One & Bit::One, Bit::One);
    /// assert_eq!(Bit::One & Bit::X, Bit::X);
    /// ```
    fn bitand(self, rhs: Bit) -> Bit {
        // SAFETY: `slang_logic_value` is a plain value struct; this is a
        // pure function of its arguments.
        Bit::from_logic_raw(unsafe {
            sys::slang_logic_value_and(self.to_logic_raw(), rhs.to_logic_raw())
        })
    }
}

impl core::ops::BitOr for Bit {
    type Output = Bit;

    /// Four-state bitwise OR — mirrors `slang::logic_t::operator|`.
    ///
    /// # Examples
    /// ```
    /// use sv_lang::Bit;
    /// assert_eq!(Bit::One | Bit::X, Bit::One);
    /// assert_eq!(Bit::Zero | Bit::Zero, Bit::Zero);
    /// assert_eq!(Bit::Zero | Bit::X, Bit::X);
    /// ```
    fn bitor(self, rhs: Bit) -> Bit {
        // SAFETY: `slang_logic_value` is a plain value struct; this is a
        // pure function of its arguments.
        Bit::from_logic_raw(unsafe {
            sys::slang_logic_value_or(self.to_logic_raw(), rhs.to_logic_raw())
        })
    }
}

impl core::ops::BitXor for Bit {
    type Output = Bit;

    /// Four-state bitwise XOR — mirrors `slang::logic_t::operator^`.
    ///
    /// # Examples
    /// ```
    /// use sv_lang::Bit;
    /// assert_eq!(Bit::One ^ Bit::Zero, Bit::One);
    /// assert_eq!(Bit::One ^ Bit::One, Bit::Zero);
    /// assert_eq!(Bit::One ^ Bit::X, Bit::X);
    /// ```
    fn bitxor(self, rhs: Bit) -> Bit {
        // SAFETY: `slang_logic_value` is a plain value struct; this is a
        // pure function of its arguments.
        Bit::from_logic_raw(unsafe {
            sys::slang_logic_value_xor(self.to_logic_raw(), rhs.to_logic_raw())
        })
    }
}

impl core::ops::Not for Bit {
    type Output = Bit;

    /// Four-state bitwise NOT (`slang::logic_t` defines `operator~` as
    /// logical negation: `X`/`Z` stay `X`, `0` and `1` invert) — mirrors
    /// `slang::logic_t::operator~`.
    ///
    /// # Examples
    /// ```
    /// use sv_lang::Bit;
    /// assert_eq!(!Bit::Zero, Bit::One);
    /// assert_eq!(!Bit::One, Bit::Zero);
    /// assert_eq!(!Bit::X, Bit::X);
    /// assert_eq!(!Bit::Z, Bit::X);
    /// ```
    fn not(self) -> Bit {
        // SAFETY: `slang_logic_value` is a plain value struct; this is a
        // pure function of its argument.
        Bit::from_logic_raw(unsafe { sys::slang_logic_value_not(self.to_logic_raw()) })
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
    /// Captured from `slang::ConstantValue::isTrue`/`isFalse` (via
    /// `slang_constant_is_true`/`_is_false`) at construction time, rather
    /// than re-derived from `bits`: those predicates are slang's own
    /// three-valued reduction-OR over the *full* value (see
    /// `SVInt::reductionOr`), which `bits` alone can't reproduce once a
    /// value is wider than the per-bit expansion cap above.
    is_true: bool,
    is_false: bool,
    leading_ones: u32,
    leading_zeros: u32,
    leading_unknowns: u32,
    leading_zs: u32,
    ones: u32,
    zeros: u32,
    xs: u32,
    zs: u32,
    active_bits: u32,
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
    /// SystemVerilog condition-truthiness of this value: true iff at least
    /// one bit is definitely 1 (unaffected by x/z bits elsewhere). Mirrors
    /// `slang::ConstantValue::isTrue` for the integer case.
    pub fn is_true(&self) -> bool {
        self.is_true
    }
    /// True iff this value is definitely all-zero (no x/z anywhere).
    /// Mirrors `slang::ConstantValue::isFalse` for the integer case.
    pub fn is_false(&self) -> bool {
        self.is_false
    }
    /// The number of leading (MSB-side) 1 bits. Does not treat unknown bits
    /// specially. Mirrors `slang::SVInt::countLeadingOnes`.
    pub fn count_leading_ones(&self) -> u32 {
        self.leading_ones
    }
    /// The number of leading (MSB-side) 0 bits. Does not treat unknown bits
    /// specially. Mirrors `slang::SVInt::countLeadingZeros`.
    pub fn count_leading_zeros(&self) -> u32 {
        self.leading_zeros
    }
    /// The number of leading (MSB-side) unknown (x or z) bits. Mirrors
    /// `slang::SVInt::countLeadingUnknowns`.
    pub fn count_leading_unknowns(&self) -> u32 {
        self.leading_unknowns
    }
    /// The number of leading (MSB-side) z bits. Mirrors
    /// `slang::SVInt::countLeadingZs`.
    pub fn count_leading_zs(&self) -> u32 {
        self.leading_zs
    }
    /// The total number of 1 bits. Mirrors `slang::SVInt::countOnes`.
    pub fn count_ones(&self) -> u32 {
        self.ones
    }
    /// The total number of 0 bits. Mirrors `slang::SVInt::countZeros`.
    pub fn count_zeros(&self) -> u32 {
        self.zeros
    }
    /// The total number of x bits. Mirrors `slang::SVInt::countXs`.
    pub fn count_xs(&self) -> u32 {
        self.xs
    }
    /// The total number of z bits. Mirrors `slang::SVInt::countZs`.
    pub fn count_zs(&self) -> u32 {
        self.zs
    }
    /// The number of "active bits": the bit width minus the number of
    /// leading zeros, i.e. the minimum width needed to hold the value
    /// ignoring sign and unknown bits. Mirrors `slang::SVInt::getActiveBits`.
    pub fn active_bits(&self) -> u32 {
        self.active_bits
    }

    /// SAFETY: `v` must be a valid `slang_svint` borrowed from a live
    /// constant; `is_true`/`is_false` must be `slang::ConstantValue::isTrue`/
    /// `isFalse` computed on that same constant.
    unsafe fn from_raw(v: sys::slang_svint, is_true: bool, is_false: bool) -> SVInt {
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
            let leading_ones = sys::slang_svint_count_leading_ones(v);
            let leading_zeros = sys::slang_svint_count_leading_zeros(v);
            let leading_unknowns = sys::slang_svint_count_leading_unknowns(v);
            let leading_zs = sys::slang_svint_count_leading_zs(v);
            let ones = sys::slang_svint_count_ones(v);
            let zeros = sys::slang_svint_count_zeros(v);
            let xs = sys::slang_svint_count_xs(v);
            let zs = sys::slang_svint_count_zs(v);
            let active_bits = sys::slang_svint_active_bits(v);
            SVInt {
                bit_width,
                is_signed,
                has_unknown,
                as_i64,
                as_u64,
                bits,
                text,
                is_true,
                is_false,
                leading_ones,
                leading_zeros,
                leading_unknowns,
                leading_zs,
                ones,
                zeros,
                xs,
                zs,
                active_bits,
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

/// The base of a number literal, for [`OwnedSVInt::from_digits`] — mirrors
/// `slang::LiteralBase`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiteralBase {
    /// Base 2 (`'b`).
    Binary,
    /// Base 8 (`'o`).
    Octal,
    /// Base 10 (`'d`), or an unsized decimal literal.
    Decimal,
    /// Base 16 (`'h`).
    Hex,
}

impl LiteralBase {
    fn to_raw(self) -> u8 {
        match self {
            LiteralBase::Binary => 0,
            LiteralBase::Octal => 1,
            LiteralBase::Decimal => 2,
            LiteralBase::Hex => 3,
        }
    }
}

/// One digit of a number literal, for [`OwnedSVInt::from_digits`]: either a
/// concrete value (`0` to `radix - 1` for the chosen
/// [`LiteralBase`](LiteralBase)) or an unknown `x`/`z` digit — mirrors a
/// single element of the `std::span<logic_t const>` slang's
/// `SVInt::fromDigits` takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Digit {
    /// A concrete digit value, `0..=15`.
    Value(u8),
    /// An unknown (`x`) digit.
    X,
    /// A high-impedance (`z`) digit.
    Z,
}

impl Digit {
    /// Matches `slang::logic_t`'s own raw encoding (`X_VALUE = 0x80`,
    /// `Z_VALUE = 0x40`), which is what `slang_svint_from_digits` expects.
    fn to_raw(self) -> u8 {
        match self {
            Digit::Value(v) => v,
            Digit::X => 0x80,
            Digit::Z => 0x40,
        }
    }
}

/// An owned, independently mutable arbitrary-width four-state integer — the
/// live counterpart of the immutable [`SVInt`] snapshot above.
///
/// Backed by its own private `slang::SVInt` (reached through a
/// `slang_constant` this handle owns exclusively), so mutating it never
/// touches a [`Design`](crate::Design)'s frozen arena and is unaffected by
/// whatever else is being read from a `Design` concurrently — the same
/// reasoning that lets `Design` itself be `Send + Sync` (see
/// `rust/SOUNDNESS-MEMOS.md`) applies here in reverse: this object shares
/// nothing with any `Design`, so it is free to mutate.
///
/// Obtain one from
/// [`Expression::constant_integer_copy`](crate::Expression::constant_integer_copy)
/// (a fresh copy of the expression's already-folded integer constant, so
/// mutating it never affects the expression or anything else).
pub struct OwnedSVInt {
    raw: sys::slang_constant,
}

// SAFETY: `raw` is a `slang_constant` this handle owns exclusively (nothing
// else holds a pointer to it), wrapping a plain heap-allocated
// `slang::SVInt` with no thread affinity. Mutation is only ever reachable
// through `&mut self`, so `Send`/`Sync` add no way to race: a `Sync` shared
// reference only permits concurrent *reads* (all pure, non-mutating C
// accessors), which is sound for a plain memory block visited by multiple
// threads without writers.
unsafe impl Send for OwnedSVInt {}
// SAFETY: as above — a `Sync` shared reference only ever permits concurrent
// pure reads (no accessor above mutates through `&self`).
unsafe impl Sync for OwnedSVInt {}

impl OwnedSVInt {
    /// Wraps an owned `slang_constant`, taking ownership of it.
    ///
    /// SAFETY: `raw` must be a valid, owned `slang_constant` handle (or
    /// null), exclusively held by the caller (nothing else may read or free
    /// it). Consumed: freed here if it isn't an integer, otherwise owned by
    /// the returned `OwnedSVInt`.
    pub(crate) unsafe fn from_raw(raw: sys::slang_constant) -> Option<OwnedSVInt> {
        if raw.is_null() {
            return None;
        }
        // SAFETY: `raw` is valid per this function's own precondition.
        let v = unsafe { sys::slang_constant_integer(raw) };
        if v.is_null() {
            // Not an integer constant: this handle isn't useful as an
            // `OwnedSVInt`, and the caller has transferred ownership to us,
            // so we're responsible for freeing it.
            // SAFETY: `raw` is a valid owned handle we're discarding.
            unsafe { sys::slang_constant_destroy(raw) };
            return None;
        }
        Some(OwnedSVInt { raw })
    }

    /// The borrowed `slang_svint` view of `self.raw`, valid for as long as
    /// `self` is alive (it borrows from `self.raw`, which `self` owns).
    fn svint(&self) -> sys::slang_svint {
        // SAFETY: `self.raw` is a valid handle for the lifetime of `self`.
        unsafe { sys::slang_constant_integer(self.raw) }
    }

    /// The borrowed `slang_constant` view of `self.raw` (see
    /// [`Type::coerce_value`](crate::Type::coerce_value)), valid for as long
    /// as `self` is alive. `self` retains ownership; the callee must not
    /// free it.
    pub(crate) fn raw(&self) -> sys::slang_constant {
        self.raw
    }

    /// An immutable snapshot of the current value (see [`SVInt`]). A pure
    /// read; does not consume or invalidate `self`.
    pub fn snapshot(&self) -> SVInt {
        // SAFETY: `self.raw` is valid; these are pure reads on it.
        let is_true = unsafe { sys::slang_constant_is_true(self.raw) };
        // SAFETY: as above.
        let is_false = unsafe { sys::slang_constant_is_false(self.raw) };
        // SAFETY: the borrowed svint is read only while `self` (and so
        // `self.raw`) is alive, which the `&self` borrow guarantees.
        unsafe { SVInt::from_raw(self.svint(), is_true, is_false) }
    }

    /// In-place bitwise AND-assignment `self &= rhs` — mirrors
    /// `slang::SVInt::operator&=`. `self`'s bit width is unchanged: unlike
    /// the C++ operator (which auto-extends a narrower operand), `self` and
    /// `rhs` must already share the same bit width, or this fails (`self`
    /// left unmodified) with [`Error::InvalidArgument`].
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [3:0] X = 4'b1100; localparam bit [3:0] Y = 4'b1010; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let x = body.find("X").unwrap().initializer().unwrap();
    /// let y = body.find("Y").unwrap().initializer().unwrap();
    /// let mut xv = x.constant_integer_copy().unwrap();
    /// let yv = y.constant_integer_copy().unwrap();
    /// xv.and_assign(&yv)?;
    /// assert_eq!(xv.snapshot().as_i64(), Some(0b1000));
    /// # Ok(()) }
    /// ```
    pub fn and_assign(&mut self, rhs: &OwnedSVInt) -> Result<(), Error> {
        let mut err = ffi::error();
        // SAFETY: both svints are borrowed from live, valid handles for the
        // duration of this call.
        unsafe { sys::slang_svint_iand(self.svint(), rhs.svint(), &mut err) };
        ffi::check(&err)
    }

    /// In-place bitwise OR-assignment `self |= rhs` — mirrors
    /// `slang::SVInt::operator|=`. Same in-place, no-width-change contract as
    /// [`and_assign`](Self::and_assign).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [3:0] X = 4'b1100; localparam bit [3:0] Y = 4'b1010; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let x = body.find("X").unwrap().initializer().unwrap();
    /// let y = body.find("Y").unwrap().initializer().unwrap();
    /// let mut xv = x.constant_integer_copy().unwrap();
    /// let yv = y.constant_integer_copy().unwrap();
    /// xv.or_assign(&yv)?;
    /// assert_eq!(xv.snapshot().as_i64(), Some(0b1110));
    /// # Ok(()) }
    /// ```
    pub fn or_assign(&mut self, rhs: &OwnedSVInt) -> Result<(), Error> {
        let mut err = ffi::error();
        // SAFETY: both svints are borrowed from live, valid handles for the
        // duration of this call.
        unsafe { sys::slang_svint_ior(self.svint(), rhs.svint(), &mut err) };
        ffi::check(&err)
    }

    /// In-place bitwise XOR-assignment `self ^= rhs` — mirrors
    /// `slang::SVInt::operator^=`. Same in-place, no-width-change contract as
    /// [`and_assign`](Self::and_assign).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [3:0] X = 4'b1100; localparam bit [3:0] Y = 4'b1010; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let x = body.find("X").unwrap().initializer().unwrap();
    /// let y = body.find("Y").unwrap().initializer().unwrap();
    /// let mut xv = x.constant_integer_copy().unwrap();
    /// let yv = y.constant_integer_copy().unwrap();
    /// xv.xor_assign(&yv)?;
    /// assert_eq!(xv.snapshot().as_i64(), Some(0b0110));
    /// # Ok(()) }
    /// ```
    pub fn xor_assign(&mut self, rhs: &OwnedSVInt) -> Result<(), Error> {
        let mut err = ffi::error();
        // SAFETY: both svints are borrowed from live, valid handles for the
        // duration of this call.
        unsafe { sys::slang_svint_ixor(self.svint(), rhs.svint(), &mut err) };
        ffi::check(&err)
    }

    /// Replaces the bit range `[msb:lsb]` of `self` with `value`, in place —
    /// mirrors `slang::SVInt::set`. `self`'s bit width is unchanged (only
    /// `msb - lsb + 1` bits of it are overwritten). Fails (`self` left
    /// unmodified) with [`Error::InvalidArgument`] if `msb < lsb` or
    /// `value`'s bit width is not exactly `msb - lsb + 1`.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [7:0] X = 8'h00; localparam bit [3:0] Y = 4'hF; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let x = body.find("X").unwrap().initializer().unwrap();
    /// let y = body.find("Y").unwrap().initializer().unwrap();
    /// let mut xv = x.constant_integer_copy().unwrap();
    /// let yv = y.constant_integer_copy().unwrap();
    /// xv.set_range(3, 0, &yv)?;
    /// assert_eq!(xv.snapshot().as_i64(), Some(0x0F));
    /// assert_eq!(xv.snapshot().bit_width(), 8);
    /// # Ok(()) }
    /// ```
    pub fn set_range(&mut self, msb: i32, lsb: i32, value: &OwnedSVInt) -> Result<(), Error> {
        let mut err = ffi::error();
        // SAFETY: both svints are borrowed from live, valid handles for the
        // duration of this call.
        unsafe { sys::slang_svint_set(self.svint(), msb, lsb, value.svint(), &mut err) };
        ffi::check(&err)
    }

    /// Sets every bit of `self` to 1, in place (bit width and signedness
    /// unchanged) — mirrors `slang::SVInt::setAllOnes`.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [3:0] X = 4'b0000; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("X").unwrap().initializer().unwrap();
    /// let mut v = init.constant_integer_copy().unwrap();
    /// v.set_all_ones();
    /// assert_eq!(v.snapshot().as_i64(), Some(0b1111));
    /// # Ok(()) }
    /// ```
    pub fn set_all_ones(&mut self) {
        // SAFETY: the svint is borrowed from `self.raw`, which `self` owns
        // and keeps alive for the duration of this call.
        unsafe { sys::slang_svint_set_all_ones(self.svint()) };
    }

    /// Sets every bit of `self` to 0, in place — mirrors
    /// `slang::SVInt::setAllZeros`.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [3:0] X = 4'b1111; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("X").unwrap().initializer().unwrap();
    /// let mut v = init.constant_integer_copy().unwrap();
    /// v.set_all_zeros();
    /// assert_eq!(v.snapshot().as_i64(), Some(0));
    /// # Ok(()) }
    /// ```
    pub fn set_all_zeros(&mut self) {
        // SAFETY: as `set_all_ones`.
        unsafe { sys::slang_svint_set_all_zeros(self.svint()) };
    }

    /// Sets every bit of `self` to x, in place — mirrors
    /// `slang::SVInt::setAllX`.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [3:0] X = 4'b0000; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("X").unwrap().initializer().unwrap();
    /// let mut v = init.constant_integer_copy().unwrap();
    /// v.set_all_x();
    /// assert!(v.snapshot().has_unknown());
    /// assert_eq!(v.snapshot().bit(0), Some(sv_lang::Bit::X));
    /// # Ok(()) }
    /// ```
    pub fn set_all_x(&mut self) {
        // SAFETY: as `set_all_ones`.
        unsafe { sys::slang_svint_set_all_x(self.svint()) };
    }

    /// Sets every bit of `self` to z, in place — mirrors
    /// `slang::SVInt::setAllZ`.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [3:0] X = 4'b0000; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("X").unwrap().initializer().unwrap();
    /// let mut v = init.constant_integer_copy().unwrap();
    /// v.set_all_z();
    /// assert!(v.snapshot().has_unknown());
    /// assert_eq!(v.snapshot().bit(0), Some(sv_lang::Bit::Z));
    /// # Ok(()) }
    /// ```
    pub fn set_all_z(&mut self) {
        // SAFETY: as `set_all_ones`.
        unsafe { sys::slang_svint_set_all_z(self.svint()) };
    }

    /// Reinterprets `self` as signed or unsigned in place, without changing
    /// its bits or width — mirrors `slang::SVInt::setSigned`.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [3:0] X = 4'b1000; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("X").unwrap().initializer().unwrap();
    /// let mut v = init.constant_integer_copy().unwrap();
    /// assert!(!v.snapshot().is_signed());
    /// v.set_signed(true);
    /// assert!(v.snapshot().is_signed());
    /// # Ok(()) }
    /// ```
    pub fn set_signed(&mut self, is_signed: bool) {
        // SAFETY: the svint is borrowed from `self.raw`, which `self` owns
        // and keeps alive for the duration of this call.
        unsafe { sys::slang_svint_set_signed(self.svint(), is_signed) };
    }

    /// Removes all unknown (x/z) bits from `self` in place, converting them
    /// to 0 — mirrors `slang::SVInt::flattenUnknowns`. `self`'s bit width is
    /// unchanged.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [3:0] X = 4'b0000; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("X").unwrap().initializer().unwrap();
    /// let mut v = init.constant_integer_copy().unwrap();
    /// v.set_all_x();
    /// assert!(v.snapshot().has_unknown());
    /// v.flatten_unknowns();
    /// assert!(!v.snapshot().has_unknown());
    /// assert_eq!(v.snapshot().as_i64(), Some(0));
    /// # Ok(()) }
    /// ```
    pub fn flatten_unknowns(&mut self) {
        // SAFETY: as `set_signed`.
        unsafe { sys::slang_svint_flatten_unknowns(self.svint()) };
    }

    /// Resizes `self` in place to the minimum number of bits that can
    /// represent its current value without changing it — mirrors
    /// `slang::SVInt::shrinkToFit`. May reallocate `self`'s internal
    /// storage.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [31:0] X = 32'd3; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("X").unwrap().initializer().unwrap();
    /// let mut v = init.constant_integer_copy().unwrap();
    /// assert_eq!(v.snapshot().bit_width(), 32);
    /// v.shrink_to_fit();
    /// assert_eq!(v.snapshot().bit_width(), 2);
    /// assert_eq!(v.snapshot().as_i64(), Some(3));
    /// # Ok(()) }
    /// ```
    pub fn shrink_to_fit(&mut self) {
        // SAFETY: as `set_signed`.
        unsafe { sys::slang_svint_shrink_to_fit(self.svint()) };
    }

    /// If bit `msb` of `self` is set, duplicates it into every bit in
    /// `[bit_width-1:msb]`, in place — mirrors
    /// `slang::SVInt::signExtendFrom`. `self`'s bit width is unchanged.
    /// Fails with [`Error::InvalidArgument`] if `msb` is not strictly less
    /// than `self`'s bit width minus one.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [7:0] X = 8'b0000_1000; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let init = body.find("X").unwrap().initializer().unwrap();
    /// let mut v = init.constant_integer_copy().unwrap();
    /// v.sign_extend_from(3)?;
    /// assert_eq!(v.snapshot().as_i64(), Some(0b1111_1000));
    /// assert_eq!(v.snapshot().bit_width(), 8);
    /// # Ok(()) }
    /// ```
    pub fn sign_extend_from(&mut self, msb: u32) -> Result<(), Error> {
        let mut err = ffi::error();
        // SAFETY: the svint is borrowed from `self.raw`, which `self` owns
        // and keeps alive for the duration of this call.
        unsafe { sys::slang_svint_sign_extend_from(self.svint(), msb, &mut err) };
        ffi::check(&err)
    }

    /// Common tail for a `slang_svint_*` static-factory C call: checks the
    /// error, then wraps the resulting owned constant into an
    /// [`OwnedSVInt`]. Every `slang_svint_*` factory only ever succeeds with
    /// a constant that wraps an SVInt, so a success with a non-integer
    /// result is treated as an internal-consistency bug rather than
    /// propagated as `None`.
    ///
    /// SAFETY: `out`/`err` must be the just-populated outputs of one of the
    /// `slang_svint_*` factory C calls.
    unsafe fn finish_factory(
        out: sys::slang_constant,
        err: sys::slang_error,
    ) -> Result<OwnedSVInt, Error> {
        ffi::check(&err)?;
        // SAFETY: `out` is a valid owned handle per this function's
        // contract (a successful factory call always returns one).
        Ok(unsafe { OwnedSVInt::from_raw(out) }
            .expect("a successful slang_svint_* factory returns an integer constant"))
    }

    /// Creates a new all-X value of the given bit width — mirrors the
    /// static `slang::SVInt::createFillX`. Fails with
    /// [`Error::InvalidArgument`] if `bits` is 0.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let v = sv_lang::OwnedSVInt::create_fill_x(4, false)?;
    /// assert!(v.snapshot().has_unknown());
    /// assert_eq!(v.snapshot().bit(0), Some(sv_lang::Bit::X));
    /// assert_eq!(v.snapshot().bit_width(), 4);
    /// # Ok(()) }
    /// ```
    pub fn create_fill_x(bits: u32, is_signed: bool) -> Result<OwnedSVInt, Error> {
        let mut err = ffi::error();
        // SAFETY: a plain scalar call; `err` is a valid out-pointer.
        let out = unsafe { sys::slang_svint_create_fill_x(bits, is_signed, &mut err) };
        // SAFETY: `out`/`err` are the outputs of the call just above.
        unsafe { Self::finish_factory(out, err) }
    }

    /// Creates a new all-Z value of the given bit width — mirrors the
    /// static `slang::SVInt::createFillZ`. Fails with
    /// [`Error::InvalidArgument`] if `bits` is 0.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let v = sv_lang::OwnedSVInt::create_fill_z(4, false)?;
    /// assert!(v.snapshot().has_unknown());
    /// assert_eq!(v.snapshot().bit(0), Some(sv_lang::Bit::Z));
    /// # Ok(()) }
    /// ```
    pub fn create_fill_z(bits: u32, is_signed: bool) -> Result<OwnedSVInt, Error> {
        let mut err = ffi::error();
        // SAFETY: a plain scalar call; `err` is a valid out-pointer.
        let out = unsafe { sys::slang_svint_create_fill_z(bits, is_signed, &mut err) };
        // SAFETY: `out`/`err` are the outputs of the call just above.
        unsafe { Self::finish_factory(out, err) }
    }

    /// Builds a value from an array of digits in the given base (e.g. the
    /// digits of a sized literal like `12'hFF`) — mirrors the static
    /// `slang::SVInt::fromDigits`. If the value doesn't fit in `bits`, it is
    /// truncated from the left (matching the SystemVerilog literal
    /// truncation rule). Fails with [`Error::InvalidArgument`] if `bits` is
    /// 0, `digits` is empty, or a digit is too large for `base`.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// use sv_lang::{Digit, LiteralBase, OwnedSVInt};
    /// let v = OwnedSVInt::from_digits(
    ///     8,
    ///     LiteralBase::Hex,
    ///     false,
    ///     false,
    ///     &[Digit::Value(0xF), Digit::Value(0xA)],
    /// )?;
    /// assert_eq!(v.snapshot().as_i64(), Some(0xFA));
    ///
    /// let x = OwnedSVInt::from_digits(4, LiteralBase::Binary, false, true, &[Digit::X])?;
    /// assert!(x.snapshot().has_unknown());
    /// # Ok(()) }
    /// ```
    pub fn from_digits(
        bits: u32,
        base: LiteralBase,
        is_signed: bool,
        any_unknown: bool,
        digits: &[Digit],
    ) -> Result<OwnedSVInt, Error> {
        let raw_digits: Vec<u8> = digits.iter().map(|d| d.to_raw()).collect();
        let mut err = ffi::error();
        // SAFETY: `raw_digits` is a valid slice for the duration of this
        // call; `err` is a valid out-pointer.
        let out = unsafe {
            sys::slang_svint_from_digits(
                bits,
                base.to_raw(),
                is_signed,
                any_unknown,
                raw_digits.as_ptr(),
                raw_digits.len() as u32,
                &mut err,
            )
        };
        // SAFETY: `out`/`err` are the outputs of the call just above.
        unsafe { Self::finish_factory(out, err) }
    }

    /// Converts a `double` to a value of the given bit width, rounding to
    /// nearest (ties away from zero) if `round` is true, truncating toward
    /// zero otherwise — mirrors the static `slang::SVInt::fromDouble`.
    /// Fails with [`Error::InvalidArgument`] if `bits` is 0.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let v = sv_lang::OwnedSVInt::from_double(32, 3.7, false, true)?;
    /// assert_eq!(v.snapshot().as_i64(), Some(4));
    /// let t = sv_lang::OwnedSVInt::from_double(32, 3.7, false, false)?;
    /// assert_eq!(t.snapshot().as_i64(), Some(3));
    /// # Ok(()) }
    /// ```
    pub fn from_double(
        bits: u32,
        value: f64,
        is_signed: bool,
        round: bool,
    ) -> Result<OwnedSVInt, Error> {
        let mut err = ffi::error();
        // SAFETY: a plain scalar call; `err` is a valid out-pointer.
        let out = unsafe { sys::slang_svint_from_double(bits, value, is_signed, round, &mut err) };
        // SAFETY: `out`/`err` are the outputs of the call just above.
        unsafe { Self::finish_factory(out, err) }
    }

    /// Converts a `float` to a value of the given bit width — mirrors the
    /// static `slang::SVInt::fromFloat`. Same rounding and error contract as
    /// [`from_double`](Self::from_double).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// let v = sv_lang::OwnedSVInt::from_float(32, 3.7f32, false, true)?;
    /// assert_eq!(v.snapshot().as_i64(), Some(4));
    /// # Ok(()) }
    /// ```
    pub fn from_float(
        bits: u32,
        value: f32,
        is_signed: bool,
        round: bool,
    ) -> Result<OwnedSVInt, Error> {
        let mut err = ffi::error();
        // SAFETY: a plain scalar call; `err` is a valid out-pointer.
        let out = unsafe { sys::slang_svint_from_float(bits, value, is_signed, round, &mut err) };
        // SAFETY: `out`/`err` are the outputs of the call just above.
        unsafe { Self::finish_factory(out, err) }
    }

    /// Concatenates `operands` (msb-first, i.e. `operands[0]` becomes the
    /// most-significant bits of the result) into a new value — mirrors the
    /// static `slang::SVInt::concat`. An empty slice yields a 1-bit zero.
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source("module m; localparam bit [3:0] X = 4'hA; localparam bit [3:0] Y = 4'hB; endmodule\n")?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let x = body.find("X").unwrap().initializer().unwrap().constant_integer_copy().unwrap();
    /// let y = body.find("Y").unwrap().initializer().unwrap().constant_integer_copy().unwrap();
    /// let cat = sv_lang::OwnedSVInt::concat(&[&x, &y])?;
    /// assert_eq!(cat.snapshot().bit_width(), 8);
    /// assert_eq!(cat.snapshot().as_i64(), Some(0xAB));
    /// # Ok(()) }
    /// ```
    pub fn concat(operands: &[&OwnedSVInt]) -> Result<OwnedSVInt, Error> {
        let raw: Vec<sys::slang_svint> = operands.iter().map(|o| o.svint()).collect();
        let mut err = ffi::error();
        // SAFETY: `raw` holds svints borrowed from operands that all outlive
        // this call; `err` is a valid out-pointer.
        let out = unsafe { sys::slang_svint_concat(raw.as_ptr(), raw.len() as u32, &mut err) };
        // SAFETY: `out`/`err` are the outputs of the call just above.
        unsafe { Self::finish_factory(out, err) }
    }

    /// Evaluates `condition ? lhs : rhs` (four-state: the result is
    /// bitwise-unknown wherever `condition` is unknown and `lhs`/`rhs`
    /// disagree there) — mirrors the static `slang::SVInt::conditional`.
    /// `lhs` and `rhs` are extended to their common width first (matching
    /// the C++ static function).
    ///
    /// ```
    /// # fn main() -> Result<(), sv_lang::Error> {
    /// # let session = sv_lang::Session::new();
    /// # let mut comp = sv_lang::Compilation::new(&session)?;
    /// # comp.add_source(
    /// #     "module m; localparam bit C = 1; localparam bit [3:0] X = 4'hA; localparam bit [3:0] Y = 4'hB; endmodule\n",
    /// # )?;
    /// # let design = comp.compile()?;
    /// let body = design.top_instances().next().unwrap().instance_body().unwrap();
    /// let c = body.find("C").unwrap().initializer().unwrap().constant_integer_copy().unwrap();
    /// let x = body.find("X").unwrap().initializer().unwrap().constant_integer_copy().unwrap();
    /// let y = body.find("Y").unwrap().initializer().unwrap().constant_integer_copy().unwrap();
    /// let r = sv_lang::OwnedSVInt::conditional(&c, &x, &y)?;
    /// assert_eq!(r.snapshot().as_i64(), Some(0xA));
    /// # Ok(()) }
    /// ```
    pub fn conditional(
        condition: &OwnedSVInt,
        lhs: &OwnedSVInt,
        rhs: &OwnedSVInt,
    ) -> Result<OwnedSVInt, Error> {
        let mut err = ffi::error();
        // SAFETY: all three svints are borrowed from live, valid handles
        // for the duration of this call.
        let out = unsafe {
            sys::slang_svint_conditional(condition.svint(), lhs.svint(), rhs.svint(), &mut err)
        };
        // SAFETY: `out`/`err` are the outputs of the call just above.
        unsafe { Self::finish_factory(out, err) }
    }
}

impl core::fmt::Display for OwnedSVInt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.snapshot().fmt(f)
    }
}

impl core::fmt::Debug for OwnedSVInt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("OwnedSVInt").field(&self.snapshot()).finish()
    }
}

impl Drop for OwnedSVInt {
    fn drop(&mut self) {
        // SAFETY: `self.raw` is a valid owned handle we hold exclusively.
        unsafe { sys::slang_constant_destroy(self.raw) };
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
    /// An associative array (not expanded here; use the string form). The
    /// `bool` is whether the map is empty (captured at construction time,
    /// since the string form alone can't tell — see
    /// [`empty`](Self::empty)).
    Map(String, bool),
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
    /// This value as an unpacked array/queue's elements, if it is one.
    pub fn as_array(&self) -> Option<&[ConstantValue]> {
        match self {
            ConstantValue::Array(els) => Some(els),
            _ => None,
        }
    }

    /// True if the value is empty: a zero-length string, or a zero-element
    /// unpacked array/queue (both represented as [`Array`](Self::Array)) or
    /// associative array ([`Map`](Self::Map)); true also for every scalar
    /// kind (integer/real/null/unbounded/union) and [`Bad`](Self::Bad), since
    /// those have no size to speak of. Mirrors
    /// `slang::ConstantValue::empty`.
    pub fn empty(&self) -> bool {
        match self {
            ConstantValue::Array(els) => els.is_empty(),
            ConstantValue::Map(_, is_empty) => *is_empty,
            ConstantValue::String(s) => s.is_empty(),
            ConstantValue::Integer(_)
            | ConstantValue::Real(_)
            | ConstantValue::Null
            | ConstantValue::Unbounded
            | ConstantValue::Union(_)
            | ConstantValue::Bad => true,
        }
    }

    /// True if this is a container kind: an unpacked array, queue (both
    /// [`Array`](Self::Array)), or associative array ([`Map`](Self::Map)).
    /// Note a tagged [`Union`](Self::Union) is *not* a container. Mirrors
    /// `slang::ConstantValue::isContainer`.
    pub fn is_container(&self) -> bool {
        matches!(self, ConstantValue::Array(_) | ConstantValue::Map(..))
    }

    /// SystemVerilog condition-truthiness: true for a nonzero/definitely-1
    /// integer, a nonzero real, the unbounded (`$`) placeholder, or a
    /// nonempty string; false for everything else (including a container,
    /// `null`, a union, or [`Bad`](Self::Bad)). Mirrors
    /// `slang::ConstantValue::isTrue`.
    pub fn is_true(&self) -> bool {
        match self {
            ConstantValue::Integer(v) => v.is_true(),
            ConstantValue::Real(r) => *r != 0.0,
            ConstantValue::Unbounded => true,
            ConstantValue::String(s) => !s.is_empty(),
            ConstantValue::Null
            | ConstantValue::Array(_)
            | ConstantValue::Map(..)
            | ConstantValue::Union(_)
            | ConstantValue::Bad => false,
        }
    }

    /// SystemVerilog condition-falsiness: true for a known-all-zero integer
    /// (no x/z bits), a zero real, `null`, or an empty string; false for
    /// everything else (including the unbounded placeholder, a container, a
    /// union, or [`Bad`](Self::Bad)). Mirrors
    /// `slang::ConstantValue::isFalse`.
    pub fn is_false(&self) -> bool {
        match self {
            ConstantValue::Integer(v) => v.is_false(),
            ConstantValue::Real(r) => *r == 0.0,
            ConstantValue::Null => true,
            ConstantValue::String(s) => s.is_empty(),
            ConstantValue::Unbounded
            | ConstantValue::Array(_)
            | ConstantValue::Map(..)
            | ConstantValue::Union(_)
            | ConstantValue::Bad => false,
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
                    // SAFETY: `raw` is valid; pure reads, no allocation.
                    let is_true = unsafe { sys::slang_constant_is_true(raw) };
                    // SAFETY: as above.
                    let is_false = unsafe { sys::slang_constant_is_false(raw) };
                    // SAFETY: `v` valid, `raw` alive.
                    ConstantValue::Integer(unsafe { SVInt::from_raw(v, is_true, is_false) })
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
                let text = unsafe { ffi::owned_str(sys::slang_constant_to_string(raw)) };
                // SAFETY: `raw` is valid; pure read, no allocation.
                let is_empty = unsafe { sys::slang_constant_empty(raw) };
                ConstantValue::Map(text, is_empty)
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
            ConstantValue::Map(s, _) | ConstantValue::Union(s) => f.write_str(s),
            ConstantValue::Bad => f.write_str("<bad>"),
        }
    }
}
