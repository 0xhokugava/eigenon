use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use eigenon::circuit::core::Circuit;
use eigenon::observable::{Hamiltonian, Pauli, PauliString};
use std::hint::black_box;

fn representative_state(n_qubits: usize) -> ndarray::ArrayD<num_complex::Complex64> {
    let mut circuit = Circuit::new(n_qubits);
    circuit.h_all();
    for qubit in 0..n_qubits.saturating_sub(1) {
        circuit.cnot(qubit, qubit + 1);
    }
    circuit.run()
}

fn bench_pauli_string_by_width(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("observable_pauli_string_width");

    for n_qubits in [4, 8, 12, 16] {
        let state = representative_state(n_qubits);
        let observable = PauliString::new([
            (0, Pauli::X),
            (n_qubits / 2, Pauli::Y),
            (n_qubits - 1, Pauli::Z),
        ])
        .unwrap();
        group.throughput(Throughput::Elements(state.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(n_qubits),
            &n_qubits,
            |bencher, &n_qubits| {
                bencher.iter(|| {
                    black_box(
                        observable
                            .expectation_value(black_box(&state), black_box(n_qubits))
                            .unwrap(),
                    )
                });
            },
        );
    }
    group.finish();
}

fn bench_hamiltonian_by_term_count(criterion: &mut Criterion) {
    const N_QUBITS: usize = 12;
    let state = representative_state(N_QUBITS);
    let mut group = criterion.benchmark_group("observable_hamiltonian_terms");

    for term_count in [1, 8, 32, 128] {
        let terms = (0..term_count).map(|index| {
            let first = index % N_QUBITS;
            let second = (index * 5 + 3) % N_QUBITS;
            let observable = if first == second {
                PauliString::single(first, Pauli::Z)
            } else {
                PauliString::new([(first, Pauli::X), (second, Pauli::Z)]).unwrap()
            };
            (1.0 / (index + 1) as f64, observable)
        });
        let hamiltonian = Hamiltonian::from_terms(terms).unwrap();
        group.throughput(Throughput::Elements((state.len() * term_count) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(term_count),
            &term_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        hamiltonian
                            .expectation_value(black_box(&state), black_box(N_QUBITS))
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
    bench_pauli_string_by_width,
    bench_hamiltonian_by_term_count
);
criterion_main!(benches);
