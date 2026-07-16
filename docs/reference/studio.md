# Clio Studio

Run `cargo run --release -p clio-studio` and open `http://127.0.0.1:4317`. Studio binds to loopback by default, serves no third-party resources, sends no telemetry, and executes programs through Clio SDK.

The editor loads canonical examples and preserves the working source in browser-local storage. Build checks syntax and semantics. Run performs the declared shot execution, then performs a separate single-shot bounded inspection so timeline scrubbing cannot alter the primary result. Validate selects the canonical known-answer validator from program metadata.

The processor inspector reports architectural status, program counter, backend, seed, shot, virtual qubits, classical registers, measurement registers, and the admitted resource plan. The quantum-state panel displays complete amplitudes only for the runtime's bounded small-state trace policy. It does not use isolated Bloch spheres for entangled states.

Replay export creates a checksummed package through `clio-replay`. Import verifies schema, checksum, engine identity, RNG identity, executable equality, and exact deterministic reproduction before loading its source.

Studio is a local processor laboratory, not a hosted multi-user service. Binding it to a non-loopback address requires an external authenticated reverse proxy and an explicit deployment security review.
