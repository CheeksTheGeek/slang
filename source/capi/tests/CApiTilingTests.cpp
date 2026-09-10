// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//
// Property tests over the repository's own test corpus: for every syntax node
// in every file, the generated member spans tile the child index space
// exactly, and the streaming walk reproduces the source text byte for byte.

#include <catch2/catch_test_macros.hpp>
#include <filesystem>
#include <string>

#include "slang/c/slang.h"

namespace fs = std::filesystem;

namespace {

struct Stats {
    size_t files = 0, nodes = 0, failures = 0;
    std::string firstFailure;
};

void checkNode(slang_node node, Stats& stats) {
    stats.nodes++;
    auto st = slang_node_struct(node);
    uint32_t pos = 0, start = 0, len = 0;
    uint32_t members = slang_syntax_struct_member_count(st);
    for (uint32_t m = 0; m < members; m++) {
        if (!slang_node_member_span(node, m, &start, &len) || start != pos) {
            if (stats.failures++ == 0) {
                auto name = slang_syntax_kind_name(node.kind);
                stats.firstFailure = std::string(name.data, name.len) + " member " +
                                     std::to_string(m);
            }
            return;
        }
        pos += len;
    }
    if (pos != slang_node_child_count(node) ||
        slang_node_member_span(node, members, &start, &len)) {
        if (stats.failures++ == 0) {
            auto name = slang_syntax_kind_name(node.kind);
            stats.firstFailure = std::string(name.data, name.len) + " child count";
        }
    }
}

} // namespace

TEST_CASE("C API: member spans tile every node in the test corpus") {
    slang_error err = SLANG_ERROR_INIT;
    auto sm = slang_source_manager_create(&err);
    Stats stats;
    std::string walked;

    slang_syntax_sink sink{
        [](void*, uint32_t, uint32_t) {},
        [](void* u, uint32_t, const char* t, size_t n) {
            static_cast<std::string*>(u)->append(t, n);
        },
        [](void* u, uint32_t, const char* t, size_t n, slang_loc, uint16_t) {
            static_cast<std::string*>(u)->append(t, n);
        },
        [](void*) {},
        [](void*, slang_member_form) {},
        [](void*) {},
        [](void*) {},
    };

    for (auto& entry : fs::recursive_directory_iterator(SLANG_TEST_DIR)) {
        if (!entry.is_regular_file())
            continue;
        auto ext = entry.path().extension();
        if (ext != ".sv" && ext != ".v" && ext != ".svh")
            continue;

        auto path = entry.path().string();
        auto tree = slang_syntax_tree_from_file(sm, path.data(), path.size(), nullptr, &err);
        if (!tree) {
            err = SLANG_ERROR_INIT;
            continue;
        }
        stats.files++;

        slang_node_visit(
            slang_syntax_tree_root(tree),
            [](slang_node node, void* user) {
                checkNode(node, *static_cast<Stats*>(user));
                return SLANG_VISIT_CONTINUE;
            },
            &stats, &err);
        REQUIRE(SLANG_SUCCESS(err.status));

        walked.clear();
        slang_syntax_tree_walk(tree, &sink, &walked, &err);
        REQUIRE(SLANG_SUCCESS(err.status));
        auto expected = slang_node_to_string(slang_syntax_tree_root(tree), &err);
        std::string_view expectedText(expected.data, expected.len);
        if (walked != expectedText && stats.failures++ == 0) {
            size_t i = 0;
            while (i < walked.size() && i < expectedText.size() && walked[i] == expectedText[i])
                i++;
            size_t from = i > 30 ? i - 30 : 0;
            stats.firstFailure = "walk text mismatch in " + path + " at byte " + std::to_string(i) +
                                 "\n  walked:   [" + walked.substr(from, 70) + "]\n  expected: [" +
                                 std::string(expectedText.substr(from, 70)) + "]";
        }
        slang_str_free(expected);

        slang_syntax_tree_release(tree);
    }
    slang_source_manager_destroy(sm);

    INFO("first failure: " << stats.firstFailure);
    CHECK(stats.files > 50);
    CHECK(stats.nodes > 2000);
    CHECK(stats.failures == 0);
}
