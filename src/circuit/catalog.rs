//! Reusable constructors for common quantum circuits.

use crate::algorithms::deutsch::DeutschOracle;
use crate::algorithms::deutsch_jozsa::DeutschJozsaOracle;
use crate::algorithms::grover_search::recommended_grover_steps;
use crate::circuit::core::Circuit;

/// One of the four maximally entangled two-qubit Bell states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BellState {
    /// `(|00> + |11>) / sqrt(2)`.
    PhiPlus,
    /// `(|00> - |11>) / sqrt(2)`.
    PhiMinus,
    /// `(|01> + |10>) / sqrt(2)`.
    PsiPlus,
    /// `(|01> - |10>) / sqrt(2)`.
    PsiMinus,
}

pub(crate) const QUERY: usize = 1;
pub(crate) const ANSWER: usize = 0;

/// Builds a two-qubit circuit that prepares the selected Bell state.
pub fn bell(state: BellState) -> Circuit {
    let mut circuit = Circuit::new(2);

    circuit.h(0);
    circuit.cnot(0, 1);

    if matches!(state, BellState::PsiPlus | BellState::PsiMinus) {
        circuit.x(1);
    }

    if matches!(state, BellState::PhiMinus | BellState::PsiMinus) {
        circuit.z(0);
    }

    circuit
}

/// Builds the two-qubit circuit for the selected Deutsch oracle.
pub fn deutsch_circuit(oracle: DeutschOracle) -> Circuit {
    let mut circuit = Circuit::new(2);

    circuit.x(ANSWER);
    circuit.h(QUERY);
    circuit.h(ANSWER);

    oracle.apply(&mut circuit);

    circuit.h(QUERY);

    circuit
}

/// Builds a Deutsch-Jozsa circuit with `num_query_qubits` query qubits.
///
/// Qubit `0` is the answer qubit. Query qubits occupy indices starting at `1`.
///
/// # Panics
///
/// Panics when `num_query_qubits` is zero.
pub fn deutsch_jozsa_circuit(num_query_qubits: usize, oracle: DeutschJozsaOracle) -> Circuit {
    assert!(
        num_query_qubits > 0,
        "Deutsch-Jozsa requires at least one query qubit"
    );
    let total_qubits = num_query_qubits + 1;
    let mut circuit = Circuit::new(total_qubits);
    circuit.x(ANSWER);

    crate::algorithms::deutsch_jozsa::apply_h_to_queries(&mut circuit, num_query_qubits);

    circuit.h(ANSWER);

    oracle.apply(&mut circuit, num_query_qubits);

    crate::algorithms::deutsch_jozsa::apply_h_to_queries(&mut circuit, num_query_qubits);

    circuit
}

/// Builds a Grover search circuit for one marked basis-state index.
///
/// The recommended number of Grover iterations is selected automatically.
///
/// # Panics
///
/// Panics when `num_qubits` is zero or `target_index` is outside the state
/// space represented by the circuit.
pub fn grover_circuit(num_qubits: usize, target_index: usize) -> Circuit {
    assert!(num_qubits > 0);
    assert!(target_index < (1usize << num_qubits));

    let grover_steps = recommended_grover_steps(num_qubits);
    let mut circuit = Circuit::new(num_qubits);

    circuit.h_all();

    for _ in 0..grover_steps {
        circuit.phase_oracle(target_index);
        circuit.diffusion();
    }

    circuit
}
