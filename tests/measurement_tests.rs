use eigenon::engine::constants::{q0, q1};
use eigenon::engine::measurement::{measure, sample_measurements_with_seed, test_measure};
use ndarray::array;
use num_complex::Complex64;
use std::f64::consts::FRAC_1_SQRT_2;

fn to_c64(re: f64) -> Complex64 {
    Complex64::new(re, 0.0)
}

#[test]
fn test_measure_deterministic() {
    // |0> must always collapse to 0
    for _ in 0..1000 {
        assert_eq!(measure(&q0()), 0);
    }

    // |1> must always collapse to 1
    for _ in 0..1000 {
        assert_eq!(measure(&q1()), 1);
    }
}

#[test]
fn test_measure_statistics() {
    let h_state = array![FRAC_1_SQRT_2, FRAC_1_SQRT_2].map(|&x| to_c64(x));
    let shots = 10_000;

    let results = test_measure(&h_state, shots);

    let p0 = *results.get(&0).unwrap_or(&0.0);
    let p1 = *results.get(&1).unwrap_or(&0.0);

    // For Hadamard expectations roughly 50/50
    // Check if it's within ±10% to avoid rare random fails
    assert!(p1 > 40.0 && p1 < 60.0, "p1 was {}", p1);
    assert!(p0 > 40.0 && p0 < 60.0, "p0 was {}", p0);

    // Sum must be exactly 100% or very close
    let total_sum: f64 = results.values().sum();
    assert!((total_sum - 100.0).abs() < 1e-9);
}

#[test]
fn seeded_measurements_preserve_the_raw_sequence() {
    let h_state = array![FRAC_1_SQRT_2, FRAC_1_SQRT_2].map(|&x| to_c64(x));

    let first = sample_measurements_with_seed(&h_state, 128, 42);
    let second = sample_measurements_with_seed(&h_state, 128, 42);
    let different_seed = sample_measurements_with_seed(&h_state, 128, 43);

    assert_eq!(first, second);
    assert_ne!(first, different_seed);
}
