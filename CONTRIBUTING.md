# Contributing

Eigenon is a quantum circuit simulation and validation toolkit.

Contributions are welcome, especially in the areas of:

* tests and edge cases
* documentation
* CLI examples
* circuit validation
* OpenQASM export/import
* small refactors that preserve behavior

## Prerequisites

You need:

* Git
* a stable Rust toolchain with Rust 2024 edition support
* Python 3 and Qiskit only when running the optional external validation suite

Rust 1.85 or newer supports the Rust 2024 edition. CI follows the latest stable
toolchain. The project does not currently guarantee a separate minimum
supported Rust version.

## Development setup

```bash
git clone https://github.com/0xhokugava/eigenon.git
cd eigenon
cargo fmt -- --check
cargo test
```

Run the CLI directly from the working tree:

```bash
cargo run -- run --qubits 2 --gate h:0 --gate cnot:0,1
```

## Choosing an issue

Before starting work:

* check whether the issue is already assigned or has a linked pull request
* comment on the issue when you intend to work on it
* ask for agreement before starting an architectural change or a large feature
* keep the proposed change focused on one problem

Documentation, focused tests, examples, validation cases and small cleanup
tasks are usually the best starting points.

## Branch and pull request workflow

1. Fork the repository if you do not have write access.
2. Create a branch from the latest `master` branch.
3. Make one focused change.
4. Add or update tests when behavior changes.
5. Run the required checks.
6. Open a pull request that explains the problem, the solution and the checks
   you ran.

Example branch names:

```text
docs/update-architecture
test/non-adjacent-cnot
fix/openqasm-ordering-comment
```

## Required checks

Run these commands from the repository root:

```bash
cargo fmt -- --check
cargo test
```

The Qiskit integration tests are optional because they require an external
Python environment:

```bash
cargo test qiskit_ -- --ignored
```

See [validation/qiskit/README.md](validation/qiskit/README.md) for the
standalone validation commands and the scripts they execute.

## Before opening a pull request

Please make sure:

* `cargo fmt -- --check` passes
* `cargo test` passes
* the change is focused and does not mix unrelated refactors
* behavior changes are covered by tests when possible
* documentation is updated when public behavior or commands change
* the pull request links its issue when one exists

## Project conventions

* qubit `0` is the least significant bit
* printed basis states use standard binary order
* controlled gates use `control, target` order
* validation scripts live outside the simulator runtime
* public behavior should remain consistent across the Circuit API, CLI and
  OpenQASM export

## Good first contributions

Good starting points are usually:

* documentation improvements
* additional tests
* CLI examples
* validation cases
* small cleanup tasks

For larger changes, please open an issue first and wait for agreement on the
scope.
