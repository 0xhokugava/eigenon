use crate::circuit::operation::{GateKind, Operation};
use crate::engine::constants::{
    gate_cnot, gate_cz, gate_h, gate_s, gate_t, gate_x, gate_y, gate_z, identity,
};
use crate::engine::measurement::measure_qubit_inplace;
use crate::engine::ops::{apply_controlled_single_qubit_gate_inplace, apply_k_qubit_gate_inplace};
use crate::engine::utils::q0_n;
use ndarray::{Array2, ArrayD};
use num_complex::Complex64;

/// A quantum circuit containing semantic operations in execution order.
///
/// Gate methods add operations and return the same circuit, so calls can be
/// chained. Calling [`Circuit::run`] starts from `|0...0>` and executes every
/// scheduled operation.
///
/// Qubit `0` is the least significant bit. Controlled gates use
/// `control, target` argument order.
pub struct Circuit {
    n_qubits: usize,
    n_classical_bits: usize,
    operations: Vec<Operation>,
}

/// Maps a semantic single-qubit gate kind to its matrix representation.
///
/// Matrix construction is kept at the execution boundary so that the circuit
/// itself preserves gate identity for export and inspection.
fn matrix_for_gate(gate: GateKind) -> Array2<Complex64> {
    match gate {
        GateKind::X => gate_x(),
        GateKind::Y => gate_y(),
        GateKind::Z => gate_z(),
        GateKind::H => gate_h(),
        GateKind::S => gate_s(),
        GateKind::T => gate_t(),
        GateKind::I => identity(),
    }
}

impl Circuit {
    /// Creates an empty circuit with no classical register.
    ///
    /// # Panics
    ///
    /// Panics when `n_qubits` is zero.
    pub fn new(n_qubits: usize) -> Self {
        assert!(n_qubits > 0, "Circuit must contain at least one qubit");
        Self {
            n_qubits,
            n_classical_bits: 0,
            operations: Vec::new(),
        }
    }

    /// Creates an empty circuit with an explicitly sized classical register.
    ///
    /// The classical register is used by measurement operations and OpenQASM
    /// export. It may be smaller, equal to or larger than the quantum register.
    ///
    /// # Panics
    ///
    /// Panics when `n_qubits` is zero.
    pub fn with_classical_bits(n_qubits: usize, n_classical_bits: usize) -> Self {
        assert!(n_qubits > 0, "Circuit must contain at least one qubit");

        Self {
            n_qubits,
            n_classical_bits,
            operations: Vec::new(),
        }
    }

    /// Schedules a single-qubit gate without executing it immediately.
    ///
    /// The operation is stored semantically as `(gate kind, target)`.
    /// Matrix application happens later in `run()`.
    fn add_single_qubit_gate(&mut self, gate: GateKind, target: usize) -> &mut Self {
        assert!(
            target < self.n_qubits,
            "Target qubit {} is out of range for {}-qubit circuit",
            target,
            self.n_qubits
        );

        self.operations
            .push(Operation::SingleQubit { gate, target });

        self
    }

    /// Applies a phase flip to the all-one's basis state.
    ///
    /// For one qubit this is just `Z`.
    /// For multiple qubits this is implemented as an MCZ with the highest-index
    /// qubit as a target and all lower-index qubits as controls.
    fn phase_flip_all_ones(&mut self) -> &mut Self {
        if self.n_qubits == 1 {
            self.z(0);
            return self;
        }

        let target = self.n_qubits - 1;
        let controls: Vec<usize> = (0..target).collect();

        self.mcz(&controls, target)
    }

    fn validate_control_target(&self, control: usize, target: usize) {
        assert!(
            control < self.n_qubits,
            "Control qubit {} is out of range for {}-qubit circuit",
            control,
            self.n_qubits
        );

        assert!(
            target < self.n_qubits,
            "Target qubit {} is out of range for {}-qubit circuit",
            target,
            self.n_qubits
        );

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

        assert!(
            target < self.n_qubits,
            "Target qubit {} is out of range for {}-qubit circuit",
            target,
            self.n_qubits
        );

        for &control in controls {
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
        }

        for i in 0..controls.len() {
            for j in (i + 1)..controls.len() {
                assert_ne!(
                    controls[i], controls[j],
                    "Duplicate control qubit {}",
                    controls[i]
                );
            }
        }
    }

    /// Returns the number of qubits in the circuit.
    pub fn n_qubits(&self) -> usize {
        self.n_qubits
    }

    /// Returns the number of classical bits allocated for the circuit.
    pub fn n_classical_bits(&self) -> usize {
        self.n_classical_bits
    }

    /// Returns the scheduled semantic operations.
    ///
    /// This is intended for inspection and export layers.
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    /// Adds a Hadamard gate on `target`.
    ///
    /// # Panics
    ///
    /// Panics when `target` is outside the quantum register.
    pub fn h(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(GateKind::H, target)
    }

    /// Adds a Pauli-X gate on `target`.
    ///
    /// # Panics
    ///
    /// Panics when `target` is outside the quantum register.
    pub fn x(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(GateKind::X, target)
    }

    /// Adds a Pauli-Y gate on `target`.
    ///
    /// # Panics
    ///
    /// Panics when `target` is outside the quantum register.
    pub fn y(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(GateKind::Y, target)
    }

    /// Adds a Pauli-Z gate on `target`.
    ///
    /// # Panics
    ///
    /// Panics when `target` is outside the quantum register.
    pub fn z(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(GateKind::Z, target)
    }

    /// Adds an S phase gate on `target`.
    ///
    /// # Panics
    ///
    /// Panics when `target` is outside the quantum register.
    pub fn s(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(GateKind::S, target)
    }

    /// Adds a T phase gate on `target`.
    ///
    /// # Panics
    ///
    /// Panics when `target` is outside the quantum register.
    pub fn t(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(GateKind::T, target)
    }

    /// Adds an identity gate on `target`.
    ///
    /// # Panics
    ///
    /// Panics when `target` is outside the quantum register.
    pub fn i(&mut self, target: usize) -> &mut Self {
        self.add_single_qubit_gate(GateKind::I, target)
    }

    /// Adds a Hadamard gate to every qubit in ascending index order.
    pub fn h_all(&mut self) -> &mut Self {
        for qubit in 0..self.n_qubits {
            self.h(qubit);
        }

        self
    }

    /// Adds a controlled-X gate using `control, target` argument order.
    ///
    /// # Panics
    ///
    /// Panics when either index is outside the quantum register or when
    /// `control` and `target` are the same qubit.
    pub fn cnot(&mut self, control: usize, target: usize) -> &mut Self {
        self.validate_control_target(control, target);
        self.operations.push(Operation::Cnot { control, target });
        self
    }

    /// Adds a controlled-Z gate using `control, target` argument order.
    ///
    /// # Panics
    ///
    /// Panics when either index is outside the quantum register or when
    /// `control` and `target` are the same qubit.
    pub fn cz(&mut self, control: usize, target: usize) -> &mut Self {
        self.validate_control_target(control, target);
        self.operations.push(Operation::Cz { control, target });
        self
    }

    /// Adds a multi-controlled X gate.
    ///
    /// The target is flipped only when every qubit in `controls` is `1`.
    ///
    /// # Panics
    ///
    /// Panics when `controls` is empty, an index is outside the quantum
    /// register, a control is duplicated or the target is also a control.
    pub fn mcx(&mut self, controls: &[usize], target: usize) -> &mut Self {
        self.validate_controls_target(controls, target);
        self.operations.push(Operation::Mcx {
            controls: controls.to_vec(),
            target,
        });
        self
    }

    /// Adds a multi-controlled Z gate.
    ///
    /// A phase flip is applied only when every control and the target are `1`.
    ///
    /// # Panics
    ///
    /// Panics when `controls` is empty, an index is outside the quantum
    /// register, a control is duplicated or the target is also a control.
    pub fn mcz(&mut self, controls: &[usize], target: usize) -> &mut Self {
        self.validate_controls_target(controls, target);
        self.operations.push(Operation::Mcz {
            controls: controls.to_vec(),
            target,
        });
        self
    }

    /// Builds a gate-level phase oracle for the selected basis state.
    ///
    /// The target state is mapped to the all-one's state using X gates,
    /// then `phase_flip_all_ones()` applies the sign flip, and the X gates
    /// are undone afterward.
    ///
    /// Qubit indexing follows the simulator convention: qubit 0 is the
    /// least significant bit of `target_index`.
    ///
    /// # Panics
    ///
    /// Panics when `target_index` cannot be represented by the circuit's
    /// quantum register.
    pub fn phase_oracle(&mut self, target_index: usize) -> &mut Self {
        assert!(
            target_index < (1usize << self.n_qubits),
            "Target index {} is out of range for {} qubits",
            target_index,
            self.n_qubits
        );

        for qubit in 0..self.n_qubits {
            if ((target_index >> qubit) & 1) == 0 {
                self.x(qubit);
            }
        }

        self.phase_flip_all_ones();

        for qubit in 0..self.n_qubits {
            if ((target_index >> qubit) & 1) == 0 {
                self.x(qubit);
            }
        }

        self
    }

    /// Builds the Grover diffusion operator at gate level.
    ///
    /// The implementation follows the standard H / X / phase-flip / X / H
    /// construction and schedules all operations through the Circuit API.
    pub fn diffusion(&mut self) -> &mut Self {
        self.h_all();
        for qubit in 0..self.n_qubits {
            self.x(qubit);
        }

        self.phase_flip_all_ones();
        for qubit in 0..self.n_qubits {
            self.x(qubit);
        }

        self.h_all();

        self
    }

    /// Schedules a measurement from a quantum bit into a classical bit.
    ///
    /// Both indices are validated against the circuit's quantum and classical
    /// register sizes. The measurement is stored as a semantic operation and
    /// is not executed immediately. The `classical_bit` mapping is preserved
    /// for operation inspection and OpenQASM export.
    ///
    /// [`Circuit::run`] collapses the quantum state when it reaches this
    /// operation, but it does not return the value of the classical bit.
    ///
    /// # Panics
    ///
    /// Panics when either index is outside its corresponding register.
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

        self.operations.push(Operation::Measure {
            qubit,
            classical_bit,
        });

        self
    }

    /// Schedules measurement of every qubit into the classical bit
    /// with the same index.
    ///
    /// Requires the classical register to contain at least as many bits
    /// as the quantum register.
    ///
    /// # Panics
    ///
    /// Panics when the classical register is smaller than the quantum
    /// register.
    pub fn measure_all(&mut self) -> &mut Self {
        assert!(
            self.n_classical_bits >= self.n_qubits,
            "measure_all requires at least as many classical bits as qubits"
        );

        for qubit in 0..self.n_qubits {
            self.measure(qubit, qubit);
        }

        self
    }

    /// Executes all scheduled operations on the initial `|0...0>` state.
    ///
    /// The circuit stores semantic operations. During execution each operation
    /// is mapped to the corresponding matrix or controlled-gate engine call.
    /// Operations are applied sequentially in insertion order. A `Measure`
    /// operation collapses the state to the subspace consistent with the
    /// sampled outcome, per the Born rule.
    ///
    /// This method returns only the final quantum state. It does not return a
    /// classical register. Each call starts again from `|0...0>`, so circuits
    /// containing measurement operations may produce different final states
    /// on repeated runs.
    pub fn run(&self) -> ArrayD<Complex64> {
        let mut state = q0_n(self.n_qubits);

        for operation in &self.operations {
            match operation {
                Operation::SingleQubit { gate, target } => {
                    let matrix = matrix_for_gate(*gate);
                    apply_k_qubit_gate_inplace(&mut state, &matrix, &[*target]);
                }

                Operation::Cnot { control, target } => {
                    let matrix = gate_cnot();
                    apply_k_qubit_gate_inplace(&mut state, &matrix, &[*control, *target]);
                }

                Operation::Cz { control, target } => {
                    let matrix = gate_cz();
                    apply_k_qubit_gate_inplace(&mut state, &matrix, &[*control, *target]);
                }

                Operation::Mcx { controls, target } => {
                    let matrix = gate_x();
                    apply_controlled_single_qubit_gate_inplace(
                        &mut state, &matrix, controls, *target,
                    );
                }

                Operation::Mcz { controls, target } => {
                    let matrix = gate_z();
                    apply_controlled_single_qubit_gate_inplace(
                        &mut state, &matrix, controls, *target,
                    );
                }

                Operation::Measure { qubit, .. } => {
                    measure_qubit_inplace(&mut state, *qubit);
                }
            }
        }

        state
    }
}
