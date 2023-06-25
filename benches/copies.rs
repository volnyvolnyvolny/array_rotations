use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rotations::{ptr_reversal_rotate, utils::*};

use std::ptr;

fn seq<const N: usize>(size: usize) -> Vec<[usize; N]> {
    let mut v = vec![[0; N]; size];
    for i in 0..size {
        v[i] = [i + 1; N];
    }
    v
}

enum Fun {
    PtrCopy,
    PtrCopyNonoverlapping,
    Copy,
    ByteCopy,
    BlockCopy,
    ShiftLeft,
    ShiftRight,
    ReversalRotate,
}

use Fun::*;

/// ```text
///  start
///  | distance = +12                  count = 3
/// [1  2  3  4  5  6  7  8  9 10 11 12 13 14 15]
///  [:///:] ------>                     [:\\\:]
///  src                                 dst
///
/// [1  .  .  .  .  .  .  .  .  .  . 12  1 14 15]
/// [1  2  .  .  .  .  .  .  .  .  . 12  1  2 15]
/// [1  .  3  .  .  .  .  .  .  .  . 12  1 ~~~ 3]
/// ```
///
/// ```text
///                                              end
///   distance = -12                    count = 3 |
/// [ 1  2  3  4  5  6  7  8  9 10 11 12 13 14 15]
///   [:\\\:]                    <------  [:///:]
///   dst                                 src
///
/// [ 1  2 15  4  .  .  .  .  .  .  .  .  .  . 15]
/// [ 1 14  .  4  .  .  .  .  .  .  .  .  . 14 15]
/// [13 ~~ 15  4  .  .  .  .  .  .  .  . 13  . 15]
/// ```
fn case<const N: usize>(
    name: &str,
    c: &mut Criterion,
    lens: &[usize],
    distances: &[isize],
    funs: Vec<Fun>,
) {
    let max_len = *lens.iter().max().unwrap();
    let max_distance = distances.iter().map(|d| d.unsigned_abs()).max().unwrap();
    let mut group = c.benchmark_group(format!("{name}/{max_len}/{N}"));
    let mut v = seq::<N>(max_len + max_distance);
    let start = *&v[..].as_mut_ptr();

    for len in lens {
        for d in distances {
            for fun in &funs {
                let l = len.clone() as isize;
                let p = if lens.len() == 1 { d } else { &l };

                let s = if d < &0 {
                    unsafe { start.add(d.unsigned_abs()) }
                } else {
                    start
                };

                match fun {
                    Copy => {
                        group.bench_with_input(BenchmarkId::new("utils::copy", p), &p, |b, _| {
                            b.iter(|| unsafe {
                                copy::<[usize; N]>(s, s.offset(*d), *len);
                            })
                        });
                    }
                    BlockCopy => {
                        group.bench_with_input(
                            BenchmarkId::new("utils::block_copy", p),
                            &p,
                            |b, _| {
                                b.iter(|| unsafe {
                                    block_copy::<[usize; N]>(s, s.offset(*d), *len)
                                })
                            },
                        );
                    }
                    ByteCopy => {
                        group.bench_with_input(
                            BenchmarkId::new("utils::byte_copy", p),
                            &p,
                            |b, _| {
                                b.iter(|| unsafe { byte_copy::<[usize; N]>(s, s.offset(*d), *len) })
                            },
                        );
                    }
                    PtrCopy => {
                        group.bench_with_input(BenchmarkId::new("ptr::copy", p), &p, |b, _| {
                            b.iter(|| unsafe { ptr::copy::<[usize; N]>(s, s.offset(*d), *len) })
                        });
                    }
                    PtrCopyNonoverlapping => {
                        group.bench_with_input(
                            BenchmarkId::new("ptr::copy_nonoverlapping", p),
                            p,
                            |b, _| {
                                b.iter(|| unsafe {
                                    ptr::copy_nonoverlapping::<[usize; N]>(s, s.offset(*d), *len)
                                })
                            },
                        );
                    }
                    ShiftLeft => {
                        group.bench_with_input(
                            BenchmarkId::new("utils::shift_left", len),
                            len,
                            |b, _l| b.iter(|| unsafe { shift_left::<[usize; N]>(s.add(1), *len) }),
                        );
                    }
                    ShiftRight => {
                        group.bench_with_input(
                            BenchmarkId::new("utils::shift_right", len),
                            len,
                            |b, _l| b.iter(|| unsafe { shift_right::<[usize; N]>(s, *len) }),
                        );
                    }
                    ReversalRotate => {
                        if *d < 0 {
                            group.bench_with_input(
                                BenchmarkId::new("ptr_reversal_rotate", len),
                                len,
                                |b, _l| {
                                    b.iter(|| unsafe {
                                        ptr_reversal_rotate::<[usize; N]>(1, s.add(1), *len)
                                    })
                                },
                            );
                        } else {
                            group.bench_with_input(
                                BenchmarkId::new("ptr_reversal_rotate", len),
                                len,
                                |b, _l| {
                                    b.iter(|| unsafe {
                                        ptr_reversal_rotate::<[usize; N]>(*len, s, 1)
                                    })
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    group.finish();
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
    case::<N>(
        "Copy",
        c,
        &[len],
        distances,
        vec![Copy, BlockCopy, ByteCopy, PtrCopy],
    );
}

/// ```text
///   start, dist = 3
///   |               len = 4
/// [ 1  2  3  4  5  6  7  8  9]
///   [://////:]
///                       d = 3
/// [ 1  2  1  2  3  4  7  8  9]
///            [://////:]
/// ```
fn case_copy_distance<const N: usize>(c: &mut Criterion, len: usize, distances: &[isize]) {
    case::<N>(
        "Copy distances",
        c,
        &[len],
        distances,
        vec![Copy, BlockCopy, ByteCopy, PtrCopyNonoverlapping, PtrCopy],
    );
}

/// Shift left
///
/// Example:
///
/// ```text
///               start
///               |   len = 4
/// [ 1  2  3  4  5  6  7  8  9]
///                  [:\\\\\\:]
///
/// [ 1  2  3  4  6  7  8  9  9]
///               [://////:]
/// ```
fn case_shift_left<const N: usize>(c: &mut Criterion, lens: &[usize]) {
    case::<N>(
        "Shift left",
        c,
        lens,
        &[-1],
        vec![
            Copy,
            BlockCopy,
            ByteCopy,
            ShiftLeft,
            ReversalRotate,
            PtrCopy,
        ],
    );
}

/// Shift left
///
/// Example:
///
/// ```text
///   start
///   | len = 4
/// [ 1  2  3  4  5  6  7  8  9]
///   [:\\\\\\:]
///
/// [ 1  1  2  3  4  7  8  9  9]
///      [://////:]
/// ```
fn case_shift_right<const N: usize>(c: &mut Criterion, lens: &[usize]) {
    case::<N>(
        "Shift right",
        c,
        lens,
        &[1],
        vec![
            Copy,
            BlockCopy,
            ByteCopy,
            ShiftRight,
            ReversalRotate,
            PtrCopy,
        ],
    );
}

/// cargo bench --bench=copies "Copy distance"
fn bench_copy_distance(c: &mut Criterion) {
    case_copy_distance::<1>(c, 1, &[1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<1>(c, 2, &[2, 3, 4, 5, 20, 50, 100]);
    case_copy_distance::<1>(c, 50, &[50, 75, 100, 150, 250]);
    case_copy_distance::<1>(c, 100, &[100, 150, 200, 300, 500]);
    case_copy_distance::<1>(c, 100_000, &[100_000, 150_000, 200_000, 300_000, 500_000]);

    case_copy_distance::<2>(c, 1, &[1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<2>(c, 2, &[2, 3, 4, 5, 20, 50, 100]);
    case_copy_distance::<2>(c, 50, &[50, 75, 100, 150, 250]);
    case_copy_distance::<2>(c, 100, &[100, 150, 200, 300, 500]);
    case_copy_distance::<2>(c, 100_000, &[100_000, 150_000, 200_000, 300_000, 500_000]);

    case_copy_distance::<4>(c, 1, &[1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<4>(c, 2, &[2, 3, 4, 5, 20, 50, 100]);
    case_copy_distance::<4>(c, 50, &[50, 75, 100, 150, 250]);
    case_copy_distance::<4>(c, 100, &[100, 150, 200, 300, 500]);
    case_copy_distance::<4>(c, 100_000, &[100_000, 150_000, 200_000, 300_000, 500_000]);

    case_copy_distance::<10>(c, 1, &[1, 2, 3, 5, 20, 50, 100]);
    case_copy_distance::<10>(c, 2, &[2, 3, 4, 5, 20, 50, 100]);
    case_copy_distance::<10>(c, 50, &[50, 75, 100, 150, 250]);
    case_copy_distance::<10>(c, 100, &[100, 150, 200, 300, 500]);
    case_copy_distance::<10>(c, 100_000, &[100_000, 150_000, 200_000, 300_000, 500_000]);
}

/// cargo bench --bench=copies "Copy"
fn bench_copy(c: &mut Criterion) {
    let distances_10: [isize; 21] = core::array::from_fn(|i| i as isize - 10);
    let distances_40: [isize; 81] = core::array::from_fn(|i| i as isize - 40);
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
    case_copy::<1>(c, 40, &distances_40);
    case_copy::<1>(c, 500, &distances_500);
    case_copy::<1>(c, 100_000, &distances_100_000);
    case_copy::<1>(c, 200_000, &distances_200_000);

    case_copy::<2>(c, 10, &distances_10);
    case_copy::<2>(c, 40, &distances_40);
    case_copy::<2>(c, 500, &distances_500);
    case_copy::<2>(c, 100_000, &distances_100_000);
    case_copy::<2>(c, 200_000, &distances_200_000);

    case_copy::<10>(c, 10, &distances_10);
    case_copy::<10>(c, 40, &distances_40);
    case_copy::<10>(c, 500, &distances_500);
    case_copy::<10>(c, 100_000, &distances_100_000);
    case_copy::<10>(c, 200_000, &distances_200_000);
}

/// cargo bench --bench=copies "Shift left"
fn bench_shift_left(c: &mut Criterion) {
    let lens_100: [usize; 100] = core::array::from_fn(|i| i + 1);
    let lens_1000: [usize; 101] = core::array::from_fn(|i| i * 10 + 1);
    let lens_10_000: [usize; 101] = core::array::from_fn(|i| i * 100 + 1);
    let lens_100_000 = [1000, 25_000, 50_000, 75_000, 100_000];

    case_shift_left::<1>(c, &lens_100);
    case_shift_left::<1>(c, &lens_1000);
    case_shift_left::<1>(c, &lens_10_000);
    case_shift_left::<1>(c, &lens_100_000);

    case_shift_left::<2>(c, &lens_100);
    case_shift_left::<2>(c, &lens_1000);
    case_shift_left::<2>(c, &lens_10_000);
    case_shift_left::<2>(c, &lens_100_000);

    case_shift_left::<10>(c, &lens_100);
    case_shift_left::<10>(c, &lens_1000);
    case_shift_left::<10>(c, &lens_10_000);
    case_shift_left::<10>(c, &lens_100_000);

    case_shift_left::<15>(c, &lens_100);
    case_shift_left::<15>(c, &lens_1000);
    case_shift_left::<15>(c, &lens_10_000);
    case_shift_left::<15>(c, &lens_100_000);

    case_shift_left::<20>(c, &lens_100);
    case_shift_left::<20>(c, &lens_1000);
    case_shift_left::<20>(c, &lens_10_000);
    case_shift_left::<20>(c, &lens_100_000);

    case_shift_left::<40>(c, &lens_100);
    case_shift_left::<40>(c, &lens_1000);
    case_shift_left::<40>(c, &lens_10_000);
    case_shift_left::<40>(c, &lens_100_000);

    case_shift_left::<80>(c, &lens_100);
    case_shift_left::<80>(c, &lens_1000);
    case_shift_left::<80>(c, &lens_10_000);
    case_shift_left::<80>(c, &lens_100_000);
}

/// cargo bench --bench=copies "Shift right"
fn bench_shift_right(c: &mut Criterion) {
    let lens_100: [usize; 100] = core::array::from_fn(|i| i + 1);
    let lens_1000: [usize; 101] = core::array::from_fn(|i| i * 10 + 1);
    let lens_10_000: [usize; 101] = core::array::from_fn(|i| i * 100 + 1);
    let lens_100_000 = [1000, 25_000, 50_000, 75_000, 100_000];

    case_shift_right::<1>(c, &lens_100);
    case_shift_right::<1>(c, &lens_1000);
    case_shift_right::<1>(c, &lens_10_000);
    case_shift_right::<1>(c, &lens_100_000);

    case_shift_right::<2>(c, &lens_100);
    case_shift_right::<2>(c, &lens_1000);
    case_shift_right::<2>(c, &lens_10_000);
    case_shift_right::<2>(c, &lens_100_000);

    case_shift_right::<10>(c, &lens_100);
    case_shift_right::<10>(c, &lens_1000);
    case_shift_right::<10>(c, &lens_10_000);
    case_shift_right::<10>(c, &lens_100_000);

    case_shift_right::<15>(c, &lens_100);
    case_shift_right::<15>(c, &lens_1000);
    case_shift_right::<15>(c, &lens_10_000);
    case_shift_right::<15>(c, &lens_100_000);

    case_shift_right::<20>(c, &lens_100);
    case_shift_right::<20>(c, &lens_1000);
    case_shift_right::<20>(c, &lens_10_000);
    case_shift_right::<20>(c, &lens_100_000);

    case_shift_right::<40>(c, &lens_100);
    case_shift_right::<40>(c, &lens_1000);
    case_shift_right::<40>(c, &lens_10_000);
    case_shift_right::<40>(c, &lens_100_000);

    case_shift_right::<80>(c, &lens_100);
    case_shift_right::<80>(c, &lens_1000);
    case_shift_right::<80>(c, &lens_10_000);
    case_shift_right::<80>(c, &lens_100_000);
}

criterion_group! {
    name = benches;

    config = Criterion::default();

    targets = bench_copy, bench_copy_distance, bench_shift_left, bench_shift_right
}

criterion_main!(benches);
