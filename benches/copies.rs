use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pprof::criterion::{Output, PProfProfiler};

use rust_rotations::{ptr_contrev_rotate, ptr_direct_rotate, ptr_reversal_rotate, utils::*};

// use std::time::Duration;
use std::ptr;

fn seq<const count: usize>(size: usize) -> Vec<[usize; count]> {
    let mut v = vec![[0; count]; size];
    for i in 0..size {
        v[i] = [i + 1; count];
    }
    v
}

/// ```text
///  start
///  | distance = 12                  count = 3
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
///   distance = 12                    count = 3 |
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
    unsafe { copy(end.sub(count), end.sub(count + distance), count) }
}

/// ```text
///   count = 4
///   start
///   |
/// [ 1  2  3  4  5  6  7  8  9]
///   [://////:]
///         distance = 2
/// [ 1  2  1  2  3  4  7  8  9]
///         [:\\\\\\:]
/// ```
fn case_copy_overlapping_forward<const count: usize>(
    c: &mut Criterion,
    len: usize,
    distances: &[usize],
) {
    let mut group = c.benchmark_group(format!("Copy forward/{len}/{count}"));
    let mut v = seq::<count>(len * 2 + 1);

    for d in distances {
        let start = *&v[..].as_mut_ptr();

        group.bench_with_input(BenchmarkId::new("utils::copy_forward (!)", d), d, |b, _| {
            b.iter(|| forward_test(copy_forward::<[usize; count]>, start, *d, len))
        });

        group.bench_with_input(BenchmarkId::new("utils::copy_backward", d), d, |b, _| {
            b.iter(|| forward_test(copy_backward::<[usize; count]>, start, *d, len))
        });

        group.bench_with_input(
            BenchmarkId::new("ptr::copy_nonoverlapping (!)", d),
            d,
            |b, _d| {
                b.iter(|| forward_test(ptr::copy_nonoverlapping::<[usize; count]>, start, *d, len))
            },
        );

        group.bench_with_input(BenchmarkId::new("ptr::copy", d), d, |b, _d| {
            b.iter(|| forward_test(ptr::copy::<[usize; count]>, start, *d, len))
        });
    }

    group.finish();
}

/// ```text
///   start
///   |               count = 4
/// [ 1  2  3  4  5  6  7  8  9]
///                  [:\\\\\\:]
///                       d = 3
/// [ 1  2  1  2  3  4  7  8  9]
///            [://////:]
/// ```
fn case_copy_overlapping_backward<const count: usize>(
    c: &mut Criterion,
    len: usize,
    distances: &[usize],
) {
    let mut group = c.benchmark_group(format!("Copy backward/{len}/{count}"));
    let mut v = seq::<count>(2 * len + 1);

    for d in distances {
        let end = *unsafe { &v[..].as_mut_ptr().add(2 * len + 1) };

        group.bench_with_input(BenchmarkId::new("utils::copy_forward", d), d, |b, _d| {
            b.iter(|| backward_test(copy_forward::<[usize; count]>, end, *d, len))
        });

        group.bench_with_input(
            BenchmarkId::new("utils::copy_backward (!)", d),
            d,
            |b, _d| b.iter(|| backward_test(copy_backward::<[usize; count]>, end, *d, len)),
        );

        group.bench_with_input(
            BenchmarkId::new("ptr::copy_nonoverlapping (!)", d),
            d,
            |b, _d| b.iter(|| backward_test(ptr::copy::<[usize; count]>, end, *d, len)),
        );

        group.bench_with_input(BenchmarkId::new("ptr::copy", d), d, |b, _d| {
            b.iter(|| backward_test(ptr::copy::<[usize; count]>, end, *d, len))
        });
    }

    group.finish();
}

/// ```text
///   start           
///   |               count = 4
/// [ 1  2  3  4  5  6  7  8  9]
///   [://////:]
///                       d = 3
/// [ 1  2  1  2  3  4  7  8  9]
///            [://////:]
/// ```
fn case_copy_distance<const count: usize>(c: &mut Criterion, len: usize, distances: &[usize]) {
    let mut group = c.benchmark_group(format!("Copy distances/{len}/{count}"));
    let max_distance = distances.iter().max().unwrap();
    let mut v = seq(len + max_distance);

    for d in distances {
        let start = *&v[..].as_mut_ptr();
        let end = unsafe { start.add(len + max_distance) };

        group.bench_with_input(BenchmarkId::new("utils::copy_forward", d), d, |b, _| {
            b.iter(|| forward_test(copy_forward::<[usize; count]>, start, *d, len))
        });

        group.bench_with_input(BenchmarkId::new("utils::copy_backward", d), d, |b, _| {
            b.iter(|| backward_test(copy_backward::<[usize; count]>, end, *d, len))
        });

        group.bench_with_input(
            BenchmarkId::new("ptr::copy_nonoverlapping (!)", d),
            d,
            |b, _| {
                b.iter(|| forward_test(ptr::copy_nonoverlapping::<[usize; count]>, start, *d, len))
            },
        );

        group.bench_with_input(BenchmarkId::new("ptr::copy", d), d, |b, _| {
            b.iter(|| forward_test(ptr::copy::<[usize; count]>, start, *d, len))
        });
    }

    group.finish();
}

/// ```text
///   start                  end
///   |              count = 4 |
/// [ 1  2  3  4  5  6  7  8  9]
///                  [:\\\\\\:]
///
/// [ 1  2  3  4  6  7  8  9  9]
///               [://////:]
/// ```
fn case_shift_left<const count: usize>(c: &mut Criterion, lens: &[usize]) {
    let max_len = *lens.iter().max().unwrap();
    let mut group = c.benchmark_group(format!("Shift left/{max_len}/{count}"));
    let mut v = seq(max_len + 1);

    let start = *&v[..].as_mut_ptr();
    let end = unsafe { start.add(max_len + 1) };

    for l in lens {
        group.bench_with_input(BenchmarkId::new("utils::copy_forward", l), l, |b, _l| {
            b.iter(|| backward_test(copy_forward::<[usize; count]>, end, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("ptr::copy", l), l, |b, _l| {
            b.iter(|| backward_test(ptr::copy::<[usize; count]>, end, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("utils::shift_left", l), l, |b, _l| {
            b.iter(|| unsafe { shift_left::<[usize; count]>(start.add(1), *l) })
        });

        group.bench_with_input(BenchmarkId::new("ptr_contrev_rotate", l), l, |b, _l| {
            b.iter(|| unsafe { ptr_contrev_rotate::<[usize; count]>(1, start.add(1), *l) })
        });

        group.bench_with_input(BenchmarkId::new("ptr_reversal_rotate", l), l, |b, _l| {
            b.iter(|| unsafe { ptr_reversal_rotate::<[usize; count]>(1, start.add(1), *l) })
        });

        group.bench_with_input(BenchmarkId::new("ptr_direct_rotate", l), l, |b, _l| {
            b.iter(|| unsafe { ptr_direct_rotate::<[usize; count]>(1, start.add(1), *l) })
        });
    }

    group.finish();
}

/// ```text
///   start                  end
///   |              count = 4 |
/// [ 1  2  3  4  5  6  7  8  9]
///                  [:\\\\\\:]
///
/// [ 1  2  3  4  6  7  8  9  9]
///               [://////:]
/// ```
fn case_shift_right<const count: usize>(c: &mut Criterion, lens: &[usize]) {
    let max_len = *lens.iter().max().unwrap();
    let mut group = c.benchmark_group(format!("Shift right/{max_len}/{count}"));
    let mut v = seq(max_len + 1);

    let start = *&v[..].as_mut_ptr();

    for l in lens {
        group.bench_with_input(BenchmarkId::new("utils::copy_backward", l), l, |b, _l| {
            b.iter(|| forward_test(copy_forward::<[usize; count]>, start, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("ptr::copy", l), l, |b, _l| {
            b.iter(|| forward_test(ptr::copy::<[usize; count]>, start, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("utils::shift_right", l), l, |b, _l| {
            b.iter(|| unsafe { shift_right::<[usize; count]>(start, *l) })
        });

        group.bench_with_input(BenchmarkId::new("ptr_contrev_rotate", l), l, |b, _l| {
            b.iter(|| unsafe { ptr_contrev_rotate::<[usize; count]>(*l, start, 1) })
        });

        group.bench_with_input(BenchmarkId::new("ptr_reversal_rotate", l), l, |b, _l| {
            b.iter(|| unsafe { ptr_reversal_rotate::<[usize; count]>(*l, start, 1) })
        });

        group.bench_with_input(BenchmarkId::new("ptr_direct_rotate", l), l, |b, _l| {
            b.iter(|| unsafe { ptr_direct_rotate::<[usize; count]>(*l, start, 1) })
        });
    }

    group.finish();
}

/// cargo bench --bench=copies "Copy distance"
fn bench_copy_distance(c: &mut Criterion) {
    case_copy_distance::<1>(c, 1, &[0, 1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<1>(c, 2, &[0, 1, 2, 3, 4, 5, 20, 50, 100]);

    case_copy_distance::<2>(c, 1, &[0, 1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<2>(c, 2, &[0, 1, 2, 3, 4, 5, 20, 50, 100]);

    case_copy_distance::<4>(c, 1, &[0, 1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<4>(c, 2, &[0, 1, 2, 3, 4, 5, 20, 50, 100]);

    case_copy_distance::<10>(c, 1, &[0, 1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<10>(c, 2, &[0, 1, 2, 3, 4, 5, 20, 50, 100]);
}

/// cargo bench --bench=copies "Copy forward"
fn bench_copy_overlapping_forward(c: &mut Criterion) {
    case_copy_overlapping_forward::<1>(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_copy_overlapping_forward::<1>(c, 100_000, &[0, 25_000, 50_000, 75_000, 99_000, 100_000]);

    case_copy_overlapping_forward::<2>(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_copy_overlapping_forward::<2>(c, 100_000, &[0, 25_000, 50_000, 75_000, 99_000, 100_000]);

    case_copy_overlapping_forward::<10>(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_copy_overlapping_forward::<10>(c, 100_000, &[0, 25_000, 50_000, 75_000, 99_000, 100_000]);
}

/// cargo bench --bench=copies "Copy backward"
fn bench_copy_overlapping_backward(c: &mut Criterion) {
    case_copy_overlapping_backward::<1>(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_copy_overlapping_backward::<1>(c, 100_000, &[0, 1000, 25_000, 50_000, 75_000, 100_000]);

    case_copy_overlapping_backward::<2>(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_copy_overlapping_backward::<2>(c, 100_000, &[0, 1000, 25_000, 50_000, 75_000, 100_000]);

    case_copy_overlapping_backward::<10>(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_copy_overlapping_backward::<10>(c, 100_000, &[0, 1000, 25_000, 50_000, 75_000, 100_000]);
}

/// cargo bench --bench=copies "Shift left"
fn bench_shift_left(c: &mut Criterion) {
    case_shift_left::<1>(c, &[2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20]);
    case_shift_left::<1>(c, &[1000, 25_000, 50_000, 75_000, 100_000]);

    case_shift_left::<2>(c, &[2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20]);
    case_shift_left::<2>(c, &[1000, 25_000, 50_000, 75_000, 100_000]);

    case_shift_left::<10>(c, &[2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20]);
    case_shift_left::<10>(c, &[1000, 25_000, 50_000, 75_000, 100_000]);

    case_shift_left::<20>(c, &[2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20]);
    case_shift_left::<20>(c, &[1000, 25_000, 50_000, 75_000, 100_000]);
}

/// cargo bench --bench=copies "Shift right"
fn bench_shift_right(c: &mut Criterion) {
    case_shift_right::<1>(c, &[2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20]);
    case_shift_right::<1>(c, &[1000, 25_000, 50_000, 75_000, 100_000]);

    case_shift_right::<2>(c, &[2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20]);
    case_shift_right::<2>(c, &[1000, 25_000, 50_000, 75_000, 100_000]);

    case_shift_right::<10>(c, &[2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20]);
    case_shift_right::<10>(c, &[1000, 25_000, 50_000, 75_000, 100_000]);

    case_shift_right::<20>(c, &[2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20]);
    case_shift_right::<20>(c, &[1000, 25_000, 50_000, 75_000, 100_000]);
}

criterion_group! {
    name = benches;

//    config = Criterion::default().sample_size(500).measurement_time(Duration::new(120, 0));
    config = Criterion::default()
             .sample_size(500)
             .with_profiler(
                  PProfProfiler::new(100, Output::Flamegraph(None))
              );

    targets = bench_copy_distance, bench_copy_overlapping_backward, bench_copy_overlapping_forward, bench_shift_left, bench_shift_right
}

criterion_main!(benches);
