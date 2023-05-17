use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rotations::*;
// use pprof::criterion::{Output, PProfProfiler};

// use std::time::Duration;
// use std::ptr;

fn seq<const count: usize>(size: usize) -> Vec<[usize; count]> {
    let mut v = vec![[0; count]; size];
    for i in 0..size {
        v[i] = [i + 1; count];
    }
    v
}

fn test<T>(
    rotate: unsafe fn(left: usize, mid: *mut T, right: usize),
    left: usize,
    p: *mut T,
    right: usize,
) {
    unsafe { rotate(left, p, right) }
}

fn buf_test<T>(
    rotate: unsafe fn(left: usize, mid: *mut T, right: usize, buffer: &mut [T]),
    left: usize,
    p: *mut T,
    right: usize,
    buffer: &mut [T],
) {
    unsafe { rotate(left, p, right, buffer) }
}

// fn case_all(c: &mut Criterion, length: usize, ls: &[usize]) {
//     use criterion::black_box;

//     let mut group = c.benchmark_group(format!("All/{length}"));
//     //    group.throughput(Throughput::Elements(length as u64));

//     // let mut group = c.benchmark_group(format!("Bridge/{len}").as_str());
//     let mut buffer = vec![0; length];
//     let mut v = seq(length);

//     for l in ls {
//         let mid = unsafe {
//             let p = &v[..].as_mut_ptr().add(l.clone());
//             p.clone()
//         };

//         let r = length - l;

//         group.bench_with_input(BenchmarkId::new("Contrev", l), l, |b, _| {
//             b.iter(|| black_box(test(ptr_contrev_rotate::<usize>, l.clone(), mid, r)))
//         });
//         group.bench_with_input(BenchmarkId::new("Direct", l), l, |b, _| {
//             b.iter(|| test(ptr_direct_rotate::<usize>, l.clone(), mid, r))
//         });
//         group.bench_with_input(BenchmarkId::new("Bridge", l), l, |b, _| {
//             b.iter(|| {
//                 black_box(buf_test(
//                     ptr_bridge_rotate::<usize>,
//                     l.clone(),
//                     mid,
//                     r,
//                     buffer.as_mut_slice(),
//                 ))
//             })
//         });
//         group.bench_with_input(BenchmarkId::new("Raft", l), l, |b, _| {
//             b.iter(|| {
//                 black_box(buf_test(
//                     ptr_raft_rotate::<usize>,
//                     l.clone(),
//                     mid,
//                     r,
//                     buffer.as_mut_slice(),
//                 ))
//             })
//         });
//         group.bench_with_input(BenchmarkId::new("Aux", l), l, |b, _| {
//             b.iter(|| {
//                 black_box(buf_test(
//                     ptr_aux_rotate::<usize>,
//                     l.clone(),
//                     mid,
//                     r,
//                     buffer.as_mut_slice(),
//                 ))
//             })
//         });
//         group.bench_with_input(BenchmarkId::new("Default", l), l, |b, _| {
//             b.iter(|| test(stable_ptr_rotate::<usize>, l.clone(), mid, r))
//         });
//         group.bench_with_input(BenchmarkId::new("Piston", l), l, |b, _| {
//             b.iter(|| test(ptr_piston_rotate::<usize>, l.clone(), mid, r))
//         });
//         group.bench_with_input(BenchmarkId::new("Helix", l), l, |b, _| {
//             b.iter(|| test(ptr_helix_rotate::<usize>, l.clone(), mid, r))
//         });
//         group.bench_with_input(BenchmarkId::new("Grail", l), l, |b, _| {
//             b.iter(|| test(ptr_grail_rotate::<usize>, l.clone(), mid, r))
//         });
//         group.bench_with_input(BenchmarkId::new("Drill", l), l, |b, _| {
//             b.iter(|| test(ptr_drill_rotate::<usize>, l.clone(), mid, r))
//         });
//         group.bench_with_input(BenchmarkId::new("GM_rec", l), l, |b, _| {
//             b.iter(|| test(ptr_griesmills_rotate_rec::<usize>, l.clone(), mid, r))
//         });
//         group.bench_with_input(BenchmarkId::new("Rev", l), l, |b, _| {
//             b.iter(|| test(ptr_reversal_rotate::<usize>, l.clone(), mid, r))
//         });
//         // group.bench_with_input(BenchmarkId::new("Trinity", l), l, |b, _| {
//         //     b.iter(|| test(ptr_trinity_rotate::<usize>, l.clone(), mid, r))
//         // });
//     }

//     group.finish();
// }

fn case_buf<const count: usize>(c: &mut Criterion, length: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Buf/{length}/{count}"));
    //    group.throughput(Throughput::Elements(length as u64));

    let mut buffer = Vec::<[usize; count]>::with_capacity(length);
    let mut v = seq::<count>(length);

    for l in ls {
        let mid = unsafe {
            let p = &v[..].as_mut_ptr().add(l.clone());
            p.clone()
        };

        let r = length - l;

        group.bench_with_input(BenchmarkId::new("Direct", l), l, |b, _| {
            b.iter(|| test(ptr_direct_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Naive aux", l), l, |b, _| {
            b.iter(|| {
                buf_test(
                    ptr_naive_aux_rotate::<[usize; count]>,
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
                    ptr_aux_rotate::<[usize; count]>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                )
            })
        });
        group.bench_with_input(BenchmarkId::new("Bridge", l), l, |b, _| {
            b.iter(|| {
                buf_test(
                    ptr_bridge_rotate::<[usize; count]>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                )
            })
        });
        group.bench_with_input(BenchmarkId::new("Raft", l), l, |b, _| {
            b.iter(|| {
                buf_test(
                    ptr_raft_rotate::<[usize; count]>,
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

fn case_contrev<const count: usize>(c: &mut Criterion, length: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Contrev/{length}/{count}"));
    //    group.throughput(Throughput::Elements(length as u64));

    let mut buffer = Vec::<[usize; count]>::with_capacity(length);
    let mut v = seq::<count>(length);

    for l in ls {
        let mid = unsafe {
            let p = &v[..].as_mut_ptr().add(l.clone());
            p.clone()
        };

        let r = length - l;

        group.bench_with_input(BenchmarkId::new("Direct", l), l, |b, _| {
            b.iter(|| test(ptr_direct_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Contrev", l), l, |b, _| {
            b.iter(|| test(ptr_contrev_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Bridge", l), l, |b, _| {
            b.iter(|| {
                buf_test(
                    ptr_bridge_rotate::<[usize; count]>,
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
                    ptr_aux_rotate::<[usize; count]>,
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

fn case_gm<const count: usize>(c: &mut Criterion, length: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("GM/{length}/{count}"));
    //    group.throughput(Throughput::Elements(length as u64));

    let mut buffer = Vec::<[usize; count]>::with_capacity(length);
    let mut v = seq::<count>(length);

    for l in ls {
        let mid = unsafe {
            let p = &v[..].as_mut_ptr().add(l.clone());
            p.clone()
        };

        let r = length - l;

        group.bench_with_input(BenchmarkId::new("Direct", l), l, |b, _| {
            b.iter(|| test(ptr_direct_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("GM", l), l, |b, _| {
            b.iter(|| test(ptr_griesmills_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("GM (rec)", l), l, |b, _| {
            b.iter(|| {
                test(
                    ptr_griesmills_rotate_rec::<[usize; count]>,
                    l.clone(),
                    mid,
                    r,
                )
            })
        });
        group.bench_with_input(BenchmarkId::new("Grail", l), l, |b, _| {
            b.iter(|| test(ptr_grail_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Drill", l), l, |b, _| {
            b.iter(|| test(ptr_drill_rotate::<[usize; count]>, l.clone(), mid, r))
        });

        // group.bench_with_input(BenchmarkId::new("Contrev", l), l, |b, _| {
        //     b.iter(|| test(ptr_contrev_rotate::<[usize; count]>, l.clone(), mid, r))
        // });
        group.bench_with_input(BenchmarkId::new("Aux", l), l, |b, _| {
            b.iter(|| {
                buf_test(
                    ptr_aux_rotate::<[usize; count]>,
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

fn case_main<const count: usize>(c: &mut Criterion, length: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Main/{length}/{count}"));
    //    group.throughput(Throughput::Elements(length as u64));

    let mut buffer = Vec::<[usize; count]>::with_capacity(length);
    let mut v = seq::<count>(length);

    for l in ls {
        let mid = unsafe {
            let p = &v[..].as_mut_ptr().add(l.clone());
            p.clone()
        };

        let r = length - l;

        group.bench_with_input(BenchmarkId::new("Direct", l), l, |b, _| {
            b.iter(|| test(ptr_direct_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Contrev", l), l, |b, _| {
            b.iter(|| test(ptr_contrev_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("GM", l), l, |b, _| {
            b.iter(|| test(ptr_griesmills_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Helix", l), l, |b, _| {
            b.iter(|| test(ptr_helix_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Piston", l), l, |b, _| {
            b.iter(|| test(ptr_piston_rotate::<[usize; count]>, l.clone(), mid, r))
        });
        group.bench_with_input(BenchmarkId::new("Rev", l), l, |b, _| {
            b.iter(|| test(ptr_reversal_rotate::<[usize; count]>, l.clone(), mid, r))
        });

        group.bench_with_input(BenchmarkId::new("Aux", l), l, |b, _| {
            b.iter(|| {
                buf_test(
                    ptr_aux_rotate::<[usize; count]>,
                    l.clone(),
                    mid,
                    r,
                    buffer.as_mut_slice(),
                )
            })
        });
        group.bench_with_input(BenchmarkId::new("Bridge", l), l, |b, _| {
            b.iter(|| {
                buf_test(
                    ptr_bridge_rotate::<[usize; count]>,
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

fn bench_short(c: &mut Criterion) {
    // 1 * usize
    case_main::<1>(c, 4, &[1, 2, 3]);
    case_main::<1>(c, 6, &[1, 2, 3, 4, 5]);
    case_main::<1>(c, 8, &[1, 2, 3, 4, 5, 6, 7]);
    case_main::<1>(c, 10, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    case_main::<1>(c, 12, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    case_main::<1>(c, 14, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]);
    case_main::<1>(c, 16, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

    case_main::<1>(c, 18, &[1, 2, 3, 5, 7, 9, 11, 13, 15, 16, 17]);
    case_main::<1>(c, 20, &[1, 2, 3, 5, 7, 9, 11, 13, 15, 17, 18, 19]);

    case_main::<1>(c, 22, &[1, 2, 3, 4, 5, 7, 10, 13, 15, 17, 19, 21]);
    case_main::<1>(c, 24, &[1, 2, 3, 4, 5, 7, 10, 13, 15, 17, 19, 21, 23, 24]);
    case_main::<1>(c, 26, &[1, 2, 3, 4, 5, 7, 10, 13, 16, 19, 22, 23, 24, 25]);
    case_main::<1>(c, 28, &[1, 2, 3, 4, 6, 8, 10, 14, 16, 19, 22, 25, 26, 27]);
    case_main::<1>(c, 30, &[1, 2, 3, 4, 6, 8, 10, 14, 16, 19, 22, 25, 28, 29]);
}

fn bench_buf(c: &mut Criterion) {
    // 1 * usize
    case_buf::<1>(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    case_buf::<1>(c, 30, &[1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 29]);

    case_buf::<1>(c, 100, &[1, 20, 32, 35, 40, 45, 51, 60, 66, 69, 80, 90, 99]);
    case_buf::<1>(
        c,
        1000,
        &[1, 32, 200, 334, 400, 485, 516, 668, 800, 900, 969, 999],
    );
    case_buf::<1>(
        c,
        10000,
        &[
            1, 32, 2000, 3334, 4000, 4985, 5016, 6668, 8000, 9000, 9969, 9999,
        ],
    );

    // 5 * usize, possible buffer size = 6 elements of 5 * usize
    case_buf::<5>(c, 6, &[1, 2, 3, 4, 5]);

    case_buf::<5>(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    case_buf::<5>(c, 30, &[1, 3, 6, 10, 13, 16, 18, 22, 25, 27, 29]);
    case_buf::<5>(c, 100, &[1, 6, 20, 30, 40, 48, 53, 60, 70, 80, 90, 95, 99]);
    case_buf::<5>(
        c,
        1000,
        &[1, 6, 200, 334, 400, 498, 503, 668, 800, 900, 995, 999],
    );
    case_buf::<5>(
        c,
        10000,
        &[
            1, 6, 200, 3334, 4000, 4998, 5003, 6668, 8000, 9000, 9995, 9999,
        ],
    );
}

fn bench_contrev(c: &mut Criterion) {
    // 1 * usize
    case_contrev::<1>(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    case_contrev::<1>(c, 30, &[1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 29]);

    case_contrev::<1>(c, 100, &[1, 20, 32, 35, 40, 45, 51, 60, 66, 69, 80, 90, 99]);
    case_contrev::<1>(
        c,
        1000,
        &[1, 32, 200, 334, 400, 485, 516, 668, 800, 900, 969, 999],
    );
    case_contrev::<1>(
        c,
        10000,
        &[
            1, 32, 2000, 3334, 4000, 4985, 5016, 6668, 8000, 9000, 9969, 9999,
        ],
    );

    // 2 * usize, possible buffer size = 16 elements of 2 * usize
    case_contrev::<2>(c, 6, &[1, 2, 3, 4, 5]);

    case_contrev::<2>(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    case_contrev::<2>(c, 30, &[1, 3, 6, 10, 13, 16, 18, 22, 25, 27, 29]);
    case_contrev::<2>(c, 100, &[1, 16, 20, 30, 40, 48, 53, 60, 70, 80, 85, 99]);
    case_contrev::<2>(
        c,
        1000,
        &[1, 16, 200, 334, 400, 498, 503, 668, 800, 900, 985, 999],
    );
    case_contrev::<2>(
        c,
        10000,
        &[
            1, 16, 200, 3334, 4000, 4998, 5003, 6668, 8000, 9000, 9985, 9999,
        ],
    );

    // 5 * usize, possible buffer size = 6 elements of 5 * usize
    case_contrev::<5>(c, 6, &[1, 2, 3, 4, 5]);

    case_contrev::<5>(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    case_contrev::<5>(c, 30, &[1, 3, 6, 10, 13, 16, 18, 22, 25, 27, 29]);
    case_contrev::<5>(c, 100, &[1, 6, 20, 30, 40, 48, 53, 60, 70, 80, 90, 95, 99]);
    case_contrev::<5>(
        c,
        1000,
        &[1, 6, 200, 334, 400, 498, 503, 668, 800, 900, 995, 999],
    );
    case_contrev::<5>(
        c,
        10000,
        &[
            1, 6, 200, 3334, 4000, 4998, 5003, 6668, 8000, 9000, 9995, 9999,
        ],
    );
}

fn bench_gm(c: &mut Criterion) {
    // 1 * usize
    case_gm::<1>(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    case_gm::<1>(c, 30, &[1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 29]);

    case_gm::<1>(c, 100, &[1, 20, 32, 35, 40, 45, 51, 60, 66, 69, 80, 90, 99]);
    case_gm::<1>(
        c,
        1000,
        &[1, 32, 200, 334, 400, 485, 516, 668, 800, 900, 969, 999],
    );
    case_gm::<1>(
        c,
        10000,
        &[
            1, 32, 2000, 3334, 4000, 4985, 5016, 6668, 8000, 9000, 9969, 9999,
        ],
    );

    // 2 * usize, possible buffer size = 16 elements of 2 * usize
    case_gm::<2>(c, 6, &[1, 2, 3, 4, 5]);

    case_gm::<2>(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    case_gm::<2>(c, 30, &[1, 3, 6, 10, 13, 16, 18, 22, 25, 27, 29]);
    case_gm::<2>(c, 100, &[1, 16, 20, 30, 40, 48, 53, 60, 70, 80, 85, 99]);
    case_gm::<2>(
        c,
        1000,
        &[1, 16, 200, 334, 400, 498, 503, 668, 800, 900, 985, 999],
    );
    case_gm::<2>(
        c,
        10000,
        &[
            1, 16, 200, 3334, 4000, 4998, 5003, 6668, 8000, 9000, 9985, 9999,
        ],
    );

    // 5 * usize, possible buffer size = 6 elements of 5 * usize
    case_gm::<5>(c, 6, &[1, 2, 3, 4, 5]);

    case_gm::<5>(c, 15, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    case_gm::<5>(c, 30, &[1, 3, 6, 10, 13, 16, 18, 22, 25, 27, 29]);
    case_gm::<5>(c, 100, &[1, 6, 20, 30, 40, 48, 53, 60, 70, 80, 90, 95, 99]);
    case_gm::<5>(
        c,
        1000,
        &[1, 6, 200, 334, 400, 498, 503, 668, 800, 900, 995, 999],
    );
    case_gm::<5>(
        c,
        10000,
        &[
            1, 6, 200, 3334, 4000, 4998, 5003, 6668, 8000, 9000, 9995, 9999,
        ],
    );
}

criterion_group! {
    name = benches;

//    config = Criterion::default().sample_size(500).measurement_time(Duration::new(120, 0));
    config = Criterion::default()
             .sample_size(1000);
             // .with_profiler(
             //      PProfProfiler::new(100, Output::Flamegraph(None))
             //  );

    targets = bench_buf, bench_contrev, bench_gm, bench_short
}

criterion_main!(benches);
