use eigenon::circuit::{BindingError, Parameter, ParameterizedCircuit};
use eigenon::observable::{Hamiltonian, Pauli, PauliString};
use eigenon::variational::{
    GradientDescent, IterationRecord, Optimizer, VariationalError, VariationalResult, VqeProblem,
};
use std::collections::BTreeMap;

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected}, found {actual}"
    );
}

fn one_qubit_problem() -> (Parameter, VqeProblem) {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(theta.clone(), 0);
    let hamiltonian = Hamiltonian::from_terms([(1.0, PauliString::single(0, Pauli::Z))]).unwrap();
    (theta, VqeProblem::new(template, hamiltonian))
}

#[test]
fn one_qubit_vqe_converges_to_known_ground_energy() {
    let (theta, problem) = one_qubit_problem();
    let optimizer = GradientDescent::new(0.2)
        .max_iterations(200)
        .convergence_tolerance(1e-10);

    let result = problem.minimize(optimizer, [(&theta, 0.25)]).unwrap();

    assert!(result.converged);
    assert_close(result.minimum, -1.0, 1e-12);
    assert_close(result.parameters[&theta].cos(), -1.0, 1e-12);
    assert!(result.history.len() < 200);
    assert_eq!(result.history[0].iteration, 0);
    assert_close(result.history[0].energy, 0.25_f64.cos(), 1e-12);
}

#[test]
fn entangled_two_qubit_vqe_respects_little_endian_qubit_indices() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(2);
    template.ry(theta.clone(), 0).cnot(0, 1);
    let hamiltonian = Hamiltonian::from_terms([
        (1.0, PauliString::single(0, Pauli::Z)),
        (1.0, PauliString::single(1, Pauli::Z)),
    ])
    .unwrap();
    let problem = VqeProblem::new(template, hamiltonian);

    let result = problem
        .minimize(
            GradientDescent::new(0.1)
                .max_iterations(300)
                .convergence_tolerance(1e-10),
            [(&theta, 0.3)],
        )
        .unwrap();

    assert!(result.converged);
    assert_close(result.minimum, -2.0, 1e-12);
}

#[test]
fn repeated_runs_are_exactly_reproducible() {
    let (theta, problem) = one_qubit_problem();
    let optimizer = GradientDescent::new(0.15)
        .max_iterations(80)
        .convergence_tolerance(1e-9);

    let first = problem.minimize(optimizer, [(&theta, 0.7)]).unwrap();
    let second = problem.minimize(optimizer, [(&theta, 0.7)]).unwrap();

    assert_eq!(first, second);
}

#[test]
fn history_records_reproducible_energies_and_gradients() {
    let theta = Parameter::new("theta");
    let phi = Parameter::new("phi");
    let mut template = ParameterizedCircuit::new(2);
    template.ry(theta.clone(), 0).rx(phi.clone(), 1).cnot(0, 1);
    let hamiltonian = Hamiltonian::from_terms([
        (0.7, PauliString::single(0, Pauli::Z)),
        (
            -0.4,
            PauliString::new([(0, Pauli::X), (1, Pauli::X)]).unwrap(),
        ),
    ])
    .unwrap();
    let problem = VqeProblem::new(template, hamiltonian);

    let result = problem
        .minimize(
            GradientDescent::new(0.05)
                .max_iterations(6)
                .convergence_tolerance(0.0),
            [(&theta, 0.4), (&phi, -0.2)],
        )
        .unwrap();

    assert_eq!(result.history.len(), 6);
    for (iteration, record) in result.history.iter().enumerate() {
        assert_eq!(record.iteration, iteration);
        assert_eq!(problem.energy(&record.parameters).unwrap(), record.energy);
        assert_eq!(
            problem.gradients(&record.parameters).unwrap(),
            record.gradients
        );
    }
}

#[test]
fn maximum_iteration_limit_returns_best_observed_point() {
    let (theta, problem) = one_qubit_problem();
    let result = problem
        .minimize(
            GradientDescent::new(0.01)
                .max_iterations(3)
                .convergence_tolerance(0.0),
            [(&theta, 0.3)],
        )
        .unwrap();

    assert!(!result.converged);
    assert_eq!(result.history.len(), 3);
    assert_eq!(result.minimum, result.history[2].energy);
    assert_eq!(result.parameters, result.history[2].parameters);
}

#[test]
fn stationary_initial_point_converges_without_an_update() {
    let (theta, problem) = one_qubit_problem();

    let result = problem
        .minimize(GradientDescent::new(0.2), [(&theta, std::f64::consts::PI)])
        .unwrap();

    assert!(result.converged);
    assert_eq!(result.history.len(), 1);
    assert_close(result.minimum, -1.0, 1e-12);
    assert_close(result.history[0].gradients[&theta], 0.0, 1e-12);
}

#[test]
fn optimizer_configuration_is_validated_before_evaluation() {
    let (theta, problem) = one_qubit_problem();

    for optimizer in [
        GradientDescent::new(0.0),
        GradientDescent::new(f64::NAN),
        GradientDescent::new(f64::INFINITY),
    ] {
        assert_eq!(
            problem.minimize(optimizer, [(&theta, 0.2)]),
            Err(VariationalError::InvalidLearningRate)
        );
    }
    assert_eq!(
        problem.minimize(GradientDescent::new(0.1).max_iterations(0), [(&theta, 0.2)]),
        Err(VariationalError::InvalidMaximumIterations)
    );
    for tolerance in [-1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            problem.minimize(
                GradientDescent::new(0.1).convergence_tolerance(tolerance),
                [(&theta, 0.2)]
            ),
            Err(VariationalError::InvalidConvergenceTolerance)
        );
    }
}

#[test]
fn initial_binding_errors_remain_typed() {
    let (theta, problem) = one_qubit_problem();
    let unknown = Parameter::new("unknown");
    let optimizer = GradientDescent::new(0.1);

    assert!(matches!(
        problem.minimize(optimizer, []),
        Err(VariationalError::Binding(BindingError::MissingParameter(parameter)))
            if parameter == theta
    ));
    assert!(matches!(
        problem.minimize(optimizer, [(&theta, 0.1), (&theta, 0.2)]),
        Err(VariationalError::Binding(BindingError::DuplicateBinding(parameter)))
            if parameter == theta
    ));
    assert!(matches!(
        problem.minimize(optimizer, [(&theta, f64::NAN)]),
        Err(VariationalError::Binding(BindingError::NonFiniteBinding(parameter)))
            if parameter == theta
    ));
    assert!(matches!(
        problem.minimize(optimizer, [(&theta, 0.1), (&unknown, 0.2)]),
        Err(VariationalError::Binding(BindingError::UnknownParameter(parameter)))
            if parameter == unknown
    ));
}

#[test]
fn measurement_is_rejected_by_both_public_objective_methods() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::with_classical_bits(1, 1);
    template.ry(theta.clone(), 0).measure(0, 0);
    let hamiltonian = Hamiltonian::from_terms([(1.0, PauliString::single(0, Pauli::Z))]).unwrap();
    let problem = VqeProblem::new(template, hamiltonian);

    let bindings = BTreeMap::from([(theta.clone(), std::f64::consts::FRAC_PI_2)]);

    assert_eq!(
        problem.energy(&bindings),
        Err(VariationalError::MeasurementUnsupported)
    );
    assert_eq!(
        problem.gradients(&bindings),
        Err(VariationalError::MeasurementUnsupported)
    );
    assert_eq!(
        problem.minimize(GradientDescent::new(0.1), [(&theta, 0.2)]),
        Err(VariationalError::MeasurementUnsupported)
    );
}

#[test]
fn huge_iteration_limit_does_not_preallocate_unbounded_history() {
    let (theta, problem) = one_qubit_problem();

    let result = problem
        .minimize(
            GradientDescent::new(0.2).max_iterations(usize::MAX),
            [(&theta, std::f64::consts::PI)],
        )
        .unwrap();

    assert!(result.converged);
    assert_eq!(result.history.len(), 1);
}

#[test]
fn non_finite_parameter_updates_are_rejected() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(theta.clone(), 0);
    let hamiltonian =
        Hamiltonian::from_terms([(f64::MAX, PauliString::single(0, Pauli::Z))]).unwrap();
    let problem = VqeProblem::new(template, hamiltonian);

    assert!(matches!(
        problem.minimize(
            GradientDescent::new(4.0).max_iterations(2),
            [(&theta, std::f64::consts::FRAC_PI_2)]
        ),
        Err(VariationalError::NonFiniteParameterUpdate(parameter)) if parameter == theta
    ));
}

#[derive(Clone, Copy)]
struct EvaluateOnce;

impl Optimizer for EvaluateOnce {
    fn optimize(
        &self,
        problem: &VqeProblem,
        initial: BTreeMap<Parameter, f64>,
    ) -> Result<VariationalResult, VariationalError> {
        let energy = problem.energy(&initial)?;
        let gradients = problem.gradients(&initial)?;
        Ok(VariationalResult {
            minimum: energy,
            parameters: initial.clone(),
            history: vec![IterationRecord {
                iteration: 0,
                parameters: initial,
                energy,
                gradients,
            }],
            converged: false,
        })
    }
}

#[test]
fn custom_optimizers_can_use_the_public_objective_interface() {
    let (theta, problem) = one_qubit_problem();

    let result = problem.minimize(EvaluateOnce, [(&theta, 0.4)]).unwrap();

    assert_eq!(result.history.len(), 1);
    assert_close(result.minimum, 0.4_f64.cos(), 1e-12);
    assert_close(result.history[0].gradients[&theta], -0.4_f64.sin(), 1e-12);
}

#[test]
fn parameter_and_history_order_are_stable_by_name() {
    let zeta = Parameter::new("zeta");
    let alpha = Parameter::new("alpha");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(zeta.clone(), 0).rx(alpha.clone(), 0);
    let hamiltonian = Hamiltonian::from_terms([(1.0, PauliString::single(0, Pauli::Z))]).unwrap();
    let problem = VqeProblem::new(template, hamiltonian);

    assert_eq!(problem.parameters(), &[zeta.clone(), alpha.clone()]);
    let result = problem
        .minimize(
            GradientDescent::new(0.1).max_iterations(1),
            [(&zeta, 0.2), (&alpha, 0.3)],
        )
        .unwrap();
    let names = result.history[0]
        .parameters
        .keys()
        .map(Parameter::name)
        .collect::<Vec<_>>();

    assert_eq!(names, ["alpha", "zeta"]);
}
