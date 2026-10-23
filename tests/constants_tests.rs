use eigenon::engine::constants::{
    gate_cnot, gate_h, gate_rx, gate_ry, gate_rz, gate_s, gate_t, gate_u, gate_x, gate_y, gate_z,
    identity, q0, q1,
};
use ndarray::{Array2, array};
use num_complex::Complex64;
use std::f64::consts::PI;

fn to_c64(re: f64) -> Complex64 {
    Complex64::new(re, 0.0)
}

#[test]
fn test_gate_x_logic() {
    // X|0> = |1>
    let result = gate_x().dot(&q0());
    assert_eq!(result, &q1());
}

#[test]
fn test_gate_h_reversibility() {
    // H(H|0>) = |0>
    let once = gate_h().dot(&q0());
    let twice = gate_h().dot(&once);

    // Use tolerance check for float precision
    for (a, b) in twice.iter().zip(q0().iter()) {
        assert!((a - b).norm() < 1e-10);
    }
}

#[test]
fn test_gate_h_is_unitary() {
    // Norm of H|0> must be 1.0
    let result = gate_h().dot(&q0());
    let norm: f64 = result.mapv(|x| x.norm_sqr()).sum();
    assert!((norm - 1.0).abs() < 1e-10);
}

#[test]
fn test_cnot_logic() {
    let input = array![to_c64(0.0), to_c64(0.0), to_c64(1.0), to_c64(0.0),];
    let output = gate_cnot().dot(&input);
    let expected = array![to_c64(0.0), to_c64(0.0), to_c64(0.0), to_c64(1.0),];
    assert_eq!(output, expected);
    let input_01 = array![to_c64(0.0), to_c64(1.0), to_c64(0.0), to_c64(0.0),];
    let output_01 = gate_cnot().dot(&input_01);
    assert_eq!(output_01, input_01);
}

#[test]
fn test_identity_logic() {
    assert_eq!(identity().dot(&q0()), q0());
}

#[test]
fn test_gate_z_logic() {
    let input = array![to_c64(0.0), to_c64(1.0)];
    let output = gate_z().dot(&input);
    let expected = array![to_c64(0.0), to_c64(-1.0)];
    for (a, b) in output.iter().zip(expected.iter()) {
        assert!((*a - *b).norm() < 1e-10);
    }
}

#[test]
fn test_gate_z_preserves_probabilities() {
    let state = gate_h().dot(&q0());
    let state_z = gate_z().dot(&state);

    let probs_before: Vec<f64> = state.iter().map(|x| x.norm_sqr()).collect();
    let probs_after: Vec<f64> = state_z.iter().map(|x| x.norm_sqr()).collect();

    for (a, b) in probs_before.iter().zip(probs_after.iter()) {
        assert!((a - b).abs() < 1e-10);
    }
}

// Helper to compare complex matrices with precision delta
fn assert_matrix_eq(a: &Array2<Complex64>, b: &Array2<Complex64>) {
    let delta = 1e-15;
    assert!(
        a.iter().zip(b.iter()).all(|(x, y)| (x - y).norm() < delta),
        "Matrices are not equal!\nLeft: {:?}\nRight: {:?}",
        a,
        b
    );
}

#[test]
fn test_pauli_identities() {
    // Identity: iXZ = Y
    let i = Complex64::i();
    let xz = gate_x().dot(&gate_z());
    let ixz = xz.mapv(|val| val * i);
    assert_matrix_eq(&ixz, &gate_y());
}

#[test]
fn test_phase_hierarchy() {
    // T^2 = S
    let t_squared = gate_t().dot(&gate_t());
    assert_matrix_eq(&t_squared, &gate_s());

    // S^2 = Z
    let s_squared = gate_s().dot(&gate_s());
    assert_matrix_eq(&s_squared, &gate_z());
}

#[test]
fn test_hadamard_conjugation() {
    // HZH = X (Hadamard transforms a Z-basis to X-basis)
    let hzh = gate_h().dot(&gate_z()).dot(&gate_h());
    assert_matrix_eq(&hzh, &gate_x());
}

#[test]
fn test_pi_rotation_matrices() {
    let zero = Complex64::new(0.0, 0.0);
    let i = Complex64::i();

    assert_matrix_eq(&gate_rx(PI), &array![[zero, -i], [-i, zero]]);
    assert_matrix_eq(
        &gate_ry(PI),
        &array![[zero, Complex64::new(-1.0, 0.0)], [1.0.into(), zero]],
    );
    assert_matrix_eq(&gate_rz(PI), &array![[-i, zero], [zero, i]]);
}

#[test]
fn test_u_gate_can_represent_pauli_x() {
    assert_matrix_eq(&gate_u(PI, 0.0, PI), &gate_x());
}
