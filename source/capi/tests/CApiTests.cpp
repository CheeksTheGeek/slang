// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT

#include <catch2/catch_test_macros.hpp>
#include <filesystem>
#include <fstream>
#include <set>
#include <string>
#include <string_view>

#include "slang/c/slang.h"

using namespace std::literals;

namespace {

std::string_view sv(slang_str s) {
    return std::string_view(s.data, s.len);
}

struct Session {
    slang_error err = SLANG_ERROR_INIT;
    slang_source_manager sm = nullptr;
    slang_syntax_tree tree = nullptr;
    slang_compilation comp = nullptr;

    explicit Session(std::string_view text, uint32_t flags = 0) {
        sm = slang_source_manager_create(&err);
        REQUIRE(SLANG_SUCCESS(err.status));

        slang_options options = slang_options_create(&err);
        slang_options_set_compilation_flags(options, flags);
        tree = slang_syntax_tree_from_text(sm, text.data(), text.size(), "test.sv", 7, nullptr, 0,
                                           options, &err);
        REQUIRE(SLANG_SUCCESS(err.status));
        REQUIRE(tree);

        comp = slang_compilation_create(options, &err);
        REQUIRE(SLANG_SUCCESS(err.status));
        slang_compilation_add_tree(comp, tree, &err);
        REQUIRE(SLANG_SUCCESS(err.status));
        slang_options_destroy(options);
    }

    ~Session() {
        slang_compilation_destroy(comp);
        slang_syntax_tree_release(tree);
        slang_source_manager_destroy(sm);
    }
};

std::string renderAll(slang_diagnostics diags, slang_error* err) {
    auto s = slang_diagnostics_render(diags, nullptr, err);
    std::string result(sv(s));
    slang_str_free(s);
    return result;
}

} // namespace

TEST_CASE("C API: version and model hashes") {
    CHECK(slang_c_version() == SLANG_C_VERSION);
    CHECK(std::string_view(slang_version_string()).find('.') != std::string_view::npos);
    CHECK(std::string_view(slang_syntax_model_hash()).size() == 64);
    CHECK(std::string_view(slang_diagnostics_model_hash()).size() == 64);
    CHECK(sv(slang_syntax_kind_name(0)) == "Unknown");
    CHECK(sv(slang_token_kind_name(0)) == "Unknown");
    CHECK(slang_syntax_kind_struct(0) == UINT32_MAX);
    CHECK(slang_syntax_kind_count() > 500);
    CHECK(slang_syntax_struct_count() > 400);
    CHECK(slang_token_kind_count() > 300);
    CHECK(slang_trivia_kind_count() > 5);
    CHECK(slang_ast_kind_count(SLANG_AST_SYMBOL) > 90);
    CHECK(sv(slang_ast_kind_name(SLANG_AST_SYMBOL, 0)) == "Unknown");
    CHECK(sv(slang_ast_kind_name(SLANG_AST_EXPRESSION, 0)) == "Invalid");
    CHECK(sv(slang_ast_kind_name(SLANG_AST_PATTERN, 9999)) == "");
    CHECK(std::string_view(slang_status_name(SLANG_ERR_IO)) == "SLANG_ERR_IO");
}

TEST_CASE("C API: every syntax kind maps to a struct with a name") {
    for (uint32_t k = 1; k < slang_syntax_kind_count(); k++) {
        auto s = slang_syntax_kind_struct(k);
        REQUIRE(s < slang_syntax_struct_count());
        CHECK(!sv(slang_syntax_struct_name(s)).empty());
        CHECK(!sv(slang_syntax_kind_name(k)).empty());
        for (uint32_t m = 0; m < slang_syntax_struct_member_count(s); m++)
            CHECK(!sv(slang_syntax_member_name(s, m)).empty());
    }
}

TEST_CASE("C API: error poisoning") {
    slang_error err = SLANG_ERROR_INIT;
    err.status = SLANG_ERR_IO;
    CHECK(slang_source_manager_create(&err) == nullptr);
    CHECK(err.status == SLANG_ERR_IO);

    err = SLANG_ERROR_INIT;
    CHECK(slang_syntax_tree_from_text(nullptr, "x", 1, nullptr, 0, nullptr, 0, nullptr, &err) ==
          nullptr);
    CHECK(err.status == SLANG_ERR_INVALID_ARG);
    CHECK(std::string_view(err.message) == "null argument");

    auto sm = slang_source_manager_create(nullptr);
    err = SLANG_ERROR_INIT;
    CHECK(slang_syntax_tree_from_file(sm, "/nonexistent/file.sv", 20, nullptr, &err) == nullptr);
    CHECK(err.status == SLANG_ERR_IO);
    CHECK(std::string_view(err.message).find("nonexistent") != std::string_view::npos);
    slang_source_manager_destroy(sm);
}

TEST_CASE("C API: parse, children, tokens, trivia") {
    Session s("module m; // hi\n  logic [7:0] a = 8'd5;\nendmodule\n");

    auto root = slang_syntax_tree_root(s.tree);
    REQUIRE(!slang_node_is_null(root));
    CHECK(sv(slang_syntax_kind_name(root.kind)) == "CompilationUnit");
    CHECK(slang_node_is_null(slang_node_parent(root)));

    // CompilationUnit: members list + EndOfFile token.
    REQUIRE(slang_node_child_count(root) == 2);
    slang_node child;
    slang_token token;
    REQUIRE(slang_node_child(root, 0, &child, &token) == SLANG_CHILD_NODE);
    CHECK(sv(slang_syntax_kind_name(child.kind)) == "ModuleDeclaration");
    CHECK(slang_node_parent(child).ptr == root.ptr);
    REQUIRE(slang_node_child(root, 1, &child, &token) == SLANG_CHILD_TOKEN);
    CHECK(sv(slang_token_kind_name(token.kind)) == "EndOfFile");
    CHECK(slang_node_child(root, 2, &child, &token) == SLANG_CHILD_NONE);

    auto first = slang_node_first_token(root);
    CHECK(sv(slang_token_kind_name(first.kind)) == "ModuleKeyword");
    CHECK(sv(slang_token_raw_text(first)) == "module");
    CHECK(slang_token_trivia_count(first) == 0);
    CHECK(slang_source_manager_line(s.sm, slang_token_location(first)) == 1);
    CHECK(slang_source_manager_column(s.sm, slang_token_location(first)) == 1);
    CHECK(sv(slang_source_manager_file_name(s.sm, slang_token_location(first))) == "test.sv");

    // The "logic" keyword carries the line comment + newline + indent as trivia.
    slang_node module;
    slang_node_child(root, 0, &module, nullptr);
    uint32_t start, len;
    CHECK(sv(slang_syntax_member_name(slang_node_struct(module), 2)) == "members");
    REQUIRE(slang_node_member_span(module, 2, &start, &len));
    REQUIRE(len == 1);
    slang_node dataDecl;
    REQUIRE(slang_node_child(module, start, &dataDecl, nullptr) == SLANG_CHILD_NODE);
    auto logic = slang_node_first_token(dataDecl);
    CHECK(sv(slang_token_raw_text(logic)) == "logic");
    REQUIRE(slang_token_trivia_count(logic) == 4);
    slang_trivia tv;
    REQUIRE(slang_token_trivia(logic, 0, &tv));
    CHECK(sv(slang_trivia_kind_name(tv.kind)) == "Whitespace");
    REQUIRE(slang_token_trivia(logic, 1, &tv));
    CHECK(sv(slang_trivia_kind_name(tv.kind)) == "LineComment");
    CHECK(sv(tv.text) == "// hi");
    REQUIRE(slang_token_trivia(logic, 2, &tv));
    CHECK(sv(slang_trivia_kind_name(tv.kind)) == "EndOfLine");
    CHECK(!slang_token_trivia(logic, 4, &tv));

    auto str = slang_node_to_string(dataDecl, &s.err);
    CHECK(sv(str) == " // hi\n  logic [7:0] a = 8'd5;");
    slang_str_free(str);

    CHECK(slang_node_is_equivalent(dataDecl, dataDecl));
    CHECK(!slang_node_is_equivalent(dataDecl, module));

    // Member spans tile the child index space exactly.
    auto st = slang_node_struct(module);
    uint32_t pos = 0;
    for (uint32_t m = 0; m < slang_syntax_struct_member_count(st); m++) {
        REQUIRE(slang_node_member_span(module, m, &start, &len));
        CHECK(start == pos);
        pos += len;
    }
    CHECK(pos == slang_node_child_count(module));
    CHECK(!slang_node_member_span(module, 999, &start, &len));
}

TEST_CASE("C API: syntax visit and walk") {
    std::string text = "module m;\n  initial begin a = 1; b = 2; end\nendmodule\n";
    Session s(text);

    int visited = 0;
    slang_node_visit(
        slang_syntax_tree_root(s.tree),
        [](slang_node, void* user) {
            (*static_cast<int*>(user))++;
            return SLANG_VISIT_CONTINUE;
        },
        &visited, &s.err);
    CHECK(SLANG_SUCCESS(s.err.status));
    CHECK(visited > 10);

    int seen = 0;
    slang_node_visit(
        slang_syntax_tree_root(s.tree),
        [](slang_node, void* user) {
            return ++*static_cast<int*>(user) == 3 ? SLANG_VISIT_BREAK : SLANG_VISIT_CONTINUE;
        },
        &seen, &s.err);
    CHECK(s.err.status == SLANG_ERR_CANCELLED);
    CHECK(seen == 3);
    s.err = SLANG_ERROR_INIT;

    struct Sink {
        std::string text;
        int depth = 0, maxDepth = 0, absent = 0, lists = 0;
    } sink;
    slang_syntax_sink cb{
        [](void* u, uint32_t, uint32_t) {
            auto& sk = *static_cast<Sink*>(u);
            sk.maxDepth = std::max(sk.maxDepth, ++sk.depth);
        },
        [](void* u, uint32_t, const char* t, size_t n) {
            static_cast<Sink*>(u)->text.append(t, n);
        },
        [](void* u, uint32_t, const char* t, size_t n, slang_loc, uint16_t) {
            static_cast<Sink*>(u)->text.append(t, n);
        },
        [](void* u) { static_cast<Sink*>(u)->absent++; },
        [](void* u, slang_member_form) { static_cast<Sink*>(u)->lists++; },
        [](void* u) { static_cast<Sink*>(u)->lists--; },
        [](void* u) { static_cast<Sink*>(u)->depth--; },
    };
    slang_syntax_tree_walk(s.tree, &cb, &sink, &s.err);
    CHECK(SLANG_SUCCESS(s.err.status));
    CHECK(sink.text == text);
    CHECK(sink.depth == 0);
    CHECK(sink.maxDepth > 4);
    CHECK(sink.lists == 0);
    CHECK(sink.absent > 0);
}

TEST_CASE("C API: diagnostics") {
    Session s("module m;\n  foo f();\n  int x = ;\nendmodule\n");

    auto parseDiags = slang_syntax_tree_diagnostics(s.tree, &s.err);
    REQUIRE(parseDiags);
    REQUIRE(slang_diagnostics_count(parseDiags) >= 1);
    slang_diag d;
    REQUIRE(slang_diagnostics_at(parseDiags, 0, &d));
    CHECK(d.severity == SLANG_SEVERITY_ERROR);
    CHECK(slang_source_manager_line(s.sm, d.location) == 3);
    auto msg = slang_diagnostics_message(parseDiags, 0, &s.err);
    CHECK(sv(msg).find("expected expression") != std::string_view::npos);
    slang_str_free(msg);
    CHECK(slang_ast_is_null(slang_diagnostics_symbol(parseDiags, 0)));
    slang_diagnostics_destroy(parseDiags);

    auto all = slang_compilation_diagnostics(s.comp, &s.err);
    REQUIRE(all);
    REQUIRE(slang_diagnostics_count(all) >= 2);
    auto text = renderAll(all, &s.err);
    CHECK(SLANG_SUCCESS(s.err.status));
    CHECK(text.find("test.sv:2:3: error: unknown module 'foo'") != std::string::npos);
    CHECK(text.find("foo f();") != std::string::npos);
    CHECK(text.find("test.sv:3:11: error: expected expression") != std::string::npos);

    bool sawUnknownModule = false;
    for (uint32_t i = 0; i < slang_diagnostics_count(all); i++) {
        slang_diagnostics_at(all, i, &d);
        auto m = slang_diagnostics_message(all, i, &s.err);
        if (sv(m).starts_with("unknown module")) {
            sawUnknownModule = true;
            CHECK(!slang_ast_is_null(slang_diagnostics_symbol(all, i)));
        }
        slang_str_free(m);
    }
    CHECK(sawUnknownModule);
    slang_diagnostics_destroy(all);
}

TEST_CASE("C API: compilation, freeze, symbols, lookup, types") {
    Session s(R"(
package p;
    localparam int W = 8;
endpackage
module leaf #(parameter int N = 4) (input logic [N-1:0] in);
    logic [N-1:0] q = ~in;
endmodule
module top;
    import p::*;
    logic [W-1:0] a;
    leaf #(.N(W)) l1(.in(a));
    leaf #(.N(W)) l2(.in(a));
    leaf #(.N(2)) l3(.in(a[1:0]));
endmodule
)",
              SLANG_COMP_DISABLE_INSTANCE_CACHING);

    slang_freeze_report report;
    slang_compilation_freeze(s.comp, SLANG_FREEZE_ALL, &report, &s.err);
    REQUIRE(SLANG_SUCCESS(s.err.status));
    CHECK(slang_compilation_is_sealed(s.comp));
    CHECK(report.symbols_elaborated > 10);
    CHECK(report.expressions_visited > 5);
    CHECK(report.expressions_folded > 0);
    CHECK(report.fold_failures == 0);

    // Freeze is idempotent.
    slang_compilation_freeze(s.comp, SLANG_FREEZE_ALL, nullptr, &s.err);
    CHECK(SLANG_SUCCESS(s.err.status));

    // Adding a tree after finalization is rejected.
    slang_compilation_add_tree(s.comp, s.tree, &s.err);
    CHECK(s.err.status == SLANG_ERR_INVALID_STATE);
    s.err = SLANG_ERROR_INIT;

    auto root = slang_compilation_root(s.comp, &s.err);
    REQUIRE(!slang_ast_is_null(root));
    CHECK(sv(slang_ast_kind_name(SLANG_AST_SYMBOL, root.kind)) == "Root");
    CHECK(slang_ast_is_null(slang_symbol_parent_scope(root)));

    REQUIRE(slang_compilation_top_instance_count(s.comp) == 1);
    auto top = slang_compilation_top_instance(s.comp, 0);
    CHECK(sv(slang_symbol_name(top)) == "top");
    CHECK(sv(slang_ast_kind_name(SLANG_AST_SYMBOL, top.kind)) == "Instance");
    CHECK(slang_symbol_is_scope(top));
    CHECK(slang_compilation_definition_count(s.comp) == 2);
    bool sawPackage = false;
    for (uint32_t i = 0; i < slang_compilation_package_count(s.comp); i++)
        sawPackage |= sv(slang_symbol_name(slang_compilation_package(s.comp, i))) == "p";
    CHECK(sawPackage);

    auto def = slang_instance_definition(top);
    CHECK(sv(slang_symbol_name(def)) == "top");
    CHECK(slang_definition_kind_of(def) == SLANG_DEFINITION_MODULE);

    // Iterate the body without allocation.
    auto body = slang_instance_body(top);
    int members = 0;
    for (auto m = slang_scope_first_member(body, &s.err); !slang_ast_is_null(m);
         m = slang_symbol_next_sibling(m))
        members++;
    CHECK(members >= 5);

    // Lookups.
    auto l2 = slang_scope_lookup(top, "l2", 2, &s.err);
    REQUIRE(!slang_ast_is_null(l2));
    auto path = slang_symbol_hierarchical_path(l2, &s.err);
    CHECK(sv(path) == "top.l2");
    slang_str_free(path);

    auto q = slang_scope_lookup(root, "top.l3.q", 8, &s.err);
    REQUIRE(!slang_ast_is_null(q));
    CHECK(slang_symbol_is_value(q));
    auto qType = slang_value_type(q, &s.err);
    REQUIRE(!slang_ast_is_null(qType));
    CHECK(slang_symbol_is_type(qType));
    auto typeName = slang_type_to_string(qType, &s.err);
    CHECK(sv(typeName) == "logic[1:0]");
    slang_str_free(typeName);
    CHECK(slang_type_bit_width(qType) == 2);
    CHECK(slang_type_is_integral(qType));
    CHECK(slang_type_is_four_state(qType));
    CHECK(!slang_type_is_signed(qType));

    auto a = slang_scope_find(body, "a", 1, &s.err);
    auto aType = slang_value_type(a, &s.err);
    CHECK(slang_type_bit_width(aType) == 8);
    CHECK(!slang_type_is_matching(aType, qType));
    CHECK(slang_type_is_assignment_compatible(aType, qType));

    // The parameter W is folded by the freeze; read it without evaluating.
    auto w = slang_scope_lookup(top, "W", 1, &s.err);
    INFO(s.err.message);
    REQUIRE(SLANG_SUCCESS(s.err.status));
    REQUIRE(!slang_ast_is_null(w));
    auto init = slang_value_initializer(w, &s.err);
    REQUIRE(!slang_ast_is_null(init));
    CHECK(init.domain == SLANG_AST_EXPRESSION);
    slang_str value;
    REQUIRE(slang_expression_cached_constant(init, &value, &s.err));
    CHECK(sv(value) == "8");
    slang_str_free(value);
    CHECK(!slang_expression_is_bad(init));
    CHECK(slang_type_bit_width(slang_expression_type(init)) == 32);

    // The initializer of q is not constant.
    auto qInit = slang_value_initializer(q, &s.err);
    REQUIRE(!slang_ast_is_null(qInit));
    CHECK(!slang_expression_cached_constant(qInit, &value, &s.err));
    CHECK(!slang_expression_eval(qInit, &value, &s.err));
    CHECK(SLANG_SUCCESS(s.err.status));
    // ~in refers to no single symbol; the operand does.
    CHECK(slang_ast_is_null(slang_expression_symbol(qInit)));

    // Syntax round trip from the AST.
    auto syntax = slang_ast_syntax(l2);
    REQUIRE(!slang_node_is_null(syntax));
    CHECK(syntax.tree == s.tree);
    auto range = slang_ast_range(l2);
    CHECK(slang_source_manager_line(s.sm, range.start) == 12);

    // Visit the whole design.
    struct Counts {
        int symbols = 0, exprs = 0, withParent = 0;
    } counts;
    slang_ast_visit(
        root,
        [](slang_ast node, slang_ast parent, void* user) {
            auto& c = *static_cast<Counts*>(user);
            if (node.domain == SLANG_AST_SYMBOL)
                c.symbols++;
            else if (node.domain == SLANG_AST_EXPRESSION)
                c.exprs++;
            if (!slang_ast_is_null(parent))
                c.withParent++;
            return SLANG_VISIT_CONTINUE;
        },
        &counts, &s.err);
    CHECK(SLANG_SUCCESS(s.err.status));
    CHECK(counts.symbols == (int)report.symbols_elaborated);
    CHECK(counts.exprs == (int)report.expressions_visited);
    CHECK(counts.withParent == counts.symbols + counts.exprs - 1);

    // Instance caching disabled: all three leaf bodies are distinct.
    auto l1 = slang_scope_lookup(top, "l1", 2, &s.err);
    CHECK(slang_instance_body(l1).ptr != slang_instance_body(l2).ptr);

    auto diags = slang_compilation_diagnostics(s.comp, &s.err);
    auto rendered = renderAll(diags, &s.err);
    INFO(rendered);
    CHECK(rendered.find("error") == std::string::npos);
    slang_diagnostics_destroy(diags);
}

TEST_CASE("C API: lookup of a variable in a frozen top instance") {
    Session s("module top; localparam int W = 8; logic [W-1:0] q; endmodule\n",
              SLANG_COMP_DISABLE_INSTANCE_CACHING);
    slang_freeze_report report;
    slang_compilation_freeze(s.comp, SLANG_FREEZE_ALL, &report, &s.err);
    REQUIRE(SLANG_SUCCESS(s.err.status));
    auto top = slang_compilation_top_instance(s.comp, 0);
    REQUIRE(sv(slang_symbol_name(top)) == "top");
    auto q = slang_scope_lookup(top, "q", 1, &s.err);
    INFO(s.err.message);
    CHECK(SLANG_SUCCESS(s.err.status));
    CHECK(!slang_ast_is_null(q));
    auto w = slang_scope_find(slang_instance_body(top), "W", 1, &s.err);
    CHECK(!slang_ast_is_null(w));
    auto q2 = slang_scope_find(slang_instance_body(top), "q", 1, &s.err);
    CHECK(!slang_ast_is_null(q2));
}

TEST_CASE("C API: driver") {
    // Write a small design to a temp file so the driver can load it by path.
    auto path = (std::filesystem::temp_directory_path() / "slang_capi_driver_test.sv").string();
    {
        std::ofstream f(path);
        f << "module leaf #(parameter int N = 4, parameter type T = logic) (input logic clk);\n"
             "    T q;\nendmodule\n"
             "module top;\n    logic clk;\n    leaf #(.N(8)) l(.clk(clk));\nendmodule\n";
    }

    slang_error err = SLANG_ERROR_INIT;
    auto driver = slang_driver_create(&err);
    REQUIRE(driver);
    slang_driver_add_option(driver, "--params", 8, SLANG_OPTION_FLAG, "show params", 11, nullptr, 0,
                            &err);
    slang_driver_add_option(driver, "--max-depth", 11, SLANG_OPTION_INT, "depth", 5, "<depth>", 7,
                            &err);
    slang_driver_add_option(driver, "--inst-prefix", 13, SLANG_OPTION_STRING, "prefix", 6,
                            "<prefix>", 8, &err);
    REQUIRE(SLANG_SUCCESS(err.status));

    // Registering the same option twice is an error.
    slang_driver_add_option(driver, "--params", 8, SLANG_OPTION_FLAG, "dup", 3, nullptr, 0, &err);
    CHECK(err.status == SLANG_ERR_INVALID_ARG);
    err = SLANG_ERROR_INIT;

    const char* argv[] = {"prog", path.c_str(), "--params", "--max-depth", "3", "--top", "top"};
    REQUIRE(slang_driver_parse_args(driver, 7, argv, &err));
    REQUIRE(SLANG_SUCCESS(err.status));

    bool flag = false;
    CHECK(slang_driver_option_flag(driver, "--params", 8, &flag));
    CHECK(flag);
    int64_t depth = 0;
    CHECK(slang_driver_option_int(driver, "--max-depth", 11, &depth));
    CHECK(depth == 3);
    slang_str prefix;
    CHECK(!slang_driver_option_string(driver, "--inst-prefix", 13, &prefix));
    CHECK(!slang_driver_option_int(driver, "--unknown", 9, &depth));

    auto help = slang_driver_help_text(driver, "overview text", 13, &err);
    CHECK(sv(help).find("overview text") != std::string_view::npos);
    CHECK(sv(help).find("--inst-prefix") != std::string_view::npos);
    CHECK(sv(help).find("<depth>") != std::string_view::npos);
    slang_str_free(help);

    REQUIRE(slang_driver_process_options(driver, &err));
    REQUIRE(slang_driver_parse_sources(driver, &err));
    REQUIRE(slang_driver_tree_count(driver) == 1);
    CHECK(slang_syntax_tree_source_manager(slang_driver_tree(driver, 0)) ==
          slang_driver_source_manager(driver));

    auto comp = slang_driver_create_compilation(driver, 0, &err);
    REQUIRE(comp);
    REQUIRE(SLANG_SUCCESS(err.status));
    CHECK(slang_compilation_source_manager(comp) == slang_driver_source_manager(driver));
    REQUIRE(slang_compilation_top_instance_count(comp) == 1);
    auto top = slang_compilation_top_instance(comp, 0);
    CHECK(sv(slang_symbol_name(top)) == "top");

    auto l = slang_scope_lookup(top, "l", 1, &err);
    REQUIRE(!slang_ast_is_null(l));
    REQUIRE(slang_instance_parameter_count(l) == 2);
    auto n = slang_instance_parameter(l, 0);
    auto t = slang_instance_parameter(l, 1);
    CHECK(sv(slang_symbol_name(n)) == "N");
    CHECK(sv(slang_symbol_name(t)) == "T");
    auto nValue = slang_parameter_value(n, &err);
    CHECK(sv(nValue) == "8");
    slang_str_free(nValue);
    auto tValue = slang_parameter_value(t, &err);
    CHECK(sv(tValue) == "logic");
    slang_str_free(tValue);
    CHECK(slang_ast_is_null(slang_instance_parameter(l, 2)));
    slang_parameter_value(top, &err);
    CHECK(err.status == SLANG_ERR_INVALID_ARG);
    err = SLANG_ERROR_INIT;

    // The syntax of a driver-loaded symbol maps back to the driver's tree.
    auto syntax = slang_ast_syntax(l);
    REQUIRE(!slang_node_is_null(syntax));
    CHECK(syntax.tree == slang_driver_tree(driver, 0));

    slang_driver_report_compilation(driver, comp, /* quiet */ true, &err);
    CHECK(slang_driver_report_diagnostics(driver, /* quiet */ true, &err));
    CHECK(SLANG_SUCCESS(err.status));

    slang_compilation_destroy(comp);
    slang_driver_destroy(driver);
    std::filesystem::remove(path);
}

TEST_CASE("C API: analysis (unused lints and drivers)") {
    Session s(R"(
module m(input logic clk, output logic o);
    logic unused_signal;
    logic driven;
    assign o = driven;
    always_ff @(posedge clk) driven <= 1'b1;
endmodule
)");
    slang_freeze_report report;
    slang_compilation_freeze(s.comp, SLANG_FREEZE_PREFOLD | SLANG_FREEZE_SEAL, &report, &s.err);
    REQUIRE(SLANG_SUCCESS(s.err.status));

    auto analysis = slang_analysis_run(s.comp, SLANG_ANALYSIS_CHECK_UNUSED, 1, &s.err);
    REQUIRE(analysis);
    REQUIRE(SLANG_SUCCESS(s.err.status));

    auto diags = slang_analysis_diagnostics(analysis, &s.err);
    REQUIRE(diags);
    bool sawUnused = false;
    for (uint32_t i = 0; i < slang_diagnostics_count(diags); i++) {
        auto msg = slang_diagnostics_message(diags, i, &s.err);
        if (sv(msg).find("unused_signal") != std::string_view::npos)
            sawUnused = true;
        slang_str_free(msg);
    }
    CHECK(sawUnused);
    slang_diagnostics_destroy(diags);

    // `driven` has two drivers: the always_ff and... only the procedural one
    // writes it; `o` is driven by the continuous assign.
    auto top = slang_compilation_top_instance(s.comp, 0);
    auto body = slang_instance_body(top);
    auto driven = slang_scope_find(body, "driven", 6, &s.err);
    REQUIRE(!slang_ast_is_null(driven));
    uint32_t drivers = slang_analysis_driver_count(analysis, driven);
    CHECK(drivers >= 1);
    slang_driver_info info;
    REQUIRE(slang_analysis_driver(analysis, driven, 0, &info));
    CHECK(slang_source_manager_line(s.sm, info.range.start) >= 1);
    CHECK(!slang_ast_is_null(info.containing_symbol));

    auto o = slang_scope_find(body, "o", 1, &s.err);
    REQUIRE(slang_analysis_driver_count(analysis, o) >= 1);
    slang_analysis_driver(analysis, o, 0, &info);
    CHECK(info.kind == SLANG_DRIVER_CONTINUOUS);

    slang_analysis_destroy(analysis);
}

TEST_CASE("C API: custom dataflow analysis") {
    // A "definitely assigned" forward lattice: the state is the set of value
    // symbols assigned on all paths reaching a point. WRITE adds a symbol; the
    // merges intersect (a var is definitely assigned only if assigned on every
    // incoming path).
    using SymSet = std::set<const void*>;

    struct Lattice {
        static void* top(void*) { return new SymSet(); }
        static void* bottom(void*) {
            // Unreachable: everything is (vacuously) assigned. Represent with a
            // sentinel: a null-marked set meaning "universe". We use a heap flag.
            auto* s = new SymSet();
            s->insert(nullptr); // sentinel marks the universe
            return s;
        }
        static void* clone(void*, const void* s) {
            return new SymSet(*static_cast<const SymSet*>(s));
        }
        static void intersectInto(SymSet& into, const SymSet& other) {
            // If either is the universe (unreachable), the result is the other.
            if (other.count(nullptr))
                return; // into unchanged
            if (into.count(nullptr)) {
                into = other;
                return;
            }
            SymSet result;
            for (auto* p : into)
                if (other.count(p))
                    result.insert(p);
            into = std::move(result);
        }
        static void join(void* u, void* into, const void* other) {
            (void)u;
            intersectInto(*static_cast<SymSet*>(into), *static_cast<const SymSet*>(other));
        }
        static void meet(void* u, void* into, const void* other) { join(u, into, other); }
        static void transfer(void*, void* state, const slang_dfa_event* ev) {
            if (ev->kind == SLANG_DFA_WRITE && !slang_ast_is_null(ev->symbol))
                static_cast<SymSet*>(state)->insert(ev->symbol.ptr);
        }
        static void drop(void*, void* s) { delete static_cast<SymSet*>(s); }
    };

    slang_dfa_lattice lattice{Lattice::top,  Lattice::bottom,   Lattice::clone, Lattice::join,
                              Lattice::meet, Lattice::transfer, Lattice::drop};

    Session s(R"(
module m(input logic c);
    logic x, y, z;
    always_comb begin
        if (c) x = 1;   // x only on one path
        y = 2;          // y on all paths
        z = y;          // read y, write z
    end
endmodule
)");
    slang_freeze_report report;
    slang_compilation_freeze(s.comp, SLANG_FREEZE_PREFOLD | SLANG_FREEZE_SEAL, &report, &s.err);
    REQUIRE(SLANG_SUCCESS(s.err.status));

    auto top = slang_compilation_top_instance(s.comp, 0);
    auto body = slang_instance_body(top);

    // Find the always_comb procedural block by iterating members.
    slang_ast block = slang_ast{nullptr, s.comp, 0, SLANG_AST_SYMBOL};
    for (auto m = slang_scope_first_member(body, &s.err); !slang_ast_is_null(m);
         m = slang_symbol_next_sibling(m)) {
        if (sv(slang_ast_kind_name(SLANG_AST_SYMBOL, m.kind)) == "ProceduralBlock") {
            block = m;
            break;
        }
    }
    REQUIRE(!slang_ast_is_null(block));

    auto* exit = static_cast<SymSet*>(slang_dfa_run(s.comp, block, &lattice, nullptr, &s.err));
    REQUIRE(SLANG_SUCCESS(s.err.status));
    REQUIRE(exit != nullptr);

    // y and z are definitely assigned at exit; x is not (conditional).
    auto x = slang_scope_find(body, "x", 1, &s.err);
    auto y = slang_scope_find(body, "y", 1, &s.err);
    auto z = slang_scope_find(body, "z", 1, &s.err);
    CHECK(exit->count(y.ptr) == 1);
    CHECK(exit->count(z.ptr) == 1);
    CHECK(exit->count(x.ptr) == 0);

    Lattice::drop(nullptr, exit);
}

TEST_CASE("C API: ELABORATE_ALL requires instance caching disabled") {
    Session s("module m; endmodule");
    slang_compilation_freeze(s.comp, SLANG_FREEZE_ELABORATE_ALL, nullptr, &s.err);
    CHECK(s.err.status == SLANG_ERR_INVALID_ARG);
    s.err = SLANG_ERROR_INIT;
    slang_compilation_freeze(s.comp, SLANG_FREEZE_PREFOLD | SLANG_FREEZE_SEAL, nullptr, &s.err);
    CHECK(SLANG_SUCCESS(s.err.status));
}
