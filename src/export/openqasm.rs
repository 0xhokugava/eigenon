use crate::circuit::core::Circuit;
use crate::circuit::operation::Operation;
use crate::export::openqasm_helper::{OpenQasmVersion, write_operations};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenQasmExportError {
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
