# ~~~
# SPDX-FileCopyrightText: Michael Popoloski
# SPDX-License-Identifier: MIT
# ~~~

add_executable(capi_tests CApiTests.cpp CApiTilingTests.cpp)
target_link_libraries(capi_tests PRIVATE slang::slang_c slang::slang Catch2::Catch2WithMain)
target_compile_definitions(capi_tests PRIVATE SLANG_TEST_DIR="${PROJECT_SOURCE_DIR}/tests/")

add_test(NAME capi_tests COMMAND capi_tests)
