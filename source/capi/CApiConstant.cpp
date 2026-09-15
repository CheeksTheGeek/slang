//------------------------------------------------------------------------------
// CApiConstant.cpp
// C API: structured constant values (ConstantValue / SVInt)
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include <vector>

#include "slang/ast/ASTContext.h"
#include "slang/ast/Compilation.h"
#include "slang/ast/Expression.h"
#include "slang/ast/expressions/LiteralExpressions.h"
#include "slang/ast/symbols/AttributeSymbol.h"
#include "slang/ast/symbols/BlockSymbols.h"
#include "slang/ast/symbols/CompilationUnitSymbols.h"
#include "slang/ast/symbols/MemberSymbols.h"
#include "slang/numeric/ConstantValue.h"
#include "slang/numeric/SVInt.h"

using namespace slang;
using namespace slang::ast;
using namespace slang::capi;

// slang_constant_t is defined in CApiInternal.h (shared with CApiScript.cpp).

namespace {

const ConstantValue& valueOf(slang_constant c) {
    return c->value;
}

const SVInt& svintOf(slang_svint v) {
    return *reinterpret_cast<const SVInt*>(v);
}

// A mutable view of the same handle, used by the small set of svint
// accessors that mutate in place (slang_svint_iand/ior/ixor/set/setAllOnes
// etc). Every slang_svint handle actually points into a private,
// exclusively-owned slang_constant_t (see slang_constant_integer) rather
// than into the compilation's frozen/shared arena, so mutating it in place
// is memory-safe as long as the caller isn't concurrently reading the same
// handle from another thread (same requirement as any other non-const C API
// handle).
SVInt& svintMutOf(slang_svint v) {
    return const_cast<SVInt&>(svintOf(v));
}

// Shared body of the fixed-width `slang_svint_as_*` accessors: succeeds only
// when the value fits `T` losslessly (`SVInt::as<T>` returns nullopt otherwise).
template<typename T>
bool svintAs(slang_svint v, T* out) {
    if (!v)
        return false;
    if (auto r = svintOf(v).as<T>()) {
        if (out)
            *out = *r;
        return true;
    }
    return false;
}

// Shared encoding for a logic_t result: 0, 1, 2 (x), or 3 (z) — same 2-bit
// scheme as slang_svint_get_bit.
uint8_t logicToU8(logic_t bit) {
    if (!bit.isUnknown())
        return (uint8_t)bit.value; // 0 or 1
    return bit.value == logic_t::x.value ? (uint8_t)2 : (uint8_t)3;
}

} // namespace

slang_constant slang_expression_constant_value(slang_ast expr, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        auto e = exprOf(expr);
        if (!e)
            return (slang_constant) nullptr;
        const ConstantValue* cv = e->getConstant();
        if (!cv || cv->bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{*cv};
    });
    return nullptr;
}

slang_constant slang_symbol_attribute_value(slang_ast sym, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Attribute)
            return (slang_constant) nullptr;
        // Forced by the freeze sweep (AttributeSymbol::getValue), so this is
        // a pure read on a frozen design.
        const ConstantValue& cv = s->as<AttributeSymbol>().getValue();
        if (cv.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{cv};
    });
    return nullptr;
}

slang_constant slang_expr_integer_literal_value(slang_ast expr, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    auto e = exprOf(expr);
    if (!e || e->kind != ExpressionKind::IntegerLiteral) {
        setError(err, SLANG_ERR_INVALID_ARG, "not an integer literal");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        return new slang_constant_t{ConstantValue(e->as<IntegerLiteral>().getValue())};
    });
    return nullptr;
}

slang_constant slang_symbol_generate_block_array_index(slang_ast sym, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::GenerateBlock)
            return (slang_constant) nullptr;
        // The underlying SVInt points at ParameterSymbol::getValue()'s memo
        // (the implicit localparam for the loop's genvar), forced pre-seal
        // by the freeze sweep — a pure read; copying it into a fresh owned
        // ConstantValue here only allocates on the C heap, never the arena.
        auto idx = s->as<GenerateBlockSymbol>().getArrayIndex();
        if (!idx)
            return (slang_constant) nullptr;
        return new slang_constant_t{ConstantValue(*idx)};
    });
    return nullptr;
}

slang_constant slang_symbol_primitive_init_val(slang_ast sym, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        auto s = symbolOf(sym);
        if (!s || s->kind != SymbolKind::Primitive)
            return (slang_constant) nullptr;
        // A direct field read, set once at construction time -- a pure
        // read of the frozen arena; the fresh slang_constant_t here only
        // allocates on the C heap.
        auto* iv = s->as<PrimitiveSymbol>().initVal;
        if (!iv || iv->bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{*iv};
    });
    return nullptr;
}

slang_constant slang_expr_string_literal_int_value(slang_ast expr, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    auto e = exprOf(expr);
    if (!e || e->kind != ExpressionKind::StringLiteral) {
        setError(err, SLANG_ERR_INVALID_ARG, "not a string literal");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        return new slang_constant_t{ConstantValue(e->as<StringLiteral>().getIntValue())};
    });
    return nullptr;
}

slang_constant slang_expression_eval_constant(slang_ast expr, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        auto e = exprOf(expr);
        if (!e)
            return (slang_constant) nullptr;
        SealLift guard(expr.compilation);
        ASTContext ctx(expr.compilation->comp->getRoot(), LookupLocation::max);
        ConstantValue cv = ctx.tryEval(*e);
        if (cv.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(cv)};
    });
    return nullptr;
}

void slang_constant_destroy(slang_constant c) {
    delete c;
}

uint32_t slang_constant_kind_of(slang_constant c) {
    SLANG_C_ACCESS(0u, {
        if (!c)
            return (uint32_t)SLANG_CONSTANT_BAD;
        const ConstantValue& v = valueOf(c);
        if (v.isInteger())
            return (uint32_t)SLANG_CONSTANT_INTEGER;
        if (v.isReal())
            return (uint32_t)SLANG_CONSTANT_REAL;
        if (v.isShortReal())
            return (uint32_t)SLANG_CONSTANT_SHORTREAL;
        if (v.isString())
            return (uint32_t)SLANG_CONSTANT_STRING;
        if (v.isNullHandle())
            return (uint32_t)SLANG_CONSTANT_NULL;
        if (v.isUnbounded())
            return (uint32_t)SLANG_CONSTANT_UNBOUNDED;
        if (v.isUnpacked())
            return (uint32_t)SLANG_CONSTANT_UNPACKED;
        if (v.isMap())
            return (uint32_t)SLANG_CONSTANT_MAP;
        if (v.isQueue())
            return (uint32_t)SLANG_CONSTANT_QUEUE;
        if (v.isUnion())
            return (uint32_t)SLANG_CONSTANT_UNION;
        return (uint32_t)SLANG_CONSTANT_BAD;
    });
}

bool slang_constant_has_unknown(slang_constant c) {
    SLANG_C_ACCESS(false, { return c && valueOf(c).hasUnknown(); });
}

bool slang_constant_real(slang_constant c, double* out) {
    SLANG_C_ACCESS(false, {
        if (!c)
            return false;
        const ConstantValue& v = valueOf(c);
        if (v.isReal()) {
            if (out)
                *out = (double)v.real();
            return true;
        }
        if (v.isShortReal()) {
            if (out)
                *out = (double)(float)v.shortReal();
            return true;
        }
        return false;
    });
}

slang_str slang_constant_string(slang_constant c) {
    SLANG_C_ACCESS(borrowed(""), {
        if (c && valueOf(c).isString())
            return owned(std::string(valueOf(c).str()));
        return borrowed("");
    });
}

uint64_t slang_constant_size(slang_constant c) {
    SLANG_C_ACCESS(0ull, {
        if (!c)
            return 0ull;
        const ConstantValue& v = valueOf(c);
        if (v.isUnpacked() || v.isQueue())
            return (uint64_t)v.size();
        return 0ull;
    });
}

slang_constant slang_constant_element(slang_constant c, uint64_t index, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!c)
            return (slang_constant) nullptr;
        const ConstantValue& v = valueOf(c);
        if (!(v.isUnpacked() || v.isQueue()) || index >= (uint64_t)v.size())
            return (slang_constant) nullptr;
        return new slang_constant_t{v.at((size_t)index)};
    });
    return nullptr;
}

slang_str slang_constant_to_string(slang_constant c) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!c)
            return borrowed("");
        return owned(valueOf(c).toString());
    });
}

slang_svint slang_constant_integer(slang_constant c) {
    SLANG_C_ACCESS((slang_svint) nullptr, {
        if (c && valueOf(c).isInteger())
            return (slang_svint)&valueOf(c).integer();
        return (slang_svint) nullptr;
    });
}

bool slang_constant_flat_int(slang_constant c, slang_svint_flat* out) {
    SLANG_C_ACCESS(false, {
        if (!c || !valueOf(c).isInteger())
            return false;
        const SVInt& i = valueOf(c).integer();
        if (!i.isSingleWord())
            return false;
        int64_t val = 0;
        if (auto s = i.as<int64_t>())
            val = *s;
        else if (auto u = i.as<uint64_t>())
            val = (int64_t)*u;
        else
            return false;
        if (out) {
            out->value = val;
            out->bit_width = (uint32_t)i.getBitWidth();
            out->is_signed = i.isSigned();
            out->has_unknown = i.hasUnknown();
        }
        return true;
    });
}

bool slang_constant_empty(slang_constant c) {
    SLANG_C_ACCESS(false, { return c && valueOf(c).empty(); });
}

bool slang_constant_is_container(slang_constant c) {
    SLANG_C_ACCESS(false, { return c && valueOf(c).isContainer(); });
}

bool slang_constant_is_true(slang_constant c) {
    SLANG_C_ACCESS(false, { return c && valueOf(c).isTrue(); });
}

bool slang_constant_is_false(slang_constant c) {
    SLANG_C_ACCESS(false, { return c && valueOf(c).isFalse(); });
}

uint64_t slang_constant_bitstream_width(slang_constant c) {
    SLANG_C_ACCESS(0ull, { return c ? valueOf(c).getBitstreamWidth() : 0ull; });
}

slang_constant slang_constant_get_slice(slang_constant c, int32_t upper, int32_t lower,
                                        slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!c)
            return (slang_constant) nullptr;
        ConstantValue result = valueOf(c).getSlice(upper, lower, ConstantValue());
        if (result.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(result)};
    });
    return nullptr;
}

slang_constant slang_constant_convert_to_real(slang_constant c, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!c)
            return (slang_constant) nullptr;
        ConstantValue result = valueOf(c).convertToReal();
        if (result.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(result)};
    });
    return nullptr;
}

slang_constant slang_constant_convert_to_short_real(slang_constant c, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!c)
            return (slang_constant) nullptr;
        ConstantValue result = valueOf(c).convertToShortReal();
        if (result.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(result)};
    });
    return nullptr;
}

slang_constant slang_constant_convert_to_str(slang_constant c, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!c)
            return (slang_constant) nullptr;
        ConstantValue result = valueOf(c).convertToStr();
        if (result.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(result)};
    });
    return nullptr;
}

slang_constant slang_constant_convert_to_byte_array(slang_constant c, uint32_t size,
                                                     bool is_signed, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!c)
            return (slang_constant) nullptr;
        ConstantValue result = valueOf(c).convertToByteArray((bitwidth_t)size, is_signed);
        if (result.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(result)};
    });
    return nullptr;
}

slang_constant slang_constant_convert_to_byte_queue(slang_constant c, bool is_signed,
                                                     slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!c)
            return (slang_constant) nullptr;
        ConstantValue result = valueOf(c).convertToByteQueue(is_signed);
        if (result.bad())
            return (slang_constant) nullptr;
        return new slang_constant_t{std::move(result)};
    });
    return nullptr;
}

uint32_t slang_svint_bit_width(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).getBitWidth() : 0u; });
}

bool slang_svint_is_signed(slang_svint v) {
    SLANG_C_ACCESS(false, { return v && svintOf(v).isSigned(); });
}

bool slang_svint_has_unknown(slang_svint v) {
    SLANG_C_ACCESS(false, { return v && svintOf(v).hasUnknown(); });
}

bool slang_svint_as_i64(slang_svint v, int64_t* out) {
    SLANG_C_ACCESS(false, { return svintAs<int64_t>(v, out); });
}

bool slang_svint_as_u64(slang_svint v, uint64_t* out) {
    SLANG_C_ACCESS(false, { return svintAs<uint64_t>(v, out); });
}

uint8_t slang_svint_get_bit(slang_svint v, uint32_t index) {
    SLANG_C_ACCESS((uint8_t)0, {
        if (!v || index >= (uint32_t)svintOf(v).getBitWidth())
            return (uint8_t)0;
        return logicToU8(svintOf(v)[(int32_t)index]);
    });
}

slang_str slang_svint_to_string(slang_svint v) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!v)
            return borrowed("");
        return owned(svintOf(v).toString());
    });
}

uint32_t slang_svint_count_leading_ones(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).countLeadingOnes() : 0u; });
}

uint32_t slang_svint_count_leading_zeros(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).countLeadingZeros() : 0u; });
}

uint32_t slang_svint_count_leading_unknowns(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).countLeadingUnknowns() : 0u; });
}

uint32_t slang_svint_count_leading_zs(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).countLeadingZs() : 0u; });
}

uint32_t slang_svint_count_ones(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).countOnes() : 0u; });
}

uint32_t slang_svint_count_zeros(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).countZeros() : 0u; });
}

uint32_t slang_svint_count_xs(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).countXs() : 0u; });
}

uint32_t slang_svint_count_zs(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).countZs() : 0u; });
}

uint32_t slang_svint_active_bits(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).getActiveBits() : 0u; });
}

uint32_t slang_svint_min_represented_bits(slang_svint v) {
    SLANG_C_ACCESS(0u, { return v ? (uint32_t)svintOf(v).getMinRepresentedBits() : 0u; });
}

bool slang_svint_is_even(slang_svint v) {
    SLANG_C_ACCESS(false, { return v && svintOf(v).isEven(); });
}

bool slang_svint_is_odd(slang_svint v) {
    SLANG_C_ACCESS(false, { return v && svintOf(v).isOdd(); });
}

bool slang_svint_is_negative(slang_svint v) {
    SLANG_C_ACCESS(false, { return v && svintOf(v).isNegative(); });
}

bool slang_svint_is_sign_extended_from(slang_svint v, uint32_t msb) {
    SLANG_C_ACCESS(false, { return v && svintOf(v).isSignExtendedFrom((bitwidth_t)msb); });
}

uint8_t slang_svint_reduction_and(slang_svint v) {
    SLANG_C_ACCESS((uint8_t)0, { return v ? logicToU8(svintOf(v).reductionAnd()) : (uint8_t)0; });
}

uint8_t slang_svint_reduction_or(slang_svint v) {
    SLANG_C_ACCESS((uint8_t)0, { return v ? logicToU8(svintOf(v).reductionOr()) : (uint8_t)0; });
}

uint8_t slang_svint_reduction_xor(slang_svint v) {
    SLANG_C_ACCESS((uint8_t)0, { return v ? logicToU8(svintOf(v).reductionXor()) : (uint8_t)0; });
}

uint8_t slang_svint_logical_impl(slang_svint lhs, slang_svint rhs) {
    SLANG_C_ACCESS((uint8_t)0, {
        if (!lhs || !rhs)
            return (uint8_t)0;
        return logicToU8(SVInt::logicalImpl(svintOf(lhs), svintOf(rhs)));
    });
}

uint8_t slang_svint_logical_equiv(slang_svint lhs, slang_svint rhs) {
    SLANG_C_ACCESS((uint8_t)0, {
        if (!lhs || !rhs)
            return (uint8_t)0;
        return logicToU8(SVInt::logicalEquiv(svintOf(lhs), svintOf(rhs)));
    });
}

namespace {

logic_t fromC(slang_logic_value v) {
    return logic_t(v.value);
}

slang_logic_value toC(logic_t v) {
    return slang_logic_value{v.value};
}

} // namespace

uint8_t slang_logic_value_value(slang_logic_value v) {
    SLANG_C_ACCESS((uint8_t)0, { return v.value; });
}

slang_logic_value slang_logic_value_x(void) {
    SLANG_C_ACCESS(slang_logic_value{}, { return toC(logic_t::x); });
}

slang_logic_value slang_logic_value_z(void) {
    SLANG_C_ACCESS(slang_logic_value{}, { return toC(logic_t::z); });
}

bool slang_logic_value_is_unknown(slang_logic_value v) {
    SLANG_C_ACCESS(false, { return fromC(v).isUnknown(); });
}

slang_logic_value slang_logic_value_and(slang_logic_value lhs, slang_logic_value rhs) {
    SLANG_C_ACCESS(slang_logic_value{}, { return toC(fromC(lhs) & fromC(rhs)); });
}

slang_logic_value slang_logic_value_or(slang_logic_value lhs, slang_logic_value rhs) {
    SLANG_C_ACCESS(slang_logic_value{}, { return toC(fromC(lhs) | fromC(rhs)); });
}

slang_logic_value slang_logic_value_xor(slang_logic_value lhs, slang_logic_value rhs) {
    SLANG_C_ACCESS(slang_logic_value{}, { return toC(fromC(lhs) ^ fromC(rhs)); });
}

slang_logic_value slang_logic_value_not(slang_logic_value v) {
    SLANG_C_ACCESS(slang_logic_value{}, { return toC(~fromC(v)); });
}

slang_constant slang_svint_pow(slang_svint v, slang_svint rhs, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v || !rhs)
            return (slang_constant) nullptr;
        return new slang_constant_t{ConstantValue(svintOf(v).pow(svintOf(rhs)))};
    });
    return nullptr;
}

slang_constant slang_svint_replicate(slang_svint v, slang_svint times, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v || !times)
            return (slang_constant) nullptr;
        return new slang_constant_t{ConstantValue(svintOf(v).replicate(svintOf(times)))};
    });
    return nullptr;
}

slang_constant slang_svint_resize(slang_svint v, uint32_t bits, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v)
            return (slang_constant) nullptr;
        if (bits == 0) {
            setError(err, SLANG_ERR_INVALID_ARG, "bits must be nonzero");
            return (slang_constant) nullptr;
        }
        return new slang_constant_t{ConstantValue(svintOf(v).resize((bitwidth_t)bits))};
    });
    return nullptr;
}

slang_constant slang_svint_reverse(slang_svint v, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v)
            return (slang_constant) nullptr;
        return new slang_constant_t{ConstantValue(svintOf(v).reverse())};
    });
    return nullptr;
}

slang_constant slang_svint_sext(slang_svint v, uint32_t bits, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v)
            return (slang_constant) nullptr;
        if (bits <= (uint32_t)svintOf(v).getBitWidth()) {
            setError(err, SLANG_ERR_INVALID_ARG,
                     "bits must be greater than the current bit width");
            return (slang_constant) nullptr;
        }
        return new slang_constant_t{ConstantValue(svintOf(v).sext((bitwidth_t)bits))};
    });
    return nullptr;
}

slang_constant slang_svint_zext(slang_svint v, uint32_t bits, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v)
            return (slang_constant) nullptr;
        if (bits <= (uint32_t)svintOf(v).getBitWidth()) {
            setError(err, SLANG_ERR_INVALID_ARG,
                     "bits must be greater than the current bit width");
            return (slang_constant) nullptr;
        }
        return new slang_constant_t{ConstantValue(svintOf(v).zext((bitwidth_t)bits))};
    });
    return nullptr;
}

slang_constant slang_svint_extend(slang_svint v, uint32_t bits, bool is_signed, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v)
            return (slang_constant) nullptr;
        if (bits <= (uint32_t)svintOf(v).getBitWidth()) {
            setError(err, SLANG_ERR_INVALID_ARG,
                     "bits must be greater than the current bit width");
            return (slang_constant) nullptr;
        }
        return new slang_constant_t{
            ConstantValue(svintOf(v).extend((bitwidth_t)bits, is_signed))};
    });
    return nullptr;
}

slang_constant slang_svint_trunc(slang_svint v, uint32_t bits, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v)
            return (slang_constant) nullptr;
        if (bits == 0) {
            setError(err, SLANG_ERR_INVALID_ARG, "bits must be nonzero");
            return (slang_constant) nullptr;
        }
        if (bits > (uint32_t)svintOf(v).getBitWidth()) {
            setError(err, SLANG_ERR_INVALID_ARG, "bits must not exceed the current bit width");
            return (slang_constant) nullptr;
        }
        return new slang_constant_t{ConstantValue(svintOf(v).trunc((bitwidth_t)bits))};
    });
    return nullptr;
}

slang_constant slang_svint_slice(slang_svint v, int32_t msb, int32_t lsb, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v)
            return (slang_constant) nullptr;
        if (msb < lsb) {
            setError(err, SLANG_ERR_INVALID_ARG, "msb must be >= lsb");
            return (slang_constant) nullptr;
        }
        return new slang_constant_t{ConstantValue(svintOf(v).slice(msb, lsb))};
    });
    return nullptr;
}

slang_constant slang_svint_xnor(slang_svint lhs, slang_svint rhs, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!lhs || !rhs)
            return (slang_constant) nullptr;
        return new slang_constant_t{ConstantValue(svintOf(lhs).xnor(svintOf(rhs)))};
    });
    return nullptr;
}

slang_constant slang_svint_and(slang_svint lhs, slang_svint rhs, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!lhs || !rhs)
            return (slang_constant) nullptr;
        return new slang_constant_t{ConstantValue(svintOf(lhs) & svintOf(rhs))};
    });
    return nullptr;
}

slang_constant slang_svint_or(slang_svint lhs, slang_svint rhs, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!lhs || !rhs)
            return (slang_constant) nullptr;
        return new slang_constant_t{ConstantValue(svintOf(lhs) | svintOf(rhs))};
    });
    return nullptr;
}

slang_constant slang_svint_xor(slang_svint lhs, slang_svint rhs, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!lhs || !rhs)
            return (slang_constant) nullptr;
        return new slang_constant_t{ConstantValue(svintOf(lhs) ^ svintOf(rhs))};
    });
    return nullptr;
}

slang_constant slang_svint_not(slang_svint v, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v)
            return (slang_constant) nullptr;
        return new slang_constant_t{ConstantValue(~svintOf(v))};
    });
    return nullptr;
}

slang_svint slang_svint_iand(slang_svint v, slang_svint rhs, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v || !rhs)
            return (slang_svint) nullptr;
        if (svintOf(v).getBitWidth() != svintOf(rhs).getBitWidth()) {
            setError(err, SLANG_ERR_INVALID_ARG, "operands must have equal bit width");
            return (slang_svint) nullptr;
        }
        svintMutOf(v) &= svintOf(rhs);
        return v;
    });
    return nullptr;
}

slang_svint slang_svint_ior(slang_svint v, slang_svint rhs, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v || !rhs)
            return (slang_svint) nullptr;
        if (svintOf(v).getBitWidth() != svintOf(rhs).getBitWidth()) {
            setError(err, SLANG_ERR_INVALID_ARG, "operands must have equal bit width");
            return (slang_svint) nullptr;
        }
        svintMutOf(v) |= svintOf(rhs);
        return v;
    });
    return nullptr;
}

slang_svint slang_svint_ixor(slang_svint v, slang_svint rhs, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!v || !rhs)
            return (slang_svint) nullptr;
        if (svintOf(v).getBitWidth() != svintOf(rhs).getBitWidth()) {
            setError(err, SLANG_ERR_INVALID_ARG, "operands must have equal bit width");
            return (slang_svint) nullptr;
        }
        svintMutOf(v) ^= svintOf(rhs);
        return v;
    });
    return nullptr;
}

void slang_svint_set(slang_svint v, int32_t msb, int32_t lsb, slang_svint value,
                      slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!v || !value) {
        setError(err, SLANG_ERR_INVALID_ARG, "null handle");
        return;
    }
    if (msb < lsb) {
        setError(err, SLANG_ERR_INVALID_ARG, "msb must be >= lsb");
        return;
    }
    if (svintOf(value).getBitWidth() != (bitwidth_t)(msb - lsb + 1)) {
        setError(err, SLANG_ERR_INVALID_ARG, "value's bit width must equal msb - lsb + 1");
        return;
    }
    SLANG_C_GUARD(err, { svintMutOf(v).set(msb, lsb, svintOf(value)); });
}

void slang_svint_set_all_ones(slang_svint v) {
    if (!v)
        return;
    SLANG_C_GUARD(nullptr, { svintMutOf(v).setAllOnes(); });
}

void slang_svint_set_all_zeros(slang_svint v) {
    if (!v)
        return;
    SLANG_C_GUARD(nullptr, { svintMutOf(v).setAllZeros(); });
}

void slang_svint_set_all_x(slang_svint v) {
    if (!v)
        return;
    SLANG_C_GUARD(nullptr, { svintMutOf(v).setAllX(); });
}

void slang_svint_set_all_z(slang_svint v) {
    if (!v)
        return;
    SLANG_C_GUARD(nullptr, { svintMutOf(v).setAllZ(); });
}

void slang_svint_set_signed(slang_svint v, bool is_signed) {
    if (!v)
        return;
    SLANG_C_GUARD(nullptr, { svintMutOf(v).setSigned(is_signed); });
}

void slang_svint_flatten_unknowns(slang_svint v) {
    if (!v)
        return;
    SLANG_C_GUARD(nullptr, { svintMutOf(v).flattenUnknowns(); });
}

void slang_svint_shrink_to_fit(slang_svint v) {
    if (!v)
        return;
    SLANG_C_GUARD(nullptr, { svintMutOf(v).shrinkToFit(); });
}

void slang_svint_sign_extend_from(slang_svint v, uint32_t msb, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!v) {
        setError(err, SLANG_ERR_INVALID_ARG, "null handle");
        return;
    }
    if (svintOf(v).getBitWidth() < 2 || msb > (uint32_t)svintOf(v).getBitWidth() - 2) {
        setError(err, SLANG_ERR_INVALID_ARG,
                 "msb must be strictly less than bit_width - 1");
        return;
    }
    SLANG_C_GUARD(err, { svintMutOf(v).signExtendFrom((bitwidth_t)msb); });
}

slang_constant slang_svint_create_fill_x(uint32_t bits, bool is_signed, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (bits == 0) {
        setError(err, SLANG_ERR_INVALID_ARG, "bits must be nonzero");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        return new slang_constant_t{
            ConstantValue(SVInt::createFillX((bitwidth_t)bits, is_signed))};
    });
    return nullptr;
}

slang_constant slang_svint_create_fill_z(uint32_t bits, bool is_signed, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (bits == 0) {
        setError(err, SLANG_ERR_INVALID_ARG, "bits must be nonzero");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        return new slang_constant_t{
            ConstantValue(SVInt::createFillZ((bitwidth_t)bits, is_signed))};
    });
    return nullptr;
}

slang_constant slang_svint_from_digits(uint32_t bits, uint8_t base, bool is_signed,
                                       bool any_unknown, const slang_svint_digit* digits,
                                       uint32_t digit_count, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (bits == 0) {
        setError(err, SLANG_ERR_INVALID_ARG, "bits must be nonzero");
        return nullptr;
    }
    if (!digits || digit_count == 0) {
        setError(err, SLANG_ERR_INVALID_ARG, "no digits provided");
        return nullptr;
    }
    if (base > (uint8_t)LiteralBase::Hex) {
        setError(err, SLANG_ERR_INVALID_ARG, "base must be 0-3");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        std::vector<logic_t> logicDigits;
        logicDigits.reserve(digit_count);
        for (uint32_t i = 0; i < digit_count; i++)
            logicDigits.push_back(logic_t(digits[i]));
        return new slang_constant_t{ConstantValue(
            SVInt::fromDigits((bitwidth_t)bits, (LiteralBase)base, is_signed, any_unknown,
                              logicDigits))};
    });
    return nullptr;
}

slang_constant slang_svint_from_double(uint32_t bits, double value, bool is_signed, bool round,
                                       slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (bits == 0) {
        setError(err, SLANG_ERR_INVALID_ARG, "bits must be nonzero");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        return new slang_constant_t{
            ConstantValue(SVInt::fromDouble((bitwidth_t)bits, value, is_signed, round))};
    });
    return nullptr;
}

slang_constant slang_svint_from_float(uint32_t bits, float value, bool is_signed, bool round,
                                      slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (bits == 0) {
        setError(err, SLANG_ERR_INVALID_ARG, "bits must be nonzero");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        return new slang_constant_t{
            ConstantValue(SVInt::fromFloat((bitwidth_t)bits, value, is_signed, round))};
    });
    return nullptr;
}

slang_constant slang_svint_concat(const slang_svint* operands, uint32_t count, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (count > 0 && !operands) {
        setError(err, SLANG_ERR_INVALID_ARG, "null operands array");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        std::vector<SVInt> ops;
        ops.reserve(count);
        for (uint32_t i = 0; i < count; i++) {
            if (!operands[i]) {
                setError(err, SLANG_ERR_INVALID_ARG, "null operand");
                return (slang_constant) nullptr;
            }
            ops.push_back(svintOf(operands[i]));
        }
        return new slang_constant_t{ConstantValue(SVInt::concat(ops))};
    });
    return nullptr;
}

slang_constant slang_svint_conditional(slang_svint condition, slang_svint lhs, slang_svint rhs,
                                       slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        if (!condition || !lhs || !rhs)
            return (slang_constant) nullptr;
        return new slang_constant_t{
            ConstantValue(SVInt::conditional(svintOf(condition), svintOf(lhs), svintOf(rhs)))};
    });
    return nullptr;
}

// ---- ConstantRange ----------------------------------------------------------
//
// slang::ConstantRange is a plain two-field value type (no allocation, no AST
// node); these functions cross the whole value by C struct rather than a
// handle, so they are pure functions of their arguments.

namespace {

ConstantRange toCpp(slang_constant_range r) {
    return ConstantRange(r.left, r.right);
}

slang_constant_range toC(ConstantRange r) {
    return slang_constant_range{r.left, r.right};
}

} // namespace

int32_t slang_constant_range_left(slang_constant_range r) {
    SLANG_C_ACCESS(0, { return r.left; });
}

int32_t slang_constant_range_right(slang_constant_range r) {
    SLANG_C_ACCESS(0, { return r.right; });
}

uint32_t slang_constant_range_width(slang_constant_range r) {
    SLANG_C_ACCESS(0u, { return (uint32_t)toCpp(r).width(); });
}

int32_t slang_constant_range_lower(slang_constant_range r) {
    SLANG_C_ACCESS(0, { return toCpp(r).lower(); });
}

int32_t slang_constant_range_upper(slang_constant_range r) {
    SLANG_C_ACCESS(0, { return toCpp(r).upper(); });
}

bool slang_constant_range_is_descending(slang_constant_range r) {
    SLANG_C_ACCESS(false, { return toCpp(r).isDescending(); });
}

slang_constant_range slang_constant_range_reverse(slang_constant_range r) {
    SLANG_C_ACCESS(r, { return toC(toCpp(r).reverse()); });
}

slang_constant_range slang_constant_range_subrange(slang_constant_range r,
                                                    slang_constant_range select) {
    SLANG_C_ACCESS(r, { return toC(toCpp(r).subrange(toCpp(select))); });
}

bool slang_constant_range_contains_point(slang_constant_range r, int32_t index) {
    SLANG_C_ACCESS(false, { return toCpp(r).containsPoint(index); });
}

bool slang_constant_range_overlaps(slang_constant_range r, slang_constant_range other) {
    SLANG_C_ACCESS(false, { return toCpp(r).overlaps(toCpp(other)); });
}

int32_t slang_constant_range_translate_index(slang_constant_range r, int32_t index) {
    SLANG_C_ACCESS(0, { return toCpp(r).translateIndex(index); });
}

bool slang_constant_range_get_indexed_range(int32_t l, int32_t width, bool descending,
                                            bool indexed_up, slang_constant_range* out) {
    SLANG_C_ACCESS(false, {
        if (!out)
            return false;
        auto result = ConstantRange::getIndexedRange(l, width, descending, indexed_up);
        if (!result)
            return false;
        *out = toC(*result);
        return true;
    });
}
