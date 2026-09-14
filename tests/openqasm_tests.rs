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
    circuit.h(0).x(0).y(0).z(0).s(0).t(0);
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
fn rejects_mcx_for_now() {
    let mut circuit = Circuit::new(3);
    circuit.mcx(&[0, 1], 2);

    let result = export_openqasm2(&circuit);

    assert!(matches!(
        result,
        Err(OpenQasmExportError::UnsupportedOperation(
            Operation::Mcx { .. }
        ))
    ));
}

#[test]
fn rejects_mcz_for_now() {
    let mut circuit = Circuit::new(3);
    circuit.mcz(&[0, 1], 2);

    let result = export_openqasm2(&circuit);

    assert!(matches!(
        result,
        Err(OpenQasmExportError::UnsupportedOperation(
            Operation::Mcz { .. }
        ))
    ));
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
fn openqasm3_rejects_mcx() {
    let mut circuit = Circuit::new(3);
    circuit.mcx(&[0, 1], 2);
    let result = export_openqasm3(&circuit);

    assert!(matches!(
        result,
        Err(OpenQasmExportError::UnsupportedOperation(
            Operation::Mcx { .. }
        ))
    ));
}

#[test]
fn openqasm3_rejects_mcz() {
    let mut circuit = Circuit::new(3);
    circuit.mcz(&[0, 1], 2);
    let result = export_openqasm3(&circuit);

    assert!(matches!(
        result,
        Err(OpenQasmExportError::UnsupportedOperation(
            Operation::Mcz { .. }
        ))
    ));
}
