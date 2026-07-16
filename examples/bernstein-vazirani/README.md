# Bernstein–Vazirani

The checked-in workloads recover hidden strings `1011` and `00101` exactly. String display follows Clio's `q(n-1)…q0` convention; oracle controlled-X gates are selected from the corresponding low-to-high logical bits.

```bash
cargo run -p clio-cli -- validate examples/bernstein-vazirani/secret-1011/main.clio
```
