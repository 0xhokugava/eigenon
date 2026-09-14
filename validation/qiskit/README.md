# Qiskit validation

This directory contains external Qiskit validation scripts for the Rust quantum simulator.

The code here is not part of the simulator runtime. It is used only as a reference check for algorithm-level behavior.

## Deutsch-Jozsa

`deutsch_jozsa.py` validates the same oracle cases as the Rust implementation:

- constant zero
- constant one
- balanced selected bit
- balanced parity

Expected behavior:

- constant oracle -> query register returns `|00...0>`
- balanced oracle -> query register does not return `|00...0>`

## Grover search

`grover.py` validates the gate-level Grover implementation against a Qiskit reference circuit.

Validated scenario:

* qubits: `3`
* target state: `|101>`
* target index: `5`
* Grover steps: `2`
* expected target probability: `0.9453125`

Eigenon result:

```text
Target state: |101> index=5
Target probability: 0.945313
Grover steps: 2
```

Qiskit result:

```text
Qiskit Grover validation

Target state: |101> index=5
Target amplitude: 0.972272 + 0.000000i
Target probability: 0.945312
Grover steps: 2

Validation passed.
```

The small display difference between `0.945313` and `0.945312` is only formatting precision. The exact expected value is `0.9453125`.

## Bit ordering

The Rust simulator uses little-endian qubit indexing: qubit `0` maps to the least significant bit.

Dirac output is printed in standard binary order, so qubit `0` appears as the rightmost bit.

## Run

```bash
python3 validation/qiskit/deutsch_jozsa.py
python3 validation/qiskit/grover.py
```

## OpenQASM 2.0 export validation

Eigenon can export a circuit to OpenQASM 2.0 and validate the exported file with Qiskit.

Generate a Bell circuit as OpenQASM:

```bash
mkdir -p tmp
cargo run --quiet -- export-openqasm --qubits 2 --gate h:0 --gate cnot:0,1 > tmp/bell.qasm
```

Validate the exported circuit with Qiskit:

```bash
python3 validation/qiskit/openqasm_export.py tmp/bell.qasm
```

## Supported OpenQASM 2.0 export subset

The current exporter supports:

* `x`
* `y`
* `z`
* `h`
* `s`
* `t`
* `cx`
* `cz`
* classical registers
* measurement

Measured circuits are parsed by Qiskit with their classical register and
qubit-to-classical-bit mapping preserved.

For statevector validation, final measurement operations are removed from a
copy of the parsed circuit before calling `Statevector.from_instruction()`.

## Qiskit integration tests

Qiskit validation tests are marked as ignored because they require an external
Python environment with Qiskit installed.

Run them explicitly with:

```bash
cargo test qiskit_ -- --ignored
```

## Not supported yet

The current OpenQASM exporter does not support:

* measurement / classical registers
* `mcx`
* `mcz`
* custom gates
* OpenQASM import
* OpenQASM 3.0

Unsupported operations return an explicit export error instead of generating invalid OpenQASM.