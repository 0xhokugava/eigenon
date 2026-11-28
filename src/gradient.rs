//! Parameter-shift gradients for parameterized circuit expectation values.

use crate::circuit::{
    BindingError, Parameter, ParameterValue, ParameterizedCircuit, ParameterizedGateKind,
    ParameterizedOperation,
};
use crate::observable::{ExpectationValue, ObservableError};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::f64::consts::FRAC_PI_2;
use std::fmt;

const MAX_RELATIVE_SHIFT_DISTORTION: f64 = 1e-12;

/// Deterministic gradient values keyed by parameter name.
pub type ParameterGradients = BTreeMap<Parameter, f64>;

/// Errors produced while evaluating a parameter-shift gradient.
#[derive(Debug, PartialEq)]
pub enum GradientError {
    /// The numeric bindings for the circuit template are invalid.
    Binding(BindingError),
    /// The observable could not be evaluated.
    Observable(ObservableError),
    /// A requested gradient parameter is not present in the template.
    UnknownParameter(Parameter),
    /// The same gradient parameter was requested more than once.
    DuplicateParameter(Parameter),
    /// A requested parameter is attached only to unsupported gate kinds.
    UnsupportedParameter(Parameter),
    /// Measurement makes the objective stochastic and is not supported by the
    /// deterministic parameter-shift API.
    MeasurementUnsupported,
    /// Finite expectation values overflowed while their gradient was summed.
    NonFiniteGradient(Parameter),
    /// The binding is too large for `theta ± pi/2` to produce distinct
    /// floating-point values.
    ShiftNotRepresentable(Parameter),
}

impl fmt::Display for GradientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binding(error) => write!(formatter, "invalid parameter bindings: {error}"),
            Self::Observable(error) => write!(formatter, "observable evaluation failed: {error}"),
            Self::UnknownParameter(parameter) => {
                write!(formatter, "unknown gradient parameter `{parameter}`")
            }
            Self::DuplicateParameter(parameter) => {
                write!(formatter, "duplicate gradient parameter `{parameter}`")
            }
            Self::UnsupportedParameter(parameter) => {
                write!(
                    formatter,
                    "parameter `{parameter}` uses an unsupported gate"
                )
            }
            Self::MeasurementUnsupported => {
                formatter.write_str("parameter-shift gradients do not support measurement")
            }
            Self::NonFiniteGradient(parameter) => {
                write!(
                    formatter,
                    "gradient for parameter `{parameter}` must be finite"
                )
            }
            Self::ShiftNotRepresentable(parameter) => write!(
                formatter,
                "parameter-shift values for `{parameter}` are not representable"
            ),
        }
    }
}

impl Error for GradientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Binding(error) => Some(error),
            Self::Observable(error) => Some(error),
            Self::UnknownParameter(_)
            | Self::DuplicateParameter(_)
            | Self::UnsupportedParameter(_)
            | Self::MeasurementUnsupported
            | Self::NonFiniteGradient(_)
            | Self::ShiftNotRepresentable(_) => None,
        }
    }
}

impl From<BindingError> for GradientError {
    fn from(error: BindingError) -> Self {
        Self::Binding(error)
    }
}

impl From<ObservableError> for GradientError {
    fn from(error: ObservableError) -> Self {
        Self::Observable(error)
    }
}

/// Computes expectation-value gradients using the standard two-eigenvalue
/// parameter-shift rule for symbolic `Rx`, `Ry`, and `Rz` operations.
///
/// When one parameter occurs in multiple rotations, each gate occurrence is
/// shifted independently and the contributions are summed. Returned values use
/// deterministic parameter-name ordering.
pub fn parameter_shift<O>(
    template: &ParameterizedCircuit,
    bindings: &[(&Parameter, f64)],
    observable: &O,
    parameters: &[Parameter],
) -> Result<ParameterGradients, GradientError>
where
    O: ExpectationValue + ?Sized,
{
    if template
        .operations()
        .iter()
        .any(|operation| matches!(operation, ParameterizedOperation::Measure { .. }))
    {
        return Err(GradientError::MeasurementUnsupported);
    }

    template.bind(bindings.iter().copied())?;
    let binding_values = bindings
        .iter()
        .map(|(parameter, value)| ((*parameter).clone(), *value))
        .collect::<BTreeMap<_, _>>();

    let mut requested = BTreeSet::new();
    for parameter in parameters {
        if !template.parameters().contains(parameter) {
            return Err(GradientError::UnknownParameter(parameter.clone()));
        }
        if !requested.insert(parameter.clone()) {
            return Err(GradientError::DuplicateParameter(parameter.clone()));
        }
    }

    let mut gradients = BTreeMap::new();
    for parameter in parameters {
        let binding = binding_values[parameter];
        if !shift_is_representable(binding) {
            return Err(GradientError::ShiftNotRepresentable(parameter.clone()));
        }
        let occurrences = supported_occurrences(template, parameter);
        if occurrences.is_empty() {
            return Err(GradientError::UnsupportedParameter(parameter.clone()));
        }

        let mut gradient = 0.0;
        for operation_index in occurrences {
            let plus =
                template.bind_with_operation_shift(&binding_values, operation_index, FRAC_PI_2);
            let minus =
                template.bind_with_operation_shift(&binding_values, operation_index, -FRAC_PI_2);
            let plus_value = observable.expectation_value(&plus.run(), plus.n_qubits())?;
            let minus_value = observable.expectation_value(&minus.run(), minus.n_qubits())?;
            gradient += 0.5 * plus_value - 0.5 * minus_value;
        }

        if !gradient.is_finite() {
            return Err(GradientError::NonFiniteGradient(parameter.clone()));
        }
        gradients.insert(parameter.clone(), gradient);
    }

    Ok(gradients)
}

fn shift_is_representable(binding: f64) -> bool {
    let plus_delta = (binding + FRAC_PI_2) - binding;
    let minus_delta = binding - (binding - FRAC_PI_2);

    [plus_delta, minus_delta].into_iter().all(|delta| {
        delta.is_finite()
            && ((delta - FRAC_PI_2) / FRAC_PI_2).abs() <= MAX_RELATIVE_SHIFT_DISTORTION
    })
}

fn supported_occurrences(template: &ParameterizedCircuit, parameter: &Parameter) -> Vec<usize> {
    template
        .operations()
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| match operation {
            ParameterizedOperation::SingleQubit { gate, .. } => match gate {
                ParameterizedGateKind::Rx(ParameterValue::Symbol(candidate))
                | ParameterizedGateKind::Ry(ParameterValue::Symbol(candidate))
                | ParameterizedGateKind::Rz(ParameterValue::Symbol(candidate))
                    if candidate == parameter =>
                {
                    Some(index)
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}
