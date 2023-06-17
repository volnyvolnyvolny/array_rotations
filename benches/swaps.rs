#![feature(slice_swap_unchecked)]

use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
// use pprof::criterion::{Output, PProfProfiler};

use rust_rotations::utils::*;

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
fn case_swap_forward<const count: usize>(c: &mut Criterion, len: usize) {
    let mut group = c.benchmark_group(format!("Swap forward/{len}/{count}"));
    let mut v = seq::<count>(len * 10 + 1);

    for mut d in 1..10 {
        d = d * len;

        let start = *&v[..].as_mut_ptr();

        group.bench_with_input(BenchmarkId::new("utils::swap_forward", d), &d, |b, _| {
            b.iter(|| forward_test(swap_forward::<[usize; count]>, start, d, len))
        });

        group.bench_with_input(BenchmarkId::new("utils::swap_backward", d), &d, |b, _| {
            b.iter(|| forward_test(swap_backward::<[usize; count]>, start, d, len))
        });

        group.bench_with_input(
            BenchmarkId::new("ptr::swap_nonoverlapping", d),
            &d,
            |b, _d| {
                b.iter(|| forward_test(ptr::swap_nonoverlapping::<[usize; count]>, start, d, len))
            },
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
fn case_swap_backward<const count: usize>(c: &mut Criterion, len: usize) {
    let mut group = c.benchmark_group(format!("Swap backward/{len}/{count}"));
    let mut v = seq::<count>(10 * len + 1);

    for mut d in 1..10 {
        d = d * len;

        let end = *unsafe { &v[..].as_mut_ptr().add(10 * len + 1) };

        group.bench_with_input(BenchmarkId::new("utils::swap_forward", d), &d, |b, _d| {
            b.iter(|| backward_test(swap_forward::<[usize; count]>, end, d, len))
        });

        group.bench_with_input(BenchmarkId::new("utils::swap_backward", d), &d, |b, _d| {
            b.iter(|| backward_test(swap_backward::<[usize; count]>, end, d, len))
        });

        group.bench_with_input(
            BenchmarkId::new("ptr::swap_nonoverlapping", d),
            &d,
            |b, _d| {
                b.iter(|| backward_test(ptr::swap_nonoverlapping::<[usize; count]>, end, d, len))
            },
        );
    }

    group.finish();
}

fn case_swap<const count: usize>(group: &mut BenchmarkGroup<WallTime>) {
    let mut v = seq::<1>(3);

    let start = v.as_mut_ptr();
    let end = unsafe { start.add(2) };

    let (x, y) = (start, unsafe { end.sub(1) });

    group.bench_with_input(BenchmarkId::new("ptr::swap", count), &1, |b, _| {
        b.iter(|| unsafe { ptr::swap(x, y) })
    });

    group.bench_with_input(BenchmarkId::new("mem::swap", count), &1, |b, _| {
        b.iter(|| unsafe { swap(start, 0, 2) })
    });

    group.bench_with_input(BenchmarkId::new("read-write", count), &1, |b, _| {
        b.iter(|| unsafe {
            let t = x.read();

            x.write(y.read());
            y.write(t);
        })
    });

    group.bench_with_input(BenchmarkId::new("ptr::replace", count), &1, |b, _| {
        b.iter(|| unsafe {
            start.write(end.sub(1).replace(start.read()));
        })
    });

    group.bench_with_input(BenchmarkId::new("slice.reverse", count), &1, |b, _| {
        b.iter(|| unsafe {
            reverse_slice(start, 3);
        })
    });

    group.bench_with_input(BenchmarkId::new("slice::swap", count), &1, |b, _| {
        b.iter(|| unsafe {
            let slice = std::slice::from_raw_parts_mut(start, count);
            slice.swap(0, 2);
        })
    });

    group.bench_with_input(BenchmarkId::new("slice::swap", count), &1, |b, _| {
        b.iter(|| unsafe {
            let slice = std::slice::from_raw_parts_mut(start, count);
            slice.swap_unchecked(0, 2);
        })
    });

    group.bench_with_input(BenchmarkId::new("vector.reverse", count), &1, |b, _| {
        b.iter(|| v.reverse());
    });
}

fn bench_swap_pair(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("Swap pair"));

    seq_macro::seq!(i in 1..=10 {
       case_swap::<i>(&mut group);
    });

    group.finish();
}

/// cargo bench --bench=swaps "Swap forward/10/\d+"
fn bench_swap_forward(c: &mut Criterion) {
    case_swap_forward::<1>(c, 10);
    case_swap_forward::<1>(c, 50);
    case_swap_forward::<1>(c, 1000);
    case_swap_forward::<1>(c, 100_000);

    case_swap_forward::<10>(c, 10);
    case_swap_forward::<10>(c, 100_000);
}

/// cargo bench --bench=swaps "Swap backward/10/\d+"
fn bench_swap_backward(c: &mut Criterion) {
    case_swap_backward::<1>(c, 10);
    case_swap_backward::<1>(c, 50);
    case_swap_backward::<1>(c, 1000);
    case_swap_backward::<1>(c, 100_000);

    case_swap_backward::<10>(c, 10);
    case_swap_backward::<10>(c, 100_000);
}

criterion_group! {
    name = benches;

//    config = Criterion::default().sample_size(500).measurement_time(Duration::new(120, 0));
    config = Criterion::default();
             // .sample_size(500)

    targets = bench_swap_backward, bench_swap_forward, bench_swap_pair
}

criterion_main!(benches);
