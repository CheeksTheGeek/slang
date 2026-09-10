//------------------------------------------------------------------------------
// CApiConstant.cpp
// C API: structured constant values (ConstantValue / SVInt)
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include "slang/ast/ASTContext.h"
#include "slang/ast/Compilation.h"
#include "slang/ast/Expression.h"
#include "slang/ast/symbols/CompilationUnitSymbols.h"
#include "slang/numeric/ConstantValue.h"
#include "slang/numeric/SVInt.h"

using namespace slang;
using namespace slang::ast;
using namespace slang::capi;

// An owned constant value handle.
struct slang_constant_t {
    slang::ConstantValue value;
};

namespace {

// Temporarily lifts the seal for an evaluation that mutates (constant caching).
struct SealGuard {
    slang_compilation comp;
    bool lifted;
    explicit SealGuard(slang_compilation c) : comp(c), lifted(c->sealed) {
        if (lifted)
            comp->comp->unfreeze();
    }
    ~SealGuard() {
        if (lifted)
            comp->comp->freeze();
    }
};

const ConstantValue& valueOf(slang_constant c) {
    return c->value;
}

const SVInt& svintOf(slang_svint v) {
    return *reinterpret_cast<const SVInt*>(v);
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

slang_constant slang_expression_eval_constant(slang_ast expr, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, {
        auto e = exprOf(expr);
        if (!e)
            return (slang_constant) nullptr;
        SealGuard guard(expr.compilation);
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
    SLANG_C_ACCESS(false, {
        if (!v)
            return false;
        if (auto r = svintOf(v).as<int64_t>()) {
            if (out)
                *out = *r;
            return true;
        }
        return false;
    });
}

bool slang_svint_as_u64(slang_svint v, uint64_t* out) {
    SLANG_C_ACCESS(false, {
        if (!v)
            return false;
        if (auto r = svintOf(v).as<uint64_t>()) {
            if (out)
                *out = *r;
            return true;
        }
        return false;
    });
}

uint8_t slang_svint_get_bit(slang_svint v, uint32_t index) {
    SLANG_C_ACCESS((uint8_t)0, {
        if (!v || index >= (uint32_t)svintOf(v).getBitWidth())
            return (uint8_t)0;
        logic_t bit = svintOf(v)[(int32_t)index];
        if (!bit.isUnknown())
            return (uint8_t)bit.value; // 0 or 1
        return bit.value == logic_t::x.value ? (uint8_t)2 : (uint8_t)3;
    });
}

slang_str slang_svint_to_string(slang_svint v) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!v)
            return borrowed("");
        return owned(svintOf(v).toString());
    });
}
