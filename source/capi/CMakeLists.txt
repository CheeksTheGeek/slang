# ~~~
# SPDX-FileCopyrightText: Michael Popoloski
# SPDX-License-Identifier: MIT
# ~~~

# -------- Generated reflection tables

add_custom_command(
  COMMAND ${Python_EXECUTABLE} ${SCRIPTS_DIR}/syntax_gen.py --dir
          ${CMAKE_CURRENT_BINARY_DIR} --syntax ${SCRIPTS_DIR}/syntax.txt --c-api
          ${CMAKE_CURRENT_BINARY_DIR}/SyntaxCApi.cpp
  OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/SyntaxCApi.cpp
  DEPENDS ${SCRIPTS_DIR}/syntax_gen.py ${SCRIPTS_DIR}/syntax.txt
          ${SCRIPTS_DIR}/triviakinds.txt ${SCRIPTS_DIR}/tokenkinds.txt
  COMMENT "Generating C API syntax tables")

add_custom_command(
  COMMAND ${Python_EXECUTABLE} ${SCRIPTS_DIR}/diagnostic_gen.py --outDir
          ${CMAKE_CURRENT_BINARY_DIR} --diagnostics ${SCRIPTS_DIR}/diagnostics.txt
          --c-api ${CMAKE_CURRENT_BINARY_DIR}/DiagCApi.cpp
  OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/DiagCApi.cpp
  DEPENDS ${SCRIPTS_DIR}/diagnostic_gen.py ${SCRIPTS_DIR}/diagnostics.txt
  COMMENT "Generating C API diagnostics tables")

# -------- Library target

add_library(
  slang_c
  CApi.cpp
  CApiSyntax.cpp
  CApiDiagnostics.cpp
  CApiAst.cpp
  CApiAnalysis.cpp
  CApiDataFlow.cpp
  CApiDriver.cpp
  ${CMAKE_CURRENT_BINARY_DIR}/SyntaxCApi.cpp
  ${CMAKE_CURRENT_BINARY_DIR}/DiagCApi.cpp)
add_library(slang::slang_c ALIAS slang_c)

target_link_libraries(slang_c PRIVATE slang::slang ${fmt_target})
target_include_directories(slang_c PUBLIC $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/include>
                                          $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>)
target_compile_definitions(slang_c PRIVATE SLANG_C_BUILDING)
if(NOT BUILD_SHARED_LIBS)
  target_compile_definitions(slang_c PUBLIC SLANG_C_STATIC)
endif()
set_target_properties(
  slang_c
  PROPERTIES CXX_VISIBILITY_PRESET hidden
             VISIBILITY_INLINES_HIDDEN YES
             VERSION ${PROJECT_VERSION}
             SOVERSION ${PROJECT_VERSION_MAJOR}
             OUTPUT_NAME slang-c)

if(SLANG_INCLUDE_INSTALL)
  install(
    TARGETS slang_c
    EXPORT slangTargets
    RUNTIME COMPONENT slang_Runtime
    LIBRARY COMPONENT slang_Runtime NAMELINK_COMPONENT slang_Development
    ARCHIVE COMPONENT slang_Development
    INCLUDES
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
endif()

if(SLANG_INCLUDE_TESTS)
  add_subdirectory(tests)
endif()
