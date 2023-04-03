use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pprof::criterion::{Output, PProfProfiler};
use rust_rotates::*;

// use std::time::Duration;
// use std::ptr;

// fn div(s: usize, diff: usize) -> (usize, usize) {
// assert!(s >= diff);
// assert!(s % 2 == diff % 2);
//
// let r = s / 2 - diff / 2;
//
// (s - r, r)
// }

// fn prepare(size: usize, diff: usize) -> (Vec<usize>, (usize, *mut usize, usize)) {
// let (l, r) = div(size, diff);
// let mut v = seq(size);
//
// unsafe {
// let p = &v[..].as_mut_ptr().add(l);
// (v, (l, p.clone(), r))
// }
// }

fn seq(size: usize) -> Vec<usize> {
    let mut v = vec![0; size];
    for i in 0..size {
        v[i] = i + 1;
    }
    v
}

fn test(
    rotate: unsafe fn(left: usize, mid: *mut usize, right: usize),
    left: usize,
    p: *mut usize,
    right: usize,
) {
    //    if left <= right {
    unsafe { rotate(left, p, right) }
    // unsafe{ rotate(right, p.add(right - left), left) }
    // } else {
    // unsafe{ rotate(left, p, right) }
    // unsafe{ rotate(right, p.sub(left - right), left) }
    // }
}

fn test_swap<T>(
    swap: unsafe fn(x: *mut T, y: *mut T, count: usize),
    x: *mut T,
    y: *mut T,
    count: usize,
) {
    if left <= right {
        unsafe { swap(x, y, count) }
        // unsafe{ rotate(right, p.add(right - left), left) }
    } else {
        // unsafe{ rotate(left, p, right) }
        // unsafe{ rotate(right, p.sub(left - right), left) }
    }
}

fn case_all(c: &mut Criterion, length: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("All/{length}"));
    let mut v = seq(*ls.into_iter().max().unwrap());

    for l in ls {
        let start = unsafe {
            let x = &v[..].as_mut_ptr();
            x.clone()
        };

        let y = unsafe { x.add(l / 2) };

        group.bench_with_input(BenchmarkId::new("ptr::swap", l), count, |b, l| {
            b.iter(|| test_swap(ptr::swap_nonoverlapping::<usize>, x, y, count))
        });
        group.bench_with_input(
            BenchmarkId::new("ptr::swap_nonoverlapping", l),
            count,
            |b, l| b.iter(|| test_swap(ptr::swap_nonoverlapping::<usize>, x, y, count)),
        );
        group.bench_with_input(
            BenchmarkId::new("utils::copy_backward", l),
            count,
            |b, l| b.iter(|| test_swap(ptr::swap_nonoverlapping::<usize>, x, y, count)),
        );
        group.bench_with_input(
            BenchmarkId::new("ptr::copy_nonoverlapping", count),
            count,
            |b, l| b.iter(|| test_swap(ptr::copy_nonoverlapping::<usize>, x, y, count)),
        );
        group.bench_with_input(BenchmarkId::new("ptr::copy", l), l, |b, l| {
            b.iter(|| test_swap(ptr::copy::<usize>, start, y, count))
        });
    }

    group.finish();
}

fn bench_all(c: &mut Criterion) {
    case_all(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
}

criterion_group! {
    name = benches;

//    config = Criterion::default().sample_size(500).measurement_time(Duration::new(120, 0));
    config = Criterion::default()
             .sample_size(500)
             .with_profiler(
                  PProfProfiler::new(100, Output::Flamegraph(None))
              );

    targets = bench_all
}

criterion_main!(benches);
