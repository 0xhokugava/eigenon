use ndarray::{Array1, ArrayD};
use num_complex::Complex64;
use rand::*;
use std::collections::HashMap;

/// Performs a quantum measurement on a state vector of any size.
/// Collapses the superposition into a basis state based on Born's rule (|ψ|²).
/// Uses cumulative probability to select an outcome from the full distribution.
pub fn measure(arr: &Array1<Complex64>) -> usize {
    let mut rng = rng();
    let dice = rng.random_range(0.0..=1.0);

    let mut cumulative_probability = 0.0;

    for (index, amplitude) in arr.iter().enumerate() {
        cumulative_probability += amplitude.norm_sqr();
        if dice < cumulative_probability {
            return index;
        }
    }

    arr.len() - 1
}

/// Measures a single qubit within a multi-qubit state vector, in place.
///
/// Sampling the full basis outcome via [`measure`] and reading off the bit at
/// `qubit` is equivalent to sampling from that qubit's marginal distribution,
/// since the marginal is the sum of the joint distribution over every other
/// qubit. The state is then collapsed to the subspace consistent with the
/// observed bit and renormalized, as required by the Born rule.
///
/// Returns the observed classical bit (0 or 1).
pub fn measure_qubit_inplace(state: &mut ArrayD<Complex64>, qubit: usize) -> u8 {
    let flat = Array1::from_iter(state.iter().cloned());
    let outcome_index = measure(&flat);
    let bit = ((outcome_index >> qubit) & 1) as u8;

    for (index, amplitude) in state.iter_mut().enumerate() {
        if ((index >> qubit) & 1) as u8 != bit {
            *amplitude = Complex64::new(0.0, 0.0);
        }
    }

    let norm = state
        .iter()
        .map(|amplitude| amplitude.norm_sqr())
        .sum::<f64>()
        .sqrt();

    for amplitude in state.iter_mut() {
        *amplitude /= norm;
    }

    bit
}

/// Runs multiple measurement simulations (shots) to gather statistics.
/// Returns a HashMap containing the probability distribution across all detected states.
pub fn test_measure(state: &Array1<Complex64>, shots: usize) -> HashMap<usize, f64> {
    let mut counts = HashMap::new();

    for _ in 0..shots {
        let res = measure(state);
        *counts.entry(res).or_insert(0.0) += 1.0;
    }

    for count in counts.values_mut() {
        *count = (*count / shots as f64) * 100.0;
    }

    counts
}
