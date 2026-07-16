# Architecture and Instruction Set

Clio uses a source-to-machine pipeline: text is parsed into directive and instruction records, labels are resolved during assembly, semantic allocation state is checked, a conservative execution plan is admitted, and the runtime performs fetch/decode/execute transitions. The typed executable retains source spans, metadata, the required qubit count, branch properties, and a source SHA-256 identity.

Architectural state includes a program counter, lifecycle status, sixteen signed 64-bit classical registers, sixteen three-state measurement registers, allocated logical qubits, comparison state, instruction counter, shot index, and total shots. The quantum state belongs to the selected backend but is addressed by typed logical qubit identifiers. Qubit zero is the least-significant state-vector bit.

The ISA includes lifecycle operations, standard single- and multi-qubit gates, arbitrary-axis rotations, controlled phase, measurement, checked classical arithmetic and logic, explicit comparison, conditional and unconditional branches, and `HALT`. Observation reports are debugger services rather than instructions, preventing state inspection from changing program semantics.

Allocation is contiguous and release is deliberately restrictive. `QFREE` may release only the highest logical qubit and only when its probability of one is within numerical zero tolerance. This permits safe vector truncation without an implicit partial trace. Reset samples and collapses when necessary, applies X after outcome one, and records the reset outcome.

Classical arithmetic traps on overflow, zero divisors, the signed minimum divided by minus one, and invalid shifts. Branches require a valid comparison where applicable. All branch targets are resolved before execution. Backward branches are permitted but admitted against the host's entire instruction ceiling and remain subject to wall-clock limits.
