use eigenon::circuit::core::Circuit;
use eigenon::export::openqasm::export_openqasm2;
use std::fs;
use std::process::Command;

#[test]
#[ignore]
fn qiskit_validates_measured_bell_export() {
    let mut circuit = Circuit::with_classical_bits(2, 2);
    circuit.h(0).cnot(0, 1).measure_all();

    let qasm = export_openqasm2(&circuit).unwrap();

    let qasm_path =
        std::env::temp_dir().join(format!("eigenon_measured_bell_{}.qasm", std::process::id()));

    fs::write(&qasm_path, qasm).unwrap();

    let validator = format!(
        "{}/validation/qiskit/openqasm_export.py",
        env!("CARGO_MANIFEST_DIR")
    );

    let status = Command::new("python3")
        .arg(validator)
        .arg(&qasm_path)
        .status()
        .expect("Failed to run Qiskit validator");

    let _ = fs::remove_file(&qasm_path);

    assert!(status.success(), "Qiskit validation failed");
}

#[test]
#[ignore]
fn qiskit_preserves_explicit_measurement_mapping() {
    let mut circuit = Circuit::with_classical_bits(2, 3);
    circuit.h(0).measure(0, 2);

    let qasm = export_openqasm2(&circuit).unwrap();

    let qasm_path = std::env::temp_dir().join(format!(
        "eigenon_measurement_mapping_{}.qasm",
        std::process::id()
    ));

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

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Classical bits: 3"));
    assert!(stdout.contains("q[0] -> c[2]"));
}
