# What Clio Is

Clio VQPU is a programmable virtual processor architecture for gate-model and hybrid classical–quantum programs. It is designed as a coherent machine: Clio Assembly is parsed and validated into Clio ISA instructions, Clio Runtime executes those instructions against explicit processor state, and Clio Engine supplies an independent built-in quantum-state implementation.

The processor model includes a program counter, statuses, classical and measurement registers, typed logical qubits, flags, budgets, instruction and shot counters, diagnostics, and trace configuration. Measurement can enter architectural state and influence later classical branches. This makes instruction lifecycle and hybrid control inspectable rather than hiding them behind a circuit-only call.

Clio Trace records bounded structured execution events. Resource admission estimates state-vector and runtime needs before dangerous allocations. Conformance, known-answer, property, statistical, fuzz, and differential validation provide evidence for correctness. The CLI, Rust and Python SDKs, and Clio Studio are interfaces to the same Rust implementation.

The project combines a usable open-source product with a reproducible research artifact. Its final claims will be constrained by implementation, primary-source literature, retained experimental data, and documented limitations.
