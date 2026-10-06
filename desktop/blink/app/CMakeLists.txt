# Xenon Blink host build, staged into the CEF distribution as
# tests/xenon/ by desktop/tools/stage-blink.py. Mirrors the cefsimple
# CMakeLists from the same distribution so every CEF-provided macro and
# default (including USE_SANDBOX=On on Windows) applies to us.

#
# Source files.
#

set(XENON_SRCS
  main.cc
  ui_scheme_handler.cc
  ui_scheme_handler.h
  xenon_app.cc
  xenon_app.h
  xenon_handler.cc
  xenon_handler.h
  )
source_group(xenon FILES ${XENON_SRCS})


#
# Shared configuration.
#

# Target executable name.
set(CEF_TARGET "xenon-blink")
# Logical target used to link the libcef library.
ADD_LOGICAL_TARGET("libcef_lib" "${CEF_LIB_DEBUG}" "${CEF_LIB_RELEASE}")

# Determine the target output directory.
SET_CEF_TARGET_OUT_DIR()


#
# Linux configuration.
#

if(OS_LINUX)
  add_executable(${CEF_TARGET} ${XENON_SRCS})
  SET_EXECUTABLE_TARGET_PROPERTIES(${CEF_TARGET})
  add_dependencies(${CEF_TARGET} libcef_dll_wrapper)
  target_link_libraries(${CEF_TARGET} libcef_lib libcef_dll_wrapper ${CEF_STANDARD_LIBS})

  # Place libraries next to the executable.
  set_target_properties(${CEF_TARGET} PROPERTIES INSTALL_RPATH "$ORIGIN")
  set_target_properties(${CEF_TARGET} PROPERTIES BUILD_WITH_INSTALL_RPATH TRUE)
  set_target_properties(${CEF_TARGET} PROPERTIES RUNTIME_OUTPUT_DIRECTORY ${CEF_TARGET_OUT_DIR})

  COPY_FILES("${CEF_TARGET}" "${CEF_BINARY_FILES}" "${CEF_BINARY_DIR}" "${CEF_TARGET_OUT_DIR}")
  COPY_FILES("${CEF_TARGET}" "${CEF_RESOURCE_FILES}" "${CEF_RESOURCE_DIR}" "${CEF_TARGET_OUT_DIR}")
  if (EXISTS "${CEF_BINARY_DIR}/libminigbm.so")
    COPY_FILES("${CEF_TARGET}" "libminigbm.so" "${CEF_BINARY_DIR}" "${CEF_TARGET_OUT_DIR}")
  endif()

  # SUID helper for the sandbox (SEC-01: the sandbox stays on).
  SET_LINUX_SUID_PERMISSIONS("${CEF_TARGET}" "${CEF_TARGET_OUT_DIR}/chrome-sandbox")
endif()


#
# Windows configuration.
#

if(OS_WINDOWS)
  if(USE_SANDBOX)
    # Shared library (DLL) target loaded by bootstrap.exe.
    add_library(${CEF_TARGET} SHARED ${XENON_SRCS})
    SET_LIBRARY_TARGET_PROPERTIES(${CEF_TARGET})

    # Copy and rename the bootstrap binary.
    COPY_SINGLE_FILE(${CEF_TARGET} ${CEF_BINARY_DIR}/bootstrap.exe ${CEF_TARGET_OUT_DIR}/${CEF_TARGET}.exe)
  else()
    # Executable target.
    add_executable(${CEF_TARGET} WIN32 ${XENON_SRCS})
    SET_EXECUTABLE_TARGET_PROPERTIES(${CEF_TARGET})
  endif()

  add_dependencies(${CEF_TARGET} libcef_dll_wrapper)
  target_link_libraries(${CEF_TARGET} libcef_lib libcef_dll_wrapper ${CEF_STANDARD_LIBS})

  COPY_FILES("${CEF_TARGET}" "${CEF_BINARY_FILES}" "${CEF_BINARY_DIR}" "${CEF_TARGET_OUT_DIR}")
  COPY_FILES("${CEF_TARGET}" "${CEF_RESOURCE_FILES}" "${CEF_RESOURCE_DIR}" "${CEF_TARGET_OUT_DIR}")

  if(USE_SANDBOX)
    # LPAC ACLs required for Windows sandbox support.
    SET_LPAC_ACLS("${CEF_TARGET}")
  endif()
endif()
