//! Named scalar parameters and reusable circuit templates.

use crate::circuit::core::Circuit;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Stable identity for one named scalar circuit parameter.
///
/// Parameters compare by their complete name. Cloning a parameter therefore
/// preserves its identity, and independently constructed parameters with the
/// same name refer to the same binding.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Parameter {
    name: String,
}

impl Parameter {
    /// Creates a named parameter.
    ///
    /// # Panics
    ///
    /// Panics when `name` is empty or contains only whitespace.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(!name.trim().is_empty(), "parameter name must not be empty");
        Self { name }
    }

    /// Returns the stable parameter name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name)
    }
}

/// A rotation angle stored in a reusable circuit template.
#[derive(Clone, Debug, PartialEq)]
pub enum ParameterValue {
    /// A finite angle already known in radians.
    Fixed(f64),
    /// An angle supplied later through [`ParameterizedCircuit::bind`].
    Symbol(Parameter),
}

impl From<f64> for ParameterValue {
    fn from(value: f64) -> Self {
        Self::Fixed(value)
    }
}

impl From<Parameter> for ParameterValue {
    fn from(parameter: Parameter) -> Self {
        Self::Symbol(parameter)
    }
}

impl From<&Parameter> for ParameterValue {
    fn from(parameter: &Parameter) -> Self {
        Self::Symbol(parameter.clone())
    }
}

/// Built-in single-qubit gates supported by a parameterized template.
#[derive(Clone, Debug, PartialEq)]
pub enum ParameterizedGateKind {
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
    /// X-axis rotation with a fixed or named angle.
    Rx(ParameterValue),
    /// Y-axis rotation with a fixed or named angle.
    Ry(ParameterValue),
    /// Z-axis rotation with a fixed or named angle.
    Rz(ParameterValue),
    /// Numeric universal gate using OpenQASM angle order.
    U { theta: f64, phi: f64, lambda: f64 },
}

/// One inspectable operation in a [`ParameterizedCircuit`].
#[derive(Clone, Debug, PartialEq)]
pub enum ParameterizedOperation {
    /// A built-in single-qubit gate and its target.
    SingleQubit {
        gate: ParameterizedGateKind,
        target: usize,
    },
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

/// Typed failures produced while binding a circuit template.
#[derive(Clone, Debug, PartialEq)]
pub enum BindingError {
    /// A required parameter did not receive a value.
    MissingParameter(Parameter),
    /// A parameter appeared more than once in the supplied bindings.
    DuplicateBinding(Parameter),
    /// A binding was supplied for a parameter not used by the template.
    UnknownParameter(Parameter),
    /// A parameter was bound to NaN or infinity.
    NonFiniteBinding(Parameter),
}

impl fmt::Display for BindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingParameter(parameter) => {
                write!(formatter, "missing binding for parameter `{parameter}`")
            }
            Self::DuplicateBinding(parameter) => {
                write!(formatter, "duplicate binding for parameter `{parameter}`")
            }
            Self::UnknownParameter(parameter) => {
                write!(formatter, "unknown parameter binding `{parameter}`")
            }
            Self::NonFiniteBinding(parameter) => {
                write!(
                    formatter,
                    "binding for parameter `{parameter}` must be finite"
                )
            }
        }
    }
}

impl Error for BindingError {}

/// Reusable circuit structure containing fixed gates and named rotations.
///
/// Operations remain in insertion order. [`ParameterizedCircuit::parameters`]
/// lists unique symbolic parameters in order of first appearance. Binding does
/// not mutate the template, so the same structure can produce any number of
/// executable numeric [`Circuit`] values.
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterizedCircuit {
    n_qubits: usize,
    n_classical_bits: usize,
    operations: Vec<ParameterizedOperation>,
    parameters: Vec<Parameter>,
}

impl ParameterizedCircuit {
    /// Creates an empty template with no classical register.
    ///
    /// # Panics
    ///
    /// Panics when `n_qubits` is zero.
    pub fn new(n_qubits: usize) -> Self {
        Self::with_classical_bits(n_qubits, 0)
    }

    /// Creates an empty template with an explicitly sized classical register.
    ///
    /// # Panics
    ///
    /// Panics when `n_qubits` is zero.
    pub fn with_classical_bits(n_qubits: usize, n_classical_bits: usize) -> Self {
        assert!(
            n_qubits > 0,
            "ParameterizedCircuit must contain at least one qubit"
        );
        Self {
            n_qubits,
            n_classical_bits,
            operations: Vec::new(),
            parameters: Vec::new(),
        }
    }

    /// Returns the number of qubits in the template.
    pub fn n_qubits(&self) -> usize {
        self.n_qubits
    }

    /// Returns the number of classical bits in the template.
    pub fn n_classical_bits(&self) -> usize {
        self.n_classical_bits
    }

    /// Returns operations in deterministic insertion order.
    pub fn operations(&self) -> &[ParameterizedOperation] {
        &self.operations
    }

    /// Returns unique symbolic parameters in order of first appearance.
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    fn add_single_qubit_gate(&mut self, gate: ParameterizedGateKind, target: usize) -> &mut Self {
        self.validate_target(target);
        self.operations
            .push(ParameterizedOperation::SingleQubit { gate, target });
        self
    }

    fn add_rotation(
        &mut self,
        gate: impl FnOnce(ParameterValue) -> ParameterizedGateKind,
        value: impl Into<ParameterValue>,
        target: usize,
        name: &str,
    ) -> &mut Self {
        self.validate_target(target);
        let value = value.into();
        match &value {
            ParameterValue::Fixed(angle) => {
                assert!(angle.is_finite(), "{name} angle must be finite");
            }
            ParameterValue::Symbol(parameter) => {
                if !self.parameters.contains(parameter) {
                    self.parameters.push(parameter.clone());
                }
            }
        }
        self.add_single_qubit_gate(gate(value), target)
    }

    fn validate_target(&self, target: usize) {
        assert!(
            target < self.n_qubits,
            "Target qubit {} is out of range for {}-qubit circuit",
            target,
            self.n_qubits
        );
    }

    fn validate_control_target(&self, control: usize, target: usize) {
        assert!(
            control < self.n_qubits,
            "Control qubit {} is out of range for {}-qubit circuit",
            control,
            self.n_qubits
        );
        self.validate_target(target);
        assert_ne!(
            control, target,
            "Control and target qubits must be different"
        );
    }

    fn validate_controls_target(&self, controls: &[usize], target: usize) {
        assert!(
            !controls.is_empty(),
            "Controlled gate must have at least one control"
        );
        self.validate_target(target);
        for (index, &control) in controls.iter().enumerate() {
            assert!(
                control < self.n_qubits,
                "Control qubit {} is out of range for {}-qubit circuit",
                control,
                self.n_qubits
            );
            assert_ne!(
                control, target,
                "Target qubit cannot also be a control qubit"
            );
            assert!(
                !controls[..index].contains(&control),
                "Duplicate control qubit {control}"
            );
        }
    }

    /// Adds a Hadamard gate.
    pub fn h(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(ParameterizedGateKind::H, target)
    }

    /// Adds a Pauli-X gate.
    pub fn x(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(ParameterizedGateKind::X, target)
    }

    /// Adds a Pauli-Y gate.
    pub fn y(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(ParameterizedGateKind::Y, target)
    }

    /// Adds a Pauli-Z gate.
    pub fn z(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(ParameterizedGateKind::Z, target)
    }

    /// Adds an S phase gate.
    pub fn s(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(ParameterizedGateKind::S, target)
    }

    /// Adds a T phase gate.
    pub fn t(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(ParameterizedGateKind::T, target)
    }

    /// Adds an identity gate.
    pub fn i(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(ParameterizedGateKind::I, target)
    }

    /// Adds an X-axis rotation with a fixed or named angle.
    pub fn rx(&mut self, value: impl Into<ParameterValue>, target: usize) -> &mut Self {
        self.add_rotation(ParameterizedGateKind::Rx, value, target, "Rx")
    }

    /// Adds a Y-axis rotation with a fixed or named angle.
    pub fn ry(&mut self, value: impl Into<ParameterValue>, target: usize) -> &mut Self {
        self.add_rotation(ParameterizedGateKind::Ry, value, target, "Ry")
    }

    /// Adds a Z-axis rotation with a fixed or named angle.
    pub fn rz(&mut self, value: impl Into<ParameterValue>, target: usize) -> &mut Self {
        self.add_rotation(ParameterizedGateKind::Rz, value, target, "Rz")
    }

    /// Adds a numeric universal single-qubit gate.
    pub fn u(&mut self, theta: f64, phi: f64, lambda: f64, target: usize) -> &mut Self {
        assert!(theta.is_finite(), "U theta angle must be finite");
        assert!(phi.is_finite(), "U phi angle must be finite");
        assert!(lambda.is_finite(), "U lambda angle must be finite");
        self.add_single_qubit_gate(ParameterizedGateKind::U { theta, phi, lambda }, target)
    }

    /// Adds a controlled-X gate using `control, target` order.
    pub fn cnot(&mut self, control: usize, target: usize) -> &mut Self {
        self.validate_control_target(control, target);
        self.operations
            .push(ParameterizedOperation::Cnot { control, target });
        self
    }

    /// Adds a controlled-Z gate using `control, target` order.
    pub fn cz(&mut self, control: usize, target: usize) -> &mut Self {
        self.validate_control_target(control, target);
        self.operations
            .push(ParameterizedOperation::Cz { control, target });
        self
    }

    /// Adds a multi-controlled X gate.
    pub fn mcx(&mut self, controls: &[usize], target: usize) -> &mut Self {
        self.validate_controls_target(controls, target);
        self.operations.push(ParameterizedOperation::Mcx {
            controls: controls.to_vec(),
            target,
        });
        self
    }

    /// Adds a multi-controlled Z gate.
    pub fn mcz(&mut self, controls: &[usize], target: usize) -> &mut Self {
        self.validate_controls_target(controls, target);
        self.operations.push(ParameterizedOperation::Mcz {
            controls: controls.to_vec(),
            target,
        });
        self
    }

    /// Adds a qubit-to-classical-bit measurement.
    pub fn measure(&mut self, qubit: usize, classical_bit: usize) -> &mut Self {
        assert!(
            qubit < self.n_qubits,
            "Qubit {} is out of range for {}-qubit circuit",
            qubit,
            self.n_qubits
        );
        assert!(
            classical_bit < self.n_classical_bits,
            "Classical bit {} is out of range for {}-bit classical register",
            classical_bit,
            self.n_classical_bits
        );
        self.operations.push(ParameterizedOperation::Measure {
            qubit,
            classical_bit,
        });
        self
    }

    /// Binds every named parameter and builds an executable numeric circuit.
    ///
    /// The template remains unchanged. Bindings may be supplied in any order;
    /// operation and parameter order are determined only by the template.
    pub fn bind<'a, I>(&self, bindings: I) -> Result<Circuit, BindingError>
    where
        I: IntoIterator<Item = (&'a Parameter, f64)>,
    {
        let values = self.collect_bindings(bindings)?;
        Ok(self.bind_values(&values, None))
    }

    fn collect_bindings<'a, I>(&self, bindings: I) -> Result<BTreeMap<Parameter, f64>, BindingError>
    where
        I: IntoIterator<Item = (&'a Parameter, f64)>,
    {
        let mut values = BTreeMap::new();
        for (parameter, value) in bindings {
            if !self.parameters.contains(parameter) {
                return Err(BindingError::UnknownParameter(parameter.clone()));
            }
            if !value.is_finite() {
                return Err(BindingError::NonFiniteBinding(parameter.clone()));
            }
            if values.insert(parameter.clone(), value).is_some() {
                return Err(BindingError::DuplicateBinding(parameter.clone()));
            }
        }

        for parameter in &self.parameters {
            if !values.contains_key(parameter) {
                return Err(BindingError::MissingParameter(parameter.clone()));
            }
        }

        Ok(values)
    }

    fn bind_values(
        &self,
        values: &BTreeMap<Parameter, f64>,
        shifted_operation: Option<(usize, f64)>,
    ) -> Circuit {
        let mut circuit = Circuit::with_classical_bits(self.n_qubits, self.n_classical_bits);
        for (index, operation) in self.operations.iter().enumerate() {
            let shift = shifted_operation
                .filter(|(operation_index, _)| *operation_index == index)
                .map_or(0.0, |(_, shift)| shift);
            bind_operation(&mut circuit, operation, values, shift);
        }
        circuit
    }

    pub(crate) fn bind_with_operation_shift(
        &self,
        values: &BTreeMap<Parameter, f64>,
        operation_index: usize,
        shift: f64,
    ) -> Circuit {
        self.bind_values(values, Some((operation_index, shift)))
    }
}

fn resolve(value: &ParameterValue, bindings: &BTreeMap<Parameter, f64>, shift: f64) -> f64 {
    match value {
        ParameterValue::Fixed(value) => *value + shift,
        ParameterValue::Symbol(parameter) => bindings[parameter] + shift,
    }
}

fn bind_operation(
    circuit: &mut Circuit,
    operation: &ParameterizedOperation,
    bindings: &BTreeMap<Parameter, f64>,
    shift: f64,
) {
    match operation {
        ParameterizedOperation::SingleQubit { gate, target } => match gate {
            ParameterizedGateKind::X => {
                circuit.x(*target);
            }
            ParameterizedGateKind::Y => {
                circuit.y(*target);
            }
            ParameterizedGateKind::Z => {
                circuit.z(*target);
            }
            ParameterizedGateKind::H => {
                circuit.h(*target);
            }
            ParameterizedGateKind::S => {
                circuit.s(*target);
            }
            ParameterizedGateKind::T => {
                circuit.t(*target);
            }
            ParameterizedGateKind::I => {
                circuit.i(*target);
            }
            ParameterizedGateKind::Rx(value) => {
                circuit.rx(resolve(value, bindings, shift), *target);
            }
            ParameterizedGateKind::Ry(value) => {
                circuit.ry(resolve(value, bindings, shift), *target);
            }
            ParameterizedGateKind::Rz(value) => {
                circuit.rz(resolve(value, bindings, shift), *target);
            }
            ParameterizedGateKind::U { theta, phi, lambda } => {
                circuit.u(*theta, *phi, *lambda, *target);
            }
        },
        ParameterizedOperation::Cnot { control, target } => {
            circuit.cnot(*control, *target);
        }
        ParameterizedOperation::Cz { control, target } => {
            circuit.cz(*control, *target);
        }
        ParameterizedOperation::Mcx { controls, target } => {
            circuit.mcx(controls, *target);
        }
        ParameterizedOperation::Mcz { controls, target } => {
            circuit.mcz(controls, *target);
        }
        ParameterizedOperation::Measure {
            qubit,
            classical_bit,
        } => {
            circuit.measure(*qubit, *classical_bit);
        }
    }
}
