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

## OpenQASM export validation

Eigenon can export circuits to OpenQASM 2.0 or 3.0 and validate the generated
file with Qiskit.

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
* one-control MCX and MCZ as `cx` and `cz`
* two-control MCX as `ccx`
* two-control MCZ decomposed as H/CCX/H
* classical registers
* measurement

OpenQASM 2.0 MCX and MCZ operations with more than two controls remain
unsupported because they require an explicit ancilla and decomposition policy.
OpenQASM 3.0 exports MCX and MCZ with any valid number of controls using the
standard `ctrl(n) @` modifier.

Measured circuits are parsed by Qiskit with their classical register and
qubit-to-classical-bit mapping preserved.

The ignored Rust integration suite also imports an external OpenQASM 2.0
program into Eigenon, exports the resulting circuit again, and asks Qiskit to
validate both the original and round-tripped forms.

For statevector validation, final measurement operations are removed from a
copy of the parsed circuit before calling `Statevector.from_instruction()`.

## Qiskit integration tests

Qiskit validation tests are marked as ignored by the default Rust test suite
because they require an external Python environment. CI installs the pinned
dependency from `validation/qiskit/requirements.txt` and runs these tests
explicitly.

Create a local virtual environment, install the same dependency as CI and run
the tests explicitly with:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install --requirement validation/qiskit/requirements.txt
cargo test qiskit_ -- --ignored
```

## Not supported yet

The current OpenQASM exporter does not support:

* custom gates

Unsupported operations return an explicit export error instead of generating
invalid OpenQASM.
