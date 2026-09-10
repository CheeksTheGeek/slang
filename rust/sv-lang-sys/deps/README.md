# Vendored header-only dependencies

These are the header-only libraries slang fetches with CMake `FetchContent`
(see `external/CMakeLists.txt`), copied here so that `sv-lang-sys` can build
slang with the `cc` crate alone. Keep the versions in lock-step with the tags
in `external/CMakeLists.txt`:

| Directory | Project | Version | License |
|---|---|---|---|
| `fmt/` | [fmtlib/fmt](https://github.com/fmtlib/fmt) | 12.2.0 | MIT (with exception) |
| `boost_regex/` | [MikePopoloski/regex](https://github.com/MikePopoloski/regex) (boost::regex, header-only fork) | boost-1.91.0 | BSL-1.0 |
| `tomlplusplus/` | [marzer/tomlplusplus](https://github.com/marzer/tomlplusplus) | v3.4.0 | MIT |

Only each project's `include/` tree and license are vendored.

To refresh after slang bumps a tag: configure slang once with CMake, then copy
`<build>/_deps/<name>-src/include` over the matching directory here and update
the table above.
