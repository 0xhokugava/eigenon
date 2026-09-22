# Roadmap

## Completed

* [x] Complex state-vector representation
* [x] Tensor products and dense matrix baseline
* [x] Matrix-free single-qubit gate execution
* [x] Matrix-free CNOT execution
* [x] Generic in-place `k`-qubit gate application
* [x] Circuit API
* [x] X, Y, Z, H, S, T, I, CNOT and CZ gates
* [x] Dirac notation output
* [x] Shot-based measurement
* [x] Deutsch algorithm
* [x] Deutsch–Jozsa algorithm
* [x] Grover search
* [x] Qiskit validation for Deutsch–Jozsa
* [x] Criterion benchmarks
* [x] `eigenon` command-line interface
* [x] Custom circuit execution through CLI
* [x] MCX and MCZ gates through Circuit API
* [x] MCX and MCZ gate syntax through CLI
* [x] Gate-level Grover oracle built from primitive gates
* [x] Gate-level Grover diffusion built from primitive gates
* [x] Preserved existing Grover result after gate-level decomposition
* [x] Qiskit validation for gate-level Grover
* [x] Source files organized into circuit, CLI, engine and export modules
* [x] Internal circuit operation representation
* [x] OpenQASM 2.0 export
* [x] OpenQASM 3.0 export
* [x] Classical register and measurement export
* [x] OpenQASM snapshot tests

## Current Focus

* [ ] keep OpenQASM 2.0 and OpenQASM 3.0 output stable
* [ ] improve contributor documentation and runnable examples
* [ ] improve validation and circuit reports
* [ ] keep qubit-ordering conventions explicit across APIs and exporters

## Next

* [ ] add basic OpenQASM import
* [ ] add circuit operation statistics
* [ ] add JSON and Markdown CLI reports
* [ ] run selected Qiskit validation in CI
* [ ] add parameterized rotation gates

## Later

* [ ] Qiskit adapter
* [ ] Braket or PennyLane adapter
* [ ] circuit-gradient experiments
* [ ] memory benchmarks by qubit count
* [ ] parallel matrix-free execution experiments

## Current Development Order

1. Stabilize OpenQASM export and its validation
2. Improve contributor onboarding and runnable examples
3. Add circuit statistics and validation reports
4. Add basic OpenQASM import
5. Add parameterized rotation gates
6. Plan additional interoperability adapters
