use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use eigenon::engine::constants::{gate_cnot, identity};
use eigenon::engine::ops::{apply_cnot_inplace, tensor_product};
use eigenon::engine::utils::q0_n;
use std::hint::black_box;

fn bench_cnot_gate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cnot_gate_application");
    group.sampling_mode(criterion::SamplingMode::Auto);
    for n in [4, 6, 8, 10, 12].iter() {
        let n = *n;

        // CNOT on the two least-significant qubits:
        // control = 1, target = 0
        let control = 1;
        let target = 0;

        let state = q0_n(n);

        // Build full matrix: I ⊗ ... ⊗ I ⊗ CNOT
        let mut full_matrix = gate_cnot().into_dyn();
        for _ in 2..n {
            full_matrix = tensor_product(&identity(), &full_matrix);
        }

        let matrix_2d = full_matrix.into_dimensionality::<ndarray::Ix2>().unwrap();

        let state_vec = state.view().into_dimensionality::<ndarray::Ix1>().unwrap();

        group.bench_with_input(BenchmarkId::new("matrix_dot", n), &n, |b, _| {
            b.iter(|| black_box(matrix_2d.dot(&state_vec)))
        });

        group.bench_with_input(BenchmarkId::new("in-place", n), &n, |b, _| {
            b.iter(|| {
                let mut state_inplace = state.clone();
                apply_cnot_inplace(&mut state_inplace, control, target);
                black_box(state_inplace)
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_cnot_gate);
criterion_main!(benches);
