# Architecture

Project separates circuit construction from low-level state-vector execution.

Custom CLI circuit follows this path:

```text
command-line arguments
→ GateSpec parsing
→ qubit validation
→ Circuit construction
→ state-vector execution
→ Dirac notation output
```

Example:

```bash
eigenon run --qubits 2 --gate h:0 --gate cnot:0,1
```

## Circuit API

Circuit API stores quantum operations in execution order and dispatches them to the simulator backend.

```rust
use eigenon::circuit::core::Circuit;

let mut circuit = Circuit::new(2);
circuit.h(0).cnot(0, 1);
let state = circuit.run();
```

Measurement remains separate from deterministic circuit execution.

## State-Vector Representation

An `n`-qubit state is stored as a complex vector containing `2^n` amplitudes.

For two qubits:

```text
index 0 → |00>
index 1 → |01>
index 2 → |10>
index 3 → |11>
```

The simulator provides helpers for:

* Dirac notation formatting
* probability calculation
* normalization checks
* state comparison with numerical tolerances
* shot-based measurement

## Qubit Ordering

Eigenon uses little-endian qubit indexing:

```text
q0 = least significant bit
```

Printed basis states use standard binary notation, so `q0` appears as the rightmost bit.

```text
X(q0) |0000> = |0001>
```

Controlled gates use:

```text
cnot(control, target)
cz(control, target)
mcx(controls, target)
mcz(controls, target)
```

For generic local gates:

```text
targets[0]     → most significant local matrix bit
targets[k - 1] → least significant local matrix bit
```

The same convention is used by the Circuit API, CLI, dense baseline, and matrix-free execution engine.

## Matrix-Free Execution

The primary execution path updates the state vector directly. Single-qubit gates operate on amplitude pairs that differ in the selected qubit bit.

Generic `k`-qubit gates operate on local blocks of `2^k` amplitudes without constructing a full-system matrix. State-vector memory complexity remains:

```text
O(2^n)
```

Temporary local storage depends on the gate width:

```text
O(2^k)
```

## Dense Baseline

Eigenon also keeps a dense full-operator implementation for correctness verification. The dense path is not intended for scalable execution. It is used as an independent baseline for comparing matrix-free results.

## Supported Operations

Single-qubit gates:

```text
I, X, Y, Z, H, S, T
```

Two-qubit gates:

```text
CNOT, CZ
```

The generic execution engine can also apply arbitrary local `2^k × 2^k` gate matrices.

## Algorithms

Current algorithm demonstrations include:

* Deutsch
* Deutsch–Jozsa
* Grover search

Deutsch–Jozsa and the gate-level Grover implementation have been compared
against Qiskit reference circuits.

## Interoperability

The circuit operation representation is shared by execution and export.
Eigenon currently exports supported circuits as OpenQASM 2.0 or OpenQASM 3.0.
External Qiskit scripts validate selected algorithms and exported circuits.

OpenQASM import, parameterized gates and additional framework adapters remain
future work.

## CLI Gate Representation

CLI gate strings are parsed into typed `GateSpec` values:

```text
h:0
x:1
cnot:0,1
cz:1,2
```

Responsibilities are separated:

```text
FromStr     → parse syntax
validate    → check circuit compatibility
apply       → add the operation to Circuit
```

## Project Structure

- `benches/`: Performance benchmarks.
- `src/algorithms/`: Algorithm-level demos and helpers, including Deutsch, Deutsch-Jozsa and Grover search.
- `src/circuit/`: High-level circuit abstraction, operation representation and reusable circuit catalog.
- `src/cli/`: Command definitions and typed gate specification parsing.
- `src/engine/`: Matrix-free execution, gate constants, measurement and state-vector utilities.
- `src/experiments/`: Modular educational and verification experiments.
    - `circuit_demos/`: User-facing demos built with the Circuit API.
    - `engine_verification/`: Low-level verification experiments for in-place execution.
    - `foundations/`: Educational examples for tensor products and basic concepts.
- `src/export/`: OpenQASM 2.0 and OpenQASM 3.0 exporters.
- `src/lib.rs`: Quantum simulator library.
- `src/main.rs`: CLI entry point for circuits, demos, verification and export.
- `tests/`: Integration tests.
- `tests/snapshots/`: Stable OpenQASM output snapshots.
- `validation/`: External validation scripts and notes for comparing simulator behavior against reference frameworks such as Qiskit.

The current design intentionally keeps the circuit representation simple. More
general intermediate representations, optimization passes and additional
backends should be introduced only when a concrete use case requires them.
