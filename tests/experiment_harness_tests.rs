use eigenon::experiments::harness::{
    EXPERIMENT_SCHEMA_VERSION, ExperimentDefinition, ExperimentRecord, run_canonical_experiment,
};

const BELL_DEFINITION: &str = include_str!("../experiments/fixtures/bell_state_probabilities.json");
const HADAMARD_DEFINITION: &str = include_str!("../experiments/fixtures/hadamard_sampling.json");

#[test]
fn deterministic_experiment_smoke_uses_saved_definition() {
    let definition = ExperimentDefinition::from_json(BELL_DEFINITION).unwrap();
    let first = run_canonical_experiment(&definition).unwrap();
    let second = run_canonical_experiment(&definition).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.schema_version, EXPERIMENT_SCHEMA_VERSION);
    assert_eq!(first.trials.len(), 2);
    assert!(first.all_comparisons_passed());
    assert!(first.trials.iter().all(|trial| trial.samples.is_empty()));
    assert!(
        definition
            .tolerances
            .matches(first.trials[0].metrics["probability_00"], 0.5)
    );
    assert!(
        definition
            .tolerances
            .matches(first.trials[0].metrics["probability_11"], 0.5)
    );
}

#[test]
fn stochastic_experiment_preserves_reproducible_raw_trials() {
    let definition = ExperimentDefinition::from_json(HADAMARD_DEFINITION).unwrap();
    let first = run_canonical_experiment(&definition).unwrap();
    let second = run_canonical_experiment(&definition).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.trials.len(), 3);
    assert!(first.all_comparisons_passed());

    for (trial_index, trial) in first.trials.iter().enumerate() {
        assert_eq!(trial.trial_index, trial_index);
        assert_eq!(trial.seed, definition.seed + trial_index as u64);
        assert_eq!(trial.samples.len(), definition.shots);
        assert_eq!(
            trial.metrics["count_0"] + trial.metrics["count_1"],
            definition.shots as f64
        );
    }
}

#[test]
fn result_json_round_trip_does_not_rerun_experiment() {
    let definition = ExperimentDefinition::from_json(HADAMARD_DEFINITION).unwrap();
    let record = run_canonical_experiment(&definition).unwrap();
    let json = record.to_json_pretty().unwrap();
    let restored = ExperimentRecord::from_json(&json).unwrap();

    assert_eq!(restored, record);
}

#[test]
fn definition_requires_explicit_valid_tolerances() {
    let invalid = BELL_DEFINITION.replace("1e-12", "-1.0");
    let error = ExperimentDefinition::from_json(&invalid).unwrap_err();

    assert!(error.to_string().contains("absolute tolerance"));
}

#[test]
fn definition_rejects_unknown_schema_version() {
    let invalid = BELL_DEFINITION.replace(
        &format!("\"schema_version\": {EXPERIMENT_SCHEMA_VERSION}"),
        "\"schema_version\": 999",
    );
    let error = ExperimentDefinition::from_json(&invalid).unwrap_err();

    assert!(error.to_string().contains("unsupported experiment schema"));
}
