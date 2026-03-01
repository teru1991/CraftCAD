cmake_minimum_required(VERSION 3.21)
project(CraftCADDesktop LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Widgets)

add_executable(craftcad_desktop
  src/main.cpp
  src/doc_store.cpp
  src/face_part_panel.cpp
  src/canvas_widget.cpp
  src/camera.cpp
  src/selection_state.cpp
  src/hittest.cpp
  src/snap_engine.cpp
  src/overlay_renderer.cpp
  src/input/numeric_input.cpp
  src/input/shortcut_map.cpp
  src/tools/tool_base.cpp
  src/tools/line_tool.cpp
  src/tools/scale_tool.cpp
  src/tools/rotate_tool.cpp
  src/tools/move_tool.cpp
  src/tools/trim_tool.cpp
  src/tools/offset_tool.cpp
  src/ffi/craftcad_ffi.cpp
)

target_include_directories(craftcad_desktop PRIVATE
  ${CMAKE_CURRENT_SOURCE_DIR}/src
)

if (NOT DEFINED FFI_LIB_DIR)
  set(FFI_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../core/target/debug")
endif()

find_library(CRAFTCAD_DESKTOP_FFI_LIB
  NAMES craftcad_ffi_desktop
  PATHS ${FFI_LIB_DIR}
  NO_DEFAULT_PATH
)

if (NOT CRAFTCAD_DESKTOP_FFI_LIB)
  message(FATAL_ERROR "craftcad_ffi_desktop library not found. Build Rust FFI first and set -DFFI_LIB_DIR=<path>.")
endif()

target_link_libraries(craftcad_desktop PRIVATE Qt6::Widgets ${CRAFTCAD_DESKTOP_FFI_LIB})
