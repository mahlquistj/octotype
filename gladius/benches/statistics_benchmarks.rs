use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use gladius::config::Configuration;
use gladius::statistics::{Measurement, TempStatistics};
use gladius::statistics_tracker::StatisticsTracker;
use gladius::{CharacterResult, State};
use web_time::Duration;

fn benchmark_statistics_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics_update");

    let config = Configuration::default();
    let update_counts = vec![100, 1000, 10000];

    for update_count in update_counts {
        group.bench_with_input(
            BenchmarkId::new("temp_statistics", update_count),
            &update_count,
            |b, &update_count| {
                b.iter(|| {
                    let mut stats = TempStatistics::default();

                    for i in 0..update_count {
                        let char = if i % 10 == 0 { 'x' } else { 'a' }; // 10% error rate
                        let result = if i % 10 == 0 {
                            CharacterResult::Wrong
                        } else {
                            CharacterResult::Correct
                        };
                        let elapsed = Duration::from_millis(i as u64 * 50); // 50ms per keystroke

                        stats.update(
                            black_box(char),
                            black_box(result),
                            black_box(i + 1),
                            black_box(elapsed),
                            black_box(&config),
                        );
                    }

                    black_box(stats)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_statistics_tracker_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics_tracker_update");

    let config = Configuration::default();
    let update_counts = vec![100, 1000, 10000];

    for update_count in update_counts {
        group.bench_with_input(
            BenchmarkId::new("full_tracker", update_count),
            &update_count,
            |b, &update_count| {
                b.iter(|| {
                    let mut tracker = StatisticsTracker::new();

                    for i in 0..update_count {
                        let char = if i % 10 == 0 { 'x' } else { 'a' }; // 10% error rate
                        let result = if i % 10 == 0 {
                            CharacterResult::Wrong
                        } else {
                            CharacterResult::Correct
                        };

                        tracker.update(
                            black_box(char),
                            black_box(result),
                            black_box(i + 1),
                            black_box(&config),
                        );
                    }

                    black_box(tracker)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_measurement_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("measurement_creation");

    // Generate varying amounts of historical data
    let history_sizes = vec![10, 100, 1000];

    for history_size in history_sizes {
        let mut previous_measurements = Vec::new();
        let mut input_history = Vec::new();

        // Create some historical data
        for i in 0..history_size {
            let timestamp = i as f64 * 0.1;

            if i % 10 == 0 {
                // Add a measurement every 10 inputs
                let measurement = Measurement::new(
                    timestamp,
                    i + 1,
                    &previous_measurements,
                    &input_history,
                    i + 1,
                    i / 10,
                    i / 20,
                );
                previous_measurements.push(measurement);
            }

            let input = gladius::statistics::Input {
                timestamp,
                char: 'a',
                result: if i % 10 == 0 {
                    CharacterResult::Wrong
                } else {
                    CharacterResult::Correct
                },
            };
            input_history.push(input);
        }

        group.bench_with_input(
            BenchmarkId::new("new_measurement", history_size),
            &(previous_measurements, input_history),
            |b, (previous_measurements, input_history)| {
                b.iter(|| {
                    Measurement::new(
                        black_box(10.0),
                        black_box(input_history.len()),
                        black_box(previous_measurements),
                        black_box(input_history),
                        black_box(input_history.len()),
                        black_box(history_size / 10),
                        black_box(history_size / 20),
                    )
                })
            },
        );
    }

    group.finish();
}

fn benchmark_statistics_finalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics_finalization");

    let config = Configuration::default();
    let input_counts = vec![1000, 5000, 10000];

    for input_count in input_counts {
        // Pre-generate a statistics object with lots of data
        let mut stats = TempStatistics::default();

        for i in 0..input_count {
            let char = if i % 10 == 0 { 'x' } else { 'a' };
            let result = if i % 10 == 0 {
                CharacterResult::Wrong
            } else {
                CharacterResult::Correct
            };
            let elapsed = Duration::from_millis(i as u64 * 50);

            stats.update(char, result, i + 1, elapsed, &config);
        }

        let final_duration = Duration::from_millis(input_count as u64 * 50);

        group.bench_with_input(
            BenchmarkId::new("finalize", input_count),
            &(stats, final_duration, input_count),
            |b, (stats, final_duration, input_count)| {
                b.iter(|| {
                    let stats_clone = stats.clone();
                    stats_clone.finalize(black_box(*final_duration), black_box(*input_count))
                })
            },
        );
    }

    group.finish();
}

fn benchmark_character_result_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("character_result_processing");

    let results = vec![
        CharacterResult::Correct,
        CharacterResult::Wrong,
        CharacterResult::Corrected,
        CharacterResult::Deleted(State::Correct),
        CharacterResult::Deleted(State::Wrong),
    ];

    for result in results {
        group.bench_with_input(
            BenchmarkId::new("single_update", format!("{:?}", result)),
            &result,
            |b, &result| {
                b.iter(|| {
                    let mut stats = TempStatistics::default();
                    let config = Configuration::default();

                    stats.update(
                        black_box('a'),
                        black_box(result),
                        black_box(1),
                        black_box(Duration::from_millis(100)),
                        black_box(&config),
                    );

                    black_box(stats)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_error_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_tracking");

    // Test scenarios with different error rates
    let error_rates = vec![0.01, 0.05, 0.10, 0.20]; // 1%, 5%, 10%, 20%
    let input_count = 1000;

    for error_rate in error_rates {
        group.bench_with_input(
            BenchmarkId::new("error_rate", format!("{}%", (error_rate * 100.0) as u32)),
            &error_rate,
            |b, &error_rate| {
                b.iter(|| {
                    let mut stats = TempStatistics::default();
                    let config = Configuration::default();

                    for i in 0..input_count {
                        let char = 'a';
                        let result = if (i as f64 / input_count as f64) < error_rate {
                            CharacterResult::Wrong
                        } else {
                            CharacterResult::Correct
                        };
                        let elapsed = Duration::from_millis(i as u64 * 50);

                        stats.update(
                            black_box(char),
                            black_box(result),
                            black_box(i + 1),
                            black_box(elapsed),
                            black_box(&config),
                        );
                    }

                    black_box(stats)
                })
            },
        );
    }

    group.finish();
}

fn benchmark_measurement_intervals(c: &mut Criterion) {
    let mut group = c.benchmark_group("measurement_intervals");

    // Test different measurement intervals
    let intervals = vec![0.5, 1.0, 2.0, 5.0]; // seconds
    let input_count = 1000;

    for interval in intervals {
        group.bench_with_input(
            BenchmarkId::new("interval", format!("{}s", interval)),
            &interval,
            |b, &interval| {
                b.iter(|| {
                    let mut stats = TempStatistics::default();
                    let config = Configuration {
                        measurement_interval_seconds: interval,
                    };

                    for i in 0..input_count {
                        let char = if i % 10 == 0 { 'x' } else { 'a' };
                        let result = if i % 10 == 0 {
                            CharacterResult::Wrong
                        } else {
                            CharacterResult::Correct
                        };
                        let elapsed = Duration::from_millis(i as u64 * 100); // 100ms per keystroke

                        stats.update(
                            black_box(char),
                            black_box(result),
                            black_box(i + 1),
                            black_box(elapsed),
                            black_box(&config),
                        );
                    }

                    black_box(stats)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_statistics_update,
    benchmark_statistics_tracker_update,
    benchmark_measurement_creation,
    benchmark_statistics_finalization,
    benchmark_character_result_processing,
    benchmark_error_tracking,
    benchmark_measurement_intervals
);
criterion_main!(benches);

