//! Quantum state-vector simulator and circuit toolkit.
//!
//! Eigenon provides a high-level circuit API backed by matrix-free state-vector
//! execution. Qubit `0` is the least significant bit and appears as the
//! rightmost bit in printed basis states.
//!
//! # Quick start
//!
//! Build a two-qubit Bell state:
//!
//! ```
//! use eigenon::circuit::core::Circuit;
//!
//! let mut circuit = Circuit::new(2);
//! circuit.h(0).cnot(0, 1);
//!
//! let state = circuit.run();
//! let probabilities: Vec<f64> = state
//!     .iter()
//!     .map(|amplitude| amplitude.norm_sqr())
//!     .collect();
//!
//! assert!((probabilities[0] - 0.5).abs() < 1e-12);
//! assert!((probabilities[3] - 0.5).abs() < 1e-12);
//! assert!(probabilities[1] < 1e-12);
//! assert!(probabilities[2] < 1e-12);
//! ```
//!
//! Controlled gates use `control, target` argument order. In the example,
//! qubit `0` controls qubit `1`.

pub mod algorithms;
pub mod circuit;
pub mod cli;
pub mod engine;
pub mod experiments;
pub mod export;
