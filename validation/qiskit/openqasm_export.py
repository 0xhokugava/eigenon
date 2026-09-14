import sys

from qiskit import qasm2
from qiskit.quantum_info import Statevector


EPS = 1e-9
DISPLAY_EPS = 1e-12


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit(
            "Usage: python3 validation/qiskit/openqasm_export.py <path-to-qasm>"
        )

    qasm_path = sys.argv[1]

    qc = qasm2.load(qasm_path)
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
    print(f"Qubits: {qc.num_qubits}")
    print()
    print(qc)
    print()
    print(state)
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