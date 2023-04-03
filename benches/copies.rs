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
///  length = 15
///                                     count = 3
/// [1  2  3  4  5  6  7  8  9 10 11 12 13 14 15]
///  |     |                             |     |
///  src                                 dst
/// ```
fn test_forward<T>(
    copy: unsafe fn(src: *const T, dst: *mut T, count: usize),
    start: *mut T,
    length: usize,
    count: usize,
) {
    unsafe { copy(start, start.add(length - count), count) }
}

fn case_forward(c: &mut Criterion, length: usize, counts: &[usize]) {
    let mut group = c.benchmark_group(format!("Copy forward/{length}"));
    let mut v = seq(length);

    for count in counts {
        let start = &v[..].as_mut_ptr();

        group.bench_with_input(
            BenchmarkId::new("utils::copy_backward", count),
            count,
            |b, _| {
                b.iter(|| {
                    test_forward(copy_backward::<usize>, start.clone(), length, count.clone())
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("ptr::copy_nonoverlapping", count),
            count,
            |b, _| {
                b.iter(|| {
                    test_forward(
                        ptr::copy_nonoverlapping::<usize>,
                        start.clone(),
                        length,
                        count.clone(),
                    )
                })
            },
        );

        group.bench_with_input(BenchmarkId::new("ptr::copy", count), count, |b, _| {
            b.iter(|| test_forward(ptr::copy::<usize>, start.clone(), length, count.clone()))
        });
    }

    group.finish();
}

/// cargo bench --bench=copies "Copy forward/2"
/// cargo bench --bench=copies "Copy forward/10"
fn bench_copy_forward(c: &mut Criterion) {
    case_forward(c, 2, &[0, 1]);
    case_forward(c, 3, &[0, 1, 2]);
    case_forward(c, 5, &[0, 1, 2, 3, 4, 5]);
    case_forward(c, 10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    case_forward(
        c,
        15,
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    );
    case_forward(
        c,
        20,
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        ],
    );
}

criterion_group! {
    name = benches;

//    config = Criterion::default().sample_size(500).measurement_time(Duration::new(120, 0));
    config = Criterion::default()
             .sample_size(500)
             .with_profiler(
                  PProfProfiler::new(100, Output::Flamegraph(None))
              );

    targets = bench_copy_forward
}

criterion_main!(benches);
