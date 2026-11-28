use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use eigenon::circuit::{Parameter, ParameterizedCircuit};
use eigenon::observable::{Hamiltonian, Pauli, PauliString};
use eigenon::variational::{GradientDescent, VqeProblem};
use std::hint::black_box;

fn vqe_problem(parameter_count: usize) -> (Vec<Parameter>, VqeProblem) {
    let parameters = (0..parameter_count)
        .map(|index| Parameter::new(format!("theta_{index}")))
        .collect::<Vec<_>>();
    let mut template = ParameterizedCircuit::new(parameter_count);
    for (qubit, parameter) in parameters.iter().enumerate() {
        template.ry(parameter.clone(), qubit);
    }
    for qubit in 0..parameter_count.saturating_sub(1) {
        template.cnot(qubit, qubit + 1);
    }
    let hamiltonian = Hamiltonian::from_terms(
        (0..parameter_count).map(|qubit| (1.0, PauliString::single(qubit, Pauli::Z))),
    )
    .unwrap();
    (parameters, VqeProblem::new(template, hamiltonian))
}

fn bench_vqe_iterations(criterion: &mut Criterion) {
    let (parameters, problem) = vqe_problem(4);
    let bindings = parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter, 0.1 * (index + 1) as f64))
        .collect::<Vec<_>>();
    let mut group = criterion.benchmark_group("variational_vqe_iterations");

    for iterations in [1, 5, 20] {
        let optimizer = GradientDescent::new(0.05)
            .max_iterations(iterations)
            .convergence_tolerance(0.0);
        group.throughput(Throughput::Elements(iterations as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(iterations),
            &iterations,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        problem
                            .minimize(black_box(optimizer), black_box(bindings.iter().copied()))
                            .unwrap(),
                    )
                });
            },
        );
    }
    group.finish();
}

fn bench_vqe_parameter_count(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("variational_vqe_parameter_count");

    for parameter_count in [1, 2, 4, 8] {
        let (parameters, problem) = vqe_problem(parameter_count);
        let bindings = parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| (parameter, 0.1 * (index + 1) as f64))
            .collect::<Vec<_>>();
        let optimizer = GradientDescent::new(0.05)
            .max_iterations(1)
            .convergence_tolerance(0.0);
        group.throughput(Throughput::Elements(parameter_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(parameter_count),
            &parameter_count,
            |bencher, _| {
                bencher.iter(|| {
                    black_box(
                        problem
                            .minimize(black_box(optimizer), black_box(bindings.iter().copied()))
                            .unwrap(),
                    )
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_vqe_iterations, bench_vqe_parameter_count);
criterion_main!(benches);
