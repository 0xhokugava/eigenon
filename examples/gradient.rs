use eigenon::circuit::{Parameter, ParameterizedCircuit};
use eigenon::gradient::parameter_shift;
use eigenon::observable::{Hamiltonian, Pauli, PauliString};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(1);
    template.ry(theta.clone(), 0);
    let hamiltonian = Hamiltonian::from_terms([(-1.0, PauliString::single(0, Pauli::Z))])?;

    let gradients = parameter_shift(
        &template,
        &[(&theta, 0.4)],
        &hamiltonian,
        std::slice::from_ref(&theta),
    )?;

    println!("dE/dtheta = {}", gradients[&theta]);
    Ok(())
}
