use eigenon::circuit::core::Circuit;
use eigenon::export::openqasm::{OpenQasmExportError, export_openqasm2, export_openqasm3};
use std::fs;
use std::process::Command;

type OpenQasmExporter = fn(&Circuit) -> Result<String, OpenQasmExportError>;

fn validate_with_qiskit(name: &str, qasm: &str) -> String {
    let qasm_path =
        std::env::temp_dir().join(format!("eigenon_{name}_{}.qasm", std::process::id()));

    fs::write(&qasm_path, qasm).unwrap();

    let validator = format!(
        "{}/validation/qiskit/openqasm_export.py",
        env!("CARGO_MANIFEST_DIR")
    );

    let output = Command::new("python3")
        .arg(validator)
        .arg(&qasm_path)
        .output()
        .expect("Failed to run Qiskit validator");

    let _ = fs::remove_file(&qasm_path);

    assert!(
        output.status.success(),
        "Qiskit validation failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("Qiskit validator output must be UTF-8")
}

#[test]
#[ignore]
fn qiskit_validates_measured_bell_export() {
    let mut circuit = Circuit::with_classical_bits(2, 2);
    circuit.h(0).cnot(0, 1).measure_all();

    let qasm = export_openqasm2(&circuit).unwrap();

    validate_with_qiskit("measured_bell", &qasm);
}

#[test]
#[ignore]
fn qiskit_preserves_explicit_measurement_mapping() {
    let mut circuit = Circuit::with_classical_bits(2, 3);
    circuit.h(0).measure(0, 2);

    let qasm = export_openqasm2(&circuit).unwrap();

    let stdout = validate_with_qiskit("measurement_mapping", &qasm);

    assert!(stdout.contains("Classical bits: 3"));
    assert!(stdout.contains("q[0] -> c[2]"));
}

#[test]
#[ignore]
fn qiskit_validates_openqasm2_mcx_and_mcz_export() {
    let mut mcx_circuit = Circuit::new(3);
    mcx_circuit.x(0).x(1).mcx(&[0, 1], 2);
    let mcx_qasm = export_openqasm2(&mcx_circuit).unwrap();
    let mcx_stdout = validate_with_qiskit("openqasm2_mcx", &mcx_qasm);

    assert!(mcx_stdout.contains("OpenQASM version: 2.0"));
    assert!(mcx_stdout.contains("P(111) ="));

    let mut mcz_circuit = Circuit::new(3);
    mcz_circuit.x(0).x(1).h(2).mcz(&[0, 1], 2).h(2);
    let mcz_qasm = export_openqasm2(&mcz_circuit).unwrap();
    let mcz_stdout = validate_with_qiskit("openqasm2_mcz", &mcz_qasm);

    assert!(mcz_stdout.contains("OpenQASM version: 2.0"));
    assert!(mcz_stdout.contains("P(111) ="));
}

#[test]
#[ignore]
fn qiskit_validates_openqasm3_mcx_and_mcz_export() {
    let mut mcx_circuit = Circuit::new(4);
    mcx_circuit.x(0).x(1).x(2).mcx(&[0, 1, 2], 3);
    let mcx_qasm = export_openqasm3(&mcx_circuit).unwrap();
    let mcx_stdout = validate_with_qiskit("openqasm3_mcx", &mcx_qasm);

    assert!(mcx_stdout.contains("OpenQASM version: 3.0"));
    assert!(mcx_stdout.contains("P(1111) ="));

    let mut mcz_circuit = Circuit::new(4);
    mcz_circuit.x(0).x(1).x(2).h(3).mcz(&[0, 1, 2], 3).h(3);
    let mcz_qasm = export_openqasm3(&mcz_circuit).unwrap();
    let mcz_stdout = validate_with_qiskit("openqasm3_mcz", &mcz_qasm);

    assert!(mcz_stdout.contains("OpenQASM version: 3.0"));
    assert!(mcz_stdout.contains("P(1111) ="));
}

#[test]
#[ignore]
fn qiskit_validates_single_control_mcx_and_mcz_export() {
    let exporters: [(&str, OpenQasmExporter); 2] =
        [("2.0", export_openqasm2), ("3.0", export_openqasm3)];

    for (version, export) in exporters {
        let mut mcx_circuit = Circuit::new(2);
        mcx_circuit.x(1).mcx(&[1], 0);
        let mcx_qasm = export(&mcx_circuit).unwrap();
        let mcx_stdout = validate_with_qiskit(&format!("openqasm{version}_single_mcx"), &mcx_qasm);

        assert!(mcx_stdout.contains(&format!("OpenQASM version: {version}")));
        assert!(mcx_stdout.contains("P(11) ="));

        let mut mcz_circuit = Circuit::new(2);
        mcz_circuit.x(1).h(0).mcz(&[1], 0).h(0);
        let mcz_qasm = export(&mcz_circuit).unwrap();
        let mcz_stdout = validate_with_qiskit(&format!("openqasm{version}_single_mcz"), &mcz_qasm);

        assert!(mcz_stdout.contains(&format!("OpenQASM version: {version}")));
        assert!(mcz_stdout.contains("P(11) ="));
    }
}

#[test]
#[ignore]
fn qiskit_validates_non_adjacent_controls_and_middle_target() {
    let mut openqasm2_circuit = Circuit::new(3);
    openqasm2_circuit.x(2).x(0).mcx(&[2, 0], 1);
    let openqasm2 = export_openqasm2(&openqasm2_circuit).unwrap();
    let openqasm2_stdout = validate_with_qiskit("openqasm2_non_adjacent_mcx", &openqasm2);

    assert!(openqasm2_stdout.contains("P(111) ="));

    let mut openqasm3_circuit = Circuit::new(5);
    openqasm3_circuit.x(4).x(0).x(2).mcx(&[4, 0, 2], 1);
    let openqasm3 = export_openqasm3(&openqasm3_circuit).unwrap();
    let openqasm3_stdout = validate_with_qiskit("openqasm3_non_adjacent_mcx", &openqasm3);

    assert!(openqasm3_stdout.contains("P(10111) ="));
}

#[test]
#[ignore]
fn qiskit_preserves_mcz_relative_phase() {
    let mut openqasm2_circuit = Circuit::new(3);
    openqasm2_circuit.x(2).x(0).h(1).mcz(&[2, 0], 1);
    let openqasm2 = export_openqasm2(&openqasm2_circuit).unwrap();
    let openqasm2_stdout = validate_with_qiskit("openqasm2_mcz_phase", &openqasm2);

    assert!(openqasm2_stdout.contains("A(101) = 0.707106781187+0.000000000000i"));
    assert!(openqasm2_stdout.contains("A(111) = -0.707106781187+0.000000000000i"));

    let mut openqasm3_circuit = Circuit::new(5);
    openqasm3_circuit.x(4).x(0).x(2).h(1).mcz(&[4, 0, 2], 1);
    let openqasm3 = export_openqasm3(&openqasm3_circuit).unwrap();
    let openqasm3_stdout = validate_with_qiskit("openqasm3_mcz_phase", &openqasm3);

    assert!(openqasm3_stdout.contains("A(10101) = 0.707106781187+0.000000000000i"));
    assert!(openqasm3_stdout.contains("A(10111) = -0.707106781187+0.000000000000i"));
}
