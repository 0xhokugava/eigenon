/// Built-in single-qubit gate kinds supported by the Circuit API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateKind {
    X,
    Y,
    Z,
    H,
    S,
    T,
    I,
}

/// Semantic circuit operation.
///
/// This representation preserves gate identity, which is required for
/// export formats such as OpenQASM. Execution code maps these operations
/// to matrices only when the circuit is run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    SingleQubit { gate: GateKind, target: usize },
    Cnot { control: usize, target: usize },
    Cz { control: usize, target: usize },
    Mcx { controls: Vec<usize>, target: usize },
    Mcz { controls: Vec<usize>, target: usize },
    Measure { qubit: usize, classical_bit: usize },
}
