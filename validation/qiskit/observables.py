import json

from qiskit import QuantumCircuit
from qiskit.quantum_info import SparsePauliOp, Statevector


circuit = QuantumCircuit(3)
circuit.h(0)
circuit.ry(0.37, 1)
circuit.cx(0, 2)
state = Statevector.from_instruction(circuit)

operators = {
    "z0": SparsePauliOp.from_list([("IIZ", 1.0)]),
    "y1": SparsePauliOp.from_list([("IYI", 1.0)]),
    "x0_x2": SparsePauliOp.from_list([("XIX", 1.0)]),
    "hamiltonian": SparsePauliOp.from_list(
        [
            ("III", 0.3),
            ("IIZ", -1.2),
            ("IYI", -0.4),
            ("XIX", 0.7),
        ]
    ),
}

print(
    json.dumps(
        {
            name: float(state.expectation_value(operator).real)
            for name, operator in operators.items()
        },
        sort_keys=True,
    )
)
