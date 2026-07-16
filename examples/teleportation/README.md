# Quantum teleportation

These workloads teleport three analytically verifiable input states from `q0` to `q2`: `|0>`, `|1>`, and `|+>`. They create an entangled `q1`–`q2` pair, perform the sender operations and measurements, then apply measurement-driven Z and X corrections to `q2`.

The plus workload applies H before the final receiver measurement, converting the expected `|+>` state to deterministic `|0>` for known-answer validation. The sender measurements remain probabilistic and should cover all correction combinations over many shots.

```bash
cargo run -p clio-cli -- run examples/teleportation/one/main.clio
cargo run -p clio-cli -- validate examples/teleportation/plus/main.clio
```

This is state teleportation within a classical state-vector simulation. It does not transmit physical quantum information.
