use crate::circuit::core::Circuit;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateSpec {
    H(usize),
    X(usize),
    Y(usize),
    Z(usize),
    S(usize),
    T(usize),
    I(usize),
    Cnot { control: usize, target: usize },
    Cz { control: usize, target: usize },
    Mcx { controls: Vec<usize>, target: usize },
    Mcz { controls: Vec<usize>, target: usize },
}

impl GateSpec {
    pub fn apply(&self, circuit: &mut Circuit) {
        match self {
            GateSpec::H(qubit) => circuit.h(*qubit),
            GateSpec::X(qubit) => circuit.x(*qubit),
            GateSpec::Y(qubit) => circuit.y(*qubit),
            GateSpec::Z(qubit) => circuit.z(*qubit),
            GateSpec::S(qubit) => circuit.s(*qubit),
            GateSpec::T(qubit) => circuit.t(*qubit),
            GateSpec::I(qubit) => circuit.i(*qubit),
            GateSpec::Cnot { control, target } => circuit.cnot(*control, *target),
            GateSpec::Cz { control, target } => circuit.cz(*control, *target),
            GateSpec::Mcx { controls, target } => circuit.mcx(controls, *target),
            GateSpec::Mcz { controls, target } => circuit.mcz(controls, *target),
        };
    }
    pub fn validate(&self, n_qubits: usize) -> Result<(), String> {
        if n_qubits == 0 {
            return Err("Circuit must contain at least one qubit".to_string());
        }
        match self {
            GateSpec::H(qubit)
            | GateSpec::X(qubit)
            | GateSpec::Y(qubit)
            | GateSpec::Z(qubit)
            | GateSpec::S(qubit)
            | GateSpec::T(qubit)
            | GateSpec::I(qubit) => {
                validate_qubit(*qubit, n_qubits)?;
            }
            GateSpec::Cnot { control, target } | GateSpec::Cz { control, target } => {
                validate_qubit(*control, n_qubits)?;
                validate_qubit(*target, n_qubits)?;

                if control == target {
                    return Err("Control and target qubits must be different".to_string());
                }
            }
            GateSpec::Mcx { controls, target } | GateSpec::Mcz { controls, target } => {
                if controls.is_empty() {
                    return Err("MCX/MCZ requires at least one control qubit".into());
                }

                for &control in controls {
                    validate_qubit(control, n_qubits)?;
                }

                validate_qubit(*target, n_qubits)?;

                if controls.contains(target) {
                    return Err("Target qubit cannot also be a control qubit".into());
                }

                for i in 0..controls.len() {
                    for j in i + 1..controls.len() {
                        if controls[i] == controls[j] {
                            return Err(format!("Duplicate control qubit: {}", controls[i]));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl FromStr for GateSpec {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (gate_name, operands) = input
            .split_once(':')
            .ok_or_else(|| format!("Invalid gate format: {input}. Expected gate:operands"))?;

        let gate_name = gate_name.trim().to_ascii_lowercase();
        let operands = operands.trim();

        match gate_name.as_str() {
            "h" => Ok(GateSpec::H(parse_qubit(operands)?)),
            "x" => Ok(GateSpec::X(parse_qubit(operands)?)),
            "y" => Ok(GateSpec::Y(parse_qubit(operands)?)),
            "z" => Ok(GateSpec::Z(parse_qubit(operands)?)),
            "s" => Ok(GateSpec::S(parse_qubit(operands)?)),
            "t" => Ok(GateSpec::T(parse_qubit(operands)?)),
            "i" => Ok(GateSpec::I(parse_qubit(operands)?)),
            "cnot" => {
                let (control, target) = parse_control_target(operands, &gate_name)?;
                Ok(GateSpec::Cnot { control, target })
            }
            "cz" => {
                let (control, target) = parse_control_target(operands, &gate_name)?;
                Ok(GateSpec::Cz { control, target })
            }
            "mcx" => {
                let (controls, target) = parse_controls_target(operands, "mcx")?;
                Ok(GateSpec::Mcx { controls, target })
            }

            "mcz" => {
                let (controls, target) = parse_controls_target(operands, "mcz")?;
                Ok(GateSpec::Mcz { controls, target })
            }
            _ => Err(format!("Unknown gate: {gate_name}")),
        }
    }
}

fn parse_qubit(value: &str) -> Result<usize, String> {
    value
        .trim()
        .parse::<usize>()
        .map_err(|_| format!("Invalid qubit index: {}", value.trim()))
}

fn parse_control_target(operands: &str, gate_name: &str) -> Result<(usize, usize), String> {
    let (control, target) = operands
        .split_once(',')
        .ok_or_else(|| format!("{gate_name} requires control,target"))?;

    let control = parse_qubit(control)?;
    let target = parse_qubit(target)?;

    if control == target {
        return Err(format!("{gate_name} control and target must be different"));
    }

    Ok((control, target))
}

fn parse_controls_target(operands: &str, gate_name: &str) -> Result<(Vec<usize>, usize), String> {
    let (controls_part, target_part) = operands
        .split_once(':')
        .ok_or_else(|| format!("{gate_name} requires controls:target"))?;

    if controls_part.is_empty() {
        return Err(format!("{gate_name} requires at least one control"));
    }

    if target_part.is_empty() {
        return Err(format!("{gate_name} requires target"));
    }

    let controls = controls_part
        .split(',')
        .map(parse_qubit)
        .collect::<Result<Vec<_>, _>>()?;

    let target = parse_qubit(target_part)?;

    Ok((controls, target))
}

fn validate_qubit(qubit: usize, n_qubits: usize) -> Result<(), String> {
    if qubit >= n_qubits {
        return Err(format!(
            "Qubit index {qubit} is out of range for a {n_qubits}-qubit circuit"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::GateSpec;
    use crate::circuit::core::Circuit;

    #[test]
    fn parses_gate_with_whitespace() {
        let gate: GateSpec = " cnot : 0, 1 ".parse().unwrap();

        assert_eq!(
            gate,
            GateSpec::Cnot {
                control: 0,
                target: 1,
            }
        );
    }

    #[test]
    fn validates_single_qubit_gate_in_range() {
        let gate = GateSpec::H(2);

        assert!(gate.validate(3).is_ok());
    }

    #[test]
    fn rejects_single_qubit_gate_out_of_range() {
        let gate = GateSpec::H(3);

        assert!(gate.validate(3).is_err());
    }

    #[test]
    fn validates_two_qubit_gate_in_range() {
        let gate = GateSpec::Cnot {
            control: 0,
            target: 2,
        };

        assert!(gate.validate(3).is_ok());
    }

    #[test]
    fn rejects_two_qubit_gate_out_of_range() {
        let gate = GateSpec::Cnot {
            control: 0,
            target: 3,
        };

        assert!(gate.validate(3).is_err());
    }

    #[test]
    fn rejects_equal_control_and_target() {
        let gate = GateSpec::Cnot {
            control: 1,
            target: 1,
        };

        assert!(gate.validate(3).is_err());
    }

    #[test]
    fn rejects_zero_qubit_circuit() {
        let gate = GateSpec::X(0);

        assert!(gate.validate(0).is_err());
    }

    #[test]
    fn applies_x_gate_to_circuit() {
        let mut circuit = Circuit::new(1);
        GateSpec::X(0).apply(&mut circuit);
        let state = circuit.run();

        assert_eq!(state[0].norm_sqr(), 0.0);
        assert_eq!(state[1].norm_sqr(), 1.0);
    }

    #[test]
    fn parses_i_gate() {
        let gate: GateSpec = "i:0".parse().unwrap();

        assert_eq!(gate, GateSpec::I(0));
    }

    #[test]
    fn applies_i_gate_to_circuit() {
        let mut circuit = Circuit::new(1);
        GateSpec::I(0).apply(&mut circuit);
        let state = circuit.run();

        assert_eq!(state[0].norm_sqr(), 1.0);
        assert_eq!(state[1].norm_sqr(), 0.0);
    }

    #[test]
    fn applies_cnot_gate_to_circuit() {
        let mut circuit = Circuit::new(2);
        GateSpec::X(0).apply(&mut circuit);
        GateSpec::Cnot {
            control: 0,
            target: 1,
        }
        .apply(&mut circuit);

        let state = circuit.run();
        assert_eq!(state[3].norm_sqr(), 1.0);
    }
}
