import json

from qiskit import QuantumCircuit
from qiskit.circuit import Parameter
from qiskit.quantum_info import SparsePauliOp, Statevector


theta = Parameter("theta")
phi = Parameter("phi")
circuit = QuantumCircuit(2)
circuit.h(0)
circuit.ry(theta, 0)
circuit.rx(phi, 1)
circuit.ry(theta, 1)
circuit.cx(0, 1)

hamiltonian = SparsePauliOp.from_list(
    [
        ("II", 0.3),
        ("ZZ", -0.8),
        ("XI", 0.4),
    ]
)
bindings = {theta: 0.29, phi: -0.43}


def energy(values):
    bound = circuit.assign_parameters(values)
    state = Statevector.from_instruction(bound)
    return float(state.expectation_value(hamiltonian).real)


epsilon = 1e-7
gradients = {}
for parameter in (theta, phi):
    plus = dict(bindings)
    minus = dict(bindings)
    plus[parameter] += epsilon
    minus[parameter] -= epsilon
    gradients[parameter.name] = (energy(plus) - energy(minus)) / (2.0 * epsilon)

print(json.dumps(gradients, sort_keys=True))
