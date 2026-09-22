use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use eigenon::engine::constants::{gate_h, identity};
use eigenon::engine::ops::{apply_gate_inplace, tensor_product};
use eigenon::engine::utils::q0_n;
use std::hint::black_box;

fn bench_single_gate(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantum_gate_application");
    group.sampling_mode(criterion::SamplingMode::Auto);
    for n in [4, 6, 8, 10, 12].iter() {
        let n = *n;
        let target = n / 2;
        let state = q0_n(n);
        let gate = gate_h();

        let mut full_matrix = identity().into_dyn();
        for i in 1..n {
            let op = if i == target { &gate } else { &identity() };
            full_matrix = tensor_product(op, &full_matrix);
        }
        let matrix_2d = full_matrix.into_dimensionality::<ndarray::Ix2>().unwrap();
        let state_vec = state.view().into_dimensionality::<ndarray::Ix1>().unwrap();

        group.bench_with_input(BenchmarkId::new("matrix_dot", n), &n, |b, _| {
            b.iter(|| black_box(matrix_2d.dot(&state_vec)))
        });

        group.bench_with_input(BenchmarkId::new("in-place", n), &n, |b, _| {
            b.iter(|| {
                let mut state_inplace = state.clone();
                apply_gate_inplace(&mut state_inplace, &gate, target);
                black_box(state_inplace)
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_single_gate);
criterion_main!(benches);
