import sys
from pathlib import Path

from qiskit import qasm2, qasm3
from qiskit.quantum_info import Statevector


EPS = 1e-9
DISPLAY_EPS = 1e-12


def load_circuit(qasm_path: str):
    source = Path(qasm_path).read_text(encoding="utf-8")
    header = source.lstrip()

    if header.startswith("OPENQASM 2.0;"):
        return "2.0", qasm2.loads(source)
    if header.startswith("OPENQASM 3.0;"):
        return "3.0", qasm3.loads(source)

    raise ValueError("Expected an OpenQASM 2.0 or 3.0 version declaration")


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit(
            "Usage: python3 validation/qiskit/openqasm_export.py <path-to-qasm>"
        )

    qasm_path = sys.argv[1]

    qasm_version, qc = load_circuit(qasm_path)
    state_circuit = qc.remove_final_measurements(inplace=False)
    state = Statevector.from_instruction(state_circuit)
    probabilities = state.probabilities_dict()

    measurements = []

    for instruction in qc.data:
        if instruction.operation.name != "measure":
            continue

        qubit = qc.find_bit(instruction.qubits[0]).index
        classical_bit = qc.find_bit(instruction.clbits[0]).index

        measurements.append((qubit, classical_bit))

    total_probability = sum(probabilities.values())
    assert abs(total_probability - 1.0) < EPS, (
        f"Expected probabilities to sum to 1.0, got {total_probability}"
    )

    print("OpenQASM export validation")
    print()
    print(f"QASM file: {qasm_path}")
    print(f"OpenQASM version: {qasm_version}")
    print(f"Qubits: {qc.num_qubits}")
    print()
    print(qc)
    print()
    print(state)
    print()
    print("Non-zero amplitudes:")

    for basis, amplitude in sorted(state.to_dict().items()):
        if abs(amplitude) > DISPLAY_EPS:
            print(
                f"  A({basis}) = "
                f"{amplitude.real:.12f}{amplitude.imag:+.12f}i"
            )

    print()
    print("Non-zero probabilities:")

    for basis, probability in sorted(probabilities.items()):
        if probability > DISPLAY_EPS:
            print(f"  P({basis}) = {probability}")

    if measurements:
        print()
    print(f"Classical bits: {qc.num_clbits}")
    print("Measurements:")

    for qubit, classical_bit in measurements:
        print(f"  q[{qubit}] -> c[{classical_bit}]")

    print()
    print("Validation passed.")


if __name__ == "__main__":
    main()
