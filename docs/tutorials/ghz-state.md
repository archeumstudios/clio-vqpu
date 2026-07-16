# GHZ States

Clio includes verified 3-, 4-, and 5-qubit GHZ workloads under `examples/ghz-state/`. Each prepares

`(|00…0> + |11…1>) / sqrt(2)`

by applying H to `q0` and controlled-X from `q0` to every other qubit. Every qubit is then measured. Correct executions contain only the all-zero and all-one bit strings.

```bash
cargo run -p clio-cli -- run examples/ghz-state/5-qubits/main.clio
cargo run -p clio-cli -- validate examples/ghz-state/5-qubits/main.clio
```

State inspection is intentionally bounded to eight qubits:

```bash
cargo run -p clio-cli -- inspect examples/ghz-state/4-qubits/main.clio
```

Inspection runs one seeded shot with state snapshots and reports complex amplitudes and probabilities after real instructions. It is not enabled by default because snapshots scale exponentially.
