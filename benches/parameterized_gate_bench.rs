use criterion::{
    BatchSize, BenchmarkId, Criterion, SamplingMode, Throughput, criterion_group, criterion_main,
};
use eigenon::circuit::core::Circuit;
use eigenon::engine::constants::{gate_h, gate_rx, gate_ry, gate_rz, gate_u};
use eigenon::engine::ops::apply_gate_inplace;
use eigenon::engine::utils::q0_n;
use ndarray::Array2;
use num_complex::Complex64;
use std::hint::black_box;

const THETA: f64 = 0.731;
const PHI: f64 = -0.417;
const LAMBDA: f64 = 1.193;

fn precomputed_gates() -> [(&'static str, Array2<Complex64>); 5] {
    [
        ("h_baseline", gate_h()),
        ("rx", gate_rx(THETA)),
        ("ry", gate_ry(THETA)),
        ("rz", gate_rz(THETA)),
        ("u", gate_u(THETA, PHI, LAMBDA)),
    ]
}

fn build_parameterized_circuit(n_qubits: usize, depth: usize, offset: usize) -> Circuit {
    let mut circuit = Circuit::new(n_qubits);

    for operation in 0..depth {
        let target = (operation * 5 + offset) % n_qubits;
        let angle = (operation + offset + 1) as f64 * 0.017;

        match operation % 4 {
            0 => {
                circuit.rx(angle, target);
            }
            1 => {
                circuit.ry(-angle, target);
            }
            2 => {
                circuit.rz(angle / 2.0, target);
            }
            _ => {
                circuit.u(angle, -angle / 3.0, angle / 5.0, target);
            }
        }
    }

    circuit
}

fn build_variational_circuit(n_qubits: usize, parameter_set: usize) -> Circuit {
    let mut circuit = Circuit::new(n_qubits);
    let base = (parameter_set + 1) as f64 * 0.013;

    for qubit in 0..n_qubits {
        let offset = qubit as f64 * 0.071;
        circuit
            .rx(base + offset, qubit)
            .ry(base * 0.5 - offset, qubit)
            .rz(-base + offset * 0.25, qubit)
            .u(base, offset, -base - offset, qubit);
    }

    for qubit in 0..n_qubits.saturating_sub(1) {
        circuit.cnot(qubit, qubit + 1);
    }

    circuit
}

fn bench_matrix_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("parameterized_matrix_construction");

    group.bench_function("h_baseline", |b| {
        b.iter(|| black_box(gate_h()));
    });
    group.bench_function("rx", |b| {
        b.iter(|| black_box(gate_rx(black_box(THETA))));
    });
    group.bench_function("ry", |b| {
        b.iter(|| black_box(gate_ry(black_box(THETA))));
    });
    group.bench_function("rz", |b| {
        b.iter(|| black_box(gate_rz(black_box(THETA))));
    });
    group.bench_function("u", |b| {
        b.iter(|| black_box(gate_u(black_box(THETA), black_box(PHI), black_box(LAMBDA))));
    });

    group.finish();
}

fn bench_application_by_width(c: &mut Criterion) {
    let mut group = c.benchmark_group("parameterized_gate_application_width");
    group.sampling_mode(SamplingMode::Auto);
    let gates = precomputed_gates();

    for n_qubits in [4, 8, 12, 16] {
        let state = q0_n(n_qubits);
        let target = n_qubits / 2;
        group.throughput(Throughput::Elements((1usize << n_qubits) as u64));

        for (name, gate) in &gates {
            group.bench_with_input(BenchmarkId::new(*name, n_qubits), &n_qubits, |b, _| {
                b.iter_batched_ref(
                    || state.clone(),
                    |working_state| {
                        apply_gate_inplace(working_state, black_box(gate), black_box(target));
                        black_box(&*working_state);
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }

    group.finish();
}

fn bench_application_by_target(c: &mut Criterion) {
    const N_QUBITS: usize = 12;

    let mut group = c.benchmark_group("parameterized_gate_application_target");
    group.sampling_mode(SamplingMode::Auto);
    group.throughput(Throughput::Elements((1usize << N_QUBITS) as u64));
    let state = q0_n(N_QUBITS);
    let gates = precomputed_gates();

    for target in [0, N_QUBITS / 2, N_QUBITS - 1] {
        for (name, gate) in &gates {
            group.bench_with_input(
                BenchmarkId::new(*name, format!("target_{target}")),
                &target,
                |b, &target| {
                    b.iter_batched_ref(
                        || state.clone(),
                        |working_state| {
                            apply_gate_inplace(working_state, black_box(gate), black_box(target));
                            black_box(&*working_state);
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }

    group.finish();
}

fn bench_circuit_execution(c: &mut Criterion) {
    let mut width_group = c.benchmark_group("parameterized_circuit_execution_width");
    const DEPTH: usize = 128;

    for n_qubits in [4, 8, 12, 16] {
        let circuit = build_parameterized_circuit(n_qubits, DEPTH, 0);
        width_group.throughput(Throughput::Elements(DEPTH as u64));
        width_group.bench_with_input(BenchmarkId::new("run", n_qubits), &n_qubits, |b, _| {
            b.iter(|| black_box(circuit.run()))
        });
    }
    width_group.finish();

    let mut depth_group = c.benchmark_group("parameterized_circuit_execution_depth");
    const N_QUBITS: usize = 8;

    for depth in [16, 128, 1_024] {
        let circuit = build_parameterized_circuit(N_QUBITS, depth, 0);
        depth_group.throughput(Throughput::Elements(depth as u64));
        depth_group.bench_with_input(BenchmarkId::new("run", depth), &depth, |b, _| {
            b.iter(|| black_box(circuit.run()));
        });
    }
    depth_group.finish();
}

fn bench_parameter_sweep(c: &mut Criterion) {
    const N_QUBITS: usize = 6;

    let mut group = c.benchmark_group("parameterized_parameter_sweep");

    for parameter_sets in [10, 100, 1_000] {
        group.throughput(Throughput::Elements(parameter_sets as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(parameter_sets),
            &parameter_sets,
            |b, &parameter_sets| {
                b.iter(|| {
                    for parameter_set in 0..parameter_sets {
                        let circuit = build_variational_circuit(N_QUBITS, parameter_set);
                        black_box(circuit.run());
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_matrix_construction,
    bench_application_by_width,
    bench_application_by_target,
    bench_circuit_execution,
    bench_parameter_sweep,
);
criterion_main!(benches);
