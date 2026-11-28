use eigenon::circuit::core::Circuit;
use eigenon::observable::{Hamiltonian, Pauli, PauliString};
use std::collections::BTreeMap;
use std::process::Command;

#[test]
#[ignore]
fn qiskit_validates_pauli_and_hamiltonian_expectations() {
    let script = format!(
        "{}/validation/qiskit/observables.py",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new("python3")
        .arg(script)
        .output()
        .expect("failed to run Qiskit observable reference");
    assert!(
        output.status.success(),
        "Qiskit observable reference failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected: BTreeMap<String, f64> = serde_json::from_slice(&output.stdout).unwrap();

    let mut circuit = Circuit::new(3);
    let state = circuit.h(0).ry(0.37, 1).cnot(0, 2).run();
    let z0 = PauliString::single(0, Pauli::Z);
    let y1 = PauliString::single(1, Pauli::Y);
    let x0_x2 = PauliString::new([(0, Pauli::X), (2, Pauli::X)]).unwrap();
    let hamiltonian = Hamiltonian::from_terms([
        (0.3, PauliString::identity()),
        (-1.2, z0.clone()),
        (-0.4, y1.clone()),
        (0.7, x0_x2.clone()),
    ])
    .unwrap();

    let actual = BTreeMap::from([
        ("z0", z0.expectation_value(&state, 3).unwrap()),
        ("y1", y1.expectation_value(&state, 3).unwrap()),
        ("x0_x2", x0_x2.expectation_value(&state, 3).unwrap()),
        (
            "hamiltonian",
            hamiltonian.expectation_value(&state, 3).unwrap(),
        ),
    ]);

    for (name, value) in actual {
        let reference = expected[name];
        assert!(
            (value - reference).abs() < 1e-12,
            "{name}: Eigenon={value}, Qiskit={reference}"
        );
    }
}
