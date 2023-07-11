/*
Copyright (C) 2023 Valentin Vasilev (3volny@gmail.com).
*/

/*
Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SO↓FTWARE.
*/

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion,
};
use rust_rotations::{ptr_reversal_rotate, utils::*};

use std::collections::HashMap;
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
    ReversalRotate,
}

use Fun::*;

fn run_fun<const N: usize>(
    group: &mut BenchmarkGroup<WallTime>,
    param: isize,
    len: usize,
    distance: isize,
    arr: *mut [usize; N],
    fun: &Fun,
) {
    match fun {
        Copy => {
            group.bench_with_input(BenchmarkId::new("utils::copy", param), &param, |b, _| {
                b.iter(|| unsafe {
                    copy::<[usize; N]>(arr, arr.offset(distance), len);
                })
            });
        }
        BlockCopy => {
            group.bench_with_input(
                BenchmarkId::new("utils::block_copy", param),
                &param,
                |b, _| {
                    b.iter(|| unsafe { block_copy::<[usize; N]>(arr, arr.offset(distance), len) })
                },
            );
        }
        ByteCopy => {
            group.bench_with_input(
                BenchmarkId::new("utils::byte_copy", param),
                &param,
                |b, _| {
                    b.iter(|| unsafe { byte_copy::<[usize; N]>(arr, arr.offset(distance), len) })
                },
            );
        }
        PtrCopy => {
            group.bench_with_input(BenchmarkId::new("ptr::copy", param), &param, |b, _| {
                b.iter(|| unsafe { ptr::copy::<[usize; N]>(arr, arr.offset(distance), len) })
            });
        }
        PtrCopyNonoverlapping => {
            group.bench_with_input(
                BenchmarkId::new("ptr::copy_nonoverlapping", param),
                &param,
                |b, _| {
                    b.iter(|| unsafe {
                        ptr::copy_nonoverlapping::<[usize; N]>(arr, arr.offset(distance), len)
                    })
                },
            );
        }
        ReversalRotate => {
            if distance < 0 {
                group.bench_with_input(
                    BenchmarkId::new("ptr_reversal_rotate", len),
                    &param,
                    |b, _l| {
                        b.iter(|| unsafe { ptr_reversal_rotate::<[usize; N]>(1, arr.add(1), len) })
                    },
                );
            } else {
                group.bench_with_input(
                    BenchmarkId::new("ptr_reversal_rotate", len),
                    &param,
                    |b, _l| b.iter(|| unsafe { ptr_reversal_rotate::<[usize; N]>(len, arr, 1) }),
                );
            }
        }
    }
}
/// ```text
///   start, dist = 2
///   |               len = 4
/// [ 1  2  3  4  5  6  7  8  9]
///   [://////:]
///
/// [ 1  2  1  2  3  4  7  8  9]
///         [://////:]
/// ```
fn case_copy_overlapping<const N: usize>(c: &mut Criterion, len: usize, distances: &[isize]) {
    case_copy::<N>(
        "Copy",
        c,
        len,
        distances,
        vec![Copy, BlockCopy, ByteCopy, PtrCopy],
    );
}

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
///   distance = -12                    count = 3 |
/// [ 1  2  3  4  5  6  7  8  9 10 11 12 13 14 15]
///   [:\\\:]                    <------  [:///:]
///   dst                                 src
///
/// [ 1  2 15  4  .  .  .  .  .  .  .  .  .  . 15]
/// [ 1 14  .  4  .  .  .  .  .  .  .  .  . 14 15]
/// [13 ~~ 15  4  .  .  .  .  .  .  .  . 13  . 15]
/// ```
fn case_copy<const N: usize>(
    name: &str,
    c: &mut Criterion,
    len: usize,
    distances: &[isize],
    funs: Vec<Fun>,
) {
    let mut g = c.benchmark_group(format!("{name}/{len}/{N}"));

    let max_distance = distances.iter().map(|d| d.unsigned_abs()).max().unwrap();
    let mut v = seq::<N>(len + max_distance);
    let start = *&v[..].as_mut_ptr();

    for d in distances {
        for fun in &funs {
            let s = if d < &0 {
                unsafe { start.add(d.unsigned_abs()) }
            } else {
                start
            };

            run_fun::<N>(&mut g, *d, len, *d, s, fun);
        }
    }

    g.finish();
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
    let funs = vec![Copy, BlockCopy, ByteCopy, ReversalRotate, PtrCopy];

    let max_len = *lens.iter().max().unwrap();
    let mut g = c.benchmark_group(format!("Shift left/{max_len}/{N}"));
    let mut v = seq::<N>(max_len + 2);
    let start = *&v[..].as_mut_ptr();

    for len in lens {
        for fun in &funs {
            let s = unsafe { start.add(1) };

            run_fun::<N>(&mut g, *len as isize, *len, 1, s, fun);
        }
    }

    g.finish();
}

/// Shift right
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
    let funs = vec![Copy, BlockCopy, ByteCopy, ReversalRotate, PtrCopy];

    let max_len = *lens.iter().max().unwrap();
    let mut g = c.benchmark_group(format!("Shift right/{max_len}/{N}"));
    let mut v = seq::<N>(max_len + 1);
    let start = *&v[..].as_mut_ptr();

    for len in lens {
        for fun in &funs {
            run_fun::<N>(&mut g, *len as isize, *len, 1, start, fun);
        }
    }

    g.finish();
}

/// ```text
///   start, dist = 5
///   |               len = 4
/// [ 1  2  3  4  5  6  7  8  9]
///   [://////:]
///
/// [ 1  2  3  4  5  1  2  3  4]
///                  [://////:]
/// ```
fn case_copy_nonoverlapping<const N: usize>(
    c: &mut Criterion,
    len: usize,
    distances: &HashMap<usize, [isize; 7]>,
) {
    case_copy::<N>(
        "Copy nonoverlapping",
        c,
        len,
        distances.get(&len).unwrap(),
        vec![Copy, ByteCopy, PtrCopyNonoverlapping, PtrCopy],
    );
}

/// cargo bench --bench=copies "Copy nonoverlapping"
fn bench_copy_nonoverlapping(c: &mut Criterion) {
    let lens = [1, 2, 50, 100, 100_000];

    let distances = HashMap::from([
        (1, [1, 2, 3, 5, 20, 50, 100]),
        (2, [2, 3, 4, 5, 20, 50, 100]),
        (50, [50, 75, 100, 150, 250, 300, 500]),
        (100, [100, 150, 200, 300, 500, 1000, 1500]),
        (
            100_000,
            [
                100_000, 150_000, 200_000, 300_000, 500_000, 1000_000, 1_500_000,
            ],
        ),
    ]);

    for l in lens {
        case_copy_nonoverlapping::<1>(c, l, &distances);
    }

    for l in lens {
        case_copy_nonoverlapping::<2>(c, l, &distances);
    }

    for l in lens {
        case_copy_nonoverlapping::<4>(c, l, &distances);
    }

    for l in lens {
        case_copy_nonoverlapping::<10>(c, l, &distances);
    }
}

/// ```text
///   start, dist = 2
///   |               len = 4
/// [ 1  2  3  4  5  6  7  8  9]
///   [://////:]
///
/// [ 1  2  1  2  3  4  7  8  9]
///         [://////:]
/// ```
fn case_copy_nonoverlapping_by_len<const N: usize>(c: &mut Criterion, lens: &[usize]) {
    let funs = vec![Copy, ByteCopy, PtrCopyNonoverlapping, PtrCopy];

    let max_len = *lens.iter().max().unwrap();
    let mut g = c.benchmark_group(format!("Copy nonoverlapping by len/{max_len}/{N}"));

    let mut v = seq::<N>(2 * max_len + 1000);
    let start = *&v[..].as_mut_ptr();

    for len in lens {
        let d = (len + 1000) as isize;

        for fun in &funs {
            run_fun::<N>(&mut g, *len as isize, *len, d, start, fun);
        }
    }

    g.finish();
}

/// cargo bench --bench=copies "Copy nonoverlapping"
fn bench_copy_nonoverlapping_by_len(c: &mut Criterion) {
    let lens_500 = &[1, 2, 4, 25, 50, 100, 200, 500];
    let lens_10_000 = &[500, 1000, 2000, 4000, 6000, 8000, 10_000];
    let lens_100_000 = &[10_000, 20_000, 40_000, 60_000, 80_000, 100_000];

    case_copy_nonoverlapping_by_len::<1>(c, lens_500);
    case_copy_nonoverlapping_by_len::<1>(c, lens_10_000);
    case_copy_nonoverlapping_by_len::<1>(c, lens_100_000);

    case_copy_nonoverlapping_by_len::<2>(c, lens_500);
    case_copy_nonoverlapping_by_len::<2>(c, lens_10_000);
    case_copy_nonoverlapping_by_len::<2>(c, lens_100_000);

    case_copy_nonoverlapping_by_len::<4>(c, lens_500);
    case_copy_nonoverlapping_by_len::<4>(c, lens_10_000);
    case_copy_nonoverlapping_by_len::<4>(c, lens_100_000);

    case_copy_nonoverlapping_by_len::<10>(c, lens_500);
    case_copy_nonoverlapping_by_len::<10>(c, lens_10_000);
    case_copy_nonoverlapping_by_len::<10>(c, lens_100_000);

    case_copy_nonoverlapping_by_len::<20>(c, lens_500);
    case_copy_nonoverlapping_by_len::<20>(c, lens_10_000);
    case_copy_nonoverlapping_by_len::<20>(c, lens_100_000);
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

    case_copy_overlapping::<1>(c, 10, &distances_10);
    case_copy_overlapping::<1>(c, 40, &distances_40);
    case_copy_overlapping::<1>(c, 500, &distances_500);
    case_copy_overlapping::<1>(c, 100_000, &distances_100_000);
    case_copy_overlapping::<1>(c, 200_000, &distances_200_000);

    case_copy_overlapping::<2>(c, 10, &distances_10);
    case_copy_overlapping::<2>(c, 40, &distances_40);
    case_copy_overlapping::<2>(c, 500, &distances_500);
    case_copy_overlapping::<2>(c, 100_000, &distances_100_000);
    case_copy_overlapping::<2>(c, 200_000, &distances_200_000);

    case_copy_overlapping::<10>(c, 10, &distances_10);
    case_copy_overlapping::<10>(c, 40, &distances_40);
    case_copy_overlapping::<10>(c, 500, &distances_500);
    case_copy_overlapping::<10>(c, 100_000, &distances_100_000);
    case_copy_overlapping::<10>(c, 200_000, &distances_200_000);
}

/// cargo bench --bench=copies "Shift left"
fn bench_shift_left(c: &mut Criterion) {
    let lens_100: [usize; 100] = core::array::from_fn(|i| i + 1);
    let lens_1000: [usize; 100] = core::array::from_fn(|i| i * 10 + 1);
    let lens_10_000: [usize; 100] = core::array::from_fn(|i| i * 100 + 1);
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

    case_shift_left::<160>(c, &lens_100);
    case_shift_left::<160>(c, &lens_1000);
    case_shift_left::<160>(c, &lens_10_000);
    case_shift_left::<160>(c, &lens_100_000);
}

/// cargo bench --bench=copies "Shift right"
fn bench_shift_right(c: &mut Criterion) {
    let lens_100: [usize; 100] = core::array::from_fn(|i| i + 1);
    let lens_1000: [usize; 101] =
        core::array::from_fn(|i| if i == 101 { 1000 } else { i * 10 + 1 });
    let lens_10_000: [usize; 101] =
        core::array::from_fn(|i| if i == 101 { 10_000 } else { i * 100 + 1 });
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

    targets = bench_copy, bench_copy_nonoverlapping, bench_copy_nonoverlapping_by_len, bench_shift_left, bench_shift_right
}

criterion_main!(benches);
