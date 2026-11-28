# Eigenon

Quantum state-vector simulator and circuit toolkit focused on matrix-free execution, correctness, explicit qubit-ordering conventions, and interoperability with external quantum tooling.

## Features

* Matrix-free in-place state-vector execution
* Generic local `k`-qubit gate application
* Single-qubit gates: X, Y, Z, H, S, T, I, Rx, Ry, Rz and U
* Two-qubit gates: CNOT and CZ
* Multi-controlled gates: MCX and MCZ
* High-level Circuit API
* Reusable circuit templates with named rotation parameters
* Matrix-free Pauli observables and Hamiltonian expectation values
* Parameter-shift gradients for symbolic rotation parameters
* Deterministic variational optimization and a minimal VQE workflow
* Dirac notation output
* Shot-based measurement simulation
* Deutsch, Deutsch–Jozsa and Grover search demos
* Dense matrix baseline for correctness verification
* Criterion benchmarks
* Command-line interface for custom circuits, demos and verification
* OpenQASM 2.0 import and OpenQASM 2.0/OpenQASM 3.0 export

## Contributing

Contributions are welcome. Browse the prepared
[good first issues](https://github.com/0xhokugava/eigenon/contribute) or read
[CONTRIBUTING.md](CONTRIBUTING.md) for the development workflow, required
checks and project conventions.

## Quick start

Eigenon requires Git and a stable Rust toolchain with Rust 2024 edition
support. Rust 1.85 or newer supports this edition. CI follows the latest stable
toolchain, so the project does not currently guarantee a separate minimum
supported Rust version.

Clone the repository and run its default checks:

```bash
git clone https://github.com/0xhokugava/eigenon.git
cd eigenon
cargo fmt -- --check
cargo test
```

## Examples

Run the Bell state example:

```bash
cargo run --example bell
```

The example creates a two-qubit circuit, applies `H` to qubit `0`, then applies
`CNOT` with qubit `0` as the control and qubit `1` as the target. It prints:

```text
Bell state: (0.707 + 0.000i)|00> + (0.707 + 0.000i)|11>
```

Qubit `0` is the least significant bit and appears as the rightmost bit in
printed basis states. More runnable examples can be added to the `examples/`
directory and executed with `cargo run --example <name>`.

## Command-line interface

Install `eigenon` binary from the repository root:

```bash
cargo install --path .
```

Run a custom circuit:

```bash
eigenon run --qubits 2 --gate h:0 --gate cnot:0,1
```

Output:

```text
🐈 Eigenon 🐈‍⬛

Qubits: 2, Gates: [H(0), Cnot { control: 0, target: 1 }]
State: (0.707 + 0.000i)|00> + (0.707 + 0.000i)|11>
```

Import and execute an OpenQASM 2.0 file:

```bash
eigenon import-openqasm --file circuit.qasm
```

The command reports the imported register and operation counts, then prints the
final state using the same qubit-ordering convention as `eigenon run`. OpenQASM
3 input is identified and rejected with a dedicated unsupported-version error;
it is not reported as malformed OpenQASM 2 input.

Supported gate syntax:

```text
h:0
x:0
y:0
z:0
s:0
t:0
i:0
rx:1.5707963267948966,0
ry:-0.5,0
rz:0.25,0
u:0.1,0.2,0.3,0
cnot:0,1
cz:0,1
mcx:0,1:2
mcz:0,1:2
```

For multi-controlled gates, the syntax is:

```text
mcx:controls:target
mcz:controls:target
```

Parameterized gate angles are decimal radians. Rotation gates use
`rx:angle,target`, `ry:angle,target` and `rz:angle,target`. The universal gate
uses `u:theta,phi,lambda,target`; `u3` is accepted as a CLI alias.

Show available algorithm demos:

```bash
eigenon demo --help
```

Show available verification commands:

```bash
eigenon verify --help
```

During development, run the current source without reinstalling the binary:

```bash
cargo run --bin eigenon -- run --qubits 2 --gate h:0 --gate cnot:0,1
```

To update an already installed local binary:

```bash
cargo install --path . --force
```

## Circuit API

```rust
use eigenon::circuit::core::Circuit;
use eigenon::engine::utils::to_dirac;

let mut circuit = Circuit::new(2);
circuit.h(0).cnot(0, 1);
let state = circuit.run();

println!("{}", to_dirac(&state));
```

The public convention for controlled gates is:

```text
cnot(control, target)
cz(control, target)
mcx(controls, target)
mcz(controls, target)
rx(theta, target)
ry(theta, target)
rz(theta, target)
u(theta, phi, lambda, target)
```

All parameterized-gate angles are expressed in radians. `u3(...)` is also
available as an alias for `u(...)`.

Qubit `q0` is the least significant bit and appears as the rightmost bit in printed basis states.

### Symbolic circuit parameters

Define a circuit structure once, inspect its required parameters and bind
different finite values without rebuilding its operations:

```rust
use eigenon::circuit::{Parameter, ParameterizedCircuit};

let theta = Parameter::new("theta");
let mut template = ParameterizedCircuit::new(1);
template.ry(theta.clone(), 0);

assert_eq!(template.parameters(), &[theta.clone()]);

let first = template.bind([(&theta, 0.5)])?;
let second = template.bind([(&theta, 1.0)])?;

# Ok::<(), eigenon::circuit::BindingError>(())
```

Named parameters are supported by `rx`, `ry` and `rz`. The same parameter may
be reused by multiple rotations. Parameter and operation ordering follows
first appearance and insertion order respectively. Binding reports typed
errors for missing, duplicate, unknown and non-finite values. It creates an
ordinary numeric `Circuit`; the template itself is not mutated.

Run the complete example with:

```bash
cargo run --example parameterized_circuit
```

### Observables and expectation values

Evaluate Pauli strings and weighted Hamiltonians directly from a statevector
without allocating a dense `2^n × 2^n` observable matrix:

```rust
use eigenon::circuit::core::Circuit;
use eigenon::observable::{Hamiltonian, Pauli, PauliString};

let mut circuit = Circuit::new(2);
circuit.h(0).cnot(0, 1);

let hamiltonian = Hamiltonian::from_terms([
    (-1.0, PauliString::new([(0, Pauli::Z), (1, Pauli::Z)])?),
    (-0.5, PauliString::single(0, Pauli::X)),
])?;

let state = circuit.run();
let energy = hamiltonian.expectation_value(&state, circuit.n_qubits())?;

# Ok::<(), Box<dyn std::error::Error>>(())
```

An empty `PauliString` represents identity. Pauli operators are stored in
ascending qubit order, while qubit `0` remains the least-significant bit of a
statevector index. Duplicate and out-of-range operators, invalid statevector
dimensions, non-finite values, and excessive imaginary residuals return typed
errors.

Run the complete example with:

```bash
cargo run --example observable
```

### Parameter-shift gradients

Differentiate Pauli or Hamiltonian expectation values for symbolic `Rx`, `Ry`
and `Rz` parameters:

```rust
use eigenon::circuit::{Parameter, ParameterizedCircuit};
use eigenon::gradient::parameter_shift;
use eigenon::observable::{Hamiltonian, Pauli, PauliString};

let theta = Parameter::new("theta");
let mut template = ParameterizedCircuit::new(1);
template.ry(theta.clone(), 0);
let hamiltonian = Hamiltonian::from_terms([
    (-1.0, PauliString::single(0, Pauli::Z)),
])?;

let gradients = parameter_shift(
    &template,
    &[(&theta, 0.4)],
    &hamiltonian,
    std::slice::from_ref(&theta),
)?;

println!("dE/dtheta = {}", gradients[&theta]);

# Ok::<(), Box<dyn std::error::Error>>(())
```

When a parameter is reused, Eigenon shifts each gate occurrence independently
and sums its contribution. This implements the chain rule correctly; shifting
all shared occurrences at once is not generally valid. Measurements are
rejected because this initial API differentiates deterministic statevector
expectations. Extremely large finite bindings are rejected when floating-point
precision cannot represent `theta ± pi/2` within a relative shift distortion
of `1e-12`.

```bash
cargo run --example gradient
```

### Variational optimization and VQE

Combine a parameterized ansatz, Hamiltonian, and pluggable classical optimizer
into a deterministic variational loop:

```rust
use eigenon::circuit::{Parameter, ParameterizedCircuit};
use eigenon::observable::{Hamiltonian, Pauli, PauliString};
use eigenon::variational::{GradientDescent, VqeProblem};

let theta = Parameter::new("theta");
let mut ansatz = ParameterizedCircuit::new(1);
ansatz.ry(theta.clone(), 0);
let hamiltonian = Hamiltonian::from_terms([
    (1.0, PauliString::single(0, Pauli::Z)),
])?;

let result = VqeProblem::new(ansatz, hamiltonian).minimize(
    GradientDescent::new(0.2)
        .max_iterations(200)
        .convergence_tolerance(1e-10),
    [(&theta, 0.25)],
)?;

assert!(result.converged);
assert!((result.minimum + 1.0).abs() < 1e-12);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Every iteration records its zero-based index, name-sorted parameter bindings,
energy, and parameter-shift gradients. The returned minimum is the best energy
actually evaluated, including when the iteration limit is reached. Gradient
descent is deterministic and uses the maximum absolute gradient as its
convergence criterion. Custom optimizers implement the public `Optimizer`
trait and reuse `VqeProblem::energy` and `VqeProblem::gradients`.
Measurement-bearing templates are rejected by both objective methods because
post-measurement statevectors would make optimization stochastic.

The ansatz, Pauli strings, and history all preserve Eigenon's convention that
qubit `0` is the least-significant statevector bit. The first example uses no
random sampling, so identical inputs produce exactly identical histories:

```bash
cargo run --example vqe
```

## OpenQASM export

Eigenon supports OpenQASM 2.0 and OpenQASM 3.0 export.

OpenQASM 2.0 is the default:

```bash
cargo run --quiet -- export-openqasm \
  --qubits 2 \
  --gate h:0 \
  --gate cnot:0,1
```

Use `--qasm-version 3` to export OpenQASM 3.0:

```bash
cargo run --quiet -- export-openqasm \
  --qubits 2 \
  --gate h:0 \
  --gate cnot:0,1 \
  --qasm-version 3
```

OpenQASM 2.0 exports MCX and MCZ operations with one or two controls. Two-control
MCX uses `ccx`, while two-control MCZ is decomposed as H/CCX/H. OpenQASM 3.0
exports any valid control count with the standard `ctrl(n) @` modifier.

Both exporters support `rx`, `ry` and `rz`. The universal single-qubit gate is
written as `u3(theta, phi, lambda)` in OpenQASM 2.0 and
`U(theta, phi, lambda)` in OpenQASM 3.0.

## OpenQASM import

Import supported OpenQASM 2.0 source through the library API when the source is
already available as a string:

```rust
use eigenon::import::openqasm::import_openqasm2;

let source = r#"OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
h q[0];
cx q[0], q[1];
"#;

let circuit = import_openqasm2(source).unwrap();
assert_eq!(circuit.n_qubits(), 2);
```

The importer supports one quantum register, an optional classical register,
indexed measurements, `id`, `x`, `y`, `z`, `h`, `s`, and `t`, plus `CX`, `cx`,
`cz`, and `ccx`. Parameterized and custom gates,
whole-register operations, control flow, reset, barriers, and multiple
registers return explicit errors. OpenQASM 3 import is not currently supported.

## Development

Run the test suite:

```bash
cargo test
```

Run benchmarks:

```bash
cargo bench
```

Run only the parameterized-gate benchmarks:

```bash
cargo bench --bench parameterized_gate_bench
```

See [`benches/README.md`](benches/README.md) for the benchmark groups,
measurement boundaries and guidance for interpreting Criterion results.

Run a deterministic, versioned research experiment:

```bash
cargo run --example reproducible_experiment -- \
  experiments/fixtures/bell_state_probabilities.json
```

See [`docs/EXPERIMENTS.md`](docs/EXPERIMENTS.md) for seeded stochastic
experiments, JSON records, explicit tolerances and raw-data analysis.

Check formatting without changing files:

```bash
cargo fmt -- --check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow, all
required CI checks and the full pull request checklist.

## Documentation

* [Architecture and implementation](docs/ARCHITECTURE.md)
* [Development roadmap](docs/ROADMAP.md)
* [Reproducible experiments](docs/EXPERIMENTS.md)
* [Issue triage and label policy](docs/TRIAGE.md)
* [External validation](validation)
* [Contribution guide](CONTRIBUTING.md)

## License

Eigenon is licensed under the GNU General Public License v3.0 or later.
