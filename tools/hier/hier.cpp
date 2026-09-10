//------------------------------------------------------------------------------
//! @file hier.cpp
//! @brief A tool for printing information about a Verilog hierarchy
//
// This tool is written against slang's C API (slang/c/slang.h) so that the
// C API always has an in-tree consumer exercising it.
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include <algorithm>
#include <boost_regex.hpp>
#include <cstdio>
#include <fmt/format.h>
#include <optional>
#include <string>
#include <string_view>
#include <unordered_map>

#include "slang/c/slang.h"

namespace {

std::string_view sv(slang_str s) {
    return std::string_view(s.data, s.len);
}

// Everything the per-instance callback needs.
struct Walk {
    slang_source_manager sm;
    boost::regex regex;
    boost::smatch match;
    bool haveRegex = false;
    bool params = false;
    int maxDepth = -1; // will never be 0, go full depth
    std::string instPrefix;
    std::optional<std::string> customFormat;
    slang_error err = SLANG_ERROR_INIT;

    // The prefix-matching and depth state of each visited instance, keyed by
    // its body, so that a child can pick up where its parent left off.
    struct State {
        int depth;
        int index;
    };
    std::unordered_map<const void*, State> states;

    // Finds the state of the nearest enclosing instance.
    State parentState(slang_ast instance) const {
        for (auto scope = slang_symbol_parent_scope(instance); !slang_ast_is_null(scope);
             scope = slang_symbol_parent_scope(scope)) {
            if (auto it = states.find(scope.ptr); it != states.end())
                return it->second;
        }
        return State{maxDepth, 0};
    }

    slang_visit visit(slang_ast instance) {
        auto definition = slang_instance_definition(instance);
        if (slang_definition_kind_of(definition) != SLANG_DEFINITION_MODULE)
            return SLANG_VISIT_SKIP;

        auto [depth, index] = parentState(instance);
        auto name = sv(slang_symbol_name(instance));
        int len = (int)name.length();
        int pathLength = (int)instPrefix.length();

        // if no instPrefix, pathLength is 0, and this check will never take place.
        // if index >= pathLength we satisfied the full instPrefix; from now on
        // we are limited only by max-depth
        if (index < pathLength) {
            if (name !=
                std::string_view(instPrefix).substr(index, std::min(pathLength - index, len))) {
                // current instance name did not match
                return SLANG_VISIT_SKIP;
            }
            index += len;
            if (index < pathLength && instPrefix[index] != '.')
                return SLANG_VISIT_SKIP; // separator needed, but didn't find one
            index++;                     // adjust for '.'
        }

        auto pathStr = slang_symbol_hierarchical_path(instance, &err);
        std::string s_inst(sv(pathStr));
        slang_str_free(pathStr);

        if (!haveRegex || boost::regex_search(s_inst, match, regex)) {
            auto s_module = sv(slang_symbol_name(definition));
            auto s_file = sv(slang_source_manager_file_name(sm, slang_symbol_location(definition)));
            if (customFormat.has_value())
                fmt::print("{}", fmt::format(fmt::runtime(customFormat.value()),
                                             fmt::arg("module", s_module), fmt::arg("inst", s_inst),
                                             fmt::arg("file", s_file)));
            else
                fmt::print("Module=\"{}\" Instance=\"{}\" File=\"{}\" ", s_module, s_inst, s_file);

            uint32_t size = slang_instance_parameter_count(instance);
            if (size && params) {
                fmt::print("Parameters: ");
                for (uint32_t i = 0; i < size; i++) {
                    auto p = slang_instance_parameter(instance, i);
                    slang_error valueErr = SLANG_ERROR_INIT;
                    auto value = slang_parameter_value(p, &valueErr);
                    std::string v(SLANG_SUCCESS(valueErr.status) ? sv(value) : "?");
                    slang_str_free(value);
                    fmt::print("{}={}{}", sv(slang_symbol_name(p)), v, i + 1 < size ? ", " : "");
                }
            }
            fmt::print("\n");
        }

        depth--;
        states[slang_instance_body(instance).ptr] = State{depth, index};
        return depth ? SLANG_VISIT_CONTINUE : SLANG_VISIT_SKIP;
    }
};

slang_visit visitor(slang_ast node, slang_ast, void* user) {
    if (node.domain == SLANG_AST_SYMBOL && slang_ast_kind_name(SLANG_AST_SYMBOL, node.kind).len &&
        sv(slang_ast_kind_name(SLANG_AST_SYMBOL, node.kind)) == "Instance") {
        return static_cast<Walk*>(user)->visit(node);
    }
    return SLANG_VISIT_CONTINUE;
}

bool failed(const slang_error& err) {
    if (SLANG_SUCCESS(err.status))
        return false;
    fmt::print(stderr, "slang-hier: {}: {}\n", slang_status_name(err.status), err.message);
    return true;
}

} // namespace

int main(int argc, char** argv) {
    slang_error err = SLANG_ERROR_INIT;
    slang_driver driver = slang_driver_create(&err);
    if (failed(err))
        return 1;

    auto add = [&](std::string_view names, slang_option_kind kind, std::string_view desc,
                   std::string_view valueName = {}) {
        slang_driver_add_option(driver, names.data(), names.size(), kind, desc.data(), desc.size(),
                                valueName.data(), valueName.size(), &err);
    };
    add("-h,--help", SLANG_OPTION_FLAG, "Display available options");
    add("--version", SLANG_OPTION_FLAG, "Display version information and exit");
    add("--params", SLANG_OPTION_FLAG, "Display instance parameter values");
    add("--max-depth", SLANG_OPTION_INT, "Maximum instance depth to be printed", "<depth>");
    add("--inst-prefix", SLANG_OPTION_STRING,
        "Skip all instance subtrees not under this prefix (inst.sub_inst...)", "<inst-prefix>");
    add("--inst-regex", SLANG_OPTION_STRING,
        "Show only instances matched by regex (scans whole tree)", "<inst-regex>");
    add("--custom-format", SLANG_OPTION_STRING,
        "Use libfmt-style strings to format output with {inst}, {module}, {file} as argument names",
        "<fmt::format string>");
    if (failed(err))
        return 1;

    if (!slang_driver_parse_args(driver, argc, argv, &err) || failed(err)) {
        slang_driver_destroy(driver);
        return 1;
    }

    bool flag = false;
    if (slang_driver_option_flag(driver, "-h,--help", 9, &flag) && flag) {
        std::string_view overview = "slang SystemVerilog compiler";
        auto help = slang_driver_help_text(driver, overview.data(), overview.size(), &err);
        printf("%s\n", std::string(sv(help)).c_str());
        slang_str_free(help);
        slang_driver_destroy(driver);
        return 0;
    }

    if (slang_driver_option_flag(driver, "--version", 9, &flag) && flag) {
        printf("slang version %s\n", slang_version_string());
        slang_driver_destroy(driver);
        return 0;
    }

    if (!slang_driver_process_options(driver, &err) || failed(err)) {
        slang_driver_destroy(driver);
        return 2;
    }

    bool ok = slang_driver_parse_sources(driver, &err);

    slang_compilation compilation = slang_driver_create_compilation(driver, 0, &err);
    if (failed(err)) {
        slang_driver_destroy(driver);
        return 2;
    }

    Walk walk;
    walk.sm = slang_driver_source_manager(driver);
    slang_driver_option_flag(driver, "--params", 8, &walk.params);
    int64_t maxDepth;
    if (slang_driver_option_int(driver, "--max-depth", 11, &maxDepth))
        walk.maxDepth = (int)maxDepth;
    slang_str str;
    if (slang_driver_option_string(driver, "--inst-prefix", 13, &str))
        walk.instPrefix = std::string(sv(str));
    if (slang_driver_option_string(driver, "--inst-regex", 12, &str)) {
        walk.regex = std::string(sv(str));
        walk.haveRegex = true;
    }
    if (slang_driver_option_string(driver, "--custom-format", 15, &str))
        walk.customFormat = std::string(sv(str));

    uint32_t count = slang_compilation_top_instance_count(compilation);
    for (uint32_t i = 0; i < count; i++) {
        slang_ast_visit(slang_compilation_top_instance(compilation, i), visitor, &walk, &err);
        if (failed(err))
            break;
    }

    slang_driver_report_compilation(driver, compilation, /* quiet */ false, &err);
    ok &= slang_driver_report_diagnostics(/* quiet */ driver, false, &err);

    slang_compilation_destroy(compilation);
    slang_driver_destroy(driver);
    return ok ? 0 : 3;
}
