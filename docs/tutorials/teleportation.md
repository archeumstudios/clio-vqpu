# Quantum Teleportation

Clio's teleportation workloads verify hybrid quantum–classical execution across three analytically known input states: `|0>`, `|1>`, and `|+>`.

The program prepares the input on `q0`, creates a Bell pair between `q1` and `q2`, performs sender-side gates and measurements, moves those results into classical registers, and conditionally applies Z and X corrections to `q2`. The receiver is then measured in the appropriate verification basis.

```bash
cargo run -p clio-cli -- validate examples/teleportation/zero/main.clio
cargo run -p clio-cli -- validate examples/teleportation/one/main.clio
cargo run -p clio-cli -- validate examples/teleportation/plus/main.clio
```

Each canonical workload requires all 1,024 receiver checks to match the expected bit and all four sender measurement combinations to occur. The plus workload applies H at the receiver before measurement, converting expected `|+>` into deterministic `|0>` for known-answer verification.

This demonstrates state transfer inside Clio's simulated processor state. It is not physical communication or physical quantum teleportation.
