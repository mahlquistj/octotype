use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use gladius::math::{Accuracy, Consistency, Ipm, Wpm};

fn benchmark_wpm_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("wpm_calculations");

    // Benchmark with various input sizes
    let test_cases = vec![
        (100, 5, 2, 1.0),         // Small dataset
        (1000, 50, 20, 10.0),     // Medium dataset
        (10000, 500, 200, 100.0), // Large dataset
    ];

    for (characters, errors, corrections, minutes) in test_cases {
        group.bench_with_input(
            BenchmarkId::new(
                "calculate",
                format!("{}chars_{}min", characters, minutes as u32),
            ),
            &(characters, errors, corrections, minutes),
            |b, &(characters, errors, corrections, minutes)| {
                b.iter(|| {
                    Wpm::calculate(
                        black_box(characters),
                        black_box(errors),
                        black_box(corrections),
                        black_box(minutes),
                    )
                })
            },
        );
    }

    group.finish();
}

fn benchmark_ipm_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipm_calculations");

    let test_cases = vec![
        (100, 120, 1.0),       // Small dataset
        (1000, 1200, 10.0),    // Medium dataset
        (10000, 12000, 100.0), // Large dataset
    ];

    for (actual_inputs, raw_inputs, minutes) in test_cases {
        group.bench_with_input(
            BenchmarkId::new(
                "calculate",
                format!("{}inputs_{}min", actual_inputs, minutes as u32),
            ),
            &(actual_inputs, raw_inputs, minutes),
            |b, &(actual_inputs, raw_inputs, minutes)| {
                b.iter(|| {
                    Ipm::calculate(
                        black_box(actual_inputs),
                        black_box(raw_inputs),
                        black_box(minutes),
                    )
                })
            },
        );
    }

    group.finish();
}

fn benchmark_accuracy_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("accuracy_calculations");

    let test_cases = vec![
        (100, 5, 2),       // Small dataset
        (1000, 50, 20),    // Medium dataset
        (10000, 500, 200), // Large dataset
    ];

    for (input_len, total_errors, total_corrections) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("calculate", format!("{}chars", input_len)),
            &(input_len, total_errors, total_corrections),
            |b, &(input_len, total_errors, total_corrections)| {
                b.iter(|| {
                    Accuracy::calculate(
                        black_box(input_len),
                        black_box(total_errors),
                        black_box(total_corrections),
                    )
                })
            },
        );
    }

    group.finish();
}

fn benchmark_consistency_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("consistency_calculations");

    // Generate test data sets of different sizes
    let test_sizes = vec![10, 100, 1000];

    for size in test_sizes {
        // Generate realistic WPM measurements with some variation
        let mut measurements = Vec::with_capacity(size);
        let base_wpm = 50.0;
        for i in 0..size {
            let variation = (i as f64 * 0.1).sin() * 5.0; // Sine wave variation
            measurements.push(Wpm {
                raw: base_wpm + variation,
                corrected: base_wpm + variation - 2.0,
                actual: base_wpm + variation - 3.0,
            });
        }

        group.bench_with_input(
            BenchmarkId::new("calculate", format!("{}measurements", size)),
            &measurements,
            |b, measurements| b.iter(|| Consistency::calculate(black_box(measurements))),
        );
    }

    group.finish();
}

fn benchmark_consistency_std_dev_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("consistency_std_dev");

    // Test different algorithms for standard deviation calculation
    let sizes = vec![10, 100, 1000, 10000];

    for size in sizes {
        let values: Vec<f64> = (0..size)
            .map(|i| 50.0 + (i as f64 * 0.1).sin() * 5.0)
            .collect();

        // Benchmark the current Welford's algorithm implementation
        group.bench_with_input(
            BenchmarkId::new("welfords_algorithm", size),
            &values,
            |b, values| b.iter(|| calculate_std_dev_welford(black_box(values))),
        );

        // Benchmark naive two-pass algorithm for comparison
        group.bench_with_input(
            BenchmarkId::new("naive_two_pass", size),
            &values,
            |b, values| b.iter(|| calculate_std_dev_naive(black_box(values))),
        );
    }

    group.finish();
}

// Helper function that mirrors the Welford's algorithm from the main code
fn calculate_std_dev_welford(values: &[f64]) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let mut mean = 0.0;
    let mut m2 = 0.0;

    for (i, &value) in values.iter().enumerate() {
        let delta = value - mean;
        mean += delta / (i + 1) as f64;
        let delta2 = value - mean;
        m2 += delta * delta2;
    }

    let variance = m2 / values.len() as f64;
    variance.sqrt()
}

// Naive two-pass algorithm for comparison
fn calculate_std_dev_naive(values: &[f64]) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    // First pass: calculate mean
    let mean = values.iter().sum::<f64>() / values.len() as f64;

    // Second pass: calculate variance
    let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;

    variance.sqrt()
}

criterion_group!(
    benches,
    benchmark_wpm_calculations,
    benchmark_ipm_calculations,
    benchmark_accuracy_calculations,
    benchmark_consistency_calculations,
    benchmark_consistency_std_dev_algorithms
);
criterion_main!(benches);

