use eigenon::circuit::core::Circuit;
use eigenon::circuit::{
    BindingError, Parameter, ParameterValue, ParameterizedCircuit, ParameterizedGateKind,
    ParameterizedOperation,
};
use eigenon::engine::utils::assert_states_close;

fn binding_error(result: Result<Circuit, BindingError>) -> BindingError {
    match result {
        Ok(_) => panic!("binding unexpectedly succeeded"),
        Err(error) => error,
    }
}

#[test]
fn named_parameter_can_be_rebound_without_changing_the_template() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(theta.clone(), 0);

    assert_eq!(template.parameters(), std::slice::from_ref(&theta));

    let first = template.bind([(&theta, 0.5)]).unwrap();
    let second = template.bind([(&theta, 1.0)]).unwrap();

    let mut expected_first = Circuit::new(1);
    expected_first.ry(0.5, 0);
    let mut expected_second = Circuit::new(1);
    expected_second.ry(1.0, 0);

    assert_states_close(&first.run(), &expected_first.run());
    assert_states_close(&second.run(), &expected_second.run());
    assert_eq!(template.parameters(), &[theta]);
}

#[test]
fn parameters_are_unique_and_ordered_by_first_use() {
    let theta = Parameter::new("theta");
    let phi = Parameter::new("phi");
    let mut template = ParameterizedCircuit::new(2);

    template
        .rz(theta.clone(), 1)
        .h(0)
        .rx(phi.clone(), 0)
        .ry(theta.clone(), 1)
        .cnot(0, 1);

    assert_eq!(template.parameters(), &[theta.clone(), phi.clone()]);

    let bound = template.bind([(&theta, 0.4), (&phi, -0.2)]).unwrap();
    let mut numeric = Circuit::new(2);
    numeric.rz(0.4, 1).h(0).rx(-0.2, 0).ry(0.4, 1).cnot(0, 1);
    assert_states_close(&bound.run(), &numeric.run());

    assert_eq!(
        template.operations(),
        &[
            ParameterizedOperation::SingleQubit {
                gate: ParameterizedGateKind::Rz(ParameterValue::Symbol(theta.clone())),
                target: 1,
            },
            ParameterizedOperation::SingleQubit {
                gate: ParameterizedGateKind::H,
                target: 0,
            },
            ParameterizedOperation::SingleQubit {
                gate: ParameterizedGateKind::Rx(ParameterValue::Symbol(phi)),
                target: 0,
            },
            ParameterizedOperation::SingleQubit {
                gate: ParameterizedGateKind::Ry(ParameterValue::Symbol(theta)),
                target: 1,
            },
            ParameterizedOperation::Cnot {
                control: 0,
                target: 1,
            },
        ]
    );
}

#[test]
fn binding_order_does_not_change_operation_order_or_result() {
    let alpha = Parameter::new("alpha");
    let beta = Parameter::new("beta");
    let mut template = ParameterizedCircuit::new(1);
    template.rx(alpha.clone(), 0).rz(beta.clone(), 0);

    let forward = template.bind([(&alpha, 0.2), (&beta, -0.7)]).unwrap();
    let reverse = template.bind([(&beta, -0.7), (&alpha, 0.2)]).unwrap();

    assert_eq!(forward.operations(), reverse.operations());
    assert_states_close(&forward.run(), &reverse.run());
}

#[test]
fn binding_reports_missing_duplicate_unknown_and_non_finite_values() {
    let theta = Parameter::new("theta");
    let other = Parameter::new("other");
    let mut template = ParameterizedCircuit::new(1);
    template.rx(theta.clone(), 0);

    assert_eq!(
        binding_error(template.bind(std::iter::empty::<(&Parameter, f64)>())),
        BindingError::MissingParameter(theta.clone())
    );
    assert_eq!(
        binding_error(template.bind([(&theta, 0.1), (&theta, 0.2)])),
        BindingError::DuplicateBinding(theta.clone())
    );
    assert_eq!(
        binding_error(template.bind([(&other, 0.1)])),
        BindingError::UnknownParameter(other)
    );
    assert_eq!(
        binding_error(template.bind([(&theta, f64::NAN)])),
        BindingError::NonFiniteBinding(theta)
    );
}

#[test]
fn independently_created_parameters_with_the_same_name_share_identity() {
    let operation_parameter = Parameter::new("theta");
    let binding_parameter = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(1);
    template.rz(operation_parameter, 0);

    assert!(template.bind([(&binding_parameter, 0.25)]).is_ok());
}

#[test]
fn fixed_angles_and_register_metadata_survive_binding() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::with_classical_bits(2, 3);
    template
        .x(0)
        .rx(0.125, 1)
        .ry(&theta, 1)
        .cz(0, 1)
        .measure(1, 2);

    let circuit = template.bind([(&theta, -0.5)]).unwrap();

    assert_eq!(circuit.n_qubits(), 2);
    assert_eq!(circuit.n_classical_bits(), 3);
    assert_eq!(circuit.operations().len(), template.operations().len());
}

#[test]
fn bound_circuit_preserves_little_endian_qubit_ordering() {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(2);
    template.x(0).rz(theta.clone(), 1);

    let state = template.bind([(&theta, 0.0)]).unwrap().run();

    assert!(state[1].norm() > 1.0 - 1e-12);
    assert!(state[0].norm() < 1e-12);
    assert!(state[2].norm() < 1e-12);
    assert!(state[3].norm() < 1e-12);
}

#[test]
#[should_panic(expected = "parameter name must not be empty")]
fn parameter_rejects_blank_name() {
    Parameter::new("   ");
}

#[test]
#[should_panic(expected = "Rx angle must be finite")]
fn template_rejects_non_finite_fixed_angle() {
    ParameterizedCircuit::new(1).rx(f64::INFINITY, 0);
}
