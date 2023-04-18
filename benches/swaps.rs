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
    swap: unsafe fn(src: *mut T, dst: *mut T, count: usize),
    start: *mut T,
    distance: usize,
    count: usize,
) {
    unsafe { swap(start, start.add(distance), count) }
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
    swap: unsafe fn(src: *mut T, dst: *mut T, count: usize),
    end: *mut T,
    distance: usize,
    count: usize,
) {
    unsafe { swap(end.sub(count), end.sub(count + distance), count) }
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
fn case_swap_forward(c: &mut Criterion, count: usize, distances: &[usize]) {
    let mut group = c.benchmark_group(format!("Swap forward/{count}"));
    let mut v = seq(count * 2 + 1);

    for d in distances {
        let start = &v[..].as_mut_ptr();

        group.bench_with_input(BenchmarkId::new("*utils::swap_forward", d), d, |b, _| {
            b.iter(|| forward_test(swap_forward::<usize>, *start, *d, count))
        });
        group.bench_with_input(BenchmarkId::new("utils::swap_backward", d), d, |b, _| {
            b.iter(|| forward_test(swap_backward::<usize>, *start, *d, count))
        });
        group.bench_with_input(
            BenchmarkId::new("ptr::swap_nonoverlapping", d),
            d,
            |b, _d| b.iter(|| forward_test(ptr::swap_nonoverlapping::<usize>, *start, *d, count)),
        );
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
fn case_swap_backward(c: &mut Criterion, count: usize, distances: &[usize]) {
    let mut group = c.benchmark_group(format!("Swap backward/{count}"));
    let len = count * 2 + 1;
    let mut v = seq(len);

    for d in distances {
        let end = unsafe { &v[..].as_mut_ptr().add(len) };

        group.bench_with_input(BenchmarkId::new("utils::swap_forward", d), d, |b, _d| {
            b.iter(|| backward_test(swap_forward::<usize>, *end, *d, count))
        });
        group.bench_with_input(BenchmarkId::new("*utils::swap_backward", d), d, |b, _d| {
            b.iter(|| backward_test(swap_backward::<usize>, *end, *d, count))
        });
        group.bench_with_input(
            BenchmarkId::new("ptr::swap_nonoverlapping", d),
            d,
            |b, _d| b.iter(|| backward_test(ptr::swap_nonoverlapping::<usize>, *end, *d, count)),
        );
    }

    group.finish();
}

/// cargo bench --bench=copies "Swap forward/10"
fn bench_swap_forward(c: &mut Criterion) {
    case_swap_forward(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_swap_forward(c, 100_000, &[0, 25_000, 50_000, 75_000, 99_000, 100_000]);
}

/// cargo bench --bench=copies "Swap backward/10"
fn bench_swap_backward(c: &mut Criterion) {
    case_swap_backward(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_swap_backward(c, 100_000, &[0, 25_000, 50_000, 75_000, 99_000, 100_000]);
}

criterion_group! {
    name = benches;

//    config = Criterion::default().sample_size(500).measurement_time(Duration::new(120, 0));
    config = Criterion::default()
             .sample_size(500)
             .with_profiler(
                  PProfProfiler::new(100, Output::Flamegraph(None))
              );

    targets = bench_swap_backward, bench_swap_forward
}

criterion_main!(benches);
