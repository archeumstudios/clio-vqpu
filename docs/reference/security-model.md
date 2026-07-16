# Security model

Clio accepts untrusted assembly text but does not promise safe execution without host limits. Admission bounds active qubits, shots, estimated memory, instructions, trace bytes, and wall-clock execution. Checked classical arithmetic traps on overflow, division by zero, invalid remainder, and invalid shifts. State allocation uses checked arithmetic and Rust safe code; every workspace crate forbids `unsafe`.

Backward branches are admitted against the complete host instruction ceiling. Trace estimates use a conservative per-event bound validated against serialized bounded-state traces. Runtime instruction and time budgets remain active after admission.

Clio Studio binds to loopback, has no remote asset dependencies or telemetry, and applies a restrictive content-security policy. It has no authentication and must not be exposed directly to a network. Replay packages are integrity checked but not cryptographically signed; SHA-256 detects mutation, not a malicious producer.

The Qiskit adapter is an optional research dependency and is never imported by the built-in engine, runtime, CLI, SDK, or Studio. External baseline environments should remain isolated.
