# Engine and Runtime Implementation

Clio Engine stores `2^n` double-precision complex amplitudes. Single-qubit gates traverse pairs whose indices differ in the target bit. Controlled gates inspect control and target masks; swap exchanges only indices with unequal selected bits. Every operation validates allocation and operand distinctness and checks normalization after mutation.

Measurement computes the sum of squared magnitudes for indices whose target bit is one. The runtime derives a shot-specific ChaCha8 stream from the declared seed and shot index, samples the outcome, zeros the rejected subspace, and renormalizes the selected amplitudes. Deterministic zero- and one-probability outcomes do not consume an ambiguous branch of randomness.

Runtime execution creates fresh processor and engine state for every shot. It increments logical and architectural instruction counters, enforces global instruction and wall-time limits, records mutations, and requires explicit `HALT`. Measurement strings are emitted in highest-written-register-first order. The final architectural state belongs to the last shot, while aggregate counts cover all shots.

The observation subsystem computes basis probabilities, per-qubit marginals, tensor-product Pauli expectations, squared norm, and single-qubit reduced density matrices. Studio inspection forces one bounded state-enabled shot. This inspection result is intentionally separate from the declared multi-shot result.

Trace records are schema-identified instruction transitions. For up to eight qubits, state-enabled traces retain every basis amplitude after each instruction. Larger states omit complete snapshots. This bound prevents the debugger from silently multiplying trace size by an unbounded state dimension.
