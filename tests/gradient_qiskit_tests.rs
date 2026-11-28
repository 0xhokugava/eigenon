use eigenon::circuit::{Parameter, ParameterizedCircuit};
use eigenon::gradient::parameter_shift;
use eigenon::observable::{Hamiltonian, Pauli, PauliString};
use std::collections::BTreeMap;
use std::process::Command;

#[test]
#[ignore]
fn qiskit_validates_parameter_shift_gradients() {
    let script = format!(
        "{}/validation/qiskit/gradients.py",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new("python3")
        .arg(script)
        .output()
        .expect("failed to run Qiskit gradient reference");
    assert!(
        output.status.success(),
        "Qiskit gradient reference failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reference: BTreeMap<String, f64> = serde_json::from_slice(&output.stdout).unwrap();

    let theta = Parameter::new("theta");
    let phi = Parameter::new("phi");
    let mut template = ParameterizedCircuit::new(2);
    template
        .h(0)
        .ry(theta.clone(), 0)
        .rx(phi.clone(), 1)
        .ry(theta.clone(), 1)
        .cnot(0, 1);
    let hamiltonian = Hamiltonian::from_terms([
        (0.3, PauliString::identity()),
        (
            -0.8,
            PauliString::new([(0, Pauli::Z), (1, Pauli::Z)]).unwrap(),
        ),
        (0.4, PauliString::single(1, Pauli::X)),
    ])
    .unwrap();

    let gradients = parameter_shift(
        &template,
        &[(&theta, 0.29), (&phi, -0.43)],
        &hamiltonian,
        &[theta.clone(), phi.clone()],
    )
    .unwrap();

    for parameter in [&theta, &phi] {
        let actual = gradients[parameter];
        let expected = reference[parameter.name()];
        assert!(
            (actual - expected).abs() < 1e-7,
            "{}: Eigenon={actual}, Qiskit={expected}",
            parameter.name()
        );
    }
}
