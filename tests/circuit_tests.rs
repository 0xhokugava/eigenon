use eigenon::circuit::catalog;
use eigenon::circuit::catalog::BellState;
use eigenon::circuit::core::Circuit;
use eigenon::circuit::operation::{GateKind, Operation};
use eigenon::engine::constants::{q0, q1};
use eigenon::engine::utils::assert_states_close;
use ndarray::ArrayD;
use num_complex::Complex64;

#[test]
fn test_circuit_bell_state() {
    let circuit = catalog::bell(BellState::PhiPlus);
    let result = circuit.run();

    let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();

    let expected = ArrayD::from_shape_vec(
        ndarray::IxDyn(&[4]),
        vec![
            Complex64::new(inv_sqrt2, 0.0), // |00>
            Complex64::new(0.0, 0.0),       // |01>
            Complex64::new(0.0, 0.0),       // |10>
            Complex64::new(inv_sqrt2, 0.0), // |11>
        ],
    )
    .unwrap();

    assert_states_close(&result, &expected);
}
#[test]
fn test_circuit_applies_operations_in_order() {
    let mut circuit = Circuit::new(1);
    let result = circuit.h(0).h(0).run();
    let expected = q0().into_dyn();
    assert_states_close(&result, &expected);
}
#[test]
fn test_circuit_x_gate() {
    let mut circuit = Circuit::new(1);
    let result = circuit.x(0).run();
    let expected = q1().into_dyn();
    assert_states_close(&result, &expected);
}
#[test]
fn test_circuit_chained_calls() {
    let mut circuit = Circuit::new(1);
    let result = circuit.x(0).x(0).run();
    let expected = q0().into_dyn();
    assert_states_close(&result, &expected);
}
#[test]
fn test_cz_basis_state() {
    let mut circuit = Circuit::new(2);
    circuit.x(0).x(1).cz(0, 1).run();
}
#[test]
fn test_cz_superposition() {
    let mut circuit = Circuit::new(2);
    circuit.h(0).h(1).cz(0, 1).run();
}

#[test]
fn mcx_applies_when_controls_are_one() {
    let mut circuit = Circuit::new(3);
    circuit.x(0).x(1).mcx(&[0, 1], 2);
    let state = circuit.run();
    assert_eq!(state[7], Complex64::new(1.0, 0.0));
}

#[test]
fn mcz_applies_phase_when_controls_are_one() {
    let mut circuit = Circuit::new(3);
    circuit.x(0).x(1).x(2).mcz(&[0, 1], 2);
    let state = circuit.run();
    assert_eq!(state[7], Complex64::new(-1.0, 0.0));
}

#[test]
fn measure_adds_measurement_operation() {
    let mut circuit = Circuit::with_classical_bits(2, 2);

    circuit.measure(1, 0);

    assert_eq!(
        circuit.operations(),
        &[Operation::Measure {
            qubit: 1,
            classical_bit: 0,
        }]
    );
}

#[test]
fn measure_all_maps_qubits_to_matching_classical_bits() {
    let mut circuit = Circuit::with_classical_bits(2, 2);

    circuit.measure_all();

    assert_eq!(
        circuit.operations(),
        &[
            Operation::Measure {
                qubit: 0,
                classical_bit: 0,
            },
            Operation::Measure {
                qubit: 1,
                classical_bit: 1,
            },
        ]
    );
}

#[test]
#[should_panic(expected = "Qubit 2 is out of range")]
fn measure_rejects_invalid_qubit() {
    let mut circuit = Circuit::with_classical_bits(2, 2);

    circuit.measure(2, 0);
}

#[test]
#[should_panic(expected = "Classical bit 2 is out of range")]
fn measure_rejects_invalid_classical_bit() {
    let mut circuit = Circuit::with_classical_bits(2, 2);

    circuit.measure(0, 2);
}

#[test]
#[should_panic(expected = "measure_all requires at least as many classical bits as qubits")]
fn measure_all_rejects_too_small_classical_register() {
    let mut circuit = Circuit::with_classical_bits(2, 1);

    circuit.measure_all();
}

#[test]
fn tracks_classical_register_size() {
    let circuit = Circuit::with_classical_bits(2, 3);

    assert_eq!(circuit.n_qubits(), 2);
    assert_eq!(circuit.n_classical_bits(), 3);
}

#[test]
fn bell_measurement_maps_qubits_to_classical_bits() {
    let mut circuit = Circuit::with_classical_bits(2, 2);
    circuit.h(0).cnot(0, 1).measure_all();

    assert_eq!(
        circuit.operations(),
        &[
            Operation::SingleQubit {
                gate: GateKind::H,
                target: 0,
            },
            Operation::Cnot {
                control: 0,
                target: 1,
            },
            Operation::Measure {
                qubit: 0,
                classical_bit: 0,
            },
            Operation::Measure {
                qubit: 1,
                classical_bit: 1,
            },
        ]
    );
}

#[test]
#[should_panic(expected = "Measurement execution is not supported by Circuit::run yet")]
fn run_rejects_measurement_operations() {
    let mut circuit = Circuit::with_classical_bits(1, 1);
    circuit.measure(0, 0);
    circuit.run();
}

#[test]
fn phase_oracle_target_index_uses_lsb_qubit_order() {
    let mut circuit = Circuit::new(3);

    circuit.h(0).h(1).h(2).phase_oracle(1);

    let state = circuit.run();
    let amplitude = 1.0 / 8.0_f64.sqrt();

    for index in 0..8 {
        let expected = if index == 1 {
            Complex64::new(-amplitude, 0.0)
        } else {
            Complex64::new(amplitude, 0.0)
        };

        assert!(
            (state[index] - expected).norm() < 1e-10,
            "unexpected amplitude at basis index {index}"
        );
    }
}
