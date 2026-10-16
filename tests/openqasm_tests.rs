use eigenon::circuit::catalog;
use eigenon::circuit::catalog::BellState;
use eigenon::circuit::core::Circuit;
use eigenon::circuit::operation::Operation;
use eigenon::export::openqasm::{OpenQasmExportError, export_openqasm2, export_openqasm3};

#[test]
fn exports_empty_circuit() {
    let circuit = Circuit::new(2);
    let qasm = export_openqasm2(&circuit).unwrap();
    insta::assert_snapshot!(qasm);
}

#[test]
fn exports_single_qubit_gates() {
    let mut circuit = Circuit::new(1);
    circuit.h(0).x(0).y(0).z(0).s(0).t(0).i(0);
    let qasm = export_openqasm2(&circuit).unwrap();
    insta::assert_snapshot!(qasm);
}

#[test]
fn exports_bell_circuit() {
    let circuit = catalog::bell(BellState::PhiPlus);
    let qasm = export_openqasm2(&circuit).unwrap();
    insta::assert_snapshot!(qasm);
}

#[test]
fn exports_cz() {
    let mut circuit = Circuit::new(2);
    circuit.h(0).cz(0, 1);
    let qasm = export_openqasm2(&circuit).unwrap();
    insta::assert_snapshot!(qasm);
}

#[test]
fn openqasm2_exports_single_control_mcx_and_mcz() {
    let mut circuit = Circuit::new(2);
    circuit.mcx(&[0], 1).mcz(&[1], 0);

    let qasm = export_openqasm2(&circuit).unwrap();

    assert!(qasm.contains("cx q[0], q[1];"));
    assert!(qasm.contains("cz q[1], q[0];"));
}

#[test]
fn openqasm2_exports_two_control_mcx_as_toffoli() {
    let mut circuit = Circuit::new(3);
    circuit.mcx(&[0, 1], 2);

    let qasm = export_openqasm2(&circuit).unwrap();

    assert!(qasm.contains("ccx q[0], q[1], q[2];"));
}

#[test]
fn openqasm2_decomposes_two_control_mcz() {
    let mut circuit = Circuit::new(3);
    circuit.mcz(&[0, 1], 2);

    let qasm = export_openqasm2(&circuit).unwrap();

    assert!(qasm.contains("h q[2];\nccx q[0], q[1], q[2];\nh q[2];"));
}

#[test]
fn openqasm2_preserves_control_order_with_middle_target() {
    let mut circuit = Circuit::new(3);
    circuit.mcx(&[2, 0], 1).mcz(&[2, 0], 1);

    let qasm = export_openqasm2(&circuit).unwrap();

    assert!(qasm.contains("ccx q[2], q[0], q[1];"));
    assert!(qasm.contains("h q[1];\nccx q[2], q[0], q[1];\nh q[1];"));
}

#[test]
fn openqasm2_rejects_more_than_two_controls() {
    let mut mcx_circuit = Circuit::new(4);
    mcx_circuit.mcx(&[0, 1, 2], 3);

    let mut mcz_circuit = Circuit::new(4);
    mcz_circuit.mcz(&[0, 1, 2], 3);

    assert!(matches!(
        export_openqasm2(&mcx_circuit),
        Err(OpenQasmExportError::UnsupportedOperation(
            Operation::Mcx { .. }
        ))
    ));
    assert!(matches!(
        export_openqasm2(&mcz_circuit),
        Err(OpenQasmExportError::UnsupportedOperation(
            Operation::Mcz { .. }
        ))
    ));
}

#[test]
fn exports_grover_primitives_with_multi_controlled_gates() {
    let mut openqasm2_circuit = Circuit::new(3);
    openqasm2_circuit.phase_oracle(5).diffusion();

    let openqasm2 = export_openqasm2(&openqasm2_circuit).unwrap();
    assert!(openqasm2.contains("ccx"));

    let mut openqasm3_circuit = Circuit::new(4);
    openqasm3_circuit.phase_oracle(9).diffusion();

    let openqasm3 = export_openqasm3(&openqasm3_circuit).unwrap();
    assert!(openqasm3.contains("ctrl(3) @ z"));
}

#[test]
fn does_not_export_classical_register_when_unused() {
    let mut circuit = Circuit::new(2);

    circuit.h(0).cnot(0, 1);

    let qasm = export_openqasm2(&circuit).unwrap();

    assert!(!qasm.contains("creg"));
    assert!(!qasm.contains("measure"));
}

#[test]
fn snapshot_bell_circuit() {
    let circuit = catalog::bell(BellState::PhiPlus);
    let qasm = export_openqasm2(&circuit).unwrap();
    insta::assert_snapshot!(qasm);
}

#[test]
fn snapshot_measured_bell_circuit() {
    let mut circuit = Circuit::with_classical_bits(2, 2);
    circuit.h(0).cnot(0, 1).measure_all();
    let qasm = export_openqasm2(&circuit).unwrap();
    insta::assert_snapshot!(qasm);
}

#[test]
fn snapshot_explicit_measurement_mapping() {
    let mut circuit = Circuit::with_classical_bits(2, 3);
    circuit.h(0).measure(0, 2);
    let qasm = export_openqasm2(&circuit).unwrap();
    insta::assert_snapshot!(qasm);
}

#[test]
fn snapshot_openqasm3_bell_circuit() {
    let circuit = catalog::bell(BellState::PhiPlus);
    let qasm = export_openqasm3(&circuit).unwrap();
    insta::assert_snapshot!(qasm);
}

#[test]
fn snapshot_openqasm3_measured_bell_circuit() {
    let mut circuit = Circuit::with_classical_bits(2, 2);
    circuit.h(0).cnot(0, 1).measure_all();
    let qasm = export_openqasm3(&circuit).unwrap();
    insta::assert_snapshot!(qasm);
}

#[test]
fn snapshot_openqasm3_explicit_measurement_mapping() {
    let mut circuit = Circuit::with_classical_bits(2, 3);
    circuit.h(0).measure(0, 2);
    let qasm = export_openqasm3(&circuit).unwrap();
    insta::assert_snapshot!(qasm);
}

#[test]
fn openqasm3_exports_mcx_with_control_modifier() {
    let mut circuit = Circuit::new(4);
    circuit.mcx(&[0, 1, 2], 3);
    let qasm = export_openqasm3(&circuit).unwrap();

    assert!(qasm.contains("ctrl(3) @ x q[0], q[1], q[2], q[3];"));
}

#[test]
fn openqasm3_exports_mcz_with_control_modifier() {
    let mut circuit = Circuit::new(4);
    circuit.mcz(&[2, 0, 1], 3);
    let qasm = export_openqasm3(&circuit).unwrap();

    assert!(qasm.contains("ctrl(3) @ z q[2], q[0], q[1], q[3];"));
}

#[test]
fn openqasm3_exports_single_control_modifier() {
    let mut circuit = Circuit::new(2);
    circuit.mcx(&[1], 0).mcz(&[0], 1);
    let qasm = export_openqasm3(&circuit).unwrap();

    assert!(qasm.contains("ctrl @ x q[1], q[0];"));
    assert!(qasm.contains("ctrl @ z q[0], q[1];"));
}

#[test]
fn openqasm3_preserves_non_adjacent_control_order_with_middle_target() {
    let mut circuit = Circuit::new(5);
    circuit.mcx(&[4, 0, 2], 1).mcz(&[2, 4, 0], 1);
    let qasm = export_openqasm3(&circuit).unwrap();

    assert!(qasm.contains("ctrl(3) @ x q[4], q[0], q[2], q[1];"));
    assert!(qasm.contains("ctrl(3) @ z q[2], q[4], q[0], q[1];"));
}
