//------------------------------------------------------------------------------
// CApiDriver.cpp
// C API: the command-line driver
//
// SPDX-FileCopyrightText: Michael Popoloski
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"
#include <map>
#include <optional>

#include "slang/driver/Driver.h"

using namespace slang;
using namespace slang::capi;

struct slang_driver_t {
    driver::Driver driver;
    slang_source_manager_t sm;
    std::vector<slang_syntax_tree> trees;

    // Registered custom options. CommandLine stores references to the value
    // slots, so they live behind stable heap addresses.
    struct Option {
        slang_option_kind kind;
        std::optional<bool> flag;
        std::optional<int64_t> integer;
        std::optional<std::string> string;
    };
    std::map<std::string, std::unique_ptr<Option>, std::less<>> options;

    slang_driver_t() : sm(driver.sourceManager) { driver.addStandardArgs(); }

    ~slang_driver_t() {
        for (auto tree : trees)
            slang_syntax_tree_release(tree);
    }

    Option* find(std::string_view names) {
        auto it = options.find(names);
        return it == options.end() ? nullptr : it->second.get();
    }
};

slang_driver slang_driver_create(slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, { return new slang_driver_t(); });
    return nullptr;
}

void slang_driver_destroy(slang_driver driver) {
    delete driver;
}

void slang_driver_add_option(slang_driver driver, const char* names, size_t names_len,
                             slang_option_kind kind, const char* description,
                             size_t description_len, const char* value_name, size_t value_name_len,
                             slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!driver || !names) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, {
        std::string key(toView(names, names_len));
        if (driver->options.contains(key)) {
            setError(err, SLANG_ERR_INVALID_ARG, "option already registered");
            return;
        }
        auto opt = std::make_unique<slang_driver_t::Option>();
        opt->kind = kind;
        auto desc = toView(description, description_len);
        auto valueName = toView(value_name, value_name_len);
        switch (kind) {
            case SLANG_OPTION_FLAG:
                driver->driver.cmdLine.add(key, opt->flag, desc);
                break;
            case SLANG_OPTION_INT:
                driver->driver.cmdLine.add(key, opt->integer, desc, valueName);
                break;
            case SLANG_OPTION_STRING:
                driver->driver.cmdLine.add(key, opt->string, desc, valueName);
                break;
            default:
                setError(err, SLANG_ERR_INVALID_ARG, "invalid option kind");
                return;
        }
        driver->options.emplace(std::move(key), std::move(opt));
    });
}

bool slang_driver_parse_args(slang_driver driver, int argc, const char* const* argv,
                             slang_error* err) {
    if (!checkEntry(err))
        return false;
    if (!driver || (argc > 0 && !argv)) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return false;
    }
    SLANG_C_GUARD(err, { return driver->driver.parseCommandLine(argc, argv); });
    return false;
}

bool slang_driver_option_flag(slang_driver driver, const char* names, size_t names_len, bool* out) {
    SLANG_C_ACCESS(false, {
        auto opt = driver && names ? driver->find(toView(names, names_len)) : nullptr;
        if (!opt || !opt->flag)
            return false;
        if (out)
            *out = *opt->flag;
        return true;
    });
}

bool slang_driver_option_int(slang_driver driver, const char* names, size_t names_len,
                             int64_t* out) {
    SLANG_C_ACCESS(false, {
        auto opt = driver && names ? driver->find(toView(names, names_len)) : nullptr;
        if (!opt || !opt->integer)
            return false;
        if (out)
            *out = *opt->integer;
        return true;
    });
}

bool slang_driver_option_string(slang_driver driver, const char* names, size_t names_len,
                                slang_str* out) {
    SLANG_C_ACCESS(false, {
        auto opt = driver && names ? driver->find(toView(names, names_len)) : nullptr;
        if (!opt || !opt->string)
            return false;
        if (out)
            *out = borrowed(*opt->string);
        return true;
    });
}

slang_str slang_driver_help_text(slang_driver driver, const char* overview, size_t overview_len,
                                 slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return borrowed("");
    }
    SLANG_C_GUARD(err, {
        return owned(driver->driver.cmdLine.getHelpText(toView(overview, overview_len)));
    });
    return borrowed("");
}

bool slang_driver_process_options(slang_driver driver, slang_error* err) {
    if (!checkEntry(err))
        return false;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return false;
    }
    SLANG_C_GUARD(err, { return driver->driver.processOptions(); });
    return false;
}

bool slang_driver_parse_sources(slang_driver driver, slang_error* err) {
    if (!checkEntry(err))
        return false;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return false;
    }
    SLANG_C_GUARD(err, {
        bool ok = driver->driver.parseAllSources();
        for (auto& tree : driver->driver.syntaxTrees) {
            auto handle = new slang_syntax_tree_t();
            handle->tree = tree;
            handle->sm = &driver->sm;
            driver->trees.push_back(handle);
        }
        return ok;
    });
    return false;
}

slang_source_manager slang_driver_source_manager(slang_driver driver) {
    SLANG_C_ACCESS(nullptr, { return driver ? &driver->sm : nullptr; });
}

uint32_t slang_driver_tree_count(slang_driver driver) {
    SLANG_C_ACCESS(0, { return driver ? (uint32_t)driver->trees.size() : 0; });
}

slang_syntax_tree slang_driver_tree(slang_driver driver, uint32_t index) {
    SLANG_C_ACCESS(nullptr, {
        if (!driver || index >= driver->trees.size())
            return nullptr;
        return driver->trees[index];
    });
}

slang_compilation slang_driver_create_compilation(slang_driver driver, uint32_t extra_flags,
                                                  slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        // Build the compilation from the driver's parsed CLI options, ORing in
        // any extra compilation flags the caller requested (e.g. the safe
        // wrapper forces DisableInstanceCaching so the design can be totalized).
        Bag bag = driver->driver.createOptionBag();
        if (extra_flags) {
            auto opts = bag.getOrDefault<ast::CompilationOptions>();
            opts.flags |= bitmask<ast::CompilationFlags>(ast::CompilationFlags(extra_flags));
            bag.set(opts);
        }

        auto compilation = std::make_unique<ast::Compilation>(bag);
        for (auto& tree : driver->driver.syntaxTrees)
            compilation->addSyntaxTree(tree);

        auto comp = new slang_compilation_t(std::move(compilation));
        comp->sm = &driver->sm;
        for (auto tree : driver->trees)
            comp->trees.push_back(slang_syntax_tree_retain(tree));
        return comp;
    });
    return nullptr;
}

void slang_driver_report_compilation(slang_driver driver, slang_compilation comp, bool quiet,
                                     slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!driver || !comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, { driver->driver.reportCompilation(*comp->comp, quiet); });
}

bool slang_driver_report_diagnostics(slang_driver driver, bool quiet, slang_error* err) {
    if (!checkEntry(err))
        return false;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return false;
    }
    SLANG_C_GUARD(err, { return driver->driver.reportDiagnostics(quiet); });
    return false;
}
