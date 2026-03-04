// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals / Squirrel Team
//
// Kokkos Tier 1 validation baseline for groundSpring.
//
// Implements the same algorithms as the Rust library using Kokkos
// parallel primitives (parallel_for, parallel_reduce, parallel_scan).
// Output is JSON with provenance for comparison against Rust/BarraCuda.
//
// Three benchmark kernels:
//   1. Anderson localization — Lyapunov exponent via transfer matrix
//   2. Statistical reductions — mean, variance, Pearson correlation
//   3. Bootstrap resampling  — percentile confidence intervals
//
// These cover the three main Kokkos patterns:
//   parallel_for   (Anderson potential generation, bootstrap sampling)
//   parallel_reduce (Lyapunov averaging, statistics, correlation)
//   Views           (matrix/vector memory management)

#include <Kokkos_Core.hpp>
#include <Kokkos_Random.hpp>

#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <algorithm>
#include <string>
#include <vector>

// ---------------------------------------------------------------------------
// Xorshift64 — matches groundspring::prng::Xorshift64 exactly
// ---------------------------------------------------------------------------

struct Xorshift64 {
    uint64_t state;

    KOKKOS_INLINE_FUNCTION
    explicit Xorshift64(uint64_t seed) : state(seed == 0 ? 1 : seed) {}

    KOKKOS_INLINE_FUNCTION
    uint64_t next() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        return state;
    }

    KOKKOS_INLINE_FUNCTION
    double next_f64() {
        return static_cast<double>(next()) / static_cast<double>(UINT64_MAX);
    }
};

// ---------------------------------------------------------------------------
// Benchmark timing
// ---------------------------------------------------------------------------

struct BenchResult {
    std::string name;
    double value;
    double elapsed_us;
};

static double now_us() {
    using clk = std::chrono::high_resolution_clock;
    static auto t0 = clk::now();
    return std::chrono::duration<double, std::micro>(clk::now() - t0).count();
}

// ---------------------------------------------------------------------------
// 1. Anderson localization — Lyapunov exponent
// ---------------------------------------------------------------------------
// Transfer-matrix method: T_n = [[E - V(n), -1], [1, 0]]
// gamma = (1/N) sum ln(norm) after renormalization at each step.
//
// The single-realization Lyapunov is inherently sequential (each step
// depends on the previous). Kokkos parallelism is over REALIZATIONS:
// generate many random potentials, compute gamma for each, then average.

static BenchResult bench_anderson_lyapunov(int n_sites, double disorder,
                                           int n_realizations, double energy,
                                           uint64_t base_seed) {
    double t0 = now_us();

    // Parallel over realizations: generate potential + compute Lyapunov
    double gamma_sum = 0.0;
    Kokkos::parallel_reduce(
        "anderson_lyapunov", n_realizations,
        KOKKOS_LAMBDA(int r, double& local_sum) {
            Xorshift64 rng(base_seed + static_cast<uint64_t>(r));
            double half_w = disorder / 2.0;
            double log_growth = 0.0;
            double v0 = 1.0, v1 = 0.0;
            for (int i = 0; i < n_sites; ++i) {
                double pot = rng.next_f64() * disorder - half_w;
                double new_0 = (energy - pot) * v0 - v1;
                double new_1 = v0;
                v0 = new_0;
                v1 = new_1;
                double norm = Kokkos::sqrt(v0 * v0 + v1 * v1);
                if (norm > 0.0) {
                    log_growth += Kokkos::log(norm);
                    v0 /= norm;
                    v1 /= norm;
                }
            }
            local_sum += log_growth / static_cast<double>(n_sites);
        },
        gamma_sum);

    double gamma_avg = gamma_sum / static_cast<double>(n_realizations);
    double elapsed = now_us() - t0;

    return {"anderson_lyapunov_averaged", gamma_avg, elapsed};
}

// ---------------------------------------------------------------------------
// 2. Statistical reductions — mean, variance, Pearson r
// ---------------------------------------------------------------------------

static BenchResult bench_mean(const Kokkos::View<double*>& data, int n) {
    double t0 = now_us();
    double sum = 0.0;
    Kokkos::parallel_reduce(
        "mean", n,
        KOKKOS_LAMBDA(int i, double& local_sum) { local_sum += data(i); },
        sum);
    double result = sum / static_cast<double>(n);
    return {"mean", result, now_us() - t0};
}

static BenchResult bench_variance(const Kokkos::View<double*>& data, int n,
                                  double mean_val) {
    double t0 = now_us();
    double ss = 0.0;
    Kokkos::parallel_reduce(
        "variance", n,
        KOKKOS_LAMBDA(int i, double& local_ss) {
            double d = data(i) - mean_val;
            local_ss += d * d;
        },
        ss);
    double result = ss / static_cast<double>(n);
    return {"variance", result, now_us() - t0};
}

static BenchResult bench_pearson_r(const Kokkos::View<double*>& x,
                                   const Kokkos::View<double*>& y, int n,
                                   double mx, double my) {
    double t0 = now_us();

    double sum_xy = 0.0, sum_xx = 0.0, sum_yy = 0.0;

    Kokkos::parallel_reduce(
        "pearson_r", n,
        KOKKOS_LAMBDA(int i, double& lxy, double& lxx, double& lyy) {
            double dx = x(i) - mx;
            double dy = y(i) - my;
            lxy += dx * dy;
            lxx += dx * dx;
            lyy += dy * dy;
        },
        sum_xy, sum_xx, sum_yy);

    double denom = std::sqrt(sum_xx * sum_yy);
    double result = (denom > 0.0) ? sum_xy / denom : 0.0;
    return {"pearson_r", result, now_us() - t0};
}

// ---------------------------------------------------------------------------
// 3. Bootstrap resampling — percentile confidence interval for the mean
// ---------------------------------------------------------------------------

static BenchResult bench_bootstrap_mean(const Kokkos::View<double*>& data,
                                        int n, int n_replicates,
                                        double confidence, uint64_t seed) {
    double t0 = now_us();

    Kokkos::View<double*> replicate_means("replicate_means", n_replicates);

    Kokkos::parallel_for(
        "bootstrap_resample", n_replicates,
        KOKKOS_LAMBDA(int r) {
            Xorshift64 rng(seed + static_cast<uint64_t>(r) * 997);
            double sum = 0.0;
            for (int i = 0; i < n; ++i) {
                int idx = static_cast<int>(rng.next() % static_cast<uint64_t>(n));
                sum += data(idx);
            }
            replicate_means(r) = sum / static_cast<double>(n);
        });

    // Copy to host and sort for percentile CI
    auto h_means = Kokkos::create_mirror_view_and_copy(Kokkos::HostSpace(),
                                                        replicate_means);
    std::vector<double> sorted(n_replicates);
    for (int i = 0; i < n_replicates; ++i) {
        sorted[static_cast<size_t>(i)] = h_means(i);
    }
    std::sort(sorted.begin(), sorted.end());

    double alpha = 1.0 - confidence;
    int lo_idx = static_cast<int>(alpha / 2.0 * static_cast<double>(n_replicates));
    int hi_idx = static_cast<int>((1.0 - alpha / 2.0) * static_cast<double>(n_replicates));
    if (hi_idx >= n_replicates) hi_idx = n_replicates - 1;

    double estimate = 0.0;
    for (int i = 0; i < n_replicates; ++i) estimate += sorted[static_cast<size_t>(i)];
    estimate /= static_cast<double>(n_replicates);

    double elapsed = now_us() - t0;

    std::printf("    bootstrap: estimate=%.10f ci=[%.10f, %.10f]\n",
                estimate, sorted[static_cast<size_t>(lo_idx)],
                sorted[static_cast<size_t>(hi_idx)]);

    return {"bootstrap_mean", estimate, elapsed};
}

// ---------------------------------------------------------------------------
// JSON output with provenance
// ---------------------------------------------------------------------------

static void emit_json(const std::vector<BenchResult>& results,
                      const char* backend_name) {
    std::printf("{\n");
    std::printf("  \"_source\": \"Kokkos Tier 1 validation baseline — groundSpring\",\n");
    std::printf("  \"_provenance\": {\n");
    std::printf("    \"baseline_date\": \"2026-03-04\",\n");
    std::printf("    \"kokkos_version\": \"4.5.01\",\n");
    std::printf("    \"backend\": \"%s\",\n", backend_name);
    std::printf("    \"generated_by\": \"kokkos_baseline/src/main.cpp\"\n");
    std::printf("  },\n");
    std::printf("  \"results\": [\n");
    for (size_t i = 0; i < results.size(); ++i) {
        std::printf("    {\"name\": \"%s\", \"value\": %.15e, \"elapsed_us\": %.1f}",
                    results[i].name.c_str(), results[i].value,
                    results[i].elapsed_us);
        if (i + 1 < results.size()) std::printf(",");
        std::printf("\n");
    }
    std::printf("  ]\n");
    std::printf("}\n");
}

// ---------------------------------------------------------------------------
// Determine Kokkos backend name
// ---------------------------------------------------------------------------

static const char* kokkos_backend_name() {
#if defined(KOKKOS_ENABLE_CUDA)
    return "CUDA";
#elif defined(KOKKOS_ENABLE_HIP)
    return "HIP";
#elif defined(KOKKOS_ENABLE_SYCL)
    return "SYCL";
#elif defined(KOKKOS_ENABLE_OPENMP)
    return "OpenMP";
#elif defined(KOKKOS_ENABLE_THREADS)
    return "Threads";
#else
    return "Serial";
#endif
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

int main(int argc, char* argv[]) {
    Kokkos::initialize(argc, argv);
    {
        std::vector<BenchResult> results;
        const char* backend = kokkos_backend_name();

        std::printf("groundSpring Kokkos Tier 1 Baseline\n");
        std::printf("  Backend: %s\n", backend);
        std::printf("  Kokkos version: %d.%d.%02d\n",
                    KOKKOS_VERSION / 10000,
                    (KOKKOS_VERSION / 100) % 100,
                    KOKKOS_VERSION % 100);
        std::printf("\n");

        // ------------------------------------------------------------------
        // 1. Anderson localization
        // ------------------------------------------------------------------
        {
            constexpr int N_SITES = 10000;
            constexpr double DISORDER = 4.0;
            constexpr int N_REALIZATIONS = 500;
            constexpr double ENERGY = 0.0;
            constexpr uint64_t BASE_SEED = 42;

            std::printf("=== Anderson Localization (Lyapunov Exponent) ===\n");
            std::printf("  N=%d, W=%.1f, realizations=%d, E=%.1f\n",
                        N_SITES, DISORDER, N_REALIZATIONS, ENERGY);

            auto r = bench_anderson_lyapunov(N_SITES, DISORDER,
                                             N_REALIZATIONS, ENERGY, BASE_SEED);
            double xi = (r.value > 0.0) ? 1.0 / r.value : 1e30;
            std::printf("  gamma_avg = %.10f (xi = %.4f)\n", r.value, xi);
            std::printf("  Derrida-Gardner: xi ~ 96/W^2 = %.4f\n",
                        96.0 / (DISORDER * DISORDER));
            std::printf("  elapsed: %.0f us\n\n", r.elapsed_us);
            results.push_back(r);
        }

        // ------------------------------------------------------------------
        // 2. Statistical reductions
        // ------------------------------------------------------------------
        {
            constexpr int N = 1000000;
            constexpr uint64_t SEED = 12345;

            std::printf("=== Statistical Reductions (N=%d) ===\n", N);

            // Generate synthetic data on device
            Kokkos::View<double*> data("data", N);
            Kokkos::View<double*> data2("data2", N);
            Kokkos::parallel_for(
                "gen_data", N,
                KOKKOS_LAMBDA(int i) {
                    Xorshift64 rng(SEED + static_cast<uint64_t>(i));
                    data(i) = rng.next_f64() * 100.0;
                    Xorshift64 rng2(SEED + 1000000 + static_cast<uint64_t>(i));
                    double noise = rng2.next_f64() * 10.0;
                    data2(i) = data(i) * 0.8 + noise + 5.0;
                });

            auto r_mean = bench_mean(data, N);
            std::printf("  mean = %.10f (%.0f us)\n",
                        r_mean.value, r_mean.elapsed_us);
            results.push_back(r_mean);

            auto r_var = bench_variance(data, N, r_mean.value);
            std::printf("  variance = %.10f (%.0f us)\n",
                        r_var.value, r_var.elapsed_us);
            results.push_back(r_var);

            // Compute mean of data2 for Pearson
            double sum2 = 0.0;
            Kokkos::parallel_reduce(
                "mean2", N,
                KOKKOS_LAMBDA(int i, double& s) { s += data2(i); },
                sum2);
            double my = sum2 / static_cast<double>(N);

            auto r_pearson = bench_pearson_r(data, data2, N, r_mean.value, my);
            std::printf("  pearson_r = %.10f (%.0f us)\n",
                        r_pearson.value, r_pearson.elapsed_us);
            results.push_back(r_pearson);
            std::printf("\n");
        }

        // ------------------------------------------------------------------
        // 3. Bootstrap resampling
        // ------------------------------------------------------------------
        {
            constexpr int N = 10000;
            constexpr int N_REPLICATES = 5000;
            constexpr double CONFIDENCE = 0.95;
            constexpr uint64_t SEED = 99;

            std::printf("=== Bootstrap Resampling (N=%d, B=%d) ===\n",
                        N, N_REPLICATES);

            Kokkos::View<double*> data("boot_data", N);
            Kokkos::parallel_for(
                "gen_boot", N,
                KOKKOS_LAMBDA(int i) {
                    Xorshift64 rng(SEED + static_cast<uint64_t>(i));
                    data(i) = rng.next_f64() * 50.0 + 25.0;
                });

            auto r = bench_bootstrap_mean(data, N, N_REPLICATES, CONFIDENCE, SEED);
            std::printf("  elapsed: %.0f us\n\n", r.elapsed_us);
            results.push_back(r);
        }

        // ------------------------------------------------------------------
        // JSON output
        // ------------------------------------------------------------------
        std::printf("=== JSON Benchmark Output ===\n");
        emit_json(results, backend);
    }
    Kokkos::finalize();
    return 0;
}
