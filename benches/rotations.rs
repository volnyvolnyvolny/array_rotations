use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rotates::*;
// use pprof::criterion::{Output, PProfProfiler};

// use std::time::Duration;
// use std::ptr;

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

fn buf_test(
    rotate: unsafe fn(left: usize, mid: *mut usize, right: usize, buffer: &mut [usize]),
    left: usize,
    p: *mut usize,
    right: usize,
    buffer: &mut [usize],
) {
    //    if left <= right {
    unsafe { rotate(left, p, right, buffer) }
    // unsafe{ rotate(right, p.add(right - left), left) }
    // } else {
    // unsafe{ rotate(left, p, right) }
    // unsafe{ rotate(right, p.sub(left - right), left) }
    // }
}

fn case_all(c: &mut Criterion, length: usize, ls: &[usize]) {
    use criterion::black_box;

    let mut group = c.benchmark_group(format!("All/{length}"));
    //    group.throughput(Throughput::Elements(length as u64));

    // let mut group = c.benchmark_group(format!("Bridge/{len}").as_str());
    let mut buffer = vec![0; length];
    let mut v = seq(length);

    for l in ls {
        let mid = unsafe {
            let p = &v[..].as_mut_ptr().add(l.clone());
            p.clone()
        };

        let r = length - l;

        group.bench_with_input(BenchmarkId::new("Contrev", l), l, |b, _| {
            b.iter(|| black_box(test(ptr_contrev_rotate::<usize>, l.clone(), mid, r)))
        });
        group.bench_with_input(BenchmarkId::new("Direct", l), l, |b, _| {
            b.iter(|| test(ptr_direct_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Bridge", l), l, |b, _| {
            b.iter(|| {
                black_box(buf_test(
                    ptr_bridge_rotate::<usize>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                ))
            })
        });
        group.bench_with_input(BenchmarkId::new("Raft", l), l, |b, _| {
            b.iter(|| {
                black_box(buf_test(
                    ptr_raft_rotate::<usize>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                ))
            })
        });
        group.bench_with_input(BenchmarkId::new("Aux", l), l, |b, _| {
            b.iter(|| {
                black_box(buf_test(
                    ptr_aux_rotate::<usize>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                ))
            })
        });
        group.bench_with_input(BenchmarkId::new("Default", l), l, |b, _| {
            b.iter(|| test(stable_ptr_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Piston", l), l, |b, _| {
            b.iter(|| test(ptr_piston_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Helix", l), l, |b, _| {
            b.iter(|| test(ptr_helix_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Grail", l), l, |b, _| {
            b.iter(|| test(ptr_grail_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Drill", l), l, |b, _| {
            b.iter(|| test(ptr_drill_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("GM_rec", l), l, |b, _| {
            b.iter(|| test(ptr_griesmills_rotate_rec::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Rev", l), l, |b, _| {
            b.iter(|| test(ptr_reversal_rotate::<usize>, l.clone(), mid, r))
        });
        // group.bench_with_input(BenchmarkId::new("Trinity", l), l, |b, _| {
        //     b.iter(|| test(ptr_trinity_rotate::<usize>, l.clone(), mid, r))
        // });
    }

    group.finish();
}

fn case_aux(c: &mut Criterion, length: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Aux/{length}"));
    //    group.throughput(Throughput::Elements(length as u64));

    // let mut group = c.benchmark_group(format!("Bridge/{len}").as_str());
    let mut buffer = Vec::<usize>::with_capacity(length);
    let mut v = seq(length);

    for l in ls {
        let mid = unsafe {
            let p = &v[..].as_mut_ptr().add(l.clone());
            p.clone()
        };

        let r = length - l;

        group.bench_with_input(BenchmarkId::new("Direct", l), l, |b, _| {
            b.iter(|| test(ptr_direct_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Naive aux", l), l, |b, _| {
            b.iter(|| {
                buf_test(
                    ptr_naive_aux_rotate::<usize>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                )
            })
        });
        group.bench_with_input(BenchmarkId::new("Aux", l), l, |b, _| {
            b.iter(|| {
                buf_test(
                    ptr_aux_rotate::<usize>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                )
            })
        });
    }

    group.finish();
}

fn case_bridge(c: &mut Criterion, length: usize, ls: &[usize]) {
    use criterion::black_box;

    let mut group = c.benchmark_group(format!("Bridge/{length}"));
    //    group.throughput(Throughput::Elements(length as u64));

    // let mut group = c.benchmark_group(format!("Bridge/{len}").as_str());
    let mut buffer = Vec::<usize>::with_capacity(length);
    let mut v = seq(length);

    for l in ls {
        let mid = unsafe {
            let p = &v[..].as_mut_ptr().add(l.clone());
            p.clone()
        };

        let r = length - l;

        group.bench_with_input(BenchmarkId::new("Contrev", l), l, |b, _| {
            b.iter(|| black_box(test(ptr_contrev_rotate::<usize>, l.clone(), mid, r)))
        });
        group.bench_with_input(BenchmarkId::new("Direct", l), l, |b, _| {
            b.iter(|| test(ptr_direct_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Bridge", l), l, |b, _| {
            b.iter(|| {
                black_box(buf_test(
                    ptr_bridge_rotate::<usize>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                ))
            })
        });
        group.bench_with_input(BenchmarkId::new("Raft", l), l, |b, _| {
            b.iter(|| {
                black_box(buf_test(
                    ptr_raft_rotate::<usize>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                ))
            })
        });
        group.bench_with_input(BenchmarkId::new("Aux", l), l, |b, _| {
            b.iter(|| {
                black_box(buf_test(
                    ptr_aux_rotate::<usize>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                ))
            })
        });
    }

    group.finish();
}

fn case_gm_helix(c: &mut Criterion, length: usize, ls: &[usize]) {
    use criterion::black_box;

    let mut group = c.benchmark_group(format!("GM/{length}"));
    //    group.throughput(Throughput::Elements(length as u64));

    // let mut group = c.benchmark_group(format!("Bridge/{len}").as_str());
    let mut buffer = Vec::<usize>::with_capacity(length);
    let mut v = seq(length);

    for l in ls {
        let mid = unsafe {
            let p = &v[..].as_mut_ptr().add(l.clone());
            p.clone()
        };

        let r = length - l;

        group.bench_with_input(BenchmarkId::new("Direct", l), l, |b, _| {
            b.iter(|| test(ptr_direct_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Helix", l), l, |b, _| {
            b.iter(|| test(ptr_helix_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Grail", l), l, |b, _| {
            b.iter(|| test(ptr_grail_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Drill", l), l, |b, _| {
            b.iter(|| test(ptr_drill_rotate::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("GM_rec", l), l, |b, _| {
            b.iter(|| test(ptr_griesmills_rotate_rec::<usize>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("GM", l), l, |b, _| {
            b.iter(|| test(ptr_griesmills_rotate::<usize>, l.clone(), mid, r))
        });
    }

    group.finish();
}

fn bench_aux(c: &mut Criterion) {
    case_aux(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    case_aux(c, 30, &[1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 29]);
    case_aux(c, 100, &[1, 20, 34, 40, 51, 60, 68, 80, 90, 99]);
    case_aux(c, 1000, &[1, 200, 334, 400, 501, 668, 800, 900, 999]);
    case_aux(
        c,
        10000,
        &[1, 2000, 3334, 4000, 5001, 6668, 8000, 9000, 9999],
    );
}

fn bench_bridge(c: &mut Criterion) {
    // case_bridge(c, 15,  &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    // case_bridge(c, 101, &[1, 20, 34, 40, 51, 60, 68, 80, 90, 101]);
    case_bridge(c, 1000, &[1, 200, 334, 400, 501, 668, 800, 900, 999]);
}

fn bench_gm_helix(c: &mut Criterion) {
    case_gm_helix(c, 1002, &[1, 200, 334, 400, 501, 668, 800, 900, 1001]);
    case_gm_helix(
        c,
        102,
        &[1, 10, 20, 30, 34, 40, 45, 51, 60, 68, 75, 80, 90, 95, 101],
    );
    case_gm_helix(c, 12, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
}

fn bench_all(c: &mut Criterion) {
    case_all(c, 1001, &[1, 200, 334, 400, 501, 668, 800, 900, 1000]);

    // case_all(c,   10, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    // case_all(c,  100, &[0, 1, 16, 32, 33, 40, 50, 60, 67, 68, 100]);
    // case_all(c, 1000, &[0, 32, 33, 100, 200, 300, 400, 500, 600, 700, 800, 900, 967, 968, 1000]);
    //    case_bridge(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    // case_all(c, 10000, &[1, 16, 32, 33, 40, 50, 60, 68, 69, 84, 100]);
    // case_all(c, 100000, &[1, 16, 32, 33, 40, 50, 60, 68, 69, 84, 100]);
    // case_all(c, 1000000, &[1, 16, 32, 33, 40, 50, 60, 68, 69, 84, 100]);

    //    case_stable(c, 100, &[1, 16, 32, 33, 40, 50, 60, 68, 69, 84, 100]);

    //    case_stable(c, 1000000, &[1000, 100000, 200000, 300000, 400000, 499990]);
    //    case_stable_vs_contrev_vs_piston(c, 1000,  &[33, 100, 200, 300, 400, 500, 600, 700, 800, 900, 967]);
    //    case_stable_vs_contrev_vs_piston(c, 10000, &[33, 1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 9967]);
    //    case_stable_vs_contrev_vs_piston_vs_bridge(c, 100000, &[33, 10000, 20000, 30000, 40000, 49983, 49992, 50008, 50017, 60000, 70000, 80000, 90000, 99967]);
    //    case_50000_stable_vs_contrev_vs_bridge(c, 100000, &[49999, 50000, 50001]);

    //    case_grail_vs_drill_vs_helix_vs_piston_vs_bridge(c, 100000, &[33, 10000, 20000, 30000, 40000, 49983, 49992, 50008, 50017, 60000, 70000, 80000, 90000, 99967]);
    //    case_contrev_vs_trinity(c, 100000, &[33, 10000, 20000, 30000, 40000, 49983, 49992, 50008, 50017, 60000, 70000, 80000, 90000, 99967]);

    //aux                  //bridge
    // case(c,      100, &[1, 5, 10, 20, 32,       33,       35]);
    // case(c,     1000, &[1,            32,      480,      490]);
    // case(c,    10000, &[1,            32,     4980,     4990]);
    // case(c,   100000, &[1,            32,    49980,    49990]);
    // case(c,  1000000, &[100, 100000, 200000, 300000, 400000, 499990]);
    // case(c, 10000000, &[1,            32, 100, 1000, 100000, 1000000, 2000000, 3000000, 4000000, 4999980,  4999990]);

    // group.finish();
}

criterion_group! {
    name = benches;

//    config = Criterion::default().sample_size(500).measurement_time(Duration::new(120, 0));
    config = Criterion::default()
             .sample_size(1000);
             // .with_profiler(
             //      PProfProfiler::new(100, Output::Flamegraph(None))
             //  );

    // targets = bench_all, bench_bridge
    targets = bench_all, bench_aux, bench_gm_helix, bench_bridge
}

criterion_main!(benches);
