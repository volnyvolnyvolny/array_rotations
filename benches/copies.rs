use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rotations::{ptr_reversal_rotate, utils::*};

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
fn case_copy<const count: usize>(c: &mut Criterion, len: usize, distances: &[usize]) {
    let mut group = c.benchmark_group(format!("Copy/{len}/{count}"));
    let mut v = seq::<count>(len * 2 + 1);

    for d in distances {
        let start = *&v[..].as_mut_ptr();

        group.bench_with_input(BenchmarkId::new("utils::copy", d), d, |b, _| {
            b.iter(|| forward_test(copy::<[usize; count]>, start, *d, len))
        });

        group.bench_with_input(BenchmarkId::new("utils::block_copy", d), d, |b, _| {
            b.iter(|| forward_test(block_copy::<[usize; count]>, start, *d, len))
        });

        group.bench_with_input(BenchmarkId::new("ptr::copy", d), d, |b, _d| {
            b.iter(|| forward_test(ptr::copy::<[usize; count]>, start, *d, len))
        });
    }

    group.finish();
}

/// ```text
///   start, dist = 3
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

        group.bench_with_input(BenchmarkId::new("utils::copy", d), d, |b, _| {
            b.iter(|| backward_test(copy::<[usize; count]>, end, *d, len))
        });

        group.bench_with_input(BenchmarkId::new("utils::block_copy", d), d, |b, _| {
            b.iter(|| backward_test(block_copy::<[usize; count]>, end, *d, len))
        });

        group.bench_with_input(
            BenchmarkId::new("ptr::copy_nonoverlapping", d),
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
        group.bench_with_input(BenchmarkId::new("utils::copy", l), l, |b, _l| {
            b.iter(|| backward_test(copy::<[usize; count]>, end, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("ptr::copy", l), l, |b, _l| {
            b.iter(|| backward_test(ptr::copy::<[usize; count]>, end, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("utils::shift_left", l), l, |b, _l| {
            b.iter(|| unsafe { shift_left::<[usize; count]>(start.add(1), *l) })
        });

        group.bench_with_input(BenchmarkId::new("ptr_reversal_rotate", l), l, |b, _l| {
            b.iter(|| unsafe { ptr_reversal_rotate::<[usize; count]>(1, start.add(1), *l) })
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
        group.bench_with_input(BenchmarkId::new("utils::copy", l), l, |b, _l| {
            b.iter(|| forward_test(copy::<[usize; count]>, start, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("ptr::copy", l), l, |b, _l| {
            b.iter(|| forward_test(ptr::copy::<[usize; count]>, start, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("utils::shift_right", l), l, |b, _l| {
            b.iter(|| unsafe { shift_right::<[usize; count]>(start, *l) })
        });

        group.bench_with_input(BenchmarkId::new("ptr_reversal_rotate", l), l, |b, _l| {
            b.iter(|| unsafe { ptr_reversal_rotate::<[usize; count]>(*l, start.add(*l), 1) })
        });
    }

    group.finish();
}

/// cargo bench --bench=copies "Copy distance"
fn bench_copy_distance(c: &mut Criterion) {
    case_copy_distance::<1>(c, 1, &[1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<1>(c, 2, &[2, 3, 4, 5, 20, 50, 100]);
    case_copy_distance::<1>(c, 100, &[100, 150, 200, 300, 500]);

    case_copy_distance::<2>(c, 1, &[1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<2>(c, 2, &[2, 3, 4, 5, 20, 50, 100]);
    case_copy_distance::<2>(c, 100, &[100, 150, 200, 300, 500]);

    case_copy_distance::<4>(c, 1, &[1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<4>(c, 2, &[2, 3, 4, 5, 20, 50, 100]);
    case_copy_distance::<4>(c, 100, &[100, 150, 200, 300, 500]);
}

/// cargo bench --bench=copies "Copy"
fn bench_copy(c: &mut Criterion) {
    case_copy::<1>(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_copy::<1>(c, 100_000, &[0, 25_000, 50_000, 75_000, 99_000, 100_000]);

    case_copy::<2>(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_copy::<2>(c, 100_000, &[0, 25_000, 50_000, 75_000, 99_000, 100_000]);

    case_copy::<10>(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_copy::<10>(c, 100_000, &[0, 25_000, 50_000, 75_000, 99_000, 100_000]);
}

/// cargo bench --bench=copies "Shift left"
fn bench_shift_left(c: &mut Criterion) {
    let arr: [usize; 50] = core::array::from_fn(|i| i + 1);
    let arr2 = [1000, 25_000, 50_000, 75_000, 100_000];

    case_shift_left::<1>(c, &arr);
    case_shift_left::<1>(c, &arr2);

    case_shift_left::<2>(c, &arr);
    case_shift_left::<2>(c, &arr2);

    case_shift_left::<10>(c, &arr);
    case_shift_left::<10>(c, &arr2);

    case_shift_left::<20>(c, &arr);
    case_shift_left::<20>(c, &arr2);

    case_shift_left::<40>(c, &arr);
    case_shift_left::<40>(c, &arr2);

    case_shift_left::<60>(c, &arr);
    case_shift_left::<60>(c, &arr2);

    case_shift_left::<80>(c, &arr);
    case_shift_left::<80>(c, &arr2);
}

/// cargo bench --bench=copies "Shift right"
fn bench_shift_right(c: &mut Criterion) {
    let arr: [usize; 50] = core::array::from_fn(|i| i + 1);
    let arr2 = [1000, 25_000, 50_000, 75_000, 100_000];

    case_shift_right::<1>(c, &arr);
    case_shift_right::<1>(c, &arr2);

    case_shift_right::<2>(c, &arr);
    case_shift_right::<2>(c, &arr2);

    case_shift_right::<10>(c, &arr);
    case_shift_right::<10>(c, &arr2);

    case_shift_right::<20>(c, &arr);
    case_shift_right::<20>(c, &arr2);

    case_shift_right::<40>(c, &arr);
    case_shift_right::<40>(c, &arr2);

    case_shift_right::<60>(c, &arr);
    case_shift_right::<60>(c, &arr2);

    case_shift_right::<80>(c, &arr);
    case_shift_right::<80>(c, &arr2);
}

criterion_group! {
    name = benches;

    config = Criterion::default();

    targets = bench_copy, bench_copy_distance, bench_shift_left, bench_shift_right
}

criterion_main!(benches);
