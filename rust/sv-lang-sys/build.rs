//! Builds slang and its C API from source, or locates a prebuilt copy.
//!
//! Source layout is resolved in this order:
//!
//! 1. `DOCS_RS` is set: nothing is compiled.
//! 2. Feature `system`: link a prebuilt `slang-c` found through `pkg-config`
//!    or `SLANG_C_LIB_DIR` / `SLANG_C_INCLUDE_DIR`.
//! 3. `vendor/` exists next to this file (a published `.crate`): compile the
//!    vendored sources with their pre-generated files.
//! 4. Otherwise this is a checkout of the slang repository: compile the
//!    in-tree sources, running slang's Python generators into `OUT_DIR`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-env-changed=SLANG_C_LIB_DIR");
    println!("cargo:rerun-if-env-changed=SLANG_C_INCLUDE_DIR");
    println!("cargo:rerun-if-changed=build.rs");

    if env::var_os("DOCS_RS").is_some() {
        return;
    }

    if env::var_os("CARGO_FEATURE_SYSTEM").is_some() {
        link_system();
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let tree = SourceTree::locate(&manifest_dir);
    let generated = tree.generated_dir(&out_dir);
    let version_info = write_version_info(&tree, &out_dir);
    let export_header = write_export_header(&out_dir);
    compile(
        &tree,
        &generated,
        &version_info,
        &export_header,
        &manifest_dir,
    );
}

/// Where the C++ sources live and how their generated files are obtained.
enum SourceTree {
    /// The `vendor/` directory of a published crate: sources plus a
    /// `generated/` directory produced by `cargo xtask vendor`.
    Vendored(PathBuf),
    /// The root of a slang repository checkout.
    InTree(PathBuf),
}

impl SourceTree {
    fn locate(manifest_dir: &Path) -> Self {
        // In a checkout of the slang repository the in-tree source is the
        // source of truth; the vendored snapshot exists only for the published
        // crate, where the repository is not present. Preferring in-tree here
        // means a `cargo xtask vendor` snapshot can never go stale against the
        // sources under development.
        let repo = manifest_dir.join("../..");
        if repo.join("source/capi").is_dir() && repo.join("scripts/syntax_gen.py").is_file() {
            return SourceTree::InTree(repo.canonicalize().unwrap());
        }
        let vendored = manifest_dir.join("vendor");
        if vendored.join("source").is_dir() {
            return SourceTree::Vendored(vendored);
        }
        panic!(
            "sv-lang-sys: no slang sources found. Expected either {} (published crate) \
             or a slang repository checkout two directories up from {}.",
            vendored.display(),
            manifest_dir.display()
        );
    }

    fn root(&self) -> &Path {
        match self {
            SourceTree::Vendored(p) | SourceTree::InTree(p) => p,
        }
    }

    /// Returns the directory holding the generated sources and headers,
    /// generating them if this is an in-tree build.
    fn generated_dir(&self, out_dir: &Path) -> PathBuf {
        match self {
            SourceTree::Vendored(root) => root.join("generated"),
            SourceTree::InTree(root) => {
                let generated = out_dir.join("generated");
                generate(root, &generated);
                generated
            }
        }
    }

    fn version(&self) -> Version {
        match self {
            SourceTree::Vendored(root) => {
                let text = fs::read_to_string(root.join("VERSION")).expect("vendor/VERSION");
                Version::parse(text.trim())
            }
            SourceTree::InTree(root) => Version::from_repo(root),
        }
    }
}

struct Version {
    major: String,
    minor: String,
    patch: String,
    prerelease: String,
    hash: String,
}

impl Version {
    /// Parses "MAJOR.MINOR.PATCH[-PRERELEASE][+HASH]".
    fn parse(s: &str) -> Version {
        let (core, hash) = s.split_once('+').unwrap_or((s, ""));
        let (core, prerelease) = core.split_once('-').unwrap_or((core, ""));
        let mut parts = core.split('.');
        Version {
            major: parts.next().unwrap_or("0").to_string(),
            minor: parts.next().unwrap_or("0").to_string(),
            patch: parts.next().unwrap_or("0").to_string(),
            prerelease: prerelease.to_string(),
            hash: hash.to_string(),
        }
    }

    /// Reads MAJOR/MINOR from the repository's CMakeLists.txt and the hash
    /// from git, mirroring cmake/gitversion.cmake.
    fn from_repo(root: &Path) -> Version {
        let cmake = fs::read_to_string(root.join("CMakeLists.txt")).expect("CMakeLists.txt");
        let get = |name: &str| {
            cmake
                .lines()
                .find_map(|l| {
                    let l = l.trim();
                    l.strip_prefix(&format!("set({name} "))
                        .map(|rest| rest.trim_end_matches(')').trim().to_string())
                })
                .unwrap_or_else(|| panic!("{name} not found in CMakeLists.txt"))
        };
        let git = |args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        };
        Version {
            major: get("SLANG_VERSION_MAJOR"),
            minor: get("SLANG_VERSION_MINOR"),
            patch: "0".to_string(),
            prerelease: String::new(),
            hash: git(&["rev-parse", "--short", "HEAD"]),
        }
    }
}

/// Runs slang's code generators into `generated`, exactly as the CMake build
/// does (see source/CMakeLists.txt and source/capi/CMakeLists.txt).
fn generate(root: &Path, generated: &Path) {
    let scripts = root.join("scripts");
    for f in [
        "syntax_gen.py",
        "diagnostic_gen.py",
        "syntax.txt",
        "tokenkinds.txt",
        "triviakinds.txt",
        "systemnames.txt",
        "diagnostics.txt",
    ] {
        println!("cargo:rerun-if-changed={}", scripts.join(f).display());
    }

    fs::create_dir_all(generated).unwrap();
    let python = env::var("PYTHON").unwrap_or_else(|_| "python3".to_string());
    let run = |args: &[&str]| {
        let status = Command::new(&python)
            .args(args)
            .current_dir(root)
            .status()
            .unwrap_or_else(|e| {
                panic!(
                    "sv-lang-sys: cannot run {python} ({e}); building from a \
                                        slang checkout needs Python 3, or set PYTHON"
                )
            });
        assert!(status.success(), "sv-lang-sys: {python} {args:?} failed");
    };
    let gen_dir = generated.to_str().unwrap();
    let syntax = scripts.join("syntax.txt");
    let diagnostics = scripts.join("diagnostics.txt");
    run(&[
        "scripts/syntax_gen.py",
        "--dir",
        gen_dir,
        "--syntax",
        syntax.to_str().unwrap(),
    ]);
    run(&[
        "scripts/syntax_gen.py",
        "--dir",
        gen_dir,
        "--syntax",
        syntax.to_str().unwrap(),
        "--c-api",
        generated.join("SyntaxCApi.cpp").to_str().unwrap(),
    ]);
    run(&[
        "scripts/diagnostic_gen.py",
        "--outDir",
        gen_dir,
        "--srcDir",
        "source",
        "--incDir",
        "include/slang",
        "--diagnostics",
        diagnostics.to_str().unwrap(),
    ]);
    run(&[
        "scripts/diagnostic_gen.py",
        "--outDir",
        gen_dir,
        "--diagnostics",
        diagnostics.to_str().unwrap(),
        "--c-api",
        generated.join("DiagCApi.cpp").to_str().unwrap(),
    ]);
}

/// Instantiates source/util/VersionInfo.cpp.in into `out_dir`, returning the
/// path of the resulting source file.
fn write_version_info(tree: &SourceTree, out_dir: &Path) -> PathBuf {
    let template = tree.root().join("source/util/VersionInfo.cpp.in");
    println!("cargo:rerun-if-changed={}", template.display());
    let v = tree.version();
    let text = fs::read_to_string(&template)
        .expect("VersionInfo.cpp.in")
        .replace("@SLANG_VERSION_MAJOR@", &v.major)
        .replace("@SLANG_VERSION_MINOR@", &v.minor)
        .replace("@SLANG_VERSION_PATCH@", &v.patch)
        .replace("@SLANG_VERSION_PRERELEASE@", &v.prerelease)
        .replace("@SLANG_VERSION_HASH@", &v.hash);
    let dir = out_dir.join("version");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("VersionInfo.cpp");
    write_if_changed(&path, &text);
    path
}

/// Writes the header that CMake's generate_export_header would produce, for a
/// static build where every export macro is empty.
fn write_export_header(out_dir: &Path) -> PathBuf {
    let dir = out_dir.join("export/slang");
    fs::create_dir_all(&dir).unwrap();
    write_if_changed(
        &dir.join("slang_export.h"),
        "// Generated by sv-lang-sys/build.rs for a static build.\n\
         #ifndef SLANG_EXPORT_H\n\
         #define SLANG_EXPORT_H\n\
         #define SLANG_EXPORT\n\
         #define SLANG_NO_EXPORT\n\
         #define SLANG_DEPRECATED __attribute__((__deprecated__))\n\
         #define SLANG_DEPRECATED_EXPORT SLANG_EXPORT SLANG_DEPRECATED\n\
         #define SLANG_DEPRECATED_NO_EXPORT SLANG_NO_EXPORT SLANG_DEPRECATED\n\
         #endif\n",
    );
    out_dir.join("export")
}

fn write_if_changed(path: &Path, text: &str) {
    if fs::read_to_string(path)
        .map(|old| old == text)
        .unwrap_or(false)
    {
        return;
    }
    fs::write(path, text).unwrap_or_else(|e| panic!("writing {}: {e}", path.display()));
}

fn compile(
    tree: &SourceTree,
    generated: &Path,
    version_info: &Path,
    export_include: &Path,
    manifest_dir: &Path,
) {
    let root = tree.root();
    let source = root.join("source");
    let include = root.join("include");
    let external = root.join("external");
    let deps = manifest_dir.join("deps");
    for dir in [&source, &include, &external, &deps] {
        println!("cargo:rerun-if-changed={}", dir.display());
    }

    let mut files = Vec::new();
    collect_cpp(&source, &mut files);
    files.retain(|f| !f.starts_with(source.join("capi/tests")));
    for f in [
        "AllSyntax.cpp",
        "SyntaxClone.cpp",
        "DiagCode.cpp",
        "TokenKind.cpp",
        "KnownSystemName.cpp",
        "SyntaxCApi.cpp",
        "DiagCApi.cpp",
    ] {
        files.push(generated.join(f));
    }
    files.push(version_info.to_path_buf());
    files.sort();

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++20")
        .warnings(false)
        // The vendored C++ objects are large; skip debug info (it is not
        // useful for a binary dependency and bloats the static archive).
        .debug(false)
        .files(&files)
        .include(&include)
        .include(generated)
        .include(export_include)
        .include(&external)
        .include(deps.join("fmt/include"))
        .include(deps.join("boost_regex/include"))
        .include(deps.join("tomlplusplus/include"))
        // The safe layer's thread-safety contract relies on slang's internal
        // assertions (e.g. BumpAllocator::isFrozen) being live in every profile.
        .define("SLANG_ASSERT_ENABLED", None)
        .define("SLANG_STATIC_DEFINE", None)
        .define("SLANG_C_STATIC", None)
        .define("SLANG_C_BUILDING", None)
        .define("SLANG_USE_THREADS", "1");

    let target = env::var("TARGET").unwrap_or_default();
    if build.get_compiler().is_like_msvc() {
        build
            .flag("/EHsc")
            .flag("/Zc:__cplusplus")
            .flag("/bigobj")
            .flag("/utf-8")
            .flag("/permissive-");
    } else {
        build.flag("-fexceptions");
        if target.contains("wasi") {
            panic!(
                "sv-lang-sys: WASI targets are provided by the `backend-wasm` feature of sv-lang, not by compiling natively"
            );
        }
        // Instrument the vendored C++ for ThreadSanitizer when SV_LANG_TSAN is
        // set, so the TSan negative gate (`sv-lang/tests/tsan_race.rs`) can see
        // races inside slang itself, not just in the Rust wrappers.
        println!("cargo:rerun-if-env-changed=SV_LANG_TSAN");
        if env::var_os("SV_LANG_TSAN").is_some() {
            build.flag("-fsanitize=thread").flag("-g");
        }
    }

    build.compile("slang-c");

    if target.contains("linux") || target.contains("freebsd") {
        println!("cargo:rustc-link-lib=pthread");
    }

    // For downstream -sys consumers and for the safe crate's own build script.
    println!("cargo:include={}", include.display());
    println!("cargo:generated={}", generated.display());
}

fn collect_cpp(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_cpp(&path, out);
        } else if path.extension().is_some_and(|e| e == "cpp") {
            out.push(path);
        }
    }
}

fn link_system() {
    let include = env::var("SLANG_C_INCLUDE_DIR").ok();
    if let Ok(lib_dir) = env::var("SLANG_C_LIB_DIR") {
        println!("cargo:rustc-link-search=native={lib_dir}");
        println!("cargo:rustc-link-lib=slang-c");
        println!("cargo:rustc-link-lib=svlang");
        if let Some(inc) = include {
            println!("cargo:include={inc}");
        }
        return;
    }
    match pkg_config::Config::new().probe("slang-c") {
        Ok(lib) => {
            for p in lib.include_paths {
                println!("cargo:include={}", p.display());
            }
        }
        Err(e) => panic!(
            "sv-lang-sys: feature `system` is enabled but slang-c was not found via pkg-config \
             ({e}); set SLANG_C_LIB_DIR (and SLANG_C_INCLUDE_DIR) or disable the feature"
        ),
    }
}
