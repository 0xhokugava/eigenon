use eigenon::circuit::core::Circuit;
use eigenon::engine::utils::to_dirac;

fn main() {
    let mut circuit = Circuit::new(2);
    circuit.h(0).cnot(0, 1);
    let state = circuit.run();
    println!("Bell state: {}", to_dirac(&state));
}
