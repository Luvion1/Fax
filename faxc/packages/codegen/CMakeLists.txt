cmake_minimum_required(VERSION 3.10)
project(faxc-cpp VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 17)

# Include directories
include_directories(src)
include_directories(src/backend)
include_directories(src/api)
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../include)

# Source files
set(SOURCES
    src/main.cpp
    src/api/json_api.cpp
    src/backend/codegen.cpp
    src/backend/optimizer.cpp
    src/backend/analyzer.cpp
    src/backend/type_system.cpp
    src/backend/symbol_table.cpp
    src/backend/instruction_builder.cpp
)

add_executable(faxc_cpp ${SOURCES})