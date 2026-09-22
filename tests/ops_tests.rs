use eigenon::engine::constants::{
    gate_cnot, gate_h, gate_s, gate_t, gate_x, gate_y, gate_z, identity, q0, q1,
};
use eigenon::engine::ops::{
    apply_cnot_inplace, apply_controlled_single_qubit_gate_inplace, apply_gate_inplace,
    apply_k_qubit_gate_inplace, apply_mcx_inplace, apply_mcz_inplace, tensor_product,
};
use eigenon::engine::utils::{assert_states_close, build_full_operator, q0_n, to_c64};
use ndarray::linalg::Dot;
use ndarray::{Array1, Array2, ArrayD};
use num_complex::Complex64;

#[test]
fn test_tensor_product() {
    let c_array = |re_vals: Vec<f64>| -> Array1<Complex64> {
        Array1::from_vec(re_vals.into_iter().map(to_c64).collect())
    };
    // Test |0> ⊗ |0> = |00> (index 0 is 1.0)
    let res00 = tensor_product(&q0(), &q0())
        .into_dimensionality::<ndarray::Ix1>()
        .expect("Should be a 1D vector");

    assert_eq!(res00, c_array(vec![1.0, 0.0, 0.0, 0.0]));

    // Test |0> ⊗ |1> = |01> (index 1 is 1.0)
    let res01 = tensor_product(&q0(), &q1())
        .into_dimensionality::<ndarray::Ix1>()
        .expect("Should be a 1D vector");
    assert_eq!(res01, c_array(vec![0.0, 1.0, 0.0, 0.0]));

    // Test |1> ⊗ |0> = |10> (index 2 is 1.0)
    let res10 = tensor_product(&q1(), &q0())
        .into_dimensionality::<ndarray::Ix1>()
        .expect("Should be a 1D vector");
    assert_eq!(res10, c_array(vec![0.0, 0.0, 1.0, 0.0]));

    // Test |1> ⊗ |1> = |11> (index 3 is 1.0)
    let res11 = tensor_product(&q1(), &q1())
        .into_dimensionality::<ndarray::Ix1>()
        .expect("Should be a 1D vector");
    assert_eq!(res11, c_array(vec![0.0, 0.0, 0.0, 1.0]));
}

#[test]
fn test_tensor_product_order_matters() {
    let res_01 = tensor_product(&q0(), &q1());
    let res_10 = tensor_product(&q1(), &q0());
    // |01> should NOT be equal to |10>
    assert_ne!(res_01, res_10);
}

#[test]
fn test_tensor_product_empty() {
    let empty_arr = Array1::from(vec![]);
    let res = tensor_product(&empty_arr, &q0());
    // The result should be empty if one of the inputs is empty
    assert_eq!(res.len(), 0);
}

#[test]
fn test_tensor_product_three_qubits() {
    // (|1> ⊗ |0>) ⊗ |0> = |100>
    // |100> in 8-dimensional space is index 4
    let res_2q = tensor_product(&q1(), &q0());
    let res_3q = tensor_product(&res_2q, &q0())
        .into_dimensionality::<ndarray::Ix1>()
        .expect("Three-qubit state must be a 1D vector");
    let mut expected = Array1::<Complex64>::zeros(8);
    expected[4] = to_c64(1.0);

    assert_eq!(res_3q, expected);
    assert_eq!(res_3q.len(), 8);
}

#[test]
fn test_apply_gate_inplace() {
    let state = tensor_product(&q0(), &q0());
    let gates = vec![
        (gate_h(), "H"),
        (gate_x(), "X"),
        (gate_y(), "Y"),
        (gate_z(), "Z"),
        (gate_s(), "S"),
        (gate_t(), "T"),
    ];

    let identity = identity();
    let n_qubits = state.len().ilog2() as usize;

    for target in 0..n_qubits {
        for (gate, name) in &gates {
            let mut state_inplace = state.clone();

            let full = match target {
                0 => tensor_product(&identity, gate),
                1 => tensor_product(gate, &identity),
                _ => unreachable!(),
            };

            let expected = full.dot(&state);
            apply_gate_inplace(&mut state_inplace, gate, target);
            for (i, (a, b)) in state_inplace.iter().zip(expected.iter()).enumerate() {
                assert!(
                    (a - b).norm() < 1e-10,
                    "Gate {} failed on target {}, index {}",
                    name,
                    target,
                    i
                );
            }
        }
    }
}

#[test]
fn qubit_zero_is_least_significant_bit() {
    let mut state = q0_n(3);

    apply_gate_inplace(&mut state, &gate_x(), 0);

    let state = state
        .into_dimensionality::<ndarray::Ix1>()
        .expect("State must be a 1D vector");

    let mut expected = Array1::<Complex64>::zeros(8);
    expected[1] = to_c64(1.0);

    assert_eq!(state, expected);
}

#[test]
fn highest_qubit_maps_to_highest_basis_bit() {
    let mut state = q0_n(3);

    apply_gate_inplace(&mut state, &gate_x(), 2);

    let state = state
        .into_dimensionality::<ndarray::Ix1>()
        .expect("State must be a 1D vector");

    let mut expected = Array1::<Complex64>::zeros(8);
    expected[4] = to_c64(1.0);

    assert_eq!(state, expected);
}

#[test]
fn test_apply_cnot_inplace() {
    let state = tensor_product(&q0(), &q0());

    let n_qubits = state.len().ilog2() as usize;

    for control in 0..n_qubits {
        for target in 0..n_qubits {
            if control == target {
                continue;
            }

            let mut state_inplace = state.clone();

            let full = match (control, target) {
                (0, 1) => gate_cnot().clone(),
                (1, 0) => {
                    let mut m = Array2::zeros((4, 4));
                    // |00> -> |00>
                    m[[0, 0]] = 1.0.into();
                    // |01> -> |11>
                    m[[3, 1]] = 1.0.into();
                    // |10> -> |10>
                    m[[2, 2]] = 1.0.into();
                    // |11> -> |01>
                    m[[1, 3]] = 1.0.into();
                    m
                }
                _ => unreachable!(),
            };

            let state_vec = state.clone().into_dimensionality::<ndarray::Ix1>().unwrap();

            let expected = full.dot(&state_vec);

            apply_cnot_inplace(&mut state_inplace, control, target);

            let state_inplace_vec = state_inplace.into_dimensionality::<ndarray::Ix1>().unwrap();

            for (i, (a, b)) in state_inplace_vec.iter().zip(expected.iter()).enumerate() {
                assert!(
                    (a - b).norm() < 1e-10,
                    "CNOT failed for control {}, target {}, index {}",
                    control,
                    target,
                    i
                );
            }
        }
    }
}

#[test]
fn test_k_qubit_gate_general() {
    let n = 3;
    let targets = vec![1, 0];

    let state_initial = q0_n(n);
    let gate = gate_cnot();

    // --- MATRIX BASELINE ---
    let full = build_full_operator(&gate, &targets, n);

    let state_vec = state_initial
        .clone()
        .into_dimensionality::<ndarray::Ix1>()
        .unwrap();

    let expected = full.dot(&state_vec).into_dyn();

    // --- IN-PLACE ---
    let mut state_inplace = state_initial.clone();
    apply_k_qubit_gate_inplace(&mut state_inplace, &gate, &targets);

    // --- ASSERT ---
    assert_states_close(&state_inplace, &expected);
}

#[test]
fn test_k_qubit_gate_k1() {
    let n = 3;
    let target = 1;

    let state_initial = q0_n(n);
    let gate = gate_h();

    let targets = vec![target];

    // --- MATRIX BASELINE ---
    let full = build_full_operator(&gate, &targets, n);

    let state_vec = state_initial
        .clone()
        .into_dimensionality::<ndarray::Ix1>()
        .unwrap();

    let expected = full.dot(&state_vec).into_dyn();

    // --- IN-PLACE ---
    let mut state_inplace = state_initial.clone();
    apply_k_qubit_gate_inplace(&mut state_inplace, &gate, &targets);

    // --- ASSERT ---
    assert_states_close(&state_inplace, &expected);
}

#[test]
fn test_k_qubit_gate_non_adjacent_targets() {
    let n = 3;
    let targets = vec![2, 0];

    let state_initial = q0_n(n);
    let gate = gate_cnot();

    // --- MATRIX BASELINE ---
    let full = build_full_operator(&gate, &targets, n);

    let state_vec = state_initial
        .clone()
        .into_dimensionality::<ndarray::Ix1>()
        .unwrap();

    let expected = full.dot(&state_vec).into_dyn();

    // --- IN-PLACE ---
    let mut state_inplace = state_initial.clone();
    apply_k_qubit_gate_inplace(&mut state_inplace, &gate, &targets);

    // --- ASSERT ---
    assert_states_close(&state_inplace, &expected);
}

#[test]
fn test_k_qubit_gate_k3() {
    let n = 3;
    let targets = vec![2, 1, 0];

    let state_initial = q0_n(n);
    let dim = 1 << targets.len();
    let mut gate = Array2::<Complex64>::zeros((dim, dim));

    for col in 0..dim {
        let row = (col + 1) % dim;
        gate[[row, col]] = Complex64::new(1.0, 0.0);
    }

    // --- MATRIX BASELINE ---
    let full = build_full_operator(&gate, &targets, n);

    let state_vec = state_initial
        .clone()
        .into_dimensionality::<ndarray::Ix1>()
        .unwrap();

    let expected = full.dot(&state_vec).into_dyn();

    // --- IN-PLACE ---
    let mut state_inplace = state_initial.clone();
    apply_k_qubit_gate_inplace(&mut state_inplace, &gate, &targets);

    // --- ASSERT ---
    assert_states_close(&state_inplace, &expected);
}

fn basis_state(n_qubits: usize, index: usize) -> ArrayD<Complex64> {
    let mut state = Array1::zeros(1usize << n_qubits);
    state[index] = Complex64::new(1.0, 0.0);
    state.into_dyn()
}

#[test]
fn controlled_x_flips_target_when_control_is_one() {
    let mut state = basis_state(2, 1);
    apply_controlled_single_qubit_gate_inplace(&mut state, &gate_x(), &[0], 1);
    assert_eq!(state[3], Complex64::new(1.0, 0.0));
}

#[test]
#[should_panic]
fn rejects_empty_controls() {
    let mut state = basis_state(2, 0);
    apply_controlled_single_qubit_gate_inplace(&mut state, &gate_x(), &[], 1);
}

#[test]
fn controlled_x_does_nothing_when_not_all_controls_are_one() {
    let mut state = basis_state(3, 1);
    apply_controlled_single_qubit_gate_inplace(&mut state, &gate_x(), &[0, 1], 2);

    assert_eq!(state[1], Complex64::new(1.0, 0.0));
    assert_eq!(state[5], Complex64::new(0.0, 0.0));
}

#[test]
fn controlled_x_supports_multiple_controls() {
    let mut state = basis_state(3, 3);

    apply_controlled_single_qubit_gate_inplace(&mut state, &gate_x(), &[0, 1], 2);

    assert_eq!(state[3], Complex64::new(0.0, 0.0));
    assert_eq!(state[7], Complex64::new(1.0, 0.0));
}

#[test]
fn controlled_z_applies_phase_when_controls_are_one_and_target_is_one() {
    let mut state = basis_state(3, 7);
    apply_controlled_single_qubit_gate_inplace(&mut state, &gate_z(), &[0, 1], 2);
    assert_eq!(state[7], Complex64::new(-1.0, 0.0));
}

#[test]
fn mcx_flips_target_when_all_controls_are_one() {
    let mut state = basis_state(3, 3);
    apply_mcx_inplace(&mut state, &[0, 1], 2);

    assert_eq!(state[3], Complex64::new(0.0, 0.0));
    assert_eq!(state[7], Complex64::new(1.0, 0.0));
}

#[test]
fn mcz_flips_phase_when_all_controls_are_one_and_target_is_one() {
    let mut state = basis_state(3, 7);
    apply_mcz_inplace(&mut state, &[0, 1], 2);
    assert_eq!(state[7], Complex64::new(-1.0, 0.0));
}
