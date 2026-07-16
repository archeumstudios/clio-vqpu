# What Clio Is Not

Clio is not a physical quantum processor or physical quantum computer. Its built-in engine uses classical computation to simulate quantum-state evolution. General state-vector memory grows exponentially: `n` qubits require `2^n` complex amplitudes, approximately `16 × 2^n` raw bytes at double precision before overhead.

Clio does not turn a classical laptop into quantum hardware, demonstrate quantum advantage or supremacy, provide unlimited qubits, replace physical QPUs, or claim to be the first VQPU. It is not promised to be faster than mature simulation systems. Floating-point behavior, available memory, trace volume, and replay compatibility impose real limits.

Clio is also not a renamed wrapper around Qiskit, Cirq, PennyLane, or another simulator. Optional adapters can support interoperability and differential validation, but the processor architecture and built-in engine remain independently functional. It is not a cloud platform, physical control stack, distributed quantum network, fault-tolerant compiler, or arbitrary host-code environment.

Clio's value is an explicit, inspectable, resource-aware processor model and integrated research platform—not a claim that classical simulation is physical quantum computation.
