//! Matrix-free Pauli observables and Hamiltonian expectation values.

use ndarray::ArrayD;
use num_complex::Complex64;
use std::error::Error;
use std::fmt;

/// Default absolute tolerance for the imaginary component of a Hermitian
/// expectation value.
pub const DEFAULT_IMAGINARY_TOLERANCE: f64 = 1e-12;

/// Non-identity single-qubit Pauli operators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pauli {
    /// Pauli X, which flips the computational-basis bit.
    X,
    /// Pauli Y, which flips the bit and applies a phase of `+i` or `-i`.
    Y,
    /// Pauli Z, which negates the amplitude when the bit is one.
    Z,
}

/// A tensor product of Pauli operators acting on selected qubits.
///
/// Operators are stored in ascending qubit order. Qubit zero is the least
/// significant bit of a statevector index. An empty string is the identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PauliString {
    operators: Vec<(usize, Pauli)>,
}

impl PauliString {
    /// Creates a Pauli string and rejects duplicate qubit indices.
    pub fn new<I>(operators: I) -> Result<Self, ObservableError>
    where
        I: IntoIterator<Item = (usize, Pauli)>,
    {
        let mut operators = operators.into_iter().collect::<Vec<_>>();
        operators.sort_unstable_by_key(|(qubit, _)| *qubit);

        if let Some(window) = operators
            .windows(2)
            .find(|window| window[0].0 == window[1].0)
        {
            return Err(ObservableError::DuplicateQubit(window[0].0));
        }

        Ok(Self { operators })
    }

    /// Creates the identity Pauli string.
    pub fn identity() -> Self {
        Self {
            operators: Vec::new(),
        }
    }

    /// Creates a one-qubit Pauli string.
    pub fn single(qubit: usize, pauli: Pauli) -> Self {
        Self {
            operators: vec![(qubit, pauli)],
        }
    }

    /// Returns operators in ascending qubit order.
    pub fn operators(&self) -> &[(usize, Pauli)] {
        &self.operators
    }

    /// Evaluates this Pauli string using the default imaginary tolerance.
    pub fn expectation_value(
        &self,
        state: &ArrayD<Complex64>,
        n_qubits: usize,
    ) -> Result<f64, ObservableError> {
        self.expectation_value_with_tolerance(state, n_qubits, DEFAULT_IMAGINARY_TOLERANCE)
    }

    /// Evaluates this Pauli string with an explicit absolute tolerance for the
    /// imaginary component.
    pub fn expectation_value_with_tolerance(
        &self,
        state: &ArrayD<Complex64>,
        n_qubits: usize,
        imaginary_tolerance: f64,
    ) -> Result<f64, ObservableError> {
        validate_tolerance(imaginary_tolerance)?;
        let amplitudes = validate_state(state, n_qubits)?;
        self.validate_qubits(n_qubits)?;
        real_part(
            expectation_complex(amplitudes, &self.operators),
            imaginary_tolerance,
        )
    }

    fn validate_qubits(&self, n_qubits: usize) -> Result<(), ObservableError> {
        if let Some(&(qubit, _)) = self.operators.iter().find(|(qubit, _)| *qubit >= n_qubits) {
            return Err(ObservableError::QubitOutOfRange { qubit, n_qubits });
        }
        Ok(())
    }
}

/// A finite real coefficient and its Pauli-string observable.
#[derive(Clone, Debug, PartialEq)]
pub struct HamiltonianTerm {
    coefficient: f64,
    observable: PauliString,
}

impl HamiltonianTerm {
    /// Returns the real coefficient multiplying this term.
    pub fn coefficient(&self) -> f64 {
        self.coefficient
    }

    /// Returns the Pauli string multiplied by the coefficient.
    pub fn observable(&self) -> &PauliString {
        &self.observable
    }
}

/// A Hermitian weighted sum of Pauli strings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hamiltonian {
    terms: Vec<HamiltonianTerm>,
}

impl Hamiltonian {
    /// Builds a Hamiltonian and rejects non-finite coefficients.
    pub fn from_terms<I>(terms: I) -> Result<Self, ObservableError>
    where
        I: IntoIterator<Item = (f64, PauliString)>,
    {
        let mut validated = Vec::new();
        for (coefficient, observable) in terms {
            if !coefficient.is_finite() {
                return Err(ObservableError::NonFiniteCoefficient);
            }
            validated.push(HamiltonianTerm {
                coefficient,
                observable,
            });
        }
        Ok(Self { terms: validated })
    }

    /// Returns terms in deterministic insertion order.
    pub fn terms(&self) -> &[HamiltonianTerm] {
        &self.terms
    }

    /// Evaluates the Hamiltonian using the default imaginary tolerance.
    pub fn expectation_value(
        &self,
        state: &ArrayD<Complex64>,
        n_qubits: usize,
    ) -> Result<f64, ObservableError> {
        self.expectation_value_with_tolerance(state, n_qubits, DEFAULT_IMAGINARY_TOLERANCE)
    }

    /// Evaluates the Hamiltonian with an explicit absolute tolerance for the
    /// final imaginary component.
    pub fn expectation_value_with_tolerance(
        &self,
        state: &ArrayD<Complex64>,
        n_qubits: usize,
        imaginary_tolerance: f64,
    ) -> Result<f64, ObservableError> {
        validate_tolerance(imaginary_tolerance)?;
        let amplitudes = validate_state(state, n_qubits)?;
        for term in &self.terms {
            term.observable.validate_qubits(n_qubits)?;
        }

        let value = self
            .terms
            .iter()
            .fold(Complex64::new(0.0, 0.0), |sum, term| {
                sum + term.coefficient * expectation_complex(amplitudes, &term.observable.operators)
            });
        real_part(value, imaginary_tolerance)
    }
}

/// Errors produced while constructing or evaluating observables.
#[derive(Clone, Debug, PartialEq)]
pub enum ObservableError {
    /// A Pauli string contains two operators for the same qubit.
    DuplicateQubit(usize),
    /// An observable targets a qubit outside the declared register.
    QubitOutOfRange { qubit: usize, n_qubits: usize },
    /// The declared qubit count cannot be represented by a `usize` state size.
    QubitCountTooLarge(usize),
    /// The state is not a one-dimensional vector.
    StateMustBeVector,
    /// The statevector length does not equal `2^n_qubits`.
    StateDimensionMismatch { expected: usize, actual: usize },
    /// The state contains NaN or infinite real or imaginary components.
    NonFiniteAmplitude { index: usize },
    /// A Hamiltonian coefficient is NaN or infinite.
    NonFiniteCoefficient,
    /// An imaginary-component tolerance is negative, NaN or infinite.
    InvalidImaginaryTolerance,
    /// A Hermitian expectation retained a larger imaginary component than the
    /// configured tolerance permits.
    NonNegligibleImaginary { imaginary: f64, tolerance: f64 },
    /// Finite inputs overflowed or otherwise produced NaN or infinity during
    /// expectation-value accumulation.
    NonFiniteExpectation,
    /// The ndarray does not expose standard contiguous statevector storage.
    NonContiguousState,
}

impl fmt::Display for ObservableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateQubit(qubit) => {
                write!(formatter, "duplicate Pauli operator for qubit {qubit}")
            }
            Self::QubitOutOfRange { qubit, n_qubits } => write!(
                formatter,
                "Pauli operator qubit {qubit} is out of range for {n_qubits} qubits"
            ),
            Self::QubitCountTooLarge(n_qubits) => {
                write!(
                    formatter,
                    "qubit count {n_qubits} is too large for this platform"
                )
            }
            Self::StateMustBeVector => formatter.write_str("state must be one-dimensional"),
            Self::StateDimensionMismatch { expected, actual } => write!(
                formatter,
                "statevector length mismatch: expected {expected}, found {actual}"
            ),
            Self::NonFiniteAmplitude { index } => {
                write!(formatter, "state amplitude at index {index} must be finite")
            }
            Self::NonFiniteCoefficient => {
                formatter.write_str("Hamiltonian coefficients must be finite")
            }
            Self::InvalidImaginaryTolerance => {
                formatter.write_str("imaginary-component tolerance must be finite and non-negative")
            }
            Self::NonNegligibleImaginary {
                imaginary,
                tolerance,
            } => write!(
                formatter,
                "expectation value imaginary component {imaginary} exceeds tolerance {tolerance}"
            ),
            Self::NonFiniteExpectation => formatter.write_str("expectation value must be finite"),
            Self::NonContiguousState => {
                formatter.write_str("statevector must use contiguous standard layout")
            }
        }
    }
}

impl Error for ObservableError {}

fn validate_tolerance(tolerance: f64) -> Result<(), ObservableError> {
    if tolerance.is_finite() && tolerance >= 0.0 {
        Ok(())
    } else {
        Err(ObservableError::InvalidImaginaryTolerance)
    }
}

fn validate_state(
    state: &ArrayD<Complex64>,
    n_qubits: usize,
) -> Result<&[Complex64], ObservableError> {
    if state.ndim() != 1 {
        return Err(ObservableError::StateMustBeVector);
    }
    let shift = u32::try_from(n_qubits)
        .ok()
        .and_then(|shift| 1usize.checked_shl(shift))
        .ok_or(ObservableError::QubitCountTooLarge(n_qubits))?;
    if state.len() != shift {
        return Err(ObservableError::StateDimensionMismatch {
            expected: shift,
            actual: state.len(),
        });
    }
    let amplitudes = state
        .as_slice()
        .ok_or(ObservableError::NonContiguousState)?;
    if let Some(index) = amplitudes
        .iter()
        .position(|amplitude| !amplitude.re.is_finite() || !amplitude.im.is_finite())
    {
        return Err(ObservableError::NonFiniteAmplitude { index });
    }
    Ok(amplitudes)
}

fn expectation_complex(amplitudes: &[Complex64], operators: &[(usize, Pauli)]) -> Complex64 {
    amplitudes
        .iter()
        .enumerate()
        .fold(Complex64::new(0.0, 0.0), |sum, (basis, amplitude)| {
            let mut transformed_basis = basis;
            let mut phase = Complex64::new(1.0, 0.0);

            for &(qubit, pauli) in operators {
                let bit_is_one = ((basis >> qubit) & 1) == 1;
                match pauli {
                    Pauli::X => transformed_basis ^= 1usize << qubit,
                    Pauli::Y => {
                        transformed_basis ^= 1usize << qubit;
                        phase *= if bit_is_one {
                            Complex64::new(0.0, -1.0)
                        } else {
                            Complex64::new(0.0, 1.0)
                        };
                    }
                    Pauli::Z => {
                        if bit_is_one {
                            phase = -phase;
                        }
                    }
                }
            }

            sum + amplitudes[transformed_basis].conj() * phase * amplitude
        })
}

fn real_part(value: Complex64, tolerance: f64) -> Result<f64, ObservableError> {
    if !value.re.is_finite() || !value.im.is_finite() {
        Err(ObservableError::NonFiniteExpectation)
    } else if value.im.abs() > tolerance {
        Err(ObservableError::NonNegligibleImaginary {
            imaginary: value.im,
            tolerance,
        })
    } else {
        Ok(value.re)
    }
}
