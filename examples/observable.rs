use eigenon::circuit::core::Circuit;
use eigenon::observable::{Hamiltonian, Pauli, PauliString};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut circuit = Circuit::new(2);
    circuit.h(0).cnot(0, 1);

    let hamiltonian = Hamiltonian::from_terms([
        (-1.0, PauliString::new([(0, Pauli::Z), (1, Pauli::Z)])?),
        (-0.5, PauliString::single(0, Pauli::X)),
    ])?;

    let state = circuit.run();
    let energy = hamiltonian.expectation_value(&state, circuit.n_qubits())?;
    println!("Energy: {energy}");

    Ok(())
}
