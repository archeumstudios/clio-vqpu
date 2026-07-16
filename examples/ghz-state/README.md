# GHZ-state workload family

The 3-, 4-, and 5-qubit workloads allocate logical qubits contiguously, apply H to `q0`, fan controlled-X from `q0`, and measure every qubit. Under Clio's basis convention, only the all-zero and all-one outcomes are valid.

```bash
cargo run -p clio-cli -- run examples/ghz-state/4-qubits/main.clio
cargo run -p clio-cli -- validate examples/ghz-state/4-qubits/main.clio
```

These examples test multi-qubit indexing and collapse beyond Bell state. They do not imply efficient large-state simulation; memory remains exponential.
