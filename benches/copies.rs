use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pprof::criterion::{Output, PProfProfiler};

use rust_rotates::utils::*;

// use std::time::Duration;
use std::ptr;

fn seq(size: usize) -> Vec<usize> {
    let mut v = vec![0; size];
    for i in 0..size {
        v[i] = i + 1;
    }
    v
}

/// ```text
///  start
///  |                                 count = 3
/// [1  2  3  4  5  6  7  8  9 10 11 12 13 14 15]
///  [:///:] ------>                     [:\\\:]
///  src                                 dst
///
/// [1  .  .  .  .  .  .  .  .  .  .  .  1 14 15]
/// [1  2  .  .  .  .  .  .  .  .  .  .  1  2 15]
/// [1  .  3  .  .  .  .  .  .  .  .  .  1 ~~~ 3]
/// ```
fn forward_test<T>(
    copy: unsafe fn(src: *const T, dst: *mut T, count: usize),
    start: *mut T,
    distance: usize,
    count: usize,
) {
    unsafe { copy(start, start.add(distance), count) }
}

/// ```text
///                                              end
///                                    count = 3 |
/// [ 1  2  3  4  5  6  7  8  9 10 11 12 13 14 15]
///   [:\\\:]                    <------  [:///:]
///   dst                                 src
///
/// [ 1  2 15  .  .  .  .  .  .  .  .  .  .  . 15]
/// [ 1 14  .  .  .  .  .  .  .  .  .  .  . 14 15]
/// [13 ~~ 15  .  .  .  .  .  .  .  .  . 13  . 15]
/// ```
fn backward_test<T>(
    copy: unsafe fn(src: *const T, dst: *mut T, count: usize),
    end: *mut T,
    distance: usize,
    count: usize,
) {
    unsafe { copy(end.sub(count), end.sub(count - distance), count) }
}

/// ```text
///   count = 4
///   start
///   |
/// [ 1  2  3  4  5  6  7  8  9]
///   [://////:]
///         d = 3
/// [ 1  2  1  2  3  4  7  8  9]
///         [:\\\\\\:]
/// ```
fn case_copy_forward(c: &mut Criterion, count: usize, distances: &[usize]) {
    let mut group = c.benchmark_group(format!("Copy forward/{count}"));
    let mut v = seq(count * 2 + 1);

    for d in distances {
        let start = &v[..].as_mut_ptr();

        group.bench_with_input(BenchmarkId::new("*utils::copy_forward", d), d, |b, _| {
            b.iter(|| forward_test(copy_forward::<usize>, *start, *d, count))
        });
        group.bench_with_input(BenchmarkId::new("utils::copy_backward", d), d, |b, _| {
            b.iter(|| forward_test(copy_backward::<usize>, *start, *d, count))
        });
        group.bench_with_input(
            BenchmarkId::new("ptr::copy_nonoverlapping", d),
            d,
            |b, _d| b.iter(|| forward_test(ptr::copy_nonoverlapping::<usize>, *start, *d, count)),
        );
        group.bench_with_input(BenchmarkId::new("ptr::copy", d), d, |b, _d| {
            b.iter(|| forward_test(ptr::copy::<usize>, *start, *d, count))
        });
    }

    group.finish();
}

/// ```text
///   start
///   |               count = 4
/// [ 1  2  3  4  5  6  7  8  9]
///                  [:\\\\\\:]
///                     d = 3
/// [ 1  2  1  2  3  4  7  8  9]
///            [://////:]
/// ```
fn case_copy_backward(c: &mut Criterion, count: usize, distances: &[usize]) {
    let mut group = c.benchmark_group(format!("Copy backward/{count}"));
    let len = count * 2 + 1;
    let mut v = seq(len);

    for d in distances {
        let end = unsafe { &v[..].as_mut_ptr().add(len) };

        group.bench_with_input(BenchmarkId::new("utils::copy_forward", d), d, |b, _d| {
            b.iter(|| backward_test(copy_forward::<usize>, *end, *d, count))
        });
        group.bench_with_input(BenchmarkId::new("*utils::copy_backward", d), d, |b, _d| {
            b.iter(|| backward_test(copy_backward::<usize>, *end, *d, count))
        });
        group.bench_with_input(
            BenchmarkId::new("ptr::copy_nonoverlapping", d),
            d,
            |b, _d| b.iter(|| backward_test(ptr::copy::<usize>, *end, *d, count)),
        );
        group.bench_with_input(BenchmarkId::new("ptr::copy", d), d, |b, _d| {
            b.iter(|| backward_test(ptr::copy::<usize>, *end, *d, count))
        });
    }

    group.finish();
}

fn case_copy_distance(c: &mut Criterion, count: usize, distances: &[usize]) {
    let mut group = c.benchmark_group(format!("Copy distances/{count}"));
    let max_distance = distances.iter().max().unwrap();
    let mut v = seq(count + max_distance);

    for d in distances {
        let start = &v[..].as_mut_ptr();

        group.bench_with_input(BenchmarkId::new("utils::copy_forward", d), d, |b, _| {
            b.iter(|| backward_test(copy_forward::<usize>, *start, *d, count))
        });
        group.bench_with_input(BenchmarkId::new("utils::copy_backward", d), d, |b, _| {
            b.iter(|| backward_test(copy_backward::<usize>, *start, *d, count))
        });
        group.bench_with_input(
            BenchmarkId::new("*ptr::copy_nonoverlapping", d),
            d,
            |b, _| b.iter(|| backward_test(ptr::copy_nonoverlapping::<usize>, *start, *d, count)),
        );
        group.bench_with_input(BenchmarkId::new("ptr::copy", d), d, |b, _| {
            b.iter(|| backward_test(ptr::copy::<usize>, *start, *d, count))
        });
    }

    group.finish();
}

/// cargo bench --bench=copies "Copy distance/1"
fn bench_copy_distance(c: &mut Criterion) {
    case_copy_distance(c, 1, &[0, 1, 2, 3, 5, 20, 50]);
    case_copy_distance(c, 2, &[0, 1, 2, 3, 4, 5, 20, 50]);
}

/// cargo bench --bench=copies "Copy forward/2"
fn bench_copy_forward(c: &mut Criterion) {
    // case_copy_forward(c, 2, &[0, 1]);
    // case_copy_forward(c, 3, &[0, 1, 2]);
    // case_copy_forward(c, 5, &[0, 1, 2, 3, 4, 5]);
    case_copy_forward(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_copy_forward(c, 100_000, &[0, 50_000, 100_000, 150_000]);

    // case_copy_forward(
    //     c,
    //     15,
    //     &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    // );
    // case_copy_forward(
    //     c,
    //     20,
    //     &[
    //         0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    //     ],
    // );
}

/// cargo bench --bench=copies "Copy backward/2"
fn bench_copy_backward(c: &mut Criterion) {
    // case_copy_backward(c, 2, &[0, 1]);
    // case_copy_backward(c, 3, &[0, 1, 2]);
    // case_copy_backward(c, 5, &[0, 1, 2, 3, 4, 5]);
    case_copy_backward(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    // case_copy_backward(
    //     c,
    //     15,
    //     &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    // );
    // case_copy_backward(
    //     c,
    //     20,
    //     &[
    //         0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    //     ],
    // );
}

criterion_group! {
    name = benches;

//    config = Criterion::default().sample_size(500).measurement_time(Duration::new(120, 0));
    config = Criterion::default()
             .sample_size(500)
             .with_profiler(
                  PProfProfiler::new(100, Output::Flamegraph(None))
              );

    targets = bench_copy_distance, bench_copy_backward, bench_copy_forward
}

criterion_main!(benches);
