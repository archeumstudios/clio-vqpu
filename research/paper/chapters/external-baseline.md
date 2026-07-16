# External Simulator Baseline

The external baseline uses IBM Qiskit 2.5.0 `quantum_info.Statevector`. Circuits are constructed independently with Qiskit operations and compared with Clio final state snapshots. Both systems treat qubit zero as the least-significant computational-basis bit. A unit global phase is aligned before amplitude error is calculated.

Six families are currently compared: Bell, three-qubit GHZ, two-qubit Grover, QFT/inverse-QFT recovery, Bernstein–Vazirani secret `1011`, and balanced-parity Deutsch–Jozsa. Reports retain maximum complex-amplitude error, total-variation distance, Jensen–Shannon divergence, and normalization error. Every retained case passes the frozen `1e-12` amplitude and probability thresholds; observed errors are at floating-point roundoff scale.

Teleportation is validated through Clio's dynamic measurement-and-feed-forward known-answer test rather than the pure-state external adapter. Extending the external baseline to matched dynamic-circuit semantics remains separate work and is not implied by the current exact-state comparison.

The adapter follows the official [Qiskit Statevector API](https://quantum.cloud.ibm.com/docs/en/api/qiskit/qiskit.quantum_info.Statevector), including its computational-basis probability and subsystem-order conventions.
