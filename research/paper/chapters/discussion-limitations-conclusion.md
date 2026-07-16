# Discussion

Clio demonstrates that processor abstractions can make hybrid quantum simulation more explicit. Program counters, typed registers, traps, admission, and traces provide stable locations for debugging and research evidence. The cost is additional parsing, planning, state-machine transitions, and trace storage compared with direct engine calls. Retained benchmarks quantify those costs for the tested machine rather than assuming they are negligible.

The restrictive qubit lifecycle trades flexibility for transparent correctness. A density-matrix backend or explicit partial-trace operation could safely support arbitrary release, but silently discarding entanglement would violate the current state contract. Likewise, debugger observations remain host operations to avoid conflating scientific measurement with architectural execution.

# Limitations

Clio is a classical, exponentially scaling state-vector simulator. It is not a physical QPU and does not provide quantum speedup. The ISA is finite, the external baseline covers pure-state subsets rather than matched dynamic teleportation, and evaluation occurs primarily on one macOS arm64 host. Floating-point tolerance, compiler behavior, process scheduling, and trace serialization affect results.

The ten-repetition dataset was initially collected before the repository's baseline commit and is therefore candidate evidence until recollected against the committed hash. Peak RSS is platform-native and not equivalent to heap allocation. The fixed trace-event estimate is conservative for evaluated workloads but requires adversarial monitoring as schemas evolve. Replay packages provide integrity, not producer authenticity or signatures.

Studio is local-only and unauthenticated. It must not be exposed directly on a network. The product does not currently include Python bindings, physical quantum hardware adapters, call/return instructions, arbitrary qubit release, or distributed execution. These absences define the released architecture rather than promises for another product version.

# Responsible Claims

Clio is described as a programmable virtual quantum processing architecture implemented in software. “Quantum” refers to simulated gate-model state and algorithms. Results establish agreement with analytic expectations and Qiskit for evaluated circuits; they do not establish physical quantum behavior, advantage, supremacy, or hardware equivalence. Performance claims identify compiler profile, host, repetition count, and workload.

# Conclusion

Clio integrates a custom assembly language and ISA, explicit processor state, an independent quantum engine, a hybrid runtime, bounded tracing, deterministic replay, resource admission, validation, developer tools, a visual processor laboratory, and reproducible research evidence. The implementation executes representative algorithms and dynamic classical control, matches an established state-vector baseline at floating-point precision for six pure-state families, and exposes machine state without replacing the owned runtime. The project supports the bounded conclusion that a coherent, inspectable, and resource-aware VQPU can be designed and implemented as a complete classical software artifact.

# References

1. M. A. Nielsen and I. L. Chuang, *Quantum Computation and Quantum Information*, Cambridge University Press.
2. IBM Quantum, “Qiskit Statevector API,” https://quantum.cloud.ibm.com/docs/en/api/qiskit/qiskit.quantum_info.Statevector.
3. P. Virtanen et al., “SciPy 1.0: Fundamental Algorithms for Scientific Computing in Python,” *Nature Methods*, 2020.
4. D. P. DiVincenzo, “The Physical Implementation of Quantum Computation,” *Fortschritte der Physik*, 2000.
5. Clio VQPU repository evidence, specifications, ADRs, source, and retained benchmark manifests included with this artifact.
