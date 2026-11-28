pub mod circuit_demos;
pub mod engine_verification;
pub mod foundations;
pub mod harness;

pub fn run_all_verifications() {
    engine_verification::inplace_verification::run();
    engine_verification::cnot_inplace_verification::run();
}

pub fn run_all_demos() {
    circuit_demos::single_qubit::run();
    foundations::tensor_product_example::run();
    circuit_demos::cnot_verification::run();
    circuit_demos::entanglement::run();
    crate::algorithms::deutsch::run_deutsch_demo();
    crate::algorithms::deutsch_jozsa::run_deutsch_jozsa_demo();
    crate::algorithms::grover_search::run_grover_demo();
}
