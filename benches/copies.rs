use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rotations::{ptr_reversal_rotate, utils::*};

// use std::time::Duration;
use std::ptr;

fn seq<const N: usize>(size: usize) -> Vec<[usize; N]> {
    let mut v = vec![[0; N]; size];
    for i in 0..size {
        v[i] = [i + 1; N];
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
fn case_copy<const N: usize>(c: &mut Criterion, len: usize, distances: &[isize]) {
    let mut group = c.benchmark_group(format!("Copy/{len}/{N}"));
    let mut v = seq::<N>(len * 2 + 1);
    let start = *&v[..].as_mut_ptr();
    let end = unsafe { start.add(len * 2 + 1) };

    for d in distances {
        if *d >= 0 {
            let d = d.unsigned_abs();

            group.bench_with_input(BenchmarkId::new("utils::copy", d), &d, |b, _d| {
                b.iter(|| forward_test(copy::<[usize; N]>, start, d, len))
            });

            group.bench_with_input(BenchmarkId::new("utils::block_copy", d), &d, |b, _| {
                b.iter(|| forward_test(block_copy::<[usize; N]>, start, d, len))
            });

            group.bench_with_input(BenchmarkId::new("utils::byte_copy", d), &d, |b, _| {
                b.iter(|| forward_test(byte_copy::<[usize; N]>, start, d, len))
            });

            group.bench_with_input(BenchmarkId::new("ptr::copy", d), &d, |b, _| {
                b.iter(|| forward_test(ptr::copy::<[usize; N]>, start, d, len))
            });
        } else {
            let d = d.unsigned_abs();

            group.bench_with_input(BenchmarkId::new("utils::copy", d), &d, |b, _d| {
                b.iter(|| backward_test(copy::<[usize; N]>, end, d, len))
            });

            group.bench_with_input(BenchmarkId::new("utils::block_copy", d), &d, |b, _| {
                b.iter(|| backward_test(block_copy::<[usize; N]>, end, d, len))
            });

            group.bench_with_input(BenchmarkId::new("utils::byte_copy", d), &d, |b, _| {
                b.iter(|| backward_test(byte_copy::<[usize; N]>, end, d, len))
            });

            group.bench_with_input(BenchmarkId::new("ptr::copy", d), &d, |b, _| {
                b.iter(|| backward_test(ptr::copy::<[usize; N]>, end, d, len))
            });
        }
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
fn case_copy_distance<const N: usize>(c: &mut Criterion, len: usize, distances: &[usize]) {
    let mut group = c.benchmark_group(format!("Copy distances/{len}/{N}"));
    let max_distance = distances.iter().max().unwrap();
    let mut v = seq(len + max_distance);

    for d in distances {
        let start = *&v[..].as_mut_ptr();
        let end = unsafe { start.add(len + max_distance) };

        group.bench_with_input(BenchmarkId::new("utils::copy", d), d, |b, _| {
            b.iter(|| backward_test(copy::<[usize; N]>, end, *d, len))
        });

        group.bench_with_input(BenchmarkId::new("utils::block_copy", d), d, |b, _| {
            b.iter(|| backward_test(block_copy::<[usize; N]>, end, *d, len))
        });

        group.bench_with_input(BenchmarkId::new("utils::byte_copy", d), d, |b, _| {
            b.iter(|| forward_test(byte_copy::<[usize; N]>, start, *d, len))
        });

        group.bench_with_input(
            BenchmarkId::new("ptr::copy_nonoverlapping", d),
            d,
            |b, _| b.iter(|| forward_test(ptr::copy_nonoverlapping::<[usize; N]>, start, *d, len)),
        );

        group.bench_with_input(BenchmarkId::new("ptr::copy", d), d, |b, _| {
            b.iter(|| forward_test(ptr::copy::<[usize; N]>, start, *d, len))
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
fn case_shift_left<const N: usize>(c: &mut Criterion, lens: &[usize]) {
    let max_len = *lens.iter().max().unwrap();
    let mut group = c.benchmark_group(format!("Shift left/{max_len}/{N}"));
    let mut v = seq(max_len + 1);

    let start = *&v[..].as_mut_ptr();
    let end = unsafe { start.add(max_len + 1) };

    for l in lens {
        group.bench_with_input(BenchmarkId::new("utils::copy", l), l, |b, _l| {
            b.iter(|| backward_test(copy::<[usize; N]>, end, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("ptr::copy", l), l, |b, _l| {
            b.iter(|| backward_test(ptr::copy::<[usize; N]>, end, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("utils::byte_copy", 1), l, |b, _| {
            b.iter(|| backward_test(byte_copy::<[usize; N]>, end, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("utils::shift_left", l), l, |b, _l| {
            b.iter(|| unsafe { shift_left::<[usize; N]>(start.add(1), *l) })
        });

        group.bench_with_input(BenchmarkId::new("ptr_reversal_rotate", l), l, |b, _l| {
            b.iter(|| unsafe { ptr_reversal_rotate::<[usize; N]>(1, start.add(1), *l) })
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
fn case_shift_right<const N: usize>(c: &mut Criterion, lens: &[usize]) {
    let max_len = *lens.iter().max().unwrap();
    let mut group = c.benchmark_group(format!("Shift right/{max_len}/{N}"));
    let mut v = seq(max_len + 1);

    let start = *&v[..].as_mut_ptr();

    for l in lens {
        group.bench_with_input(BenchmarkId::new("utils::copy", l), l, |b, _l| {
            b.iter(|| forward_test(copy::<[usize; N]>, start, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("ptr::copy", l), l, |b, _l| {
            b.iter(|| forward_test(ptr::copy::<[usize; N]>, start, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("utils::byte_copy", 1), l, |b, _| {
            b.iter(|| forward_test(byte_copy::<[usize; N]>, start, 1, *l))
        });

        group.bench_with_input(BenchmarkId::new("utils::shift_right", l), l, |b, _l| {
            b.iter(|| unsafe { shift_right::<[usize; N]>(start, *l) })
        });

        group.bench_with_input(BenchmarkId::new("ptr_reversal_rotate", l), l, |b, _l| {
            b.iter(|| unsafe { ptr_reversal_rotate::<[usize; N]>(*l, start.add(*l), 1) })
        });
    }

    group.finish();
}

/// cargo bench --bench=copies "Copy distance"
fn bench_copy_distance(c: &mut Criterion) {
    case_copy_distance::<1>(c, 1, &[1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<1>(c, 2, &[2, 3, 4, 5, 20, 50, 100]);
    case_copy_distance::<1>(c, 100, &[100, 150, 200, 300, 500]);
    case_copy_distance::<1>(c, 100_000, &[100_000, 150_000, 200_000, 300_000, 500_000]);

    case_copy_distance::<2>(c, 1, &[1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<2>(c, 2, &[2, 3, 4, 5, 20, 50, 100]);
    case_copy_distance::<2>(c, 100, &[100, 150, 200, 300, 500]);
    case_copy_distance::<2>(c, 100_000, &[100_000, 150_000, 200_000, 300_000, 500_000]);

    case_copy_distance::<4>(c, 1, &[1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<4>(c, 2, &[2, 3, 4, 5, 20, 50, 100]);
    case_copy_distance::<4>(c, 100, &[100, 150, 200, 300, 500]);
    case_copy_distance::<4>(c, 100_000, &[100_000, 150_000, 200_000, 300_000, 500_000]);
}

/// cargo bench --bench=copies "Copy"
fn bench_copy(c: &mut Criterion) {
    let distances_10: [isize; 21] = core::array::from_fn(|i| i as isize - 10);
    let distances_50: [isize; 101] = core::array::from_fn(|i| i as isize - 50);
    let distances_500: [isize; 101] = core::array::from_fn(|i| (i as isize - 50) * 10);
    let distances_100_000 = [
        -100_000, -99_000, -75_000, -50_000, -25_000, -5000, 0, 5000, 25_000, 50_000, 75_000,
        99_000, 100_000,
    ];
    let distances_200_000 = [
        -200_000, -190_000, -150_000, -100_000, -75_000, -50_000, -10_000, 0, 10_000, 50_000,
        75_000, 100_000, 150_000, 190_000, 200_000,
    ];

    case_copy::<1>(c, 10, &distances_10);
    case_copy::<1>(c, 50, &distances_50);
    case_copy::<1>(c, 500, &distances_500);
    case_copy::<1>(c, 100_000, &distances_100_000);
    case_copy::<1>(c, 200_000, &distances_200_000);

    case_copy::<2>(c, 10, &distances_10);
    case_copy::<2>(c, 50, &distances_50);
    case_copy::<2>(c, 500, &distances_500);
    case_copy::<2>(c, 100_000, &distances_100_000);
    case_copy::<2>(c, 200_000, &distances_200_000);

    case_copy::<10>(c, 10, &distances_10);
    case_copy::<10>(c, 50, &distances_50);
    case_copy::<10>(c, 500, &distances_500);
    case_copy::<10>(c, 100_000, &distances_100_000);
    case_copy::<10>(c, 200_000, &distances_200_000);
}

/// cargo bench --bench=copies "Shift left"
fn bench_shift_left(c: &mut Criterion) {
    let lens: [usize; 50] = core::array::from_fn(|i| i + 1);
    let lens_100_000 = [1000, 25_000, 50_000, 75_000, 100_000];

    case_shift_left::<1>(c, &lens);
    case_shift_left::<1>(c, &lens_100_000);

    case_shift_left::<2>(c, &lens);
    case_shift_left::<2>(c, &lens_100_000);

    case_shift_left::<10>(c, &lens);
    case_shift_left::<10>(c, &lens_100_000);

    case_shift_left::<15>(c, &lens);
    case_shift_left::<15>(c, &lens_100_000);

    case_shift_left::<20>(c, &lens);
    case_shift_left::<20>(c, &lens_100_000);

    case_shift_left::<40>(c, &lens);
    case_shift_left::<40>(c, &lens_100_000);

    case_shift_left::<80>(c, &lens);
    case_shift_left::<80>(c, &lens_100_000);
}

/// cargo bench --bench=copies "Shift right"
fn bench_shift_right(c: &mut Criterion) {
    let lens: [usize; 50] = core::array::from_fn(|i| i + 1);
    let lens_100_000 = [1000, 25_000, 50_000, 75_000, 100_000];

    case_shift_right::<1>(c, &lens);
    case_shift_right::<1>(c, &lens_100_000);

    case_shift_right::<2>(c, &lens);
    case_shift_right::<2>(c, &lens_100_000);

    case_shift_right::<10>(c, &lens);
    case_shift_right::<10>(c, &lens_100_000);

    case_shift_right::<15>(c, &lens);
    case_shift_right::<15>(c, &lens_100_000);

    case_shift_right::<20>(c, &lens);
    case_shift_right::<20>(c, &lens_100_000);

    case_shift_right::<40>(c, &lens);
    case_shift_right::<40>(c, &lens_100_000);

    case_shift_right::<80>(c, &lens);
    case_shift_right::<80>(c, &lens_100_000);
}

criterion_group! {
    name = benches;

    config = Criterion::default();

    targets = bench_copy, bench_copy_distance, bench_shift_left, bench_shift_right
}

criterion_main!(benches);
