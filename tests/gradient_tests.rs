use eigenon::circuit::{BindingError, Parameter, ParameterizedCircuit};
use eigenon::gradient::{GradientError, parameter_shift};
use eigenon::observable::{Hamiltonian, ObservableError, Pauli, PauliString};

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected}, found {actual}"
    );
}

#[test]
fn rx_ry_and_rz_gradients_match_analytic_results() {
    let theta = Parameter::new("theta");
    let z = PauliString::single(0, Pauli::Z);
    let x = PauliString::single(0, Pauli::X);
    let angle = 0.37;

    let mut rx = ParameterizedCircuit::new(1);
    rx.rx(theta.clone(), 0);
    let rx_gradient =
        parameter_shift(&rx, &[(&theta, angle)], &z, std::slice::from_ref(&theta)).unwrap();
    assert_close(rx_gradient[&theta], -angle.sin(), 1e-12);

    let mut ry = ParameterizedCircuit::new(1);
    ry.ry(theta.clone(), 0);
    let ry_gradient =
        parameter_shift(&ry, &[(&theta, angle)], &z, std::slice::from_ref(&theta)).unwrap();
    assert_close(ry_gradient[&theta], -angle.sin(), 1e-12);

    let mut rz = ParameterizedCircuit::new(1);
    rz.h(0).rz(theta.clone(), 0);
    let rz_gradient =
        parameter_shift(&rz, &[(&theta, angle)], &x, std::slice::from_ref(&theta)).unwrap();
    assert_close(rz_gradient[&theta], -angle.sin(), 1e-12);
}

#[test]
fn shared_parameter_accumulates_each_gate_contribution() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(theta.clone(), 0).ry(theta.clone(), 0);
    let z = PauliString::single(0, Pauli::Z);
    let angle = 0.41;

    let gradients = parameter_shift(
        &template,
        &[(&theta, angle)],
        &z,
        std::slice::from_ref(&theta),
    )
    .unwrap();

    assert_close(gradients[&theta], -2.0 * (2.0 * angle).sin(), 1e-12);
}

#[test]
fn multiple_parameter_hamiltonian_gradients_match_central_differences() {
    let theta = Parameter::new("theta");
    let phi = Parameter::new("phi");
    let mut template = ParameterizedCircuit::new(2);
    template
        .h(0)
        .ry(theta.clone(), 0)
        .cnot(0, 1)
        .rx(phi.clone(), 1)
        .rz(theta.clone(), 1);
    let hamiltonian = Hamiltonian::from_terms([
        (
            -0.8,
            PauliString::new([(0, Pauli::Z), (1, Pauli::Z)]).unwrap(),
        ),
        (0.3, PauliString::single(0, Pauli::X)),
        (-0.2, PauliString::single(1, Pauli::Y)),
    ])
    .unwrap();
    let theta_value = 0.23;
    let phi_value = -0.51;

    let gradients = parameter_shift(
        &template,
        &[(&theta, theta_value), (&phi, phi_value)],
        &hamiltonian,
        &[theta.clone(), phi.clone()],
    )
    .unwrap();

    let epsilon = 1e-6;
    let energy = |theta_binding: f64, phi_binding: f64| {
        let circuit = template
            .bind([(&theta, theta_binding), (&phi, phi_binding)])
            .unwrap();
        hamiltonian
            .expectation_value(&circuit.run(), circuit.n_qubits())
            .unwrap()
    };
    let theta_difference = (energy(theta_value + epsilon, phi_value)
        - energy(theta_value - epsilon, phi_value))
        / (2.0 * epsilon);
    let phi_difference = (energy(theta_value, phi_value + epsilon)
        - energy(theta_value, phi_value - epsilon))
        / (2.0 * epsilon);

    assert_close(gradients[&theta], theta_difference, 1e-8);
    assert_close(gradients[&phi], phi_difference, 1e-8);
}

#[test]
fn gradient_map_order_is_deterministic_by_parameter_name() {
    let zeta = Parameter::new("zeta");
    let alpha = Parameter::new("alpha");
    let mut template = ParameterizedCircuit::new(1);
    template.rx(zeta.clone(), 0).ry(alpha.clone(), 0);
    let observable = PauliString::single(0, Pauli::Z);

    let gradients = parameter_shift(
        &template,
        &[(&zeta, 0.1), (&alpha, 0.2)],
        &observable,
        &[zeta.clone(), alpha.clone()],
    )
    .unwrap();
    let names = gradients.keys().map(Parameter::name).collect::<Vec<_>>();

    assert_eq!(names, ["alpha", "zeta"]);
}

#[test]
fn subset_gradient_still_requires_complete_bindings() {
    let theta = Parameter::new("theta");
    let phi = Parameter::new("phi");
    let mut template = ParameterizedCircuit::new(1);
    template.rx(theta.clone(), 0).ry(phi.clone(), 0);
    let observable = PauliString::single(0, Pauli::Z);

    assert!(matches!(
        parameter_shift(
            &template,
            &[(&theta, 0.1)],
            &observable,
            std::slice::from_ref(&theta)
        ),
        Err(GradientError::Binding(BindingError::MissingParameter(parameter)))
            if parameter == phi
    ));
}

#[test]
fn requested_parameters_must_be_known_and_unique() {
    let theta = Parameter::new("theta");
    let unknown = Parameter::new("unknown");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(theta.clone(), 0);
    let observable = PauliString::single(0, Pauli::Z);

    assert!(matches!(
        parameter_shift(
            &template,
            &[(&theta, 0.2)],
            &observable,
            std::slice::from_ref(&unknown)
        ),
        Err(GradientError::UnknownParameter(parameter)) if parameter == unknown
    ));
    assert!(matches!(
        parameter_shift(
            &template,
            &[(&theta, 0.2)],
            &observable,
            &[theta.clone(), theta.clone()]
        ),
        Err(GradientError::DuplicateParameter(parameter)) if parameter == theta
    ));
}

#[test]
fn measurement_is_rejected_before_stochastic_execution() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::with_classical_bits(1, 1);
    template.ry(theta.clone(), 0).measure(0, 0);
    let observable = PauliString::single(0, Pauli::Z);

    assert!(matches!(
        parameter_shift(
            &template,
            &[(&theta, 0.2)],
            &observable,
            std::slice::from_ref(&theta)
        ),
        Err(GradientError::MeasurementUnsupported)
    ));
}

#[test]
fn observable_errors_are_preserved() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(theta.clone(), 0);
    let invalid = PauliString::single(1, Pauli::Z);

    assert!(matches!(
        parameter_shift(
            &template,
            &[(&theta, 0.2)],
            &invalid,
            std::slice::from_ref(&theta)
        ),
        Err(GradientError::Observable(
            ObservableError::QubitOutOfRange {
                qubit: 1,
                n_qubits: 1
            }
        ))
    ));
}

#[test]
fn scaling_is_applied_before_subtraction_to_avoid_intermediate_overflow() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(theta.clone(), 0);
    let hamiltonian =
        Hamiltonian::from_terms([(f64::MAX, PauliString::single(0, Pauli::Z))]).unwrap();

    let gradients = parameter_shift(
        &template,
        &[(&theta, std::f64::consts::FRAC_PI_2)],
        &hamiltonian,
        std::slice::from_ref(&theta),
    )
    .unwrap();

    assert!(gradients[&theta].is_finite());
    assert_eq!(gradients[&theta], -f64::MAX);
}

#[test]
fn gradient_accumulation_rejects_truly_non_finite_sum() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(theta.clone(), 0).ry(theta.clone(), 0);
    let hamiltonian =
        Hamiltonian::from_terms([(f64::MAX, PauliString::single(0, Pauli::Z))]).unwrap();

    assert!(matches!(
        parameter_shift(
            &template,
            &[(&theta, std::f64::consts::FRAC_PI_4)],
            &hamiltonian,
            std::slice::from_ref(&theta)
        ),
        Err(GradientError::NonFiniteGradient(parameter)) if parameter == theta
    ));
}

#[test]
fn unrepresentable_parameter_shift_is_rejected_explicitly() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(theta.clone(), 0);
    let observable = PauliString::single(0, Pauli::Z);

    assert!(matches!(
        parameter_shift(
            &template,
            &[(&theta, f64::MAX)],
            &observable,
            std::slice::from_ref(&theta)
        ),
        Err(GradientError::ShiftNotRepresentable(parameter)) if parameter == theta
    ));

    assert!(matches!(
        parameter_shift(
            &template,
            &[(&theta, 2_f64.powi(52))],
            &observable,
            std::slice::from_ref(&theta)
        ),
        Err(GradientError::ShiftNotRepresentable(parameter)) if parameter == theta
    ));
}
