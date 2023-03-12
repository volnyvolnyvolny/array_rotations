use criterion::{criterion_group, criterion_main, Criterion};
use rotate::*;

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
    unsafe{ rotate(left, p, right) }
}

fn benchmark(c: &mut Criterion) {
    let len = 1000000;
    let mut v = seq(len);

    for l in [1, 10, 50, 100, 200, 400, 1000, 99999, 199998, 299997, 399996, 499995] {
        let p = 
            unsafe {
                let p = &v[..].as_mut_ptr().add(l);
                p.clone()
            };

        let r = len - l;

        c.bench_function(format!("{l}/{r}_stable").as_str(),     |b| b.iter(|| test(stable_ptr_rotate::<usize>,     l, p, r)));
        c.bench_function(format!("{l}/{r}_bridge").as_str(),     |b| b.iter(|| test(ptr_bridge_rotate::<usize>,     l, p, r)));
        c.bench_function(format!("{l}/{r}_aux").as_str(),        |b| b.iter(|| test(ptr_aux_rotate::<usize>,        l, p, r)));
        c.bench_function(format!("{l}/{r}_rev").as_str(),        |b| b.iter(|| test(ptr_reversal_rotate::<usize>,   l, p, r)));
        c.bench_function(format!("{l}/{r}_contrev").as_str(),    |b| b.iter(|| test(ptr_reversal_rotate::<usize>,   l, p, r)));
        c.bench_function(format!("{l}/{r}_gm").as_str(),         |b| b.iter(|| test(ptr_griesmills_rotate::<usize>, l, p, r)));
        c.bench_function(format!("{l}/{r}_juggling").as_str(),   |b| b.iter(|| test(ptr_juggling_rotate::<usize>,   l, p, r)));
        c.bench_function(format!("{l}/{r}_trinity").as_str(),    |b| b.iter(|| test(ptr_trinity_rotate::<usize>,    l, p, r)));
        c.bench_function(format!("{l}/{r}_comb").as_str(),       |b| b.iter(|| test(ptr_trinity_rotate::<usize>,    l, p, r)));
        c.bench_function(format!("{l}/{r}_piston").as_str(),     |b| b.iter(|| test(ptr_piston_rotate::<usize>,     l, p, r)));
        c.bench_function(format!("{l}/{r}_piston_rec").as_str(), |b| b.iter(|| test(ptr_piston_rotate_rec::<usize>, l, p, r)));
    }
}

criterion_group!{
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.1).sample_size(500);
    targets = benchmark
}
criterion_main!(benches);
