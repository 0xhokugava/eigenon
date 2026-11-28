use eigenon::circuit::core::Circuit;
use eigenon::observable::{Hamiltonian, ObservableError, Pauli, PauliString};
use ndarray::{ArrayD, IxDyn, array};
use num_complex::Complex64;
use std::f64::consts::FRAC_1_SQRT_2;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "expected {expected}, found {actual}"
    );
}

#[test]
fn single_qubit_paulis_match_analytic_eigenstates() {
    let plus = array![FRAC_1_SQRT_2, FRAC_1_SQRT_2]
        .mapv(|value| Complex64::new(value, 0.0))
        .into_dyn();
    let plus_i = array![
        Complex64::new(FRAC_1_SQRT_2, 0.0),
        Complex64::new(0.0, FRAC_1_SQRT_2),
    ]
    .into_dyn();
    let zero = array![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)].into_dyn();

    assert_close(
        PauliString::single(0, Pauli::X)
            .expectation_value(&plus, 1)
            .unwrap(),
        1.0,
    );
    assert_close(
        PauliString::single(0, Pauli::Y)
            .expectation_value(&plus_i, 1)
            .unwrap(),
        1.0,
    );
    assert_close(
        PauliString::single(0, Pauli::Z)
            .expectation_value(&zero, 1)
            .unwrap(),
        1.0,
    );
}

#[test]
fn bell_state_has_expected_correlations() {
    let mut circuit = Circuit::new(2);
    let state = circuit.h(0).cnot(0, 1).run();

    let xx = PauliString::new([(0, Pauli::X), (1, Pauli::X)]).unwrap();
    let yy = PauliString::new([(0, Pauli::Y), (1, Pauli::Y)]).unwrap();
    let zz = PauliString::new([(0, Pauli::Z), (1, Pauli::Z)]).unwrap();
    let z0 = PauliString::single(0, Pauli::Z);

    assert_close(xx.expectation_value(&state, 2).unwrap(), 1.0);
    assert_close(yy.expectation_value(&state, 2).unwrap(), -1.0);
    assert_close(zz.expectation_value(&state, 2).unwrap(), 1.0);
    assert_close(z0.expectation_value(&state, 2).unwrap(), 0.0);
}

#[test]
fn hamiltonian_combines_terms_without_dense_matrices() {
    let mut circuit = Circuit::new(2);
    let state = circuit.h(0).cnot(0, 1).run();
    let hamiltonian = Hamiltonian::from_terms([
        (0.25, PauliString::identity()),
        (
            -1.0,
            PauliString::new([(0, Pauli::Z), (1, Pauli::Z)]).unwrap(),
        ),
        (-0.5, PauliString::single(0, Pauli::X)),
    ])
    .unwrap();

    assert_close(hamiltonian.expectation_value(&state, 2).unwrap(), -0.75);
    assert_eq!(hamiltonian.terms().len(), 3);
    assert_close(hamiltonian.terms()[0].coefficient(), 0.25);
}

#[test]
fn pauli_operators_are_sorted_and_duplicate_qubits_are_rejected() {
    let string = PauliString::new([(3, Pauli::X), (0, Pauli::Z), (2, Pauli::Y)]).unwrap();
    assert_eq!(
        string.operators(),
        &[(0, Pauli::Z), (2, Pauli::Y), (3, Pauli::X)]
    );

    assert_eq!(
        PauliString::new([(1, Pauli::X), (1, Pauli::Z)]),
        Err(ObservableError::DuplicateQubit(1))
    );
}

#[test]
fn little_endian_qubit_indices_are_explicit() {
    let mut circuit = Circuit::new(3);
    let state = circuit.x(0).x(2).run();

    assert_close(
        PauliString::single(0, Pauli::Z)
            .expectation_value(&state, 3)
            .unwrap(),
        -1.0,
    );
    assert_close(
        PauliString::single(1, Pauli::Z)
            .expectation_value(&state, 3)
            .unwrap(),
        1.0,
    );
    assert_close(
        PauliString::single(2, Pauli::Z)
            .expectation_value(&state, 3)
            .unwrap(),
        -1.0,
    );
}

#[test]
fn identity_expectation_is_the_state_norm() {
    let state = array![Complex64::new(2.0, 0.0), Complex64::new(0.0, -1.0),].into_dyn();

    assert_close(
        PauliString::identity()
            .expectation_value(&state, 1)
            .unwrap(),
        5.0,
    );
}

#[test]
fn evaluation_validates_qubits_dimensions_shape_and_finite_values() {
    let one_qubit = array![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)].into_dyn();
    let invalid_qubit = PauliString::single(1, Pauli::Z);
    assert_eq!(
        invalid_qubit.expectation_value(&one_qubit, 1),
        Err(ObservableError::QubitOutOfRange {
            qubit: 1,
            n_qubits: 1,
        })
    );

    let wrong_length = ArrayD::from_elem(IxDyn(&[3]), Complex64::new(0.0, 0.0));
    assert_eq!(
        PauliString::identity().expectation_value(&wrong_length, 1),
        Err(ObservableError::StateDimensionMismatch {
            expected: 2,
            actual: 3,
        })
    );

    let matrix = ArrayD::from_elem(IxDyn(&[2, 2]), Complex64::new(0.0, 0.0));
    assert_eq!(
        PauliString::identity().expectation_value(&matrix, 2),
        Err(ObservableError::StateMustBeVector)
    );

    let non_finite = array![Complex64::new(f64::NAN, 0.0), Complex64::new(0.0, 0.0),].into_dyn();
    assert_eq!(
        PauliString::identity().expectation_value(&non_finite, 1),
        Err(ObservableError::NonFiniteAmplitude { index: 0 })
    );
}

#[test]
fn hamiltonian_rejects_non_finite_coefficients() {
    assert_eq!(
        Hamiltonian::from_terms([(f64::INFINITY, PauliString::identity())]),
        Err(ObservableError::NonFiniteCoefficient)
    );
}

#[test]
fn imaginary_tolerance_must_be_finite_and_non_negative() {
    let state = array![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)].into_dyn();
    let observable = PauliString::single(0, Pauli::Z);

    for tolerance in [-1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            observable.expectation_value_with_tolerance(&state, 1, tolerance),
            Err(ObservableError::InvalidImaginaryTolerance)
        );
    }
    assert_close(
        observable
            .expectation_value_with_tolerance(&state, 1, 0.0)
            .unwrap(),
        1.0,
    );
}

#[test]
fn pauli_accumulation_rejects_non_finite_result() {
    let state = array![Complex64::new(f64::MAX, 0.0), Complex64::new(f64::MAX, 0.0),].into_dyn();

    assert_eq!(
        PauliString::single(0, Pauli::Y).expectation_value(&state, 1),
        Err(ObservableError::NonFiniteExpectation)
    );
}

#[test]
fn hamiltonian_accumulation_rejects_non_finite_result() {
    let state = array![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)].into_dyn();
    let hamiltonian = Hamiltonian::from_terms([
        (f64::MAX, PauliString::identity()),
        (f64::MAX, PauliString::identity()),
    ])
    .unwrap();

    assert_eq!(
        hamiltonian.expectation_value(&state, 1),
        Err(ObservableError::NonFiniteExpectation)
    );
}
