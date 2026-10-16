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

            Operation::Mcx { controls, target } => {
                write_multi_controlled_gate(output, version, operation, "x", controls, *target)?;
            }

            Operation::Mcz { controls, target } => {
                write_multi_controlled_gate(output, version, operation, "z", controls, *target)?;
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

fn write_multi_controlled_gate(
    output: &mut String,
    version: OpenQasmVersion,
    operation: &Operation,
    gate: &str,
    controls: &[usize],
    target: usize,
) -> Result<(), OpenQasmExportError> {
    match version {
        OpenQasmVersion::V2 => {
            write_openqasm2_multi_controlled_gate(output, operation, gate, controls, target)
        }
        OpenQasmVersion::V3 => {
            let modifier = if controls.len() == 1 {
                "ctrl".to_string()
            } else {
                format!("ctrl({})", controls.len())
            };
            let operands = qasm_operands(controls, target);
            output.push_str(&format!("{modifier} @ {gate} {operands};\n"));
            Ok(())
        }
    }
}

fn write_openqasm2_multi_controlled_gate(
    output: &mut String,
    operation: &Operation,
    gate: &str,
    controls: &[usize],
    target: usize,
) -> Result<(), OpenQasmExportError> {
    match (gate, controls) {
        ("x", [control]) => {
            output.push_str(&format!("cx q[{control}], q[{target}];\n"));
        }
        ("z", [control]) => {
            output.push_str(&format!("cz q[{control}], q[{target}];\n"));
        }
        ("x", [first, second]) => {
            output.push_str(&format!("ccx q[{first}], q[{second}], q[{target}];\n"));
        }
        ("z", [first, second]) => {
            output.push_str(&format!("h q[{target}];\n"));
            output.push_str(&format!("ccx q[{first}], q[{second}], q[{target}];\n"));
            output.push_str(&format!("h q[{target}];\n"));
        }
        _ => {
            return Err(OpenQasmExportError::UnsupportedOperation(operation.clone()));
        }
    }

    Ok(())
}

fn qasm_operands(controls: &[usize], target: usize) -> String {
    controls
        .iter()
        .copied()
        .chain(std::iter::once(target))
        .map(|qubit| format!("q[{qubit}]"))
        .collect::<Vec<_>>()
        .join(", ")
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
