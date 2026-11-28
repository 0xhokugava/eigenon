# Reproducible experiments

Eigenon's experiment harness stores both its input definition and raw trial
results as versioned JSON. It is intended for small local research workflows,
cross-validation and regression evidence. It does not replace Criterion
microbenchmarks or the public CLI protocol planned in issue #119.

## Run a saved experiment

Run the deterministic Bell-state fixture and print the raw record:

```bash
cargo run --example reproducible_experiment -- \
  experiments/fixtures/bell_state_probabilities.json
```

Run the seeded Hadamard sampling fixture and save the result:

```bash
cargo run --example reproducible_experiment -- \
  experiments/fixtures/hadamard_sampling.json \
  /tmp/hadamard-result.json
```

Running the same configuration with the same Eigenon dependency set produces
the same raw sample sequence. Each trial derives its seed by wrapping addition
of the zero-based trial index to the configured base seed.

## Definition schema version 1

```json
{
  "schema_version": 1,
  "experiment_id": "hadamard-seeded-sampling",
  "fixture": "hadamard_sampling",
  "parameters": {},
  "seed": 12345,
  "trials": 3,
  "shots": 1024,
  "tolerances": {
    "absolute": 0.1,
    "relative": 0.0
  }
}
```

Supported canonical fixtures are:

- `bell_state_probabilities`: deterministic Bell-state probabilities;
- `hadamard_sampling`: seeded shot sampling of `H|0>`.

The comparison rule is:

```text
abs(observed - expected) <= absolute + relative * abs(expected)
```

Both tolerances are required, finite and non-negative. Stochastic fixtures
require at least one shot and all definitions require at least one trial.

## Result records

Every result contains:

- schema and Eigenon versions;
- the complete validated definition;
- source revision when supplied through `EIGENON_GIT_REVISION` at build time;
- operating-system target, architecture and build profile;
- reference implementation metadata;
- individual trial seeds, metrics, raw samples and comparisons.

Basis-state sample indices and binary metric labels use Eigenon's little-endian
qubit convention: `q0` is the least-significant bit and therefore the rightmost
bit in a printed binary label.

The harness deliberately omits usernames, home directories, environment
variables and absolute input paths. External validators can set the reference
name and version when they add their own fixture runner in the future.

## Analysis and plotting

Raw-data generation and presentation are separate. Plotting or report tools
must read a saved `ExperimentRecord` through `ExperimentRecord::read_json` or
another JSON consumer. They must not silently rerun the experiment. This keeps
figures traceable to the exact samples and metadata that produced them.

The JSON schema is versioned independently from plot implementations. A schema
change that cannot be read compatibly requires a new `schema_version`.
