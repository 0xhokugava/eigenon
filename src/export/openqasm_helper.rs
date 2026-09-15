use crate::circuit::core::Circuit;
use crate::circuit::operation::{GateKind, Operation};
use crate::export::openqasm::OpenQasmExportError;

#[derive(Clone, Copy)]
pub enum OpenQasmVersion {
    V2,
    V3,
}

pub fn write_operations(
    circuit: &Circuit,
    output: &mut String,
    version: OpenQasmVersion,
) -> Result<(), OpenQasmExportError> {
    for operation in circuit.operations() {
        match operation {
            Operation::SingleQubit { gate, target } => {
                output.push_str(&format!("{} q[{}];\n", openqasm_gate_name(*gate), target));
            }

            Operation::Cnot { control, target } => {
                output.push_str(&format!("cx q[{}], q[{}];\n", control, target));
            }

            Operation::Cz { control, target } => {
                output.push_str(&format!("cz q[{}], q[{}];\n", control, target));
            }

            Operation::Mcx { .. } | Operation::Mcz { .. } => {
                return Err(OpenQasmExportError::UnsupportedOperation(operation.clone()));
            }

            Operation::Measure {
                qubit,
                classical_bit,
            } => match version {
                OpenQasmVersion::V2 => {
                    output.push_str(&format!("measure q[{}] -> c[{}];\n", qubit, classical_bit));
                }
                OpenQasmVersion::V3 => {
                    output.push_str(&format!("c[{}] = measure q[{}];\n", classical_bit, qubit));
                }
            },
        }
    }

    Ok(())
}

fn openqasm_gate_name(gate: GateKind) -> &'static str {
    match gate {
        GateKind::X => "x",
        GateKind::Y => "y",
        GateKind::Z => "z",
        GateKind::H => "h",
        GateKind::S => "s",
        GateKind::T => "t",
        GateKind::I => "id",
    }
}
