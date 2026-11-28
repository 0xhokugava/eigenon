//! Versioned, reproducible experiment records for local research workflows.

use crate::circuit::core::Circuit;
use crate::engine::measurement::sample_measurements_with_seed;
use ndarray::Array1;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

/// Current JSON schema version for experiment definitions and records.
pub const EXPERIMENT_SCHEMA_VERSION: u32 = 1;

/// Built-in experiment fixtures with stable semantics.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalFixture {
    /// Two-qubit Bell-state probabilities compared with their analytical values.
    BellStateProbabilities,
    /// Repeated measurements of `H|0>` compared with a balanced distribution.
    HadamardSampling,
}

/// Explicit numerical tolerances used for reference comparisons.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Tolerances {
    /// Fixed error allowance.
    pub absolute: f64,
    /// Error allowance relative to the absolute reference value.
    pub relative: f64,
}

impl Tolerances {
    /// Returns whether `observed` agrees with `expected` under these tolerances.
    ///
    /// The allowed error is `absolute + relative * abs(expected)`.
    pub fn matches(self, observed: f64, expected: f64) -> bool {
        let allowed_error = self.absolute + self.relative * expected.abs();
        (observed - expected).abs() <= allowed_error
    }

    fn validate(self) -> Result<(), ExperimentError> {
        if !self.absolute.is_finite() || self.absolute < 0.0 {
            return Err(ExperimentError::InvalidDefinition(
                "absolute tolerance must be finite and non-negative".to_string(),
            ));
        }
        if !self.relative.is_finite() || self.relative < 0.0 {
            return Err(ExperimentError::InvalidDefinition(
                "relative tolerance must be finite and non-negative".to_string(),
            ));
        }
        Ok(())
    }
}

/// Saved input configuration for one reproducible experiment.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExperimentDefinition {
    /// Schema used to decode this definition.
    pub schema_version: u32,
    /// Stable user-selected identifier for the experiment.
    pub experiment_id: String,
    /// Canonical workload to execute.
    pub fixture: CanonicalFixture,
    /// Domain-specific numeric parameters preserved in the result record.
    #[serde(default)]
    pub parameters: BTreeMap<String, f64>,
    /// Base seed. Trial `n` uses `seed + n` with wrapping addition.
    pub seed: u64,
    /// Number of independent trials to preserve.
    pub trials: usize,
    /// Shots per trial. Deterministic fixtures may use zero.
    pub shots: usize,
    /// Required tolerances for analytical or external comparisons.
    pub tolerances: Tolerances,
}

impl ExperimentDefinition {
    /// Validates schema compatibility and all configuration fields.
    pub fn validate(&self) -> Result<(), ExperimentError> {
        if self.schema_version != EXPERIMENT_SCHEMA_VERSION {
            return Err(ExperimentError::UnsupportedSchema {
                found: self.schema_version,
                supported: EXPERIMENT_SCHEMA_VERSION,
            });
        }
        if self.experiment_id.trim().is_empty() {
            return Err(ExperimentError::InvalidDefinition(
                "experiment_id must not be empty".to_string(),
            ));
        }
        if self.trials == 0 {
            return Err(ExperimentError::InvalidDefinition(
                "trials must be greater than zero".to_string(),
            ));
        }
        if self.fixture == CanonicalFixture::HadamardSampling && self.shots == 0 {
            return Err(ExperimentError::InvalidDefinition(
                "hadamard_sampling requires at least one shot".to_string(),
            ));
        }
        if self
            .parameters
            .iter()
            .any(|(name, value)| name.trim().is_empty() || !value.is_finite())
        {
            return Err(ExperimentError::InvalidDefinition(
                "parameter names must be non-empty and values must be finite".to_string(),
            ));
        }
        self.tolerances.validate()
    }

    /// Decodes and validates a JSON experiment definition.
    pub fn from_json(json: &str) -> Result<Self, ExperimentError> {
        let definition: Self = serde_json::from_str(json)?;
        definition.validate()?;
        Ok(definition)
    }

    /// Reads and validates a JSON experiment definition from disk.
    pub fn read_json(path: impl AsRef<Path>) -> Result<Self, ExperimentError> {
        Self::from_json(&fs::read_to_string(path)?)
    }

    /// Encodes this definition as stable, pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, ExperimentError> {
        self.validate()?;
        Ok(serde_json::to_string_pretty(self)?)
    }
}

/// Non-sensitive execution environment recorded with every experiment.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct EnvironmentMetadata {
    /// Operating-system target reported by Rust.
    pub os: String,
    /// CPU architecture target reported by Rust.
    pub arch: String,
    /// `debug` or `release` build profile.
    pub build_profile: String,
}

impl EnvironmentMetadata {
    fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            build_profile: if cfg!(debug_assertions) {
                "debug".to_string()
            } else {
                "release".to_string()
            },
        }
    }
}

/// Reference implementation used to validate recorded metrics.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ReferenceMetadata {
    /// Stable reference name, such as `analytical` or `qiskit-aer`.
    pub implementation: String,
    /// Reference version when one is available.
    pub version: Option<String>,
}

/// One explicit comparison against a reference value.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReferenceComparison {
    /// Metric being checked.
    pub metric: String,
    /// Value produced by Eigenon.
    pub observed: f64,
    /// Value provided by the reference.
    pub expected: f64,
    /// Absolute difference between observed and expected values.
    pub absolute_error: f64,
    /// Maximum allowed error for this comparison.
    pub allowed_error: f64,
    /// Whether the comparison passed.
    pub passed: bool,
}

/// Raw result of a single experiment trial.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TrialResult {
    /// Zero-based trial index.
    pub trial_index: usize,
    /// Exact seed assigned to this trial.
    pub seed: u64,
    /// Named raw and derived metrics.
    pub metrics: BTreeMap<String, f64>,
    /// Raw sampled basis-state indices, in sampling order.
    pub samples: Vec<usize>,
    /// Analytical or external reference comparisons.
    pub comparisons: Vec<ReferenceComparison>,
}

/// Complete machine-readable output of an experiment run.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExperimentRecord {
    /// Schema used to decode this record.
    pub schema_version: u32,
    /// Eigenon package version that generated the record.
    pub eigenon_version: String,
    /// Source revision supplied at compile time through `EIGENON_GIT_REVISION`.
    pub revision: Option<String>,
    /// Original validated input definition.
    pub definition: ExperimentDefinition,
    /// Non-sensitive execution environment.
    pub environment: EnvironmentMetadata,
    /// Reference used by the canonical fixture.
    pub reference: ReferenceMetadata,
    /// Every trial in execution order.
    pub trials: Vec<TrialResult>,
}

impl ExperimentRecord {
    /// Decodes a version-compatible experiment record from JSON.
    pub fn from_json(json: &str) -> Result<Self, ExperimentError> {
        let record: Self = serde_json::from_str(json)?;
        if record.schema_version != EXPERIMENT_SCHEMA_VERSION {
            return Err(ExperimentError::UnsupportedSchema {
                found: record.schema_version,
                supported: EXPERIMENT_SCHEMA_VERSION,
            });
        }
        record.definition.validate()?;
        Ok(record)
    }

    /// Reads an experiment record without executing the experiment again.
    pub fn read_json(path: impl AsRef<Path>) -> Result<Self, ExperimentError> {
        Self::from_json(&fs::read_to_string(path)?)
    }

    /// Encodes this record as stable, pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, ExperimentError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Writes the raw record to disk for later analysis or plotting.
    pub fn write_json(&self, path: impl AsRef<Path>) -> Result<(), ExperimentError> {
        fs::write(path, self.to_json_pretty()?)?;
        Ok(())
    }

    /// Returns true when every recorded reference comparison passed.
    pub fn all_comparisons_passed(&self) -> bool {
        self.trials
            .iter()
            .flat_map(|trial| &trial.comparisons)
            .all(|comparison| comparison.passed)
    }
}

/// Errors returned while loading, validating, running or storing experiments.
#[derive(Debug)]
pub enum ExperimentError {
    /// The JSON schema is not supported by this Eigenon version.
    UnsupportedSchema { found: u32, supported: u32 },
    /// The definition contains an invalid value.
    InvalidDefinition(String),
    /// The definition or record could not be decoded or encoded.
    Json(serde_json::Error),
    /// A definition or record could not be read or written.
    Io(std::io::Error),
}

impl fmt::Display for ExperimentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema { found, supported } => write!(
                formatter,
                "unsupported experiment schema version {found}; expected {supported}"
            ),
            Self::InvalidDefinition(message) => {
                write!(formatter, "invalid experiment definition: {message}")
            }
            Self::Json(error) => write!(formatter, "experiment JSON error: {error}"),
            Self::Io(error) => write!(formatter, "experiment I/O error: {error}"),
        }
    }
}

impl Error for ExperimentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::UnsupportedSchema { .. } | Self::InvalidDefinition(_) => None,
        }
    }
}

impl From<serde_json::Error> for ExperimentError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<std::io::Error> for ExperimentError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Runs one of the canonical fixtures and returns every raw trial result.
pub fn run_canonical_experiment(
    definition: &ExperimentDefinition,
) -> Result<ExperimentRecord, ExperimentError> {
    definition.validate()?;

    let trials = (0..definition.trials)
        .map(|trial_index| {
            let seed = definition.seed.wrapping_add(trial_index as u64);
            match definition.fixture {
                CanonicalFixture::BellStateProbabilities => {
                    run_bell_trial(definition, trial_index, seed)
                }
                CanonicalFixture::HadamardSampling => {
                    run_hadamard_trial(definition, trial_index, seed)
                }
            }
        })
        .collect();

    Ok(ExperimentRecord {
        schema_version: EXPERIMENT_SCHEMA_VERSION,
        eigenon_version: env!("CARGO_PKG_VERSION").to_string(),
        revision: option_env!("EIGENON_GIT_REVISION")
            .filter(|revision| !revision.is_empty())
            .map(str::to_string),
        definition: definition.clone(),
        environment: EnvironmentMetadata::current(),
        reference: ReferenceMetadata {
            implementation: "analytical".to_string(),
            version: None,
        },
        trials,
    })
}

fn run_bell_trial(definition: &ExperimentDefinition, trial_index: usize, seed: u64) -> TrialResult {
    let mut circuit = Circuit::new(2);
    circuit.h(0).cnot(0, 1);
    let state = circuit.run();
    let probabilities: Vec<f64> = state.iter().map(|amplitude| amplitude.norm_sqr()).collect();

    let expected = [0.5, 0.0, 0.0, 0.5];
    let mut metrics = BTreeMap::new();
    let mut comparisons = Vec::new();

    for (basis_index, (&observed, &expected)) in
        probabilities.iter().zip(expected.iter()).enumerate()
    {
        let metric = format!("probability_{basis_index:02b}");
        metrics.insert(metric.clone(), observed);
        comparisons.push(compare(metric, observed, expected, definition.tolerances));
    }

    TrialResult {
        trial_index,
        seed,
        metrics,
        samples: Vec::new(),
        comparisons,
    }
}

fn run_hadamard_trial(
    definition: &ExperimentDefinition,
    trial_index: usize,
    seed: u64,
) -> TrialResult {
    let mut circuit = Circuit::new(1);
    circuit.h(0);
    let state = Array1::<Complex64>::from_iter(circuit.run().iter().copied());
    let samples = sample_measurements_with_seed(&state, definition.shots, seed);
    let count_zero = samples.iter().filter(|&&sample| sample == 0).count();
    let count_one = samples.len() - count_zero;
    let frequency_zero = count_zero as f64 / definition.shots as f64;
    let frequency_one = count_one as f64 / definition.shots as f64;

    let metrics = BTreeMap::from([
        ("count_0".to_string(), count_zero as f64),
        ("count_1".to_string(), count_one as f64),
        ("frequency_0".to_string(), frequency_zero),
        ("frequency_1".to_string(), frequency_one),
    ]);
    let comparisons = vec![
        compare(
            "frequency_0".to_string(),
            frequency_zero,
            0.5,
            definition.tolerances,
        ),
        compare(
            "frequency_1".to_string(),
            frequency_one,
            0.5,
            definition.tolerances,
        ),
    ];

    TrialResult {
        trial_index,
        seed,
        metrics,
        samples,
        comparisons,
    }
}

fn compare(
    metric: String,
    observed: f64,
    expected: f64,
    tolerances: Tolerances,
) -> ReferenceComparison {
    let absolute_error = (observed - expected).abs();
    let allowed_error = tolerances.absolute + tolerances.relative * expected.abs();

    ReferenceComparison {
        metric,
        observed,
        expected,
        absolute_error,
        allowed_error,
        passed: tolerances.matches(observed, expected),
    }
}
