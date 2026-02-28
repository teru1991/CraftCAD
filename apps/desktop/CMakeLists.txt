cmake_minimum_required(VERSION 3.21)
project(CraftCADDesktop LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Widgets)

add_executable(craftcad_desktop
  src/main.cpp
)

target_include_directories(craftcad_desktop PRIVATE
  ${CMAKE_CURRENT_SOURCE_DIR}/../../core/ffi_desktop/include
)

if (WIN32)
  set(FFI_LIB_NAME craftcad_ffi_desktop.dll)
elseif (APPLE)
  set(FFI_LIB_NAME libcraftcad_ffi_desktop.dylib)
else()
  set(FFI_LIB_NAME libcraftcad_ffi_desktop.so)
endif()

if (NOT DEFINED FFI_LIB_DIR)
  set(FFI_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../core/target/release")
endif()

find_library(CRAFTCAD_DESKTOP_FFI_LIB
  NAMES craftcad_ffi_desktop
  PATHS ${FFI_LIB_DIR}
  NO_DEFAULT_PATH
)

if (NOT CRAFTCAD_DESKTOP_FFI_LIB)
  message(FATAL_ERROR "craftcad_ffi_desktop library not found. Build Rust FFI first and set -DFFI_LIB_DIR=<path>.")
endif()

target_link_libraries(craftcad_desktop PRIVATE
  Qt6::Widgets
  ${CRAFTCAD_DESKTOP_FFI_LIB}
)
