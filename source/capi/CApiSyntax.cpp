//------------------------------------------------------------------------------
// CApiSyntax.cpp
// C API: syntax trees, nodes, tokens, trivia and traversal
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"

#include "slang/syntax/AllSyntax.h"
#include "slang/syntax/SyntaxPrinter.h"

using namespace slang;
using namespace slang::capi;
using namespace slang::syntax;
using namespace slang::parsing;

// A printer configured to reproduce the tree's token stream losslessly:
// every token and trivia item in the tree, with macros and includes expanded
// (as they are in the tree). This is what slang_syntax_tree_walk streams.
static SyntaxPrinter losslessPrinter() {
    SyntaxPrinter printer;
    printer.setIncludeDirectives(true)
        .setIncludeSkipped(true)
        .setIncludeTrivia(true)
        .setExpandMacros(true)
        .setExpandIncludes(true)
        .setSquashNewlines(false);
    return printer;
}

// Structured trivia (directives, skipped text) has no raw text of its own;
// its text is the printed form of what it wraps.
static bool isStructuredTrivia(TriviaKind kind) {
    return kind == TriviaKind::Directive || kind == TriviaKind::SkippedSyntax ||
           kind == TriviaKind::SkippedTokens;
}

static slang_str triviaText(const Trivia& trivia) {
    if (isStructuredTrivia(trivia.kind))
        return owned(losslessPrinter().print(trivia).str());
    return borrowed(trivia.getRawText());
}

// ---- Trees ------------------------------------------------------------------

static slang_syntax_tree wrapTree(std::shared_ptr<SyntaxTree> tree, slang_source_manager sm) {
    auto handle = new slang_syntax_tree_t();
    handle->tree = std::move(tree);
    handle->sm = sm;
    return handle;
}

slang_syntax_tree slang_syntax_tree_from_text(slang_source_manager sm, const char* text,
                                              size_t text_len, const char* name, size_t name_len,
                                              const char* path, size_t path_len,
                                              slang_options options, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!sm || !text) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        auto nameView = toView(name, name_len);
        // fromFileInMemory always parses a full compilation unit; fromText
        // would guess at a smaller construct for test convenience.
        auto tree = SyntaxTree::fromFileInMemory(toView(text, text_len), sm->sm,
                                                 nameView.empty() ? "source"sv : nameView,
                                                 toView(path, path_len),
                                                 options ? options->toBag() : Bag());
        return wrapTree(std::move(tree), sm);
    });
    return nullptr;
}

slang_syntax_tree slang_syntax_tree_from_file(slang_source_manager sm, const char* path,
                                              size_t path_len, slang_options options,
                                              slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!sm || !path) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        auto result = SyntaxTree::fromFile(toView(path, path_len), sm->sm,
                                           options ? options->toBag() : Bag());
        if (!result) {
            setError(err, SLANG_ERR_IO,
                     fmt::format("{}: {}", result.error().second, result.error().first.message()));
            return nullptr;
        }
        return wrapTree(std::move(*result), sm);
    });
    return nullptr;
}

slang_syntax_tree slang_syntax_tree_from_buffer(slang_source_manager sm, slang_buffer_id buffer,
                                                slang_options options, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!sm || !buffer) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        SourceBuffer sb;
        sb.id = BufferID(buffer, ""sv);
        sb.data = sm->sm.getSourceText(sb.id);
        auto tree = SyntaxTree::fromBuffer(sb, sm->sm, options ? options->toBag() : Bag());
        return wrapTree(std::move(tree), sm);
    });
    return nullptr;
}

slang_syntax_tree slang_syntax_tree_retain(slang_syntax_tree tree) {
    if (tree)
        tree->refs.fetch_add(1, std::memory_order_relaxed);
    return tree;
}

void slang_syntax_tree_release(slang_syntax_tree tree) {
    if (tree && tree->refs.fetch_sub(1, std::memory_order_acq_rel) == 1)
        delete tree;
}

slang_node slang_syntax_tree_root(slang_syntax_tree tree) {
    if (!tree)
        return noNode(nullptr);
    return toC(&tree->tree->root(), tree);
}

slang_diagnostics slang_syntax_tree_diagnostics(slang_syntax_tree tree, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!tree) {
        setError(err, SLANG_ERR_INVALID_ARG, "null tree");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        auto result = new slang_diagnostics_t();
        result->sm = tree->sm;
        result->comp = nullptr;
        auto& diags = tree->tree->diagnostics();
        result->diags.assign(diags.begin(), diags.end());
        return result;
    });
    return nullptr;
}

slang_source_manager slang_syntax_tree_source_manager(slang_syntax_tree tree) {
    return tree ? tree->sm : nullptr;
}

slang_str slang_syntax_tree_to_string(slang_syntax_tree tree, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    if (!tree) {
        setError(err, SLANG_ERR_INVALID_ARG, "null tree");
        return borrowed("");
    }
    SLANG_C_GUARD(err, { return owned(SyntaxPrinter::printFile(*tree->tree)); });
    return borrowed("");
}

// ---- Nodes ------------------------------------------------------------------

bool slang_node_is_null(slang_node node) {
    return node.ptr == nullptr;
}

slang_node slang_node_parent(slang_node node) {
    SLANG_C_ACCESS(node, {
        auto n = fromC(node);
        if (!n)
            return node;
        return toC(n->parent.get(), node.tree);
    });
}

uint32_t slang_node_struct(slang_node node) {
    SLANG_C_ACCESS(UINT32_MAX, { return slang_syntax_kind_struct(node.kind); });
}

uint32_t slang_node_child_count(slang_node node) {
    SLANG_C_ACCESS(0, {
        auto n = fromC(node);
        return n ? (uint32_t)n->getChildCount() : 0;
    });
}

slang_child_tag slang_node_child(slang_node node, uint32_t index, slang_node* node_out,
                                 slang_token* token_out) {
    SLANG_C_ACCESS(SLANG_CHILD_NONE, {
    auto n = fromC(node);
    if (!n || index >= n->getChildCount())
        return SLANG_CHILD_NONE;

    if (auto child = n->childNode(index)) {
        if (node_out)
            *node_out = toC(child, node.tree);
        return SLANG_CHILD_NODE;
    }

    auto token = n->childToken(index);
    if (!token)
        return SLANG_CHILD_NONE;

    if (token_out)
        *token_out = toC(n, index, token, node.tree);
    return SLANG_CHILD_TOKEN;
    });
}

bool slang_node_member_span(slang_node node, uint32_t member, uint32_t* start_out,
                            uint32_t* len_out) {
    SLANG_C_ACCESS(false, {
        auto n = fromC(node);
        if (!n)
            return false;
        uint32_t start = 0, len = 0;
        if (!gen::memberSpan(*n, member, start, len))
            return false;
        if (start_out)
            *start_out = start;
        if (len_out)
            *len_out = len;
        return true;
    });
}

slang_range slang_node_range(slang_node node) {
    SLANG_C_ACCESS(slang_range{}, {
        auto n = fromC(node);
        if (!n)
            return slang_range{};
        return toC(n->sourceRange());
    });
}

slang_str slang_node_to_string(slang_node node, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    auto n = fromC(node);
    if (!n) {
        setError(err, SLANG_ERR_INVALID_ARG, "null node");
        return borrowed("");
    }
    SLANG_C_GUARD(err, { return owned(losslessPrinter().print(*n).str()); });
    return borrowed("");
}

// Locates the first (or last) token in a subtree as an (owner, index) pair.
static bool findEdgeToken(const SyntaxNode& n, bool first, slang_syntax_tree tree,
                          slang_token& out) {
    size_t count = n.getChildCount();
    for (size_t k = 0; k < count; k++) {
        size_t i = first ? k : count - 1 - k;
        if (auto child = n.childNode(i)) {
            if (findEdgeToken(*child, first, tree, out))
                return true;
        }
        else if (auto token = n.childToken(i)) {
            out = toC(&n, (uint32_t)i, token, tree);
            return true;
        }
    }
    return false;
}

slang_token slang_node_first_token(slang_node node) {
    SLANG_C_ACCESS(noToken(node.tree), {
        slang_token out = noToken(node.tree);
        if (auto n = fromC(node))
            findEdgeToken(*n, true, node.tree, out);
        return out;
    });
}

slang_token slang_node_last_token(slang_node node) {
    SLANG_C_ACCESS(noToken(node.tree), {
        slang_token out = noToken(node.tree);
        if (auto n = fromC(node))
            findEdgeToken(*n, false, node.tree, out);
        return out;
    });
}

bool slang_node_is_equivalent(slang_node a, slang_node b) {
    SLANG_C_ACCESS(false, {
        auto na = fromC(a);
        auto nb = fromC(b);
        if (!na || !nb)
            return na == nb;
        return na->isEquivalentTo(*nb);
    });
}

// ---- Tokens -----------------------------------------------------------------

slang_loc slang_token_location(slang_token token) {
    SLANG_C_ACCESS(slang_loc{}, {
        auto t = fromC(token);
        return t ? toC(t.location()) : slang_loc{};
    });
}

slang_range slang_token_range(slang_token token) {
    SLANG_C_ACCESS(slang_range{}, {
        auto t = fromC(token);
        return t ? toC(t.range()) : slang_range{};
    });
}

// Missing tokens were synthesized by error recovery and occupy no source text,
// even though slang gives them their kind's canonical spelling.
static std::string_view rawText(Token token) {
    return token.isMissing() ? std::string_view() : token.rawText();
}

slang_str slang_token_raw_text(slang_token token) {
    SLANG_C_ACCESS(borrowed(""), {
        auto t = fromC(token);
        return t ? borrowed(rawText(t)) : borrowed("");
    });
}

slang_str slang_token_value_text(slang_token token) {
    SLANG_C_ACCESS(borrowed(""), {
        auto t = fromC(token);
        return t ? borrowed(t.valueText()) : borrowed("");
    });
}

uint32_t slang_token_trivia_count(slang_token token) {
    SLANG_C_ACCESS(0, {
        auto t = fromC(token);
        return t ? (uint32_t)t.trivia().size() : 0;
    });
}

bool slang_token_trivia(slang_token token, uint32_t index, slang_trivia* out) {
    SLANG_C_ACCESS(false, {
        auto t = fromC(token);
        if (!t)
            return false;
        auto trivia = t.trivia();
        if (index >= trivia.size())
            return false;
        if (out) {
            auto& tv = trivia[index];
            *out = slang_trivia{(uint32_t)tv.kind, 0, triviaText(tv)};
        }
        return true;
    });
}

slang_node slang_token_trivia_syntax(slang_token token, uint32_t index) {
    SLANG_C_ACCESS(noNode(token.tree), {
        auto t = fromC(token);
        if (!t)
            return noNode(token.tree);
        auto trivia = t.trivia();
        if (index >= trivia.size())
            return noNode(token.tree);
        return toC(trivia[index].syntax(), token.tree);
    });
}

// ---- Traversal --------------------------------------------------------------

static bool visitRec(const SyntaxNode& n, slang_syntax_tree tree, slang_node_visitor visitor,
                     void* user) {
    switch (visitor(toC(&n, tree), user)) {
        case SLANG_VISIT_BREAK:
            return false;
        case SLANG_VISIT_SKIP:
            return true;
        case SLANG_VISIT_CONTINUE:
            break;
    }
    size_t count = n.getChildCount();
    for (size_t i = 0; i < count; i++) {
        if (auto child = n.childNode(i)) {
            if (!visitRec(*child, tree, visitor, user))
                return false;
        }
    }
    return true;
}

void slang_node_visit(slang_node node, slang_node_visitor visitor, void* user, slang_error* err) {
    if (!checkEntry(err))
        return;
    auto n = fromC(node);
    if (!n || !visitor) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, {
        if (!visitRec(*n, node.tree, visitor, user))
            setError(err, SLANG_ERR_CANCELLED, "traversal cancelled by visitor");
    });
}

static void walkChild(const SyntaxNode& n, size_t i, const slang_syntax_sink& sink, void* user);

// Emits one event per declared member of the node's struct, grouping the
// flattened children that belong to list members under start_list/finish_list.
static void walkRec(const SyntaxNode& n, const slang_syntax_sink& sink, void* user) {
    uint32_t structId = slang_syntax_kind_struct((uint32_t)n.kind);
    sink.start_node(user, (uint32_t)n.kind, structId);

    uint32_t members = slang_syntax_struct_member_count(structId);
    uint32_t start = 0, len = 0;
    for (uint32_t m = 0; m < members; m++) {
        if (!gen::memberSpan(n, m, start, len))
            break;
        auto form = slang_syntax_member_form(structId, m);
        if (form == SLANG_MEMBER_LIST || form == SLANG_MEMBER_SEPARATED_LIST ||
            form == SLANG_MEMBER_TOKEN_LIST) {
            sink.start_list(user, form);
            for (uint32_t i = start; i < start + len; i++)
                walkChild(n, i, sink, user);
            sink.finish_list(user);
        }
        else {
            walkChild(n, start, sink, user);
        }
    }
    sink.finish_node(user);
}

static void walkChild(const SyntaxNode& n, size_t i, const slang_syntax_sink& sink, void* user) {
    if (auto child = n.childNode(i)) {
        walkRec(*child, sink, user);
    }
    else if (auto token = n.childToken(i)) {
        for (auto& tv : token.trivia()) {
            auto text = triviaText(tv);
            sink.trivia(user, (uint32_t)tv.kind, text.data, text.len);
            slang_str_free(text);
        }
        auto text = rawText(token);
        sink.token(user, (uint32_t)token.kind, text.data(), text.size(), toC(token.location()),
                   token.isMissing() ? SLANG_TOKEN_MISSING : 0);
    }
    else {
        sink.absent(user);
    }
}

void slang_syntax_tree_walk(slang_syntax_tree tree, const slang_syntax_sink* sink, void* user,
                            slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!tree || !sink || !sink->start_node || !sink->trivia || !sink->token || !sink->absent ||
        !sink->start_list || !sink->finish_list ||
        !sink->finish_node) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, { walkRec(tree->tree->root(), *sink, user); });
}

// ---- Unstable ---------------------------------------------------------------

#if SLANG_C_API_ALLOW_UNSTABLE
const void* slang_unstable_native_node_ptr(slang_node node) {
    return node.ptr;
}
#endif
