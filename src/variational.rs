//! Deterministic variational optimization and a minimal VQE workflow.

use crate::circuit::{BindingError, Parameter, ParameterizedCircuit, ParameterizedOperation};
use crate::gradient::{GradientError, ParameterGradients, parameter_shift};
use crate::observable::{Hamiltonian, ObservableError};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// One fully recorded objective evaluation.
#[derive(Clone, Debug, PartialEq)]
pub struct IterationRecord {
    /// Zero-based optimizer iteration.
    pub iteration: usize,
    /// Parameters used for this evaluation, ordered by name.
    pub parameters: BTreeMap<Parameter, f64>,
    /// Hamiltonian expectation value.
    pub energy: f64,
    /// Parameter-shift gradients at these parameters, ordered by name.
    pub gradients: ParameterGradients,
}

/// Result returned by a classical optimizer.
#[derive(Clone, Debug, PartialEq)]
pub struct VariationalResult {
    /// Lowest energy observed during the run.
    pub minimum: f64,
    /// Parameters that produced `minimum`.
    pub parameters: BTreeMap<Parameter, f64>,
    /// Complete deterministic evaluation history.
    pub history: Vec<IterationRecord>,
    /// Whether the configured convergence criterion was met.
    pub converged: bool,
}

/// Pluggable classical optimizer interface for a [`VqeProblem`].
pub trait Optimizer {
    /// Optimizes `problem` from a complete validated parameter map.
    fn optimize(
        &self,
        problem: &VqeProblem,
        initial: BTreeMap<Parameter, f64>,
    ) -> Result<VariationalResult, VariationalError>;
}

/// A parameterized ansatz and Hamiltonian objective.
#[derive(Clone, Debug, PartialEq)]
pub struct VqeProblem {
    template: ParameterizedCircuit,
    hamiltonian: Hamiltonian,
}

impl VqeProblem {
    /// Creates a variational objective from an ansatz and Hamiltonian.
    pub fn new(template: ParameterizedCircuit, hamiltonian: Hamiltonian) -> Self {
        Self {
            template,
            hamiltonian,
        }
    }

    /// Returns the parameterized ansatz.
    pub fn template(&self) -> &ParameterizedCircuit {
        &self.template
    }

    /// Returns the Hamiltonian objective.
    pub fn hamiltonian(&self) -> &Hamiltonian {
        &self.hamiltonian
    }

    /// Returns required parameters in template first-use order.
    pub fn parameters(&self) -> &[Parameter] {
        self.template.parameters()
    }

    /// Evaluates the Hamiltonian expectation for a complete binding map.
    pub fn energy(&self, bindings: &BTreeMap<Parameter, f64>) -> Result<f64, VariationalError> {
        self.validate_deterministic()?;
        let circuit = self.template.bind(
            bindings
                .iter()
                .map(|(parameter, value)| (parameter, *value)),
        )?;
        Ok(self
            .hamiltonian
            .expectation_value(&circuit.run(), circuit.n_qubits())?)
    }

    /// Evaluates parameter-shift gradients for every template parameter.
    pub fn gradients(
        &self,
        bindings: &BTreeMap<Parameter, f64>,
    ) -> Result<ParameterGradients, VariationalError> {
        self.validate_deterministic()?;
        let binding_list = bindings
            .iter()
            .map(|(parameter, value)| (parameter, *value))
            .collect::<Vec<_>>();
        Ok(parameter_shift(
            &self.template,
            &binding_list,
            &self.hamiltonian,
            self.template.parameters(),
        )?)
    }

    fn validate_deterministic(&self) -> Result<(), VariationalError> {
        if self
            .template
            .operations()
            .iter()
            .any(|operation| matches!(operation, ParameterizedOperation::Measure { .. }))
        {
            return Err(VariationalError::MeasurementUnsupported);
        }
        Ok(())
    }

    /// Runs a pluggable optimizer from explicit initial bindings.
    pub fn minimize<'a, O, I>(
        &self,
        optimizer: O,
        initial: I,
    ) -> Result<VariationalResult, VariationalError>
    where
        O: Optimizer,
        I: IntoIterator<Item = (&'a Parameter, f64)>,
    {
        let initial = collect_initial_bindings(&self.template, initial)?;
        optimizer.optimize(self, initial)
    }
}

/// Deterministic full-batch gradient descent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientDescent {
    learning_rate: f64,
    max_iterations: usize,
    convergence_tolerance: f64,
}

impl GradientDescent {
    /// Creates gradient descent with 100 iterations and `1e-8` tolerance.
    pub fn new(learning_rate: f64) -> Self {
        Self {
            learning_rate,
            max_iterations: 100,
            convergence_tolerance: 1e-8,
        }
    }

    /// Sets the maximum number of recorded objective evaluations.
    pub fn max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Sets the maximum absolute-gradient convergence threshold.
    pub fn convergence_tolerance(mut self, convergence_tolerance: f64) -> Self {
        self.convergence_tolerance = convergence_tolerance;
        self
    }

    /// Returns the configured learning rate.
    pub fn learning_rate(&self) -> f64 {
        self.learning_rate
    }

    /// Returns the maximum number of evaluations.
    pub fn maximum_iterations(&self) -> usize {
        self.max_iterations
    }

    /// Returns the absolute-gradient convergence threshold.
    pub fn tolerance(&self) -> f64 {
        self.convergence_tolerance
    }

    fn validate(self) -> Result<(), VariationalError> {
        if !self.learning_rate.is_finite() || self.learning_rate <= 0.0 {
            return Err(VariationalError::InvalidLearningRate);
        }
        if self.max_iterations == 0 {
            return Err(VariationalError::InvalidMaximumIterations);
        }
        if !self.convergence_tolerance.is_finite() || self.convergence_tolerance < 0.0 {
            return Err(VariationalError::InvalidConvergenceTolerance);
        }
        Ok(())
    }
}

impl Optimizer for GradientDescent {
    fn optimize(
        &self,
        problem: &VqeProblem,
        initial: BTreeMap<Parameter, f64>,
    ) -> Result<VariationalResult, VariationalError> {
        self.validate()?;

        let mut current = initial;
        // Do not allocate from an untrusted iteration bound up front. A run
        // may converge on its first evaluation even when the limit is large.
        let mut history = Vec::new();
        let mut best_energy = f64::INFINITY;
        let mut best_parameters = current.clone();

        for iteration in 0..self.max_iterations {
            // Gradients are evaluated first so templates containing measurement
            // fail before any stochastic objective execution.
            let gradients = problem.gradients(&current)?;
            let energy = problem.energy(&current)?;
            if !energy.is_finite() {
                return Err(VariationalError::NonFiniteEnergy);
            }

            if energy < best_energy {
                best_energy = energy;
                best_parameters = current.clone();
            }

            history.push(IterationRecord {
                iteration,
                parameters: current.clone(),
                energy,
                gradients: gradients.clone(),
            });

            let max_gradient = gradients
                .values()
                .map(|gradient| gradient.abs())
                .fold(0.0, f64::max);
            if max_gradient <= self.convergence_tolerance {
                return Ok(VariationalResult {
                    minimum: best_energy,
                    parameters: best_parameters,
                    history,
                    converged: true,
                });
            }

            if iteration + 1 == self.max_iterations {
                break;
            }

            for (parameter, gradient) in gradients {
                let value = current[&parameter] - self.learning_rate * gradient;
                if !value.is_finite() {
                    return Err(VariationalError::NonFiniteParameterUpdate(parameter));
                }
                current.insert(parameter, value);
            }
        }

        Ok(VariationalResult {
            minimum: best_energy,
            parameters: best_parameters,
            history,
            converged: false,
        })
    }
}

/// Errors produced by variational objectives and optimizers.
#[derive(Debug, PartialEq)]
pub enum VariationalError {
    /// Initial or per-iteration circuit bindings are invalid.
    Binding(BindingError),
    /// Parameter-shift gradient evaluation failed.
    Gradient(GradientError),
    /// Hamiltonian expectation evaluation failed.
    Observable(ObservableError),
    /// Learning rate is non-finite or not positive.
    InvalidLearningRate,
    /// Maximum iteration count is zero.
    InvalidMaximumIterations,
    /// Convergence tolerance is negative or non-finite.
    InvalidConvergenceTolerance,
    /// An objective evaluation returned NaN or infinity.
    NonFiniteEnergy,
    /// Measurement would make the variational objective stochastic.
    MeasurementUnsupported,
    /// A finite gradient update produced NaN or infinity.
    NonFiniteParameterUpdate(Parameter),
}

impl fmt::Display for VariationalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binding(error) => write!(formatter, "invalid variational bindings: {error}"),
            Self::Gradient(error) => write!(formatter, "gradient evaluation failed: {error}"),
            Self::Observable(error) => write!(formatter, "observable evaluation failed: {error}"),
            Self::InvalidLearningRate => {
                formatter.write_str("learning rate must be finite and positive")
            }
            Self::InvalidMaximumIterations => {
                formatter.write_str("maximum iterations must be greater than zero")
            }
            Self::InvalidConvergenceTolerance => {
                formatter.write_str("convergence tolerance must be finite and non-negative")
            }
            Self::NonFiniteEnergy => formatter.write_str("variational energy must be finite"),
            Self::MeasurementUnsupported => {
                formatter.write_str("variational objectives do not support measurement")
            }
            Self::NonFiniteParameterUpdate(parameter) => write!(
                formatter,
                "gradient update for parameter `{parameter}` must be finite"
            ),
        }
    }
}

impl Error for VariationalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Binding(error) => Some(error),
            Self::Gradient(error) => Some(error),
            Self::Observable(error) => Some(error),
            Self::InvalidLearningRate
            | Self::InvalidMaximumIterations
            | Self::InvalidConvergenceTolerance
            | Self::NonFiniteEnergy
            | Self::MeasurementUnsupported
            | Self::NonFiniteParameterUpdate(_) => None,
        }
    }
}

impl From<BindingError> for VariationalError {
    fn from(error: BindingError) -> Self {
        Self::Binding(error)
    }
}

impl From<GradientError> for VariationalError {
    fn from(error: GradientError) -> Self {
        Self::Gradient(error)
    }
}

impl From<ObservableError> for VariationalError {
    fn from(error: ObservableError) -> Self {
        Self::Observable(error)
    }
}

fn collect_initial_bindings<'a, I>(
    template: &ParameterizedCircuit,
    initial: I,
) -> Result<BTreeMap<Parameter, f64>, VariationalError>
where
    I: IntoIterator<Item = (&'a Parameter, f64)>,
{
    let initial = initial.into_iter().collect::<Vec<_>>();
    template.bind(initial.iter().copied())?;
    Ok(initial
        .into_iter()
        .map(|(parameter, value)| (parameter.clone(), value))
        .collect())
}
