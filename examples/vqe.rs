use eigenon::circuit::{Parameter, ParameterizedCircuit};
use eigenon::observable::{Hamiltonian, Pauli, PauliString};
use eigenon::variational::{GradientDescent, VqeProblem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theta = Parameter::new("theta");
    let mut ansatz = ParameterizedCircuit::new(1);
    ansatz.ry(theta.clone(), 0);

    // H = Z has eigenvalues +1 for |0> and -1 for |1>. Eigenon's qubit 0 is
    // the least-significant statevector bit, matching every other public API.
    let hamiltonian = Hamiltonian::from_terms([(1.0, PauliString::single(0, Pauli::Z))])?;
    let problem = VqeProblem::new(ansatz, hamiltonian);
    let optimizer = GradientDescent::new(0.2)
        .max_iterations(200)
        .convergence_tolerance(1e-10);

    let result = problem.minimize(optimizer, [(&theta, 0.25)])?;

    println!("converged: {}", result.converged);
    println!("iterations: {}", result.history.len());
    println!("minimum energy: {:.12}", result.minimum);
    println!("theta: {:.12}", result.parameters[&theta]);
    Ok(())
}
