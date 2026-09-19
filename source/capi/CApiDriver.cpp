//------------------------------------------------------------------------------
// CApiDriver.cpp
// C API: the command-line driver
//
// SPDX-FileCopyrightText: Chaitanya Sharma
// SPDX-License-Identifier: MIT
//------------------------------------------------------------------------------
#include "CApiInternal.h"
#include <filesystem>
#include <map>
#include <optional>

#include "slang/analysis/AnalysisOptions.h"
#include "slang/diagnostics/TextDiagnosticClient.h"
#include "slang/driver/Driver.h"

using namespace slang;
using namespace slang::capi;

// The handle behind slang_driver_diag_engine: a stable-address wrapper around
// the driver's own DiagnosticEngine member (never reallocated for the life of
// the owning slang_driver_t).
struct slang_diag_engine_t {
    DiagnosticEngine& engine;
};

// The handle behind slang_command_file_metadata_at. Owned by the driver's
// `commandFiles` snapshot (see slang_driver_t::refreshCommandFileMetadata) --
// a copy is required because Driver::commandFileMetadata is a
// boost::unordered_flat_map, whose element addresses are not stable across
// insertion/rehash, so we cannot hand out raw pointers into it directly.
struct slang_command_file_metadata_t {
    std::string path;
    std::vector<std::string> defines;
};

struct slang_driver_t {
    driver::Driver driver;
    slang_source_manager_t sm;
    std::vector<slang_syntax_tree> trees;
    slang_diag_engine_t diagEngineHandle;
    // Stable-address wrappers around the driver's own textDiagClient and
    // sourceLoader members (never reallocated for the life of this handle),
    // mirroring diagEngineHandle above — see slang_driver_text_diag_client
    // and slang_driver_source_loader.
    slang_text_diag_client_t textDiagClientHandle;
    slang_source_loader_t sourceLoaderHandle;
    bool stdArgsAdded = false;
    std::vector<std::unique_ptr<slang_command_file_metadata_t>> commandFiles;

    // Registered custom options. CommandLine stores references to the value
    // slots, so they live behind stable heap addresses.
    struct Option {
        slang_option_kind kind;
        std::optional<bool> flag;
        std::optional<int64_t> integer;
        std::optional<std::string> string;
    };
    std::map<std::string, std::unique_ptr<Option>, std::less<>> options;

    explicit slang_driver_t(bool addStandardArgs) :
        sm(driver.sourceManager), diagEngineHandle{driver.diagEngine},
        textDiagClientHandle{driver.textDiagClient}, sourceLoaderHandle{driver.sourceLoader, &sm} {
        if (addStandardArgs) {
            driver.addStandardArgs();
            stdArgsAdded = true;
        }
    }

    ~slang_driver_t() {
        for (auto tree : trees)
            slang_syntax_tree_release(tree);
    }

    Option* find(std::string_view names) {
        auto it = options.find(names);
        return it == options.end() ? nullptr : it->second.get();
    }

    // Rebuilds the stable snapshot exposed by slang_driver_command_file_metadata_*
    // from the driver's live (unstable-address) map. Called after every
    // processCommandFiles call, since that is the only thing that can grow it.
    void refreshCommandFileMetadata() {
        commandFiles.clear();
        commandFiles.reserve(driver.getCommandFileMetadata().size());
        for (auto& [path, meta] : driver.getCommandFileMetadata()) {
            auto handle = std::make_unique<slang_command_file_metadata_t>();
            handle->path = path.string();
            handle->defines = meta.defines;
            commandFiles.push_back(std::move(handle));
        }
    }
};

slang_driver slang_driver_create(slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, { return new slang_driver_t(/* addStandardArgs */ true); });
    return nullptr;
}

slang_driver slang_driver_create_bare(slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, { return new slang_driver_t(/* addStandardArgs */ false); });
    return nullptr;
}

void slang_driver_destroy(slang_driver driver) {
    delete driver;
}

void slang_driver_add_standard_args(slang_driver driver, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return;
    }
    if (driver->stdArgsAdded)
        return;
    SLANG_C_GUARD(err, {
        driver->driver.addStandardArgs();
        driver->stdArgsAdded = true;
    });
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

// An owned CommandLine::ParseOptions set (see slang_parse_options_create).
struct slang_parse_options_t {
    CommandLine::ParseOptions opts;
};

slang_parse_options slang_parse_options_create(slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, { return new slang_parse_options_t(); });
    return nullptr;
}

void slang_parse_options_destroy(slang_parse_options options) {
    delete options;
}

void slang_parse_options_set_support_comments(slang_parse_options options, bool value) {
    if (options)
        options->opts.supportComments = value;
}

bool slang_parse_options_support_comments(slang_parse_options options) {
    return options && options->opts.supportComments;
}

void slang_parse_options_set_ignore_program_name(slang_parse_options options, bool value) {
    if (options)
        options->opts.ignoreProgramName = value;
}

bool slang_parse_options_ignore_program_name(slang_parse_options options) {
    return options && options->opts.ignoreProgramName;
}

void slang_parse_options_set_expand_env_vars(slang_parse_options options, bool value) {
    if (options)
        options->opts.expandEnvVars = value;
}

bool slang_parse_options_expand_env_vars(slang_parse_options options) {
    return options && options->opts.expandEnvVars;
}

void slang_parse_options_set_ignore_duplicates(slang_parse_options options, bool value) {
    if (options)
        options->opts.ignoreDuplicates = value;
}

bool slang_parse_options_ignore_duplicates(slang_parse_options options) {
    return options && options->opts.ignoreDuplicates;
}

bool slang_driver_parse_args_with_options(slang_driver driver, int argc, const char* const* argv,
                                          slang_parse_options options, slang_error* err) {
    if (!checkEntry(err))
        return false;
    if (!driver || (argc > 0 && !argv)) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return false;
    }
    SLANG_C_GUARD(err, {
        CommandLine::ParseOptions empty;
        return driver->driver.parseCommandLine(argc, argv, options ? options->opts : empty);
    });
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
        // Build the compilation via the driver's real createCompilation() (which
        // internally calls createOptionBag() and wires up the default source
        // library, library maps, and user-defined subroutines), ORing in any
        // extra compilation flags the caller requested (e.g. the safe wrapper
        // forces DisableInstanceCaching so the design can be totalized).
        auto compilation = driver->driver.createCompilation(
            bitmask<ast::CompilationFlags>(ast::CompilationFlags(extra_flags)));

        auto comp = new slang_compilation_t(std::move(compilation));
        comp->sm = &driver->sm;
        for (auto tree : driver->trees)
            comp->trees.push_back(slang_syntax_tree_retain(tree));
        return comp;
    });
    return nullptr;
}

slang_bag slang_driver_create_option_bag(slang_driver driver, slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return nullptr;
    }
    SLANG_C_GUARD(err, { return new slang_bag_t{driver->driver.createOptionBag()}; });
    return nullptr;
}

void slang_bag_destroy(slang_bag bag) {
    delete bag;
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

uint32_t slang_driver_language_version(slang_driver driver) {
    SLANG_C_ACCESS(0, { return driver ? (uint32_t)driver->driver.languageVersion : 0; });
}

bool slang_driver_process_command_files(slang_driver driver, const char* pattern,
                                        size_t pattern_len, bool make_relative, bool separate_unit,
                                        slang_error* err) {
    if (!checkEntry(err))
        return false;
    if (!driver || !pattern) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return false;
    }
    SLANG_C_GUARD(err, {
        bool ok = driver->driver.processCommandFiles(toView(pattern, pattern_len), make_relative,
                                                      separate_unit);
        // Refresh regardless of `ok`: a file that partially failed may still
        // have contributed metadata (setCurrentCommandFile registers the entry
        // before the file's own options are parsed).
        driver->refreshCommandFileMetadata();
        return ok;
    });
    return false;
}

uint32_t slang_driver_command_file_metadata_count(slang_driver driver) {
    SLANG_C_ACCESS(0, { return driver ? (uint32_t)driver->commandFiles.size() : 0; });
}

slang_command_file_metadata slang_driver_command_file_metadata_at(slang_driver driver,
                                                                   uint32_t index) {
    SLANG_C_ACCESS(nullptr, {
        if (!driver || index >= driver->commandFiles.size())
            return nullptr;
        return driver->commandFiles[index].get();
    });
}

slang_str slang_command_file_metadata_path(slang_command_file_metadata meta) {
    SLANG_C_ACCESS(borrowed(""), { return meta ? borrowed(meta->path) : borrowed(""); });
}

uint32_t slang_command_file_metadata_define_count(slang_command_file_metadata meta) {
    SLANG_C_ACCESS(0, { return meta ? (uint32_t)meta->defines.size() : 0; });
}

slang_str slang_command_file_metadata_define_at(slang_command_file_metadata meta, uint32_t index) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!meta || index >= meta->defines.size())
            return borrowed("");
        return borrowed(meta->defines[index]);
    });
}

void slang_driver_optionally_write_dep_files(slang_driver driver, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return;
    }
    SLANG_C_GUARD(err, { driver->driver.optionallyWriteDepFiles(); });
}

slang_diag_engine slang_driver_diag_engine(slang_driver driver) {
    SLANG_C_ACCESS(nullptr, { return driver ? &driver->diagEngineHandle : nullptr; });
}

int32_t slang_diag_engine_num_errors(slang_diag_engine engine) {
    SLANG_C_ACCESS(0, { return engine ? (int32_t)engine->engine.getNumErrors() : 0; });
}

int32_t slang_diag_engine_num_warnings(slang_diag_engine engine) {
    SLANG_C_ACCESS(0, { return engine ? (int32_t)engine->engine.getNumWarnings() : 0; });
}

slang_analysis_options slang_driver_get_analysis_options(slang_driver driver) {
    slang_analysis_options none{};
    if (!driver)
        return none;
    SLANG_C_ACCESS(none, {
        auto ao = driver->driver.getAnalysisOptions();
        return slang_analysis_options{
            (uint32_t)ao.flags.bits(),
            ao.maxCaseAnalysisSteps,
            ao.maxLoopAnalysisSteps,
        };
    });
}

bool slang_driver_run_preprocessor(slang_driver driver, uint32_t flags, slang_error* err) {
    if (!checkEntry(err))
        return false;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return false;
    }
    SLANG_C_GUARD(err, {
        return driver->driver.runPreprocessor(
            bitmask<driver::PreprocessOutputFlags>(driver::PreprocessOutputFlags(flags)));
    });
    return false;
}

void slang_driver_report_macros(slang_driver driver, bool group_by_file, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return;
    }
    SLANG_C_GUARD(err, { driver->driver.reportMacros(group_by_file); });
}

bool slang_driver_report_parse_diags(slang_driver driver, slang_error* err) {
    if (!checkEntry(err))
        return false;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return false;
    }
    SLANG_C_GUARD(err, { return driver->driver.reportParseDiags(); });
    return false;
}

slang_analysis slang_driver_run_analysis(slang_driver driver, slang_compilation comp,
                                         slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    if (!driver || !comp) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return nullptr;
    }
    SLANG_C_GUARD(err, {
        auto manager = driver->driver.runAnalysis(*comp->comp);
        // Driver::runAnalysis freezes then unfreezes the real BumpAllocator
        // itself around the analysis pass; resync our own `sealed` bookkeeping
        // to its actual post-call state rather than assume it is unchanged
        // (see the SealLift / slang_compilation_freeze machinery this mirrors).
        comp->sealed = comp->comp->isFrozen();
        auto result = new slang_analysis_t();
        result->manager = std::move(manager);
        result->comp = comp;
        return result;
    });
    return nullptr;
}

bool slang_driver_run_full_compilation(slang_driver driver, bool quiet, slang_error* err) {
    if (!checkEntry(err))
        return false;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return false;
    }
    SLANG_C_GUARD(err, { return driver->driver.runFullCompilation(quiet); });
    return false;
}

void slang_driver_set_terminal_colors_enabled(slang_driver driver, bool enable, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!driver) {
        setError(err, SLANG_ERR_INVALID_ARG, "null driver");
        return;
    }
    SLANG_C_GUARD(err, { driver->driver.setTerminalColorsEnabled(enable); });
}

slang_source_loader slang_driver_source_loader(slang_driver driver) {
    SLANG_C_ACCESS(nullptr, { return driver ? &driver->sourceLoaderHandle : nullptr; });
}

void slang_source_loader_add_files(slang_source_loader loader, const char* pattern,
                                   size_t pattern_len, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!loader || !pattern) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, { loader->loader.addFiles(toView(pattern, pattern_len)); });
}

void slang_source_loader_add_library_files(slang_source_loader loader, const char* library_name,
                                           size_t library_name_len, const char* pattern,
                                           size_t pattern_len, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!loader || !pattern) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, {
        loader->loader.addLibraryFiles(toView(library_name, library_name_len),
                                       toView(pattern, pattern_len));
    });
}

void slang_source_loader_add_library_maps(slang_source_loader loader, const char* pattern,
                                          size_t pattern_len, const char* base_path,
                                          size_t base_path_len, slang_options options,
                                          slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!loader || !pattern) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, {
        std::filesystem::path base(toView(base_path, base_path_len));
        loader->loader.addLibraryMaps(toView(pattern, pattern_len), base,
                                      options ? options->toBag() : Bag());
    });
}

void slang_source_loader_add_search_directories(slang_source_loader loader, const char* pattern,
                                                 size_t pattern_len, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!loader || !pattern) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, { loader->loader.addSearchDirectories(toView(pattern, pattern_len)); });
}

void slang_source_loader_add_search_extension(slang_source_loader loader, const char* extension,
                                              size_t extension_len, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!loader || !extension) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, { loader->loader.addSearchExtension(toView(extension, extension_len)); });
}

namespace {

// Builds an owned vector<string> from a (count, const char* const*) array,
// tolerating a null array when count is 0 (see the string-array arguments of
// slang_source_loader_add_separate_unit).
std::vector<std::string> toStringVector(const char* const* items, size_t count) {
    std::vector<std::string> result;
    if (!items)
        return result;
    result.reserve(count);
    for (size_t i = 0; i < count; i++)
        result.emplace_back(items[i] ? items[i] : "");
    return result;
}

} // namespace

void slang_source_loader_add_separate_unit(slang_source_loader loader,
                                           const char* const* file_patterns,
                                           size_t file_patterns_count,
                                           const char* const* include_paths,
                                           size_t include_paths_count, const char* const* defines,
                                           size_t defines_count, const char* library_name,
                                           size_t library_name_len,
                                           const char* const* warning_options,
                                           size_t warning_options_count, slang_error* err) {
    if (!checkEntry(err))
        return;
    if (!loader || (file_patterns_count && !file_patterns)) {
        setError(err, SLANG_ERR_INVALID_ARG, "null argument");
        return;
    }
    SLANG_C_GUARD(err, {
        auto patterns = toStringVector(file_patterns, file_patterns_count);
        auto includePaths = toStringVector(include_paths, include_paths_count);
        auto defs = toStringVector(defines, defines_count);
        auto warnings = toStringVector(warning_options, warning_options_count);
        std::string library(toView(library_name, library_name_len));
        loader->loader.addSeparateUnit(patterns, includePaths, std::move(defs), library,
                                       std::move(warnings));
    });
}

bool slang_source_loader_has_files(slang_source_loader loader) {
    SLANG_C_ACCESS(false, { return loader && loader->loader.hasFiles(); });
}

uint32_t slang_source_loader_error_count(slang_source_loader loader) {
    SLANG_C_ACCESS(0u, {
        return loader ? (uint32_t)loader->loader.getErrors().size() : 0u;
    });
}

slang_str slang_source_loader_error_at(slang_source_loader loader, uint32_t index) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!loader)
            return borrowed("");
        auto errors = loader->loader.getErrors();
        if (index >= errors.size())
            return borrowed("");
        return borrowed(errors[index]);
    });
}

namespace {

// Grows `loader->libraryMapTrees` to match `loader->loader.getLibraryMaps()`,
// wrapping any newly-appeared trees. The real list only ever grows (new
// entries are appended by addLibraryMaps), so already-wrapped entries are
// left untouched.
void syncLibraryMapTrees(slang_source_loader loader) {
    auto& maps = loader->loader.getLibraryMaps();
    for (size_t i = loader->libraryMapTrees.size(); i < maps.size(); i++) {
        auto handle = new slang_syntax_tree_t();
        handle->tree = maps[i];
        handle->sm = loader->sm;
        loader->libraryMapTrees.push_back(handle);
    }
}

} // namespace

uint32_t slang_source_loader_library_map_count(slang_source_loader loader) {
    SLANG_C_ACCESS(0u, {
        if (!loader)
            return 0u;
        syncLibraryMapTrees(loader);
        return (uint32_t)loader->libraryMapTrees.size();
    });
}

slang_syntax_tree slang_source_loader_library_map_at(slang_source_loader loader, uint32_t index) {
    SLANG_C_ACCESS(nullptr, {
        if (!loader)
            return nullptr;
        syncLibraryMapTrees(loader);
        if (index >= loader->libraryMapTrees.size())
            return nullptr;
        return loader->libraryMapTrees[index];
    });
}

uint32_t slang_source_loader_load_sources(slang_source_loader loader, slang_error* err) {
    if (!checkEntry(err))
        return 0u;
    if (!loader) {
        setError(err, SLANG_ERR_INVALID_ARG, "null loader");
        return 0u;
    }
    SLANG_C_GUARD(err, {
        loader->loadedBuffers = loader->loader.loadSources();
        return (uint32_t)loader->loadedBuffers.size();
    });
    return 0u;
}

slang_buffer_id slang_source_loader_loaded_buffer_id(slang_source_loader loader, uint32_t index) {
    SLANG_C_ACCESS(0u, {
        if (!loader || index >= loader->loadedBuffers.size())
            return 0u;
        return loader->loadedBuffers[index].id.getId();
    });
}

slang_str slang_source_loader_loaded_buffer_text(slang_source_loader loader, uint32_t index) {
    SLANG_C_ACCESS(borrowed(""), {
        if (!loader || index >= loader->loadedBuffers.size())
            return borrowed("");
        // Every loaded buffer carries a trailing '\0' sentinel (see
        // SourceManager::assignText / the comment on getSourceLine); drop it
        // so callers never see it as part of the text.
        auto text = loader->loadedBuffers[index].data;
        if (!text.empty() && text.back() == '\0')
            text.remove_suffix(1);
        return borrowed(text);
    });
}

// An owned slang::driver::SourceOptions value (see slang_source_options_create).
struct slang_source_options_t {
    driver::SourceOptions opts{};
};

slang_source_options slang_source_options_create(slang_error* err) {
    if (!checkEntry(err))
        return nullptr;
    SLANG_C_GUARD(err, { return new slang_source_options_t(); });
    return nullptr;
}

void slang_source_options_destroy(slang_source_options options) {
    delete options;
}

void slang_source_options_set_num_threads(slang_source_options options, bool has_value,
                                          uint32_t value) {
    if (!options)
        return;
    if (has_value)
        options->opts.numThreads = value;
    else
        options->opts.numThreads.reset();
}

bool slang_source_options_num_threads(slang_source_options options, uint32_t* out) {
    SLANG_C_ACCESS(false, {
        if (!options || !options->opts.numThreads)
            return false;
        if (out)
            *out = *options->opts.numThreads;
        return true;
    });
}

void slang_source_options_set_single_unit(slang_source_options options, bool value) {
    if (options)
        options->opts.singleUnit = value;
}

bool slang_source_options_single_unit(slang_source_options options) {
    return options && options->opts.singleUnit;
}

void slang_source_options_set_only_lint(slang_source_options options, bool value) {
    if (options)
        options->opts.onlyLint = value;
}

bool slang_source_options_only_lint(slang_source_options options) {
    return options && options->opts.onlyLint;
}

void slang_source_options_set_libraries_inherit_macros(slang_source_options options, bool value) {
    if (options)
        options->opts.librariesInheritMacros = value;
}

bool slang_source_options_libraries_inherit_macros(slang_source_options options) {
    return options && options->opts.librariesInheritMacros;
}

slang_text_diag_client slang_driver_text_diag_client(slang_driver driver) {
    SLANG_C_ACCESS(nullptr, { return driver ? &driver->textDiagClientHandle : nullptr; });
}

slang_str slang_text_diag_client_get_string(slang_text_diag_client client, slang_error* err) {
    if (!checkEntry(err))
        return borrowed("");
    if (!client) {
        setError(err, SLANG_ERR_INVALID_ARG, "null client");
        return borrowed("");
    }
    SLANG_C_GUARD(err, { return owned(client->client->getString()); });
    return borrowed("");
}

bool slang_text_diag_client_empty(slang_text_diag_client client) {
    SLANG_C_ACCESS(true, { return !client || client->client->empty(); });
}
