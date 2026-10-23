/// Built-in single-qubit gate kinds supported by the Circuit API.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GateKind {
    /// Pauli-X gate.
    X,
    /// Pauli-Y gate.
    Y,
    /// Pauli-Z gate.
    Z,
    /// Hadamard gate.
    H,
    /// S phase gate.
    S,
    /// T phase gate.
    T,
    /// Identity gate.
    I,
    /// Rotation around the X axis by an angle in radians.
    Rx(f64),
    /// Rotation around the Y axis by an angle in radians.
    Ry(f64),
    /// Rotation around the Z axis by an angle in radians.
    Rz(f64),
    /// Universal single-qubit gate using OpenQASM's theta, phi and lambda
    /// angle convention, in radians.
    U { theta: f64, phi: f64, lambda: f64 },
}

/// Semantic circuit operation.
///
/// This representation preserves gate identity, which is required for
/// export formats such as OpenQASM. Execution code maps these operations
/// to matrices only when the circuit is run.
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    /// A built-in single-qubit gate and its target.
    SingleQubit { gate: GateKind, target: usize },
    /// Controlled-X operation in `control, target` order.
    Cnot { control: usize, target: usize },
    /// Controlled-Z operation in `control, target` order.
    Cz { control: usize, target: usize },
    /// Multi-controlled X operation.
    Mcx { controls: Vec<usize>, target: usize },
    /// Multi-controlled Z operation.
    Mcz { controls: Vec<usize>, target: usize },
    /// Measurement from a qubit into a classical bit.
    Measure { qubit: usize, classical_bit: usize },
}
