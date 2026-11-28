use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use eigenon::circuit::{Parameter, ParameterizedCircuit};
use eigenon::gradient::parameter_shift;
use eigenon::observable::{Hamiltonian, Pauli, PauliString};
use std::hint::black_box;

fn benchmark_hamiltonian(n_qubits: usize) -> Hamiltonian {
    Hamiltonian::from_terms((0..n_qubits).map(|qubit| {
        (
            1.0 / (qubit + 1) as f64,
            PauliString::single(qubit, Pauli::Z),
        )
    }))
    .unwrap()
}

fn bench_gradient_parameter_count(criterion: &mut Criterion) {
    const N_QUBITS: usize = 8;
    let observable = benchmark_hamiltonian(N_QUBITS);
    let mut group = criterion.benchmark_group("gradient_parameter_count");

    for parameter_count in [1, 2, 4, 8] {
        let parameters = (0..parameter_count)
            .map(|index| Parameter::new(format!("theta_{index}")))
            .collect::<Vec<_>>();
        let mut template = ParameterizedCircuit::new(N_QUBITS);
        for (qubit, parameter) in parameters.iter().enumerate() {
            template.h(qubit).ry(parameter.clone(), qubit);
        }
        for qubit in 0..N_QUBITS - 1 {
            template.cnot(qubit, qubit + 1);
        }
        let bindings = parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| (parameter, 0.1 * (index + 1) as f64))
            .collect::<Vec<_>>();
        group.throughput(Throughput::Elements(parameter_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(parameter_count),
            &parameter_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        parameter_shift(
                            black_box(&template),
                            black_box(&bindings),
                            black_box(&observable),
                            black_box(&parameters),
                        )
                        .unwrap(),
                    )
                });
            },
        );
    }
    group.finish();
}

fn bench_shared_parameter_occurrences(criterion: &mut Criterion) {
    let parameter = Parameter::new("theta");
    let observable = PauliString::single(0, Pauli::Z);
    let mut group = criterion.benchmark_group("gradient_shared_parameter_occurrences");

    for occurrences in [1, 4, 16] {
        let mut template = ParameterizedCircuit::new(1);
        for _ in 0..occurrences {
            template.ry(parameter.clone(), 0);
        }
        let bindings = [(&parameter, 0.17)];
        group.throughput(Throughput::Elements(occurrences as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(occurrences),
            &occurrences,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        parameter_shift(
                            black_box(&template),
                            black_box(&bindings),
                            black_box(&observable),
                            std::slice::from_ref(&parameter),
                        )
                        .unwrap(),
                    )
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_gradient_parameter_count,
    bench_shared_parameter_occurrences
);
criterion_main!(benches);
