# Eigenon

Quantum state-vector simulator and circuit toolkit focused on matrix-free execution, correctness, explicit qubit-ordering conventions, and interoperability with external quantum tooling.

## Features

* Matrix-free in-place state-vector execution
* Generic local `k`-qubit gate application
* Single-qubit gates: X, Y, Z, H, S, T, I
* Two-qubit gates: CNOT and CZ
* Multi-controlled gates: MCX and MCZ
* High-level Circuit API
* Dirac notation output
* Shot-based measurement simulation
* Deutsch, Deutsch–Jozsa and Grover search demos
* Dense matrix baseline for correctness verification
* Criterion benchmarks
* Command-line interface for custom circuits, demos and verification
* OpenQASM 2.0 and OpenQASM 3.0 export

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

Supported gate syntax:

```text
h:0
x:0
y:0
z:0
s:0
t:0
i:0
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
```

Qubit `q0` is the least significant bit and appears as the rightmost bit in printed basis states.

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

## Development

Run the test suite:

```bash
cargo test
```

Run benchmarks:

```bash
cargo bench
```

Check formatting without changing files:

```bash
cargo fmt -- --check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow and the
full pull request checklist.

## Documentation

* [Architecture and implementation](docs/ARCHITECTURE.md)
* [Development roadmap](docs/ROADMAP.md)
* [External validation](validation)
* [Contribution guide](CONTRIBUTING.md)

## License

Eigenon is licensed under the GNU General Public License v3.0 or later.
