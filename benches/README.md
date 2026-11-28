# Benchmarks

Eigenon uses Criterion for repeatable microbenchmarks. Run the complete suite
with:

```bash
cargo bench
```

Run only the parameterized-gate suite with:

```bash
cargo bench --bench parameterized_gate_bench
```

Run only the observable suite with:

```bash
cargo bench --bench observable_bench
```

Run only the gradient suite with:

```bash
cargo bench --bench gradient_bench
```

Criterion writes HTML reports under `target/criterion/`.

## Parameterized-gate benchmark groups

`parameterized_matrix_construction` isolates the cost of constructing the
2x2 matrices for `Rx`, `Ry`, `Rz`, and `U`. Hadamard construction is included
as a fixed-gate baseline. Deterministic, non-special angles are passed through
`black_box` so the compiler cannot replace the trigonometric work with a
constant result.

`parameterized_gate_application_width` applies precomputed matrices while
increasing the statevector width. `parameterized_gate_application_target`
holds the statevector width constant and moves the target between the least
significant, middle, and most significant qubits. The state clone used to
reset each sample is performed with Criterion batching and is not part of the
measured gate-application routine. Throughput is reported in amplitudes
processed per second.

`parameterized_circuit_execution_width` and
`parameterized_circuit_execution_depth` measure complete `Circuit::run`
workloads. Circuit construction happens outside the measured closure, while
initial-state allocation and per-operation matrix construction remain inside
because they are part of `Circuit::run`. Throughput is reported in semantic
gates executed per second.

`parameterized_parameter_sweep` models the current variational workflow before
symbolic parameter binding exists. It intentionally measures both rebuilding
and executing a circuit for every numeric parameter set. Throughput is
reported in parameter sets processed per second. This group is the baseline
for future parameter-binding work; its semantics should remain stable when a
bound-circuit benchmark is added later.

## Observable benchmark groups

`observable_pauli_string_width` holds a three-operator Pauli string constant
while increasing the statevector from 4 to 16 qubits. Throughput is reported
in amplitudes visited per second.

`observable_hamiltonian_terms` holds a 12-qubit state constant and increases
the Hamiltonian from 1 to 128 Pauli terms. State preparation and observable
construction occur outside the measured closure. Throughput reports total
amplitude-term evaluations, making matrix-free scaling visible without mixing
it with circuit execution.

## Gradient benchmark groups

`gradient_parameter_count` measures complete parameter-shift evaluation for a
fixed eight-qubit variational circuit while increasing the number of requested
parameters from one to eight. Throughput is reported in gradients produced.

`gradient_shared_parameter_occurrences` keeps one named parameter and varies
how many `Ry` gates reuse it. The benchmark includes the required independent
plus/minus executions for every occurrence and reports gate-occurrence
contributions per second.

## Interpreting results

Compare results produced on the same machine, toolchain, build profile, and
power configuration. Criterion detects statistical changes, but results from
different hosts are not directly comparable. Matrix construction and matrix
application are separate groups: combining their timings would hide whether a
change affected trigonometric matrix creation or statevector traversal.
