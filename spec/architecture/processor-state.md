# Processor state

The architectural state contains the program counter, lifecycle status, sixteen signed classical registers, sixteen three-state measurement registers, allocated virtual-qubit identifiers, the latest signed comparison state, per-shot instruction counter, shot index, and total shots. Quantum amplitudes are backend state addressed through allocated logical qubits.

Status begins `Ready`, enters `Running`, and terminates as `Halted`, `Completed`, or `Trapped`. Validation and resource failures occur before active execution and are represented by `ValidationFailed` and `ResourceRejected` at host boundaries. `Paused` is reserved for an external debugger controller; the current batch runtime does not synthesize pauses.

Each instruction transition records status before and after, source span, allocation/release, measurement or classical mutation, branch decision, comparison state, and—when enabled and bounded—a complete small-state snapshot. The runtime increments the program counter before ordinary execution and replaces it with a resolved branch target when a branch is taken.
