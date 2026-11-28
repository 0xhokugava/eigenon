use eigenon::circuit::{Parameter, ParameterizedCircuit};
use eigenon::engine::utils::to_dirac;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theta = Parameter::new("theta");
    let mut template = ParameterizedCircuit::new(2);
    template.h(0).ry(theta.clone(), 1).cnot(1, 0);

    println!("Parameters: {:?}", template.parameters());

    for angle in [0.0, std::f64::consts::FRAC_PI_2] {
        let circuit = template.bind([(&theta, angle)])?;
        println!("theta={angle}: {}", to_dirac(&circuit.run()));
    }

    Ok(())
}
