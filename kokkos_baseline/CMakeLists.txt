# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals / Squirrel Team
#
# Kokkos Tier 1 validation baseline for groundSpring.
# Python validates correctness. Kokkos validates competitiveness.
#
# Build:
#   cmake -B build -DCMAKE_BUILD_TYPE=Release
#   cmake --build build -j$(nproc)
#
# Build with CUDA:
#   cmake -B build -DCMAKE_BUILD_TYPE=Release -DKokkos_ENABLE_CUDA=ON
#   cmake --build build -j$(nproc)
#
# Build with OpenMP:
#   cmake -B build -DCMAKE_BUILD_TYPE=Release -DKokkos_ENABLE_OPENMP=ON
#   cmake --build build -j$(nproc)

cmake_minimum_required(VERSION 3.22)

option(ENABLE_CUDA "Build with Kokkos CUDA backend" OFF)

if(ENABLE_CUDA)
    project(groundspring_kokkos_baseline VERSION 0.1.0 LANGUAGES CXX CUDA)
else()
    project(groundspring_kokkos_baseline VERSION 0.1.0 LANGUAGES CXX)
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(FetchContent)

# Kokkos options must be set before FetchContent populates
if(ENABLE_CUDA)
    set(Kokkos_ENABLE_CUDA ON CACHE BOOL "" FORCE)
    set(Kokkos_ENABLE_CUDA_LAMBDA ON CACHE BOOL "" FORCE)
    set(Kokkos_ENABLE_COMPILE_AS_CMAKE_LANGUAGE ON CACHE BOOL "" FORCE)
endif()

FetchContent_Declare(
    kokkos
    GIT_REPOSITORY https://github.com/kokkos/kokkos.git
    GIT_TAG        4.5.01
    GIT_SHALLOW    TRUE
)

if(NOT kokkos_POPULATED)
    FetchContent_MakeAvailable(kokkos)
endif()

add_executable(kokkos_baseline
    src/main.cpp
)

if(ENABLE_CUDA)
    set_source_files_properties(src/main.cpp PROPERTIES LANGUAGE CUDA)
    target_compile_options(kokkos_baseline PRIVATE
        $<$<COMPILE_LANGUAGE:CUDA>:--extended-lambda --expt-relaxed-constexpr>
    )
else()
    target_compile_options(kokkos_baseline PRIVATE
        $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall -Wextra -Wpedantic>
    )
endif()

target_link_libraries(kokkos_baseline PRIVATE Kokkos::kokkos)
