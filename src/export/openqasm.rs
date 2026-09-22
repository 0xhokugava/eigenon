//! OpenQASM 2.0 and OpenQASM 3.0 circuit export.
//!
//! Export preserves Eigenon's qubit indices. Qubit `0` remains the least
//! significant bit when the generated circuit is interpreted by compatible
//! tooling such as Qiskit.
//!
//! # Supported operations
//!
//! Both exporters support the built-in `I`, `X`, `Y`, `Z`, `H`, `S` and `T`
//! single-qubit gates, plus CNOT, CZ and measurement operations. A classical
//! register is emitted only when the circuit declares classical bits.
//!
//! MCX and MCZ operations return
//! [`OpenQasmExportError::UnsupportedOperation`]. The circuit model does not
//! currently represent custom or parameterized gates.

use crate::circuit::core::Circuit;
use crate::circuit::operation::Operation;
use crate::export::openqasm_helper::{OpenQasmVersion, write_operations};
use std::fmt;

/// Error returned when a circuit cannot be represented by an exporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenQasmExportError {
    /// The circuit contains an operation unsupported by the selected format.
    UnsupportedOperation(Operation),
}

impl fmt::Display for OpenQasmExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenQasmExportError::UnsupportedOperation(operation) => {
                write!(f, "Unsupported OpenQASM operation: {:?}", operation)
            }
        }
    }
}

impl std::error::Error for OpenQasmExportError {}

/// Exports a circuit as OpenQASM 2.0 source text.
///
/// The output includes `qelib1.inc`, a quantum register and a classical
/// register when the circuit uses classical bits.
///
/// # Example
///
/// ```
/// use eigenon::circuit::core::Circuit;
/// use eigenon::export::openqasm::export_openqasm2;
///
/// let mut circuit = Circuit::new(2);
/// circuit.h(0).cnot(0, 1);
///
/// let qasm = export_openqasm2(&circuit)?;
/// assert!(qasm.contains("h q[0];"));
/// assert!(qasm.contains("cx q[0], q[1];"));
///
/// # Ok::<(), eigenon::export::openqasm::OpenQasmExportError>(())
/// ```
///
/// # Errors
///
/// Returns [`OpenQasmExportError::UnsupportedOperation`] when the circuit
/// contains an operation that the OpenQASM 2.0 exporter cannot represent.
pub fn export_openqasm2(circuit: &Circuit) -> Result<String, OpenQasmExportError> {
    let mut output = String::new();

    output.push_str("OPENQASM 2.0;\n");
    output.push_str("include \"qelib1.inc\";\n\n");
    output.push_str(&format!("qreg q[{}];\n", circuit.n_qubits()));

    if circuit.n_classical_bits() > 0 {
        output.push_str(&format!("creg c[{}];\n", circuit.n_classical_bits()));
    }

    output.push('\n');

    write_operations(circuit, &mut output, OpenQasmVersion::V2)?;

    Ok(output)
}

/// Exports a circuit as OpenQASM 3.0 source text.
///
/// The output includes `stdgates.inc`, a qubit register and a classical bit
/// register when the circuit uses classical bits.
///
/// # Errors
///
/// Returns [`OpenQasmExportError::UnsupportedOperation`] when the circuit
/// contains an operation that the OpenQASM 3.0 exporter cannot represent.
pub fn export_openqasm3(circuit: &Circuit) -> Result<String, OpenQasmExportError> {
    let mut output = String::new();

    output.push_str("OPENQASM 3.0;\n");
    output.push_str("include \"stdgates.inc\";\n\n");
    output.push_str(&format!("qubit[{}] q;\n", circuit.n_qubits()));

    if circuit.n_classical_bits() > 0 {
        output.push_str(&format!("bit[{}] c;\n", circuit.n_classical_bits()));
    }

    output.push('\n');

    write_operations(circuit, &mut output, OpenQasmVersion::V3)?;

    Ok(output)
}
