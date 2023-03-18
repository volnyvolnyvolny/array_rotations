use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rotate::*;
use std::time::Duration;

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

// fn test(rotate: unsafe fn(left: usize, mid: *mut usize, right: usize), size: usize, diff: usize){
    // let (_vec, (l, p, r)) = prepare(size, diff);
    // unsafe{ rotate(l, p, r); }
// }

fn seq(size: usize) -> Vec<usize> {
    let mut v = vec![0; size];
    for i in 0..size { v[i] = i+1; }
    v
}

fn test(rotate: unsafe fn(left: usize, mid: *mut usize, right: usize), left: usize, p: *mut usize, right: usize){
//    if left <= right {
        unsafe{ rotate(left, p, right) }
        // unsafe{ rotate(right, p.add(right - left), left) }
    // } else {
        // unsafe{ rotate(left, p, right) }
        // unsafe{ rotate(right, p.sub(left - right), left) }
    // }
}

fn case_grail_vs_drill_vs_helix_vs_piston_vs_bridge(c: &mut Criterion, len: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Rotations (GM vs Grail vs Drill vs Helix vs Piston vs Bridge) (len = {len})").as_str());
    let mut v = seq(len);

    for l in ls {
        let p =
            unsafe {
                let p = &v[..].as_mut_ptr().add(l.clone());
                p.clone()
            };
    
        let r = len - l;

        group.bench_with_input(BenchmarkId::new("Piston", l),   l, |b, l| b.iter(|| test(ptr_piston_rotate::<usize>,     l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Helix", l),    l, |b, l| b.iter(|| test(ptr_helix_rotate::<usize> ,     l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Grail", l),    l, |b, l| b.iter(|| test(ptr_grail_rotate::<usize>,      l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Drill", l),    l, |b, l| b.iter(|| test(ptr_drill_rotate::<usize>,      l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("GM", l),       l, |b, l| b.iter(|| test(ptr_griesmills_rotate::<usize>, l.clone(), p, r)));

        group.bench_with_input(BenchmarkId::new("Bridge", l),   l, |b, l| b.iter(|| test(ptr_bridge_rotate::<usize>,     l.clone(), p, r)));
    }

    group.finish();
}

fn case(c: &mut Criterion, len: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Rotations (Stable vs Piston) (len = {len}) [left]").as_str());
    let mut v = seq(len);

    for l in ls {
        let p =
            unsafe {
                let p = &v[..].as_mut_ptr().add(l.clone());
                p.clone()
            };
    
        let r = len - l;

        group.bench_with_input(BenchmarkId::new("Stable", l),   l, |b, l| b.iter(|| test(stable_ptr_rotate::<usize>,     l.clone(), p, r)));

        group.bench_with_input(BenchmarkId::new("Aux", l),      l, |b, l| b.iter(|| test(ptr_aux_rotate::<usize>,        l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Bridge", l),   l, |b, l| b.iter(|| test(ptr_bridge_rotate::<usize>,     l.clone(), p, r)));

        group.bench_with_input(BenchmarkId::new("Piston", l),   l, |b, l| b.iter(|| test(ptr_piston_rotate::<usize>,     l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("GM", l),       l, |b, l| b.iter(|| test(ptr_griesmills_rotate::<usize>, l.clone(), p, r)));
 
        group.bench_with_input(BenchmarkId::new("Rev", l),      l, |b, l| b.iter(|| test(ptr_reversal_rotate::<usize>,   l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Contrev", l),  l, |b, l| b.iter(|| test(ptr_contrev_rotate::<usize>,    l.clone(), p, r)));

        group.bench_with_input(BenchmarkId::new("Trinity", l),  l, |b, l| b.iter(|| test(ptr_trinity_rotate::<usize>,    l.clone(), p, r)));
//        group.bench_with_input(BenchmarkId::new("Comb", l),     l, |b, l| b.iter(|| test(ptr_comb_rotate::<usize>,       l.clone(), p, r)));

        group.bench_with_input(BenchmarkId::new("Juggling", l), l, |b, l| b.iter(|| test(ptr_juggling_rotate::<usize>,   l.clone(), p, r)));
    }

    group.finish();
}

fn case_stable(c: &mut Criterion, len: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Rotations (Stable vs Auxiliary) (len = {len})").as_str());
    let mut v = seq(len);

    for l in ls {
        let p =
            unsafe {
                let p = &v[..].as_mut_ptr().add(l.clone());
                p.clone()
            };
    
        let r = len - l;

        group.bench_with_input(BenchmarkId::new("Stable", l), l, |b, l| b.iter(|| test(stable_ptr_rotate::<usize>, l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Aux", l),    l, |b, l| b.iter(|| test(ptr_aux_rotate::<usize>,    l.clone(), p, r)));
    }

    group.finish();
}

fn case_stable_vs_piston(c: &mut Criterion, len: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Rotations (Stable vs Piston) (len = {len}) [left]").as_str());
    let mut v = seq(len);

    for l in ls {
        let p =
            unsafe {
                let p = &v[..].as_mut_ptr().add(l.clone());
                p.clone()
            };
    
        let r = len - l;

        group.bench_with_input(BenchmarkId::new("Stable", l),   l, |b, l| b.iter(|| test(stable_ptr_rotate::<usize>, l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Piston", l),   l, |b, l| b.iter(|| test(ptr_piston_rotate::<usize>, l.clone(), p, r)));
    }

    group.finish();
}

fn case_stable_vs_contrev_vs_piston(c: &mut Criterion, len: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Rotations (Stable vs Contrev vs Piston) (len = {len})").as_str());
    let mut v = seq(len);

    for l in ls {
        let p =
            unsafe {
                let p = &v[..].as_mut_ptr().add(l.clone());
                p.clone()
            };
    
        let r = len - l;

        group.bench_with_input(BenchmarkId::new("Stable", l),  l, |b, l| b.iter(|| test(stable_ptr_rotate::<usize>,  l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Piston", l),  l, |b, l| b.iter(|| test(ptr_piston_rotate::<usize>,  l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Contrev", l), l, |b, l| b.iter(|| test(ptr_contrev_rotate::<usize>, l.clone(), p, r)));
    }

    group.finish();
}

fn case_stable_vs_contrev_vs_piston_vs_bridge(c: &mut Criterion, len: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Rotations (Stable vs Contrev vs Piston vs Bridge) (len = {len})").as_str());
    let mut v = seq(len);

    for l in ls {
        let p =
            unsafe {
                let p = &v[..].as_mut_ptr().add(l.clone());
                p.clone()
            };
    
        let r = len - l;

        group.bench_with_input(BenchmarkId::new("Stable", l),  l, |b, l| b.iter(|| test(stable_ptr_rotate::<usize>,  l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Piston", l),  l, |b, l| b.iter(|| test(ptr_piston_rotate::<usize>,  l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Contrev", l), l, |b, l| b.iter(|| test(ptr_contrev_rotate::<usize>, l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Bridge", l),  l, |b, l| b.iter(|| test(ptr_bridge_rotate::<usize>,  l.clone(), p, r)));
    }

    group.finish();
}

fn case_contrev_vs_trinity(c: &mut Criterion, len: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Rotations (Contrev vs Trinity) (len = {len})").as_str());
    let mut v = seq(len);

    for l in ls {
        let p =
            unsafe {
                let p = &v[..].as_mut_ptr().add(l.clone());
                p.clone()
            };
    
        let r = len - l;

        group.bench_with_input(BenchmarkId::new("Contrev", l), l, |b, l| b.iter(|| test(ptr_contrev_rotate::<usize>, l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Trinity", l), l, |b, l| b.iter(|| test(ptr_bridge_rotate::<usize>,  l.clone(), p, r)));
    }

    group.finish();
}

fn case_50000_stable_vs_contrev_vs_bridge(c: &mut Criterion, len: usize, ls: &[usize]) {
    let mut group = c.benchmark_group(format!("Rotations (Stable vs Contrev vs Bridge) (len = 100000)").as_str());
    let mut v = seq(len);

    for l in ls {
        let p =
            unsafe {
                let p = &v[..].as_mut_ptr().add(l.clone());
                p.clone()
            };
    
        let r = len - l;

        group.bench_with_input(BenchmarkId::new("Stable", l),  l, |b, l| b.iter(|| test(stable_ptr_rotate::<usize>,  l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Contrev", l), l, |b, l| b.iter(|| test(ptr_contrev_rotate::<usize>, l.clone(), p, r)));
        group.bench_with_input(BenchmarkId::new("Bridge", l), l, |b, l| b.iter(|| test(ptr_bridge_rotate::<usize>, l.clone(), p, r)));
    }

    group.finish();
}

fn benchmark(c: &mut Criterion) {
//    case_stable(c, 100, &[1, 16, 32, 33, 40, 50, 60, 68, 69, 84, 100]);

//    case_stable(c, 1000000, &[1000, 100000, 200000, 300000, 400000, 499990]);
//    case_stable_vs_contrev_vs_piston(c, 1000,  &[33, 100, 200, 300, 400, 500, 600, 700, 800, 900, 967]);
//    case_stable_vs_contrev_vs_piston(c, 10000, &[33, 1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 9967]);
//    case_stable_vs_contrev_vs_piston_vs_bridge(c, 100000, &[33, 10000, 20000, 30000, 40000, 49983, 49992, 50008, 50017, 60000, 70000, 80000, 90000, 99967]);
//    case_50000_stable_vs_contrev_vs_bridge(c, 100000, &[49999, 50000, 50001]);

    case_grail_vs_drill_vs_helix_vs_piston_vs_bridge(c, 100000, &[33, 10000, 20000, 30000, 40000, 49983, 49992, 50008, 50017, 60000, 70000, 80000, 90000, 99967]);
//    case_contrev_vs_trinity(c, 100000, &[33, 10000, 20000, 30000, 40000, 49983, 49992, 50008, 50017, 60000, 70000, 80000, 90000, 99967]);

                         //aux                  //bridge
    // case(c,      100, &[1, 5, 10, 20, 32,       33,       35]);
    // case(c,     1000, &[1,            32,      480,      490]);
    // case(c,    10000, &[1,            32,     4980,     4990]);
    // case(c,   100000, &[1,            32,    49980,    49990]);
    // case(c,  1000000, &[100, 100000, 200000, 300000, 400000, 499990]);
    // case(c, 10000000, &[1,            32, 100, 1000, 100000, 1000000, 2000000, 3000000, 4000000, 4999980,  4999990]);
}

criterion_group!{
    name = benches;
    // This can be any expression that returns a `Criterion` object.
//    config = Criterion::default().sample_size(500).measurement_time(Duration::new(120, 0));
    config = Criterion::default().sample_size(500);
    targets = benchmark
}
criterion_main!(benches);
