//! High-level circuit construction and semantic operation representation.
//!
//! [`core::Circuit`] is the main user-facing builder. Operations are stored in
//! insertion order and can be executed by the state-vector backend or inspected
//! by exporters. [`catalog`] provides ready-made circuits for common examples.

pub mod catalog;
pub mod core;
pub mod operation;
